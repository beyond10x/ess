//! Adversary pass 2 for structured precondition literals (beyond10x/ess#205).
//!
//! Two surfaces the correction left: the row a precondition creates is checked only for fields its
//! branch `sets:` from a plain `input.<field>`, so a field written by a fallback, a struct source or
//! a literal in the outcome is never held to the entity's invariants; and the branch a precondition
//! selects is decided over its scalar inputs only, so a guard over a list, map or struct literal the
//! unit now admits is undecided.

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

/// One creating command over `probe.core.User`, whose `rank: Integer` field carries the invariant
/// `rank >= 1`. `input` is the precondition's input map, `inputs` the command's extra input lines,
/// `sets` the branch's `sets:` map, `types` extra type lines.
fn ranked(format: &str, input: &str, inputs: &str, sets: &str, types: &str) -> String {
    format!(
        "format: {format}
system: probe
version: v1
preconditions:
  - command: probe.core.Open
    as: probe.core.Server
    input: {{user_id: u1{input}}}
domain: probe.core
types:
  - {{name: probe.core.Unused, kind: enum, variants: [A]}}
{types}entities:
  - name: probe.core.User
    identity: {{name: user_id, type: String}}
    fields:
      - {{name: rank, type: Integer}}
      - {{name: window, type: 'Optional<probe.core.Window>'}}
    invariants: [rank >= 1]
    lifecycle: {{initial: Open, states: [Open], terminal: [Open]}}
actors:
  - {{name: probe.core.Server, may: [probe.core.Open]}}
events:
  - {{name: probe.core.Opened, fields: [{{name: user_id, type: String}}]}}
commands:
  - name: probe.core.Open
    input:
      - {{name: user_id, type: String}}
{inputs}    outcomes:
      - name: opened
        creates: probe.core.User
        instance: user_id
        sets: {sets}
        emits: [probe.core.Opened]
        payload: {{probe.core.Opened: {{user_id: input.user_id}}}}
"
    )
}

const WINDOW: &str = "  - name: probe.core.Window\n    kind: struct\n    fields:\n      - {name: low, type: Integer}\n";

/// The control the correction does hold: a rank set from `input.rank` is checked, `0` refused.
#[test]
fn control_a_rank_set_from_the_input_is_held_to_the_invariant() {
    let inputs = "      - {name: rank, type: Integer}\n";
    let sets = "{rank: input.rank}";
    admitted(&ranked("ess/15", ", rank: 2", inputs, sets, WINDOW));
    refused(&ranked("ess/15", ", rank: 0", inputs, sets, WINDOW));
}

/// The same rank, set through a literal fallback `{input: rank, else: 5}` (ess/16): the precondition
/// sends `rank: 0`, so the row holds `0` exactly as it does above, and it is admitted.
#[test]
fn a_rank_set_through_a_fallback_from_a_literal_input_is_held_to_the_invariant() {
    let inputs = "      - {name: rank, type: 'Optional<Integer>'}\n";
    let sets = "{rank: {input: rank, else: 5}}";
    admitted(&ranked("ess/16", ", rank: 2", inputs, sets, WINDOW));
    refused(&ranked("ess/16", ", rank: 0", inputs, sets, WINDOW));
}

/// The precondition leaves the optional `rank` out, so the row holds the fallback `0`: known from
/// the document alone, false, and admitted as unknown.
#[test]
fn an_omitted_input_whose_fallback_breaks_the_invariant_is_refused() {
    let inputs = "      - {name: rank, type: 'Optional<Integer>'}\n";
    admitted(&ranked(
        "ess/16",
        ", rank: 2",
        inputs,
        "{rank: {input: rank, else: 0}}",
        WINDOW,
    ));
    refused(&ranked(
        "ess/16",
        "",
        inputs,
        "{rank: {input: rank, else: 0}}",
        WINDOW,
    ));
}

/// A struct-typed field set one source per member, `{window: {low: input.low}}`, with the entity
/// invariant over the member: the precondition's literal `low: 0` makes it false.
#[test]
fn a_struct_sourced_field_the_invariant_forbids_is_refused() {
    let entity_invariant = |low: &str| {
        ranked(
            "ess/15",
            &format!(", low: {low}"),
            "      - {name: low, type: Integer}\n",
            "{rank: 1, window: {low: input.low}}",
            WINDOW,
        )
        .replace(
            "invariants: [rank >= 1]",
            "invariants: [rank >= 1, window.low >= 1]",
        )
    };
    admitted(&entity_invariant("2"));
    refused(&entity_invariant("0"));
}

/// A literal the branch itself writes, `sets: {rank: 0}`: every row this precondition creates breaks
/// `rank >= 1`, and the interpreter and the explorers refuse it as setup.
#[test]
fn a_literal_set_the_invariant_forbids_is_refused() {
    admitted(&ranked("ess/15", "", "", "{rank: 2}", WINDOW));
    refused(&ranked("ess/15", "", "", "{rank: 0}", WINDOW));
}

/// Two branches of one command: `tagged` when `tags.count >= 1`, the default `untagged`. `tags` is a
/// list literal the unit admits; the guard over its count is decided by every reader but this one.
fn guarded(guard: &str, input: &str, input_type: &str) -> String {
    format!(
        "format: ess/15
system: probe
version: v1
preconditions:
  - command: probe.core.Open
    as: probe.core.Server
    input: {{user_id: u1, probe: {input}}}
domain: probe.core
types:
{WINDOW}entities:
  - name: probe.core.User
    identity: {{name: user_id, type: String}}
    lifecycle: {{initial: Open, states: [Open], terminal: [Open]}}
actors:
  - {{name: probe.core.Server, may: [probe.core.Open]}}
events:
  - {{name: probe.core.Opened, fields: [{{name: user_id, type: String}}]}}
  - {{name: probe.core.Plain}}
commands:
  - name: probe.core.Open
    input:
      - {{name: user_id, type: String}}
      - {{name: probe, type: '{input_type}'}}
    outcomes:
      - name: guarded
        when: '{guard}'
        creates: probe.core.User
        instance: user_id
        emits: [probe.core.Opened]
        payload: {{probe.core.Opened: {{user_id: input.user_id}}}}
      - name: plain
        emits: [probe.core.Plain]
"
    )
}

/// A guard over a list literal's count.
#[test]
fn a_guard_over_a_list_literals_count_is_decided() {
    admitted(&guarded("probe.count >= 1", "[a]", "List<String>"));
}

/// A guard over a struct literal's member.
#[test]
fn a_guard_over_a_struct_literals_member_is_decided() {
    admitted(&guarded("probe.low >= 1", "{low: 2}", "probe.core.Window"));
}
