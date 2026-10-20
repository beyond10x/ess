//! A ranked head scoped by another head's published value, where the outer selection tests each
//! slot's related rows (beyond10x/ess#618).
//!
//! `MakeMatch` publishes the best open slot that holds a Present membership on each side, by
//! `[score desc]` with `ties: unordered`, then the best Present membership on each side of that
//! slot by `[element_score desc]`, read through `payload.demo.chain.Matched.slot_id`. The members the
//! slot's tests need are joined before the members ranked for the head, so synthesis ranks the
//! head above them, or names the gap where no creating input can.
use std::collections::BTreeMap;

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::interpret::Interpreted;
use ess_conformance::target::*;
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
      - {name: score, type: Integer}
    outcomes:
      - name: opened
        creates: demo.chain.Slot
        instance: slot_id
        sets: {group_id: input.group_id, score: input.score}
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

/// The fixture with both member heads ranked by `[element_score asc]`: the members the slot's
/// tests are arranged with, created before any other member of the head slot, would rank first
/// unless the arrangement ranks the head above them.
fn ascending() -> String {
    edit(
        MODEL,
        "order_by: [element_score desc]",
        "order_by: [element_score asc]",
    )
}

/// The fixture with every score drawn from `low..=high`: the members the slot's tests are arranged
/// with take values from the same range as the rows ranked for the head.
fn ranged(low: u8, high: u8) -> String {
    let typed = edit(
        MODEL,
        "types:\n",
        &format!(
            "types:\n  - {{name: demo.chain.Score, kind: newtype, of: Integer, invariants: \
             [value >= {low}, value <= {high}]}}\n"
        ),
    );
    let typed = edit(
        &typed,
        "{name: score, type: Integer}",
        "{name: score, type: demo.chain.Score}",
    );
    edit(
        &typed,
        "{name: element_score, type: Integer}",
        "{name: element_score, type: demo.chain.Score}",
    )
}

fn narrow() -> String {
    ranged(0, 7)
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

fn statuses<T: ConformanceTarget>(
    suite: &ess_conformance::ConformanceSuite,
    target: &T,
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

fn text(value: &str) -> Node {
    Node::Text(value.to_owned())
}

/// A slot a scenario opened, as the steps send it.
#[derive(Debug, Clone)]
struct SentSlot {
    id: Node,
    group: Node,
    score: Node,
    open: bool,
}

/// A membership a scenario joined, as the steps send it.
#[derive(Debug, Clone)]
struct SentMember {
    id: Node,
    slot: Node,
    side: Node,
    key: Node,
    score: Node,
    present: bool,
}

/// The slots and memberships a sequence of sends leaves, in the order they were created.
#[derive(Debug, Default, Clone)]
struct Held {
    slots: Vec<SentSlot>,
    members: Vec<SentMember>,
}

impl Held {
    fn record(&mut self, command: &str, field: &dyn Fn(&str) -> Node) {
        match command {
            "demo.chain.OpenSlot" => self.slots.push(SentSlot {
                id: field("slot_id"),
                group: field("group_id"),
                score: field("score"),
                open: true,
            }),
            "demo.chain.CloseSlot" => {
                for slot in self
                    .slots
                    .iter_mut()
                    .filter(|slot| slot.id == field("slot_id"))
                {
                    slot.open = false;
                }
            }
            "demo.chain.Join" => self.members.push(SentMember {
                id: field("membership_id"),
                slot: field("slot_id"),
                side: field("side"),
                key: field("element_key"),
                score: field("element_score"),
                present: true,
            }),
            "demo.chain.Leave" => {
                for member in self
                    .members
                    .iter_mut()
                    .filter(|member| member.id == field("membership_id"))
                {
                    member.present = false;
                }
            }
            _ => {}
        }
    }

    /// The Present members of `side` in `slot`, in the order they joined.
    fn present(&self, slot: &Node, side: &str) -> Vec<&SentMember> {
        self.members
            .iter()
            .filter(|member| member.present && member.slot == *slot && member.side == text(side))
            .collect()
    }

    /// The open slots of `group` with a Present member on each side, in the order they opened.
    fn ready(&self, group: &Node) -> Vec<&SentSlot> {
        self.slots
            .iter()
            .filter(|slot| {
                slot.open
                    && slot.group == *group
                    && !self.present(&slot.id, "Left").is_empty()
                    && !self.present(&slot.id, "Right").is_empty()
            })
            .collect()
    }
}

/// The first of `rows` by `score`, descending or ascending.
fn best<'r, R>(
    rows: impl IntoIterator<Item = &'r R>,
    score: impl Fn(&R) -> &Node,
    descending: bool,
) -> Option<&'r R>
where
    R: 'r,
{
    rows.into_iter().min_by(|left, right| {
        let order = score(left).cmp(score(right));
        if descending {
            order.reverse()
        } else {
            order
        }
    })
}

/// What the `matched` scenario leaves before `MakeMatch`, and the group it matches in.
fn arranged_before_match(suite: &ess_conformance::ConformanceSuite) -> (Node, Held) {
    let steps = &suite
        .scenarios
        .iter()
        .find(|(id, _)| id.to_string() == MATCHED)
        .expect("the matched scenario")
        .1
        .steps;
    let mut named: BTreeMap<String, Node> = BTreeMap::new();
    let mut created: Option<Node> = None;
    let mut held = Held::default();
    for step in steps {
        if let ess_conformance::ScenarioStep::CaptureInstance { instance, .. } = step {
            if let Some(identity) = created.take() {
                named.insert(instance.to_string(), identity);
            }
            continue;
        }
        let ess_conformance::ScenarioStep::ExecuteCommand { command, input, .. } = step else {
            continue;
        };
        let field = |name: &str| match input.get(name) {
            Some(ess_conformance::ScenarioValue::Instance { instance }) => named
                .get(&instance.to_string())
                .cloned()
                .unwrap_or(Node::Null),
            Some(value) => value.as_literal().cloned().unwrap_or(Node::Null),
            None => Node::Null,
        };
        let command = command.to_string();
        created = match command.as_str() {
            "demo.chain.OpenSlot" => Some(field("slot_id")),
            "demo.chain.Join" => Some(field("membership_id")),
            _ => None,
        };
        if command == "demo.chain.MakeMatch" {
            return (field("group_id"), held);
        }
        held.record(&command, &field);
    }
    panic!("the matched scenario sends MakeMatch: {steps:#?}");
}

/// The `matched` scenario ranks one of several ready slots first, neither the first nor the last
/// opened, and arranges a ready slot beside it; in the head slot, each side holds several Present
/// members whose head is unique and not the first to join — the members the slot's tests need
/// joined first — and, where three rows are ranked (`three`), not the last either. The healthy
/// interpreter passes every scenario of the suite.
fn assert_ranked(text: &str, descending: bool, three: bool) {
    let suite = synthesized(text);
    let (group, held) = arranged_before_match(&suite);
    let ready = held.ready(&group);
    assert!(ready.len() >= 2, "several ready slots: {held:#?}");
    let first = best(ready.iter().copied(), |slot| &slot.score, true).expect("a head slot");
    assert_ne!(ready[0].id, first.id, "{held:#?}");
    assert_ne!(ready[ready.len() - 1].id, first.id, "{held:#?}");
    for side in ["Left", "Right"] {
        let members = held.present(&first.id, side);
        assert!(members.len() >= 3, "{side}: several members: {held:#?}");
        let inner = best(members.iter().copied(), |member| &member.score, descending)
            .expect("an inner head");
        let tied = members
            .iter()
            .filter(|member| member.score == inner.score)
            .count();
        assert_eq!(tied, 1, "{side}: the inner head is unique: {held:#?}");
        assert_ne!(
            members[0].id, inner.id,
            "{side}: not the first to join: {held:#?}"
        );
        if three {
            assert_ne!(
                members[members.len() - 1].id,
                inner.id,
                "{side}: not the last to join: {held:#?}"
            );
        }
    }
    let healthy = statuses(&suite, &Interpreted::for_model(ir(text)));
    assert_eq!(failed(&healthy), Vec::<&str>::new(), "{healthy:#?}");
    assert!(healthy.contains_key(MATCHED), "{healthy:#?}");
}

#[test]
fn a_member_head_scoped_by_a_ranked_slot_is_ranked_among_several_members() {
    assert_ranked(MODEL, true, true);
}

#[test]
fn an_ascending_member_head_ranks_above_the_members_the_slot_test_needs() {
    assert_ranked(&ascending(), false, true);
}

/// Under a narrow range the head still ranks above the member the slot's test joined first, with no
/// tie under `ties: unordered`.
#[test]
fn a_narrow_score_range_still_ranks_the_member_heads() {
    assert_ranked(&narrow(), true, false);
}

/// A target that publishes the right slot but the wrong member of each side: the healthy
/// interpreter, its published keys replaced by the member the rule picks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Wrong {
    /// The first Present member of the side to join the slot.
    FirstJoined,
    /// The last Present member of the side to join the slot.
    LastJoined,
    /// The best Present member of the side in the first ready slot opened.
    FirstReadySlot,
}

struct WrongTarget {
    inner: Interpreted,
    rule: Wrong,
    descending: bool,
    held: std::cell::RefCell<Held>,
}

impl WrongTarget {
    fn new(text: &str, rule: Wrong, descending: bool) -> Self {
        Self {
            inner: Interpreted::for_model(ir(text)),
            rule,
            descending,
            held: std::cell::RefCell::new(Held::default()),
        }
    }

    fn key(&self, group: &Node, slot: &Node, side: &str) -> Option<Node> {
        let held = self.held.borrow();
        let scope = match self.rule {
            Wrong::FirstReadySlot => held.ready(group).first()?.id.clone(),
            Wrong::FirstJoined | Wrong::LastJoined => slot.clone(),
        };
        let members = held.present(&scope, side);
        let member = match self.rule {
            Wrong::FirstJoined => members.first().copied(),
            Wrong::LastJoined => members.last().copied(),
            Wrong::FirstReadySlot => best(
                members.iter().copied(),
                |member| &member.score,
                self.descending,
            ),
        };
        member.map(|member| member.key.clone())
    }
}

impl ConformanceTarget for WrongTarget {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        self.inner.identity()
    }
    fn begin_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        *self.held.borrow_mut() = Held::default();
        self.inner.begin_scenario(scenario)
    }
    fn end_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.inner.end_scenario(scenario)
    }
    fn establish_entity(&self, request: EntitySetupRequest) -> Result<(), TargetError> {
        self.inner.establish_entity(request)
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        let command = request.command.to_string();
        let input = request.input.clone();
        let mut answer = self.inner.execute_command(request)?;
        let field = |name: &str| input.get(name).cloned().unwrap_or(Node::Null);
        let taken = answer
            .outcome
            .as_ref()
            .map(|taken| taken.outcome.to_string());
        if command == "demo.chain.MakeMatch" {
            if taken.as_deref() == Some("matched") {
                let group = field("group_id");
                for event in &mut answer.direct_events {
                    let slot = event.payload.get("slot_id").cloned().unwrap_or(Node::Null);
                    for (name, side) in [("left_key", "Left"), ("right_key", "Right")] {
                        match self.key(&group, &slot, side) {
                            Some(key) => {
                                event.payload.insert(name.to_owned(), key);
                            }
                            None => {
                                event.payload.remove(name);
                            }
                        }
                    }
                }
            }
        } else if taken.is_some() {
            self.held.borrow_mut().record(&command, &field);
        }
        Ok(answer)
    }
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        self.inner.query_view(request)
    }
    fn observe_events(
        &self,
        request: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        self.inner.observe_events(request)
    }
    fn configure_external_outcome(
        &self,
        control: ExternalOutcomeControl,
    ) -> Result<(), TargetError> {
        self.inner.configure_external_outcome(control)
    }
    fn redeliver_event(&self, request: RedeliveryRequest) -> Result<(), TargetError> {
        self.inner.redeliver_event(request)
    }
}

fn assert_caught(text: &str, descending: bool, rules: &[Wrong]) {
    let suite = synthesized(text);
    for &rule in rules {
        let statuses = statuses(&suite, &WrongTarget::new(text, rule, descending));
        assert!(
            failed(&statuses).contains(&MATCHED),
            "{rule:?}: {MATCHED} passed: {statuses:#?}"
        );
    }
}

#[test]
fn a_target_taking_the_wrong_member_or_slot_fails_the_scenario() {
    assert_caught(
        MODEL,
        true,
        &[Wrong::FirstJoined, Wrong::LastJoined, Wrong::FirstReadySlot],
    );
}

#[test]
fn a_target_taking_the_first_member_to_join_fails_under_an_ascending_order() {
    assert_caught(
        &ascending(),
        false,
        &[Wrong::FirstJoined, Wrong::LastJoined, Wrong::FirstReadySlot],
    );
}

/// Two ranked rows fit beside the slot test's member in the narrow range, so the head is the last
/// of them to join, as in any arrangement of two; it is never the member the slot test needed.
#[test]
fn a_target_taking_the_wrong_member_fails_under_a_narrow_score_range() {
    assert_caught(
        &narrow(),
        true,
        &[Wrong::FirstJoined, Wrong::FirstReadySlot],
    );
}

/// Where the range leaves no arrangement ranking a member head above the members the slot's tests
/// need without a tie, the branch is refused naming that, not as a branch admitting one row.
#[test]
fn a_range_too_narrow_to_rank_the_head_above_the_slot_test_members_is_refused_by_name() {
    let synthesis = ess_conformance::synthesize::synthesize(&ir(&ranged(0, 3)));
    assert!(
        !synthesis
            .suite
            .scenarios
            .keys()
            .any(|id| id.to_string() == MATCHED),
        "{MATCHED} is refused"
    );
    let refusals: Vec<String> = synthesis
        .refusals
        .iter()
        .map(|refusal| format!("{refusal:?}"))
        .filter(|refusal| refusal.contains("demo.chain.MakeMatch/matched"))
        .collect();
    assert!(
        refusals.iter().any(|refusal| refusal.contains(
            "the ranked head is not arranged above the rows another selector's test places in its scope"
        )),
        "{refusals:#?}"
    );
    assert!(
        !refusals
            .iter()
            .any(|refusal| refusal.contains("admits at most one selected row")),
        "{refusals:#?}"
    );
}

/// Where the range leaves no value above the first slot dealt, the slots are dealt from the other
/// end of the range, and the branch stands.
#[test]
fn a_range_with_no_value_above_the_first_row_dealt_is_dealt_from_the_other_end() {
    let text = ranged(1, 5);
    let suite = synthesized(&text);
    let healthy = statuses(&suite, &Interpreted::for_model(ir(&text)));
    assert_eq!(failed(&healthy), Vec::<&str>::new(), "{healthy:#?}");
    assert!(healthy.contains_key(MATCHED), "{healthy:#?}");
}
