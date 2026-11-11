//! A binding whose destination row is named by the trigger's own input keeps the trigger on the
//! branch that publishes the binding's event.
//!
//! `Announce` publishes `Announced` only on a branch that reads the input `account_id`, and the
//! binding `activate-on-announce` invokes `Activate` on the account that input names. Arranging
//! the account first and sending the trigger naming it sends `Announce` down another branch, so
//! the event is never published and a correct implementation fails every binding scenario. The
//! binding is refused by name instead, for a `when_related:` branch and for an input guard alike.

use ess_compiler::{resolve::compile, source::SourceMap, EssIr};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::synthesize::{synthesize, Synthesis};
use ess_conformance::{AdmittedSuite, Runner};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use serde_json::Value;

const RELATED: &str = include_str!("fixtures/binding-destination-trigger-branch.yaml");
const GUARDED: &str = include_str!("fixtures/binding-destination-trigger-branch-guard.yaml");

fn model(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).expect("the fixture parses");
    let spec = Specification::assemble([(Source::new("model.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("the fixture validates:\n{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|d| panic!("the fixture resolves:\n{d}"))
}

fn scenario_ids(synthesis: &Synthesis) -> Vec<String> {
    let document: Value =
        serde_json::from_str(&synthesis.suite.to_canonical_json().expect("serialises"))
            .expect("JSON");
    document["scenarios"]
        .as_object()
        .expect("scenarios")
        .keys()
        .cloned()
        .collect()
}

fn refused_and_passing(text: &str) {
    let ir = model(text);
    let synthesis = synthesize(&ir);

    let refusals: Vec<String> = synthesis.refusals.iter().map(ToString::to_string).collect();
    // One refusal per scenario kind the binding would have had, each with the same reason.
    assert!(!refusals.is_empty(), "the binding is refused");
    for refusal in &refusals {
        for part in [
            "ESS-SYNTH-010",
            "activate-on-announce",
            "Activate` on an existing row named by",
            "the trigger's branch `demo.acct.Announce/announced` reads the input that names the row",
        ] {
            assert!(refusal.contains(part), "`{part}` in {refusal}");
        }
    }
    let ids = scenario_ids(&synthesis);
    // The mapping scenario reads the document through the plain trigger where the row cannot be
    // arranged, so it stays and passes below; every aspect needing the row is refused.
    let kept: Vec<&String> = ids
        .iter()
        .filter(|id| id.contains("activate-on-announce/binding"))
        .collect();
    assert_eq!(
        kept,
        ["activate-on-announce/binding/mapping"],
        "only the mapping scenario of the refused binding: {ids:#?}"
    );

    let admitted = AdmittedSuite::from_suite(&synthesis.suite).expect("the suite is admitted");
    let report = Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &Interpreted::for_model(ir))
        .into_report();
    let failed: Vec<String> = report
        .scenarios
        .iter()
        .filter(|scenario| scenario.status != Status::Passed)
        .map(|scenario| format!("{} {:?}", scenario.scenario, scenario.status))
        .collect();
    assert_eq!(failed, Vec::<String>::new());
}

#[test]
fn a_binding_destination_arrangement_keeps_the_trigger_on_its_branch() {
    refused_and_passing(RELATED);
}

#[test]
fn an_input_guard_reading_the_destination_input_refuses_the_binding() {
    refused_and_passing(GUARDED);
}
