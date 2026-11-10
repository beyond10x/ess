//! Adversary cases for `ess_gen::http::caller_attributes` (beyond10x/ess#740): caller reads the
//! validation admits and the interpreter reads, held against the `x-ess-caller` list.
use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

fn compiled(source: &str) -> EssIr {
    let raw = RawSpecFile::parse(source).unwrap();
    let spec = Specification::assemble([(Source::new("model.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap()
}

fn attributes(ir: &EssIr, command: &str) -> Vec<String> {
    let command = &ir.commands()[&command.parse().unwrap()];
    ess_gen::http::caller_attributes(ir, command)
}

/// A struct-typed actor attribute read through a member: `caller.org.id`. The domain counts it a
/// caller read (`is_caller_path`: more than one segment), so the branch answers `403`, and the
/// interpreter reads it (the caller source takes the tail path). The operation must then name the
/// attribute it reads.
const ORG: &str = "format: ess/34
system: example
version: v1
summary: Members pass a gate of their own organisation.
domain: example.gate
types:
  - {name: example.gate.OrgId, kind: newtype, of: Uuid}
  - name: example.gate.Org
    kind: struct
    fields:
      - {name: id, type: example.gate.OrgId}
actors:
  - name: example.gate.Member
    attributes:
      - {name: org, type: example.gate.Org}
    may: [example.gate.Pass]
errors:
  - {name: example.gate.OtherOrg, fields: []}
commands:
  - name: example.gate.Pass
    input:
      - {name: org_id, type: example.gate.OrgId}
    outcomes:
      - name: other-org
        when: caller.org.id != input.org_id
        error: example.gate.OtherOrg
      - name: passed
        emits: [example.gate.Passed]
        payload:
          example.gate.Passed: {org_id: input.org_id}
events:
  - name: example.gate.Passed
    fields:
      - {name: org_id, type: example.gate.OrgId}
components:
  - component: example
    summary: Holds the gate.
    owns:
      domains: [example.gate]
    accepts:
      commands: [example.gate.Pass]
    publishes:
      events: [example.gate.Passed]
    reached_by: in_process
";

#[test]
fn a_caller_read_through_a_struct_attribute_is_listed() {
    let ir = compiled(ORG);
    let pass = &ir.commands()[&"example.gate.Pass".parse().unwrap()];
    assert_eq!(
        ess_gen::http::status(&pass.outcomes[0]),
        "403",
        "the branch is decided by the caller"
    );
    assert_eq!(attributes(&ir, "example.gate.Pass"), ["org"]);
}

/// An input and a stored field named `caller` keep being read as themselves; the actor's own
/// attribute `key` is read by neither guard. Pins the `fields` skip in `caller_attributes`: without
/// it both commands list `key`.
const SHADOW: &str = r"format: ess/19
system: demo
version: v1
domain: demo.notes
types:
  - {name: demo.notes.Id, kind: newtype, of: Uuid}
  - name: demo.notes.Bundle
    kind: struct
    fields: [{name: key, type: String}]
entities:
  - name: demo.notes.Record
    identity: {name: id, type: demo.notes.Id}
    fields: [{name: caller, type: demo.notes.Bundle}]
    lifecycle: {initial: Stored, states: [Stored], terminal: [Stored], transitions: []}
actors:
  - name: demo.notes.User
    attributes: [{name: key, type: String}]
    may: [demo.notes.Create, demo.notes.Edit]
errors:
  - {name: demo.notes.Denied, summary: Wrong key.}
commands:
  - name: demo.notes.Create
    input: [{name: caller, type: demo.notes.Bundle}, {name: key, type: String}]
    outcomes:
      - {name: denied, when: 'caller.key != stored-value', error: demo.notes.Denied}
      - name: created
        creates: demo.notes.Record
        instance: id
        sets: {caller: input.caller}
        emits: [demo.notes.Created]
        payload: {demo.notes.Created: {id: {generated: true}}}
  - name: demo.notes.Edit
    input: [{name: id, type: demo.notes.Id}, {name: key, type: String}]
    outcomes:
      - name: denied
        when_subject: {predicate: 'caller.key != input.key'}
        error: demo.notes.Denied
      - name: edited
        updates: demo.notes.Record
        instance: id
        emits: [demo.notes.Edited]
        payload: {demo.notes.Edited: {id: input.id}}
events:
  - name: demo.notes.Created
    fields: [{name: id, type: demo.notes.Id}]
  - name: demo.notes.Edited
    fields: [{name: id, type: demo.notes.Id}]
views:
  - name: demo.notes.Records
    source: demo.notes.Record
    consistency: read_your_writes
    fields: [{name: id, type: demo.notes.Id}, {name: caller, type: demo.notes.Bundle}]
";

#[test]
fn a_field_named_caller_is_no_caller_read() {
    let ir = compiled(SHADOW);
    assert_eq!(attributes(&ir, "demo.notes.Create"), Vec::<String>::new());
    assert_eq!(attributes(&ir, "demo.notes.Edit"), Vec::<String>::new());
}

/// The input guard (`when:`) of an `external:` branch narrowed by `when_subject:`, whose
/// `when_subject:` predicate reads no caller: acceptance position 1, the input-guard half, which
/// the unit's own reproduction does not exercise (its `Veto` reads the caller in the predicate).
const EXTERNAL_INPUT_GUARD: &str = "format: ess/34
system: example
version: v1
summary: Desks an owner relabels.
domain: example.desk
types:
  - {name: example.desk.DeskId, kind: newtype, of: Uuid}
  - {name: example.desk.OwnerId, kind: newtype, of: Uuid}
actors:
  - name: example.desk.Author
    attributes:
      - {name: owner_id, type: example.desk.OwnerId}
    may: [example.desk.Veto]
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
commands:
  - name: example.desk.Veto
    input:
      - {name: desk_id, type: example.desk.DeskId}
      - {name: owner_id, type: example.desk.OwnerId}
      - {name: label, type: String}
    outcomes:
      - {name: no-such-desk, unknown_instance: true, error: example.desk.NoSuchDesk}
      - name: vetoed
        external: someone outside the model vetoes a relabel by the owner
        when: caller.owner_id == input.owner_id
        when_subject: {predicate: label == \"closed\"}
        error: example.desk.Vetoed
      - name: relabelled
        updates: example.desk.Desk
        instance: desk_id
        sets: {label: input.label}
        emits: [example.desk.Relabelled]
        payload:
          example.desk.Relabelled: {desk_id: input.desk_id}
events:
  - name: example.desk.Relabelled
    fields:
      - {name: desk_id, type: example.desk.DeskId}
components:
  - component: example
    summary: Holds desks.
    owns:
      domains: [example.desk]
    accepts:
      commands: [example.desk.Veto]
    publishes:
      events: [example.desk.Relabelled]
    reached_by: in_process
";

#[test]
fn the_input_guard_of_an_external_subject_branch_is_listed() {
    let ir = compiled(EXTERNAL_INPUT_GUARD);
    assert_eq!(attributes(&ir, "example.desk.Veto"), ["owner_id"]);
}
