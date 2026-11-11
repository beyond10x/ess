//! An authored scenario retires the refusal it covers (beyond10x/ess#735).
//!
//! The specification has a refusal synthesis cannot witness: `TakeNote` refuses a note of more
//! than 2000 tags, past the 1024 elements synthesis builds (`ESS-SYNTH-018`), and an authored
//! scenario sending 2001 tags reaches it. (The report's own `ConfirmDecision/already-exists` is
//! witnessed by synthesis since the existing-instance arrangement reads the creator's related row.)
//! `covers:` (`ess-scenario/7`) names the refusal's scenario id; the occurrence stays in the suite,
//! carries a `covered_by` mark (`ess-conformance/65`), and declared coverage counts it complete only
//! when the covering scenario is selected and passes in the same run.

use ess_compiler::{resolve::compile, source::SourceMap, EssIr};
use ess_conformance::{
    coverage::{AdmittedInput, Origins, Scope},
    coverage_build::{self, CoverageSource},
    interpret::Interpreted,
    CountReport, CountStatus, Runner, ScenarioId,
};
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source,
};
use std::{
    fs,
    path::{Path, PathBuf},
};

const REFUSED: &str = "example.review.TakeNote/outcome/too-many-tags";
const COVERING: &str = "example.review/authored/a-note-with-too-many-tags";

/// The specification and scenario are stored as `*.in` so the repository-model walkers
/// (`synthesis_suite_bytes_table`) do not pin them as models.
fn fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/authored-covers")
}

fn model() -> EssIr {
    let mut sources = SourceMap::new();
    let mut parsed = Vec::new();
    for name in ["system.yaml", "review.yaml"] {
        let original = fs::read_to_string(fixture().join(format!("{name}.in"))).unwrap();
        let raw = RawSpecFile::parse(&original).unwrap();
        sources.insert(name, original);
        parsed.push((Source::new(name), raw));
    }
    compile(&Specification::assemble(parsed).unwrap(), &sources).unwrap()
}

fn scenario() -> String {
    fs::read_to_string(fixture().join("scenarios/too-many-tags.yaml.in")).unwrap()
}

fn build(ir: &EssIr, text: &str) -> Result<AdmittedInput, ess_conformance::AdmissionError> {
    let source = CoverageSource::new("scenarios/too-many-tags.yaml", text).unwrap();
    coverage_build::build(ir, &[source], Scope::System, Origins::GeneratedAndAuthored)
}

fn id(text: &str) -> ScenarioId {
    ScenarioId::parse(text).unwrap()
}

fn report(ir: &EssIr, input: &AdmittedInput) -> CountReport {
    let suite = input.selected();
    let run =
        Runner::for_suite(suite.suite()).run_admitted(suite, &Interpreted::for_model(ir.clone()));
    CountReport::from_run(&run, suite).unwrap()
}

#[test]
fn a_passing_covering_scenario_completes_declared_coverage() {
    let ir = model();
    let input = build(&ir, &scenario()).unwrap();
    let inventory = input.selected().coverage().unwrap();
    assert_eq!(inventory.counts.refused, 1, "the occurrence is kept");
    assert_eq!(inventory.refused[0].scenario, Some(id(REFUSED)));
    assert_eq!(inventory.refused[0].covered_by, Some(id(COVERING)));
    assert_eq!(
        input
            .selected()
            .suite()
            .provenance
            .suite_version
            .to_string(),
        "ess-conformance/65"
    );
    assert!(inventory.is_complete());

    let report = report(&ir, &input);
    assert_eq!(report.execution_status(), CountStatus::Passed);
    assert_eq!(report.conformance_status(), CountStatus::Passed);
}

#[test]
fn a_failing_covering_scenario_does_not_complete_declared_coverage() {
    let ir = model();
    let failing = scenario().replace("outcome: taken", "outcome: note-exists");
    let input = build(&ir, &failing).unwrap();
    let report = report(&ir, &input);
    assert_eq!(report.execution_status(), CountStatus::Failed);
    assert_ne!(report.conformance_status(), CountStatus::Passed);
}

#[test]
fn a_child_suite_without_the_covering_scenario_is_inconclusive() {
    let ir = model();
    let input = build(&ir, &scenario()).unwrap();
    let kept: Vec<ScenarioId> = input
        .selected()
        .suite()
        .scenarios
        .keys()
        .filter(|scenario| **scenario != id(COVERING))
        .cloned()
        .collect();
    let child = input.select(&kept).unwrap();
    assert!(!child.selected().coverage().unwrap().is_complete());
    let report = report(&ir, &child);
    assert_eq!(report.execution_status(), CountStatus::Passed);
    assert_eq!(report.conformance_status(), CountStatus::Inconclusive);
}

#[test]
fn a_cover_of_an_id_synthesis_did_not_refuse_is_refused_by_name() {
    let ir = model();
    let missing = "example.review.ConfirmDecision/outcome/missing";
    let text = scenario().replace(REFUSED, missing);
    let error = build(&ir, &text).unwrap_err().to_string();
    assert!(error.contains("covers.unknown-scenario"), "{error}");
    assert!(error.contains(missing), "{error}");
}

#[test]
fn covers_is_an_unsupported_key_below_scenario_format_7() {
    let ir = model();
    let text = scenario().replace("ess-scenario/7", "ess-scenario/6");
    let input = build(&ir, &text).unwrap();
    let inventory = input.selected().coverage().unwrap();
    let refusal = inventory
        .refused
        .iter()
        .find(|refusal| refusal.source.is_some())
        .expect("the document is refused");
    assert!(refusal.message.contains("covers"), "{}", refusal.message);
    assert!(
        refusal.message.contains("ess-scenario/7"),
        "{}",
        refusal.message
    );
    assert!(inventory.refused.iter().all(|r| r.covered_by.is_none()));
    assert!(!inventory.is_complete());
}

#[test]
fn without_covers_the_suite_keeps_its_format_and_its_refusal_bytes() {
    let ir = model();
    let text = scenario().replace(
        "ess-scenario/7\ncovers:\n  - example.review.TakeNote/outcome/too-many-tags",
        "ess-scenario/6",
    );
    let input = build(&ir, &text).unwrap();
    let suite = input.selected();
    assert_ne!(
        suite.suite().provenance.suite_version.to_string(),
        "ess-conformance/65"
    );
    assert!(!suite.original_json().contains("covered_by"));
    assert!(!suite.coverage().unwrap().is_complete());
}
