//! Explicit coverage acquisition and selection; legacy command paths keep their own defaults.
use super::{Format, SpecPath, SuiteTarget};
use anyhow::{bail, Result};
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
    crate::input_discovery::authored(path, true)?
        .into_iter()
        .map(|input| Ok(CoverageSource::new(input.identity, input.text)?))
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
    compact: bool,
) -> Result<ExitCode> {
    let Ok((ir, _)) = super::resolved(&input.path, input.format)? else {
        return Ok(ExitCode::from(1));
    };
    let admitted = fresh(&ir, scenarios, component, authored)?;
    let suite = admitted.selected();
    let inventory = suite.coverage().expect("coverage builder");
    let compact_json;
    let json = if compact {
        compact_json = ess_conformance::coverage::compact_suite_document(suite.suite(), inventory)?;
        &compact_json
    } else {
        suite.original_json()
    };
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
                super::write_owned_files(
                    out,
                    "conformance-go",
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
    super::write_owned_artifacts(out, "conformance-browser", &artifacts)?;
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
    if suite.suite().provenance.suite_version.major() >= 8 && report_format != "2" {
        bail!("suite/8 and /9 require explicit --report-format 2 before execution");
    }
    if suite.suite().provenance.suite_version.major() >= 5 && report_format != "2" {
        bail!("suite/5, /6 and /7 require explicit --report-format 2 before execution");
    }
    Ok(target_run())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn newer_suite_pair_refusal_precedes_target_construction() {
        for major in [6, 7] {
            let mut document = serde_json::json!({
                "provenance":{"suite_version":format!("ess-conformance/{major}"),"system":"example","specification_version":"v1",
                    "spec_digest":"a".repeat(64),"contract_digest":"b".repeat(64)},
                "scenarios":{}
            });
            if major == 7 {
                document["coverage"] = serde_json::json!({
                    "selection":{"scope":{"kind":"system"},"origins":"generated","filter":{"kind":"all"}},
                    "knowledge":"complete_inventory","generated":[],"authored":[],"outside":[],"refused":[],
                    "authored_sources":{},"counts":{"generated":0,"authored":0,"outside":0,"refused":0}
                });
            }
            let suite = AdmittedSuite::from_json(&document.to_string()).unwrap();
            let constructed = std::cell::Cell::new(0);
            assert!(execute(&suite, "1", || constructed.set(constructed.get() + 1)).is_err());
            assert_eq!(
                constructed.get(),
                0,
                "report choice must precede all target effects"
            );
            execute(&suite, "2", || constructed.set(constructed.get() + 1)).unwrap();
            assert_eq!(constructed.get(), 1);
        }
    }

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
