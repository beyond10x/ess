//! A `Json` value in a suite (beyond10x/ess#138): its witness is a small object, and a payload
//! that copies it is compared structurally.

use std::collections::BTreeMap;

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::{Holds, ScenarioStep, ScenarioValue};
use ess_domain::types::Primitive;
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::node::Node;

const MODEL: &str = "format: ess/15
system: demo
version: v1
domain: demo.msgs
types:
  - {name: demo.msgs.Body, kind: newtype, of: Json}
events:
  - name: demo.msgs.Sent
    fields:
      - {name: body, type: demo.msgs.Body}
      - {name: headers, type: Json}
actors:
  - {name: demo.msgs.Sender, may: [demo.msgs.Send]}
commands:
  - name: demo.msgs.Send
    input:
      - {name: body, type: demo.msgs.Body}
      - {name: headers, type: Json}
    outcomes:
      - name: sent
        emits: [demo.msgs.Sent]
        payload:
          demo.msgs.Sent: {body: input.body, headers: input.headers}
";

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("msgs.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap()
}

#[test]
fn issue_138_a_json_input_is_an_object_and_its_event_is_compared_structurally() {
    let result = ess_conformance::synthesize::synthesize(&ir(MODEL));
    assert!(result.refusals.is_empty(), "{:#?}", result.refusals);
    let mut sent = Vec::new();
    let mut expected = Vec::new();
    for step in result
        .suite
        .scenarios
        .values()
        .flat_map(|scenario| &scenario.steps)
    {
        match step {
            ScenarioStep::ExecuteCommand { input, .. } => sent.push(input.clone()),
            ScenarioStep::ExpectEvent { payload, shape, .. } => {
                expected.push(payload.clone());
                for field in ["body", "headers"] {
                    let leaf = shape.leaves().get(field).expect("described");
                    assert_eq!(
                        leaf.holds,
                        Holds::Primitive {
                            kind: Primitive::Json
                        }
                    );
                }
            }
            _ => {}
        }
    }
    assert!(!sent.is_empty() && !expected.is_empty());
    let Some(ScenarioValue::Literal {
        value: Node::Map(body),
    }) = sent[0].get("body")
    else {
        panic!("the witness of a Json input is an object: {:?}", sent[0]);
    };
    assert!(!body.is_empty());
    let Some(ScenarioValue::Literal { value: headers }) = sent[0].get("headers") else {
        panic!("{:?}", sent[0]);
    };
    assert_ne!(
        &Node::Map(body.clone()),
        headers,
        "two Json fields are never interchangeable"
    );
    assert_eq!(
        expected[0].get("body"),
        Some(&Node::Map(body.clone())),
        "the event is held to exactly the object that was sent"
    );
}

#[test]
fn a_json_leaf_admits_any_value_and_equality_is_structural() {
    let json = Holds::Primitive {
        kind: Primitive::Json,
    };
    let object = |entries: &[(&str, Node)]| {
        Node::Map(
            entries
                .iter()
                .map(|(key, value)| ((*key).to_owned(), value.clone()))
                .collect::<BTreeMap<_, _>>(),
        )
    };
    for value in [
        object(&[("a", Node::Seq(vec![Node::Bool(true)]))]),
        Node::Seq(Vec::new()),
        Node::Text("text".to_owned()),
        Node::Bool(false),
    ] {
        assert!(json.admits(&value), "{value:?}");
    }
    assert_eq!(
        object(&[("a", Node::Bool(true)), ("b", Node::Text("x".to_owned()))]),
        object(&[("b", Node::Text("x".to_owned())), ("a", Node::Bool(true))]),
        "key order is not part of a JSON object"
    );
}
