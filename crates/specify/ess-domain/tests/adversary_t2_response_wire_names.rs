//! Adversarial cases for the response wire namespace in the specification pass.

use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_primitives::error::{ValidationCode, ValidationError, ValidationErrors};

fn model(input: &str, response: &str) -> String {
    format!(
        "format: ess/22
system: demo
version: v1
domain: demo.core
events:
  - name: demo.core.Sent
    fields:
      - {{name: id, type: String}}
commands:
  - name: demo.core.Send
    input:
{input}
    response:
{response}
    outcomes:
      - name: sent
        returns: true
        emits: [demo.core.Sent]
        payload:
          demo.core.Sent:
            id: {{generated: true}}
actors:
  - name: demo.core.Writer
    may: [demo.core.Send]
components:
  - component: messages
    owns: {{domains: [demo.core]}}
    accepts: {{commands: [demo.core.Send]}}
    publishes: {{events: [demo.core.Sent]}}
    reached_by: network
"
    )
}

fn assemble(text: &str) -> Result<Specification, ValidationErrors> {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("well formed: {error}"));
    Specification::assemble([(Source::new("demo.yaml"), raw)])
}

fn duplicates(text: &str) -> Vec<ValidationError> {
    let Err(errors) = assemble(text) else {
        panic!("validated, and should have been refused")
    };
    errors
        .into_iter()
        .filter(|error| error.code == ValidationCode::DuplicateDeclaration)
        .collect()
}

const INPUT: &str = "      - {name: text, type: String}";

#[test]
fn the_field_level_wire_spelling_collides_in_the_response() {
    let found = duplicates(&model(
        INPUT,
        "      - {name: reply, type: String}\n      - {name: other, type: String, wire: reply}",
    ));
    assert_eq!(found.len(), 1, "{found:?}");
    assert_eq!(found[0].location, "command.demo.core.Send.response[1]");
}

#[test]
fn two_renamed_optional_fields_on_one_wire_name_collide() {
    let found = duplicates(&model(
        INPUT,
        "      - {name: a, type: Optional<String>, naming: {wire: x}}\n      - {name: b, type: Optional<Integer>, naming: {wire: x}}",
    ));
    assert_eq!(found.len(), 1, "{found:?}");
    assert_eq!(found[0].location, "command.demo.core.Send.response[1]");
    assert!(
        found[0]
            .message
            .contains("command.demo.core.Send.response[0]"),
        "{}",
        found[0].message
    );
}

#[test]
fn a_third_field_names_the_first_and_every_later_collision_is_reported() {
    let found = duplicates(&model(
        INPUT,
        "      - {name: reply, type: String}\n      - {name: middle, type: String}\n      - {name: c, type: String, naming: {wire: reply}}\n      - {name: d, type: String, naming: {wire: reply}}",
    ));
    let at: Vec<_> = found.iter().map(|e| e.location.as_str()).collect();
    assert_eq!(
        at,
        [
            "command.demo.core.Send.response[2]",
            "command.demo.core.Send.response[3]"
        ],
        "{found:?}"
    );
    for error in &found {
        assert!(
            error.message.contains("command.demo.core.Send.response[0]"),
            "{}",
            error.message
        );
    }
}

#[test]
fn casing_variants_are_distinct_wire_names() {
    assemble(&model(
        INPUT,
        "      - {name: foo_bar, type: String}\n      - {name: fooBar, type: String}\n      - {name: x, type: String, naming: {wire: FooBar}}",
    ))
    .unwrap_or_else(|errors| panic!("{errors}"));
}

#[test]
fn a_repeated_response_name_is_reported_as_often_as_a_repeated_input_name() {
    let repeated = "      - {name: a, type: String}\n      - {name: a, type: String}";
    let response = duplicates(&model(INPUT, repeated)).len();
    let input = duplicates(&model(repeated, "      - {name: r, type: String}")).len();
    assert_eq!(response, input, "response {response}, input {input}");
}
