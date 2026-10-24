//! Adversary cases for `example:` on a command input of `List` type (ess/31).
//!
//! Each case drives the change from the format history row it wrote: the example is held as a
//! precondition's literal is, synthesis sends it wherever no guard reads into the list, and a
//! model without one keeps everything it had.

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::report::Status;
use ess_conformance::{AdmittedSuite, ConformanceSuite, Runner, ScenarioStep, ScenarioValue};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::error::{ValidationCode, ValidationErrors};
use ess_primitives::node::Node;

const HANDSHAKE: &str = r#"format: ess/31
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
    fields:
      - {name: kinds, type: "List<demo.server.ConnectionType>"}
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
      - {name: hints, type: "Optional<List<String>>", example: [x]}
    outcomes:
      - name: accepted
        creates: demo.server.Client
        instance: client_id
        emits: [demo.server.HandshakeAccepted]
        payload:
          demo.server.HandshakeAccepted:
            client_id: {generated: true}
        sets: {kinds: input.supported_connection_types}
views:
  - name: demo.server.Clients
    source: demo.server.Client
    consistency: read_your_writes
    fields:
      - {name: client_id, type: Uuid}
      - {name: kinds, type: "List<demo.server.ConnectionType>"}
"#;

const LEDGER: &str = include_str!("fixtures/list-of-references.yaml");

fn assemble(text: &str) -> Result<Specification, ValidationErrors> {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    Specification::assemble([(Source::new("spec.yaml"), raw)])
}

fn ir(text: &str) -> EssIr {
    let spec = assemble(text).unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn suite(model: &EssIr) -> ConformanceSuite {
    let synthesis = ess_conformance::synthesize::synthesize(model);
    assert!(
        synthesis.refusals.is_empty(),
        "nothing is refused: {:#?}",
        synthesis.refusals
    );
    synthesis.suite
}

fn passes_on_the_interpreter(model: EssIr, suite: &ConformanceSuite) {
    let admitted = AdmittedSuite::from_suite(suite).expect("the suite is admitted");
    let run = Runner::for_suite(admitted.suite()).run_admitted(
        &admitted,
        &ess_conformance::interpret::Interpreted::for_model(model),
    );
    assert_ne!(run.scenarios.len(), 0);
    let failed: Vec<_> = run
        .scenarios
        .iter()
        .filter(|result| result.status != Status::Passed)
        .collect();
    assert!(failed.is_empty(), "{failed:#?}");
}

/// Every value each scenario sends at `field` of `command`, with the scenario's id.
fn sent(suite: &ConformanceSuite, command: &str, field: &str) -> Vec<(String, ScenarioValue)> {
    let mut found = Vec::new();
    for (id, scenario) in &suite.scenarios {
        for step in &scenario.steps {
            if let ScenarioStep::ExecuteCommand {
                command: name,
                input,
                ..
            } = step
            {
                if name.to_string() == command {
                    if let Some(value) = input.get(field) {
                        found.push((id.to_string(), value.clone()));
                    }
                }
            }
        }
    }
    found
}

fn with(model: &str, from: &str, to: &str) -> String {
    assert!(model.contains(from), "the model carries {from:?}");
    model.replacen(from, to, 1)
}

/// The model with a list newtype whose own invariant counts it.
fn with_tags(model: &str) -> String {
    with(
        model,
        "entities:\n",
        "  - name: demo.server.Tags\n    kind: newtype\n    of: \"List<String>\"\n    invariants: [value.count >= 1]\nentities:\n",
    )
}

/// A list newtype counting itself, on an input whose example satisfies the count: synthesis has
/// the example as its witness, as a scalar newtype's example is, so nothing is refused.
#[test]
fn adv_a_counted_list_newtype_with_an_example_is_not_refused_by_synthesis() {
    // The type is stored and published, so its invariant is observed through the view.
    let text = with(
        &with(
            &with(
                &with(
                    &with_tags(HANDSHAKE),
                    "      - {name: hints, type: \"Optional<List<String>>\", example: [x]}",
                    "      - {name: hints, type: \"Optional<List<String>>\", example: [x]}\n      - {name: tags, type: \"demo.server.Tags\", example: [a]}",
                ),
                "        sets: {kinds: input.supported_connection_types}",
                "        sets: {kinds: input.supported_connection_types, tags: input.tags}",
            ),
            "      - {name: kinds, type: \"List<demo.server.ConnectionType>\"}\n    lifecycle:",
            "      - {name: kinds, type: \"List<demo.server.ConnectionType>\"}\n      - {name: tags, type: \"demo.server.Tags\"}\n    lifecycle:",
        ),
        "      - {name: client_id, type: Uuid}\n      - {name: kinds, type: \"List<demo.server.ConnectionType>\"}\n",
        "      - {name: client_id, type: Uuid}\n      - {name: kinds, type: \"List<demo.server.ConnectionType>\"}\n      - {name: tags, type: \"demo.server.Tags\"}\n",
    );
    let model = ir(&text);
    let suite = suite(&model);
    let expected = ScenarioValue::Literal {
        value: Node::Seq(vec![Node::Text("a".to_owned())]),
    };
    for (id, value) in sent(&suite, "demo.server.Handshake", "tags") {
        assert_eq!(value, expected, "{id}");
    }
    passes_on_the_interpreter(model, &suite);
}

fn refused_with(text: &str, code: ValidationCode, site: &str, says: &str) {
    let Err(errors) = assemble(text) else {
        panic!("must refuse:\n{text}");
    };
    assert!(
        errors.as_slice().iter().any(|error| error.code == code
            && error.location.contains(site)
            && error.message.contains(says)),
        "expected {code:?} at …{site} saying {says:?}, got:\n{errors}"
    );
}

/// An example on a list of references a `when_related` guard reads: the example's identities
/// name no row, so wherever synthesis sends the example instead of arranged rows the interpreter
/// takes the refusal where the suite expects acceptance.
#[test]
fn adv_a_list_of_references_with_an_example_still_passes_on_the_interpreter() {
    let text = with(
        &with(LEDGER, "format: ess/28", "format: ess/31"),
        "      - {name: services, type: \"List<ledger.desk.ServiceId>\"}\n    outcomes:",
        "      - {name: services, type: \"List<ledger.desk.ServiceId>\", example: [\"7d1c6a52-3f0e-4b8a-9a51-0c2f4d6e8b10\"]}\n    outcomes:",
    );
    let model = ir(&text);
    let suite = suite(&model);
    passes_on_the_interpreter(model, &suite);
}

/// The unit's own model, extended by a view and a copy into a stored field, run end to end.
#[test]
fn adv_the_handshake_suite_passes_on_the_interpreter() {
    let model = ir(HANDSHAKE);
    let suite = suite(&model);
    passes_on_the_interpreter(model, &suite);
}

/// The need: every success scenario of the command sends a list naming a member the target
/// knows. A scenario that creates a second client (a view reading two rows) must not send `[]`.
#[test]
fn adv_every_handshake_sends_a_nonempty_connection_list() {
    let model = ir(HANDSHAKE);
    let suite = suite(&model);
    let all = sent(
        &suite,
        "demo.server.Handshake",
        "supported_connection_types",
    );
    assert_ne!(all.len(), 0);
    let empty: Vec<_> = all
        .iter()
        .filter(|(_, value)| {
            matches!(value, ScenarioValue::Literal { value: Node::Seq(items) } if items.is_empty())
        })
        .collect();
    assert!(
        empty.is_empty(),
        "{} of {} sends are []: {empty:#?}",
        empty.len(),
        all.len()
    );
}

/// An `Optional<List<…>>` input with an example sends it somewhere.
#[test]
fn adv_an_optional_list_example_is_sent() {
    let suite = suite(&ir(HANDSHAKE));
    let expected = ScenarioValue::Literal {
        value: Node::Seq(vec![Node::Text("x".to_owned())]),
    };
    let all = sent(&suite, "demo.server.Handshake", "hints");
    assert!(
        all.iter().any(|(_, value)| *value == expected),
        "no scenario sends hints: [x]: {all:#?}"
    );
}

/// The generated JSON Schema carries the list example as one example value, `[["websocket"]]`.
#[test]
fn adv_the_json_schema_carries_the_list_example() {
    let artifacts = ess_gen::generate_all(&ir(HANDSHAKE)).unwrap();
    let compact = |text: &str| text.split_whitespace().collect::<String>();
    let found = artifacts.values().any(|artifact| {
        let text = compact(&artifact.contents);
        text.contains("\"examples\":[[\"websocket\"]]")
    });
    assert!(
        found,
        "no artifact carries examples: [[\"websocket\"]]; paths: {:?}",
        artifacts.keys().collect::<Vec<_>>()
    );
}

/// Boundaries the validation must hold: nested lists, an instant, a struct's unknown member, a
/// list newtype's own count invariant, a Binary64 element.
#[test]
fn adv_list_examples_are_held_to_the_element_type() {
    let site = "command.demo.server.Handshake.input";
    let nested = with(
        &with_tags(HANDSHAKE),
        "      - {name: hints, type: \"Optional<List<String>>\", example: [x]}",
        "      - {name: hints, type: \"Optional<List<String>>\", example: [x]}\n      - {name: grid, type: \"List<List<demo.server.ConnectionType>>\", example: [[websocket]]}\n      - {name: nodes, type: \"List<demo.server.Node>\"}\n      - {name: tags, type: \"demo.server.Tags\"}\n      - {name: at, type: \"List<Timestamp>\"}\n      - {name: weights, type: \"List<Binary64>\"}",
    );
    assemble(&nested).unwrap_or_else(|errors| panic!("nested list admitted: {errors}"));
    refused_with(
        &with(
            &nested,
            "example: [[websocket]]",
            "example: [[carrier-pigeon]]",
        ),
        ValidationCode::TypeMismatch,
        &format!("{site}.grid.example"),
        "element 0",
    );
    let line = |field: &str, ty: &str, example: &str| {
        (
            format!("      - {{name: {field}, type: \"{ty}\"}}"),
            format!("      - {{name: {field}, type: \"{ty}\", example: {example}}}"),
        )
    };
    for (field, ty, example, code, says) in [
        (
            "nodes",
            "List<demo.server.Node>",
            "[{label: a, weight: 1, extra: 2}]",
            None,
            "extra",
        ),
        (
            "tags",
            "demo.server.Tags",
            "[]",
            Some(ValidationCode::ConflictingDeclaration),
            "",
        ),
        (
            "at",
            "List<Timestamp>",
            "[\"yesterday\"]",
            Some(ValidationCode::TypeMismatch),
            "element 0",
        ),
        (
            "weights",
            "List<Binary64>",
            "[1.5]",
            Some(ValidationCode::TypeMismatch),
            "Binary64",
        ),
    ] {
        let (from, to) = line(field, ty, example);
        let text = with(&nested, &from, &to);
        match code {
            Some(code) => refused_with(&text, code, &format!("{site}.{field}.example"), says),
            None => {
                assert!(
                    assemble(&text).is_err(),
                    "{field}: {example} must be refused"
                );
            }
        }
    }
}

/// A struct element breaking its struct's invariant conflicts, as a precondition literal does.
#[test]
fn adv_a_struct_element_breaking_its_invariant_is_refused() {
    let text = with(
        &with(
            HANDSHAKE,
            "      - {name: weight, type: Integer}\n",
            "      - {name: weight, type: Integer}\n    invariants: [weight > 0]\n",
        ),
        "      - {name: hints, type: \"Optional<List<String>>\", example: [x]}",
        "      - {name: hints, type: \"Optional<List<String>>\", example: [x]}\n      - {name: nodes, type: \"List<demo.server.Node>\", example: [{label: a, weight: 0}]}",
    );
    let Err(errors) = assemble(&text) else {
        panic!("must refuse a struct element with weight 0");
    };
    assert!(
        errors
            .as_slice()
            .iter()
            .any(|error| error.location.contains("nodes.example")
                && error.code == ValidationCode::ConflictingDeclaration),
        "{errors}"
    );
}

/// A guard reading into the list with an example on it: the suite still passes end to end.
#[test]
fn adv_a_guarded_list_with_an_example_passes_on_the_interpreter() {
    let text = with(
        &with(
            HANDSHAKE,
            "events:\n",
            "errors:\n  - name: demo.server.NoTransport\n    fields: []\nevents:\n",
        ),
        "    outcomes:\n      - name: accepted\n",
        "    outcomes:\n      - name: no-transport\n        when: supported_connection_types.count == 0\n        error: demo.server.NoTransport\n      - name: accepted\n",
    );
    let model = ir(&text);
    let suite = suite(&model);
    passes_on_the_interpreter(model, &suite);
}

/// Below ess/31 an `Optional<List<…>>` example keeps its refusal, its hint naming ess/31.
#[test]
fn adv_an_optional_list_example_below_ess_31_names_ess_31() {
    let text = with(
        &with(
            HANDSHAKE,
            "      - {name: supported_connection_types, type: \"List<demo.server.ConnectionType>\", example: [websocket]}",
            "      - {name: supported_connection_types, type: \"List<demo.server.ConnectionType>\"}",
        ),
        "format: ess/31",
        "format: ess/30",
    );
    let Err(errors) = assemble(&text) else {
        panic!("must refuse below ess/31");
    };
    let error = errors
        .as_slice()
        .iter()
        .find(|error| error.location.contains("hints.example"))
        .unwrap_or_else(|| panic!("a refusal at hints.example: {errors}"));
    assert_eq!(error.code, ValidationCode::TypeMismatch);
    assert!(
        error
            .hint
            .as_deref()
            .is_some_and(|hint| hint.contains("ess/31")),
        "{error:?}"
    );
}
