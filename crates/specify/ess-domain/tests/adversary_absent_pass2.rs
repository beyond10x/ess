//! Adversary pass 2 against the ess/16 refusal of a guard that needs a required input to be
//! absent (beyond10x/ess#170): the De Morgan correction, the `exists` spelling, and a required
//! field read through a required struct input.

use ess_domain::Specification;
use ess_primitives::error::{ValidationCode, ValidationErrors};

fn assemble(body: &str) -> Result<Specification, ValidationErrors> {
    let raw = ess_domain::spec::RawSpecFile::parse(body)
        .unwrap_or_else(|error| panic!("the document parses: {error}\n{body}"));
    Specification::assemble([(ess_domain::system::Source::new("notes.yaml"), raw)])
}

/// `demo.notes.SubmitNote` with `input` as its input list and `guard` on a refusing branch.
fn notes(format: u32, input: &str, guard: &str) -> String {
    format!(
        "format: ess/{format}
system: demo
version: v1
domain: demo.notes
types:
  - {{name: demo.notes.NoteId, kind: newtype, of: Uuid}}
  - name: demo.notes.Body
    kind: struct
    fields:
      - {{name: text, type: String}}
      - {{name: note, type: Optional<String>}}
entities:
  - name: demo.notes.Note
    identity: {{name: note_id, type: demo.notes.NoteId}}
    fields:
      - {{name: text, type: String}}
    lifecycle: {{initial: Open, states: [Open], terminal: [Open], transitions: []}}
actors:
  - {{name: demo.notes.Service, may: [demo.notes.SubmitNote]}}
errors:
  - name: demo.notes.Refused
    summary: Refused.
    fields: []
commands:
  - name: demo.notes.SubmitNote
    input:
{input}    outcomes:
      - name: refused
        when: {guard}
        error: demo.notes.Refused
      - name: submitted
        creates: demo.notes.Note
        instance: note_id
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
const BODY: &str = "      - {name: body, type: demo.notes.Body}\n";

fn never_holds(result: &Result<Specification, ValidationErrors>) -> bool {
    match result {
        Ok(_) => false,
        Err(errors) => errors.as_slice().iter().any(|error| {
            error.code == ValidationCode::TypeMismatch
                && error.to_string().contains("never holds")
                && error
                    .hint
                    .as_deref()
                    .is_some_and(|hint| hint.contains("input_absent"))
        }),
    }
}

/// Satisfiable guards the corrected De Morgan walk must admit.
#[test]
fn satisfiable_guards_over_a_required_input_are_admitted() {
    for guard in [
        // not(not defined) = defined: always holds.
        "{not: 'missing(text)'}",
        // defined and text != x.
        "{not: {any: ['missing(text)', 'text == \"x\"']}}",
        // (missing or x) and x: holds when text == x.
        "{all: [{any: ['missing(text)', 'text == \"x\"']}, 'text == \"x\"']}",
        // not(missing and x) = defined or not x: always holds.
        "{not: {all: ['missing(text)', 'text == \"x\"']}}",
    ] {
        let body = notes(16, TEXT, guard);
        assert!(
            assemble(&body).is_ok(),
            "`{guard}` can hold and is refused: {:?}",
            assemble(&body).err().map(|e| e.to_string())
        );
    }
}

/// Unsatisfiable-for-want-of-a-required-input guards, in the spellings the parser accepts.
#[test]
fn unsatisfiable_guards_in_every_spelling_are_refused() {
    for guard in [
        "'not exists(text)'",
        "{not: {all: ['defined(text)', 'exists(text)']}}",
        "{any: ['missing(text)', {not: 'defined(text)'}]}",
        "{all: [{any: ['missing(text)', 'missing(text)']}, 'text == \"x\"']}",
        "{not: {any: ['defined(text)', 'text == \"x\"']}}",
    ] {
        let result = assemble(&notes(16, TEXT, guard));
        assert!(never_holds(&result), "`{guard}` is not refused: {result:?}");
    }
}

/// The optional-field control: `missing(body.note)` reads an `Optional` field and stays admitted.
/// `missing(body.text)` reads a required field of a required struct input: it can no more hold
/// than `missing(text)` does over a required `text`, and synthesis can no more witness it, yet
/// the refusal only reads one-segment paths.
#[test]
fn a_required_field_of_a_required_struct_input_is_refused_like_a_top_level_one() {
    assemble(&notes(16, BODY, "'missing(body.note)'"))
        .unwrap_or_else(|errors| panic!("precondition: an Optional nested field: {errors}"));
    assemble(&notes(15, BODY, "'missing(body.text)'")).unwrap_or_else(|errors| {
        panic!("precondition: the guard is well-formed at ess/15: {errors}")
    });
    let result = assemble(&notes(16, BODY, "'missing(body.text)'"));
    assert!(
        never_holds(&result),
        "`missing(body.text)` over a required `body: demo.notes.Body` whose `text` is required is \
         admitted at ess/16: {:?}",
        result.as_ref().err().map(ToString::to_string)
    );
}
