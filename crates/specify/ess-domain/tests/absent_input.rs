//! A command declares the outcome for a request whose input is absent as a whole
//! (`input_absent: true`, ess/16; beyond10x/ess#170, `docs/design/outcome-shapes.md`).
//!
//! The marker sits beside `wrong_state:` and `unknown_instance:`: one per command, naming the
//! error it reports and nothing else. Below `ess/16` it is refused with
//! `unsupported_format_version`. The guard the issue had to write instead — `not defined(text)`
//! over a required input — validated and could never be witnessed; it is now refused at validate
//! time, with a hint naming the marker.

use ess_domain::command::{OutcomeCondition, RawOutcome, TestStrategy};
use ess_domain::Specification;
use ess_primitives::error::{ValidationCode, ValidationErrors};

fn assemble(body: &str) -> Result<Specification, ValidationErrors> {
    let raw = ess_domain::spec::RawSpecFile::parse(body)
        .unwrap_or_else(|error| panic!("the document parses: {error}\n{body}"));
    Specification::assemble([(ess_domain::system::Source::new("notes.yaml"), raw)])
}

fn admitted(body: &str) -> Specification {
    assemble(body).unwrap_or_else(|errors| panic!("the model is admitted: {errors}\n{body}"))
}

fn refused(body: &str) -> ValidationErrors {
    match assemble(body) {
        Ok(_) => panic!("the model is refused:\n{body}"),
        Err(errors) => errors,
    }
}

fn assert_code(errors: &ValidationErrors, code: ValidationCode, needle: &str) {
    assert!(
        errors
            .as_slice()
            .iter()
            .any(|error| error.code == code && error.to_string().contains(needle)),
        "expected {code:?} mentioning `{needle}`, got:\n{errors}"
    );
}

/// The #170 repro, with `outcomes` as the branches of `demo.notes.SubmitNote` and `input` as its
/// input list.
fn notes(format: u32, input: &str, outcomes: &str) -> String {
    format!(
        "format: ess/{format}
system: demo
version: v1
domain: demo.notes
types:
  - {{name: demo.notes.NoteId, kind: newtype, of: Uuid}}
entities:
  - name: demo.notes.Note
    identity: {{name: note_id, type: demo.notes.NoteId}}
    fields:
      - {{name: text, type: String}}
    lifecycle: {{initial: Open, states: [Open], terminal: [Open], transitions: []}}
actors:
  - {{name: demo.notes.Service, may: [demo.notes.SubmitNote]}}
errors:
  - name: demo.notes.BodyMissing
    summary: The request carried no body.
    fields: []
commands:
  - name: demo.notes.SubmitNote
    input:
{input}    outcomes:
{outcomes}      - name: submitted
        creates: demo.notes.Note
        instance: note_id
        sets: {{text: input.text}}
        emits: [demo.notes.NoteSubmitted]
        payload:
          demo.notes.NoteSubmitted: {{note_id: {{generated: true}}}}
events:
  - name: demo.notes.NoteSubmitted
    fields:
      - {{name: note_id, type: demo.notes.NoteId}}
views:
  - name: demo.notes.NoteDetails
    source: demo.notes.Note
    consistency: read_your_writes
    fields:
      - {{name: note_id, type: demo.notes.NoteId}}
"
    )
}

const TEXT: &str = "      - {name: text, type: String}\n";

const ABSENT: &str =
    "      - {name: body-missing, input_absent: true, error: demo.notes.BodyMissing}\n";

fn submit_outcome<'s>(spec: &'s Specification, name: &str) -> &'s ess_domain::command::Outcome {
    spec.commands()
        .get(&"demo.notes.SubmitNote".parse().unwrap())
        .expect("SubmitNote is declared")
        .outcomes
        .iter()
        .find(|outcome| outcome.name.as_str() == name)
        .unwrap_or_else(|| panic!("SubmitNote/{name} is declared"))
}

#[test]
fn an_absent_input_has_its_own_outcome() {
    let spec = admitted(&notes(16, TEXT, ABSENT));
    let outcome = submit_outcome(&spec, "body-missing");
    assert_eq!(outcome.condition, OutcomeCondition::InputAbsent);
    assert_eq!(outcome.test_strategy(), TestStrategy::SendNoInput);
    assert_eq!(TestStrategy::SendNoInput.as_str(), "send_no_input");
    assert!(outcome.refuses);
    assert_eq!(
        outcome.error.as_ref().map(ToString::to_string).as_deref(),
        Some("demo.notes.BodyMissing")
    );
    let raw = RawOutcome::from(outcome.clone());
    assert!(raw.input_absent, "the marker is written back");
    assert_eq!(raw.refuses, None, "a refusing marker writes no `refuses:`");
    assert_eq!(raw.when, None);
}

#[test]
fn the_marker_changes_nothing_about_the_input_it_is_beside() {
    let spec = admitted(&notes(16, TEXT, ABSENT));
    let command = spec
        .commands()
        .get(&"demo.notes.SubmitNote".parse().unwrap())
        .unwrap();
    assert_eq!(command.input.len(), 1);
    assert!(
        !matches!(command.input[0].type_ref, ess_domain::TypeRef::Optional(_)),
        "the required field stays required"
    );
}

#[test]
fn input_absent_is_refused_below_ess_16() {
    let errors = refused(&notes(15, TEXT, ABSENT));
    assert_code(
        &errors,
        ValidationCode::UnsupportedFormatVersion,
        "input_absent",
    );
    assert_code(&errors, ValidationCode::UnsupportedFormatVersion, "ess/16");
}

#[test]
fn input_absent_names_its_error_and_is_declared_once() {
    let errors = refused(&notes(
        16,
        TEXT,
        "      - {name: body-missing, input_absent: true}\n",
    ));
    assert_code(&errors, ValidationCode::MissingDeclaration, "body-missing");

    let errors = refused(&notes(
        16,
        TEXT,
        "      - {name: body-missing, input_absent: true, error: demo.notes.BodyMissing}
      - {name: no-body, input_absent: true, error: demo.notes.BodyMissing}
",
    ));
    assert_code(
        &errors,
        ValidationCode::ConflictingDeclaration,
        "input_absent",
    );

    let errors = refused(&notes(
        16,
        TEXT,
        "      - {name: body-missing, input_absent: true, refuses: false}\n",
    ));
    assert_code(&errors, ValidationCode::ConflictingDeclaration, "refuses");
}

#[test]
fn input_absent_is_one_condition_and_changes_nothing() {
    for marker in [
        "{name: body-missing, input_absent: true, when: text == \"x\", error: demo.notes.BodyMissing}",
        "{name: body-missing, input_absent: true, wrong_state: true, error: demo.notes.BodyMissing}",
        "{name: body-missing, input_absent: true, unknown_instance: true, error: demo.notes.BodyMissing}",
        "{name: body-missing, input_absent: true, external: the gateway lost it, error: demo.notes.BodyMissing}",
    ] {
        let errors = refused(&notes(16, TEXT, &format!("      - {marker}\n")));
        assert_code(&errors, ValidationCode::ConflictingDeclaration, "input_absent");
    }
    let errors = refused(&notes(
        16,
        TEXT,
        "      - {name: body-missing, input_absent: true, error: demo.notes.BodyMissing, emits: [demo.notes.NoteSubmitted]}\n",
    ));
    assert_code(
        &errors,
        ValidationCode::ConflictingDeclaration,
        "input_absent",
    );
}

#[test]
fn input_absent_needs_an_input_to_be_absent() {
    let errors = refused(
        &notes(16, "      []\n", ABSENT).replace("    input:\n      []\n", "    input: []\n"),
    );
    assert_code(&errors, ValidationCode::UnreachableBranch, "body-missing");
}

// ---- the validate/synthesize disagreement of #170 ----------------------------------------------

#[test]
fn the_issue_repro_is_refused_at_validate_naming_input_absent() {
    for guard in [
        "'not defined(text)'",
        "'missing(text)'",
        "{all: ['text == \"x\"', 'missing(text)']}",
        "{not: {any: ['defined(text)', 'text == \"x\"']}}",
    ] {
        let errors = refused(&notes(
            16,
            TEXT,
            &format!(
                "      - {{name: body-missing, when: {guard}, error: demo.notes.BodyMissing}}\n"
            ),
        ));
        assert_code(&errors, ValidationCode::TypeMismatch, "text");
        assert!(
            errors.as_slice().iter().any(|error| error
                .hint
                .as_deref()
                .is_some_and(|hint| hint.contains("input_absent"))),
            "the hint names `input_absent` for `{guard}`:\n{errors:#?}"
        );
    }
}

#[test]
fn a_guard_that_can_hold_is_admitted_with_its_dead_disjunct() {
    for guard in [
        "{any: ['text == \"x\"', 'missing(text)']}",
        "{not: {all: ['defined(text)', 'text == \"x\"']}}",
    ] {
        admitted(&notes(
            16,
            TEXT,
            &format!(
                "      - {{name: body-missing, when: {guard}, error: demo.notes.BodyMissing}}\n"
            ),
        ));
    }
}

#[test]
fn below_ess_16_the_guard_keeps_the_meaning_it_had() {
    for format in [14, 15] {
        admitted(&notes(
            format,
            TEXT,
            "      - {name: body-missing, when: not defined(text), error: demo.notes.BodyMissing}\n",
        ));
    }
}

#[test]
fn absence_of_an_optional_input_stays_admitted() {
    admitted(&notes(
        16,
        "      - {name: text, type: String}\n      - {name: note, type: Optional<String>}\n",
        "      - {name: body-missing, when: not defined(note), error: demo.notes.BodyMissing}\n",
    ));
    admitted(&notes(
        16,
        "      - {name: text, type: String}\n      - {name: note, type: Optional<String>}\n",
        "      - {name: body-missing, when: missing(note), error: demo.notes.BodyMissing}\n",
    ));
}
