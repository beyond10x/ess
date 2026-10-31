//! A `!=` guard against the value an input's `example:` holds is refuted
//! (`story:a-text-inequality-against-an-example-is-refuted`).
//!
//! The example is the plain instance's base, and a leaf's ladder tries the guard's literals, each
//! one apart from the base. Where the example is the guard's own literal, that literal is the base,
//! so without the leaf's own witness on its ladder no candidate takes the guard's other side: the
//! refusal a `!=` names, or the branch past a `==`, is never sent. Both branches are synthesized
//! here, for a struct member reached through a structured example (ess/33) and for scalar inputs
//! of each leaf kind whose ladder holds only the guard's literals.

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::report::Status;
use ess_conformance::synthesize::{synthesize, Synthesis};
use ess_conformance::{AdmittedSuite, Runner, ScenarioStep, ScenarioValue};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::node::Node;

const ACCEPTED: &str = "demo.docs.Submit/outcome/accepted";
const REFUSED: &str = "demo.docs.Submit/outcome/unsupported";

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("spec.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|errors| panic!("{errors}\n{text}"))
}

fn yaml(text: &str) -> Node {
    ess_yaml::from_str(text).unwrap_or_else(|error| panic!("{error}: {text}"))
}

/// The suite synthesized for `text`, held to have no refusal and to pass on the interpreted
/// target.
fn synthesized(text: &str) -> Synthesis {
    let result = synthesize(&ir(text));
    assert!(
        result.refusals.is_empty(),
        "no refusal: {:?}\n{text}",
        result.refusals
    );
    let admitted =
        AdmittedSuite::from_suite(&result.suite).unwrap_or_else(|error| panic!("{error}"));
    let run = Runner::for_suite(&result.suite).run_admitted(
        &admitted,
        &ess_conformance::interpret::Interpreted::for_model(ir(text)),
    );
    assert!(!run.scenarios.is_empty(), "the run executes the suite");
    for scenario in &run.scenarios {
        assert_eq!(
            scenario.status,
            Status::Passed,
            "{}: {scenario:#?}",
            scenario.scenario
        );
    }
    result
}

/// Every value `scenario`'s steps executing `demo.docs.Submit` sent at `field`; panics where the
/// suite has no such scenario.
fn sent(result: &Synthesis, scenario: &str, field: &str) -> Vec<Node> {
    let scenario = result
        .suite
        .scenarios
        .iter()
        .find(|(id, _)| id.to_string() == scenario)
        .unwrap_or_else(|| {
            panic!(
                "no scenario {scenario}: {:?}",
                result.suite.scenarios.keys().collect::<Vec<_>>()
            )
        })
        .1;
    let mut values = Vec::new();
    for step in &scenario.steps {
        if let ScenarioStep::ExecuteCommand { command, input, .. } = step {
            if command.to_string() != "demo.docs.Submit" {
                continue;
            }
            match input.get(field) {
                Some(ScenarioValue::Literal { value }) => values.push(value.clone()),
                None => {}
                other => panic!("{field} is {other:?}"),
            }
        }
    }
    values
}

/// One command `demo.docs.Submit` taking `name: String` and `inputs`, refused as `unsupported`
/// where `when` holds.
fn model(types: &str, inputs: &str, when: &str) -> String {
    format!(
        "format: ess/33
system: demo
version: v1
domain: demo.docs
types:
{types}entities:
  - name: demo.docs.Doc
    identity: {{name: doc_id, type: Uuid}}
    fields:
      - {{name: name, type: String}}
    lifecycle: {{initial: Draft, states: [Draft], terminal: [Draft]}}
errors:
  - name: demo.docs.Unsupported
    fields:
      - {{name: name, type: String}}
events:
  - name: demo.docs.Submitted
    fields:
      - {{name: doc_id, type: Uuid}}
commands:
  - name: demo.docs.Submit
    input:
      - {{name: name, type: String}}
{inputs}    outcomes:
      - name: unsupported
        when: '{when}'
        error: demo.docs.Unsupported
        payload:
          demo.docs.Unsupported: {{name: input.name}}
      - name: accepted
        creates: demo.docs.Doc
        instance: doc_id
        sets: {{name: input.name}}
        emits: [demo.docs.Submitted]
        payload:
          demo.docs.Submitted: {{doc_id: {{generated: true}}}}
"
    )
}

const VERSIONED: &str = "  - name: demo.docs.Body
    kind: struct
    undeclared_fields: ignored
    fields:
      - {name: version, type: String}
";

#[test]
fn a_struct_member_inequality_against_its_example_sends_both_branches() {
    let example = "{version: \"1\", origin: upload}";
    let result = synthesized(&model(
        VERSIONED,
        &format!("      - {{name: body, type: demo.docs.Body, example: {example}}}\n"),
        "body.version != \"1\"",
    ));
    let accepted = sent(&result, ACCEPTED, "body");
    assert!(!accepted.is_empty(), "the accepted step sends body");
    for value in accepted {
        assert_eq!(value, yaml(example), "the accepted step sends the example");
    }
    let refused = sent(&result, REFUSED, "body");
    assert!(!refused.is_empty(), "the refusing step sends body");
    for value in refused {
        let members = value.as_map().expect("a struct");
        assert_ne!(
            members.get("version"),
            Some(&Node::Text("1".to_owned())),
            "the refusing step sends a version the guard does not hold: {value}"
        );
        assert_eq!(
            members.get("origin"),
            Some(&Node::Text("upload".to_owned())),
            "the undeclared member is carried as written: {value}"
        );
    }
}

#[test]
fn a_struct_member_equality_against_its_example_sends_both_branches() {
    // The guard's other polarity: the example is the refused value, and the accepted branch needs
    // a value apart from it.
    let example = "{version: \"1\", origin: upload}";
    let result = synthesized(&model(
        VERSIONED,
        &format!("      - {{name: body, type: demo.docs.Body, example: {example}}}\n"),
        "body.version == \"1\"",
    ));
    for value in sent(&result, REFUSED, "body") {
        assert_eq!(value, yaml(example), "the refusing step sends the example");
    }
    let accepted = sent(&result, ACCEPTED, "body");
    assert!(!accepted.is_empty(), "the accepted step sends body");
    for value in accepted {
        let members = value.as_map().expect("a struct");
        assert_ne!(
            members.get("version"),
            Some(&Node::Text("1".to_owned())),
            "{value}"
        );
    }
}

#[test]
fn a_scalar_inequality_against_its_example_sends_both_branches() {
    // Each scalar leaf kind whose ladder holds the guard's literals and nothing else.
    for (kind, example) in [
        ("String", "\"v1\""),
        ("Uuid", "\"00000000-0000-4000-8000-000000000001\""),
        ("Timestamp", "\"2026-01-01T00:00:00Z\""),
    ] {
        let text = model(
            "  []\n",
            &format!("      - {{name: tag, type: {kind}, example: {example}}}\n"),
            &format!("tag != {example}"),
        );
        let result = synthesized(&text);
        let accepted = sent(&result, ACCEPTED, "tag");
        assert!(!accepted.is_empty(), "{kind}: the accepted step sends tag");
        for value in accepted {
            assert_eq!(
                value,
                yaml(example),
                "{kind}: the accepted step sends the example"
            );
        }
        let refused = sent(&result, REFUSED, "tag");
        assert!(!refused.is_empty(), "{kind}: the refusing step sends tag");
        for value in refused {
            assert_ne!(
                value,
                yaml(example),
                "{kind}: the refusing step sends another value"
            );
        }
    }
}
