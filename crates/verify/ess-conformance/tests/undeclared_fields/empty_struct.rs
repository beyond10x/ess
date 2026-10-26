//! A command input whose type is a struct with `fields: []` (`ess/32`, beyond10x/ess#649), open
//! or closed: synthesis sends a value of it and the interpreted target passes its own suite.

use super::{ir, refusals, Runner, Status};
use ess_conformance::synthesize::synthesize;
use ess_conformance::AdmittedSuite;

/// `{ack}` is the declaration of `notes.ack.Ack`, the type of the input `extra`.
const MODEL: &str = "\
format: ess/32
system: notes
version: v1
domain: notes.ack
summary: An acknowledgment is an object with no declared members.
types:
  - {name: notes.ack.NoteId, kind: newtype, of: Uuid}
  - name: notes.ack.Ack
    kind: struct
{ack}    fields: []
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

const CREATED: &str = "notes.ack.CreateNote/outcome/created";

#[test]
fn an_empty_struct_input_synthesizes_and_passes_on_the_interpreted_target() {
    for ack in ["    undeclared_fields: ignored\n", ""] {
        let text = MODEL.replace("{ack}", ack);
        let result = synthesize(&ir(&text));
        let suite = result.suite;
        assert!(
            suite.scenarios.keys().any(|id| id.to_string() == CREATED),
            "{ack:?}: the outcome is synthesized: {:#?}",
            refusals(&synthesize(&ir(&text)))
        );
        let admitted = AdmittedSuite::from_suite(&suite).unwrap();
        let run = Runner::for_suite(&suite).run_admitted(
            &admitted,
            &ess_conformance::interpret::Interpreted::for_model(ir(&text)),
        );
        assert!(!run.scenarios.is_empty(), "{ack:?}: scenarios ran");
        for scenario in &run.scenarios {
            assert_eq!(scenario.status, Status::Passed, "{ack:?}: {scenario:#?}");
        }
    }
}
