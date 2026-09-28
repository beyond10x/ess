//! Adversary cases for the `not defined(f)` / `missing(f)` refusal of beyond10x/ess#170.
//!
//! The refusal claims "the guard never holds and no request can take the branch". It collects every
//! `defined(f)` under an odd number of negations and refuses it, without asking whether the
//! negation sits over a conjunction. `not (defined(text) and text == "x")` over a required `text`
//! reads `text != "x"`: it holds for most requests, so the branch is reachable and the refusal's
//! own claim is false.

use ess_domain::Specification;
use ess_primitives::error::{ValidationCode, ValidationErrors};

fn assemble(body: &str) -> Result<Specification, ValidationErrors> {
    let raw = ess_domain::spec::RawSpecFile::parse(body)
        .unwrap_or_else(|error| panic!("the document parses: {error}\n{body}"));
    Specification::assemble([(ess_domain::system::Source::new("notes.yaml"), raw)])
}

fn notes(format: u32, outcomes: &str) -> String {
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
  - name: demo.notes.NotX
    summary: The text is not x.
    fields: []
commands:
  - name: demo.notes.SubmitNote
    input:
      - {{name: text, type: String}}
    outcomes:
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

fn never_holds(errors: &ValidationErrors) -> Vec<String> {
    errors
        .as_slice()
        .iter()
        .filter(|error| {
            error.code == ValidationCode::TypeMismatch && error.to_string().contains("never holds")
        })
        .map(ToString::to_string)
        .collect()
}

/// `not (defined(text) and text == "x")` is `text != "x"` for a required `text`: satisfiable.
#[test]
fn a_negated_conjunction_containing_defined_is_not_refused_as_never_holding() {
    let body = notes(
        16,
        "      - name: not-x
        when: {not: {all: ['defined(text)', 'text == \"x\"']}}
        error: demo.notes.NotX
",
    );
    if let Err(errors) = assemble(&body) {
        let wrong = never_holds(&errors);
        assert!(
            wrong.is_empty(),
            "a guard that holds for every text but \"x\" is refused as never holding:\n{}",
            wrong.join("\n")
        );
    }
}

/// `text == "x" or missing(text)` holds whenever `text == "x"`; the claim "never holds" is false
/// for the guard as a whole.
#[test]
fn a_disjunction_whose_other_arm_holds_is_not_refused_as_never_holding() {
    let body = notes(
        16,
        "      - name: not-x
        when: {any: ['text == \"x\"', 'missing(text)']}
        error: demo.notes.NotX
",
    );
    if let Err(errors) = assemble(&body) {
        let wrong = never_holds(&errors);
        assert!(
            wrong.is_empty(),
            "a guard that holds for text == \"x\" is refused as never holding:\n{}",
            wrong.join("\n")
        );
    }
}
