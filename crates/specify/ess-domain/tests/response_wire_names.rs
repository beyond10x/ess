//! A command response is a wire namespace of its own: the specification pass refuses two response
//! fields of one wire name, as it does for input fields, and lets a response reuse an input's name.

use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_primitives::error::{ValidationCode, ValidationErrors};

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

const INPUT: &str = "      - {name: text, type: String}";

#[test]
fn two_response_fields_of_one_wire_name_are_refused_at_the_later_field() {
    let text = model(
        INPUT,
        "      - {name: reply, type: String}\n      - {name: other, type: String, naming: {wire: reply}}",
    );
    let Err(errors) = assemble(&text) else {
        panic!("validated, and should have been refused")
    };
    let found: Vec<_> = errors
        .into_iter()
        .filter(|error| error.code == ValidationCode::DuplicateDeclaration)
        .collect();
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
fn a_response_may_share_a_wire_name_with_an_input_field() {
    let text = model(
        INPUT,
        "      - {name: reply, type: String, naming: {wire: text}}",
    );
    assemble(&text).unwrap_or_else(|errors| panic!("{errors}"));
}
