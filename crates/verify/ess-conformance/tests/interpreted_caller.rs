//! Authentication facts are typed per invocation and never substituted with request input.
use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::interpret::{
    execute::{self, Externals, Store},
    Interpreted,
};
use ess_conformance::target::*;
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::{ids::CorrelationId, node::Node};
use std::collections::BTreeMap;

const MODEL: &str = r"format: ess/19
system: demo
version: v1
domain: demo.notes
types:
  - {name: demo.notes.Id, kind: newtype, of: Uuid}
  - name: demo.notes.Bag
    kind: struct
    fields: [{name: key, type: String}, {name: flag, type: Boolean}, {name: opt, type: 'Optional<String>'}]
entities:
  - name: demo.notes.Record
    identity: {name: id, type: demo.notes.Id}
    fields: [{name: bag, type: demo.notes.Bag}]
    lifecycle: {initial: Stored, states: [Stored], terminal: [Stored], transitions: []}
actors:
  - name: demo.notes.User
    attributes: [{name: key, type: String}, {name: flag, type: Boolean}, {name: opt, type: 'Optional<String>'}]
    may: [demo.notes.Create, demo.notes.Choose]
errors:
  - name: demo.notes.Denied
    summary: Wrong key.
    fields: [{name: bag, type: demo.notes.Bag}]
commands:
  - name: demo.notes.Create
    input: [{name: key, type: String}]
    outcomes:
      - name: denied
        when: key != caller.key
        error: demo.notes.Denied
        payload: {demo.notes.Denied: {bag: {key: {caller: key}, flag: {caller: flag}, opt: {caller: opt}}}}
      - name: created
        creates: demo.notes.Record
        instance: id
        sets: {bag: {key: {caller: key}, flag: {caller: flag}, opt: {caller: opt}}}
        emits: [demo.notes.Created]
        payload: {demo.notes.Created: {id: {generated: true}, bag: {key: {caller: key}, flag: {caller: flag}, opt: {caller: opt}}}}
  - name: demo.notes.Choose
    input: [{name: key, type: String}]
    outcomes:
      - name: offered
        external: The provider offers a result.
        emits: [demo.notes.Offered]
        payload: {demo.notes.Offered: {key: {caller: key}}}
      - name: quiet
        error: demo.notes.Denied
events:
  - name: demo.notes.Created
    fields: [{name: id, type: demo.notes.Id}, {name: bag, type: demo.notes.Bag}]
  - name: demo.notes.Offered
    fields: [{name: key, type: String}]
views:
  - name: demo.notes.Records
    source: demo.notes.Record
    consistency: read_your_writes
    fields: [{name: id, type: demo.notes.Id}, {name: bag, type: demo.notes.Bag}]
";

fn ir(source: &str) -> EssIr {
    let spec = Specification::assemble([(
        Source::new("caller.yaml"),
        RawSpecFile::parse(source).unwrap(),
    )])
    .unwrap();
    compile(&spec, &SourceMap::new()).unwrap()
}
fn map<const N: usize>(values: [(&str, Node); N]) -> BTreeMap<String, Node> {
    values
        .into_iter()
        .map(|(key, value)| (key.into(), value))
        .collect()
}
fn text(value: &str) -> Node {
    Node::Text(value.into())
}
fn caller(key: &str, flag: bool) -> BTreeMap<String, Node> {
    map([("key", text(key)), ("flag", Node::Bool(flag))])
}
fn request(
    command: &str,
    key: &str,
    caller: Option<BTreeMap<String, Node>>,
) -> SemanticCommandRequest {
    SemanticCommandRequest {
        command: format!("demo.notes.{command}").parse().unwrap(),
        input: map([("key", text(key))]),
        actor: Some("demo.notes.User".parse().unwrap()),
        caller,
        correlation: CorrelationId::new("caller-test").unwrap(),
    }
}
fn target() -> Interpreted {
    let target = Interpreted::for_model(ir(MODEL));
    target
        .begin_scenario(&ScenarioContext::new(
            "demo.notes/authored/caller".parse().unwrap(),
            CorrelationId::new("caller-test").unwrap(),
        ))
        .unwrap();
    target
}
fn rows(target: &Interpreted) -> Vec<BTreeMap<String, Node>> {
    target
        .query_view(SemanticViewRequest {
            view: "demo.notes.Records".parse().unwrap(),
            params: BTreeMap::new(),
            consistency: ess_primitives::consistency::QueryConsistency::Current,
            correlation: CorrelationId::new("caller-test").unwrap(),
            deadline: Deadline::at(ess_primitives::time::Timestamp::from_epoch_millis(0)),
        })
        .unwrap()
        .rows
}

#[test]
fn nested_stored_event_and_error_values_use_the_current_typed_caller() {
    let target = target();
    for (key, flag) in [("first", false), ("second", true)] {
        let credentials = caller(key, flag);
        let expected = Node::Map(credentials.clone());
        let created = target
            .execute_command(request("Create", key, Some(credentials.clone())))
            .unwrap();
        assert_eq!(
            created.outcome.unwrap().to_string(),
            "demo.notes.Create/created"
        );
        assert_eq!(created.direct_events[0].payload["bag"], expected);
        let denied = target
            .execute_command(request("Create", "input-decoy", Some(credentials)))
            .unwrap();
        assert_eq!(
            denied.outcome.unwrap().to_string(),
            "demo.notes.Create/denied"
        );
        assert_eq!(denied.error.unwrap().fields["bag"], expected);
        assert!(denied.direct_events.is_empty());
    }
    let rows = rows(&target);
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0]["bag"], Node::Map(caller("first", false)));
    assert_eq!(rows[1]["bag"], Node::Map(caller("second", true)));
    // Explicit null and omitted Optional are the same absent attribute, never stale data.
    let mut credentials = caller("third", false);
    credentials.insert("opt".into(), Node::Null);
    let third = target
        .execute_command(request("Create", "third", Some(credentials)))
        .unwrap();
    assert_eq!(
        third.direct_events[0].payload["bag"],
        Node::Map(caller("third", false))
    );
}

#[test]
fn invalid_callers_do_not_consume_controls_or_create_rows_and_no_context_is_reused() {
    let target = target();
    target
        .configure_external_outcome(ExternalOutcomeControl {
            force: ess_conformance::scenario::OutcomeRef::new(
                "demo.notes.Choose".parse().unwrap(),
                "offered".parse().unwrap(),
            ),
            correlation: CorrelationId::new("caller-test").unwrap(),
        })
        .unwrap();
    let invalid = [
        BTreeMap::new(),
        map([("key", text("valid"))]),
        map([("key", text("valid")), ("flag", text("false"))]),
        map([("key", Node::Null), ("flag", Node::Bool(false))]),
        map([
            ("key", text("valid")),
            ("flag", Node::Bool(false)),
            ("extra", text("no")),
        ]),
    ];
    for credentials in invalid {
        let answer = target.execute_command(request("Choose", "valid", Some(credentials)));
        assert!(
            matches!(answer, Err(TargetError::Unavailable { .. })),
            "{answer:?}"
        );
        assert!(rows(&target).is_empty());
    }
    let mut no_actor = request("Choose", "valid", Some(caller("valid", false)));
    no_actor.actor = None;
    assert!(matches!(
        target.execute_command(no_actor),
        Err(TargetError::Unavailable { .. })
    ));
    let mut ungranted = request("Choose", "valid", Some(caller("valid", false)));
    ungranted.actor = Some("demo.notes.Stranger".parse().unwrap());
    assert!(matches!(
        target.execute_command(ungranted),
        Err(TargetError::NotGranted { .. })
    ));
    let offered = target
        .execute_command(request("Choose", "decoy", Some(caller("valid", false))))
        .unwrap();
    assert_eq!(
        offered.outcome.unwrap().to_string(),
        "demo.notes.Choose/offered"
    );
    assert_eq!(offered.direct_events[0].payload["key"], text("valid"));
    let created = target
        .execute_command(request("Create", "valid", Some(caller("valid", false))))
        .unwrap();
    assert_eq!(
        created.direct_events[0].payload["id"],
        text("00000000-0000-4000-8000-000000000001")
    );
    let before = rows(&target);
    let absent = AbsentInputRequest {
        command: "demo.notes.Create".parse().unwrap(),
        actor: Some("demo.notes.User".parse().unwrap()),
        caller: Some(BTreeMap::new()),
        correlation: CorrelationId::new("caller-test").unwrap(),
    };
    assert!(matches!(
        target.execute_command_without_input(absent.clone()),
        Err(TargetError::Unavailable { .. })
    ));
    let undeclared = target
        .execute_command_without_input(AbsentInputRequest {
            caller: Some(caller("valid", false)),
            ..absent
        })
        .unwrap();
    assert!(undeclared.outcome.is_none());
    assert!(undeclared.direct_events.is_empty());
    assert_eq!(rows(&target), before);
    let missing = target.execute_command(request("Create", "valid", None));
    assert!(
        matches!(missing, Err(TargetError::Unsupported { .. })),
        "{missing:?}"
    );
    assert_eq!(rows(&target), before);
    let model = ir(MODEL);
    let no_context = execute::execute(
        &model,
        &Store::default(),
        &"demo.notes.Create".parse().unwrap(),
        &map([("key", text("valid"))]),
        &Externals::Withheld,
    );
    assert!(
        matches!(no_context, Err(execute::Undetermined::Undecidable { .. })),
        "{no_context:?}"
    );
}

#[test]
fn declared_input_and_stored_fields_named_caller_keep_their_original_namespace() {
    let source = r"format: ess/19
system: demo
version: v1
domain: demo.notes
types:
  - {name: demo.notes.Id, kind: newtype, of: Uuid}
  - name: demo.notes.Bundle
    kind: struct
    fields: [{name: key, type: String}]
entities:
  - name: demo.notes.Record
    identity: {name: id, type: demo.notes.Id}
    fields: [{name: caller, type: demo.notes.Bundle}]
    lifecycle: {initial: Stored, states: [Stored], terminal: [Stored], transitions: []}
actors:
  - name: demo.notes.User
    attributes: [{name: key, type: String}]
    may: [demo.notes.Create, demo.notes.Edit]
errors:
  - {name: demo.notes.Denied, summary: Wrong key.}
commands:
  - name: demo.notes.Create
    input: [{name: caller, type: demo.notes.Bundle}, {name: key, type: String}]
    outcomes:
      - {name: denied, when: 'caller.key != stored-value', error: demo.notes.Denied}
      - name: created
        creates: demo.notes.Record
        instance: id
        sets: {caller: input.caller}
        emits: [demo.notes.Created]
        payload: {demo.notes.Created: {id: {generated: true}}}
  - name: demo.notes.Edit
    input: [{name: id, type: demo.notes.Id}, {name: key, type: String}]
    outcomes:
      - name: denied
        when_subject: {predicate: 'caller.key != input.key'}
        error: demo.notes.Denied
      - name: edited
        updates: demo.notes.Record
        instance: id
        emits: [demo.notes.Edited]
        payload: {demo.notes.Edited: {id: input.id}}
events:
  - name: demo.notes.Created
    fields: [{name: id, type: demo.notes.Id}]
  - name: demo.notes.Edited
    fields: [{name: id, type: demo.notes.Id}]
views:
  - name: demo.notes.Records
    source: demo.notes.Record
    consistency: read_your_writes
    fields: [{name: id, type: demo.notes.Id}, {name: caller, type: demo.notes.Bundle}]
";
    let target = Interpreted::for_model(ir(source));
    target
        .begin_scenario(&ScenarioContext::new(
            "demo.notes/authored/shadow".parse().unwrap(),
            CorrelationId::new("caller-test").unwrap(),
        ))
        .unwrap();
    let authentication = map([("key", text("authentication-decoy"))]);
    let mut create = request("Create", "stored-value", Some(authentication.clone()));
    create.input.insert(
        "caller".into(),
        Node::Map(map([("key", text("stored-value"))])),
    );
    let created = target.execute_command(create).unwrap();
    assert_eq!(
        created.outcome.unwrap().to_string(),
        "demo.notes.Create/created"
    );
    let id = created.direct_events[0].payload["id"].clone();
    for (key, expected) in [
        ("authentication-decoy", "denied"),
        ("stored-value", "edited"),
    ] {
        let mut edit = request("Edit", key, Some(authentication.clone()));
        edit.input.insert("id".into(), id.clone());
        let result = target.execute_command(edit).unwrap();
        assert_eq!(
            result.outcome.unwrap().to_string(),
            format!("demo.notes.Edit/{expected}")
        );
    }
    assert_eq!(
        rows(&target)[0]["caller"],
        Node::Map(map([("key", text("stored-value"))]))
    );
}
