//! Adversary, pass 2, `story:caller-value-source-and-guard` (beyond10x/ess#168).
//!
//! A mutant of `caller_value::decides` that dropped "the earlier branch accepts" would answer 403
//! for an `otherwise` refusal that follows a caller-decided *refusal* and an input-guarded
//! acceptance: the caller did not decide it, the empty text did.
use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

const NOTES: &str = "format: ess/16
system: demo
version: v1
summary: Minimal repro.
domain: demo.notes
types:
  - {name: demo.notes.NoteId, kind: newtype, of: Uuid}
  - {name: demo.notes.AccountId, kind: newtype, of: Uuid}
  - {name: demo.notes.AgentId, kind: newtype, of: Uuid}
entities:
  - name: demo.notes.Note
    identity: {name: note_id, type: demo.notes.NoteId}
    fields:
      - {name: account_id, type: demo.notes.AccountId}
      - {name: agent_id, type: demo.notes.AgentId}
      - {name: text, type: String}
    lifecycle: {initial: Open, states: [Open], terminal: [Open], transitions: []}
actors:
  - name: demo.notes.AccountUser
    attributes:
      - {name: account_id, type: demo.notes.AccountId}
      - {name: agent_id, type: demo.notes.AgentId}
    may: [demo.notes.CreateNote, demo.notes.EditNote]
commands:
  - name: demo.notes.CreateNote
    input:
      - {name: text, type: String}
    outcomes:
      - name: created
        creates: demo.notes.Note
        instance: note_id
        sets: {account_id: {caller: account_id}, agent_id: {caller: agent_id}, text: input.text}
        emits: [demo.notes.NoteCreated]
        payload:
          demo.notes.NoteCreated: {note_id: {generated: true}, account_id: {caller: account_id}, text: input.text}
  - name: demo.notes.EditNote
    input:
      - {name: note_id, type: demo.notes.NoteId}
      - {name: text, type: String}
    outcomes:
      - name: forbidden
        when_subject: {predicate: agent_id != caller.agent_id}
        error: demo.notes.NotYourNote
      - name: edited
        updates: demo.notes.Note
        instance: note_id
        sets: {text: input.text}
        emits: [demo.notes.NoteEdited]
        payload:
          demo.notes.NoteEdited: {note_id: input.note_id}
errors:
  - name: demo.notes.NotYourNote
    summary: The caller is not the note's agent.
events:
  - name: demo.notes.NoteCreated
    fields:
      - {name: note_id, type: demo.notes.NoteId}
      - {name: account_id, type: demo.notes.AccountId}
      - {name: text, type: String}
  - name: demo.notes.NoteEdited
    fields:
      - {name: note_id, type: demo.notes.NoteId}
views:
  - name: demo.notes.NoteDetails
    source: demo.notes.Note
    consistency: read_your_writes
    fields:
      - {name: note_id, type: demo.notes.NoteId}
      - {name: account_id, type: demo.notes.AccountId}
components:
  - component: notes-service
    owns:
      domains: [demo.notes]
    accepts:
      commands: [demo.notes.CreateNote, demo.notes.EditNote]
    publishes:
      events: [demo.notes.NoteCreated, demo.notes.NoteEdited]
";

/// [`NOTES`] with an accepting branch guarded on the input alone, and an `otherwise` refusal for
/// an empty text after it. The caller decides `forbidden`; `blank` is decided by the text.
fn blank_after_caller_refusal() -> String {
    NOTES
        .replace(
            "      - name: edited\n        updates: demo.notes.Note\n",
            "      - name: edited\n        when: text != \"\"\n        updates: demo.notes.Note\n",
        )
        .replace(
            "          demo.notes.NoteEdited: {note_id: input.note_id}\n",
            "          demo.notes.NoteEdited: {note_id: input.note_id}\n      - name: blank\n        error: demo.notes.EmptyText\n",
        )
        .replace(
            "errors:\n",
            "errors:\n  - name: demo.notes.EmptyText\n    summary: The text is empty.\n",
        )
}

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap();
    let spec = Specification::assemble([(Source::new("notes.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap()
}

#[test]
fn adversary_caller_pass2_an_otherwise_refusal_after_a_caller_refusal_is_not_forbidden() {
    let ir = ir(&blank_after_caller_refusal());
    let edit = &ir.commands()[&"demo.notes.EditNote".parse().unwrap()];
    let names: Vec<&str> = edit.outcomes.iter().map(|o| o.name.as_str()).collect();
    assert_eq!(names, ["forbidden", "edited", "blank"]);
    assert_eq!(ess_gen::http::status(&edit.outcomes[0]), "403");
    assert!(!edit.outcomes[2].decided_by_caller);
    assert_ne!(
        ess_gen::http::status(&edit.outcomes[2]),
        "403",
        "the agent itself meets `blank` for an empty text; the caller does not decide it"
    );
}
