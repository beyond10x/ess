//! Actual subject snapshots and exact arithmetic, independent of synthesized expectations.
use ess_compiler::{resolve::compile, source::SourceMap};
use ess_conformance::{interpret::Interpreted, target::*};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::{
    consistency::QueryConsistency, facts::Number, ids::CorrelationId, node::Node, time::Timestamp,
};
use std::collections::BTreeMap;

const MODEL: &str = r"format: ess/20
system: demo
version: v1
domain: demo.values
types:
  - name: demo.values.Packet
    kind: struct
    fields: [{name: identity, type: IDTYPE}, {name: count, type: Integer}, {name: note, type: 'Optional<String>'}]
entities:
  - name: demo.values.Row
    identity: {name: id, type: IDTYPE}
    fields: [{name: count, type: Integer}, {name: mirror, type: Integer}, {name: score, type: Decimal}, {name: note, type: 'Optional<String>'}, {name: packet, type: 'Optional<demo.values.Packet>'}]
    lifecycle: {initial: Held, states: [Held], terminal: [Held]}
events:
  - name: demo.values.Updated
    fields: [{name: packet, type: demo.values.Packet}, {name: score, type: Decimal}]
commands:
  - name: demo.values.Update
    input: [{name: id, type: IDTYPE}]
    outcomes:
      - name: updated
        updates: demo.values.Row
        instance: id
        sets:
          count: {increment: 1}
          mirror: {subject: count}
          score: {increment: '-0.25'}
          note: {cleared: true}
          packet: {identity: {subject: id}, count: {subject: count}, note: {subject: note}}
        emits: [demo.values.Updated]
        payload:
          demo.values.Updated: {packet: {identity: {subject: id}, count: {subject: count}, note: {subject: note}}, score: {subject: score}}
views:
  - name: demo.values.Rows
    source: demo.values.Row
    consistency: read_your_writes
    fields: [{name: id, type: IDTYPE}, {name: count, type: Integer}, {name: mirror, type: Integer}, {name: score, type: Decimal}, {name: note, type: 'Optional<String>'}, {name: packet, type: 'Optional<demo.values.Packet>'}]
";

fn correlation() -> CorrelationId {
    CorrelationId::new("subject-values").unwrap()
}
fn number(value: i64) -> Node {
    Node::Number(value.into())
}
fn decimal(value: &str) -> Node {
    Node::Number(Number::decimal_literal(value).unwrap())
}
fn target(source: &str, id: &Node, count: i64, note: Option<&str>) -> Interpreted {
    let spec = Specification::assemble([(
        Source::new("values.yaml"),
        RawSpecFile::parse(source).unwrap(),
    )])
    .unwrap();
    let target = Interpreted::for_model(compile(&spec, &SourceMap::new()).unwrap());
    target
        .begin_scenario(&ScenarioContext::new(
            "demo.values/authored/snapshot".parse().unwrap(),
            correlation(),
        ))
        .unwrap();
    let mut fields = BTreeMap::from([
        ("count".into(), number(count)),
        ("mirror".into(), number(-1)),
        ("score".into(), decimal("0.5")),
    ]);
    if let Some(note) = note {
        fields.insert("note".into(), Node::Text(note.into()));
    }
    target
        .establish_entity(EntitySetupRequest {
            entity: "demo.values.Row".parse().unwrap(),
            identity: id.clone(),
            fields,
            state: "Held".parse().unwrap(),
            correlation: correlation(),
        })
        .unwrap();
    target
}
fn update(target: &Interpreted, id: &Node) -> Result<SemanticCommandResult, TargetError> {
    target.execute_command(SemanticCommandRequest {
        command: "demo.values.Update".parse().unwrap(),
        actor: None,
        caller: None,
        input: BTreeMap::from([("id".into(), id.clone())]),
        correlation: correlation(),
    })
}
fn rows(target: &Interpreted) -> Vec<ViewRow> {
    target
        .query_view(SemanticViewRequest {
            view: "demo.values.Rows".parse().unwrap(),
            params: BTreeMap::new(),
            consistency: QueryConsistency::Current,
            correlation: correlation(),
            deadline: Deadline::at(Timestamp::from_epoch_millis(0)),
        })
        .unwrap()
        .rows
}

#[test]
fn nested_event_and_assignment_read_the_same_pre_outcome_snapshot() {
    for id in [
        number(9_007_199_254_740_993),
        Node::Map(BTreeMap::from([("part".into(), number(7))])),
    ] {
        let source = MODEL.replace("IDTYPE", "Json");
        let target = target(&source, &id, 9_007_199_254_740_993, Some("before"));
        let result = update(&target, &id).unwrap();
        let expected = Node::Map(BTreeMap::from([
            ("identity".into(), id.clone()),
            ("count".into(), number(9_007_199_254_740_993)),
            ("note".into(), Node::Text("before".into())),
        ]));
        assert_eq!(result.direct_events[0].payload["packet"], expected);
        assert_eq!(result.direct_events[0].payload["score"], decimal("0.5"));
        let row = &rows(&target)[0];
        assert_eq!(row["count"], number(9_007_199_254_740_994));
        assert_eq!(row["mirror"], number(9_007_199_254_740_993));
        assert_eq!(row["score"], decimal("0.25"));
        assert_eq!(row["packet"], expected);
        assert_eq!(row["note"], Node::Null);
    }
}

#[test]
fn absent_optional_subject_leaf_stays_absent() {
    for wrapped in [false, true] {
        let mut source = MODEL.replace("IDTYPE", "Integer");
        if wrapped {
            source = source
                .replace("Optional<String>", "demo.values.Note")
                .replace("          note: {cleared: true}\n", "")
                .replace(
                    "types:\n",
                    "types:\n  - {name: demo.values.Note, kind: newtype, of: 'Optional<String>'}\n",
                );
        }
        let id = number(7);
        let target = target(&source, &id, 2, None);
        let result = update(&target, &id).unwrap();
        let Node::Map(packet) = &result.direct_events[0].payload["packet"] else {
            panic!("packet")
        };
        assert!(!packet.contains_key("note"));
        assert_eq!(packet["identity"], id);
        assert_eq!(packet["count"], number(2));
    }
}

#[test]
fn required_subject_reads_never_use_an_earlier_write_or_guess_a_missing_value() {
    use ess_conformance::interpret::execute::{execute, Externals, Store};
    for assignment in ["{increment: 1}", "1"] {
        let source = MODEL
            .replace("IDTYPE", "Integer")
            .replace("count: {increment: 1}", &format!("count: {assignment}"))
            .replace(
                "events:\n",
                "events:\n  - {name: demo.values.Created, fields: [{name: id, type: Integer}]}\n",
            )
            .replace(
                "commands:\n",
                "commands:\n  - name: demo.values.Create\n    input: []\n    outcomes:\n      - name: created\n        creates: demo.values.Row\n        instance: id\n        sets: {mirror: 0, score: '0.5'}\n        emits: [demo.values.Created]\n        payload: {demo.values.Created: {id: {generated: true}}}\n",
            );
        let spec = Specification::assemble([(
            Source::new("missing.yaml"),
            RawSpecFile::parse(&source).unwrap(),
        )])
        .unwrap();
        let ir = compile(&spec, &SourceMap::new()).unwrap();
        let created = execute(
            &ir,
            &Store::default(),
            &"demo.values.Create".parse().unwrap(),
            &BTreeMap::new(),
            &Externals::Withheld,
        )
        .unwrap()
        .remove(0);
        let id = created.events[0].payload["id"].clone();
        let row = created
            .next
            .instance_typed(&"demo.values.Row".parse().unwrap(), &id)
            .unwrap();
        assert!(!row.fields.contains_key("count"));
        let before = created.next.clone();
        let error = execute(
            &ir,
            &created.next,
            &"demo.values.Update".parse().unwrap(),
            &BTreeMap::from([("id".into(), id)]),
            &Externals::Withheld,
        )
        .unwrap_err();
        assert!(error.to_string().contains("count"), "{error}");
        assert_eq!(created.next, before);
    }
}

#[test]
fn overflowing_and_constrained_increments_leave_every_field_unchanged() {
    for (count, constrained) in [(i64::MAX, false), (10, true)] {
        let mut source = MODEL.replace("IDTYPE", "Integer");
        if constrained {
            source = source.replace(
                "{name: count, type: Integer}",
                "{name: count, type: demo.values.Count}",
            );
            source = source.replace(
                "{name: mirror, type: Integer}",
                "{name: mirror, type: demo.values.Count}",
            );
            source = source.replace("types:\n", "types:\n  - {name: demo.values.Count, kind: newtype, of: Integer, invariants: ['value <= 10']}\n");
        }
        let id = number(7);
        let target = target(&source, &id, count, Some("preserved"));
        let before = rows(&target);
        let error = update(&target, &id).unwrap_err();
        assert!(
            !error.to_string().contains("not interpreted yet"),
            "{error}"
        );
        assert_eq!(rows(&target), before);
        assert!(target
            .observe_events(EventObservationRequest {
                event: "demo.values.Updated".parse().unwrap(),
                correlation: correlation(),
                deadline: Deadline::at(Timestamp::from_epoch_millis(0))
            })
            .unwrap()
            .is_empty());
    }
}

#[test]
fn an_at_rest_invariant_failure_publishes_neither_partial_writes_nor_events() {
    let source = MODEL.replace("IDTYPE", "Integer").replace(
        "    lifecycle:",
        "    invariants: ['count <= 10']\n    lifecycle:",
    );
    let id = number(7);
    let target = target(&source, &id, 10, Some("preserved"));
    let before = rows(&target);
    let error = update(&target, &id).unwrap_err();
    assert!(error.to_string().contains("invariant"), "{error}");
    assert_eq!(rows(&target), before);
    assert!(target
        .observe_events(EventObservationRequest {
            event: "demo.values.Updated".parse().unwrap(),
            correlation: correlation(),
            deadline: Deadline::at(Timestamp::from_epoch_millis(0)),
        })
        .unwrap()
        .is_empty());
}

#[test]
fn a_wrong_state_error_reads_the_typed_identity_from_the_addressed_row() {
    use ess_conformance::interpret::execute::{execute, Externals, Store};
    let source = include_str!("fixtures/error-payload-sources.yaml")
        .replace("of: Uuid", "of: Integer")
        .replace(
            "{order_id: input.order_id, quantity: {subject: quantity}}",
            "{order_id: {subject: order_id}, quantity: {subject: quantity}}",
        );
    let spec = Specification::assemble([(
        Source::new("errors.yaml"),
        RawSpecFile::parse(&source).unwrap(),
    )])
    .unwrap();
    let ir = compile(&spec, &SourceMap::new()).unwrap();
    let created = execute(
        &ir,
        &Store::default(),
        &"demo.order.PlaceOrder".parse().unwrap(),
        &BTreeMap::from([("quantity".into(), number(7))]),
        &Externals::Withheld,
    )
    .unwrap()
    .remove(0);
    let id = created.events[0].payload["order_id"].clone();
    let invoke = |store| {
        execute(
            &ir,
            store,
            &"demo.order.CloseOrder".parse().unwrap(),
            &BTreeMap::from([("order_id".into(), id.clone())]),
            &Externals::Withheld,
        )
        .unwrap()
        .remove(0)
    };
    let closed = invoke(&created.next);
    let refused = invoke(&closed.next);
    assert_eq!(refused.error.unwrap().fields["order_id"], id);
    assert_eq!(refused.next, closed.next);
    assert!(refused.events.is_empty());
}

#[test]
fn negative_integer_and_positive_decimal_increments_are_exact() {
    let source = MODEL
        .replace("IDTYPE", "Integer")
        .replace("count: {increment: 1}", "count: {increment: -2}")
        .replace("score: {increment: '-0.25'}", "score: {increment: '0.25'}");
    let id = number(7);
    let target = target(&source, &id, -5, None);
    update(&target, &id).unwrap();
    let row = &rows(&target)[0];
    assert_eq!(row["count"], number(-7));
    assert_eq!(row["score"], decimal("0.75"));
}
