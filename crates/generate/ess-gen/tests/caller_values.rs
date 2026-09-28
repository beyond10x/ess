//! The published contract names the caller (source format `ess/16`, beyond10x/ess#168,
//! `docs/design/caller-values.md`): an actor's page lists what its credential carries, an operation
//! names the caller attributes it reads, and a refusal the caller decides answers `403`.
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

fn ir() -> EssIr {
    let raw = RawSpecFile::parse(NOTES).unwrap();
    let spec = Specification::assemble([(Source::new("notes.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap()
}

fn artifact(ir: &EssIr, part: &str) -> String {
    ess_gen::generate_all(ir)
        .unwrap()
        .into_iter()
        .filter(|(path, _)| path.contains(part))
        .map(|(path, artifact)| format!("== {path}\n{}", artifact.contents))
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn a_refusal_the_caller_decides_answers_forbidden() {
    let ir = ir();
    let edit = &ir.commands()[&"demo.notes.EditNote".parse().unwrap()];
    assert_eq!(ess_gen::http::status(&edit.outcomes[0]), "403");
    assert_eq!(
        ess_gen::http::status(&edit.outcomes[1]),
        ess_gen::http::TAKEN
    );
    let openapi = artifact(&ir, "openapi");
    assert!(openapi.contains("'403':"), "{openapi}");
}

#[test]
fn the_openapi_operation_names_the_caller_attributes_it_reads() {
    let openapi = artifact(&ir(), "openapi");
    assert!(openapi.contains("x-ess-caller:"), "{openapi}");
    assert!(openapi.contains("- account_id"), "{openapi}");
    assert!(openapi.contains("- agent_id"), "{openapi}");
}

#[test]
fn the_actor_page_lists_what_the_credential_carries() {
    let docs = artifact(&ir(), ".md");
    assert!(
        docs.contains("Its credential carries:"),
        "the actor names its attributes: {docs}"
    );
    assert!(docs.contains("`account_id`"), "{docs}");
}
