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

/// A view declaring `absent:` (ess/25, beyond10x/ess#533) answers its default row for an identity
/// no row carries, and the stored row for one that a row carries: the identity is the requested
/// value, each literal typed by its field, the `Optional` field left out of `row:` absent.
#[test]
fn interpreter_answers_the_declared_default_row_for_an_unstored_identity() {
    use ess_conformance::target::{
        ConformanceTarget, Deadline, EventObservationRequest, SemanticCommandRequest,
        SemanticViewRequest,
    };
    use ess_primitives::{
        consistency::QueryConsistency, facts::Number, ids::CorrelationId, node::Node,
        time::Timestamp,
    };
    use std::collections::BTreeMap;
    let spec = Specification::assemble([(
        Source::new("view-absent-row.yaml"),
        RawSpecFile::parse(include_str!("fixtures/view-absent-row.yaml")).unwrap(),
    )])
    .unwrap();
    let target = Interpreted::for_model(compile(&spec, &SourceMap::new()).unwrap());
    let correlation = CorrelationId::new("absent").unwrap();
    let deadline = || Deadline::at(Timestamp::from_epoch_millis(0));
    target
        .execute_command(SemanticCommandRequest {
            command: "demo.status.SetFlag".parse().unwrap(),
            actor: None,
            caller: None,
            input: BTreeMap::from([("level".into(), Node::Number(Number::new(3.0).unwrap()))]),
            correlation: correlation.clone(),
        })
        .unwrap();
    let stored = target
        .observe_events(EventObservationRequest {
            event: "demo.status.FlagSet".parse().unwrap(),
            correlation: correlation.clone(),
            deadline: deadline(),
        })
        .unwrap()[0]
        .payload["id"]
        .clone();
    let read = |id: Node| {
        target
            .query_view(SemanticViewRequest {
                view: "demo.status.FlagById".parse().unwrap(),
                params: BTreeMap::from([("id".into(), id)]),
                consistency: QueryConsistency::Current,
                correlation: correlation.clone(),
                deadline: deadline(),
            })
            .unwrap()
            .rows
    };
    let unstored = Node::Text("00000000-0000-4000-8000-0000000005aa".into());
    assert_ne!(stored, unstored);
    assert_eq!(
        read(unstored.clone()),
        [BTreeMap::from([
            ("id".to_owned(), unstored),
            ("paused".to_owned(), Node::Bool(false)),
            ("level".to_owned(), Node::Number(Number::new(0.0).unwrap())),
            ("state".to_owned(), Node::Text("Set".into())),
            ("note".to_owned(), Node::Null),
        ])],
        "the default row"
    );
    let rows = read(stored.clone());
    assert_eq!(rows.len(), 1, "{rows:?}");
    assert_eq!(rows[0]["id"], stored);
    assert_eq!(rows[0]["level"], Node::Number(Number::new(3.0).unwrap()));
}

/// A view ranked by a `Timestamp` descending, ties broken by a newtype over `Integer`.
const RANKED: &str = r"format: ess/16
system: ranked
version: v1
domain: ranked.api
types:
  - {name: ranked.api.Rank, kind: newtype, of: Integer}
entities:
  - name: ranked.api.Entry
    identity: {name: id, type: Uuid}
    fields: [{name: stamp, type: Timestamp}, {name: rank, type: ranked.api.Rank}]
    lifecycle: {initial: Stored, states: [Stored], terminal: [Stored], transitions: []}
events:
  - name: ranked.api.Stored
    fields: [{name: id, type: Uuid}]
commands:
  - name: ranked.api.Store
    input: [{name: stamp, type: Timestamp}, {name: rank, type: ranked.api.Rank}]
    outcomes:
      - name: stored
        creates: ranked.api.Entry
        instance: id
        sets: {stamp: input.stamp, rank: input.rank}
        emits: [ranked.api.Stored]
        payload: {ranked.api.Stored: {id: {generated: true}}}
views:
  - name: ranked.api.Entries
    source: ranked.api.Entry
    consistency: read_your_writes
    order_by: [stamp desc, rank]
    fields: [{name: id, type: Uuid}, {name: stamp, type: Timestamp}, {name: rank, type: ranked.api.Rank}]
";

fn ranked_row(id: &str, stamp: &str, rank: i64) -> ess_conformance::target::ViewRow {
    use ess_primitives::node::Node;
    std::collections::BTreeMap::from([
        ("id".to_owned(), Node::Text(id.into())),
        ("stamp".to_owned(), Node::Text(stamp.into())),
        ("rank".to_owned(), Node::Number(rank.into())),
    ])
}

/// A consumer ordering rows it holds calls the interpreter's own ordering and gets the order the
/// interpreted target answers the view in: instants, not spellings, for the `Timestamp`, and the
/// number under the newtype for the tie-break.
#[test]
fn a_consumer_orders_view_rows_as_the_interpreter_answers_them() {
    use ess_conformance::interpret::values::order_view_rows;
    use ess_conformance::target::{
        ConformanceTarget, Deadline, SemanticCommandRequest, SemanticViewRequest,
    };
    use ess_primitives::{
        consistency::QueryConsistency, ids::CorrelationId, node::Node, time::Timestamp,
    };
    use std::collections::BTreeMap;
    let spec = Specification::assemble([(
        Source::new("ranked.yaml"),
        RawSpecFile::parse(RANKED).unwrap(),
    )])
    .unwrap();
    let ir = compile(&spec, &SourceMap::new()).unwrap();
    let target = Interpreted::for_model(ir.clone());
    let correlation = CorrelationId::new("ranked").unwrap();
    // Two instants, each spelled twice, and ranks whose text order is not their numeric order.
    let stored = [
        ("2026-01-05T09:00:00Z", 10_i64),
        ("2026-01-05T11:00:00+02:00", 9),
        ("2026-01-05T10:00:00Z", 100),
        ("2026-01-05T09:00:00Z", 2),
        ("2026-01-05T12:00:00+02:00", 20),
        ("2026-01-05T09:00:00Z", 1000),
    ];
    for (stamp, rank) in stored {
        target
            .execute_command(SemanticCommandRequest {
                command: "ranked.api.Store".parse().unwrap(),
                actor: None,
                caller: None,
                input: BTreeMap::from([
                    ("stamp".into(), Node::Text(stamp.into())),
                    ("rank".into(), Node::Number(rank.into())),
                ]),
                correlation: correlation.clone(),
            })
            .unwrap();
    }
    let answered = target
        .query_view(SemanticViewRequest {
            view: "ranked.api.Entries".parse().unwrap(),
            params: BTreeMap::new(),
            consistency: QueryConsistency::Current,
            correlation,
            deadline: Deadline::at(Timestamp::from_epoch_millis(0)),
        })
        .unwrap()
        .rows;
    assert_eq!(answered.len(), stored.len());
    let ranks: Vec<_> = answered.iter().map(|row| row["rank"].clone()).collect();
    assert_eq!(
        ranks,
        [20, 100, 2, 9, 10, 1000].map(|rank: i64| Node::Number(rank.into())),
        "the interpreter's own answer: {answered:#?}"
    );
    let view = &ir.views()[&"ranked.api.Entries"
        .parse::<ess_domain::name::QualifiedName>()
        .unwrap()];
    for shift in 0..answered.len() {
        let mut shuffled = answered.clone();
        shuffled.reverse();
        shuffled.rotate_left(shift);
        order_view_rows(&ir, view, &mut shuffled);
        assert_eq!(shuffled, answered, "rotated by {shift}");
    }
}

/// Ties on every `order_by` key are left where the caller put them: the order is a stable sort.
#[test]
fn rows_tied_on_every_order_key_keep_the_callers_order() {
    use ess_conformance::interpret::values::order_view_rows;
    let spec = Specification::assemble([(
        Source::new("ranked.yaml"),
        RawSpecFile::parse(RANKED).unwrap(),
    )])
    .unwrap();
    let ir = compile(&spec, &SourceMap::new()).unwrap();
    let view = &ir.views()[&"ranked.api.Entries"
        .parse::<ess_domain::name::QualifiedName>()
        .unwrap()];
    let tied = |id: &str| ranked_row(id, "2026-01-05T09:00:00Z", 5);
    // The same instant spelled another way ties too.
    let respelled = ranked_row("d", "2026-01-05T11:00:00+02:00", 5);
    let first = ranked_row("first", "2026-01-06T00:00:00Z", 1);
    let last = ranked_row("last", "2026-01-05T09:00:00Z", 6);
    for order in [["a", "b", "c"], ["c", "a", "b"], ["b", "c", "a"]] {
        let mut rows = vec![
            last.clone(),
            tied(order[0]),
            respelled.clone(),
            tied(order[1]),
            first.clone(),
            tied(order[2]),
        ];
        order_view_rows(&ir, view, &mut rows);
        assert_eq!(
            rows,
            [
                first.clone(),
                tied(order[0]),
                respelled.clone(),
                tied(order[1]),
                tied(order[2]),
                last.clone(),
            ],
            "{order:?}"
        );
    }
}
