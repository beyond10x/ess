//! Typed identities remain values throughout actual setup, commands and observations.
use ess_compiler::{resolve::compile, source::SourceMap, EssIr};
use ess_conformance::{interpret::Interpreted, target::*};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::{
    consistency::QueryConsistency, ids::CorrelationId, node::Node, time::Timestamp,
};
use std::collections::BTreeMap;

const MODEL: &str = r"format: ess/20
system: demo
version: v1
domain: demo.ids
entities:
  - name: demo.ids.Row
    identity: {name: id, type: IDTYPE}
    fields: [{name: note, type: String}]
    lifecycle: {initial: Held, states: [Held], terminal: [Held]}
events:
  - {name: demo.ids.Created, fields: [{name: id, type: IDTYPE}]}
  - {name: demo.ids.Changed, fields: []}
commands:
  - name: demo.ids.Create
    input: [{name: note, type: String}]
    outcomes:
      - name: created
        creates: demo.ids.Row
        instance: id
        sets: {note: input.note}
        emits: [demo.ids.Created]
        payload: {demo.ids.Created: {id: {generated: true}}}
  - name: demo.ids.Update
    input: [{name: id, type: IDTYPE}, {name: note, type: String}]
    outcomes:
      - name: updated
        updates: demo.ids.Row
        instance: id
        sets: {note: input.note}
        emits: [demo.ids.Changed]
views:
  - name: demo.ids.Rows
    source: demo.ids.Row
    consistency: read_your_writes
    fields: [{name: id, type: IDTYPE}, {name: note, type: String}]
";
fn model(source: &str) -> EssIr {
    let spec =
        Specification::assemble([(Source::new("ids.yaml"), RawSpecFile::parse(source).unwrap())])
            .unwrap();
    compile(&spec, &SourceMap::new()).unwrap()
}
fn context() -> ScenarioContext {
    ScenarioContext::new(
        "demo.ids/authored/typed".parse().unwrap(),
        CorrelationId::new("typed").unwrap(),
    )
}
fn target(source: &str) -> Interpreted {
    let target = Interpreted::for_model(model(source));
    target.begin_scenario(&context()).unwrap();
    target
}
fn text(value: &str) -> Node {
    Node::Text(value.into())
}
fn setup(target: &Interpreted, identity: Node, note: &str) -> Result<(), TargetError> {
    target.establish_entity(EntitySetupRequest {
        entity: "demo.ids.Row".parse().unwrap(),
        identity,
        fields: BTreeMap::from([("note".into(), text(note))]),
        state: "Held".parse().unwrap(),
        correlation: context().correlation,
    })
}
fn rows(target: &Interpreted) -> Vec<ViewRow> {
    target
        .query_view(SemanticViewRequest {
            view: "demo.ids.Rows".parse().unwrap(),
            params: BTreeMap::new(),
            consistency: QueryConsistency::Current,
            correlation: context().correlation,
            deadline: Deadline::at(Timestamp::from_epoch_millis(0)),
        })
        .unwrap()
        .rows
}
fn invoke(
    target: &Interpreted,
    command: &str,
    input: BTreeMap<String, Node>,
) -> Result<SemanticCommandResult, TargetError> {
    target.execute_command(SemanticCommandRequest {
        command: format!("demo.ids.{command}").parse().unwrap(),
        actor: None,
        caller: None,
        input,
        correlation: context().correlation,
    })
}

#[test]
fn setup_and_update_preserve_admitted_scalar_and_structured_identity_values() {
    for (kind, ids) in [
        (
            "Integer",
            vec![
                Node::Number(9_007_199_254_740_992_i64.into()),
                Node::Number(9_007_199_254_740_993_i64.into()),
            ],
        ),
        ("Boolean", vec![Node::Bool(false), Node::Bool(true)]),
        (
            "Json",
            vec![
                text("7"),
                Node::Number(7_i64.into()),
                Node::Seq(vec![Node::Bool(true)]),
                Node::Map(BTreeMap::from([(
                    "part".into(),
                    Node::Number(7_i64.into()),
                )])),
            ],
        ),
    ] {
        let target = target(&MODEL.replace("IDTYPE", kind));
        for (index, id) in ids.iter().enumerate() {
            setup(&target, id.clone(), &format!("before{index}")).unwrap();
        }
        assert_eq!(rows(&target).len(), ids.len());
        for id in &ids {
            let result = invoke(
                &target,
                "Update",
                BTreeMap::from([("id".into(), id.clone()), ("note".into(), text("after"))]),
            )
            .unwrap();
            assert_eq!(
                result.outcome.unwrap().to_string(),
                "demo.ids.Update/updated"
            );
            let observed = rows(&target);
            assert_eq!(observed.iter().filter(|row| row["id"] == *id).count(), 1);
            assert_eq!(
                observed.iter().find(|row| row["id"] == *id).unwrap()["note"],
                text("after")
            );
        }
        target.begin_scenario(&context()).unwrap();
        assert_eq!(rows(&target).len(), 0);
    }
}

#[test]
fn typed_duplicates_and_invalid_setup_never_replace_existing_rows() {
    let target = target(&MODEL.replace("IDTYPE", "Integer"));
    setup(&target, Node::Number(7_i64.into()), "original").unwrap();
    let before = rows(&target);
    for id in [
        serde_json::from_str::<Node>("7.0").unwrap(),
        text("7"),
        Node::Null,
    ] {
        assert!(setup(&target, id, "replacement").is_err());
        assert_eq!(rows(&target), before);
    }
}

#[test]
fn constrained_identity_setup_validates_before_storage_and_eventual_projection() {
    let source = MODEL.replace("IDTYPE", "demo.ids.Id").replace("read_your_writes", "eventual")
        + "\ntypes:\n  - {name: demo.ids.Id, kind: newtype, of: Integer, invariants: ['value >= 10', 'value <= 12']}\n";
    let target = target(&source);
    setup(&target, Node::Number(10_i64.into()), "kept").unwrap();
    assert_eq!(rows(&target)[0]["id"], Node::Number(10_i64.into()));
    let before = rows(&target);
    assert!(setup(&target, Node::Number(9_i64.into()), "bad").is_err());
    assert_eq!(rows(&target), before);
}

#[test]
fn generated_numeric_creation_skips_held_witness_values_without_overwrite() {
    let source = MODEL.replace("IDTYPE", "Integer");
    let witness = target(&source);
    let first = invoke(
        &witness,
        "Create",
        BTreeMap::from([("note".into(), text("first"))]),
    )
    .unwrap()
    .direct_events[0]
        .payload["id"]
        .clone();
    assert!(matches!(first, Node::Number(_)));
    let target = target(&source);
    setup(&target, first.clone(), "reserved").unwrap();
    for _ in 0..3 {
        invoke(
            &target,
            "Create",
            BTreeMap::from([("note".into(), text("new"))]),
        )
        .unwrap();
    }
    let observed = rows(&target);
    assert_eq!(observed.len(), 4);
    assert_eq!(
        observed.iter().find(|row| row["id"] == first).unwrap()["note"],
        text("reserved")
    );
    assert_eq!(
        observed
            .iter()
            .map(|row| row["id"].clone())
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        4
    );
}

fn query(target: &Interpreted, view: &str) -> Vec<ViewRow> {
    target
        .query_view(SemanticViewRequest {
            view: view.parse().unwrap(),
            params: BTreeMap::new(),
            consistency: QueryConsistency::Current,
            correlation: context().correlation,
            deadline: Deadline::at(Timestamp::from_epoch_millis(0)),
        })
        .unwrap()
        .rows
}
fn command(
    target: &Interpreted,
    name: &str,
    input: BTreeMap<String, Node>,
) -> SemanticCommandResult {
    target
        .execute_command(SemanticCommandRequest {
            command: name.parse().unwrap(),
            actor: None,
            caller: None,
            input,
            correlation: context().correlation,
        })
        .unwrap()
}
#[test]
fn numeric_related_guards_select_the_addressed_row_among_decoys() {
    let source = include_str!("fixtures/related-guard-sign-in.yaml").replace(
        "name: demo.signin.TenantId, kind: newtype, of: Uuid",
        "name: demo.signin.TenantId, kind: newtype, of: Integer",
    );
    let target = target(&source);
    for (id, client) in [(7_i64, "match"), (8_i64, "decoy")] {
        target
            .establish_entity(EntitySetupRequest {
                entity: "demo.signin.Configuration".parse().unwrap(),
                identity: Node::Number(id.into()),
                fields: BTreeMap::from([("redirect_client".into(), text(client))]),
                state: "Active".parse().unwrap(),
                correlation: context().correlation,
            })
            .unwrap();
    }
    for (id, expected) in [
        (7_i64, "initiated"),
        (8_i64, "no-redirect-entry"),
        (9_i64, "no-configuration"),
    ] {
        let result = command(
            &target,
            "demo.signin.InitiateSignIn",
            BTreeMap::from([
                ("tenant".into(), Node::Number(id.into())),
                ("client".into(), text("match")),
            ]),
        );
        assert_eq!(
            result.outcome.unwrap().to_string(),
            format!("demo.signin.InitiateSignIn/{expected}")
        );
    }
    let rows = query(&target, "demo.signin.SignIns");
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0]["tenant"], Node::Number(7_i64.into()));
}

#[test]
fn numeric_filtered_and_secondary_effects_preserve_decoys_and_exclude_primary() {
    let source = include_str!("../../../specify/ess-compiler/tests/fixtures/set-effects.yaml")
        .replace(
            "name: demo.desk.SessionId, kind: newtype, of: String",
            "name: demo.desk.SessionId, kind: newtype, of: Integer",
        );
    let target = target(&source);
    for (id, team) in [(1_i64, "one"), (2, "one"), (3, "two")] {
        target
            .establish_entity(EntitySetupRequest {
                entity: "demo.desk.Session".parse().unwrap(),
                identity: Node::Number(id.into()),
                fields: BTreeMap::from([
                    ("team".into(), text(team)),
                    ("note".into(), text("before")),
                    ("on_hold".into(), Node::Bool(false)),
                ]),
                state: "Open".parse().unwrap(),
                correlation: context().correlation,
            })
            .unwrap();
    }
    let result = command(
        &target,
        "demo.desk.NoteTeam",
        BTreeMap::from([("team".into(), text("one")), ("note".into(), text("after"))]),
    );
    assert_eq!(
        result.direct_events[0].payload["noted"],
        Node::Number(2_i64.into())
    );
    command(
        &target,
        "demo.desk.Invite",
        BTreeMap::from([("session_id".into(), Node::Number(1_i64.into()))]),
    );
    let rows = query(&target, "demo.desk.SessionDetails");
    assert_eq!(rows.len(), 3);
    for row in rows {
        let id = &row["session_id"];
        assert_eq!(
            row["on_hold"],
            Node::Bool(*id == Node::Number(2_i64.into()))
        );
        assert_eq!(
            row["note"],
            text(if *id == Node::Number(3_i64.into()) {
                "before"
            } else {
                "after"
            })
        );
    }
}

fn history(ir: &EssIr, view: bool) -> ess_conformance::history::History {
    let digest = ess_conformance::scenario::SuiteProvenance::of(ir).spec_digest;
    let operation = if view {
        serde_json::json!({
            "operation_id":"00000000-0000-4000-8000-000000000001", "client":0,
            "command":"demo.ids.Rows", "subject_key":"", "rows":["7"], "outcome":"read",
            "invoked_at":1,"returned_at":2,"completion":"Returned"
        })
    } else {
        serde_json::json!({
            "operation_id":"00000000-0000-4000-8000-000000000001", "client":0,
            "command":"demo.ids.Create", "subject_key":"7", "outcome":"created",
            "invoked_at":1,"returned_at":2,"completion":"Returned"
        })
    };
    let mut value: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/register/linearizable.json")).unwrap();
    value["spec_digest"] = serde_json::to_value(&digest).unwrap();
    value["operations"] = serde_json::json!([operation]);
    ess_conformance::history::read(&serde_json::to_vec(&value).unwrap(), &digest).unwrap()
}
#[test]
fn string_history_never_guesses_a_nontext_subject_identity() {
    for kind in ["Integer", "Boolean"] {
        let ir = model(&MODEL.replace("IDTYPE", kind));
        let error = ess_conformance::linearize::check(&ir, &history(&ir, false), 100).unwrap_err();
        assert!(error.to_string().contains("ess-history/1"), "{error}");
        let checked = ess_conformance::linearize::check(&ir, &history(&ir, true), 100).unwrap();
        assert_eq!(checked.not_judged.len(), 1);
        assert!(checked.not_judged[0].reason.contains("ess-history/1"));
    }
    for kind in ["String", "Json"] {
        let ir = model(&MODEL.replace("IDTYPE", kind));
        assert_eq!(
            ess_conformance::linearize::check(&ir, &history(&ir, false), 100)
                .unwrap()
                .verdict,
            ess_conformance::history::Verdict::Linearizable
        );
    }
}

#[test]
fn generated_numeric_identity_invariant_is_checked_before_committing() {
    use ess_conformance::interpret::execute::{
        execute_generating, Externals, Generated, GeneratedSlot, Store, Undetermined,
    };
    let source = MODEL.replace("IDTYPE", "Integer").replace(
        "    fields: [{name: note, type: String}]",
        "    fields: [{name: note, type: String}]\n    invariants: ['id > 0']",
    );
    let ir = model(&source);
    let store = Store::default();
    let result = execute_generating(
        &ir,
        &store,
        &"demo.ids.Create".parse().unwrap(),
        &BTreeMap::from([("note".into(), text("kept"))]),
        &Externals::Withheld,
        &Generated::Given(BTreeMap::from([(
            GeneratedSlot::new("demo.ids.Created".parse().unwrap(), "id"),
            Node::Number((-1_i64).into()),
        )])),
    );
    assert!(
        matches!(result, Err(Undetermined::BrokenInvariant { .. })),
        "{result:?}"
    );
    assert_eq!(store.instances().count(), 0);
}

#[test]
fn exhausted_boolean_generation_refuses_without_changing_rows() {
    let target = target(&MODEL.replace("IDTYPE", "Boolean"));
    for _ in 0..2 {
        invoke(
            &target,
            "Create",
            BTreeMap::from([("note".into(), text("kept"))]),
        )
        .unwrap();
    }
    let before = rows(&target);
    assert_eq!(before.len(), 2);
    assert!(invoke(
        &target,
        "Create",
        BTreeMap::from([("note".into(), text("extra"))])
    )
    .unwrap_err()
    .is_unsupported());
    assert_eq!(rows(&target), before);
}

#[test]
fn typed_delete_and_filtered_view_keep_the_other_identity() {
    let source = MODEL.replace("IDTYPE", "Integer").replace("views:\n", "  - name: demo.ids.Delete\n    input: [{name: id, type: Integer}]\n    outcomes:\n      - {name: deleted, deletes: demo.ids.Row, instance: id}\nviews:\n");
    let source = source.replace(
        "    consistency: read_your_writes",
        "    consistency: read_your_writes\n    filter: id >= 7",
    );
    let target = target(&source);
    for id in [6_i64, 7, 8] {
        setup(&target, Node::Number(id.into()), "kept").unwrap();
    }
    assert_eq!(rows(&target).len(), 2);
    let result = invoke(
        &target,
        "Delete",
        BTreeMap::from([("id".into(), Node::Number(7_i64.into()))]),
    )
    .unwrap();
    assert_eq!(
        result.outcome.unwrap().to_string(),
        "demo.ids.Delete/deleted"
    );
    assert_eq!(rows(&target).len(), 1);
    assert_eq!(rows(&target)[0]["id"], Node::Number(8_i64.into()));
}

#[test]
fn all_row_iteration_keeps_typed_values_and_given_collisions_are_atomic() {
    use ess_conformance::interpret::execute::{
        execute_generating, Externals, Generated, GeneratedSlot, Store,
    };
    let ir = model(&MODEL.replace("IDTYPE", "Json"));
    let mut store = Store::default();
    let values = [
        text("7"),
        Node::Number(7_i64.into()),
        Node::Bool(true),
        Node::Seq(vec![text("nested")]),
    ];
    for id in &values {
        let generated = Generated::Given(BTreeMap::from([(
            GeneratedSlot::new("demo.ids.Created".parse().unwrap(), "id"),
            id.clone(),
        )]));
        let steps = execute_generating(
            &ir,
            &store,
            &"demo.ids.Create".parse().unwrap(),
            &BTreeMap::from([("note".into(), text("kept"))]),
            &Externals::Withheld,
            &generated,
        )
        .unwrap();
        assert_eq!(steps.len(), 1);
        store = steps[0].next.clone();
        let duplicate = execute_generating(
            &ir,
            &store,
            &"demo.ids.Create".parse().unwrap(),
            &BTreeMap::from([("note".into(), text("replace"))]),
            &Externals::Withheld,
            &generated,
        );
        assert!(duplicate.is_err());
    }
    let actual: std::collections::BTreeSet<_> = store
        .instances()
        .map(|(_, key, _)| serde_json::to_string(key).unwrap())
        .collect();
    assert_eq!(
        actual,
        values
            .iter()
            .map(|value| serde_json::to_string(value).unwrap())
            .collect()
    );
    assert_eq!(
        store
            .instance(&"demo.ids.Row".parse().unwrap(), "7")
            .unwrap()
            .fields["note"],
        text("kept")
    );
}

#[test]
fn numeric_json_history_evidence_is_rejected_without_text_coercion() {
    let ir = model(&MODEL.replace("IDTYPE", "Json"));
    let digest = ess_conformance::scenario::SuiteProvenance::of(&ir).spec_digest;
    for view in [false, true] {
        let mut value = serde_json::to_value(history(&ir, view)).unwrap();
        if view {
            value["operations"][0]["rows"] = serde_json::json!([7]);
        } else {
            value["operations"][0]["subject_key"] = serde_json::json!(7);
        }
        assert!(
            ess_conformance::history::read(&serde_json::to_vec(&value).unwrap(), &digest).is_err()
        );
    }
}
