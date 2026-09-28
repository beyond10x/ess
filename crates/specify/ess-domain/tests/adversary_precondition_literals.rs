//! Adversary cases for structured precondition literals (beyond10x/ess#205).
//!
//! Each case first shows the construct is legal (a control literal the checker admits), then
//! writes a literal the declared type refuses and asserts the checker refuses it too.

use ess_domain::Specification;
use ess_primitives::error::ValidationErrors;

fn assemble(body: &str) -> Result<Specification, ValidationErrors> {
    let raw = ess_domain::spec::RawSpecFile::parse(body)
        .unwrap_or_else(|error| panic!("the document parses: {error}\n{body}"));
    Specification::assemble([(ess_domain::system::Source::new("probe.yaml"), raw)])
}

fn admitted(body: &str) {
    if let Err(errors) = assemble(body) {
        panic!("the model is admitted, got:\n{errors}\n{body}");
    }
}

fn refused(body: &str) {
    assert!(
        assemble(body).is_err(),
        "the model is refused, and was admitted:\n{body}"
    );
}

/// A one-command system whose command takes `input_type` as `probe`, with `types` declared and the
/// precondition sending `literal` for it.
fn system(types: &str, input_type: &str, literal: &str, entity_extra: &str) -> String {
    format!(
        "format: ess/15
system: probe
version: v1
preconditions:
  - command: probe.core.Open
    as: probe.core.Server
    input: {{user_id: u1, probe: {literal}}}
domain: probe.core
types:
{types}
entities:
  - name: probe.core.User
    identity: {{name: user_id, type: String}}
    fields:
      - {{name: probe, type: '{input_type}'}}
{entity_extra}    lifecycle: {{initial: Open, states: [Open], terminal: [Open]}}
actors:
  - {{name: probe.core.Server, may: [probe.core.Open]}}
events:
  - {{name: probe.core.Opened, fields: [{{name: user_id, type: String}}]}}
commands:
  - name: probe.core.Open
    input:
      - {{name: user_id, type: String}}
      - {{name: probe, type: '{input_type}'}}
    outcomes:
      - name: opened
        creates: probe.core.User
        instance: user_id
        sets: {{probe: input.probe}}
        emits: [probe.core.Opened]
        payload: {{probe.core.Opened: {{user_id: input.user_id}}}}
"
    )
}

const FILLER: &str = "  - {name: probe.core.Unused, kind: enum, variants: [A]}\n";

/// A newtype over a list carries its own invariant; the structured path reads only the terminal
/// list, so the newtype's `value.count` invariant is never held against the literal.
#[test]
fn a_newtype_over_a_list_holds_its_invariant() {
    let types = "  - name: probe.core.Codes\n    kind: newtype\n    of: 'List<String>'\n    invariants: [value.count >= 1]\n";
    admitted(&system(types, "probe.core.Codes", "[a]", ""));
    refused(&system(types, "probe.core.Codes", "[]", ""));
}

/// A newtype over a struct: its invariant reads the wrapped struct's field.
#[test]
fn a_newtype_over_a_struct_holds_its_invariant() {
    let types = "  - name: probe.core.Window\n    kind: struct\n    fields:\n      - {name: low, type: Integer}\n  - name: probe.core.Positive\n    kind: newtype\n    of: probe.core.Window\n    invariants: [value.low >= 1]\n";
    admitted(&system(types, "probe.core.Positive", "{low: 2}", ""));
    refused(&system(types, "probe.core.Positive", "{low: 0}", ""));
}

/// A struct invariant over a nested struct's field is a complete path the registry checks, and
/// the literal says exactly what that field is.
#[test]
fn a_struct_invariant_over_a_nested_field_holds() {
    let types = "  - name: probe.core.Inner\n    kind: struct\n    fields:\n      - {name: low, type: Integer}\n  - name: probe.core.Outer\n    kind: struct\n    fields:\n      - {name: inner, type: probe.core.Inner}\n    invariants: [inner.low >= 5]\n";
    admitted(&system(types, "probe.core.Outer", "{inner: {low: 7}}", ""));
    refused(&system(types, "probe.core.Outer", "{inner: {low: 1}}", ""));
}

/// A struct invariant over a list field's count.
#[test]
fn a_struct_invariant_over_a_list_count_holds() {
    let types = "  - name: probe.core.Bag\n    kind: struct\n    fields:\n      - {name: items, type: 'List<String>'}\n    invariants: [items.count >= 1]\n";
    admitted(&system(types, "probe.core.Bag", "{items: [a]}", ""));
    refused(&system(types, "probe.core.Bag", "{items: []}", ""));
}

/// A struct field typed by a newtype over `Optional<..>` admits absence: `null` is admitted for
/// it, so leaving it out must be too.
#[test]
fn a_struct_field_typed_by_an_optional_newtype_may_be_left_out() {
    let types = "  - name: probe.core.Maybe\n    kind: newtype\n    of: 'Optional<String>'\n  - name: probe.core.Account\n    kind: struct\n    fields:\n      - {name: name, type: String}\n      - {name: note, type: probe.core.Maybe}\n";
    admitted(&system(
        types,
        "probe.core.Account",
        "{name: n, note: null}",
        "",
    ));
    admitted(&system(types, "probe.core.Account", "{name: n}", ""));
}

/// A struct whose one written field is named `fixture`, or a map with the key `fixture`, is a
/// literal of the declared type; it is not a fixture reference.
#[test]
fn a_struct_literal_with_a_field_named_fixture_is_a_literal() {
    let types = "  - name: probe.core.Ref\n    kind: struct\n    fields:\n      - {name: fixture, type: String}\n      - {name: note, type: 'Optional<String>'}\n";
    admitted(&system(
        types,
        "probe.core.Ref",
        "{fixture: abc, note: x}",
        "",
    ));
    admitted(&system(types, "probe.core.Ref", "{fixture: abc}", ""));
}

#[test]
fn a_map_literal_with_the_key_fixture_is_a_literal() {
    admitted(&system(
        FILLER,
        "Map<String, String>",
        "{fixture: abc, other: x}",
        "",
    ));
    admitted(&system(FILLER, "Map<String, String>", "{fixture: abc}", ""));
}

/// The entity the precondition creates holds an invariant over the list it stores.
#[test]
fn a_list_literal_the_created_entity_refuses_is_refused() {
    let invariant = "    invariants: [probe.count >= 1]\n";
    admitted(&system(FILLER, "List<String>", "[a]", invariant));
    refused(&system(FILLER, "List<String>", "[]", invariant));
}

/// The same entity-invariant shape over a scalar, for the origin of the case above.
#[test]
fn a_scalar_literal_the_created_entity_refuses_is_refused() {
    let invariant = "    invariants: [probe >= 1]\n";
    admitted(&system(FILLER, "Integer", "2", invariant));
    refused(&system(FILLER, "Integer", "0", invariant));
}
