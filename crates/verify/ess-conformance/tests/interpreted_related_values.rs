//! Actual native related-value reads retain typed addresses and the complete original snapshot.
use ess_compiler::{resolve::compile, source::SourceMap, EssIr};
use ess_conformance::{interpret::Interpreted, target::*, AdmittedSuite, Runner};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::{
    consistency::QueryConsistency, ids::CorrelationId, node::Node, time::Timestamp,
};
use std::collections::BTreeMap;

const MODEL: &str = r"format: ess/20
system: demo
version: v1
domain: demo.related
types:
  - {name: demo.related.RootId, kind: newtype, of: IDTYPE}
  - {name: demo.related.LinkId, kind: newtype, of: Integer}
  - name: demo.related.Packet
    kind: struct
    fields: [{name: identity, type: demo.related.RootId}, {name: note, type: 'Optional<String>'}, {name: data, type: Json}]
entities:
  - name: demo.related.Root
    identity: {name: id, type: demo.related.RootId}
    fields: [{name: note, type: 'Optional<String>'}, {name: data, type: Json}]
    lifecycle: {initial: Held, states: [Held], terminal: [Held]}
  - name: demo.related.Link
    identity: {name: id, type: demo.related.LinkId}
    fields: [{name: packet, type: 'Optional<demo.related.Packet>'}, {name: root_id, type: demo.related.RootId}]
    relations: [{name: root, kind: references, target: demo.related.Root, cardinality: one, via: root_id}]
    lifecycle: {initial: Held, states: [Held], terminal: [Held]}
errors:
  - {name: demo.related.Denied, fields: []}
  - {name: demo.related.Rejected, fields: [{name: packet, type: demo.related.Packet}]}
events:
  - {name: demo.related.Created, fields: [{name: id, type: demo.related.LinkId}, {name: packet, type: demo.related.Packet}]}
  - {name: demo.related.Changed, fields: [{name: packet, type: demo.related.Packet}]}
  - {name: demo.related.RootChanged, fields: [{name: data, type: Json}]}
commands:
  - name: demo.related.Create
    input: [{name: root_id, type: demo.related.RootId}]
    outcomes:
      - name: created
        creates: demo.related.Link
        instance: id
        sets:
          root_id: input.root_id
          packet: {identity: {related: {via: CREATIONVIA, field: id}}, note: {related: {via: CREATIONVIA, field: note}}, data: {related: {via: CREATIONVIA, field: data}}}
        emits: [demo.related.Created]
        payload:
          demo.related.Created:
            id: {generated: true}
            packet: {identity: {related: {via: CREATIONVIA, field: id}}, note: {related: {via: CREATIONVIA, field: note}}, data: {related: {via: CREATIONVIA, field: data}}}
  - name: demo.related.Update
    input: [{name: id, type: demo.related.LinkId}, {name: next, type: demo.related.RootId}, {name: deny, type: Boolean}]
    outcomes:
      - {name: denied, when: deny == true, error: demo.related.Denied}
      - name: updated
        updates: demo.related.Link
        instance: id
        sets:
          root_id: input.next
          packet: {identity: {related: {via: root_id, field: id}}, note: {related: {via: root_id, field: note}}, data: {related: {via: root_id, field: data}}}
        emits: [demo.related.Changed]
        payload:
          demo.related.Changed: {packet: {identity: {related: {via: root_id, field: id}}, note: {related: {via: root_id, field: note}}, data: {related: {via: root_id, field: data}}}}
  - name: demo.related.ChangeRoot
    input: [{name: id, type: demo.related.RootId}, {name: data, type: Json}]
    outcomes:
      - name: changed
        updates: demo.related.Root
        instance: id
        sets: {data: input.data}
        emits: [demo.related.RootChanged]
        payload: {demo.related.RootChanged: {data: {related: {via: input.id, field: data}}}}
  - name: demo.related.Reject
    input: [{name: root_id, type: demo.related.RootId}, {name: deny, type: Boolean}]
    outcomes:
      - name: rejected
        when: deny == true
        error: demo.related.Rejected
        payload:
          demo.related.Rejected: {packet: {identity: {related: {via: input.root_id, field: id}}, note: {related: {via: input.root_id, field: note}}, data: {related: {via: input.root_id, field: data}}}}
      - {name: denied, error: demo.related.Denied}
views:
  - name: demo.related.Links
    source: demo.related.Link
    consistency: read_your_writes
    fields: [{name: id, type: demo.related.LinkId}, {name: root_id, type: demo.related.RootId}, {name: packet, type: 'Optional<demo.related.Packet>'}]
";
fn model(source: &str) -> EssIr {
    let spec = Specification::assemble([(
        Source::new("related.yaml"),
        RawSpecFile::parse(source).unwrap(),
    )])
    .unwrap();
    compile(&spec, &SourceMap::new()).unwrap()
}
fn source(kind: &str, via: &str) -> String {
    MODEL.replace("IDTYPE", kind).replace("CREATIONVIA", via)
}
fn number(n: i64) -> Node {
    Node::Number(n.into())
}
fn fields(pairs: &[(&str, Node)]) -> BTreeMap<String, Node> {
    pairs
        .iter()
        .map(|(k, v)| ((*k).into(), v.clone()))
        .collect()
}
fn correlation() -> CorrelationId {
    CorrelationId::new("related").unwrap()
}
fn target(source: &str) -> Interpreted {
    let target = Interpreted::for_model(model(source));
    target
        .begin_scenario(&ScenarioContext::new(
            "demo.related/authored/read".parse().unwrap(),
            correlation(),
        ))
        .unwrap();
    target
}
fn setup(target: &Interpreted, entity: &str, identity: Node, values: BTreeMap<String, Node>) {
    target
        .establish_entity(EntitySetupRequest {
            entity: format!("demo.related.{entity}").parse().unwrap(),
            identity,
            fields: values,
            state: "Held".parse().unwrap(),
            correlation: correlation(),
        })
        .unwrap();
}
fn invoke(
    target: &Interpreted,
    command: &str,
    input: BTreeMap<String, Node>,
) -> Result<SemanticCommandResult, TargetError> {
    target.execute_command(SemanticCommandRequest {
        command: format!("demo.related.{command}").parse().unwrap(),
        actor: None,
        caller: None,
        input,
        correlation: correlation(),
    })
}
fn links(target: &Interpreted) -> Vec<ViewRow> {
    target
        .query_view(SemanticViewRequest {
            view: "demo.related.Links".parse().unwrap(),
            params: BTreeMap::new(),
            consistency: QueryConsistency::Current,
            correlation: correlation(),
            deadline: Deadline::at(Timestamp::from_epoch_millis(0)),
        })
        .unwrap()
        .rows
}
fn published(target: &Interpreted, event: &str) -> Vec<ObservedEvent> {
    target
        .observe_events(EventObservationRequest {
            event: format!("demo.related.{event}").parse().unwrap(),
            correlation: correlation(),
            deadline: Deadline::at(Timestamp::from_epoch_millis(0)),
        })
        .unwrap()
}

#[test]
fn typed_related_reads_keep_the_old_reference_while_the_subject_changes_it() {
    for (kind, old, next) in [
        (
            "Integer",
            number(9_007_199_254_740_992),
            number(9_007_199_254_740_993),
        ),
        (
            "Json",
            Node::Map(fields(&[("key", number(1))])),
            Node::Map(fields(&[("key", number(2))])),
        ),
    ] {
        let target = target(&source(kind, "input.root_id"));
        for id in [&old, &next] {
            setup(
                &target,
                "Root",
                id.clone(),
                fields(&[("note", Node::Text("same".into())), ("data", Node::Null)]),
            );
        }
        setup(
            &target,
            "Link",
            number(77),
            fields(&[("root_id", old.clone())]),
        );
        let result = invoke(
            &target,
            "Update",
            fields(&[
                ("id", number(77)),
                ("next", next.clone()),
                ("deny", Node::Bool(false)),
            ]),
        )
        .unwrap();
        let expected = Node::Map(fields(&[
            ("identity", old),
            ("note", Node::Text("same".into())),
            ("data", Node::Null),
        ]));
        assert_eq!(result.direct_events[0].payload["packet"], expected);
        let rows = links(&target);
        assert_eq!(rows[0]["root_id"], next);
        assert_eq!(rows[0]["packet"], expected);
    }
}

#[test]
fn input_and_creation_subject_carriers_read_optional_and_null_fields() {
    use ess_compiler::ir::{ResolvedPayloadValue, ResolvedRelatedVia};
    for via in ["root_id", "input.root_id"] {
        let source = source("Integer", via);
        let ir = model(&source);
        let command = &ir.commands()[&"demo.related.Create".parse().unwrap()];
        let packet = command.outcomes[0]
            .sets
            .iter()
            .find(|set| set.target == "packet")
            .unwrap();
        let ResolvedPayloadValue::Struct { fields: members } = &packet.value else {
            panic!("packet")
        };
        assert!(members.iter().all(|member| matches!(
            &member.value,
            ResolvedPayloadValue::RelatedField {
                via: ResolvedRelatedVia::Subject { .. },
                ..
            }
        ) == (via == "root_id")));
        for note in [None, Some(Node::Null), Some(Node::Text("present".into()))] {
            let target = target(&source);
            let mut row = fields(&[("data", Node::Null)]);
            if let Some(value) = &note {
                row.insert("note".into(), value.clone());
            }
            setup(&target, "Root", number(11), row.clone());
            let result = invoke(&target, "Create", fields(&[("root_id", number(11))])).unwrap();
            row.insert("identity".into(), number(11));
            let expected = Node::Map(row);
            assert_eq!(result.direct_events[0].payload["packet"], expected);
            assert_eq!(links(&target)[0]["packet"], expected);
        }
    }
}

#[test]
fn updating_the_related_row_still_emits_its_original_value() {
    let target = target(&source("Integer", "input.root_id"));
    setup(
        &target,
        "Root",
        number(11),
        fields(&[("data", Node::Text("before".into()))]),
    );
    let result = invoke(
        &target,
        "ChangeRoot",
        fields(&[("id", number(11)), ("data", Node::Text("after".into()))]),
    )
    .unwrap();
    assert_eq!(
        result.direct_events[0].payload["data"],
        Node::Text("before".into())
    );
    let again = invoke(
        &target,
        "ChangeRoot",
        fields(&[("id", number(11)), ("data", Node::Null)]),
    )
    .unwrap();
    assert_eq!(
        again.direct_events[0].payload["data"],
        Node::Text("after".into())
    );
}

#[test]
fn a_missing_related_row_is_not_an_absent_optional_field_and_failure_is_atomic() {
    let target = target(&source("Integer", "input.root_id"));
    setup(
        &target,
        "Link",
        number(77),
        fields(&[("root_id", number(11))]),
    );
    let before = links(&target);
    let error = invoke(
        &target,
        "Update",
        fields(&[
            ("id", number(77)),
            ("next", number(22)),
            ("deny", Node::Bool(false)),
        ]),
    )
    .unwrap_err();
    assert!(error.to_string().contains("related"), "{error}");
    assert_eq!(links(&target), before);
    assert!(published(&target, "Changed").is_empty());
    let refused = invoke(
        &target,
        "Update",
        fields(&[
            ("id", number(77)),
            ("next", number(22)),
            ("deny", Node::Bool(true)),
        ]),
    )
    .unwrap();
    assert_eq!(
        refused.error.unwrap().error.to_string(),
        "demo.related.Denied"
    );
    assert_eq!(links(&target), before);
}

#[test]
fn a_nested_error_reads_the_exact_related_row_without_writing() {
    let target = target(&source("Integer", "input.root_id"));
    setup(
        &target,
        "Root",
        number(11),
        fields(&[("data", Node::Bool(true))]),
    );
    let result = invoke(
        &target,
        "Reject",
        fields(&[("root_id", number(11)), ("deny", Node::Bool(true))]),
    )
    .unwrap();
    assert_eq!(
        result.error.unwrap().fields["packet"],
        Node::Map(fields(&[
            ("identity", number(11)),
            ("data", Node::Bool(true))
        ]))
    );
    assert!(result.direct_events.is_empty());
    assert!(links(&target).is_empty());
}

#[test]
fn actual_guard_and_copied_value_fixture_suites_execute_without_dropping_assertions() {
    for source in [
        include_str!("fixtures/related-guard-copied-value.yaml"),
        include_str!("fixtures/related-copied-view-parameter.yaml"),
        include_str!("fixtures/subject-guard-copied-field.yaml"),
    ] {
        let ir = model(source);
        let synthesis = ess_conformance::synthesize::synthesize(&ir);
        let admitted = AdmittedSuite::from_suite(&synthesis.suite).unwrap();
        let run = Runner::for_suite(admitted.suite())
            .run_admitted(&admitted, &Interpreted::for_model(ir));
        assert!(!run.scenarios.is_empty());
        assert!(
            run.scenarios
                .iter()
                .all(|result| result.status == ess_conformance::report::Status::Passed),
            "{:#?}",
            run.scenarios
        );
    }
}

const IDENTITY_CARRIER: &str = r"format: ess/20
system: demo
version: v1
domain: demo.related
types:
  - {name: demo.related.Id, kind: newtype, of: Integer}
entities:
  - name: demo.related.Root
    identity: {name: id, type: demo.related.Id}
    fields: [{name: data, type: Json}]
    lifecycle: {initial: Held, states: [Held], terminal: [Held]}
  - name: demo.related.Link
    identity: {name: id, type: demo.related.Id}
    fields: [{name: data, type: Json}]
    relations: [{name: root, kind: references, target: demo.related.Root, cardinality: one, via: id}]
    lifecycle: {initial: Held, states: [Held], terminal: [Held]}
events:
  - {name: demo.related.Created, fields: [{name: id, type: demo.related.Id}, {name: data, type: Json}]}
commands:
  - name: demo.related.Create
    input: [{name: id, type: demo.related.Id}]
    outcomes:
      - name: created
        creates: demo.related.Link
        instance: id
        sets: {data: {related: {via: id, field: data}}}
        emits: [demo.related.Created]
        payload: {demo.related.Created: {id: input.id, data: {related: {via: id, field: data}}}}
";
#[test]
fn creation_can_read_through_its_input_supplied_identity_reference() {
    use ess_compiler::ir::{ResolvedPayloadValue, ResolvedRelatedVia};
    let ir = model(IDENTITY_CARRIER);
    let command = &ir.commands()[&"demo.related.Create".parse().unwrap()];
    assert!(matches!(
        &command.outcomes[0].sets[0].value,
        ResolvedPayloadValue::RelatedField { via: ResolvedRelatedVia::Subject { field, .. }, .. } if field == "id"
    ));
    let target = target(IDENTITY_CARRIER);
    let id = number(9_007_199_254_740_993);
    setup(
        &target,
        "Root",
        id.clone(),
        fields(&[("data", Node::Text("root".into()))]),
    );
    let result = invoke(&target, "Create", fields(&[("id", id.clone())])).unwrap();
    assert_eq!(
        result.direct_events[0].payload,
        fields(&[("id", id), ("data", Node::Text("root".into()))])
    );
}

#[test]
fn optional_only_reads_distinguish_a_missing_row_from_an_absent_member() {
    for wrapped in [false, true] {
        let mut source = source("Integer", "input.root_id")
            .replace("commands:\n", "  - {name: demo.related.OptionalRead, fields: [{name: note, type: 'Optional<String>'}]}\ncommands:\n")
            .replace("views:\n", "  - name: demo.related.ReadOptional\n    input: [{name: root_id, type: demo.related.RootId}]\n    outcomes:\n      - name: read\n        emits: [demo.related.OptionalRead]\n        payload: {demo.related.OptionalRead: {note: {related: {via: input.root_id, field: note}}}}\nviews:\n");
        if wrapped {
            source = source
                .replace("Optional<String>", "demo.related.Note")
                .replace(
                "types:\n",
                "types:\n  - {name: demo.related.Note, kind: newtype, of: 'Optional<String>'}\n",
            );
        }
        let target = target(&source);
        assert!(invoke(&target, "ReadOptional", fields(&[("root_id", number(11))])).is_err());
        assert!(published(&target, "OptionalRead").is_empty());
        setup(&target, "Root", number(11), fields(&[("data", Node::Null)]));
        let result = invoke(&target, "ReadOptional", fields(&[("root_id", number(11))])).unwrap();
        assert!(result.direct_events[0].payload.is_empty());
    }
}

#[test]
fn related_reads_do_not_publish_writes_or_events_when_a_later_constraint_fails() {
    let source = source("Integer", "input.root_id").replace(
        "    relations: [{name: root,",
        "    invariants: ['root_id <= 20']\n    relations: [{name: root,",
    );
    let target = target(&source);
    setup(&target, "Root", number(11), fields(&[("data", Node::Null)]));
    setup(
        &target,
        "Link",
        number(77),
        fields(&[("root_id", number(11))]),
    );
    let before = links(&target);
    let error = invoke(
        &target,
        "Update",
        fields(&[
            ("id", number(77)),
            ("next", number(22)),
            ("deny", Node::Bool(false)),
        ]),
    )
    .unwrap_err();
    assert!(error.to_string().contains("invariant"), "{error}");
    assert_eq!(links(&target), before);
    assert!(published(&target, "Changed").is_empty());
}

#[test]
fn wrong_state_errors_read_the_related_row_the_subject_currently_references() {
    let source = source("Integer", "input.root_id")
        .replace("via: root_id}]\n    lifecycle: {initial: Held, states: [Held], terminal: [Held]}", "via: root_id}]\n    lifecycle: {initial: Held, states: [Held, Done], terminal: [Done], transitions: [{name: finish, from: [Held], to: Done}]}")
        .replace("        updates: demo.related.Link", "        moves: demo.related.Link.finish")
        .replace("  - name: demo.related.ChangeRoot", "      - name: wrong-state\n        wrong_state: true\n        error: demo.related.Rejected\n        payload: {demo.related.Rejected: {packet: {identity: {related: {via: root_id, field: id}}, note: {related: {via: root_id, field: note}}, data: {related: {via: root_id, field: data}}}}}\n  - name: demo.related.ChangeRoot");
    let target = target(&source);
    for (id, label) in [(11, "first"), (22, "second")] {
        setup(
            &target,
            "Root",
            number(id),
            fields(&[("data", Node::Text(label.into()))]),
        );
    }
    setup(
        &target,
        "Link",
        number(77),
        fields(&[("root_id", number(11))]),
    );
    let input = fields(&[
        ("id", number(77)),
        ("next", number(22)),
        ("deny", Node::Bool(false)),
    ]);
    invoke(&target, "Update", input.clone()).unwrap();
    let before = links(&target);
    let result = invoke(&target, "Update", input).unwrap();
    assert_eq!(
        result.error.unwrap().fields["packet"],
        Node::Map(fields(&[
            ("identity", number(22)),
            ("data", Node::Text("second".into()))
        ]))
    );
    assert_eq!(links(&target), before);
    assert!(result.direct_events.is_empty());
    assert_eq!(published(&target, "Changed").len(), 1);
}

#[test]
fn a_stored_guard_refusal_reads_related_values_through_its_selected_subject() {
    let source = source("Integer", "input.root_id").replace(
        "      - name: updated\n",
        "      - name: refused-by-row\n        when_subject: {predicate: root_id == 11}\n        error: demo.related.Rejected\n        payload: {demo.related.Rejected: {packet: {identity: {related: {via: root_id, field: id}}, note: {related: {via: root_id, field: note}}, data: {related: {via: root_id, field: data}}}}}\n      - name: updated\n",
    );
    let target = target(&source);
    setup(
        &target,
        "Root",
        number(11),
        fields(&[("data", Node::Bool(true))]),
    );
    setup(
        &target,
        "Link",
        number(77),
        fields(&[("root_id", number(11))]),
    );
    let before = links(&target);
    let result = invoke(
        &target,
        "Update",
        fields(&[
            ("id", number(77)),
            ("next", number(22)),
            ("deny", Node::Bool(false)),
        ]),
    )
    .unwrap();
    assert_eq!(
        result.error.unwrap().fields["packet"],
        Node::Map(fields(&[
            ("identity", number(11)),
            ("data", Node::Bool(true))
        ]))
    );
    assert_eq!(links(&target), before);
    assert!(result.direct_events.is_empty());
}

#[test]
fn a_related_required_value_unknown_to_the_store_is_never_guessed() {
    let source = source("Integer", "input.root_id")
        .replace("commands:\n", "  - {name: demo.related.RootCreated, fields: [{name: id, type: demo.related.RootId}]}\ncommands:\n")
        .replace("views:\n", "  - name: demo.related.CreateRoot\n    outcomes:\n      - name: created\n        creates: demo.related.Root\n        instance: id\n        emits: [demo.related.RootCreated]\n        payload: {demo.related.RootCreated: {id: {generated: true}}}\nviews:\n");
    let target = target(&source);
    let created = invoke(&target, "CreateRoot", BTreeMap::new()).unwrap();
    let id = created.direct_events[0].payload["id"].clone();
    let error = invoke(&target, "Create", fields(&[("root_id", id)])).unwrap_err();
    assert!(error.to_string().contains("data"), "{error}");
    assert!(links(&target).is_empty());
    assert!(published(&target, "Created").is_empty());
}

#[test]
fn an_unknown_original_reference_is_not_replaced_by_the_new_assignment() {
    let source = source("Integer", "input.root_id")
        .replace("          root_id: input.root_id\n", "")
        .replace(
            "{name: id, type: demo.related.LinkId}, {name: root_id, type: demo.related.RootId}, {name: packet, type: 'Optional<demo.related.Packet>'}",
            "{name: id, type: demo.related.LinkId}, {name: packet, type: 'Optional<demo.related.Packet>'}",
        );
    let target = target(&source);
    setup(&target, "Root", number(11), fields(&[("data", Node::Null)]));
    let created = invoke(&target, "Create", fields(&[("root_id", number(11))])).unwrap();
    let id = created.direct_events[0].payload["id"].clone();
    let before = links(&target);
    assert_eq!(before.len(), 1);
    let error = invoke(
        &target,
        "Update",
        fields(&[
            ("id", id),
            ("next", number(11)),
            ("deny", Node::Bool(false)),
        ]),
    )
    .unwrap_err();
    assert!(error.to_string().contains("subject.root_id"), "{error}");
    assert_eq!(links(&target), before);
    assert!(published(&target, "Changed").is_empty());
}
