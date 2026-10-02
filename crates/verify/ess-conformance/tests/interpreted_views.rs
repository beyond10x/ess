//! The actual interpreter computes source-declared views over its own command state.
use ess_compiler::{resolve::compile, source::SourceMap};
use ess_conformance::{interpret::Interpreted, report::Status, AdmittedSuite, Runner, ScenarioId};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
fn run(source: &str, aggregate: bool) {
    let spec = Specification::assemble([(
        Source::new("model.yaml"),
        RawSpecFile::parse(source).unwrap(),
    )])
    .unwrap();
    let ir = compile(&spec, &SourceMap::new()).unwrap();
    let mut synthesis = ess_conformance::synthesize(&ir);
    if aggregate {
        synthesis
            .suite
            .scenarios
            .retain(|id, _| matches!(id, ScenarioId::Aggregate { .. }));
    }
    assert!(
        !synthesis.suite.scenarios.is_empty(),
        "{:?}",
        synthesis.refusals
    );
    let input = AdmittedSuite::from_suite(&synthesis.suite).unwrap();
    let report = Runner::for_suite(input.suite()).run_admitted(&input, &Interpreted::for_model(ir));
    let failures: Vec<_> = report
        .scenarios
        .iter()
        .filter(|scenario| scenario.status != Status::Passed)
        .collect();
    assert!(failures.is_empty(), "{failures:?}");
}
#[test]
fn all_source_aggregate_functions_execute() {
    run(include_str!("fixtures/aggregate-views.yaml"), true);
}
#[test]
fn source_paging_filter_and_total_execute() {
    run(include_str!("fixtures/view-paging.yaml"), false);
}

const TIMED: &str = r"format: ess/16
system: timed
version: v1
domain: timed.api
entities:
  - name: timed.api.Entry
    identity: {name: id, type: Uuid}
    fields: [{name: stamp, type: Timestamp}]
    lifecycle: {initial: Stored, states: [Stored], terminal: [Stored], transitions: []}
events:
  - name: timed.api.Stored
    fields: [{name: id, type: Uuid}]
commands:
  - name: timed.api.Store
    input: [{name: stamp, type: Timestamp}]
    outcomes:
      - name: stored
        creates: timed.api.Entry
        instance: id
        sets: {stamp: input.stamp}
        emits: [timed.api.Stored]
        payload: {timed.api.Stored: {id: {generated: true}}}
views:
  - name: timed.api.Entries
    source: timed.api.Entry
    consistency: read_your_writes
    params: [{name: before, type: 'Optional<Timestamp>'}]
    filter: {any: ['not defined(param.before)', 'stamp < param.before']}
    order_by: [stamp desc]
    fields: [{name: id, type: Uuid}, {name: stamp, type: Timestamp}]
";
fn timed_query(
    before: Option<ess_primitives::node::Node>,
) -> Vec<ess_conformance::target::ViewRow> {
    use ess_conformance::target::{
        ConformanceTarget, Deadline, SemanticCommandRequest, SemanticViewRequest,
    };
    use ess_primitives::{
        consistency::QueryConsistency, ids::CorrelationId, node::Node, time::Timestamp,
    };
    use std::collections::BTreeMap;
    let spec = Specification::assemble([(
        Source::new("timed.yaml"),
        RawSpecFile::parse(TIMED).unwrap(),
    )])
    .unwrap();
    let target = Interpreted::for_model(compile(&spec, &SourceMap::new()).unwrap());
    let correlation = CorrelationId::new("timed").unwrap();
    for stamp in ["2026-01-05T09:00:03Z", "2026-01-05T10:00:01+02:00"] {
        target
            .execute_command(SemanticCommandRequest {
                command: "timed.api.Store".parse().unwrap(),
                actor: None,
                caller: None,
                input: BTreeMap::from([("stamp".into(), Node::Text(stamp.into()))]),
                correlation: correlation.clone(),
            })
            .unwrap();
    }
    target
        .query_view(SemanticViewRequest {
            view: "timed.api.Entries".parse().unwrap(),
            params: before
                .map(|value| BTreeMap::from([("before".into(), value)]))
                .unwrap_or_default(),
            consistency: QueryConsistency::Current,
            correlation,
            deadline: Deadline::at(Timestamp::from_epoch_millis(0)),
        })
        .unwrap()
        .rows
}
#[test]
fn timestamp_order_uses_instants_and_preserves_offset_spelling() {
    let rows = timed_query(Some(ess_primitives::node::Node::Null));
    assert_eq!(rows[0]["stamp"].as_text(), Some("2026-01-05T09:00:03Z"));
    assert_eq!(
        rows[1]["stamp"].as_text(),
        Some("2026-01-05T10:00:01+02:00")
    );
}
#[test]
fn timestamp_filter_uses_declared_instant_order() {
    let rows = timed_query(Some(ess_primitives::node::Node::Text(
        "2026-01-05T09:00:02Z".into(),
    )));
    assert_eq!(rows.len(), 1);
    assert_eq!(
        rows[0]["stamp"].as_text(),
        Some("2026-01-05T10:00:01+02:00")
    );
}
#[test]
fn absent_optional_view_parameter_is_an_absent_fact() {
    assert_eq!(timed_query(None).len(), 2);
}

#[test]
fn required_group_key_is_not_invented_when_the_source_never_sets_it() {
    use ess_conformance::target::{
        ConformanceTarget, Deadline, SemanticCommandRequest, SemanticViewRequest, TargetError,
    };
    use ess_primitives::{
        consistency::QueryConsistency, ids::CorrelationId, node::Node, time::Timestamp,
    };
    use std::collections::BTreeMap;
    let source = TIMED
        .replace("Timestamp", "String")
        .replace("        sets: {stamp: input.stamp}\n", "")
        .replace("    params: [{name: before, type: 'Optional<String>'}]\n", "")
        .replace("    filter: {any: ['not defined(param.before)', 'stamp < param.before']}\n", "")
        .replace("    order_by: [stamp desc]\n", "    group_by: [stamp]\n")
        .replace("    fields: [{name: id, type: Uuid}, {name: stamp, type: String}]", "    fields: [{name: stamp, type: String}, {name: count, type: Integer, aggregate: {count: {}}}]");
    let spec = Specification::assemble([(
        Source::new("missing-group.yaml"),
        RawSpecFile::parse(&source).unwrap(),
    )])
    .unwrap();
    let target = Interpreted::for_model(compile(&spec, &SourceMap::new()).unwrap());
    let correlation = CorrelationId::new("missing-group").unwrap();
    target
        .execute_command(SemanticCommandRequest {
            command: "timed.api.Store".parse().unwrap(),
            actor: None,
            caller: None,
            input: BTreeMap::from([("stamp".into(), Node::Text("2026-01-05T09:00:03Z".into()))]),
            correlation: correlation.clone(),
        })
        .unwrap();
    let answer = target.query_view(SemanticViewRequest {
        view: "timed.api.Entries".parse().unwrap(),
        params: BTreeMap::new(),
        consistency: QueryConsistency::Current,
        correlation,
        deadline: Deadline::at(Timestamp::from_epoch_millis(0)),
    });
    assert!(
        matches!(answer, Err(TargetError::Unsupported { .. })),
        "{answer:?}"
    );
}
