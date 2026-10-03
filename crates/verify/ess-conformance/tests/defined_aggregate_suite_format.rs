//! `defined()` or `missing()` over an `Optional` aggregate requires the round-3 reading
//! (beyond10x/ess#176). Fresh suites now also declare isolation in /34 (coverage /35).
//!
//! A runner from 0.37.0 binds no fact at a struct's or a list's own path, so it reads
//! `defined(metrics)` as `false` for a queue that holds metrics: the #176 invariant
//! `any: [state == Paused, not defined(metrics)]` then passes on every row without checking it.
//! The number moves so that such a runner refuses the suite by version instead. The same predicate
//! over an `Optional` scalar does not require the aggregate reading.
use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::{
    scenario::ViewExpectation, synthesize::synthesize, AdmittedSuite, ConformanceSuite,
    ScenarioStep,
};
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source,
};

const QUEUE: &str = include_str!("fixtures/defined-over-optional-aggregates.yaml");

fn edit(text: &str, from: &str, to: &str) -> String {
    assert!(text.contains(from), "fixture edit did not land: {from}");
    text.replace(from, to)
}

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("queue.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn suite(ir: &EssIr) -> ConformanceSuite {
    let synthesis = synthesize(ir);
    assert!(
        synthesis.refusals.is_empty(),
        "nothing is refused: {:#?}",
        synthesis.refusals
    );
    synthesis.suite
}

/// Whether some view expectation of the suite carries a predicate reading `defined(metrics)`.
fn carries_defined(suite: &ConformanceSuite) -> bool {
    suite.scenarios.values().any(|scenario| {
        scenario.steps.iter().any(|step| match step {
            ScenarioStep::ExpectView {
                expectation: ViewExpectation::Satisfies { predicate },
                ..
            }
            | ScenarioStep::EventuallyView {
                expectation: ViewExpectation::Satisfies { predicate },
                ..
            } => predicate.to_string().contains("defined(metrics)"),
            _ => false,
        })
    })
}

fn coverage_document(ir: &EssIr) -> String {
    ess_conformance::coverage_build::build(
        ir,
        &[],
        ess_conformance::coverage::Scope::System,
        ess_conformance::coverage::Origins::Generated,
    )
    .unwrap_or_else(|error| panic!("{error}"))
    .selected()
    .original_json()
    .to_owned()
}

/// The queue with `metrics` an `Optional<Integer>` instead of an `Optional` struct.
fn scalar_metrics() -> String {
    let text = edit(
        QUEUE,
        "{name: metrics, type: Optional<demo.queue.Metrics>}",
        "{name: metrics, type: Optional<Integer>}",
    );
    edit(
        &text,
        "{name: metrics, type: demo.queue.Metrics}",
        "{name: metrics, type: Integer}",
    )
}

/// The queue with `metrics` an `Optional<List<Integer>>`.
fn list_metrics() -> String {
    let text = edit(
        QUEUE,
        "{name: metrics, type: Optional<demo.queue.Metrics>}",
        "{name: metrics, type: Optional<List<Integer>>}",
    );
    edit(
        &text,
        "{name: metrics, type: demo.queue.Metrics}",
        "{name: metrics, type: List<Integer>}",
    )
}

#[test]
fn defined_over_an_optional_struct_selects_suite_26_and_its_coverage_27() {
    let ir = ir(QUEUE);
    let suite = suite(&ir);
    assert!(carries_defined(&suite), "the invariant reaches the suite");
    assert!(ess_conformance::defined_aggregates::used_by(&ir, &suite));
    assert_eq!(
        suite.provenance.suite_version.to_string(),
        "ess-conformance/34"
    );
    let original = coverage_document(&ir);
    assert!(original.contains("\"ess-conformance/35\""), "{original}");
    AdmittedSuite::from_json(&original).unwrap_or_else(|error| panic!("{error}"));
}

#[test]
fn defined_over_an_optional_list_selects_suite_26() {
    let ir = ir(&list_metrics());
    let suite = suite(&ir);
    assert!(carries_defined(&suite), "the invariant reaches the suite");
    assert!(ess_conformance::defined_aggregates::used_by(&ir, &suite));
    assert_eq!(
        suite.provenance.suite_version.to_string(),
        "ess-conformance/34"
    );
}

#[test]
fn missing_over_an_optional_struct_selects_suite_26() {
    let text = edit(QUEUE, "{not: \"defined(metrics)\"}", "\"missing(metrics)\"");
    let ir = ir(&text);
    let suite = suite(&ir);
    assert!(ess_conformance::defined_aggregates::used_by(&ir, &suite));
    assert_eq!(
        suite.provenance.suite_version.to_string(),
        "ess-conformance/34"
    );
}

#[test]
fn defined_over_an_optional_scalar_keeps_fresh_selection_idempotent() {
    let ir = ir(&scalar_metrics());
    let suite = suite(&ir);
    assert!(carries_defined(&suite), "the invariant reaches the suite");
    assert!(!ess_conformance::defined_aggregates::used_by(&ir, &suite));
    let major = suite.provenance.suite_version.major();
    assert_eq!(major, 34);
    let mut reselected = suite.clone();
    reselected.select_fresh_format();
    assert_eq!(
        reselected.to_canonical_json().expect("serializes"),
        suite.to_canonical_json().expect("serializes"),
        "the suite is exactly what the construct-free selection writes"
    );
    let original = coverage_document(&ir);
    assert!(original.contains("\"ess-conformance/35\""), "{original}");
}

#[test]
fn a_view_filter_alone_is_decided_at_synthesis_without_aggregate_predicate_vocabulary() {
    // Without the invariant, `defined(metrics)` is read only by a view filter. Synthesis decides
    // which rows that view shows and the suite carries the rows, not the predicate, so a runner of
    // any age compares them correctly: nothing in the suite asks for the new reading.
    let text = edit(
        QUEUE,
        "    invariants:\n      - any: [state == Paused, {not: \"defined(metrics)\"}]\n",
        "",
    );
    let text = format!(
        "{text}  - name: demo.queue.WithMetrics\n    source: demo.queue.Queue\n    consistency: read_your_writes\n    filter: 'defined(metrics)'\n    fields:\n      - {{name: queue_id, type: demo.queue.QueueId}}\n      - {{name: metrics, type: Optional<demo.queue.Metrics>}}\n"
    );
    let ir = ir(&text);
    let suite = suite(&ir);
    assert!(!carries_defined(&suite));
    assert!(!ess_conformance::defined_aggregates::used_by(&ir, &suite));
    assert_eq!(suite.provenance.suite_version.major(), 34);
}
