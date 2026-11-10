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

const DESK: &str = "format: ess/34
system: example
version: v1
summary: Desks an owner relabels and locks.
domain: example.desk
types:
  - {name: example.desk.DeskId, kind: newtype, of: Uuid}
  - {name: example.desk.OwnerId, kind: newtype, of: Uuid}
actors:
  - name: example.desk.Author
    attributes:
      - {name: owner_id, type: example.desk.OwnerId}
    may: [example.desk.Open, example.desk.Veto, example.desk.Lock, example.desk.Stamp, example.desk.Claim, example.desk.Audit]
entities:
  - name: example.desk.Desk
    identity: {name: desk_id, type: example.desk.DeskId}
    fields:
      - {name: owner_id, type: Optional<example.desk.OwnerId>}
      - {name: label, type: String}
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
errors:
  - {name: example.desk.NoSuchDesk, fields: []}
  - {name: example.desk.Vetoed, fields: []}
  - name: example.desk.Locked
    fields:
      - {name: by, type: example.desk.OwnerId}
commands:
  - name: example.desk.Open
    input:
      - {name: desk_id, type: example.desk.DeskId}
      - {name: label, type: String}
    outcomes:
      - name: opened
        creates: example.desk.Desk
        instance: desk_id
        sets:
          label: input.label
          owner_id: {caller: owner_id}
        emits: [example.desk.Opened]
        payload:
          example.desk.Opened: {desk_id: input.desk_id}
  - name: example.desk.Veto
    input:
      - {name: desk_id, type: example.desk.DeskId}
      - {name: label, type: String}
    outcomes:
      - {name: no-such-desk, unknown_instance: true, error: example.desk.NoSuchDesk}
      - name: vetoed
        external: someone outside the model vetoes a relabel by the owner
        when_subject: {predicate: owner_id == caller.owner_id}
        error: example.desk.Vetoed
      - name: relabelled
        updates: example.desk.Desk
        instance: desk_id
        sets: {label: input.label}
        emits: [example.desk.Relabelled]
        payload:
          example.desk.Relabelled: {desk_id: input.desk_id}
  - name: example.desk.Lock
    input:
      - {name: desk_id, type: example.desk.DeskId}
    outcomes:
      - name: locked
        error: example.desk.Locked
        payload:
          example.desk.Locked: {by: {caller: owner_id}}
  - name: example.desk.Stamp
    input:
      - {name: desk_id, type: example.desk.DeskId}
    outcomes:
      - name: stamped
        emits: [example.desk.Stamped]
        payload:
          example.desk.Stamped: {by: {caller: owner_id}}
  - name: example.desk.Claim
    input:
      - {name: desk_id, type: example.desk.DeskId}
      - {name: owner_id, type: example.desk.OwnerId}
    outcomes:
      - name: not-yours
        when: caller.owner_id == input.owner_id
        when_related: {via: input.desk_id, predicate: label == \"closed\"}
        error: example.desk.Vetoed
      - name: no-desk
        when_related: {via: input.desk_id, exists: false}
        error: example.desk.NoSuchDesk
      - name: claimed
        error: example.desk.Locked
        payload:
          example.desk.Locked: {by: input.owner_id}
  - name: example.desk.Audit
    input:
      - {name: desk_id, type: example.desk.DeskId}
      - {name: owner_id, type: example.desk.OwnerId}
    outcomes:
      - name: owns-none
        when: caller.owner_id == input.owner_id
        when_related: {entity: example.desk.Desk, where: label == \"closed\", exists: true}
        error: example.desk.NoSuchDesk
      - name: audited
        emits: [example.desk.Stamped]
        payload:
          example.desk.Stamped: {by: input.owner_id}
events:
  - name: example.desk.Opened
    fields:
      - {name: desk_id, type: example.desk.DeskId}
  - name: example.desk.Relabelled
    fields:
      - {name: desk_id, type: example.desk.DeskId}
  - name: example.desk.Stamped
    fields:
      - {name: by, type: example.desk.OwnerId}
components:
  - component: example
    summary: Holds desks.
    owns:
      domains: [example.desk]
    accepts:
      commands: [example.desk.Open, example.desk.Veto, example.desk.Lock, example.desk.Stamp, example.desk.Claim, example.desk.Audit]
    publishes:
      events: [example.desk.Opened, example.desk.Relabelled, example.desk.Stamped]
    reached_by: in_process
";

const DESK_COMMANDS: [&str; 6] = ["Veto", "Lock", "Claim", "Audit", "Open", "Stamp"];

fn desk() -> EssIr {
    let raw = RawSpecFile::parse(DESK).unwrap();
    let spec = Specification::assemble([(Source::new("desk.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap()
}

#[test]
fn every_caller_read_position_is_listed() {
    let ir = desk();
    for name in DESK_COMMANDS {
        let command = &ir.commands()[&format!("example.desk.{name}").parse().unwrap()];
        assert_eq!(
            ess_gen::http::caller_attributes(&ir, command),
            ["owner_id"],
            "{name}"
        );
    }
}

#[test]
fn the_openapi_operation_lists_a_caller_read_in_every_position() {
    let openapi = artifact(&desk(), "openapi");
    for name in DESK_COMMANDS {
        let start = openapi
            .find(&format!("/desk/commands/{name}:"))
            .unwrap_or_else(|| panic!("no operation for {name}: {openapi}"));
        let operation = &openapi[start..];
        let end = operation[1..]
            .find("\n  /")
            .map_or(operation.len(), |at| at + 1);
        assert!(
            operation[..end].contains("x-ess-caller:\n      - owner_id"),
            "{name}: {}",
            &operation[..end]
        );
    }
}
