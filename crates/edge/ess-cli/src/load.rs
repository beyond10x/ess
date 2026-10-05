//! Filesystem loading for ESS specifications and infrastructure observations.

use std::fs;
use std::path::Path;

use anyhow::{bail, Context, Result};
use ess_compiler::source::SourceMap;
use ess_compiler::{Diagnostics, EssIr};

#[cfg(test)]
#[path = "load_accounting_tests.rs"]
mod accounting_tests;

/// A loaded specification or every accumulated diagnostic.
pub(crate) enum LoadedSpec {
    /// The specification compiled to a resolved IR.
    Compiled { ir: Box<EssIr>, files_read: usize },
    /// Parsing, assembly, or reference resolution refused the input.
    Refused {
        files_read: usize,
        problems: Vec<String>,
        diagnostics: Diagnostics,
    },
}

/// The parsed source documents of a specification, before assembly.
pub(crate) struct RawLoaded {
    /// Every document, as parsed, with the source it came from.
    pub(crate) parsed: Vec<(ess_domain::system::Source, ess_domain::spec::RawSpecFile)>,
    /// The text of every document, for locating diagnostics.
    pub(crate) texts: SourceMap,
    /// How many files were read.
    pub(crate) files_read: usize,
    /// Every document that did not parse; empty when all did.
    pub(crate) problems: Vec<String>,
}

/// Reads and parses every source document of a specification, and assembles nothing.
///
/// The one parse path: [`specification`] is this followed by assembly and compilation, and
/// `ess verify conform mutate` edits what this returns before doing the same.
pub(crate) fn raw_specification(path: &Path) -> Result<RawLoaded> {
    let inputs =
        crate::input_discovery::acquire(path, crate::input_discovery::Kind::Specification)?;
    Ok(parse_specification_inputs(&inputs))
}

fn parse_specification_inputs(inputs: &[crate::input_discovery::Input]) -> RawLoaded {
    let files_read = inputs.len();

    let mut parsed = Vec::new();
    let mut texts = SourceMap::new();
    let mut problems = Vec::new();
    // Read together, so a newtype declared in one file may key a map in another.
    let every = inputs
        .iter()
        .map(|input| input.text.as_str())
        .collect::<Vec<_>>();
    let results = ess_domain::spec::RawSpecFile::parse_all(&every);
    for (input, result) in inputs.iter().zip(results) {
        let source = ess_domain::system::Source::new(input.identity.clone());
        texts.insert(source.as_str(), input.text.as_str());
        match result {
            Ok(raw) => parsed.push((source, raw)),
            Err(error) => problems.push(format!("{}: {error}", source.as_str())),
        }
    }
    RawLoaded {
        parsed,
        texts,
        files_read,
        problems,
    }
}

/// Parses, assembles, validates, and resolves a specification.
pub(crate) fn specification(path: &Path) -> Result<LoadedSpec> {
    Ok(compile_specification(raw_specification(path)?))
}

/// Retain the same original acquisition used to compile browser source authority.
pub(crate) fn browser_specification(
    path: &Path,
) -> Result<(
    LoadedSpec,
    Vec<ess_conformance::web_execution::bundle::SourceDocument>,
)> {
    let inputs =
        crate::input_discovery::acquire(path, crate::input_discovery::Kind::Specification)?;
    let sources = inputs
        .iter()
        .enumerate()
        .map(
            |(index, input)| ess_conformance::web_execution::bundle::SourceDocument {
                path: format!("sources/{index:04}.yaml"),
                text: input.text.clone(),
            },
        )
        .collect();
    Ok((
        compile_specification(parse_specification_inputs(&inputs)),
        sources,
    ))
}

/// [`specification`], with the advisory warnings of a specification that compiled
/// (beyond10x/ess#437): a relation the model only implies. Empty for a refused one, whose errors
/// are the repair.
pub(crate) fn advised_specification(path: &Path) -> Result<(LoadedSpec, Diagnostics)> {
    Ok(compile_advised(raw_specification(path)?, true))
}

fn compile_specification(raw: RawLoaded) -> LoadedSpec {
    compile_advised(raw, false).0
}

/// Compiles `raw`, and with `advise` collects the warnings of a specification that compiled. The
/// warnings are read from the assembled specification beside the compilation, never from or into
/// the IR, so a specification compiles to the same bytes either way.
fn compile_advised(raw: RawLoaded, advise: bool) -> (LoadedSpec, Diagnostics) {
    let RawLoaded {
        parsed,
        texts,
        files_read,
        problems,
    } = raw;

    if !problems.is_empty() {
        let refused = LoadedSpec::Refused {
            files_read,
            problems,
            diagnostics: Diagnostics::new(),
        };
        return (refused, Diagnostics::new());
    }

    let labels = parsed
        .iter()
        .map(|(source, _)| source.to_string())
        .collect::<Vec<_>>();
    let assembled = match ess_domain::spec::Specification::assemble(parsed) {
        Ok(specification) => specification,
        Err(errors) => {
            let diagnostics = ess_compiler::resolve::diagnose_locating(&errors, &texts, &labels);
            let refused = LoadedSpec::Refused {
                files_read,
                problems: errors.as_slice().iter().map(ToString::to_string).collect(),
                diagnostics,
            };
            return (refused, Diagnostics::new());
        }
    };

    match ess_compiler::compile(&assembled, &texts) {
        Ok(ir) => {
            let warnings = if advise {
                ess_compiler::resolve::advise_locating(&assembled, &texts, &labels)
            } else {
                Diagnostics::new()
            };
            let compiled = LoadedSpec::Compiled {
                ir: Box::new(ir),
                files_read,
            };
            (compiled, warnings)
        }
        Err(diagnostics) => {
            let refused = LoadedSpec::Refused {
                files_read,
                problems: Vec::new(),
                diagnostics,
            };
            (refused, Diagnostics::new())
        }
    }
}

/// A validated infrastructure IR or accumulated input refusals.
pub(crate) enum LoadedInfra {
    /// The resolved infrastructure model.
    Ir(Box<infra_compiler::InfraIr>),
    /// The input was a document, but it violated its format contract.
    Refused(infra_domain::ValidationErrors),
}

/// Reads an `infra-observation/1`, `/2` or `/3` bundle, or a persisted `infra-ir/1`, `/2` or `/3`.
pub(crate) fn infrastructure(path: &Path) -> Result<LoadedInfra> {
    let text = fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    let value: serde_json::Value = ess_primitives::json::from_str(&text)
        .with_context(|| format!("{} is not JSON", path.display()))?;
    match value.get("format").and_then(serde_json::Value::as_str) {
        Some(
            infra_domain::OBSERVATION_FORMAT
            | infra_domain::observation::SCOPED_OBSERVATION_FORMAT
            | infra_domain::PRESENCE_OBSERVATION_FORMAT,
        ) => {
            let raw: infra_domain::RawBundle = serde_json::from_value(value)
                .with_context(|| format!("{} is not an observation bundle", path.display()))?;
            Ok(match infra_domain::Observation::try_from(raw) {
                Ok(observation) => LoadedInfra::Ir(Box::new(infra_compiler::compile(&observation))),
                Err(errors) => LoadedInfra::Refused(errors),
            })
        }
        Some(infra_compiler::IR_FORMAT | "infra-ir/2" | infra_compiler::PRESENCE_IR_FORMAT) => {
            Ok(match infra_compiler::read_document(&value) {
                Ok(ir) => LoadedInfra::Ir(Box::new(ir)),
                Err(errors) => LoadedInfra::Refused(errors),
            })
        }
        other => bail!(
            "{} declares format {:?}; expected `{}` or `{}`",
            path.display(),
            other.unwrap_or("<none>"),
            infra_domain::PRESENCE_OBSERVATION_FORMAT,
            infra_compiler::PRESENCE_IR_FORMAT
        ),
    }
}
