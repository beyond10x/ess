//! Adversary, pass 2, `story:caller-value-source-and-guard` (beyond10x/ess#168): the refusal text
//! for actors that declare one attribute at different types lists each type once.

const SPEC: &str = "format: ess/16
system: demo
version: v1
summary: Minimal repro.
domains: [demo.notes]
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
      - {name: text, type: String}
    lifecycle: {initial: Open, states: [Open], terminal: [Open], transitions: []}
actors:
  - name: demo.notes.Alpha
    attributes:
      - {name: account_id, type: demo.notes.AccountId}
    may: [demo.notes.CreateNote]
  - name: demo.notes.Beta
    attributes:
      - {name: account_id, type: demo.notes.AgentId}
    may: [demo.notes.CreateNote]
  - name: demo.notes.Gamma
    attributes:
      - {name: account_id, type: demo.notes.AccountId}
    may: [demo.notes.CreateNote]
commands:
  - name: demo.notes.CreateNote
    input:
      - {name: text, type: String}
    outcomes:
      - name: created
        creates: demo.notes.Note
        instance: note_id
        sets: {account_id: {caller: account_id}, text: input.text}
        emits: [demo.notes.NoteCreated]
events:
  - name: demo.notes.NoteCreated
    fields:
      - {name: text, type: String}
views: []
";

#[test]
fn adversary_caller_pass2_attribute_types_that_disagree_are_each_named_once() {
    let raw = ess_domain::spec::RawSpecFile::parse(SPEC).unwrap_or_else(|e| panic!("{e}"));
    let errors =
        ess_domain::Specification::assemble([(ess_domain::system::Source::new("notes.yaml"), raw)])
            .map(|_| ())
            .expect_err("the actors disagree on `account_id`")
            .to_string();
    let line = errors
        .lines()
        .find(|line| line.contains("at different types"))
        .unwrap_or_else(|| panic!("a refusal naming the types:\n{errors}"));
    let listed = line
        .split("at different types:")
        .nth(1)
        .expect("the types follow");
    assert_eq!(
        listed.matches("demo.notes.AccountId").count(),
        1,
        "each disagreeing type is named once: {line}"
    );
}
