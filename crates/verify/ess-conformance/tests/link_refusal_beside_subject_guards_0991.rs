//! Hotfix 0.99.11: a command whose `when_subject:` refusal compares the row's stored link with an
//! input (`group_id != input.group_id`) beside a `when_related:` guard reading the row that input
//! names, and a second `when_subject:` refusal over another stored field (`kind != Worker`). Since
//! 0.99.7 the related row is arranged through the subject's link; every scenario synthesized for
//! such a command must pass on the interpreter, the reference target, so the addressed row is also
//! arranged where the other refusal is not selected.

use std::collections::BTreeMap;

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::synthesize::synthesize;
use ess_conformance::{AdmittedSuite, AdvancingClock, ConformanceSuite, Ids, Runner, RunnerConfig};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("model.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn run_on(suite: &ConformanceSuite, target: EssIr) -> BTreeMap<String, Status> {
    let admitted = AdmittedSuite::from_suite(suite).unwrap_or_else(|error| panic!("{error}"));
    Runner::new(
        RunnerConfig::default(),
        AdvancingClock::default(),
        Ids::for_suite(suite),
    )
    .run_admitted(&admitted, &Interpreted::for_model(target))
    .into_report()
    .scenarios
    .into_iter()
    .map(|scenario| (scenario.scenario.to_string(), scenario.status))
    .collect()
}

/// Synthesizes `text`, runs the suite on the interpreter, and fails naming every scenario that did
/// not pass, with its steps; every id in `expected` must be synthesized.
fn every_scenario_passes(text: &str, expected: &[&str]) {
    let model = ir(text);
    let synthesis = synthesize(&model);
    let suite = synthesis.suite;
    let statuses = run_on(&suite, model);
    eprintln!("statuses: {statuses:#?}");
    eprintln!("refusals: {:#?}", synthesis.refusals);
    let failing: Vec<_> = statuses
        .iter()
        .filter(|(_, status)| **status != Status::Passed)
        .collect();
    let steps: Vec<_> = suite
        .scenarios
        .iter()
        .filter(|(id, _)| failing.iter().any(|(name, _)| **name == id.to_string()))
        .map(|(id, scenario)| format!("{id}: {:#?}", scenario.steps))
        .collect();
    assert!(
        failing.is_empty(),
        "failing: {failing:#?}\n{}",
        steps.join("\n")
    );
    for id in expected {
        assert!(
            statuses.contains_key(*id),
            "{id} synthesized: {statuses:#?}"
        );
    }
}

const MODEL: &str = "format: ess/37
system: demo
version: v1
domain: demo.crew
types:
  - {name: demo.crew.GroupId, kind: newtype, of: Uuid}
  - {name: demo.crew.MemberId, kind: newtype, of: Uuid}
  - {name: demo.crew.Kind, kind: enum, variants: [Lead, Worker]}
errors:
  - {name: demo.crew.MemberNotFound}
  - {name: demo.crew.ForeignMember}
  - {name: demo.crew.NotAWorker}
  - {name: demo.crew.GroupNotFound}
events:
  - name: demo.crew.GroupCreated
    fields:
      - {name: group_id, type: demo.crew.GroupId}
  - name: demo.crew.MemberAdded
    fields:
      - {name: member_id, type: demo.crew.MemberId}
  - name: demo.crew.NoteSet
    fields:
      - {name: member_id, type: demo.crew.MemberId}
views:
  - name: demo.crew.Members
    source: demo.crew.Member
    consistency: read_your_writes
    fields:
      - {name: member_id, type: demo.crew.MemberId}
      - {name: group_id, type: demo.crew.GroupId}
      - {name: kind, type: demo.crew.Kind}
      - {name: note, type: String}
entities:
  - name: demo.crew.Group
    identity: {name: group_id, type: demo.crew.GroupId}
    fields:
      - {name: name, type: String}
    lifecycle: {initial: Active, states: [Active], terminal: [Active], transitions: []}
  - name: demo.crew.Member
    identity: {name: member_id, type: demo.crew.MemberId}
    fields:
      - {name: group_id, type: demo.crew.GroupId}
      - {name: kind, type: demo.crew.Kind}
      - {name: note, type: String}
    relations:
      - {name: group, kind: references, target: demo.crew.Group, cardinality: one, via: group_id}
    lifecycle: {initial: Live, states: [Live], terminal: [Live], transitions: []}
commands:
  - name: demo.crew.CreateGroup
    input:
      - {name: name, type: String}
    outcomes:
      - name: created
        creates: demo.crew.Group
        instance: group_id
        sets:
          name: input.name
        emits: [demo.crew.GroupCreated]
        payload:
          demo.crew.GroupCreated:
            group_id: {generated: true}
  - name: demo.crew.AddMember
    input:
      - {name: group_id, type: demo.crew.GroupId}
      - {name: kind, type: demo.crew.Kind}
      - {name: note, type: String}
    outcomes:
      - name: no-such-group
        when_related: {via: input.group_id, exists: false}
        error: demo.crew.GroupNotFound
      - name: added
        creates: demo.crew.Member
        instance: member_id
        sets:
          group_id: input.group_id
          kind: input.kind
          note: input.note
        emits: [demo.crew.MemberAdded]
        payload:
          demo.crew.MemberAdded:
            member_id: {generated: true}
  - name: demo.crew.SetNote
    input:
      - {name: group_id, type: demo.crew.GroupId}
      - {name: member_id, type: demo.crew.MemberId}
      - {name: note, type: String}
    outcomes:
      - name: no-such-member
        unknown_instance: true
        error: demo.crew.MemberNotFound
      - name: foreign-member
        when_subject: {predicate: group_id != input.group_id}
        error: demo.crew.ForeignMember
      - name: not-a-worker
        when_subject: {predicate: kind != Worker}
        error: demo.crew.NotAWorker
      - name: no-such-group
        when_related: {via: input.group_id, exists: false}
        error: demo.crew.GroupNotFound
      - name: set
        updates: demo.crew.Member
        instance: member_id
        sets:
          note: input.note
        emits: [demo.crew.NoteSet]
        payload:
          demo.crew.NoteSet:
            member_id: input.member_id
";

/// The accepting branch is sent for a member of the group the input names, and a worker.
#[test]
fn a_link_refusal_arrangement_keeps_the_other_subject_refusals_refuted() {
    every_scenario_passes(MODEL, &["demo.crew.SetNote/outcome/set"]);
}
