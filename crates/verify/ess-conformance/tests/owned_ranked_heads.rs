//! A ranked head over rows an owner owns, scoped by another head's published owner (beyond10x/ess#621).
//!
//! `Slot` owns its `Membership` rows through `slot_id`. `MakeMatch` publishes the best open slot of
//! a group by `[score desc]`, then the best Present membership of that slot by
//! `[element_score desc]`, read through `payload.demo.owned.Matched.slot_id`. Synthesis creates the
//! memberships naming the head slot rather than under slots of their own, which it refused as rows
//! unarranged before; a target taking the wrong member fails.
use std::collections::BTreeMap;

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::interpret::Interpreted;
use ess_conformance::target::*;
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::node::Node;

const MODEL: &str = r"format: ess/27
system: demo
version: v1
domain: demo.owned
types:
  - {name: demo.owned.SlotId, kind: newtype, of: Uuid}
  - {name: demo.owned.MembershipId, kind: newtype, of: Uuid}
  - {name: demo.owned.MatchId, kind: newtype, of: Uuid}
entities:
  - name: demo.owned.Slot
    identity: {name: slot_id, type: demo.owned.SlotId}
    fields:
      - {name: group_id, type: String}
      - {name: score, type: Integer}
    relations:
      - {name: members, kind: owns, target: demo.owned.Membership, cardinality: many, via: slot_id}
    lifecycle:
      initial: Open
      states: [Open, Closed]
      terminal: [Closed]
      transitions:
        - {name: close, from: [Open], to: Closed}
  - name: demo.owned.Membership
    identity: {name: membership_id, type: demo.owned.MembershipId}
    fields:
      - {name: slot_id, type: demo.owned.SlotId}
      - {name: element_key, type: String}
      - {name: element_score, type: Integer}
    lifecycle:
      initial: Present
      states: [Present, Removed]
      terminal: [Removed]
      transitions:
        - {name: remove, from: [Present], to: Removed}
  - name: demo.owned.Match
    identity: {name: match_id, type: demo.owned.MatchId}
    fields:
      - {name: slot_id, type: demo.owned.SlotId}
    lifecycle:
      initial: Made
      states: [Made]
      terminal: [Made]
      transitions: []
events:
  - name: demo.owned.SlotOpened
    fields:
      - {name: slot_id, type: demo.owned.SlotId}
  - name: demo.owned.SlotClosed
    fields:
      - {name: slot_id, type: demo.owned.SlotId}
  - name: demo.owned.Joined
    fields:
      - {name: membership_id, type: demo.owned.MembershipId}
  - name: demo.owned.Left
    fields:
      - {name: membership_id, type: demo.owned.MembershipId}
  - name: demo.owned.Matched
    fields:
      - {name: match_id, type: demo.owned.MatchId}
      - {name: slot_id, type: demo.owned.SlotId}
      - {name: element_key, type: String}
commands:
  - name: demo.owned.OpenSlot
    input:
      - {name: slot_id, type: demo.owned.SlotId}
      - {name: group_id, type: String}
      - {name: score, type: Integer}
    outcomes:
      - name: opened
        creates: demo.owned.Slot
        instance: slot_id
        sets: {group_id: input.group_id, score: input.score}
        emits: [demo.owned.SlotOpened]
        payload:
          demo.owned.SlotOpened: {slot_id: input.slot_id}
  - name: demo.owned.Join
    input:
      - {name: membership_id, type: demo.owned.MembershipId}
      - {name: slot_id, type: demo.owned.SlotId}
      - {name: element_key, type: String}
      - {name: element_score, type: Integer}
    outcomes:
      - name: joined
        creates: demo.owned.Membership
        instance: membership_id
        sets:
          slot_id: input.slot_id
          element_key: input.element_key
          element_score: input.element_score
        emits: [demo.owned.Joined]
        payload:
          demo.owned.Joined: {membership_id: input.membership_id}
  - name: demo.owned.CloseSlot
    input:
      - {name: slot_id, type: demo.owned.SlotId}
    outcomes:
      - name: closed
        moves: demo.owned.Slot.close
        instance: slot_id
        emits: [demo.owned.SlotClosed]
        payload:
          demo.owned.SlotClosed: {slot_id: input.slot_id}
  - name: demo.owned.Leave
    input:
      - {name: membership_id, type: demo.owned.MembershipId}
    outcomes:
      - name: left
        moves: demo.owned.Membership.remove
        instance: membership_id
        emits: [demo.owned.Left]
        payload:
          demo.owned.Left: {membership_id: input.membership_id}
  - name: demo.owned.MakeMatch
    input:
      - {name: match_id, type: demo.owned.MatchId}
      - {name: group_id, type: String}
    outcomes:
      - name: matched
        creates: demo.owned.Match
        instance: match_id
        sets:
          slot_id: {payload: demo.owned.Matched.slot_id}
        emits: [demo.owned.Matched]
        payload:
          demo.owned.Matched:
            match_id: input.match_id
            slot_id:
              related:
                entity: demo.owned.Slot
                where: {all: [group_id == input.group_id, state == Open]}
                order_by: [score desc]
                ties: unordered
                first: slot_id
            element_key:
              related:
                entity: demo.owned.Membership
                where: {all: [slot_id == payload.demo.owned.Matched.slot_id, state == Present]}
                order_by: [element_score desc]
                ties: unordered
                first: element_key
";

const MATCHED: &str = "demo.owned.MakeMatch/outcome/matched";

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}"));
    let spec = Specification::assemble([(Source::new("owned.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("the fixture validates:\n{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn edit(text: &str, from: &str, to: &str) -> String {
    assert!(text.contains(from), "fixture edit did not land: {from}");
    text.replace(from, to)
}

/// The fixture without the ownership relation: the shape beyond10x/ess#618 synthesizes.
fn unowned() -> String {
    edit(
        MODEL,
        "    relations:\n      - {name: members, kind: owns, target: demo.owned.Membership, \
         cardinality: many, via: slot_id}\n",
        "",
    )
}

/// The fixture with the slot selection testing that the slot holds a Present membership.
fn tested(text: &str) -> String {
    edit(
        text,
        "where: {all: [group_id == input.group_id, state == Open]}",
        "where:\n                  all:\n                    - group_id == input.group_id\n                    - state == Open\n                    - exists:\n                        in: {entity: demo.owned.Membership}\n                        as: member\n                        that: {all: [member.slot_id == slot_id, member.state == Present]}",
    )
}

fn synthesized(text: &str) -> ess_conformance::ConformanceSuite {
    let synthesis = ess_conformance::synthesize::synthesize(&ir(text));
    let refusals: Vec<String> = synthesis
        .refusals
        .iter()
        .map(|refusal| format!("{refusal:?}"))
        .filter(|refusal| refusal.contains("demo.owned.MakeMatch/matched"))
        .collect();
    assert!(
        synthesis
            .suite
            .scenarios
            .keys()
            .any(|id| id.to_string() == MATCHED),
        "{MATCHED} is synthesized\nrefusals: {refusals:#?}"
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
            "demo.owned.OpenSlot" => self.slots.push(SentSlot {
                id: field("slot_id"),
                group: field("group_id"),
                score: field("score"),
                open: true,
            }),
            "demo.owned.CloseSlot" => {
                for slot in self
                    .slots
                    .iter_mut()
                    .filter(|slot| slot.id == field("slot_id"))
                {
                    slot.open = false;
                }
            }
            "demo.owned.Join" => self.members.push(SentMember {
                id: field("membership_id"),
                slot: field("slot_id"),
                key: field("element_key"),
                score: field("element_score"),
                present: true,
            }),
            "demo.owned.Leave" => {
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

    /// The Present members of `slot`, in the order they joined.
    fn present(&self, slot: &Node) -> Vec<&SentMember> {
        self.members
            .iter()
            .filter(|member| member.present && member.slot == *slot)
            .collect()
    }

    /// The open slots of `group`, in the order they opened; with `tested`, only those holding a
    /// Present member.
    fn open(&self, group: &Node, tested: bool) -> Vec<&SentSlot> {
        self.slots
            .iter()
            .filter(|slot| {
                slot.open && slot.group == *group && (!tested || !self.present(&slot.id).is_empty())
            })
            .collect()
    }
}

/// The first of `rows` by a descending `score`.
fn best<'r, R: 'r>(
    rows: impl IntoIterator<Item = &'r R>,
    score: impl Fn(&R) -> &Node,
) -> Option<&'r R> {
    rows.into_iter()
        .min_by(|left, right| score(right).cmp(score(left)))
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
            "demo.owned.OpenSlot" => Some(field("slot_id")),
            "demo.owned.Join" => Some(field("membership_id")),
            _ => None,
        };
        if command == "demo.owned.MakeMatch" {
            return (field("group_id"), held);
        }
        held.record(&command, &field);
    }
    panic!("the matched scenario sends MakeMatch: {steps:#?}");
}

/// The `matched` scenario ranks one of several open slots first, and in it several Present
/// members, created under that slot, whose head is unique and not the first to join. The healthy
/// interpreter passes every scenario of the suite.
fn assert_ranked(text: &str, tested: bool) {
    let suite = synthesized(text);
    let (group, held) = arranged_before_match(&suite);
    let open = held.open(&group, tested);
    assert!(open.len() >= 2, "several open slots: {held:#?}");
    let top = best(open.iter().copied(), |slot| &slot.score).expect("a head slot");
    let members = held.present(&top.id);
    assert!(members.len() >= 2, "several members: {held:#?}");
    let inner = best(members.iter().copied(), |member| &member.score).expect("an inner head");
    let tied = members
        .iter()
        .filter(|member| member.score == inner.score)
        .count();
    assert_eq!(tied, 1, "the inner head is unique: {held:#?}");
    assert_ne!(members[0].id, inner.id, "not the first to join: {held:#?}");
    let healthy = statuses(&suite, &Interpreted::for_model(ir(text)));
    assert_eq!(failed(&healthy), Vec::<&str>::new(), "{healthy:#?}");
    assert!(healthy.contains_key(MATCHED), "{healthy:#?}");
}

#[test]
fn the_unowned_shape_still_ranks_the_member_head() {
    assert_ranked(&unowned(), false);
}

#[test]
fn a_member_head_over_owned_rows_is_ranked_under_the_head_slot() {
    assert_ranked(MODEL, false);
}

#[test]
fn a_member_head_over_owned_rows_tested_by_the_slot_selection_is_ranked() {
    assert_ranked(&tested(MODEL), true);
}

/// A target that publishes the right slot but the wrong member: the healthy interpreter, its
/// published `element_key` replaced by the member the rule picks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Wrong {
    /// The first Present member to join the slot.
    FirstJoined,
    /// The best Present member of any open slot of the group.
    AnySlot,
}

struct WrongTarget {
    inner: Interpreted,
    rule: Wrong,
    held: std::cell::RefCell<Held>,
}

impl WrongTarget {
    fn new(text: &str, rule: Wrong) -> Self {
        Self {
            inner: Interpreted::for_model(ir(text)),
            rule,
            held: std::cell::RefCell::new(Held::default()),
        }
    }

    fn key(&self, group: &Node, slot: &Node) -> Option<Node> {
        let held = self.held.borrow();
        let member = match self.rule {
            Wrong::FirstJoined => held.present(slot).first().copied(),
            Wrong::AnySlot => {
                let open: Vec<&Node> = held
                    .open(group, false)
                    .iter()
                    .map(|slot| &slot.id)
                    .collect();
                best(
                    held.members
                        .iter()
                        .filter(|member| member.present && open.contains(&&member.slot)),
                    |member| &member.score,
                )
            }
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
        if command == "demo.owned.MakeMatch" {
            if taken.as_deref() == Some("matched") {
                let group = field("group_id");
                for event in &mut answer.direct_events {
                    let slot = event.payload.get("slot_id").cloned().unwrap_or(Node::Null);
                    match self.key(&group, &slot) {
                        Some(key) => {
                            event.payload.insert("element_key".to_owned(), key);
                        }
                        None => {
                            event.payload.remove("element_key");
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

fn assert_caught(text: &str) {
    let suite = synthesized(text);
    for rule in [Wrong::FirstJoined, Wrong::AnySlot] {
        let statuses = statuses(&suite, &WrongTarget::new(text, rule));
        assert!(
            failed(&statuses).contains(&MATCHED),
            "{rule:?}: {MATCHED} passed: {statuses:#?}"
        );
    }
}

#[test]
fn a_target_taking_the_wrong_owned_member_fails_the_scenario() {
    assert_caught(MODEL);
}

#[test]
fn a_target_taking_the_wrong_owned_member_fails_where_the_slot_selection_tests_it() {
    assert_caught(&tested(MODEL));
}

#[test]
fn a_target_taking_the_wrong_member_fails_without_the_ownership() {
    assert_caught(&unowned());
}
