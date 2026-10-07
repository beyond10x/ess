//! Adversary, w1fix pass 2: a fresh coverage suite for an authored `satisfies` reading `missing()` over
//! an `Optional` struct (beyond10x/ess#176). Pass 1 drove the ordinary `conform author` verb; the
//! coverage writer (`coverage_build::coverage_version`) takes the same construct only through the
//! model argument it gained in this unit, and nothing drives it with an authored scenario, with
//! and without generated obligations. Then a subset selected from that coverage suite must keep
//! /35 and its exact parent, since a selection narrows ids only.
use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::{
    coverage::{Origins, Scope},
    coverage_build::{build, CoverageSource},
    scenario::ScenarioInitialState,
};
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source,
};

const QUEUE: &str = include_str!("fixtures/defined-over-optional-aggregates.yaml");

const SCENARIO: &str = "type: ess-scenario/1
domain: demo.queue
scenario: a-fresh-queue-holds-no-metrics
summary: A queue just opened holds no metrics.
timeline:
  - at: 2026-01-05T09:00:00Z
    command: demo.queue.OpenQueue
    actor: demo.queue.Operator
    input: {}
    outcome: opened
assert:
  - view: demo.queue.QueueById
    satisfies: missing(metrics)
";

/// The queue without its invariant, so nothing synthesized reads `defined(metrics)`.
fn ir() -> EssIr {
    let text = QUEUE.replace(
        "    invariants:\n      - any: [state == Paused, {not: \"defined(metrics)\"}]\n",
        "",
    );
    assert_ne!(text, QUEUE, "the invariant is removed");
    let raw = RawSpecFile::parse(&text).unwrap_or_else(|error| panic!("{error}"));
    let spec = Specification::assemble([(Source::new("queue.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn version(json: &str) -> String {
    json.split("\"suite_version\":")
        .nth(1)
        .and_then(|rest| rest.split('"').nth(1))
        .unwrap_or("<none>")
        .to_owned()
}

fn check(origins: Origins) {
    let ir = ir();
    let source = CoverageSource::new("scenarios/fresh.yaml", SCENARIO).expect("identity");
    let input =
        build(&ir, &[source], Scope::System, origins).unwrap_or_else(|error| panic!("{error}"));
    let original = input.selected().original_json().to_owned();
    assert_eq!(
        input.selected().suite().provenance.scenario_initial_state,
        Some(ScenarioInitialState::Empty),
        "{origins:?}: fresh coverage suite initial state"
    );
    assert!(input.parents().is_empty(), "{origins:?}: unfiltered source");
    assert!(
        original.contains("missing(metrics)") || original.contains("defined(metrics)"),
        "the authored predicate reaches the coverage suite: {original}"
    );
    assert!(ess_conformance::defined_aggregates::used_by(
        &ir,
        input.selected().suite()
    ));
    assert_eq!(
        ess_conformance::defined_aggregates::COVERAGE,
        27,
        "the detected construct retains its historical coverage minimum"
    );
    assert_eq!(version(&original), "ess-conformance/35", "{origins:?}");
    let ids: Vec<_> = input
        .selected()
        .suite()
        .scenarios
        .keys()
        .filter(|id| id.to_string().contains("a-fresh-queue-holds-no-metrics"))
        .cloned()
        .collect();
    assert_eq!(ids.len(), 1, "the authored scenario is selected");
    let narrowed = input.select(&ids).unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(
        version(narrowed.selected().original_json()),
        "ess-conformance/35"
    );
    assert_eq!(
        narrowed
            .selected()
            .suite()
            .provenance
            .scenario_initial_state,
        Some(ScenarioInitialState::Empty)
    );
    assert_eq!(
        narrowed
            .selected()
            .suite()
            .scenarios
            .keys()
            .cloned()
            .collect::<Vec<_>>(),
        ids,
        "{origins:?}: selection retains exactly the authored id"
    );
    assert_eq!(
        narrowed.document().parent_suites,
        vec![original],
        "{origins:?}: selection retains exact full parent lineage"
    );
}

#[test]
fn adversary_w1fix_pass2_an_authored_presence_predicate_writes_fresh_coverage_35() {
    check(Origins::Authored);
}

#[test]
fn adversary_w1fix_pass2_an_authored_presence_predicate_beside_generated_writes_fresh_coverage_35()
{
    check(Origins::GeneratedAndAuthored);
}
