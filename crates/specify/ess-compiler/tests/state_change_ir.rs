//! `when_state_changes:` lands in the IR as the held states it admits, not as the word.
//!
//! The author writes one answer and names no state. A consumer arranging a scenario needs the
//! states that answer admits, and this is the check that it does not have to re-derive them: the
//! compiled condition carries the transition's own `from` set already partitioned, in the same
//! place a literal `when_subject_state:` guard carries its one state.

use ess_compiler::ir::ResolvedCondition;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

/// The adopter shape: a push re-reports the state a row already holds.
const MODEL: &str = "format: ess/4
system: demo
version: v1
domain: demo.room
types:
  - name: demo.room.Reported
    kind: enum
    variants: [Pending, Added, Gone]
entities:
  - name: demo.room.Member
    identity: {name: member_id, type: Uuid}
    fields:
      - {name: on_hold, type: Boolean}
    lifecycle:
      initial: Pending
      states: [Pending, Added, Gone]
      terminal: [Gone]
      transitions:
        - {name: join, from: [Pending, Added, Gone], to: Added}
        - {name: leave, from: [Pending, Added, Gone], to: Gone}
events:
  - name: demo.room.MemberUpdated
    fields:
      - {name: member_id, type: Uuid}
commands:
  - name: demo.room.Report
    input:
      - {name: member_id, type: Uuid}
      - {name: state, type: demo.room.Reported}
      - {name: on_hold, type: Boolean}
    outcomes:
      - name: joined
        when: state == Added
        when_state_changes: true
        moves: demo.room.Member.join
        instance: member_id
        emits: [demo.room.MemberUpdated]
        payload:
          demo.room.MemberUpdated:
            member_id: input.member_id
      - name: gone
        when: state == Gone
        when_state_changes: true
        moves: demo.room.Member.leave
        instance: member_id
        emits: [demo.room.MemberUpdated]
        payload:
          demo.room.MemberUpdated:
            member_id: input.member_id
      - name: refreshed
        updates: demo.room.Member
        instance: member_id
        sets:
          on_hold: input.on_hold
        emits: [demo.room.MemberUpdated]
        payload:
          demo.room.MemberUpdated:
            member_id: input.member_id
";

fn ir() -> ess_compiler::ir::EssIr {
    let raw = RawSpecFile::parse(MODEL).expect("the document parses");
    let spec =
        Specification::assemble([(Source::new("state-change.yaml"), raw)]).expect("it validates");
    let mut sources = SourceMap::new();
    sources.insert("state-change.yaml", MODEL.to_owned());
    compile(&spec, &sources).unwrap_or_else(|errors| panic!("{errors}"))
}

#[test]
fn the_compiled_condition_carries_the_held_states_the_answer_admits() {
    let ir = ir();
    let command = ir
        .commands()
        .values()
        .find(|command| command.name.to_string() == "demo.room.Report")
        .expect("the command");
    let joined = command
        .outcomes
        .iter()
        .find(|outcome| outcome.name.as_str() == "joined")
        .expect("the branch");
    let ResolvedCondition::StateChange {
        changes,
        states,
        predicate,
    } = &joined.condition
    else {
        panic!(
            "the condition is not the one written: {:?}",
            joined.condition
        );
    };
    assert!(*changes);
    // `join` arrives at `Added` and runs from every state, so the first report of `Added` is every
    // held state but `Added` itself — read off the transition, never authored.
    assert_eq!(
        states
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(","),
        "Gone,Pending"
    );
    assert_eq!(
        predicate.as_ref().map(ToString::to_string),
        Some("state == Added".to_owned()),
        "the input guard stays beside it, conjunctive"
    );
    assert_eq!(
        joined.test_strategy.as_str(),
        "construct_input_in_state",
        "a scenario must establish one of those states before it sends anything"
    );
}

#[test]
fn it_serialises_under_its_own_kind_and_leaves_every_other_branch_alone() {
    let ir = ir();
    let command = ir
        .commands()
        .values()
        .find(|command| command.name.to_string() == "demo.room.Report")
        .expect("the command");
    let written = serde_json::to_string(&command.outcomes).expect("the IR serialises");
    assert!(
        written.contains("\"kind\":\"state_change\""),
        "a consumer reads it by kind: {written}"
    );
    assert!(
        written.contains("\"changes\":true")
            && written.contains("\"states\":[\"Gone\",\"Pending\"]"),
        "both halves of the answer are in the bytes: {written}"
    );
    // The branch that answers every later report is untouched: no key, no kind, nothing added.
    let refreshed = command
        .outcomes
        .iter()
        .find(|outcome| outcome.name.as_str() == "refreshed")
        .expect("the default");
    assert_eq!(refreshed.condition, ResolvedCondition::Otherwise);
    assert_eq!(refreshed.test_strategy.as_str(), "default_branch");
}
