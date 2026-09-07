//! Filesystem loading for ESS specifications and infrastructure observations.

use std::fs;
use std::path::Path;

use anyhow::{bail, Context, Result};
use ess_compiler::source::SourceMap;
use ess_compiler::{Diagnostics, EssIr};

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

/// Parses, assembles, validates, and resolves a specification.
pub(crate) fn specification(path: &Path) -> Result<LoadedSpec> {
    let inputs =
        crate::input_discovery::acquire(path, crate::input_discovery::Kind::Specification)?;
    let files_read = inputs.len();

    let mut parsed = Vec::new();
    let mut texts = SourceMap::new();
    let mut problems = Vec::new();
    for input in inputs {
        let source = ess_domain::system::Source::new(input.identity);
        let text = input.text;
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
        Some(infra_domain::OBSERVATION_FORMAT | "infra-observation/2") => {
            let raw: infra_domain::RawBundle = serde_json::from_value(value)
                .with_context(|| format!("{} is not an observation bundle", path.display()))?;
            Ok(match infra_domain::Observation::try_from(raw) {
                Ok(observation) => LoadedInfra::Ir(Box::new(infra_compiler::compile(&observation))),
                Err(errors) => LoadedInfra::Refused(errors),
            })
        }
        Some(infra_compiler::IR_FORMAT | "infra-ir/2") => {
            Ok(match infra_compiler::read_document(&value) {
                Ok(ir) => LoadedInfra::Ir(Box::new(ir)),
                Err(errors) => LoadedInfra::Refused(errors),
            })
        }
        other => bail!(
            "{} declares format {:?}; expected `{}` or `{}`",
            path.display(),
            other.unwrap_or("<none>"),
            infra_domain::OBSERVATION_FORMAT,
            infra_compiler::IR_FORMAT
        ),
    }
}
