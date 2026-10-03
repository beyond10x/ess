//! Experimental protocol admission checks.
use ess_compiler::protocol::parse_and_compile;

const MODEL: &str = r"
format: ess-protospec/1
name: example
bounds: {max_steps: 100, max_states: 1000, max_time_ms: 10000}
participants:
  - name: client
    initial: Idle
    states: [Idle, Done]
    fields: []
    inputs:
      - name: start
        fields: [{name: id, type: String}]
    timers: []
    transitions:
      - name: begin
        from: Idle
        to: Done
        trigger: {kind: input, name: start}
        effects:
          - kind: send
            channel: wire
            message: request
            exchange: {kind: input, field: id}
            logical: {kind: literal, value: request}
            payload: {id: {kind: input, field: id}}
  - name: server
    initial: Idle
    states: [Idle]
    fields: []
    inputs: []
    timers: []
    transitions: []
messages:
  - name: request
    fields: [{name: id, type: String}]
channels:
  - {name: wire, from: client, to: server, ordering: fifo, capacity: 2, loss: false, duplication: false}
properties: []
";

#[test]
fn protocol_compiles_with_a_semantics_digest() {
    let compiled = parse_and_compile(MODEL).expect("valid model");
    assert_eq!(compiled.digest().len(), 64);
    assert_eq!(compiled.model().participants.len(), 2);
    assert_eq!(
        compiled.digest(),
        parse_and_compile(MODEL).unwrap().digest()
    );
    let changed = parse_and_compile(&MODEL.replace("capacity: 2", "capacity: 3")).unwrap();
    assert_ne!(compiled.digest(), changed.digest());
}

#[test]
fn protocol_refuses_unknown_and_duplicate_keys_and_versions() {
    for text in [
        MODEL.replace("ess-protospec/1", "ess-protospec/2"),
        format!("{MODEL}\nunknown: true\n"),
        MODEL.replace("name: example", "name: example\nname: overwritten"),
        MODEL.replace("value: request", "value: request, value: overwritten"),
    ] {
        assert!(parse_and_compile(&text).is_err(), "{text}");
    }
}

#[test]
fn protocol_refuses_unresolved_references_and_wrong_payload_types() {
    for text in [
        MODEL.replace("channel: wire", "channel: missing"),
        MODEL.replace("message: request", "message: missing"),
        MODEL.replace("field: id", "field: missing"),
        MODEL.replace(
            "payload: {id: {kind: input, field: id}}",
            "payload: {id: {kind: literal, value: 42}}",
        ),
        MODEL.replace("payload: {id: {kind: input, field: id}}", "payload: {}"),
        MODEL.replace("capacity: 2", "capacity: 0"),
        MODEL.replace("type: String", "type: Json"),
    ] {
        assert!(parse_and_compile(&text).is_err(), "{text}");
    }
}
