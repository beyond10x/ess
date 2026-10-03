//! The native target owns one actual response; payloads copy it without an expectation oracle.
use ess_compiler::{resolve::compile, source::SourceMap, EssIr};
use ess_conformance::{interpret::Interpreted, target::*};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::{
    consistency::QueryConsistency, ids::CorrelationId, node::Node, time::Timestamp,
};
use std::collections::BTreeMap;

const MODEL: &str = r"format: ess/21
system: demo
version: v1
domain: demo.response
types:
  - {name: demo.response.Large, kind: newtype, of: Integer, invariants: ['value == 9007199254740993']}
  - name: demo.response.Packet
    kind: struct
    fields: [{name: number, type: demo.response.Large}, {name: sequence, type: 'List<Integer>'}, {name: flags, type: 'Map<String, Boolean>'}]
  - name: demo.response.Envelope
    kind: struct
    fields: [{name: left, type: demo.response.Packet}, {name: right, type: demo.response.Packet}, {name: optional, type: 'Optional<String>'}]
events:
  - name: demo.response.Returned
    fields: [{name: packet, type: demo.response.Packet}, {name: envelope, type: demo.response.Envelope}, {name: receipt, type: String}]
  - name: demo.response.Mirrored
    fields: [{name: packet, type: demo.response.Packet}]
commands:
  - name: demo.response.Read
    response: [{name: packet, type: demo.response.Packet}, {name: optional, type: 'Optional<String>'}]
    outcomes:
      - name: returned
        emits: [demo.response.Returned, demo.response.Mirrored]
        payload:
          demo.response.Returned:
            packet: {response: packet}
            envelope: {left: {response: packet}, right: {response: packet}, optional: {response: optional}}
            receipt: {generated: true}
          demo.response.Mirrored: {packet: {response: packet}}
";
fn model(source: &str) -> EssIr {
    let spec = Specification::assemble([(
        Source::new("response.yaml"),
        RawSpecFile::parse(source).unwrap(),
    )])
    .unwrap();
    compile(&spec, &SourceMap::new()).unwrap()
}
fn correlation() -> CorrelationId {
    CorrelationId::new("actual-response").unwrap()
}
fn call(
    target: &Interpreted,
    command: &str,
    input: BTreeMap<String, Node>,
) -> Result<SemanticCommandResult, TargetError> {
    target.execute_command(SemanticCommandRequest {
        command: command.parse().unwrap(),
        actor: None,
        caller: None,
        input,
        correlation: correlation(),
    })
}
#[test]
fn nested_whole_aggregates_and_repeated_events_copy_one_actual_response() {
    let target = Interpreted::for_model(model(MODEL));
    let first = call(&target, "demo.response.Read", BTreeMap::new()).unwrap();
    let second = call(&target, "demo.response.Read", BTreeMap::new()).unwrap();
    for result in [&first, &second] {
        let response = result.response.as_ref().unwrap();
        let packet = &response["packet"];
        let Node::Map(fields) = packet else {
            panic!("packet")
        };
        assert_eq!(
            fields["number"],
            Node::Number(9_007_199_254_740_993i64.into())
        );
        assert!(matches!(fields["sequence"], Node::Seq(_)));
        assert!(matches!(fields["flags"], Node::Map(_)));
        assert_eq!(result.direct_events.len(), 2);
        assert_eq!(result.direct_events[0].payload["packet"], *packet);
        assert_eq!(result.direct_events[1].payload["packet"], *packet);
        let Node::Map(envelope) = &result.direct_events[0].payload["envelope"] else {
            panic!("envelope")
        };
        assert_eq!(envelope["left"], *packet);
        assert_eq!(envelope["right"], *packet);
        assert_eq!(envelope.get("optional"), response.get("optional"));
        assert!(matches!(
            result.direct_events[0].payload["receipt"],
            Node::Text(_)
        ));
    }
    assert_ne!(
        first.response, second.response,
        "successive actual invocation values"
    );
    assert_ne!(
        first.direct_events[0].payload["receipt"],
        second.direct_events[0].payload["receipt"]
    );
}

#[test]
fn public_step_apis_do_not_acquire_response_authority() {
    use ess_conformance::interpret::execute::{
        execute, execute_generating, Externals, Generated, Store,
    };
    let ir = model(MODEL);
    let command = "demo.response.Read".parse().unwrap();
    assert!(execute(
        &ir,
        &Store::default(),
        &command,
        &BTreeMap::new(),
        &Externals::Withheld
    )
    .is_err());
    for generated in [
        Generated::Counter,
        Generated::Given(BTreeMap::new()),
        Generated::Recorded(BTreeMap::new()),
    ] {
        assert!(execute_generating(
            &ir,
            &Store::default(),
            &command,
            &BTreeMap::new(),
            &Externals::Withheld,
            &generated
        )
        .is_err());
    }
}

const ATOMIC: &str = r"format: ess/21
system: demo
version: v1
domain: demo.response
entities:
  - name: demo.response.Row
    identity: {name: id, type: Integer}
    fields: [{name: count, type: Integer}]
    invariants: ['count <= 10']
    lifecycle: {initial: Held, states: [Held], terminal: [Held]}
events:
  - name: demo.response.Updated
    fields: [{name: receipt, type: String}]
commands:
  - name: demo.response.Update
    input: [{name: id, type: Integer}, {name: count, type: Integer}]
    response: [{name: secret, type: String}, {name: receipt, type: String}]
    outcomes:
      - name: updated
        updates: demo.response.Row
        instance: id
        sets: {count: input.count}
        returns: true
        one_time_response: [secret]
        emits: [demo.response.Updated]
        payload: {demo.response.Updated: {receipt: {response: receipt}}}
views:
  - name: demo.response.Rows
    source: demo.response.Row
    consistency: read_your_writes
    fields: [{name: id, type: Integer}, {name: count, type: Integer}]
";
fn established() -> Interpreted {
    let target = Interpreted::for_model(model(ATOMIC));
    begin(&target);
    target
        .establish_entity(EntitySetupRequest {
            entity: "demo.response.Row".parse().unwrap(),
            identity: Node::Number(7i64.into()),
            fields: BTreeMap::from([("count".into(), Node::Number(2i64.into()))]),
            state: "Held".parse().unwrap(),
            correlation: correlation(),
        })
        .unwrap();
    target
}
fn begin(target: &Interpreted) {
    target
        .begin_scenario(&ScenarioContext::new(
            "demo.response/authored/actual".parse().unwrap(),
            correlation(),
        ))
        .unwrap();
}
fn update(target: &Interpreted, count: i64) -> Result<SemanticCommandResult, TargetError> {
    call(
        target,
        "demo.response.Update",
        BTreeMap::from([
            ("id".into(), Node::Number(7i64.into())),
            ("count".into(), Node::Number(count.into())),
        ]),
    )
}
fn rows(target: &Interpreted) -> Vec<ViewRow> {
    target
        .query_view(SemanticViewRequest {
            view: "demo.response.Rows".parse().unwrap(),
            params: BTreeMap::new(),
            consistency: QueryConsistency::Current,
            correlation: correlation(),
            deadline: Deadline::at(Timestamp::from_epoch_millis(0)),
        })
        .unwrap()
        .rows
}
#[test]
fn failed_prepared_response_discards_issuance_rows_and_events() {
    let target = established();
    let fresh = established();
    let before = rows(&target);
    let error = update(&target, 11).unwrap_err();
    assert!(error.to_string().contains("invariant"), "{error}");
    assert_eq!(rows(&target), before);
    let result = update(&target, 3).unwrap();
    let control = update(&fresh, 3).unwrap();
    assert_eq!(
        result, control,
        "failed preparation cannot advance issuance or publication"
    );
    let again = update(&target, 4).unwrap();
    let first_response = result.response.unwrap();
    let second_response = again.response.unwrap();
    assert_ne!(first_response["secret"], second_response["secret"]);
    assert_eq!(
        result.direct_events[0].payload["receipt"],
        first_response["receipt"]
    );
    assert_eq!(
        again.direct_events[0].payload["receipt"],
        second_response["receipt"]
    );
    for event in result.direct_events.iter().chain(&again.direct_events) {
        assert!(!format!("{event:?}").contains(first_response["secret"].as_text().unwrap()));
        assert!(!format!("{event:?}").contains(second_response["secret"].as_text().unwrap()));
    }
    assert!(!format!("{target:?}").contains(first_response["secret"].as_text().unwrap()));
}

#[test]
fn actual_response_and_event_mutations_are_independently_detected() {
    let ir = model(include_str!("fixtures/response-payload.yaml"));
    let command = ir.commands().values().next().unwrap();
    let observation =
        ess_conformance::response::Observation::of(&ir, command, &command.outcomes[0])
            .unwrap()
            .remove(0);
    let target = Interpreted::for_model(ir);
    let result = call(&target, "demo.api.Cancel", BTreeMap::new()).unwrap();
    let response = result.response.as_ref().unwrap();
    let payload = &result.direct_events[0].payload;
    observation.compare(Some(response), payload).unwrap();
    let mut changed_response = response.clone();
    let Node::Map(item) = changed_response.get_mut("item").unwrap() else {
        panic!("item")
    };
    item.insert("remaining".into(), Node::Number((-123i64).into()));
    assert!(observation
        .compare(Some(&changed_response), payload)
        .is_err());
    let mut changed_event = payload.clone();
    let Node::Map(item) = changed_event.get_mut("item").unwrap() else {
        panic!("item")
    };
    item.insert("remaining".into(), Node::Number((-456i64).into()));
    assert!(observation.compare(Some(response), &changed_event).is_err());
}

#[test]
fn missing_and_wrong_state_outcomes_issue_no_response() {
    let source = ATOMIC
        .replace("lifecycle: {initial: Held, states: [Held], terminal: [Held]}", "lifecycle: {initial: Held, states: [Held, Done], terminal: [Done], transitions: [{name: finish, from: [Held], to: Done}]}")
        .replace("updates: demo.response.Row", "moves: demo.response.Row.finish")
        .replace("views:\n", "      - {name: missing, unknown_instance: true, refuses: false}\n      - {name: done, wrong_state: true, refuses: false}\nviews:\n");
    let target = Interpreted::for_model(model(&source));
    begin(&target);
    let missing = update(&target, 3).unwrap();
    assert_eq!(missing.outcome.unwrap().outcome.to_string(), "missing");
    assert!(missing.response.is_none());
    assert!(missing.direct_events.is_empty());
    target
        .establish_entity(EntitySetupRequest {
            entity: "demo.response.Row".parse().unwrap(),
            identity: Node::Number(7i64.into()),
            fields: BTreeMap::from([("count".into(), Node::Number(2i64.into()))]),
            state: "Done".parse().unwrap(),
            correlation: correlation(),
        })
        .unwrap();
    let done = update(&target, 3).unwrap();
    assert_eq!(done.outcome.unwrap().outcome.to_string(), "done");
    assert!(done.response.is_none());
    assert!(done.direct_events.is_empty());
}

#[test]
fn a_response_needed_only_by_nested_destinations_is_still_prepared() {
    let source = MODEL
        .replace(
            "fields: [{name: packet, type: demo.response.Packet}, {name: envelope",
            "fields: [{name: envelope",
        )
        .replace("            packet: {response: packet}\n", "")
        .replace(
            "        emits: [demo.response.Returned, demo.response.Mirrored]",
            "        emits: [demo.response.Returned]",
        )
        .replace(
            "          demo.response.Mirrored: {packet: {response: packet}}\n",
            "",
        );
    let target = Interpreted::for_model(model(&source));
    let result = call(&target, "demo.response.Read", BTreeMap::new()).unwrap();
    let response = result.response.unwrap();
    let Node::Map(envelope) = &result.direct_events[0].payload["envelope"] else {
        panic!("envelope")
    };
    assert_eq!(envelope["left"], response["packet"]);
    assert_eq!(envelope["right"], response["packet"]);
}

#[test]
fn whole_lists_maps_and_optional_fields_keep_declared_presence() {
    for ty in ["List<Integer>", "Map<String, Boolean>", "Optional<String>"] {
        for presence in [
            "",
            ", presence: null_when_absent",
            ", presence: omitted_when_absent",
        ] {
            if ty != "Optional<String>" && !presence.is_empty() {
                continue;
            }
            let source = format!(
                r"format: ess/21
system: demo
version: v1
domain: demo.response
events:
  - name: demo.response.Value
    fields: [{{name: value, type: '{ty}'}}]
commands:
  - name: demo.response.Read
    response: [{{name: value, type: '{ty}'{presence}}}]
    outcomes:
      - name: returned
        emits: [demo.response.Value]
        payload: {{demo.response.Value: {{value: {{response: value}}}}}}
"
            );
            let ir = model(&source);
            let command = ir.commands().values().next().unwrap();
            let observation =
                ess_conformance::response::Observation::of(&ir, command, &command.outcomes[0])
                    .unwrap()
                    .remove(0);
            let target = Interpreted::for_model(ir);
            let result = call(&target, "demo.response.Read", BTreeMap::new()).unwrap();
            let response = result.response.as_ref().unwrap();
            observation
                .compare(Some(response), &result.direct_events[0].payload)
                .unwrap();
            assert_eq!(
                response.get("value"),
                result.direct_events[0].payload.get("value")
            );
        }
    }
}

const RESPONSE_IDENTITY: &str = r"format: ess/4
system: demo
version: v1
domain: demo.response
entities:
  - name: demo.response.Row
    identity: {name: id, type: Uuid}
    lifecycle: {initial: Held, states: [Held], terminal: [Held]}
events:
  - name: demo.response.Created
    fields: [{name: id, type: Uuid}]
commands:
  - name: demo.response.Create
    response: [{name: id, type: Uuid}]
    outcomes:
      - name: created
        creates: demo.response.Row
        instance: id
        emits: [demo.response.Created]
        payload: {demo.response.Created: {id: {response: id}}}
views:
  - name: demo.response.CreatedRows
    source: demo.response.Row
    consistency: read_your_writes
    fields: [{name: id, type: Uuid}]
";

fn created_rows(target: &Interpreted) -> Vec<ViewRow> {
    target
        .query_view(SemanticViewRequest {
            view: "demo.response.CreatedRows".parse().unwrap(),
            params: BTreeMap::new(),
            consistency: QueryConsistency::Current,
            correlation: correlation(),
            deadline: Deadline::at(Timestamp::from_epoch_millis(0)),
        })
        .unwrap()
        .rows
}

#[test]
fn creation_uses_one_actual_response_identity_for_row_and_event() {
    let target = Interpreted::for_model(model(RESPONSE_IDENTITY));
    let first = call(&target, "demo.response.Create", BTreeMap::new()).unwrap();
    let second = call(&target, "demo.response.Create", BTreeMap::new()).unwrap();
    let first_id = &first.response.as_ref().unwrap()["id"];
    let second_id = &second.response.as_ref().unwrap()["id"];
    assert_ne!(first_id, second_id);
    assert_eq!(&first.direct_events[0].payload["id"], first_id);
    assert_eq!(&second.direct_events[0].payload["id"], second_id);
    let held = created_rows(&target);
    assert_eq!(held.len(), 2);
    for identity in [first_id, second_id] {
        assert!(held.iter().any(|row| row.get("id") == Some(identity)));
    }
}

#[test]
fn failed_creation_discards_its_prepared_response_identity() {
    let source = RESPONSE_IDENTITY
        .replace("    lifecycle:", "    fields: [{name: count, type: Integer}]\n    invariants: ['count <= 10']\n    lifecycle:")
        .replace("    response:", "    input: [{name: count, type: Integer}]\n    response:")
        .replace("        creates:", "        sets: {count: input.count}\n        creates:");
    let target = Interpreted::for_model(model(&source));
    let fresh = Interpreted::for_model(model(&source));
    let create = |target: &Interpreted, count: i64| {
        call(
            target,
            "demo.response.Create",
            BTreeMap::from([("count".into(), Node::Number(count.into()))]),
        )
    };
    assert!(create(&target, 11)
        .unwrap_err()
        .to_string()
        .contains("invariant"));
    assert!(created_rows(&target).is_empty());
    assert_eq!(create(&target, 3).unwrap(), create(&fresh, 3).unwrap());
}
