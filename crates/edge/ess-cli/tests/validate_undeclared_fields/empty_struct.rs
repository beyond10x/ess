//! A struct with `fields: []` (`ess/32`, beyond10x/ess#649): open, every JSON object is one of its
//! values; closed, `{}` is its only value. Below `ess/32` it keeps `ESS-TYPE-007`, its message
//! unchanged and its hint naming `ess/32`, and the struct stays declared, so the input that uses it
//! is not refused a second time as an undeclared type. An enum with no variants has no value at any
//! format.

use super::validate;

/// A notes model whose command input `extra` is the type `notes.Ack`, declared by `{ack}`.
const MODEL: &str = "\
format: ess/{format}
system: notes
version: v1
domain: notes.ack
summary: An acknowledgment is an object with no declared members.
types:
  - {name: notes.ack.NoteId, kind: newtype, of: Uuid}
{ack}
entities:
  - name: notes.ack.Note
    identity: {name: note_id, type: notes.ack.NoteId}
    fields:
      - {name: body, type: String}
    lifecycle:
      initial: Active
      states: [Active]
      terminal: [Active]
      transitions: []
events:
  - name: notes.ack.NoteCreated
    fields: [{name: note_id, type: notes.ack.NoteId}]
actors:
  - name: notes.ack.Writer
    may: [notes.ack.CreateNote]
commands:
  - name: notes.ack.CreateNote
    input:
      - {name: body, type: String}
      - {name: extra, type: notes.ack.Ack}
    outcomes:
      - name: created
        creates: notes.ack.Note
        instance: note_id
        emits: [notes.ack.NoteCreated]
        payload: {notes.ack.NoteCreated: {note_id: {generated: true}}}
        sets: {body: input.body}
views:
  - name: notes.ack.Notes
    source: notes.ack.Note
    consistency: read_your_writes
    fields:
      - {name: note_id, type: notes.ack.NoteId}
      - {name: body, type: String}
";

const OPEN: &str =
    "  - name: notes.ack.Ack\n    kind: struct\n    undeclared_fields: ignored\n    fields: []";
const CLOSED: &str = "  - name: notes.ack.Ack\n    kind: struct\n    fields: []";
const NO_VARIANTS: &str = "  - name: notes.ack.Ack\n    kind: enum\n    variants: []";

fn model(format: u32, ack: &str) -> String {
    MODEL
        .replace("{format}", &format.to_string())
        .replace("{ack}", ack)
}

#[test]
fn an_open_struct_with_no_fields_validates_from_ess_32() {
    let validated = validate(&model(32, OPEN));
    assert_eq!(validated.code, Some(0), "{}", validated.output);
    assert!(validated.problems.is_empty(), "{}", validated.output);
}

#[test]
fn a_closed_struct_with_no_fields_validates_from_ess_32() {
    let validated = validate(&model(32, CLOSED));
    assert_eq!(validated.code, Some(0), "{}", validated.output);
    assert!(validated.problems.is_empty(), "{}", validated.output);
}

#[test]
fn a_struct_with_no_fields_below_ess_32_is_refused_naming_ess_32() {
    for ack in [OPEN, CLOSED] {
        let validated = validate(&model(31, ack));
        assert_eq!(validated.code, Some(1), "{}", validated.output);
        let refusal = validated.diagnostic("types.notes.ack.Ack.fields");
        assert_eq!(refusal["code"], "ESS-TYPE-007", "{}", validated.output);
        assert_eq!(
            refusal["message"], "`notes.ack.Ack` declares no fields, so no value can be one",
            "the message is unchanged:\n{}",
            validated.output
        );
        let hint = refusal["hint"].as_str().unwrap_or_default();
        assert!(hint.contains("ess/32"), "the hint names `ess/32`: {hint}");
        assert_eq!(
            validated.diagnostics.len(),
            1,
            "no cascade at the input using the struct:\n{}",
            validated.output
        );
    }
}

#[test]
fn an_enum_with_no_variants_is_still_refused_at_ess_32() {
    let validated = validate(&model(32, NO_VARIANTS));
    assert_eq!(validated.code, Some(1), "{}", validated.output);
    let refusal = validated.diagnostic("types.notes.ack.Ack.variants");
    assert_eq!(refusal["code"], "ESS-TYPE-007", "{}", validated.output);
    assert_eq!(
        refusal["message"], "`notes.ack.Ack` declares no variants, so no value can be one",
        "{}",
        validated.output
    );
}
