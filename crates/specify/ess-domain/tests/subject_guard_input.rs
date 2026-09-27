//! `input.<field>` in a `when_subject` predicate (beyond10x/ess#157, E6) and `equals_ignore_case` /
//! `in_ignore_case` (beyond10x/ess#140, E7) at validation: what they admit under `ess/15`, the refusal
//! under every earlier header, the typing each is held to, and the finite proof declining both.
//! `docs/design/value-expressions.md` §§ E6, E7 is the binding design.

use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::error::{ValidationCode, ValidationErrors};

fn admits(text: &str) -> Result<Specification, ValidationErrors> {
    let raw = RawSpecFile::parse(text)
        .unwrap_or_else(|error| panic!("the document parses: {error}\n{text}"));
    Specification::assemble([(Source::new("guards.yaml"), raw)])
}

fn refusal(text: &str) -> ValidationErrors {
    match admits(text) {
        Ok(_) => panic!("must refuse:\n{text}"),
        Err(errors) => errors,
    }
}

fn codes(errors: &ValidationErrors) -> Vec<ValidationCode> {
    errors.as_slice().iter().map(|error| error.code).collect()
}

/// The #157 repro: a stop is ignored when it names a recording other than the call's current one.
///
/// `{guard}` is the `when_subject` predicate, `{default}` the guard of the sibling (empty for a
/// genuine default), and the entity and the input carry one field of every kind the typing tells
/// apart.
fn recording(format: &str, guard: &str, default: &str) -> String {
    format!(
        r#"format: {format}
system: calls
version: v1
domain: calls.rec
types:
  - name: calls.rec.RecordingId
    kind: newtype
    of: String
  - name: calls.rec.Mode
    kind: enum
    variants: [Audio, Video]
entities:
  - name: calls.rec.Call
    identity: {{name: call_id, type: Uuid}}
    fields:
      - {{name: recording_id, type: calls.rec.RecordingId}}
      - {{name: label, type: String}}
      - {{name: attempts, type: Integer}}
      - {{name: mode, type: calls.rec.Mode}}
    lifecycle:
      initial: Recording
      states: [Recording, Stopped]
      terminal: [Stopped]
      transitions:
        - {{name: stop, from: [Recording], to: Stopped}}
events:
  - name: calls.rec.Started
    fields: [{{name: call_id, type: Uuid}}]
  - name: calls.rec.Stopped
    fields: []
commands:
  - name: calls.rec.Start
    input:
      - {{name: recording_id, type: calls.rec.RecordingId}}
    outcomes:
      - name: started
        creates: calls.rec.Call
        instance: call_id
        sets: {{recording_id: input.recording_id, label: "x", attempts: 0, mode: Audio}}
        emits: [calls.rec.Started]
        payload:
          calls.rec.Started:
            call_id: {{generated: true}}
  - name: calls.rec.Stop
    input:
      - {{name: call_id, type: Uuid}}
      - {{name: recording_id, type: calls.rec.RecordingId}}
      - {{name: label, type: String}}
      - {{name: attempts, type: Integer}}
      - {{name: mode, type: calls.rec.Mode}}
      - {{name: maybe, type: Optional<String>}}
    outcomes:
      - name: other-recording
        when_subject:
          predicate: {guard}
        preserves: calls.rec.Call
        instance: call_id
      - name: stopped
        moves: calls.rec.Call.stop
        instance: call_id
        emits: [calls.rec.Stopped]{default}
"#
    )
}

const GUARD: &str = "recording_id != input.recording_id";

#[test]
fn the_157_repro_validates_under_ess_15() {
    admits(&recording("ess/15", GUARD, "")).unwrap_or_else(|errors| panic!("{errors}"));
}

#[test]
fn every_comparison_operator_and_connective_reads_the_input() {
    for guard in [
        "recording_id == input.recording_id",
        "label == input.label",
        "attempts < input.attempts",
        "mode == input.mode",
        "{all: [label == x, recording_id != input.recording_id]}",
        "{not: {recording_id: {eq: input.recording_id}}}",
        "label == input.maybe",
    ] {
        admits(&recording("ess/15", guard, ""))
            .unwrap_or_else(|errors| panic!("{guard}: {errors}"));
    }
}

#[test]
fn every_earlier_header_refuses_an_input_operand_with_the_format_it_needs() {
    for format in ["ess/9", "ess/13", "ess/14"] {
        let errors = refusal(&recording(format, GUARD, ""));
        let text = errors.to_string();
        assert_eq!(
            codes(&errors),
            [ValidationCode::UnsupportedFormatVersion],
            "{format}: {text}"
        );
        assert!(text.contains("ess/15"), "{format}: {text}");
        assert!(text.contains("when_subject"), "{format}: {text}");
    }
}

#[test]
fn operands_of_types_that_do_not_compare_are_refused() {
    for guard in [
        "attempts == input.label",
        "label == input.attempts",
        "recording_id == input.attempts",
    ] {
        let errors = refusal(&recording("ess/15", guard, ""));
        assert!(
            codes(&errors).contains(&ValidationCode::TypeMismatch),
            "{guard}: {errors}"
        );
    }
}

#[test]
fn an_input_field_the_command_does_not_declare_is_refused_by_name() {
    let errors = refusal(&recording("ess/15", "recording_id != input.recording", ""));
    let text = errors.to_string();
    assert!(
        codes(&errors).contains(&ValidationCode::UnobservableFact)
            || codes(&errors).contains(&ValidationCode::UndeclaredReference),
        "{text}"
    );
    assert!(text.contains("input.recording"), "{text}");
}

#[test]
fn a_selector_past_an_input_scalar_is_refused() {
    let errors = refusal(&recording(
        "ess/15",
        "recording_id != input.recording_id.x",
        "",
    ));
    assert!(
        codes(&errors).contains(&ValidationCode::UnobservableFact),
        "{errors}"
    );
}

/// A bare word on the right is a literal, as everywhere: `input` alone is the text "input", and an
/// input on the left compares with the text a bare stored-field name spells (#74), not the field.
#[test]
fn a_bare_word_on_the_right_stays_a_literal() {
    admits(&recording("ess/15", "recording_id != input", ""))
        .unwrap_or_else(|errors| panic!("{errors}"));
    let errors = refusal(&recording(
        "ess/15",
        "input.recording_id != recording_id",
        "",
    ));
    assert!(
        codes(&errors).contains(&ValidationCode::TypeMismatch),
        "{errors}"
    );
}

/// The finite prover cannot enumerate an input's value against a stored one, so the guard is open
/// and the command needs a genuine default, as for any open guard.
#[test]
fn the_guard_is_open_so_a_guarded_sibling_needs_a_default() {
    let guarded_sibling = recording(
        "ess/15",
        GUARD,
        "\n        when_subject:\n          predicate: recording_id == input.recording_id",
    );
    let errors = refusal(&guarded_sibling);
    assert!(
        codes(&errors).contains(&ValidationCode::NonExhaustiveBranches),
        "{errors}"
    );
    // Even over an enum, which alone the prover would close.
    let enum_guard = recording(
        "ess/15",
        "mode == input.mode",
        "\n        when_subject:\n          predicate: mode != input.mode",
    );
    assert!(
        codes(&refusal(&enum_guard)).contains(&ValidationCode::NonExhaustiveBranches),
        "an enum comparison with the input is still open"
    );
}

/// A stored field named `input` keeps being read as that field: no document that validated before
/// changes meaning under the new header.
#[test]
fn a_stored_field_named_input_is_still_read_as_the_stored_field() {
    let text = recording("ess/15", "input.name == x", "")
        .replace(
            "      - {name: label, type: String}\n",
            "      - {name: label, type: String}\n      - {name: input, type: calls.rec.Party}\n",
        )
        .replace(
            "types:\n",
            "types:\n  - name: calls.rec.Party\n    kind: struct\n    fields:\n      - {name: name, type: String}\n",
        )
        .replace(
            "sets: {recording_id: input.recording_id, label: \"x\", attempts: 0, mode: Audio}",
            "sets: {recording_id: input.recording_id, label: \"x\", attempts: 0, mode: Audio, input: {name: n}}",
        );
    // `input.name` is the stored struct's member; the command declares no input `name`.
    admits(&text).unwrap_or_else(|errors| panic!("{errors}"));
}

// ---- E7 ------------------------------------------------------------------------------------------

/// The #140 repro, with `lower(source) == "web"` written as the operator it asked for.
fn lookup(format: &str, guard: &str) -> String {
    format!(
        r"format: {format}
system: demo
version: v1
domain: demo.orders
types:
  - name: demo.orders.Source
    kind: newtype
    of: String
  - name: demo.orders.Channel
    kind: enum
    variants: [Web, Phone]
errors:
  - name: demo.orders.UnknownSource
    summary: The source is not a known source.
    fields: []
events:
  - name: demo.orders.Looked
    fields:
      - {{name: source, type: Optional<String>}}
commands:
  - name: demo.orders.Lookup
    input:
      - {{name: source, type: Optional<String>}}
      - {{name: typed, type: demo.orders.Source}}
      - {{name: channel, type: demo.orders.Channel}}
      - {{name: count, type: Integer}}
      - {{name: at, type: Timestamp}}
      - {{name: tags, type: List<String>}}
    outcomes:
      - name: unknown-source
        when: {guard}
        error: demo.orders.UnknownSource
      - name: found
        emits: [demo.orders.Looked]
        payload:
          demo.orders.Looked:
            source: input.source
"
    )
}

#[test]
fn the_140_repro_validates_under_ess_15() {
    for guard in [
        "{source: {equals_ignore_case: web}}",
        "{not: {source: {in_ignore_case: [web, phone]}}}",
        "{typed: {equals_ignore_case: WEB}}",
        "{typed: {in_ignore_case: [a]}}",
        "{exists: {in: tags, as: t, that: {t: {equals_ignore_case: vip}}}}",
        r#"{source: {equals_ignore_case: ""}}"#,
    ] {
        admits(&lookup("ess/15", guard)).unwrap_or_else(|errors| panic!("{guard}: {errors}"));
    }
}

#[test]
fn every_earlier_header_refuses_the_operators_with_the_format_they_need() {
    for format in ["ess/8", "ess/14"] {
        for guard in [
            "{source: {equals_ignore_case: web}}",
            "{source: {in_ignore_case: [web]}}",
        ] {
            let errors = refusal(&lookup(format, guard));
            let text = errors.to_string();
            assert_eq!(
                codes(&errors),
                [ValidationCode::UnsupportedFormatVersion],
                "{format} {guard}: {text}"
            );
            assert!(text.contains("ess/15"), "{text}");
        }
    }
}

#[test]
fn only_string_and_its_newtypes_are_folded() {
    for guard in [
        "{channel: {equals_ignore_case: web}}",
        "{count: {equals_ignore_case: web}}",
        "{at: {in_ignore_case: [web]}}",
        "{tags: {equals_ignore_case: web}}",
    ] {
        let errors = refusal(&lookup("ess/15", guard));
        assert!(
            codes(&errors).contains(&ValidationCode::TypeMismatch),
            "{guard}: {errors}"
        );
    }
}

#[test]
fn a_literal_that_is_not_text_is_refused() {
    for guard in [
        "{source: {equals_ignore_case: 44}}",
        "{source: {in_ignore_case: [web, true]}}",
    ] {
        let errors = refusal(&lookup("ess/15", guard));
        assert!(
            codes(&errors).contains(&ValidationCode::TypeMismatch),
            "{guard}: {errors}"
        );
    }
}

#[test]
fn an_empty_list_is_refused_because_it_matches_nothing() {
    let errors = refusal(&lookup("ess/15", "{source: {in_ignore_case: []}}"));
    assert!(
        codes(&errors).contains(&ValidationCode::EmptyDeclaration),
        "{errors}"
    );
}

#[test]
fn the_operators_are_gated_under_when_subject_too() {
    let text = recording("ess/14", "{label: {equals_ignore_case: x}}", "");
    let errors = refusal(&text);
    assert!(
        codes(&errors).contains(&ValidationCode::UnsupportedFormatVersion),
        "{errors}"
    );
    admits(&recording("ess/15", "{label: {equals_ignore_case: x}}", ""))
        .unwrap_or_else(|errors| panic!("{errors}"));
}

/// A fold is open to the finite prover, so it needs a default beside it like any open guard.
#[test]
fn a_fold_guard_is_open_to_the_finite_proof() {
    let text = lookup("ess/15", "{typed: {equals_ignore_case: web}}").replace(
        "      - name: found\n",
        "      - name: found\n        when: {not: {typed: {equals_ignore_case: web}}}\n",
    );
    let errors = refusal(&text);
    assert!(
        codes(&errors).contains(&ValidationCode::NonExhaustiveBranches),
        "{errors}"
    );
}
