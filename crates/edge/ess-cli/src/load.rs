//! Filesystem loading for ESS specifications and infrastructure observations.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use ess_compiler::source::SourceMap;
use ess_compiler::{Diagnostics, EssIr};

const SPECIFICATION_HEADER: &str = "system.yaml";

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

fn specification_files(path: &Path) -> Result<Vec<PathBuf>> {
    if path.is_file() {
        return Ok(vec![path.to_path_buf()]);
    }
    if !path.join(SPECIFICATION_HEADER).is_file() {
        bail!(
            "{} is not an ESS specification: a directory must contain `{SPECIFICATION_HEADER}`",
            path.display()
        );
    }

    let mut files = Vec::new();
    let mut visited = BTreeSet::new();
    let mut pending = vec![path.to_path_buf()];
    while let Some(directory) = pending.pop() {
        let identity = directory
            .canonicalize()
            .with_context(|| format!("resolving {}", directory.display()))?;
        if !visited.insert(identity) {
            continue;
        }
        for entry in
            fs::read_dir(&directory).with_context(|| format!("reading {}", directory.display()))?
        {
            let entry = entry.with_context(|| format!("reading {}", directory.display()))?;
            let child = entry.path();
            if child.is_dir() {
                pending.push(child);
            } else if child
                .extension()
                .is_some_and(|extension| extension == "yaml" || extension == "yml")
            {
                files.push(child);
            }
        }
    }
    files.sort();
    Ok(files)
}

/// Parses, assembles, validates, and resolves a specification.
pub(crate) fn specification(path: &Path) -> Result<LoadedSpec> {
    let root = path
        .canonicalize()
        .with_context(|| format!("resolving {}", path.display()))?;
    let files = specification_files(&root)?;
    let files_read = files.len();
    let base = if root.is_file() {
        root.parent().unwrap_or(root.as_path())
    } else {
        root.as_path()
    };

    let mut parsed = Vec::new();
    let mut texts = SourceMap::new();
    let mut problems = Vec::new();
    for file in files {
        let relative = file.strip_prefix(base).unwrap_or(file.as_path());
        let source = ess_domain::system::Source::new(relative.display().to_string());
        let text =
            fs::read_to_string(&file).with_context(|| format!("reading {}", file.display()))?;
        texts.insert(source.as_str(), text.as_str());
        match ess_domain::spec::RawSpecFile::parse(&text) {
            Ok(raw) => parsed.push((source, raw)),
            Err(error) => problems.push(format!("{}: {error}", source.as_str())),
        }
    }

    if !problems.is_empty() {
        return Ok(LoadedSpec::Refused {
            files_read,
            problems,
            diagnostics: Diagnostics::new(),
        });
    }

    let labels = parsed
        .iter()
        .map(|(source, _)| source.to_string())
        .collect::<Vec<_>>();
    let assembled = match ess_domain::spec::Specification::assemble(parsed) {
        Ok(specification) => specification,
        Err(errors) => {
            let diagnostics = ess_compiler::resolve::diagnose_locating(&errors, &texts, &labels);
            return Ok(LoadedSpec::Refused {
                files_read,
                problems: errors.as_slice().iter().map(ToString::to_string).collect(),
                diagnostics,
            });
        }
    };

    match ess_compiler::compile(&assembled, &texts) {
        Ok(ir) => Ok(LoadedSpec::Compiled {
            ir: Box::new(ir),
            files_read,
        }),
        Err(diagnostics) => Ok(LoadedSpec::Refused {
            files_read,
            problems: Vec::new(),
            diagnostics,
        }),
    }
}

/// A validated infrastructure IR or accumulated input refusals.
pub(crate) enum LoadedInfra {
    /// The resolved infrastructure model.
    Ir(Box<infra_compiler::InfraIr>),
    /// The input was a document, but it violated its format contract.
    Refused(infra_domain::ValidationErrors),
}

/// Reads either `infra-observation/1` or persisted `infra-ir/1`.
pub(crate) fn infrastructure(path: &Path) -> Result<LoadedInfra> {
    let text = fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    let value: serde_json::Value =
        serde_json::from_str(&text).with_context(|| format!("{} is not JSON", path.display()))?;
    match value.get("format").and_then(serde_json::Value::as_str) {
        Some(infra_domain::OBSERVATION_FORMAT) => {
            let raw: infra_domain::RawBundle = serde_json::from_value(value)
                .with_context(|| format!("{} is not an observation bundle", path.display()))?;
            Ok(match infra_domain::Observation::try_from(raw) {
                Ok(observation) => LoadedInfra::Ir(Box::new(infra_compiler::compile(&observation))),
                Err(errors) => LoadedInfra::Refused(errors),
            })
        }
        Some(infra_compiler::IR_FORMAT) => Ok(match infra_compiler::read_document(&value) {
            Ok(ir) => LoadedInfra::Ir(Box::new(ir)),
            Err(errors) => LoadedInfra::Refused(errors),
        }),
        other => bail!(
            "{} declares format {:?}; expected `{}` or `{}`",
            path.display(),
            other.unwrap_or("<none>"),
            infra_domain::OBSERVATION_FORMAT,
            infra_compiler::IR_FORMAT
        ),
    }
}
