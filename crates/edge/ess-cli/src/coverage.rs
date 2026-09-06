//! Explicit coverage acquisition and selection; legacy command paths keep their own defaults.
use super::{Format, SpecPath, SuiteTarget};
use anyhow::{bail, Context, Result};
use ess_compiler::EssIr;
use ess_conformance::{
    coverage::{AdmittedInput, Origins, Scope},
    coverage_build::{self, CoverageSource},
    AdmittedSuite, ScenarioId,
};
use std::{
    fs,
    path::{Path, PathBuf},
    process::ExitCode,
};

pub(super) fn sources(path: Option<&Path>) -> Result<Vec<CoverageSource>> {
    let Some(path) = path else {
        return Ok(Vec::new());
    };
    let metadata =
        fs::symlink_metadata(path).with_context(|| format!("reading {}", path.display()))?;
    if metadata.file_type().is_symlink() {
        bail!("coverage authored root/file is a symlink");
    }
    let (root, mut files) = if metadata.is_file() {
        (
            path.parent().unwrap_or(Path::new(".")),
            vec![path.to_path_buf()],
        )
    } else if metadata.is_dir() {
        let files = fs::read_dir(path)?
            .map(|entry| entry.map(|e| e.path()))
            .collect::<std::io::Result<Vec<_>>>()?
            .into_iter()
            .filter(|p| p.extension().is_some_and(|e| e == "yaml" || e == "yml"))
            .collect::<Vec<_>>();
        if files.is_empty() {
            bail!("refused --scenarios: no .yaml or .yml scenario files found directly in this directory; subdirectories are not searched");
        }
        (path, files)
    } else {
        bail!("coverage authored input must be a regular file or directory");
    };
    let root = if root.as_os_str().is_empty() {
        Path::new(".")
    } else {
        root
    };
    if fs::symlink_metadata(root)?.file_type().is_symlink() {
        bail!("coverage authored root is a symlink");
    }
    files.sort();
    files
        .iter()
        .map(|file| {
            let metadata = fs::symlink_metadata(file)?;
            if !metadata.is_file() || metadata.file_type().is_symlink() {
                bail!(
                    "coverage authored input is not a regular non-symlink file: {}",
                    file.display()
                );
            }
            let identity = file.strip_prefix(root).or_else(|_| {
                file.file_name()
                    .map(Path::new)
                    .ok_or_else(|| anyhow::anyhow!("missing file name"))
            })?;
            let identity = identity
                .to_str()
                .context("authored identity is not UTF-8")?;
            let text =
                fs::read_to_string(file).with_context(|| format!("reading {}", file.display()))?;
            Ok(CoverageSource::new(identity, text)?)
        })
        .collect()
}
pub(super) fn fresh(
    ir: &EssIr,
    path: Option<&Path>,
    component: Option<&str>,
    authored: bool,
) -> Result<AdmittedInput> {
    let sources = sources(path)?;
    let scope = component.map_or(Ok(Scope::System), Scope::component)?;
    let origins = if authored {
        Origins::Authored
    } else if path.is_some() {
        Origins::GeneratedAndAuthored
    } else {
        Origins::Generated
    };
    Ok(coverage_build::build(ir, &sources, scope, origins)?)
}
pub(super) fn generate(
    input: &SpecPath,
    target: SuiteTarget,
    out: Option<&Path>,
    component: Option<&str>,
    scenarios: Option<&Path>,
    authored: bool,
) -> Result<ExitCode> {
    let Ok((ir, _)) = super::resolved(&input.path, input.format)? else {
        return Ok(ExitCode::from(1));
    };
    let admitted = fresh(&ir, scenarios, component, authored)?;
    let suite = admitted.selected();
    let inventory = suite.coverage().expect("coverage builder");
    let json = suite.original_json();
    match target {
        SuiteTarget::Ir => {
            if let Some(out) = out {
                let destination = super::preflight_named_output(out)?;
                fs::write(destination, json)?;
            }
        }
        SuiteTarget::Go => {
            let files = ess_conformance::go::emit_input(&admitted)?;
            if let Some(out) = out {
                super::write_generated_files(
                    out,
                    files.iter().map(|f| (f.path.as_str(), f.contents.as_str())),
                )?;
            }
        }
    }
    match input.format {
        Format::Json => print!("{json}"),
        Format::Yaml => super::render(
            &serde_json::from_str::<serde_json::Value>(json)?,
            Format::Yaml,
        )?,
        Format::Text => {
            for refusal in &inventory.refused {
                println!("refused[{}]: {}", refusal.code, refusal.message);
            }
            for outside in &inventory.outside {
                println!("outside: {} ({:?})", outside.scenario, outside.reason);
            }
            println!(
                "{} selected scenario(s), {} authored source(s), {} refusal occurrence(s)",
                suite.suite().len(),
                inventory.authored_sources.len(),
                inventory.refused.len()
            );
        }
    }
    Ok(if inventory.is_complete() {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    })
}
pub(super) fn select(
    suite: Option<&Path>,
    input: Option<&Path>,
    ids: &Path,
    out: &Path,
) -> Result<ExitCode> {
    let input = match (suite, input) {
        (Some(path), None) => {
            AdmittedInput::from_suite(AdmittedSuite::from_json(&fs::read_to_string(path)?)?)?
        }
        (None, Some(path)) => AdmittedInput::from_json(&fs::read_to_string(path)?)?,
        _ => bail!("select requires exactly one --suite or --suite-input"),
    };
    let ids: Vec<ScenarioId> = serde_json::from_str(&fs::read_to_string(ids)?)?;
    let child = input.select(&ids)?;
    let text = child.document().to_canonical_json()?;
    let destination: PathBuf = super::preflight_named_output(out)?;
    fs::write(destination, text)?;
    Ok(ExitCode::SUCCESS)
}
pub(super) fn web(
    input: &SpecPath,
    scenarios: Option<&Path>,
    out: Option<&Path>,
) -> Result<ExitCode> {
    let Ok((ir, _)) = super::resolved(&input.path, input.format)? else {
        return Ok(ExitCode::from(1));
    };
    let admitted = fresh(&ir, scenarios, None, true)?;
    let artifacts = ess_conformance::web::emit_input(&ir, &admitted)?;
    super::write_artifacts(out, &artifacts)?;
    Ok(
        if admitted
            .selected()
            .coverage()
            .expect("coverage builder")
            .is_complete()
        {
            ExitCode::SUCCESS
        } else {
            ExitCode::from(1)
        },
    )
}

// Kept around the closure so the report choice is checked before target construction.
pub(super) fn execute<R>(
    suite: &AdmittedSuite,
    report_format: &str,
    target_run: impl FnOnce() -> R,
) -> Result<R> {
    if suite.coverage().is_some() && report_format != "2" {
        bail!("suite/5 requires explicit --report-format 2 before execution");
    }
    Ok(target_run())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn suite5_pair_refusal_precedes_target_construction() {
        let original = serde_json::json!({
            "provenance":{"suite_version":"ess-conformance/5","system":"example","specification_version":"v1",
                "spec_digest":"a".repeat(64),"contract_digest":"b".repeat(64)},
            "scenarios":{},
            "coverage":{"selection":{"scope":{"kind":"system"},"origins":"generated","filter":{"kind":"all"}},
                "knowledge":"complete_inventory","generated":[],"authored":[],"outside":[],"refused":[],
                "authored_sources":{},"counts":{"generated":0,"authored":0,"outside":0,"refused":0}}
        }).to_string();
        let suite = AdmittedSuite::from_json(&original).unwrap();
        let constructed = std::cell::Cell::new(0);
        let refused = execute(&suite, "1", || constructed.set(constructed.get() + 1));
        assert!(refused.is_err());
        assert_eq!(constructed.get(), 0);
        execute(&suite, "2", || constructed.set(constructed.get() + 1)).unwrap();
        assert_eq!(constructed.get(), 1);
    }
}
