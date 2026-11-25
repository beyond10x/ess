//! `ess specify vendor` (beyond10x/ess#663): writes a consumer's copy of an owner's type and the
//! composition entries that prove it, or with `--check` says which copy the owner has moved away
//! from (`docs/design/composition-type-conformance.md`, "Generating the copy").

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use anyhow::{Context, Result};
use ess_domain::name::QualifiedName;

use crate::{load, ServiceInput};

/// What `ess specify vendor` was given.
pub(crate) struct Request {
    pub(crate) from: String,
    pub(crate) service: String,
    pub(crate) component: Option<String>,
    pub(crate) type_name: String,
    pub(crate) into: String,
    pub(crate) out: PathBuf,
    pub(crate) conformances: PathBuf,
    pub(crate) check: bool,
}

/// A refusal: the sentence on standard error and exit 1.
fn refuse(reason: &str) -> ExitCode {
    eprintln!("error: {reason}");
    ExitCode::from(1)
}

fn compiled(loaded: load::LoadedSpec, path: &Path) -> Result<Box<ess_compiler::EssIr>, ExitCode> {
    match loaded {
        load::LoadedSpec::Compiled { ir, .. } => Ok(ir),
        load::LoadedSpec::Refused {
            problems,
            diagnostics,
            ..
        } => {
            eprintln!("{} was refused:", path.display());
            for problem in problems {
                eprintln!("  - {problem}");
            }
            for diagnostic in diagnostics.as_slice() {
                eprintln!("  - {diagnostic}");
            }
            Err(ExitCode::from(1))
        }
    }
}

/// The arguments, read: the two services, the type and the domain.
struct Parsed {
    from: ServiceInput,
    service: ServiceInput,
    type_name: QualifiedName,
    into: QualifiedName,
}

/// Reads the arguments the parser leaves as text, refusing each by name.
fn parse(request: &Request) -> Result<Parsed, String> {
    let from = request
        .from
        .parse::<ServiceInput>()
        .map_err(|reason| format!("--from `{}`: {reason}", request.from))?;
    let service = request
        .service
        .parse::<ServiceInput>()
        .map_err(|reason| format!("--service `{}`: {reason}", request.service))?;
    if from.key == service.key {
        return Err(format!(
            "--from and --service both name the service key `{}`: the owner and the consumer \
             are two services of one composition",
            from.key
        ));
    }
    let type_name = QualifiedName::new(&request.type_name).map_err(|reason| {
        format!(
            "--type `{}` is not a qualified type name: {reason}",
            request.type_name
        )
    })?;
    let into = QualifiedName::new(&request.into).map_err(|reason| {
        format!(
            "--into `{}` is not a qualified domain name: {reason}",
            request.into
        )
    })?;
    Ok(Parsed {
        from,
        service,
        type_name,
        into,
    })
}

/// The consumer's component: `--component` when the consumer declares it, else the one whose
/// `owns` names `into`.
fn consumer_component(
    consumer: &ess_compiler::EssIr,
    component: Option<&str>,
    into: &QualifiedName,
) -> Result<String, String> {
    if let Some(component) = component {
        if consumer
            .components()
            .keys()
            .any(|name| name.to_string() == component)
        {
            return Ok(component.to_owned());
        }
        return Err(format!(
            "--component `{component}` is not a component `{}` declares",
            consumer.system()
        ));
    }
    consumer
        .components()
        .values()
        .find(|candidate| candidate.owns.iter().any(|domain| domain.name() == into))
        .map(|owning| owning.name.to_string())
        .ok_or_else(|| {
            format!(
                "no component of `{}` owns `{into}`: name the consumer's component with \
                 --component",
                consumer.system()
            )
        })
}

pub(crate) fn run(request: &Request) -> Result<ExitCode> {
    let Parsed {
        from,
        service,
        type_name,
        into,
    } = match parse(request) {
        Ok(parsed) => parsed,
        Err(reason) => return Ok(refuse(&reason)),
    };
    let owner = match compiled(load::specification(&from.path)?, &from.path) {
        Ok(owner) => owner,
        Err(code) => return Ok(code),
    };
    let vendored = match ess_composition::vendor(&owner, &type_name, &into) {
        Ok(vendored) => vendored,
        Err(reason) => return Ok(refuse(&reason)),
    };

    // The consumer as it stands, without the file this writes: the copy a second run reads there
    // is the one it replaces.
    let nothing = BTreeSet::new();
    let base = match compiled(
        load::specification_replacing(&service.path, &request.out, None, &nothing)?,
        &service.path,
    ) {
        Ok(base) => base,
        Err(code) => return Ok(code),
    };
    let component = match consumer_component(&base, request.component.as_deref(), &into) {
        Ok(component) => component,
        Err(reason) => return Ok(refuse(&reason)),
    };
    let collisions = vendored.collisions(&base);
    if !collisions.is_empty() {
        let names = collisions
            .iter()
            .map(|name| format!("`{name}`"))
            .collect::<Vec<_>>()
            .join(", ");
        return Ok(refuse(&format!(
            "`{}` already declares {names} under another shape than the copy of `{type_name}`",
            base.system()
        )));
    }

    let source = vendored.source();
    let copies = vendored.blocks().keys().cloned().collect::<BTreeSet<_>>();
    let consumer = match compiled(
        load::specification_replacing(&service.path, &request.out, Some(&source), &copies)?,
        &service.path,
    ) {
        Ok(consumer) => consumer,
        Err(code) => return Ok(code),
    };
    let fragment = vendored.conformances(&from.key, &service.key, &consumer, &component);

    if request.check {
        return Ok(check(request, &vendored, &source, &fragment));
    }
    fs::write(&request.out, &source)
        .with_context(|| format!("writing {}", request.out.display()))?;
    fs::write(&request.conformances, &fragment)
        .with_context(|| format!("writing {}", request.conformances.display()))?;
    println!(
        "{} type(s) of {} {} copied into {into}: {} and {}",
        copies.len(),
        owner.system(),
        owner.version(),
        request.out.display(),
        request.conformances.display()
    );
    Ok(ExitCode::SUCCESS)
}

/// Compares the files with what would be written, naming each copy that differs.
fn check(
    request: &Request,
    vendored: &ess_composition::Vendored,
    source: &str,
    fragment: &str,
) -> ExitCode {
    let mut differences = Vec::new();
    match fs::read_to_string(&request.out) {
        Ok(written) if written == source => {}
        Ok(written) => {
            let moved = vendored
                .blocks()
                .iter()
                .filter(|(_, block)| !written.contains(block.as_str()))
                .map(|(name, _)| format!("`{name}`"))
                .collect::<Vec<_>>();
            differences.push(if moved.is_empty() {
                format!("{}: its header differs", request.out.display())
            } else {
                format!(
                    "{}: {} differ(s) from the owner's type",
                    request.out.display(),
                    moved.join(", ")
                )
            });
        }
        Err(_) => differences.push(format!("{}: not written", request.out.display())),
    }
    match fs::read_to_string(&request.conformances) {
        Ok(written) if written == fragment => {}
        Ok(_) => differences.push(format!(
            "{}: the pinned services or entries differ",
            request.conformances.display()
        )),
        Err(_) => differences.push(format!("{}: not written", request.conformances.display())),
    }
    if differences.is_empty() {
        println!(
            "{} and {} are up to date",
            request.out.display(),
            request.conformances.display()
        );
        return ExitCode::SUCCESS;
    }
    eprintln!("the vendored copy is out of date; run the command without --check:");
    for difference in differences {
        eprintln!("  - {difference}");
    }
    ExitCode::from(1)
}
