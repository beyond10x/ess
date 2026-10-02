//! Actual sequential-model set effects, including no-match, snapshot and write boundaries.
use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::interpret::execute::{
    execute, execute_generating, Externals, Generated, GeneratedSlot, Step, Store, Undetermined,
};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::node::Node;
use std::collections::BTreeMap;

const MODEL: &str = include_str!("../../../specify/ess-compiler/tests/fixtures/set-effects.yaml");
fn model(text: &str) -> EssIr {
    let spec =
        Specification::assemble([(Source::new("sets.yaml"), RawSpecFile::parse(text).unwrap())])
            .unwrap();
    compile(&spec, &SourceMap::new()).unwrap()
}
fn values<const N: usize>(entries: [(&str, Node); N]) -> BTreeMap<String, Node> {
    entries
        .into_iter()
        .map(|(name, value)| (name.into(), value))
        .collect()
}
fn text(value: &str) -> Node {
    Node::Text(value.into())
}
fn invoke(
    ir: &EssIr,
    store: &mut Store,
    command: &str,
    input: &BTreeMap<String, Node>,
) -> Result<Step, Undetermined> {
    let mut steps = execute(
        ir,
        store,
        &format!("demo.desk.{command}").parse().unwrap(),
        input,
        &Externals::Withheld,
    )?;
    assert_eq!(steps.len(), 1);
    let step = steps.remove(0);
    *store = step.next.clone();
    Ok(step)
}
fn open(ir: &EssIr, store: &mut Store, id: &str, team: &str, note: &str) {
    let mut steps = execute_generating(
        ir,
        store,
        &"demo.desk.Open".parse().unwrap(),
        &values([("team", text(team)), ("note", text(note))]),
        &Externals::Withheld,
        &Generated::Given(BTreeMap::from([(
            GeneratedSlot::new("demo.desk.SessionOpened".parse().unwrap(), "session_id"),
            text(id),
        )])),
    )
    .unwrap();
    assert_eq!(steps.len(), 1);
    *store = steps.remove(0).next;
}
fn row<'a>(store: &'a Store, id: &str) -> &'a ess_conformance::interpret::execute::Instance {
    store
        .instance(&"demo.desk.Session".parse().unwrap(), id)
        .unwrap()
}
fn count(step: &Step, field: &str) -> Node {
    step.events[0].payload[field].clone()
}

#[test]
fn an_excluded_primary_does_not_need_the_secondary_filter_facts() {
    let source = MODEL
        .replace("          team: input.team\n", "")
        .replace(
            "  - name: demo.desk.Invite\n    input:\n",
            "  - name: demo.desk.Invite\n    input:\n      - {name: team, type: demo.desk.Team}\n",
        )
        .replace("where: team == subject.team", "where: team == input.team");
    let ir = model(&source);
    let mut store = Store::default();
    open(&ir, &mut store, "primary", "one", "kept");
    assert!(!row(&store, "primary").fields.contains_key("team"));
    let input = values([("session_id", text("primary")), ("team", text("one"))]);
    let result = invoke(&ir, &mut store, "Invite", &input).unwrap();
    assert_eq!(
        result.outcome.unwrap().to_string(),
        "demo.desk.Invite/invited"
    );
    assert_eq!(result.events.len(), 1);
    assert_eq!(result.events[0].payload["session_id"], text("primary"));
    assert_eq!(row(&store, "primary").fields["note"], text("kept"));
    assert_eq!(row(&store, "primary").fields["on_hold"], Node::Bool(false));

    // The same missing fact on an eligible secondary row remains undecidable.
    open(&ir, &mut store, "secondary", "one", "kept too");
    let before = store.clone();
    assert!(matches!(
        invoke(&ir, &mut store, "Invite", &input),
        Err(Undetermined::Undecidable { .. })
    ));
    assert_eq!(store, before);
}

#[test]
fn applied_row_counts_exclude_filter_and_from_decoys_but_include_equal_value_updates() {
    let ir = model(MODEL);
    let mut store = Store::default();
    for id in ["a", "b", "c", "parked"] {
        open(&ir, &mut store, id, "one", "before");
    }
    open(&ir, &mut store, "other", "two", "before");
    invoke(
        &ir,
        &mut store,
        "Park",
        &values([("session_id", text("parked"))]),
    )
    .unwrap();
    let parked = row(&store, "parked").clone();
    let other = row(&store, "other").clone();
    let input = values([("team", text("one")), ("note", text("after"))]);
    let end = invoke(&ir, &mut store, "EndTeam", &input).unwrap();
    assert_eq!(count(&end, "ended"), Node::Number(3_i64.into()));
    for id in ["a", "b", "c"] {
        assert_eq!(row(&store, id).state.to_string(), "Ended");
        assert_eq!(row(&store, id).fields["note"], text("after"));
    }
    assert_eq!(row(&store, "parked"), &parked);
    assert_eq!(row(&store, "other"), &other);
    let before = store.clone();
    assert_eq!(
        count(
            &invoke(&ir, &mut store, "EndTeam", &input).unwrap(),
            "ended"
        ),
        Node::Number(0_i64.into())
    );
    assert_eq!(store, before);
    for _ in 0..2 {
        assert_eq!(
            count(
                &invoke(&ir, &mut store, "NoteTeam", &input).unwrap(),
                "noted"
            ),
            Node::Number(4_i64.into())
        );
    }
    let before = store.clone();
    let zero = values([("team", text("missing")), ("note", text("ignored"))]);
    assert_eq!(
        count(
            &invoke(&ir, &mut store, "NoteTeam", &zero).unwrap(),
            "noted"
        ),
        Node::Number(0_i64.into())
    );
    assert_eq!(store, before);
}

#[test]
fn secondary_selections_use_original_subject_and_rows_and_writes_follow_declaration_order() {
    let source = MODEL.replace("        sets:\n          on_hold: false\n        affects:","        sets:\n          on_hold: false\n          team: two\n        affects:")
        .replace("              on_hold: true\nviews:","              on_hold: true\n              note: first\n          - entity: demo.desk.Session\n            where: {all: [team == subject.team, on_hold == false]}\n            sets: {note: second}\nviews:");
    assert!(source.contains("note: second"));
    let ir = model(&source);
    let mut store = Store::default();
    for id in ["subject", "a", "b", "c"] {
        open(&ir, &mut store, id, "one", "before");
    }
    open(&ir, &mut store, "other", "two", "before");
    let other = row(&store, "other").clone();
    invoke(
        &ir,
        &mut store,
        "Invite",
        &values([("session_id", text("subject"))]),
    )
    .unwrap();
    assert_eq!(row(&store, "subject").fields["team"], text("two"));
    assert_eq!(row(&store, "subject").fields["on_hold"], Node::Bool(false));
    assert_eq!(row(&store, "subject").fields["note"], text("before"));
    for id in ["a", "b", "c"] {
        assert_eq!(row(&store, id).fields["on_hold"], Node::Bool(true));
        assert_eq!(row(&store, id).fields["note"], text("second"));
    }
    assert_eq!(row(&store, "other"), &other);
}

#[test]
fn primary_failure_and_unknown_filters_never_apply_secondary_or_partial_writes() {
    let source = MODEL.replace(
        "        updates: demo.desk.Session\n        instance: session_id",
        "        moves: demo.desk.Session.park\n        instance: session_id",
    );
    let ir = model(&source);
    let mut store = Store::default();
    open(&ir, &mut store, "subject", "one", "before");
    open(&ir, &mut store, "other", "one", "before");
    invoke(
        &ir,
        &mut store,
        "Park",
        &values([("session_id", text("subject"))]),
    )
    .unwrap();
    let before = store.clone();
    for id in ["subject", "missing"] {
        let answer = invoke(
            &ir,
            &mut store,
            "Invite",
            &values([("session_id", text(id))]),
        )
        .unwrap();
        assert_eq!(answer.outcome, None);
        assert!(answer.events.is_empty());
        assert_eq!(store, before);
    }
    let source = MODEL.replace("          team: input.team\n", "");
    let ir = model(&source);
    let mut store = Store::default();
    open(&ir, &mut store, "unknown", "one", "before");
    let before = store.clone();
    assert!(matches!(
        invoke(
            &ir,
            &mut store,
            "EndTeam",
            &values([("team", text("one")), ("note", text("after"))])
        ),
        Err(Undetermined::Undecidable { .. })
    ));
    assert_eq!(store, before);
}

#[test]
fn secondary_and_set_subject_invariants_are_checked_before_returning_a_step() {
    let source = MODEL.replace(
        "    lifecycle:\n",
        "    invariants: ['on_hold == false']\n    lifecycle:\n",
    );
    let ir = model(&source);
    let mut store = Store::default();
    open(&ir, &mut store, "subject", "one", "before");
    open(&ir, &mut store, "other", "one", "before");
    let before = store.clone();
    assert!(matches!(
        invoke(
            &ir,
            &mut store,
            "Invite",
            &values([("session_id", text("subject"))])
        ),
        Err(Undetermined::BrokenInvariant { .. })
    ));
    assert_eq!(store, before);
    let source = MODEL.replace(
        "    lifecycle:\n",
        "    invariants: ['note != \"forbidden\"']\n    lifecycle:\n",
    );
    let ir = model(&source);
    let mut store = Store::default();
    open(&ir, &mut store, "a", "one", "before");
    open(&ir, &mut store, "b", "one", "before");
    let before = store.clone();
    assert!(matches!(
        invoke(
            &ir,
            &mut store,
            "NoteTeam",
            &values([("team", text("one")), ("note", text("forbidden"))])
        ),
        Err(Undetermined::BrokenInvariant { .. })
    ));
    assert_eq!(store, before);
}

const WRITES: &str = r"format: ess/16
system: demo
version: v1
domain: demo.desk
types:
  - {name: demo.desk.Note, kind: newtype, of: String, prefix: 'n-'}
entities:
  - name: demo.desk.Row
    identity: {name: id, type: Uuid}
    fields:
      - {name: group, type: String}
      - {name: note, type: demo.desk.Note}
      - {name: opt, type: 'Optional<String>'}
      - {name: flag, type: Boolean}
      - {name: amount, type: Integer}
      - {name: stamp, type: Uuid}
    lifecycle: {initial: Stored, states: [Stored], terminal: [Stored], transitions: []}
events:
  - {name: demo.desk.Opened, fields: [{name: id, type: Uuid}]}
  - {name: demo.desk.Patched, fields: [{name: count, type: Integer}]}
commands:
  - name: demo.desk.Open
    input: [{name: group, type: String}]
    outcomes:
      - name: opened
        creates: demo.desk.Row
        instance: id
        sets: {group: input.group, note: n-before, opt: before, flag: false, amount: 1, stamp: {generated: true}}
        emits: [demo.desk.Opened]
        payload: {demo.desk.Opened: {id: {generated: true}}}
  - name: demo.desk.Patch
    input:
      - {name: group, type: String}
      - {name: note, type: 'Optional<demo.desk.Note>'}
      - {name: flag, type: 'Optional<Boolean>'}
      - {name: amount, type: 'Optional<Integer>'}
    outcomes:
      - name: patched
        updates: demo.desk.Row
        instances: {where: group == input.group}
        sets: {note: {input: note, else: n-fallback}, opt: {cleared: true}, flag: {input: flag, else: true}, amount: {input: amount, else: 7}, stamp: {generated: true}}
        emits: [demo.desk.Patched]
        payload: {demo.desk.Patched: {count: {count: changed}}}
";

#[test]
fn typed_writes_preserve_falsy_inputs_and_apply_fallback_generation_and_clear() {
    for supplied in [None, Some(Node::Null), Some(text("n-present"))] {
        let ir = model(WRITES);
        let mut store = Store::default();
        let mut ids = Vec::new();
        for group in ["one", "one", "other"] {
            let step = invoke(&ir, &mut store, "Open", &values([("group", text(group))])).unwrap();
            ids.push(step.events[0].payload["id"].as_text().unwrap().to_owned());
        }
        let entity = "demo.desk.Row".parse().unwrap();
        let before = store.clone();
        let mut input = values([("group", text("one"))]);
        if let Some(value) = &supplied {
            input.insert("note".into(), value.clone());
            input.insert(
                "flag".into(),
                if *value == Node::Null {
                    Node::Null
                } else {
                    Node::Bool(false)
                },
            );
            input.insert(
                "amount".into(),
                if *value == Node::Null {
                    Node::Null
                } else {
                    Node::Number(0_i64.into())
                },
            );
        }
        let answer = invoke(&ir, &mut store, "Patch", &input).unwrap();
        assert_eq!(count(&answer, "count"), Node::Number(2_i64.into()));
        let present = supplied.as_ref().is_some_and(|value| *value != Node::Null);
        for id in &ids[..2] {
            let after = store.instance(&entity, id).unwrap();
            assert_eq!(
                after.fields["note"],
                text(if present { "n-present" } else { "n-fallback" })
            );
            assert_eq!(after.fields["flag"], Node::Bool(!present));
            assert_eq!(
                after.fields["amount"],
                Node::Number((if present { 0_i64 } else { 7_i64 }).into())
            );
            assert!(!after.fields.contains_key("opt"));
            let stamp = after.fields["stamp"].as_text().unwrap();
            assert!(ess_primitives::facts::is_canonical_uuid(stamp));
            assert_ne!(
                after.fields["stamp"],
                before.instance(&entity, id).unwrap().fields["stamp"]
            );
        }
        assert_eq!(
            store.instance(&entity, &ids[2]),
            before.instance(&entity, &ids[2])
        );
    }
}

#[test]
fn constrained_literal_violation_cannot_publish_partial_set_writes() {
    let source = WRITES
        .replace(
            "prefix: 'n-'",
            "prefix: 'n-', invariants: ['value != \"n-forbidden\"']",
        )
        .replace("else: n-fallback", "else: n-forbidden");
    let ir = model(&source);
    let mut store = Store::default();
    invoke(&ir, &mut store, "Open", &values([("group", text("one"))])).unwrap();
    let before = store.clone();
    assert!(invoke(&ir, &mut store, "Patch", &values([("group", text("one"))])).is_err());
    assert_eq!(store, before);
}

#[test]
fn generated_fallback_and_present_empty_text_are_not_confused_with_absence() {
    let source = WRITES.replace("else: n-fallback", "else: {generated: true}")
        .replace("      - {name: amount, type: 'Optional<Integer>'}", "      - {name: amount, type: 'Optional<Integer>'}\n      - {name: opt, type: 'Optional<String>'}")
        .replace("opt: {cleared: true}", "opt: {input: opt, else: fallback}");
    let ir = model(&source);
    for empty in [false, true] {
        let mut store = Store::default();
        let opened = invoke(&ir, &mut store, "Open", &values([("group", text("one"))])).unwrap();
        let id = opened.events[0].payload["id"].as_text().unwrap();
        let mut input = values([("group", text("one"))]);
        if empty {
            input.insert("opt".into(), text(""));
        }
        invoke(&ir, &mut store, "Patch", &input).unwrap();
        let after = store
            .instance(&"demo.desk.Row".parse().unwrap(), id)
            .unwrap();
        assert!(after.fields["note"].as_text().unwrap().starts_with("n-"));
        assert_eq!(
            after.fields["opt"],
            text(if empty { "" } else { "fallback" })
        );
    }
}

#[test]
fn secondary_exclusion_compares_entity_and_identity_not_just_key_text() {
    let other = r"  - name: demo.desk.Other
    identity: {name: session_id, type: demo.desk.SessionId}
    fields: [{name: team, type: demo.desk.Team}, {name: on_hold, type: Boolean}]
    lifecycle: {initial: Stored, states: [Stored], terminal: [Stored], transitions: []}
";
    let open_other = r"  - name: demo.desk.OpenOther
    input: [{name: team, type: demo.desk.Team}]
    outcomes:
      - name: opened
        creates: demo.desk.Other
        instance: session_id
        sets: {team: input.team, on_hold: false}
        emits: [demo.desk.OtherOpened]
        payload: {demo.desk.OtherOpened: {session_id: {generated: true}}}
";
    let source = MODEL.replace("events:\n", &format!("{other}events:\n  - name: demo.desk.OtherOpened\n    fields: [{{name: session_id, type: demo.desk.SessionId}}]\n"))
        .replace("commands:\n", &format!("commands:\n{open_other}"))
        .replace("              on_hold: true\nviews:", "              on_hold: true\n          - entity: demo.desk.Other\n            where: team == subject.team\n            sets: {on_hold: true}\nviews:");
    let ir = model(&source);
    let mut store = Store::default();
    open(&ir, &mut store, "same", "one", "before");
    let steps = execute_generating(
        &ir,
        &store,
        &"demo.desk.OpenOther".parse().unwrap(),
        &values([("team", text("one"))]),
        &Externals::Withheld,
        &Generated::Given(BTreeMap::from([(
            GeneratedSlot::new("demo.desk.OtherOpened".parse().unwrap(), "session_id"),
            text("same"),
        )])),
    )
    .unwrap();
    assert_eq!(steps.len(), 1);
    store = steps[0].next.clone();
    invoke(
        &ir,
        &mut store,
        "Invite",
        &values([("session_id", text("same"))]),
    )
    .unwrap();
    assert_eq!(row(&store, "same").fields["on_hold"], Node::Bool(false));
    assert_eq!(
        store
            .instance(&"demo.desk.Other".parse().unwrap(), "same")
            .unwrap()
            .fields["on_hold"],
        Node::Bool(true)
    );
}

#[test]
fn admitted_struct_set_fields_use_the_same_typed_leaf_sources() {
    let source = WRITES.replace("entities:\n", "  - name: demo.desk.Bag\n    kind: struct\n    fields: [{name: label, type: demo.desk.Note}, {name: enabled, type: Boolean}]\nentities:\n")
        .replace("    lifecycle:", "      - {name: bag, type: 'Optional<demo.desk.Bag>'}\n    lifecycle:")
        .replace("opt: {cleared: true}", "opt: {cleared: true}, bag: {label: {input: note, else: n-bag}, enabled: {input: flag, else: true}}");
    let ir = model(&source);
    let mut store = Store::default();
    let opened = invoke(&ir, &mut store, "Open", &values([("group", text("one"))])).unwrap();
    let id = opened.events[0].payload["id"].as_text().unwrap();
    invoke(
        &ir,
        &mut store,
        "Patch",
        &values([("group", text("one")), ("flag", Node::Bool(false))]),
    )
    .unwrap();
    assert_eq!(
        store
            .instance(&"demo.desk.Row".parse().unwrap(), id)
            .unwrap()
            .fields["bag"],
        Node::Map(values([
            ("label", text("n-bag")),
            ("enabled", Node::Bool(false))
        ]))
    );
}
