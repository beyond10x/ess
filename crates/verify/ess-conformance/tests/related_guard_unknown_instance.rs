//! `unknown_instance:` beside a `when_related:` guard (source format `ess/27`, downstream report,
//! 2026-10-18).
//!
//! `tests/fixtures/related-guard-unknown-instance.yaml`: `ChangeTier` addresses a seat by the
//! identity its input carries and reads the tier its input names. The precedence order answers an
//! unknown tier first (step 1), then an unknown seat (step 3), then a closed tier (step 5), then
//! the change. The `unknown-seat` scenario sends an identity no seat carries beside a tier that is
//! present and closed, so a target reading the tier before the seat's existence answers
//! `tier-closed` and fails it; `ess-synth`'s `related_guard_unknown_instance_served.rs` holds a
//! generated target edited to read the tier first to that verdict.

use std::collections::BTreeMap;

use ess_compiler::ir::EssIr;
use ess_compiler::ir::PrecedencePlan;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::synthesize::synthesize;
use ess_conformance::{
    AdmittedSuite, AdvancingClock, ConformanceSuite, Ids, Runner, RunnerConfig, ScenarioStep,
    ScenarioValue,
};
use ess_domain::command::precedence::Phase;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_primitives::node::Node;

const MODEL: &str = include_str!("fixtures/related-guard-unknown-instance.yaml");
const UNKNOWN_SEAT: &str = "demo.seats.ChangeTier/outcome/unknown-seat";

fn model() -> EssIr {
    let raw = RawSpecFile::parse(MODEL).unwrap_or_else(|error| panic!("{error}"));
    let spec = Specification::assemble([(Source::new("related-guard-unknown-instance.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn suite() -> ConformanceSuite {
    let synthesis = synthesize(&model());
    assert!(
        synthesis.refusals.is_empty(),
        "nothing is refused: {:#?}",
        synthesis.refusals
    );
    synthesis.suite
}

fn run(suite: &ConformanceSuite) -> BTreeMap<String, Status> {
    let admitted = AdmittedSuite::from_suite(suite).unwrap_or_else(|error| panic!("{error}"));
    Runner::new(
        RunnerConfig::default(),
        AdvancingClock::default(),
        Ids::for_suite(suite),
    )
    .run_admitted(&admitted, &Interpreted::for_model(model()))
    .into_report()
    .scenarios
    .into_iter()
    .map(|scenario| (scenario.scenario.to_string(), scenario.status))
    .collect()
}

#[test]
fn every_branch_beside_unknown_instance_is_synthesized() {
    let ids: Vec<String> = suite().scenarios.keys().map(ToString::to_string).collect();
    for required in [
        UNKNOWN_SEAT,
        "demo.seats.ChangeTier/outcome/unknown-tier",
        "demo.seats.ChangeTier/outcome/tier-closed",
        "demo.seats.ChangeTier/outcome/changed",
    ] {
        assert!(ids.iter().any(|id| id == required), "{required}: {ids:#?}");
    }
}

#[test]
fn the_unknown_identity_is_sent_beside_a_present_closed_tier() {
    let suite = suite();
    let scenario = suite
        .scenarios
        .iter()
        .find(|(id, _)| id.to_string() == UNKNOWN_SEAT)
        .map(|(_, scenario)| scenario)
        .expect("synthesized");
    let mut closed = None;
    let mut sent = None;
    for step in &scenario.steps {
        let ScenarioStep::ExecuteCommand { command, input, .. } = step else {
            continue;
        };
        match command.to_string().as_str() {
            "demo.seats.DefineTier" => {
                closed = Some(input.get("open").cloned());
            }
            "demo.seats.ChangeTier" => sent = Some(input.clone()),
            _ => {}
        }
    }
    assert_eq!(
        closed,
        Some(Some(ScenarioValue::Literal {
            value: Node::Bool(false)
        })),
        "the tier is defined closed: {:#?}",
        scenario.steps
    );
    let sent = sent.expect("the command is sent");
    assert!(
        matches!(sent.get("tier_id"), Some(ScenarioValue::Instance { .. })),
        "the tier the scenario defined is named: {sent:?}"
    );
    assert!(
        matches!(sent.get("seat_id"), Some(ScenarioValue::Literal { .. })),
        "the seat is an identity no row carries: {sent:?}"
    );
}

#[test]
fn the_interpreter_passes_every_scenario() {
    let statuses = run(&suite());
    assert!(
        statuses.values().all(|status| *status == Status::Passed),
        "{statuses:#?}"
    );
}

/// The order every consumer reads: the unknown tier at step 1, the unknown seat at step 3, the
/// closed tier at step 5 — before the change, which answers at step 6.
#[test]
fn the_precedence_plan_answers_the_unknown_seat_before_the_closed_tier() {
    let ir = model();
    let command = ir
        .commands()
        .values()
        .find(|command| command.name.to_string() == "demo.seats.ChangeTier")
        .expect("declared");
    let plan = PrecedencePlan::new(command, ir.format());
    let phase_of = |name: &str| {
        plan.phases()
            .iter()
            .find(|planned| {
                planned
                    .branches
                    .iter()
                    .any(|branch| branch.name.as_str() == name)
            })
            .map(|planned| planned.phase)
    };
    assert_eq!(phase_of("unknown-tier"), Some(Phase::RelatedRow));
    assert_eq!(phase_of("unknown-seat"), Some(Phase::Existence));
    assert_eq!(phase_of("tier-closed"), Some(Phase::PresentRelated));
    assert_eq!(phase_of("changed"), Some(Phase::Default));
}
