//! A list input's `example:` is what synthesis sends (ess/31, an adopter report, 2026-10-22).
//!
//! Without one, a list no guard reads is sent empty, and an implementation that accepts only a
//! list naming a member it knows refuses every synthesized request. With one, the plain instance
//! sends the example as written, as it does a scalar input's.

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::{ScenarioStep, ScenarioValue};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::node::Node;

const MODEL: &str = r#"format: ess/31
system: demo
version: v1
domain: demo.server
types:
  - name: demo.server.ConnectionType
    kind: enum
    variants: [long-polling, websocket]
  - name: demo.server.Node
    kind: struct
    fields:
      - {name: label, type: String}
      - {name: weight, type: Integer}
entities:
  - name: demo.server.Client
    identity: {name: client_id, type: Uuid}
    fields: []
    lifecycle:
      initial: Connected
      states: [Connected]
      terminal: [Connected]
      transitions: []
events:
  - name: demo.server.HandshakeAccepted
    fields:
      - {name: client_id, type: Uuid}
commands:
  - name: demo.server.Handshake
    input:
      - {name: supported_connection_types, type: "List<demo.server.ConnectionType>", example: [websocket]}
      - {name: nodes, type: "List<demo.server.Node>", example: [{label: first, weight: 2}]}
      - {name: hints, type: "List<String>"}
    outcomes:
      - name: accepted
        creates: demo.server.Client
        instance: client_id
        emits: [demo.server.HandshakeAccepted]
        payload:
          demo.server.HandshakeAccepted:
            client_id: {generated: true}
"#;

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("spec.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap()
}

/// Every value the suite sent at `field` of `demo.server.Handshake`.
fn sent(field: &str) -> Vec<Node> {
    let result = ess_conformance::synthesize::synthesize(&ir(MODEL));
    assert!(
        result.refusals.is_empty(),
        "no refusal: {:?}",
        result.refusals
    );
    let mut values = Vec::new();
    for scenario in result.suite.scenarios.values() {
        for step in &scenario.steps {
            if let ScenarioStep::ExecuteCommand { command, input, .. } = step {
                if command.to_string() != "demo.server.Handshake" {
                    continue;
                }
                match input.get(field) {
                    Some(ScenarioValue::Literal { value }) => values.push(value.clone()),
                    other => panic!("{field} is {other:?}"),
                }
            }
        }
    }
    assert!(!values.is_empty(), "the suite executes the command");
    values
}

#[test]
fn a_list_input_with_an_example_is_sent_as_the_example() {
    for value in sent("supported_connection_types") {
        assert_eq!(value, Node::Seq(vec![Node::Text("websocket".to_owned())]));
    }
}

#[test]
fn a_list_of_structs_with_an_example_is_sent_as_the_example() {
    let expected: Node = ess_yaml::from_str("[{label: first, weight: 2}]").unwrap();
    for value in sent("nodes") {
        assert_eq!(value, expected);
    }
}

#[test]
fn a_list_input_without_an_example_is_still_sent_empty() {
    for value in sent("hints") {
        assert_eq!(value, Node::Seq(Vec::new()));
    }
}
