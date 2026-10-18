//! A row-set guard over a field a creating branch copies from another row with
//! `{related: {via, field}}` (downstream report, 2026-10-18).
//!
//! `tests/fixtures/row-set-related-copy.yaml`: `Issue` stores on a seat whether its tier
//! qualifies, read from the tier `input.tier_id` names; `Check` answers `qualified` when a seat of
//! the holder qualifies. The seat's `qualifies` is nothing the issuing input carries, so synthesis
//! arranges it through the tier: a tier defined with the value the row needs, then the seat issued
//! under it. Before this the walk over the steps left the copied value unknown, and both branches
//! of `Check` were refused with `ESS-SYNTH-001` ("its own row set is Unknown").

use std::collections::BTreeMap;

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::synthesize::synthesize;
use ess_conformance::{
    AdmittedSuite, AdvancingClock, ConformanceSuite, Ids, Runner, RunnerConfig, ScenarioStep,
    ScenarioValue,
};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_primitives::node::Node;

const MODEL: &str = include_str!("fixtures/row-set-related-copy.yaml");
const COPY: &str = "qualifies: {related: {via: input.tier_id, field: qualifies}}";

fn model(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}"));
    let spec = Specification::assemble([(Source::new("row-set-related-copy.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn suite() -> ConformanceSuite {
    let synthesis = synthesize(&model(MODEL));
    assert!(
        synthesis.refusals.is_empty(),
        "nothing is refused: {:#?}",
        synthesis.refusals
    );
    synthesis.suite
}

fn run(suite: &ConformanceSuite, target: &Interpreted) -> BTreeMap<String, Status> {
    let admitted = AdmittedSuite::from_suite(suite).unwrap_or_else(|error| panic!("{error}"));
    Runner::new(
        RunnerConfig::default(),
        AdvancingClock::default(),
        Ids::for_suite(suite),
    )
    .run_admitted(&admitted, target)
    .into_report()
    .scenarios
    .into_iter()
    .map(|scenario| (scenario.scenario.to_string(), scenario.status))
    .collect()
}

#[test]
fn both_branches_of_a_row_set_over_a_copied_field_are_synthesized() {
    let ids: Vec<String> = suite().scenarios.keys().map(ToString::to_string).collect();
    for required in [
        "demo.seats.Check/outcome/qualified",
        "demo.seats.Check/outcome/not-qualified",
    ] {
        assert!(ids.iter().any(|id| id == required), "{required}: {ids:#?}");
    }
}

#[test]
fn the_copied_value_is_steered_through_the_row_it_is_copied_from() {
    let suite = suite();
    let scenario = suite
        .scenarios
        .iter()
        .find(|(id, _)| id.to_string() == "demo.seats.Check/outcome/qualified")
        .map(|(_, scenario)| scenario)
        .expect("synthesized");
    // Every tier the scenario defines, by the value it qualifies with.
    let mut defined: Vec<bool> = Vec::new();
    let mut issued_under_arranged_tier = 0;
    for step in &scenario.steps {
        let ScenarioStep::ExecuteCommand { command, input, .. } = step else {
            continue;
        };
        match command.to_string().as_str() {
            "demo.seats.DefineTier" => match input.get("qualifies") {
                Some(ScenarioValue::Literal {
                    value: Node::Bool(flag),
                }) => defined.push(*flag),
                other => panic!("a tier is defined with a literal: {other:?}"),
            },
            "demo.seats.Issue" => {
                assert!(
                    matches!(input.get("tier_id"), Some(ScenarioValue::Instance { .. })),
                    "a seat is issued under a tier the scenario defined: {input:?}"
                );
                issued_under_arranged_tier += 1;
            }
            _ => {}
        }
    }
    assert!(
        defined.contains(&true) && defined.contains(&false),
        "the selected seat's tier qualifies and the decoy's does not: {defined:?}"
    );
    assert!(
        issued_under_arranged_tier >= 3,
        "{issued_under_arranged_tier}"
    );
}

#[test]
fn the_interpreter_passes_and_a_target_copying_no_tier_fails() {
    let suite = suite();
    let healthy = run(&suite, &Interpreted::for_model(model(MODEL)));
    assert!(
        healthy.values().all(|status| *status == Status::Passed),
        "{healthy:#?}"
    );
    // A target storing `qualifies: true` on every seat, whatever its tier holds.
    assert!(MODEL.contains(COPY));
    let faulty = MODEL.replace(COPY, "qualifies: true");
    let statuses = run(&suite, &Interpreted::for_model(model(&faulty)));
    assert_ne!(
        statuses.get("demo.seats.Check/outcome/not-qualified"),
        Some(&Status::Passed),
        "{statuses:#?}"
    );
}
