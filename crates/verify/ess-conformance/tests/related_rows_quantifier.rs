//! A selector tests each candidate's related rows, `exists: {in: {entity: E}, as: b, that: P}`
//! (`docs/design/related-rows-quantifier.md`, "Interpreter and history checker", "Synthesis",
//! "Decisive acceptance"; beyond10x/ess#551).
//!
//! `tests/fixtures/related-rows-quantifier.yaml`: `OpenSlot` opens a scored slot in a group; `Join`
//! adds a scored membership to one side of a slot and `Leave` removes it; `MatchInGroup` refuses
//! with `NoMatchableSlot` while no open slot of the group holds a `Present` membership on each
//! side, and otherwise publishes the best such slot by `[score desc, slot_id]`, then the best
//! membership on each side of that slot.
use std::collections::BTreeMap;

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::interpret::execute::{execute, Externals, Step, Store, Undetermined};
use ess_conformance::interpret::Interpreted;
use ess_conformance::target::*;
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::ids::CorrelationId;
use ess_primitives::node::Node;

const MODEL: &str = include_str!("fixtures/related-rows-quantifier.yaml");

const SLOT_A: &str = "00000000-0000-4000-8000-00000000000a";
const SLOT_B: &str = "00000000-0000-4000-8000-00000000000b";
const SLOT_C: &str = "00000000-0000-4000-8000-00000000000c";
const SLOT_OTHER_GROUP: &str = "00000000-0000-4000-8000-00000000000d";
const PAIR: &str = "00000000-0000-4000-8000-0000000000aa";

fn membership(n: u8) -> String {
    format!("00000000-0000-4000-8000-0000000001{n:02x}")
}

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}"));
    let spec = Specification::assemble([(Source::new("slots.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("the fixture validates:\n{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn edit(text: &str, from: &str, to: &str) -> String {
    assert!(text.contains(from), "fixture edit did not land: {from}");
    text.replacen(from, to, 1)
}

/// The fixture with `MatchInGroup`'s `no-match` refusal taken away: the heads read alone.
fn unguarded() -> String {
    let start = MODEL.find("      - name: no-match\n").expect("the refusal");
    let end = MODEL.find("      - name: paired\n").expect("the pairing");
    format!("{}{}", &MODEL[..start], &MODEL[end..])
}

fn text(value: &str) -> Node {
    Node::Text(value.to_owned())
}

fn number(value: i64) -> Node {
    Node::Number(ess_primitives::facts::Number::from(value))
}

fn correlation() -> CorrelationId {
    CorrelationId::new("related-rows").unwrap()
}

fn target(text: &str) -> Interpreted {
    let target = Interpreted::for_model(ir(text));
    target
        .begin_scenario(&ScenarioContext::new(
            "demo.slots/authored/related-rows".parse().unwrap(),
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
        .execute_command(request(&format!("demo.slots.{command}"), input))
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

fn leave(target: &dyn ConformanceTarget, n: u8) {
    send(
        target,
        "Leave",
        &[("membership_id", text(&membership(n)))],
        "left",
    );
}

fn match_in(target: &dyn ConformanceTarget, group: &str) -> SemanticCommandResult {
    target
        .execute_command(request(
            "demo.slots.MatchInGroup",
            &[("pair_id", text(PAIR)), ("group_id", text(group))],
        ))
        .unwrap_or_else(|error| panic!("MatchInGroup answers: {error}"))
}

// ---- the interpreter (unit R3) --------------------------------------------------------------------

/// C (40) ranks first in group `g` but holds no membership on its right side; B (30) holds one on
/// each side, and A (20) too. The pair is made on B, with B's own members.
#[test]
fn the_interpreter_skips_a_higher_scored_slot_with_an_empty_side() {
    let target = target(MODEL);
    open(&target, SLOT_A, "g", 20);
    open(&target, SLOT_B, "g", 30);
    open(&target, SLOT_C, "g", 40);
    open(&target, SLOT_OTHER_GROUP, "h", 99);
    join(&target, 1, SLOT_C, "Left", "l-in-c", 90);
    join(&target, 2, SLOT_B, "Left", "l-in-b", 10);
    join(&target, 3, SLOT_B, "Right", "r-in-b", 10);
    join(&target, 4, SLOT_A, "Left", "l-in-a", 50);
    join(&target, 5, SLOT_A, "Right", "r-in-a", 50);
    join(&target, 6, SLOT_OTHER_GROUP, "Right", "r-in-other", 1);
    let paired = match_in(&target, "g");
    assert_eq!(outcome(&paired), "paired", "{paired:?}");
    let event = &paired.direct_events[0].payload;
    assert_eq!(event["slot_id"], text(SLOT_B), "{event:?}");
    assert_eq!(event["left_key"], text("l-in-b"), "{event:?}");
    assert_eq!(event["right_key"], text("r-in-b"), "{event:?}");
}

/// No open slot of the group holds a membership on each side: the model refuses by name, whatever
/// waits in another group or on one side of each slot.
#[test]
fn no_matchable_slot_is_refused_by_name() {
    let target = target(MODEL);
    let refused = match_in(&target, "g");
    assert_eq!(outcome(&refused), "no-match", "an empty group: {refused:?}");

    open(&target, SLOT_A, "g", 20);
    open(&target, SLOT_C, "g", 40);
    open(&target, SLOT_OTHER_GROUP, "h", 99);
    join(&target, 1, SLOT_C, "Left", "l-in-c", 90);
    join(&target, 2, SLOT_A, "Right", "r-in-a", 50);
    join(&target, 3, SLOT_OTHER_GROUP, "Left", "l-in-other", 1);
    join(&target, 4, SLOT_OTHER_GROUP, "Right", "r-in-other", 1);
    let refused = match_in(&target, "g");
    assert_eq!(outcome(&refused), "no-match", "{refused:?}");
    assert!(refused.direct_events.is_empty(), "{refused:?}");
}

/// A membership that left its slot is no member of that side: B's right side is empty once its
/// one right member left, so the pair is made on A; once A's left member leaves too, nothing
/// matches.
#[test]
fn a_removed_member_does_not_make_a_side_matchable() {
    let target = target(MODEL);
    open(&target, SLOT_A, "g", 20);
    open(&target, SLOT_B, "g", 30);
    join(&target, 1, SLOT_B, "Left", "l-in-b", 10);
    join(&target, 2, SLOT_B, "Right", "r-in-b", 10);
    join(&target, 3, SLOT_A, "Left", "l-in-a", 50);
    join(&target, 4, SLOT_A, "Right", "r-in-a", 50);
    leave(&target, 2);
    let paired = match_in(&target, "g");
    assert_eq!(outcome(&paired), "paired", "{paired:?}");
    assert_eq!(
        paired.direct_events[0].payload["slot_id"],
        text(SLOT_A),
        "{paired:?}"
    );

    let target = self::target(MODEL);
    open(&target, SLOT_A, "g", 20);
    join(&target, 3, SLOT_A, "Left", "l-in-a", 50);
    join(&target, 4, SLOT_A, "Right", "r-in-a", 50);
    leave(&target, 3);
    let refused = match_in(&target, "g");
    assert_eq!(outcome(&refused), "no-match", "{refused:?}");
}

fn run(
    ir: &EssIr,
    store: &mut Store,
    command: &str,
    input: &[(&str, Node)],
) -> Result<Step, Undetermined> {
    let input: BTreeMap<String, Node> = input
        .iter()
        .map(|(name, value)| ((*name).to_owned(), value.clone()))
        .collect();
    let mut steps = execute(
        ir,
        store,
        &format!("demo.slots.{command}").parse().unwrap(),
        &input,
        &Externals::Withheld,
    )?;
    assert_eq!(steps.len(), 1);
    let step = steps.remove(0);
    *store = step.next.clone();
    Ok(step)
}

/// A side the store does not hold leaves every related row's test unknown: which slot is the head
/// is not guessed, and the store is unchanged.
#[test]
fn an_abstract_related_value_leaves_the_head_undecidable() {
    let model = ir(&edit(&unguarded(), "          side: input.side\n", ""));
    let mut store = Store::default();
    for (slot, score) in [(SLOT_A, 20), (SLOT_B, 30)] {
        run(
            &model,
            &mut store,
            "OpenSlot",
            &[
                ("slot_id", text(slot)),
                ("group_id", text("g")),
                ("score", number(score)),
            ],
        )
        .unwrap_or_else(|error| panic!("opened: {error:?}"));
    }
    for (n, slot, side) in [
        (1, SLOT_A, "Left"),
        (2, SLOT_A, "Right"),
        (3, SLOT_B, "Left"),
    ] {
        run(
            &model,
            &mut store,
            "Join",
            &[
                ("membership_id", text(&membership(n))),
                ("slot_id", text(slot)),
                ("side", text(side)),
                ("element_key", text(&format!("k{n}"))),
                ("element_score", number(1)),
            ],
        )
        .unwrap_or_else(|error| panic!("joined: {error:?}"));
    }
    let before = store.clone();
    let answered = run(
        &model,
        &mut store,
        "MatchInGroup",
        &[("pair_id", text(PAIR)), ("group_id", text("g"))],
    );
    assert!(
        matches!(answered, Err(Undetermined::Undecidable { .. })),
        "{answered:?}"
    );
    assert_eq!(store, before);
}

// ---- synthesis (unit R4) --------------------------------------------------------------------------

const PAIRED: &str = "demo.slots.MatchInGroup/outcome/paired";
const NO_MATCH: &str = "demo.slots.MatchInGroup/outcome/no-match";

fn synthesized(text: &str) -> ess_conformance::synthesize::Synthesis {
    ess_conformance::synthesize::synthesize(&ir(text))
}

fn suite_with(text: &str, scenarios: &[&str]) -> ess_conformance::ConformanceSuite {
    let synthesis = synthesized(text);
    let ids: Vec<String> = synthesis
        .suite
        .scenarios
        .keys()
        .map(ToString::to_string)
        .collect();
    for scenario in scenarios {
        assert!(
            ids.iter().any(|id| id == scenario),
            "{scenario} is synthesized: {ids:#?}\nrefusals: {:#?}",
            synthesis.refusals
        );
    }
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
    present: bool,
}

/// The slots and memberships a scenario arranges before it sends `MatchInGroup`, and the group it
/// matches in.
fn arranged_before_match(
    suite: &ess_conformance::ConformanceSuite,
    scenario: &str,
) -> (Node, Vec<SentSlot>, Vec<SentMember>) {
    let steps = &suite
        .scenarios
        .iter()
        .find(|(id, _)| id.to_string() == scenario)
        .expect("the scenario")
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
            "demo.slots.OpenSlot" => Some(literal(input, "slot_id")),
            "demo.slots.Join" => Some(literal(input, "membership_id")),
            _ => None,
        };
        match command.to_string().as_str() {
            "demo.slots.OpenSlot" => slots.push(SentSlot {
                id: literal(input, "slot_id"),
                group: literal(input, "group_id"),
                score: literal(input, "score"),
                open: true,
            }),
            "demo.slots.CloseSlot" => {
                let id = literal(input, "slot_id");
                for slot in slots.iter_mut().filter(|slot| slot.id == id) {
                    slot.open = false;
                }
            }
            "demo.slots.Join" => members.push(SentMember {
                id: literal(input, "membership_id"),
                slot: literal(input, "slot_id"),
                side: literal(input, "side"),
                present: true,
            }),
            "demo.slots.Leave" => {
                let id = literal(input, "membership_id");
                for member in members.iter_mut().filter(|member| member.id == id) {
                    member.present = false;
                }
            }
            "demo.slots.MatchInGroup" => return (literal(input, "group_id"), slots, members),
            _ => {}
        }
    }
    panic!("{scenario} sends MatchInGroup: {steps:#?}");
}

/// Whether `slot` holds a membership on `side` that `rule` counts.
fn holds(members: &[SentMember], slot: &Node, side: &str, rule: Rule) -> bool {
    let side = text(side);
    members.iter().any(|member| {
        (rule == Rule::DropsTie || member.slot == *slot)
            && (rule == Rule::DropsSide || member.side == side)
            && (rule == Rule::DropsState || member.present)
    })
}

/// Whether `rule` takes `slot` as a candidate the test admits.
fn admits(members: &[SentMember], slot: &Node, rule: Rule) -> bool {
    match rule {
        Rule::IgnoresTest => true,
        Rule::LeftOnly => holds(members, slot, "Left", Rule::Healthy),
        _ => holds(members, slot, "Left", rule) && holds(members, slot, "Right", rule),
    }
}

/// The open slot of `group` ranked first by `[score desc, slot_id]` among those `rule` admits.
fn head<'s>(
    slots: &'s [SentSlot],
    members: &[SentMember],
    group: &Node,
    rule: Rule,
) -> Option<&'s SentSlot> {
    slots
        .iter()
        .filter(|slot| slot.open && slot.group == *group && admits(members, &slot.id, rule))
        .min_by(|left, right| {
            right
                .score
                .cmp(&left.score)
                .then_with(|| left.id.cmp(&right.id))
        })
}

/// The `paired` scenario arranges, above the head the model picks, a slot of the group whose left
/// side fails the test and one whose right side does, each holding a `Present` membership on the
/// other side; and the `no-match` scenario leaves only such slots open. The healthy interpreter
/// passes every scenario of the suite.
#[test]
fn synthesis_arranges_a_higher_scored_candidate_that_fails_the_test() {
    let suite = suite_with(MODEL, &[PAIRED, NO_MATCH]);
    let (group, slots, members) = arranged_before_match(&suite, PAIRED);
    let healthy = head(&slots, &members, &group, Rule::Healthy).expect("a matchable head");
    for (failing, other) in [("Left", "Right"), ("Right", "Left")] {
        assert!(
            slots.iter().any(|slot| {
                slot.open
                    && slot.group == group
                    && slot.score > healthy.score
                    && !holds(&members, &slot.id, failing, Rule::Healthy)
                    && holds(&members, &slot.id, other, Rule::Healthy)
            }),
            "a slot ranked above the head {healthy:?} whose {failing} side is empty: \
             {slots:#?}\n{members:#?}"
        );
    }
    let (group, slots, members) = arranged_before_match(&suite, NO_MATCH);
    assert_eq!(
        head(&slots, &members, &group, Rule::Healthy).map(|slot| &slot.id),
        None,
        "{slots:#?}\n{members:#?}"
    );
    assert!(
        head(&slots, &members, &group, Rule::IgnoresTest).is_some(),
        "an open slot of the group that fails the test: {slots:#?}"
    );
    let statuses = statuses(&suite, &Interpreted::for_model(ir(MODEL)));
    assert_eq!(failed(&statuses), Vec::<&str>::new(), "{statuses:#?}");
}

/// A target deciding the test by a rule the model does not state: the healthy interpreter, whose
/// published slot is replaced by the head the rule picks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Rule {
    /// The model's own rule.
    Healthy,
    /// The best slot of the group, whatever its members.
    IgnoresTest,
    /// A slot with a present left member.
    LeftOnly,
    /// A member counts whatever its state.
    DropsState,
    /// A member counts whatever its side.
    DropsSide,
    /// A member counts whatever slot it is in.
    DropsTie,
}

struct Faulty {
    inner: Interpreted,
    rule: Rule,
    slots: std::cell::RefCell<Vec<SentSlot>>,
    members: std::cell::RefCell<Vec<SentMember>>,
}

impl Faulty {
    fn new(rule: Rule) -> Self {
        Self {
            inner: Interpreted::for_model(ir(MODEL)),
            rule,
            slots: std::cell::RefCell::new(Vec::new()),
            members: std::cell::RefCell::new(Vec::new()),
        }
    }
}

impl ConformanceTarget for Faulty {
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
            ("demo.slots.OpenSlot", "opened") => self.slots.borrow_mut().push(SentSlot {
                id: field("slot_id"),
                group: field("group_id"),
                score: field("score"),
                open: true,
            }),
            ("demo.slots.CloseSlot", "closed") => {
                for slot in self.slots.borrow_mut().iter_mut() {
                    if slot.id == field("slot_id") {
                        slot.open = false;
                    }
                }
            }
            ("demo.slots.Join", "joined") => self.members.borrow_mut().push(SentMember {
                id: field("membership_id"),
                slot: field("slot_id"),
                side: field("side"),
                present: true,
            }),
            ("demo.slots.Leave", "left") => {
                for member in self.members.borrow_mut().iter_mut() {
                    if member.id == field("membership_id") {
                        member.present = false;
                    }
                }
            }
            ("demo.slots.MatchInGroup", "paired") => {
                let slots = self.slots.borrow();
                let members = self.members.borrow();
                let picked = head(&slots, &members, &field("group_id"), self.rule);
                for event in &mut answer.direct_events {
                    if let Some(slot) = picked {
                        event.payload.insert("slot_id".to_owned(), slot.id.clone());
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

fn assert_faulty_fails(rule: Rule) {
    let suite = suite_with(MODEL, &[PAIRED]);
    let statuses = statuses(&suite, &Faulty::new(rule));
    assert!(
        failed(&statuses).contains(&PAIRED),
        "{rule:?}: {PAIRED} passed: {statuses:#?}"
    );
}

#[test]
fn the_healthy_rule_passes_every_scenario() {
    let suite = suite_with(MODEL, &[PAIRED]);
    let statuses = statuses(&suite, &Faulty::new(Rule::Healthy));
    assert_eq!(failed(&statuses), Vec::<&str>::new(), "{statuses:#?}");
}

#[test]
fn a_target_ignoring_the_test_fails_a_scenario() {
    assert_faulty_fails(Rule::IgnoresTest);
}

#[test]
fn a_target_testing_one_side_fails_a_scenario() {
    assert_faulty_fails(Rule::LeftOnly);
}

#[test]
fn a_target_dropping_the_state_conjunct_fails_a_scenario() {
    assert_faulty_fails(Rule::DropsState);
}

#[test]
fn a_target_dropping_the_side_conjunct_fails_a_scenario() {
    assert_faulty_fails(Rule::DropsSide);
}

#[test]
fn a_target_dropping_the_tie_fails_a_scenario() {
    assert_faulty_fails(Rule::DropsTie);
}

fn refusals(text: &str) -> (Vec<String>, String) {
    let synthesis = synthesized(text);
    let ids = synthesis
        .suite
        .scenarios
        .keys()
        .map(ToString::to_string)
        .collect();
    (ids, format!("{:?}", synthesis.refusals))
}

/// A membership whose slot no creating command writes from its input is a row no scenario ties to
/// a slot it arranged: the pairing is refused by name.
#[test]
fn an_untied_creation_is_a_named_gap() {
    let untied = edit(
        MODEL,
        "          slot_id: input.slot_id\n          side: input.side\n",
        "          side: input.side\n",
    );
    let (ids, refusals) = refusals(&untied);
    assert!(!ids.iter().any(|id| id == PAIRED), "{ids:#?}");
    assert!(
        refusals.contains(
            "the rows of `demo.slots.Membership` related to a candidate are not arranged: no \
             creating command writes `slot_id` from its input"
        ),
        "{refusals}"
    );
}

/// A slot opens with a score of at most five, and the arrangement ranks its head at five: no fresh
/// slot outranks it, so no failing candidate ranks above the head. That is named, and the pairing
/// stands on its other rows.
#[test]
fn an_unsteerable_first_key_is_a_named_gap_and_the_scenario_stands() {
    let capped = edit(
        &edit(
            MODEL,
            "errors:\n",
            "errors:\n  - {name: demo.slots.TooHigh, summary: A score above five., fields: []}\n",
        ),
        "    outcomes:
      - name: opened",
        "    outcomes:
      - name: too-high
        when: input.score > 5
        error: demo.slots.TooHigh
      - name: opened",
    );
    let (ids, refusals) = refusals(&capped);
    assert!(
        ids.iter().any(|id| id == PAIRED),
        "{ids:#?}\nrefusals: {refusals}"
    );
    assert!(
        refusals.contains("ranks above the head of `demo.slots.Slot`"),
        "{refusals}"
    );
}
