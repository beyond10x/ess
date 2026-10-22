//! A ranked head whose selection reads a field the creating command leaves unset, which a later
//! command sets (beyond10x/ess#626).
//!
//! `MakeMatch` publishes the best open, enabled slot of a group that holds a Present membership on
//! each side, by `[score desc]` with `ties: unordered`. `OpenSlot` creates an enabled slot but
//! always scores it 0, so no two of its slots rank apart; `ScoreSlot` creates a slot with the score
//! it is sent and leaves `enabled` unset, which `EnableSlot` sets afterwards. Synthesis arranges the
//! ranked slots through `ScoreSlot` and then `EnableSlot`, so the head is observed, where it
//! refused the branch as admitting at most one selected row before.
use std::collections::BTreeMap;

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::interpret::Interpreted;
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::node::Node;

const MODEL: &str = r"format: ess/27
system: demo
version: v1
domain: demo.chain
types:
  - {name: demo.chain.SlotId, kind: newtype, of: Uuid}
  - {name: demo.chain.MembershipId, kind: newtype, of: Uuid}
  - {name: demo.chain.MatchId, kind: newtype, of: Uuid}
  - {name: demo.chain.Side, kind: enum, variants: [Left, Right]}
entities:
  - name: demo.chain.Slot
    identity: {name: slot_id, type: demo.chain.SlotId}
    fields:
      - {name: group_id, type: String}
      - {name: score, type: Integer}
      - {name: enabled, type: Boolean}
    lifecycle:
      initial: Open
      states: [Open, Closed]
      terminal: [Closed]
      transitions:
        - {name: close, from: [Open], to: Closed}
  - name: demo.chain.Membership
    identity: {name: membership_id, type: demo.chain.MembershipId}
    fields:
      - {name: slot_id, type: demo.chain.SlotId}
      - {name: side, type: demo.chain.Side}
      - {name: element_key, type: String}
      - {name: element_score, type: Integer}
    lifecycle:
      initial: Present
      states: [Present, Removed]
      terminal: [Removed]
      transitions:
        - {name: remove, from: [Present], to: Removed}
  - name: demo.chain.Match
    identity: {name: match_id, type: demo.chain.MatchId}
    fields:
      - {name: slot_id, type: demo.chain.SlotId}
    lifecycle:
      initial: Made
      states: [Made]
      terminal: [Made]
      transitions: []
events:
  - name: demo.chain.SlotOpened
    fields:
      - {name: slot_id, type: demo.chain.SlotId}
  - name: demo.chain.SlotClosed
    fields:
      - {name: slot_id, type: demo.chain.SlotId}
  - name: demo.chain.Joined
    fields:
      - {name: membership_id, type: demo.chain.MembershipId}
  - name: demo.chain.Left
    fields:
      - {name: membership_id, type: demo.chain.MembershipId}
  - name: demo.chain.Matched
    fields:
      - {name: match_id, type: demo.chain.MatchId}
      - {name: slot_id, type: demo.chain.SlotId}
      - {name: left_key, type: String}
      - {name: right_key, type: String}
commands:
  - name: demo.chain.OpenSlot
    input:
      - {name: slot_id, type: demo.chain.SlotId}
      - {name: group_id, type: String}
    outcomes:
      - name: opened
        creates: demo.chain.Slot
        instance: slot_id
        sets: {group_id: input.group_id, score: 0, enabled: true}
        emits: [demo.chain.SlotOpened]
        payload:
          demo.chain.SlotOpened: {slot_id: input.slot_id}
  - name: demo.chain.ScoreSlot
    input:
      - {name: slot_id, type: demo.chain.SlotId}
      - {name: group_id, type: String}
      - {name: score, type: Integer}
    outcomes:
      - name: scored
        creates: demo.chain.Slot
        instance: slot_id
        sets: {group_id: input.group_id, score: input.score}
        emits: [demo.chain.SlotOpened]
        payload:
          demo.chain.SlotOpened: {slot_id: input.slot_id}
  - name: demo.chain.EnableSlot
    input:
      - {name: slot_id, type: demo.chain.SlotId}
      - {name: enabled, type: Boolean}
    outcomes:
      - name: set
        updates: demo.chain.Slot
        instance: slot_id
        sets: {enabled: input.enabled}
        emits: [demo.chain.SlotOpened]
        payload:
          demo.chain.SlotOpened: {slot_id: input.slot_id}
  - name: demo.chain.CloseSlot
    input:
      - {name: slot_id, type: demo.chain.SlotId}
    outcomes:
      - name: closed
        moves: demo.chain.Slot.close
        instance: slot_id
        emits: [demo.chain.SlotClosed]
        payload:
          demo.chain.SlotClosed: {slot_id: input.slot_id}
  - name: demo.chain.Join
    input:
      - {name: membership_id, type: demo.chain.MembershipId}
      - {name: slot_id, type: demo.chain.SlotId}
      - {name: side, type: demo.chain.Side}
      - {name: element_key, type: String}
      - {name: element_score, type: Integer}
    outcomes:
      - name: joined
        creates: demo.chain.Membership
        instance: membership_id
        sets:
          slot_id: input.slot_id
          side: input.side
          element_key: input.element_key
          element_score: input.element_score
        emits: [demo.chain.Joined]
        payload:
          demo.chain.Joined: {membership_id: input.membership_id}
  - name: demo.chain.Leave
    input:
      - {name: membership_id, type: demo.chain.MembershipId}
    outcomes:
      - name: left
        moves: demo.chain.Membership.remove
        instance: membership_id
        emits: [demo.chain.Left]
        payload:
          demo.chain.Left: {membership_id: input.membership_id}
  - name: demo.chain.MakeMatch
    input:
      - {name: match_id, type: demo.chain.MatchId}
      - {name: group_id, type: String}
    outcomes:
      - name: matched
        creates: demo.chain.Match
        instance: match_id
        sets:
          slot_id: {payload: demo.chain.Matched.slot_id}
        emits: [demo.chain.Matched]
        payload:
          demo.chain.Matched:
            match_id: input.match_id
            slot_id:
              related:
                entity: demo.chain.Slot
                where:
                  all:
                    - group_id == input.group_id
                    - state == Open
                    - enabled == true
                    - exists:
                        in: {entity: demo.chain.Membership}
                        as: member
                        that: {all: [member.slot_id == slot_id, member.side == Left, member.state == Present]}
                    - exists:
                        in: {entity: demo.chain.Membership}
                        as: member
                        that: {all: [member.slot_id == slot_id, member.side == Right, member.state == Present]}
                order_by: [score desc]
                ties: unordered
                first: slot_id
            left_key:
              related:
                entity: demo.chain.Membership
                where: {all: [slot_id == payload.demo.chain.Matched.slot_id, side == Left, state == Present]}
                order_by: [element_score desc]
                ties: unordered
                first: element_key
            right_key:
              related:
                entity: demo.chain.Membership
                where: {all: [slot_id == payload.demo.chain.Matched.slot_id, side == Right, state == Present]}
                order_by: [element_score desc]
                ties: unordered
                first: element_key
";

const MATCHED: &str = "demo.chain.MakeMatch/outcome/matched";

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}"));
    let spec = Specification::assemble([(Source::new("chain.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("the fixture validates:\n{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn edit(text: &str, from: &str, to: &str) -> String {
    assert!(text.contains(from), "fixture edit did not land: {from}");
    text.replace(from, to)
}

fn synthesized(text: &str) -> ess_conformance::ConformanceSuite {
    let synthesis = ess_conformance::synthesize::synthesize(&ir(text));
    let ids: Vec<String> = synthesis
        .suite
        .scenarios
        .keys()
        .map(ToString::to_string)
        .collect();
    assert!(
        ids.iter().any(|id| id == MATCHED),
        "{MATCHED} is synthesized: {ids:#?}\nrefusals: {:#?}",
        synthesis.refusals
    );
    synthesis.suite
}

fn statuses(
    suite: &ess_conformance::ConformanceSuite,
    target: &Interpreted,
) -> BTreeMap<String, ess_conformance::report::Status> {
    let admitted =
        ess_conformance::AdmittedSuite::from_suite(suite).unwrap_or_else(|error| panic!("{error}"));
    ess_conformance::Runner::new(
        ess_conformance::RunnerConfig::default(),
        ess_conformance::AdvancingClock::default(),
        ess_conformance::Ids::for_suite(suite),
    )
    .run_admitted(&admitted, target)
    .into_report()
    .scenarios
    .into_iter()
    .map(|scenario| (scenario.scenario.to_string(), scenario.status))
    .collect()
}

fn failed(statuses: &BTreeMap<String, ess_conformance::report::Status>) -> Vec<&str> {
    statuses
        .iter()
        .filter(|(_, status)| **status != ess_conformance::report::Status::Passed)
        .map(|(id, _)| id.as_str())
        .collect()
}

/// The commands the `matched` scenario sends before `MakeMatch`, in order, each with the literal
/// `score` it sends where it sends one.
fn sent_before_match(suite: &ess_conformance::ConformanceSuite) -> Vec<(String, Option<Node>)> {
    let steps = &suite
        .scenarios
        .iter()
        .find(|(id, _)| id.to_string() == MATCHED)
        .expect("the matched scenario")
        .1
        .steps;
    let mut sent = Vec::new();
    for step in steps {
        let ess_conformance::ScenarioStep::ExecuteCommand { command, input, .. } = step else {
            continue;
        };
        let command = command.to_string();
        if command == "demo.chain.MakeMatch" {
            return sent;
        }
        let score = input
            .get("score")
            .and_then(|value| value.as_literal().cloned());
        sent.push((command, score));
    }
    panic!("the matched scenario sends MakeMatch: {steps:#?}");
}

/// The branch is witnessed, and the healthy interpreter passes every scenario of the suite.
#[test]
fn a_ranked_head_filtering_on_a_field_a_later_command_sets_is_witnessed() {
    let suite = synthesized(MODEL);
    let healthy = statuses(&suite, &Interpreted::for_model(ir(MODEL)));
    assert_eq!(failed(&healthy), Vec::<&str>::new(), "{healthy:#?}");
    assert!(healthy.contains_key(MATCHED), "{healthy:#?}");
}

/// The slots ranked for the head are scored apart by `ScoreSlot` and enabled by `EnableSlot`
/// afterwards: no `OpenSlot` slot, all scored 0, ranks apart from another.
#[test]
fn the_ranked_slots_are_scored_apart_and_enabled_by_the_later_command() {
    let sent = sent_before_match(&synthesized(MODEL));
    let scores: Vec<&Node> = sent
        .iter()
        .filter(|(command, _)| command == "demo.chain.ScoreSlot")
        .filter_map(|(_, score)| score.as_ref())
        .collect();
    let distinct: std::collections::BTreeSet<&Node> = scores.iter().copied().collect();
    assert!(distinct.len() >= 2, "several scores: {sent:#?}");
    let enabled = sent
        .iter()
        .filter(|(command, _)| command == "demo.chain.EnableSlot")
        .count();
    assert!(enabled >= 2, "each ranked slot is enabled: {sent:#?}");
}

/// A target ranking the slots the other way publishes another slot, and fails the scenario.
#[test]
fn a_target_ranking_the_slots_ascending_fails_the_matched_scenario() {
    let suite = synthesized(MODEL);
    let wrong = edit(MODEL, "order_by: [score desc]", "order_by: [score asc]");
    let statuses = statuses(&suite, &Interpreted::for_model(ir(&wrong)));
    assert!(
        failed(&statuses).contains(&MATCHED),
        "{MATCHED} passed: {statuses:#?}"
    );
}

/// Without the `enabled` conjunct the branch was already witnessed; it still is.
#[test]
fn without_the_later_set_field_the_branch_is_still_witnessed() {
    let text = edit(MODEL, "                    - enabled == true\n", "");
    let suite = synthesized(&text);
    let healthy = statuses(&suite, &Interpreted::for_model(ir(&text)));
    assert_eq!(failed(&healthy), Vec::<&str>::new(), "{healthy:#?}");
}
