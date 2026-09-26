//! `starts_with`, `ends_with` and `contains` (beyond10x/ess#95) at validation: where they apply,
//! what they refuse, the `ess/8` gate at every predicate position, and the finite proof declining
//! them. `docs/design/string-predicate-operators.md` is the binding design.

use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::error::{ValidationCode, ValidationErrors};

fn admits(text: &str) -> Result<Specification, ValidationErrors> {
    let raw = RawSpecFile::parse(text)
        .unwrap_or_else(|error| panic!("the document parses: {error}\n{text}"));
    Specification::assemble([(Source::new("strings.yaml"), raw)])
}

fn refusal(text: &str) -> ValidationErrors {
    match admits(text) {
        Ok(_) => panic!("must refuse:\n{text}"),
        Err(errors) => errors,
    }
}

/// One model whose `Open` command is guarded by `{guard}`, with a default branch beside it.
///
/// Every type the checker has to tell apart is an input: text, a newtype of text two deep, an
/// optional text, a list of text, and every other scalar and an aggregate.
fn guarded(format: &str, guard: &str) -> String {
    format!(
        r#"format: {format}
system: calls
version: v1
domain: calls.core
types:
  - name: calls.core.Channel
    kind: enum
    variants: [Web, Phone]
  - name: calls.core.PhoneNumber
    kind: newtype
    of: String
    invariants:
      - value != ""
  - name: calls.core.Caller
    kind: newtype
    of: calls.core.PhoneNumber
  - name: calls.core.Party
    kind: struct
    fields:
      - {{name: number, type: String}}
entities:
  - name: calls.core.Call
    identity: {{name: call_id, type: Uuid}}
    fields:
      - {{name: note, type: String}}
    lifecycle:
      initial: Open
      states: [Open]
      terminal: [Open]
events:
  - name: calls.core.Opened
    fields: [{{name: call_id, type: Uuid}}]
errors:
  - name: calls.core.Refused
    fields: []
commands:
  - name: calls.core.Open
    input:
      - {{name: note, type: String}}
      - {{name: caller, type: calls.core.Caller}}
      - {{name: subject, type: Optional<String>}}
      - {{name: tags, type: List<String>}}
      - {{name: channel, type: calls.core.Channel}}
      - {{name: ref, type: Uuid}}
      - {{name: at, type: Timestamp}}
      - {{name: wait, type: Duration}}
      - {{name: blob, type: Bytes}}
      - {{name: count, type: Integer}}
      - {{name: gift, type: Boolean}}
      - {{name: party, type: calls.core.Party}}
    outcomes:
      - name: opened
        when: {guard}
        creates: calls.core.Call
        instance: call_id
        sets: {{note: input.note}}
        emits: [calls.core.Opened]
        payload:
          calls.core.Opened:
            call_id: {{generated: true}}
      - name: refused
        error: calls.core.Refused
"#
    )
}

fn codes(errors: &ValidationErrors) -> Vec<ValidationCode> {
    errors.as_slice().iter().map(|error| error.code).collect()
}

#[test]
fn a_string_or_a_newtype_of_one_at_any_depth_or_through_optional_is_admitted() {
    for guard in [
        r#"{note: {starts_with: "+44"}}"#,
        r#"{caller: {starts_with: "+44"}}"#,
        r#"{subject: {contains: "urgent"}}"#,
        r#"{party.number: {ends_with: "0"}}"#,
        r#"{exists: {in: tags, as: t, that: {t: {starts_with: "vip"}}}}"#,
        r#"{not: {note: {contains: "spam"}}}"#,
        r#"{note: {starts_with: "A", ends_with: "0"}}"#,
    ] {
        if let Err(errors) = admits(&guarded("ess/8", guard)) {
            panic!("{guard} is admitted: {errors}");
        }
    }
}

#[test]
fn every_other_type_is_refused_as_a_type_mismatch_naming_the_operator() {
    for (field, declared) in [
        ("channel", "calls.core.Channel"),
        ("ref", "Uuid"),
        ("at", "Timestamp"),
        ("wait", "Duration"),
        ("blob", "Bytes"),
        ("count", "Integer"),
        ("gift", "Boolean"),
        ("party", "calls.core.Party"),
        ("tags", "List<String>"),
    ] {
        let errors = refusal(&guarded(
            "ess/8",
            &format!("{{{field}: {{starts_with: \"A\"}}}}"),
        ));
        let error = errors
            .as_slice()
            .iter()
            .find(|error| error.code == ValidationCode::TypeMismatch)
            .unwrap_or_else(|| panic!("{field}: {:?}", codes(&errors)));
        assert!(
            error
                .message
                .contains("operator `starts_with` does not admit")
                && error.message.contains(declared),
            "{field}: {}",
            error.message
        );
    }
}

#[test]
fn a_number_or_boolean_operand_is_refused_and_told_to_quote_it() {
    for operand in ["+44", "44", "true"] {
        let errors = refusal(&guarded(
            "ess/8",
            &format!("{{note: {{starts_with: {operand}}}}}"),
        ));
        let error = errors
            .as_slice()
            .iter()
            .find(|error| error.code == ValidationCode::TypeMismatch)
            .unwrap_or_else(|| panic!("{operand}: {:?}", codes(&errors)));
        assert!(
            error.message.contains("not text") && error.message.contains("quote"),
            "{operand}: {}",
            error.message
        );
    }
}

#[test]
fn an_empty_operand_is_refused_as_an_empty_declaration_pointing_at_defined() {
    for operator in ["starts_with", "ends_with", "contains"] {
        let errors = refusal(&guarded(
            "ess/8",
            &format!("{{note: {{{operator}: \"\"}}}}"),
        ));
        let error = errors
            .as_slice()
            .iter()
            .find(|error| error.code == ValidationCode::EmptyDeclaration)
            .unwrap_or_else(|| panic!("{operator}: {:?}", codes(&errors)));
        assert!(error.message.contains("defined(note)"), "{}", error.message);
    }
}

#[test]
fn a_literal_that_names_a_declared_field_is_refused_and_not_told_about_window_members() {
    for operand in ["gift", r#""gift""#, "note"] {
        let errors = refusal(&guarded(
            "ess/8",
            &format!("{{subject: {{contains: {operand}}}}}"),
        ));
        let error = errors
            .as_slice()
            .iter()
            .find(|error| error.code == ValidationCode::TypeMismatch)
            .unwrap_or_else(|| panic!("{operand}: {:?}", codes(&errors)));
        assert!(
            error.message.contains("as the text literal")
                && error.message.contains("compares with a literal only")
                && !error.message.contains("window."),
            "{operand}: {}",
            error.message
        );
    }
}

#[test]
fn a_path_the_owner_does_not_declare_is_refused_as_today() {
    let errors = refusal(&guarded("ess/8", r#"{nothing: {starts_with: "A"}}"#));
    assert!(
        errors.contains(ValidationCode::UnobservableFact)
            || errors.contains(ValidationCode::UndeclaredReference),
        "{:?}",
        codes(&errors)
    );
}

#[test]
fn a_guard_and_its_complement_still_need_a_default_because_the_prover_does_not_read_text() {
    let text = guarded("ess/8", r#"{note: {starts_with: "+44"}}"#).replace(
        "      - name: refused\n        error: calls.core.Refused\n",
        "      - name: refused\n        when: {not: {note: {starts_with: \"+44\"}}}\n        error: calls.core.Refused\n",
    );
    let errors = refusal(&text);
    assert!(
        errors.contains(ValidationCode::NonExhaustiveBranches),
        "{:?}",
        codes(&errors)
    );
}

const GATE: &str = "string predicate operators require specification format ess/8";

fn gated(errors: &ValidationErrors) -> bool {
    errors.as_slice().iter().any(|error| {
        error.code == ValidationCode::UnsupportedFormatVersion && error.message.contains(GATE)
    })
}

/// Every predicate position, each carrying a string operator, in a document that is valid at
/// `ess/8`. At `ess/7` each is refused by the one gate, and the same document with a comparison in
/// place of the operator is not, so formats stay cumulative.
fn positions() -> Vec<(&'static str, String)> {
    let subject = include_str!("../../../verify/ess-conformance/tests/fixtures/subject-history.yaml")
        .replace("format: ess/6", "format: ess/8")
        .replace(
            "  - name: calls.core.Answer\n    input: [{name: call_id, type: Uuid}]",
            "  - name: calls.core.Answer\n    input: [{name: call_id, type: Uuid}, {name: via, type: String}]",
        );
    let selection =
        include_str!("../../../generate/ess-synth/tests/fixtures/binding-selection.yaml")
            .replace("format: ess/3", "format: ess/8");
    vec![
        ("a command guard", guarded("ess/8", r#"{note: {starts_with: "+44"}}"#)),
        (
            "a newtype invariant",
            guarded("ess/8", "note == x").replace(
                "      - value != \"\"",
                "      - value: {starts_with: \"+\"}",
            ),
        ),
        (
            "a struct invariant",
            guarded("ess/8", "note == x").replace(
                "      - {name: number, type: String}\n",
                "      - {name: number, type: String}\n    invariants:\n      - number: {ends_with: \"0\"}\n",
            ),
        ),
        (
            "an entity invariant",
            guarded("ess/8", "note == x").replace(
                "      - {name: note, type: String}\n    lifecycle:",
                "      - {name: note, type: String}\n    invariants:\n      - not: {note: {contains: \"spam\"}}\n    lifecycle:",
            ),
        ),
        (
            "a view filter",
            guarded("ess/8", "note == x")
                + "views:\n  - name: calls.core.Noted\n    source: calls.core.Call\n    consistency: eventual\n    filter: {note: {contains: \"urgent\"}}\n    fields:\n      - {name: call_id, type: Uuid}\n",
        ),
        (
            "the input guard beside when_subject",
            subject.replace(
                "        when_subject: {field: answer_history, equals: Answered}\n",
                "        when_subject: {field: answer_history, equals: Answered}\n        when: {via: {ends_with: \"0\"}}\n",
            ),
        ),
        (
            "the input guard beside external",
            subject.replace(
                "      - name: gone\n        wrong_state: true\n        error: calls.core.Gone\n  - name: calls.core.Report",
                "      - name: provider-refused\n        external: the provider refuses\n        when: {via: {starts_with: \"sip:\"}}\n        error: calls.core.Gone\n      - name: gone\n        wrong_state: true\n        error: calls.core.Gone\n  - name: calls.core.Report",
            ),
        ),
        (
            "a binding selection",
            selection.replace(
                "          where: 'item.id != \"\"'",
                "          where: {item.id: {starts_with: \"leg-\"}}",
            ),
        ),
    ]
}

#[test]
fn every_predicate_position_is_admitted_at_ess_8_and_refused_below_it_by_one_gate() {
    for (position, text) in positions() {
        if let Err(errors) = admits(&text) {
            panic!("{position} is admitted at ess/8: {errors}\n{text}");
        }
        for below in ["ess/7", "ess/1"] {
            let older = text.replace("format: ess/8", &format!("format: {below}"));
            let errors = refusal(&older);
            assert!(gated(&errors), "{position} at {below}: {errors}");
        }
    }
}

#[test]
fn the_state_positions_are_gated_too() {
    // The input guard beside `when_subject_state` and beside `when_state_changes`.
    let fixture =
        include_str!("../../../verify/ess-conformance/tests/fixtures/subject-history.yaml")
            .replace("format: ess/6", "format: ess/8");
    let text = fixture.replace(
        "views:\n",
        "  - name: calls.core.Note\n    input: [{name: call_id, type: Uuid}, {name: via, type: String}]\n    outcomes:\n      - name: restated\n        when_subject_state: Ended\n        when: {via: {contains: \"x\"}}\n        updates: calls.core.Call\n        instance: call_id\n        emits: [calls.core.Observed]\n      - name: fallback\n        updates: calls.core.Call\n        instance: call_id\n        emits: [calls.core.Observed]\nviews:\n",
    );
    if let Err(errors) = admits(&text) {
        panic!("when_subject_state with a string guard is admitted at ess/8: {errors}\n{text}");
    }
    assert!(gated(&refusal(
        &text.replace("format: ess/8", "format: ess/7")
    )));

    // The adopter shape of `tests/state_change.rs`, with a text guard on the first report.
    let changes = r#"format: ess/8
system: demo
version: v1
domain: demo.room
types:
  - name: demo.room.Reported
    kind: enum
    variants: [Pending, Added, Gone]
entities:
  - name: demo.room.Member
    identity: {name: member_id, type: Uuid}
    fields:
      - {name: note, type: String}
    lifecycle:
      initial: Pending
      states: [Pending, Added, Gone]
      terminal: [Gone]
      transitions:
        - {name: join, from: [Pending, Added, Gone], to: Added}
        - {name: leave, from: [Pending, Added, Gone], to: Gone}
events:
  - name: demo.room.MemberUpdated
    fields: [{name: member_id, type: Uuid}]
commands:
  - name: demo.room.Report
    input:
      - {name: member_id, type: Uuid}
      - {name: state, type: demo.room.Reported}
      - {name: note, type: String}
    outcomes:
      - name: joined
        when: {all: [state == Added, {note: {starts_with: "vip"}}]}
        when_state_changes: true
        moves: demo.room.Member.join
        instance: member_id
        emits: [demo.room.MemberUpdated]
        payload:
          demo.room.MemberUpdated:
            member_id: input.member_id
      - name: gone
        when: state == Gone
        moves: demo.room.Member.leave
        instance: member_id
        emits: [demo.room.MemberUpdated]
        payload:
          demo.room.MemberUpdated:
            member_id: input.member_id
      - name: refreshed
        updates: demo.room.Member
        instance: member_id
        sets:
          note: input.note
        emits: [demo.room.MemberUpdated]
        payload:
          demo.room.MemberUpdated:
            member_id: input.member_id
"#;
    if let Err(errors) = admits(changes) {
        panic!("when_state_changes with a string guard is admitted at ess/8: {errors}\n{changes}");
    }
    assert!(gated(&refusal(
        &changes.replace("format: ess/8", "format: ess/7")
    )));
}

#[test]
fn a_document_without_the_operators_keeps_every_older_format() {
    // ess/4 is the oldest format this model's generated payload source is written in.
    for format in ["ess/4", "ess/7", "ess/8"] {
        if let Err(errors) = admits(&guarded(format, "note == x")) {
            panic!("{format}: {errors}");
        }
    }
}

#[test]
fn a_selection_takes_a_text_literal_no_longer_than_an_admitted_read() {
    let selection =
        include_str!("../../../generate/ess-synth/tests/fixtures/binding-selection.yaml")
            .replace("format: ess/3", "format: ess/8");
    let with = |guard: &str| {
        selection.replace(
            "          where: 'item.id != \"\"'",
            &format!("          where: {guard}"),
        )
    };
    for guard in [
        r#"{item.id: {contains: "x"}}"#,
        r#"{not: {item.from: {ends_with: "0"}}}"#,
    ] {
        if let Err(errors) = admits(&with(guard)) {
            panic!("{guard}: {errors}");
        }
    }
    let long = "x".repeat(4097);
    for guard in [
        format!("{{item.id: {{starts_with: \"{long}\"}}}}"),
        r#"{item.id: {starts_with: ""}}"#.to_owned(),
        r#"{item.domain: {starts_with: "In"}}"#.to_owned(),
        "{item.id: {starts_with: 44}}".to_owned(),
    ] {
        let errors = refusal(&with(&guard));
        assert!(!errors.is_empty(), "{guard}");
    }
}
