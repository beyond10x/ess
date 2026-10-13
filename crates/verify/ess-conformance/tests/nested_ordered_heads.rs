//! Nested heads of ordered selections: a selector reads a value its own outcome publishes
//! (`docs/design/published-value-operand.md`, "Interpreter and history checker", "Synthesis",
//! "Decisive acceptance"; beyond10x/ess#549).
//!
//! `tests/fixtures/nested-ordered-heads.yaml`: `OpenSlot` opens a scored slot in a group; `Join`
//! adds a scored membership to one side of a slot; `MatchInGroup` refuses with `Empty` while no slot
//! of the group is open, and otherwise publishes the best open slot of the group by
//! `[score desc, slot_id]`, then the best membership on each side **of that slot** by
//! `[element_score desc, membership_id]`. The inner selectors read the outer head through
//! `payload.demo.nested.PairMade.slot_id`, and the pair stores the slot and the left key through
//! `{payload: …}`.
use std::collections::BTreeMap;

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::interpret::Interpreted;
use ess_conformance::target::*;
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::ids::CorrelationId;
use ess_primitives::node::Node;

const MODEL: &str = include_str!("fixtures/nested-ordered-heads.yaml");

const SLOT_A: &str = "00000000-0000-4000-8000-00000000000a";
const SLOT_B: &str = "00000000-0000-4000-8000-00000000000b";
const SLOT_C: &str = "00000000-0000-4000-8000-00000000000c";
const SLOT_OTHER_GROUP: &str = "00000000-0000-4000-8000-00000000000d";
const SLOT_CLOSED: &str = "00000000-0000-4000-8000-00000000000e";
const PAIR: &str = "00000000-0000-4000-8000-0000000000aa";

fn membership(n: u8) -> String {
    format!("00000000-0000-4000-8000-0000000001{n:02x}")
}

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}"));
    let spec = Specification::assemble([(Source::new("nested.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("the fixture validates:\n{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn edit(text: &str, from: &str, to: &str) -> String {
    assert!(text.contains(from), "fixture edit did not land: {from}");
    text.replacen(from, to, 1)
}

/// The fixture with `MatchInGroup`'s `exists: false` refusal taken away: the heads read alone.
fn unguarded() -> String {
    edit(
        MODEL,
        "      - name: empty
        when_related:
          entity: demo.nested.Slot
          where: {all: [group_id == input.group_id, state == Open]}
          exists: false
        error: demo.nested.Empty
",
        "",
    )
}

fn text(value: &str) -> Node {
    Node::Text(value.to_owned())
}

fn number(value: i64) -> Node {
    Node::Number(ess_primitives::facts::Number::from(value))
}

fn correlation() -> CorrelationId {
    CorrelationId::new("nested-heads").unwrap()
}

fn target(text: &str) -> Interpreted {
    let target = Interpreted::for_model(ir(text));
    target
        .begin_scenario(&ScenarioContext::new(
            "demo.nested/authored/nested-heads".parse().unwrap(),
            correlation(),
        ))
        .unwrap();
    target
}

fn request(command: &str, input: &[(&str, Node)]) -> SemanticCommandRequest {
    SemanticCommandRequest {
        command: command.parse().unwrap(),
        actor: None,
        caller: None,
        input: input
            .iter()
            .map(|(name, value)| ((*name).to_owned(), value.clone()))
            .collect(),
        correlation: correlation(),
    }
}

fn outcome(result: &SemanticCommandResult) -> String {
    result
        .outcome
        .as_ref()
        .map_or_else(|| "<none>".to_owned(), |taken| taken.outcome.to_string())
}

fn send(target: &dyn ConformanceTarget, command: &str, input: &[(&str, Node)], expected: &str) {
    let answer = target
        .execute_command(request(&format!("demo.nested.{command}"), input))
        .unwrap_or_else(|error| panic!("{command} answers: {error}"));
    assert_eq!(outcome(&answer), expected, "{command}: {answer:?}");
}

fn open(target: &dyn ConformanceTarget, slot: &str, group: &str, score: i64) {
    send(
        target,
        "OpenSlot",
        &[
            ("slot_id", text(slot)),
            ("group_id", text(group)),
            ("score", number(score)),
        ],
        "opened",
    );
}

fn join(target: &dyn ConformanceTarget, n: u8, slot: &str, side: &str, key: &str, score: i64) {
    send(
        target,
        "Join",
        &[
            ("membership_id", text(&membership(n))),
            ("slot_id", text(slot)),
            ("side", text(side)),
            ("element_key", text(key)),
            ("element_score", number(score)),
        ],
        "joined",
    );
}

fn match_in(target: &dyn ConformanceTarget, group: &str) -> Result<SemanticCommandResult, String> {
    target
        .execute_command(request(
            "demo.nested.MatchInGroup",
            &[("pair_id", text(PAIR)), ("group_id", text(group))],
        ))
        .map_err(|error| error.to_string())
}

fn pairs(target: &dyn ConformanceTarget) -> Vec<BTreeMap<String, Node>> {
    target
        .query_view(SemanticViewRequest {
            view: "demo.nested.Pairs".parse().unwrap(),
            params: BTreeMap::new(),
            consistency: ess_primitives::consistency::QueryConsistency::Current,
            correlation: correlation(),
            deadline: Deadline::at(ess_primitives::time::Timestamp::from_epoch_millis(0)),
        })
        .expect("the view answers")
        .rows
}

/// Slots A (20), B (30) and C (10) open in group `g`, inserted in that order, so B is the outer
/// head; a better slot in another group, and a better slot in `g` that has closed. Every slot but B
/// holds a membership on each side that would outrank B's, so an inner read scoped by any other
/// slot, or by none, answers a different key.
fn arranged(target: &dyn ConformanceTarget) {
    open(target, SLOT_A, "g", 20);
    open(target, SLOT_OTHER_GROUP, "h", 99);
    open(target, SLOT_CLOSED, "g", 50);
    open(target, SLOT_B, "g", 30);
    open(target, SLOT_C, "g", 10);
    send(
        target,
        "CloseSlot",
        &[("slot_id", text(SLOT_CLOSED))],
        "closed",
    );
    join(target, 1, SLOT_A, "Left", "l-in-a", 90);
    join(target, 2, SLOT_B, "Left", "l-low", 10);
    join(target, 3, SLOT_B, "Left", "l-head", 40);
    join(target, 4, SLOT_B, "Right", "r-head", 35);
    join(target, 5, SLOT_C, "Left", "l-in-c", 95);
    join(target, 6, SLOT_C, "Right", "r-in-c", 95);
    join(target, 7, SLOT_B, "Right", "r-low", 5);
    join(target, 8, SLOT_CLOSED, "Left", "l-in-closed", 99);
    join(target, 9, SLOT_OTHER_GROUP, "Right", "r-in-other", 99);
}

#[test]
fn the_interpreter_takes_each_side_of_the_slot_it_chose() {
    let target = target(MODEL);
    arranged(&target);
    let paired = match_in(&target, "g").expect("answered");
    assert_eq!(outcome(&paired), "paired", "{paired:?}");
    let event = &paired.direct_events[0].payload;
    assert_eq!(event["slot_id"], text(SLOT_B), "{event:?}");
    assert_eq!(event["left_key"], text("l-head"), "{event:?}");
    assert_eq!(event["right_key"], text("r-head"), "{event:?}");
}

/// The right side of the U2 model also requires an element key other than the left one: its
/// selector reads two published values, one of which is itself read through the slot. Whatever
/// order the payload fields are declared or held in, each is resolved after what it reads.
#[test]
fn a_selector_reading_a_value_that_reads_another_resolves_after_both() {
    let chained = edit(
        MODEL,
        "where: {all: [slot_id == payload.demo.nested.PairMade.slot_id, side == Right, state == Present]}",
        "where: {all: [element_key != payload.demo.nested.PairMade.left_key, slot_id == payload.demo.nested.PairMade.slot_id, side == Right, state == Present]}",
    );
    let target = target(&chained);
    open(&target, SLOT_A, "g", 20);
    open(&target, SLOT_B, "g", 30);
    join(&target, 1, SLOT_B, "Left", "shared", 40);
    join(&target, 2, SLOT_B, "Right", "shared", 50);
    join(&target, 3, SLOT_B, "Right", "r-second", 20);
    join(&target, 4, SLOT_A, "Right", "r-in-a", 30);
    let paired = match_in(&target, "g").expect("answered");
    assert_eq!(outcome(&paired), "paired", "{paired:?}");
    let event = &paired.direct_events[0].payload;
    assert_eq!(event["slot_id"], text(SLOT_B), "{event:?}");
    assert_eq!(event["left_key"], text("shared"), "{event:?}");
    assert_eq!(event["right_key"], text("r-second"), "{event:?}");
}

#[test]
fn an_empty_outer_selection_publishes_nothing() {
    // The model's own answer for a group with no open slot is its refusal.
    let target = target(MODEL);
    open(&target, SLOT_OTHER_GROUP, "h", 99);
    join(&target, 1, SLOT_OTHER_GROUP, "Left", "l-in-other", 1);
    join(&target, 2, SLOT_OTHER_GROUP, "Right", "r-in-other", 1);
    let refused = match_in(&target, "g").expect("answered");
    assert_eq!(outcome(&refused), "empty");
    assert!(refused.direct_events.is_empty(), "{refused:?}");
    assert_eq!(pairs(&target), Vec::new(), "no pair is stored");

    // Without it, no outer head supplies no slot, and an inner read of it supplies nothing either.
    let target = self::target(&unguarded());
    open(&target, SLOT_OTHER_GROUP, "h", 99);
    join(&target, 1, SLOT_OTHER_GROUP, "Left", "l-in-other", 1);
    join(&target, 2, SLOT_OTHER_GROUP, "Right", "r-in-other", 1);
    let answered = match_in(&target, "g");
    assert!(
        answered
            .as_ref()
            .map_or(true, |result| result.outcome.is_none()),
        "no outer head publishes nothing: {answered:?}"
    );
    assert_eq!(pairs(&target), Vec::new(), "no pair is stored");

    // An outer head whose left side is empty publishes nothing, whatever waits in another slot.
    let target = self::target(MODEL);
    open(&target, SLOT_A, "g", 10);
    open(&target, SLOT_B, "g", 30);
    join(&target, 1, SLOT_A, "Left", "l-in-a", 90);
    join(&target, 2, SLOT_B, "Right", "r-head", 35);
    let answered = match_in(&target, "g");
    assert!(
        answered
            .as_ref()
            .map_or(true, |result| result.outcome.is_none()),
        "an empty inner selection publishes nothing: {answered:?}"
    );
    assert_eq!(pairs(&target), Vec::new(), "no pair is stored");
}

#[test]
fn the_event_and_the_stored_copy_hold_the_operand_value() {
    let target = target(MODEL);
    arranged(&target);
    let paired = match_in(&target, "g").expect("answered");
    assert_eq!(outcome(&paired), "paired", "{paired:?}");
    let event = &paired.direct_events[0].payload;
    let rows = pairs(&target);
    assert_eq!(rows.len(), 1, "{rows:?}");
    assert_eq!(rows[0]["slot_id"], event["slot_id"], "{rows:?} {event:?}");
    assert_eq!(rows[0]["left_key"], event["left_key"], "{rows:?} {event:?}");
    assert_eq!(rows[0]["slot_id"], text(SLOT_B), "{rows:?}");
    assert_eq!(rows[0]["left_key"], text("l-head"), "{rows:?}");
}

// ---- synthesis (unit U4) --------------------------------------------------------------------------

const PAIRED: &str = "demo.nested.MatchInGroup/outcome/paired";

fn synthesized(text: &str) -> ess_conformance::synthesize::Synthesis {
    ess_conformance::synthesize::synthesize(&ir(text))
}

fn suite_with_paired(text: &str) -> ess_conformance::ConformanceSuite {
    let synthesis = synthesized(text);
    let ids: Vec<String> = synthesis
        .suite
        .scenarios
        .keys()
        .map(ToString::to_string)
        .collect();
    assert!(
        ids.iter().any(|id| id == PAIRED),
        "{PAIRED} is synthesized: {ids:#?}\nrefusals: {:#?}",
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

/// An open slot a scenario arranged, as the steps send it.
#[derive(Debug, Clone)]
struct SentSlot {
    id: Node,
    group: Node,
    score: Node,
    open: bool,
}

/// A membership a scenario arranged, as the steps send it.
#[derive(Debug, Clone)]
struct SentMember {
    id: Node,
    slot: Node,
    side: Node,
    key: Node,
    score: Node,
    present: bool,
}

/// What the `paired` scenario arranges before `MatchInGroup`: the group it matches in, the slots in
/// the order opened and the memberships in the order joined.
fn arranged_before_match(
    suite: &ess_conformance::ConformanceSuite,
) -> (Node, Vec<SentSlot>, Vec<SentMember>) {
    let steps = &suite
        .scenarios
        .iter()
        .find(|(id, _)| id.to_string() == PAIRED)
        .expect("the paired scenario")
        .1
        .steps;
    // An instance a step captured is named by later steps instead of its literal identity.
    let mut named: BTreeMap<String, Node> = BTreeMap::new();
    let mut created: Option<Node> = None;
    let mut slots: Vec<SentSlot> = Vec::new();
    let mut members: Vec<SentMember> = Vec::new();
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
        let literal =
            |input: &BTreeMap<String, ess_conformance::ScenarioValue>, name: &str| match input
                .get(name)
            {
                Some(ess_conformance::ScenarioValue::Instance { instance }) => named
                    .get(&instance.to_string())
                    .cloned()
                    .unwrap_or(Node::Null),
                Some(value) => value.as_literal().cloned().unwrap_or(Node::Null),
                None => Node::Null,
            };
        created = match command.to_string().as_str() {
            "demo.nested.OpenSlot" => Some(literal(input, "slot_id")),
            "demo.nested.Join" => Some(literal(input, "membership_id")),
            _ => None,
        };
        match command.to_string().as_str() {
            "demo.nested.OpenSlot" => slots.push(SentSlot {
                id: literal(input, "slot_id"),
                group: literal(input, "group_id"),
                score: literal(input, "score"),
                open: true,
            }),
            "demo.nested.CloseSlot" => {
                let id = literal(input, "slot_id");
                for slot in slots.iter_mut().filter(|slot| slot.id == id) {
                    slot.open = false;
                }
            }
            "demo.nested.Join" => members.push(SentMember {
                id: literal(input, "membership_id"),
                slot: literal(input, "slot_id"),
                side: literal(input, "side"),
                key: literal(input, "element_key"),
                score: literal(input, "element_score"),
                present: true,
            }),
            "demo.nested.Leave" => {
                let id = literal(input, "membership_id");
                for member in members.iter_mut().filter(|member| member.id == id) {
                    member.present = false;
                }
            }
            "demo.nested.MatchInGroup" => return (literal(input, "group_id"), slots, members),
            _ => {}
        }
    }
    panic!("the paired scenario sends MatchInGroup: {steps:#?}");
}

/// The first of `rows` by `[score desc, id]`.
fn best<'r, R>(
    rows: impl IntoIterator<Item = &'r R>,
    key: impl Fn(&R) -> (&Node, &Node),
) -> Option<&'r R>
where
    R: 'r,
{
    rows.into_iter().min_by(|left, right| {
        let (left_score, left_id) = key(left);
        let (right_score, right_id) = key(right);
        right_score
            .cmp(left_score)
            .then_with(|| left_id.cmp(right_id))
    })
}

/// The `paired` scenario arranges three open slots of the group, the outer head neither the first
/// nor the last opened; and for each side, a membership in the first and in the last opened slot
/// that ranks above the inner head of the head slot and carries another key. The healthy
/// interpreter passes every scenario of the suite.
#[test]
fn synthesis_arranges_cross_decoys_in_the_other_slots() {
    let suite = suite_with_paired(MODEL);
    let (group, slots, members) = arranged_before_match(&suite);
    let open: Vec<&SentSlot> = slots
        .iter()
        .filter(|slot| slot.open && slot.group == group)
        .collect();
    assert!(open.len() >= 3, "three open slots in the group: {slots:#?}");
    let head = best(open.iter().copied(), |slot| (&slot.score, &slot.id)).expect("a head slot");
    let first = open[0];
    let last = open[open.len() - 1];
    assert_ne!(first.id, head.id, "{slots:#?}");
    assert_ne!(last.id, head.id, "{slots:#?}");
    for side in ["Left", "Right"] {
        let side = text(side);
        let of = |slot: &Node| -> Vec<&SentMember> {
            members
                .iter()
                .filter(|member| member.present && member.side == side && member.slot == *slot)
                .collect()
        };
        let in_head = of(&head.id);
        assert!(
            in_head.len() >= 2,
            "{side:?}: a ranked inner head: {members:#?}"
        );
        let inner = best(in_head.iter().copied(), |member| {
            (&member.score, &member.id)
        })
        .expect("an inner head");
        for other in [first, last] {
            assert!(
                of(&other.id)
                    .iter()
                    .any(|member| member.score > inner.score && member.key != inner.key),
                "{side:?}: a cross decoy in slot {:?} ranking above the inner head {inner:?}: \
                 {members:#?}",
                other.id
            );
        }
    }
    let healthy = statuses(&suite, &Interpreted::for_model(ir(MODEL)));
    assert_eq!(failed(&healthy), Vec::<&str>::new(), "{healthy:#?}");
}

/// A target scoping the inner reads by a rule the model does not state: the healthy interpreter,
/// whose published keys are replaced by the heads the rule picks. Where the rule's scope holds no
/// membership of a side, the target publishes no key for it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WrongScope {
    /// The first slot of the group opened, and still open.
    FirstInserted,
    /// The last slot of the group opened, and still open.
    LastInserted,
    /// No slot: the best membership of the side anywhere.
    IgnoresSlot,
}

struct Wrong {
    inner: Interpreted,
    rule: WrongScope,
    slots: std::cell::RefCell<Vec<SentSlot>>,
    members: std::cell::RefCell<Vec<SentMember>>,
}

impl Wrong {
    fn new(rule: WrongScope) -> Self {
        Self {
            inner: Interpreted::for_model(ir(MODEL)),
            rule,
            slots: std::cell::RefCell::new(Vec::new()),
            members: std::cell::RefCell::new(Vec::new()),
        }
    }

    fn key(&self, group: &Node, side: &str) -> Option<Node> {
        let slots = self.slots.borrow();
        let open: Vec<&SentSlot> = slots
            .iter()
            .filter(|slot| slot.open && slot.group == *group)
            .collect();
        let scope = match self.rule {
            WrongScope::FirstInserted => Some(open.first()?.id.clone()),
            WrongScope::LastInserted => Some(open.last()?.id.clone()),
            WrongScope::IgnoresSlot => None,
        };
        let members = self.members.borrow();
        let side = text(side);
        best(
            members.iter().filter(|member| {
                member.present
                    && member.side == side
                    && scope.as_ref().is_none_or(|slot| member.slot == *slot)
            }),
            |member| (&member.score, &member.id),
        )
        .map(|member| member.key.clone())
    }
}

impl ConformanceTarget for Wrong {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        self.inner.identity()
    }
    fn begin_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.slots.borrow_mut().clear();
        self.members.borrow_mut().clear();
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
        match (command.as_str(), outcome(&answer).as_str()) {
            ("demo.nested.OpenSlot", "opened") => self.slots.borrow_mut().push(SentSlot {
                id: field("slot_id"),
                group: field("group_id"),
                score: field("score"),
                open: true,
            }),
            ("demo.nested.CloseSlot", "closed") => {
                for slot in self.slots.borrow_mut().iter_mut() {
                    if slot.id == field("slot_id") {
                        slot.open = false;
                    }
                }
            }
            ("demo.nested.Join", "joined") => self.members.borrow_mut().push(SentMember {
                id: field("membership_id"),
                slot: field("slot_id"),
                side: field("side"),
                key: field("element_key"),
                score: field("element_score"),
                present: true,
            }),
            ("demo.nested.Leave", "left") => {
                for member in self.members.borrow_mut().iter_mut() {
                    if member.id == field("membership_id") {
                        member.present = false;
                    }
                }
            }
            ("demo.nested.MatchInGroup", "paired") => {
                let group = field("group_id");
                let left = self.key(&group, "Left");
                let right = self.key(&group, "Right");
                for event in &mut answer.direct_events {
                    for (name, key) in [("left_key", &left), ("right_key", &right)] {
                        match key {
                            Some(key) => {
                                event.payload.insert(name.to_owned(), key.clone());
                            }
                            None => {
                                event.payload.remove(name);
                            }
                        }
                    }
                }
            }
            _ => {}
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

fn assert_wrong_scope_fails(rule: WrongScope) {
    let suite = suite_with_paired(MODEL);
    let statuses = statuses(&suite, &Wrong::new(rule));
    assert!(
        failed(&statuses).contains(&PAIRED),
        "{rule:?}: {PAIRED} passed: {statuses:#?}"
    );
}

#[test]
fn a_target_scoping_by_the_first_inserted_slot_fails_a_scenario() {
    assert_wrong_scope_fails(WrongScope::FirstInserted);
}

#[test]
fn a_target_scoping_by_the_last_inserted_slot_fails_a_scenario() {
    assert_wrong_scope_fails(WrongScope::LastInserted);
}

#[test]
fn a_target_ignoring_the_slot_fails_a_scenario() {
    assert_wrong_scope_fails(WrongScope::IgnoresSlot);
}

/// A slot whose identity the target mints is a scope no scenario names in advance: the branch
/// reading it is refused by name, and the refusal says why.
#[test]
fn a_minted_scope_is_a_named_gap() {
    let minted = edit(
        MODEL,
        "    input:
      - {name: slot_id, type: demo.nested.SlotId}
      - {name: group_id, type: String}
      - {name: score, type: Integer}
    outcomes:
      - name: opened
        creates: demo.nested.Slot
        instance: slot_id
        sets: {group_id: input.group_id, score: input.score}
        emits: [demo.nested.SlotOpened]
        payload:
          demo.nested.SlotOpened: {slot_id: input.slot_id}",
        "    input:
      - {name: group_id, type: String}
      - {name: score, type: Integer}
    outcomes:
      - name: opened
        creates: demo.nested.Slot
        instance: slot_id
        sets: {group_id: input.group_id, score: input.score}
        emits: [demo.nested.SlotOpened]
        payload:
          demo.nested.SlotOpened: {slot_id: {generated: true}}",
    );
    let synthesis = synthesized(&minted);
    let ids: Vec<String> = synthesis
        .suite
        .scenarios
        .keys()
        .map(ToString::to_string)
        .collect();
    assert!(!ids.iter().any(|id| id == PAIRED), "{ids:#?}");
    let refusals = format!("{:?}", synthesis.refusals);
    assert!(
        refusals.contains("a value the target mints, which no scenario names in advance"),
        "{refusals}"
    );
    assert!(refusals.contains("demo.nested.Membership"), "{refusals}");
}
