//! The head of an ordered selection on the native interpreter (`docs/design/ordered-related-head.md`,
//! "Interpreter and history checker", "Decisive acceptance"; beyond10x/ess#543).
//!
//! `tests/fixtures/ordered-related-head.yaml`: `Join` adds a scored membership to a slot; `Pick`
//! refuses with `Empty` while no membership of the slot is `Present`, and otherwise pairs the one
//! ranked first by `[element_score desc, membership_id]`, publishing and storing its key.
//!
//! The rows are arranged so that the head is neither the first nor the last inserted, nor the head
//! under the first key's reversed direction, and a tie on the score is broken by the identity
//! against the insertion order. Synthesis arranges a witness of the head that tells it from each of
//! those, and a target choosing by any of them fails it.
use std::collections::BTreeMap;

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::interpret::execute::{execute, Externals, Step, Store, Undetermined};
use ess_conformance::interpret::Interpreted;
use ess_conformance::target::*;
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::ids::CorrelationId;
use ess_primitives::node::Node;

const MODEL: &str = include_str!("fixtures/ordered-related-head.yaml");

const S1: &str = "00000000-0000-4000-8000-000000000001";
const S2: &str = "00000000-0000-4000-8000-000000000002";
const S3: &str = "00000000-0000-4000-8000-000000000003";
const OTHER_SLOT: &str = "00000000-0000-4000-8000-000000000004";
const REMOVED: &str = "00000000-0000-4000-8000-000000000005";
const TIE_LATER_ID: &str = "00000000-0000-4000-8000-000000000009";
const TIE_EARLIER_ID: &str = "00000000-0000-4000-8000-000000000008";
const PAIR: &str = "00000000-0000-4000-8000-0000000000aa";

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}"));
    let spec = Specification::assemble([(Source::new("ranked.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("the fixture validates:\n{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn edit(text: &str, from: &str, to: &str) -> String {
    assert!(text.contains(from), "fixture edit did not land: {from}");
    text.replacen(from, to, 1)
}

/// The fixture with `Pick`'s `exists: false` refusal taken away: the head read alone.
fn unguarded() -> String {
    edit(
        MODEL,
        "      - name: empty
        when_related:
          entity: demo.ranked.Membership
          where: {all: [slot_id == input.slot_id, state == Present]}
          exists: false
        error: demo.ranked.Empty
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
    CorrelationId::new("ordered-head").unwrap()
}

fn target(text: &str) -> Interpreted {
    let target = Interpreted::for_model(ir(text));
    target
        .begin_scenario(&ScenarioContext::new(
            "demo.ranked/authored/ordered-head".parse().unwrap(),
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

fn join(target: &dyn ConformanceTarget, id: &str, slot: &str, key: &str, score: i64) {
    let answer = target
        .execute_command(request(
            "demo.ranked.Join",
            &[
                ("membership_id", text(id)),
                ("slot_id", text(slot)),
                ("element_key", text(key)),
                ("element_score", number(score)),
            ],
        ))
        .unwrap_or_else(|error| panic!("a membership joins: {error}"));
    assert_eq!(outcome(&answer), "joined");
}

fn pick(target: &dyn ConformanceTarget, slot: &str) -> SemanticCommandResult {
    target
        .execute_command(request(
            "demo.ranked.Pick",
            &[("pair_id", text(PAIR)), ("slot_id", text(slot))],
        ))
        .unwrap_or_else(|error| panic!("answered: {error}"))
}

fn pairs(target: &dyn ConformanceTarget) -> Vec<BTreeMap<String, Node>> {
    target
        .query_view(SemanticViewRequest {
            view: "demo.ranked.Pairs".parse().unwrap(),
            params: BTreeMap::new(),
            consistency: ess_primitives::consistency::QueryConsistency::Current,
            correlation: correlation(),
            deadline: Deadline::at(ess_primitives::time::Timestamp::from_epoch_millis(0)),
        })
        .expect("the view answers")
        .rows
}

/// S1, S2 and S3 in slot `a`, inserted in that order with scores 20, 30 and 10, so S2 is the head;
/// a higher score in another slot, and a higher score in this slot that has left.
fn arranged(target: &dyn ConformanceTarget) {
    join(target, S1, "a", "k-first", 20);
    join(target, OTHER_SLOT, "b", "k-other-slot", 99);
    join(target, REMOVED, "a", "k-removed", 50);
    join(target, S2, "a", "k-head", 30);
    join(target, S3, "a", "k-last", 10);
    let left = target
        .execute_command(request(
            "demo.ranked.Leave",
            &[("membership_id", text(REMOVED))],
        ))
        .unwrap_or_else(|error| panic!("left: {error}"));
    assert_eq!(outcome(&left), "left");
}

#[test]
fn the_interpreter_publishes_the_ranked_head() {
    let target = target(MODEL);
    arranged(&target);
    let picked = pick(&target, "a");
    assert_eq!(outcome(&picked), "picked");
    let event = &picked.direct_events[0].payload;
    assert_eq!(event["element_key"], text("k-head"), "{event:?}");
    assert_eq!(event["membership_id"], text(S2), "{event:?}");
    let rows = pairs(&target);
    assert_eq!(rows.len(), 1, "{rows:?}");
    assert_eq!(rows[0]["element_key"], text("k-head"), "{rows:?}");
}

/// A tie on the score is broken by the identity, ascending, whatever the insertion order.
#[test]
fn a_tie_is_broken_by_the_identity_not_the_insertion_order() {
    let target = target(MODEL);
    join(&target, TIE_LATER_ID, "t", "k-later-id", 30);
    join(&target, S3, "t", "k-lower-score", 10);
    join(&target, TIE_EARLIER_ID, "t", "k-earlier-id", 30);
    let picked = pick(&target, "t");
    assert_eq!(outcome(&picked), "picked");
    let event = &picked.direct_events[0].payload;
    assert_eq!(event["element_key"], text("k-earlier-id"), "{event:?}");
    assert_eq!(event["membership_id"], text(TIE_EARLIER_ID), "{event:?}");
}

#[test]
fn an_empty_selection_publishes_nothing() {
    // The model's own answer for an empty slot is its refusal.
    let target = target(MODEL);
    join(&target, OTHER_SLOT, "b", "k-other-slot", 99);
    let refused = pick(&target, "a");
    assert_eq!(outcome(&refused), "empty");
    assert!(refused.direct_events.is_empty(), "{refused:?}");
    assert_eq!(pairs(&target), Vec::new(), "no pair is stored");

    // Without it, the head read alone supplies no value: no outcome is published and no row stored.
    let target = self::target(&unguarded());
    join(&target, OTHER_SLOT, "b", "k-other-slot", 99);
    let answered = target.execute_command(request(
        "demo.ranked.Pick",
        &[("pair_id", text(PAIR)), ("slot_id", text("a"))],
    ));
    assert!(
        answered
            .as_ref()
            .map_or(true, |result| result.outcome.is_none()),
        "an empty selection supplies no value: {answered:?}"
    );
    assert_eq!(pairs(&target), Vec::new(), "no pair is stored");
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
        &format!("demo.ranked.{command}").parse().unwrap(),
        &input,
        &Externals::Withheld,
    )?;
    assert_eq!(steps.len(), 1);
    let step = steps.remove(0);
    *store = step.next.clone();
    Ok(step)
}

/// A score the store does not hold leaves the head undecided between two members: the history is
/// not guessed, and the store is unchanged.
#[test]
fn an_abstract_key_leaves_the_head_undecidable() {
    let model = ir(&edit(
        MODEL,
        "          element_score: input.element_score\n",
        "",
    ));
    let mut store = Store::default();
    for (id, key) in [(S1, "k-first"), (S2, "k-second")] {
        run(
            &model,
            &mut store,
            "Join",
            &[
                ("membership_id", text(id)),
                ("slot_id", text("a")),
                ("element_key", text(key)),
                ("element_score", number(1)),
            ],
        )
        .unwrap_or_else(|error| panic!("joined: {error:?}"));
    }
    let before = store.clone();
    let answered = run(
        &model,
        &mut store,
        "Pick",
        &[("pair_id", text(PAIR)), ("slot_id", text("a"))],
    );
    assert!(
        matches!(answered, Err(Undetermined::Undecidable { .. })),
        "{answered:?}"
    );
    assert_eq!(store, before);
}

// ---- synthesis (unit U4) --------------------------------------------------------------------------

const PICKED: &str = "demo.ranked.Pick/outcome/picked";

fn synthesized(text: &str) -> ess_conformance::synthesize::Synthesis {
    ess_conformance::synthesize::synthesize(&ir(text))
}

fn suite_with_picked(text: &str) -> ess_conformance::ConformanceSuite {
    let synthesis = synthesized(text);
    let ids: Vec<String> = synthesis
        .suite
        .scenarios
        .keys()
        .map(ToString::to_string)
        .collect();
    assert!(
        ids.iter().any(|id| id == PICKED),
        "{PICKED} is synthesized: {ids:#?}\nrefusals: {:#?}",
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

/// Every `Join` the `picked` scenario sends before `Pick`, as (slot, key, score), in order.
fn joins_before_pick(suite: &ess_conformance::ConformanceSuite) -> (String, Vec<(String, Node)>) {
    let steps = &suite
        .scenarios
        .iter()
        .find(|(id, _)| id.to_string() == PICKED)
        .expect("the picked scenario")
        .1
        .steps;
    let literal = |input: &BTreeMap<String, ess_conformance::ScenarioValue>, name: &str| {
        input
            .get(name)
            .and_then(|value| value.as_literal())
            .cloned()
            .unwrap_or(Node::Null)
    };
    let mut joins = Vec::new();
    for step in steps {
        if let ess_conformance::ScenarioStep::ExecuteCommand { command, input, .. } = step {
            match command.to_string().as_str() {
                "demo.ranked.Join" => joins.push((
                    literal(input, "slot_id")
                        .as_text()
                        .unwrap_or_default()
                        .to_owned(),
                    literal(input, "element_score"),
                )),
                "demo.ranked.Pick" => {
                    let slot = literal(input, "slot_id")
                        .as_text()
                        .unwrap_or_default()
                        .to_owned();
                    return (slot, joins);
                }
                _ => {}
            }
        }
    }
    panic!("the picked scenario sends Pick: {steps:#?}");
}

/// A target choosing the head by a rule the model does not state: the healthy interpreter, whose
/// published head is replaced by the one the rule picks among the memberships present in the slot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WrongHead {
    /// The first membership the slot received.
    FirstInserted,
    /// The last membership the slot received.
    LastInserted,
    /// The first by `[element_score asc, membership_id]`: the first key's direction reversed.
    ReversedFirstKey,
    /// By `element_score desc`, ties broken by insertion order rather than by the identity.
    TiesByInsertion,
    /// By `[element_score desc, membership_id desc]`: the tie-break's direction reversed.
    TieBreakReversed,
}

struct Member {
    id: Node,
    slot: Node,
    key: Node,
    score: Node,
    present: bool,
}

struct Wrong {
    inner: Interpreted,
    rule: WrongHead,
    joined: std::cell::RefCell<Vec<Member>>,
}

impl Wrong {
    fn new(rule: WrongHead) -> Self {
        Self::with_model(rule, MODEL)
    }

    fn with_model(rule: WrongHead, model: &str) -> Self {
        Self {
            inner: Interpreted::for_model(ir(model)),
            rule,
            joined: std::cell::RefCell::new(Vec::new()),
        }
    }

    fn head(&self, slot: &Node) -> Option<(Node, Node)> {
        let joined = self.joined.borrow();
        let present: Vec<&Member> = joined
            .iter()
            .filter(|member| member.present && member.slot == *slot)
            .collect();
        let chosen = match self.rule {
            WrongHead::FirstInserted => present.first().copied(),
            WrongHead::LastInserted => present.last().copied(),
            WrongHead::ReversedFirstKey => present
                .iter()
                .copied()
                .min_by(|left, right| (&left.score, &left.id).cmp(&(&right.score, &right.id))),
            WrongHead::TiesByInsertion => present
                .iter()
                .copied()
                .rev()
                .max_by(|left, right| left.score.cmp(&right.score)),
            WrongHead::TieBreakReversed => present
                .iter()
                .copied()
                .max_by(|left, right| (&left.score, &left.id).cmp(&(&right.score, &right.id))),
        }?;
        Some((chosen.key.clone(), chosen.id.clone()))
    }
}

impl ConformanceTarget for Wrong {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        self.inner.identity()
    }
    fn begin_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.joined.borrow_mut().clear();
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
            ("demo.ranked.Join", "joined") => self.joined.borrow_mut().push(Member {
                id: field("membership_id"),
                slot: field("slot_id"),
                key: field("element_key"),
                score: field("element_score"),
                present: true,
            }),
            ("demo.ranked.Leave", "left") => {
                for member in self.joined.borrow_mut().iter_mut() {
                    if member.id == field("membership_id") {
                        member.present = false;
                    }
                }
            }
            ("demo.ranked.Pick", "picked") => {
                if let Some((key, id)) = self.head(&field("slot_id")) {
                    for event in &mut answer.direct_events {
                        event.payload.insert("element_key".to_owned(), key.clone());
                        event.payload.insert("membership_id".to_owned(), id.clone());
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

/// The `picked` scenario arranges at least three memberships in the slot it picks from, and the
/// head it expects is neither the first nor the last of them; the healthy interpreter passes every
/// scenario of the command.
#[test]
fn synthesis_arranges_a_witness_whose_head_is_not_the_first_inserted_row() {
    let suite = suite_with_picked(MODEL);
    let (slot, joins) = joins_before_pick(&suite);
    let in_slot: Vec<&Node> = joins
        .iter()
        .filter(|(held, _)| *held == slot)
        .map(|(_, score)| score)
        .collect();
    assert!(
        in_slot.len() >= 3,
        "three memberships in the slot: {joins:#?}"
    );
    let best = in_slot.iter().copied().max().expect("a score");
    assert_ne!(in_slot.first().copied(), Some(best), "{joins:#?}");
    assert_ne!(in_slot.last().copied(), Some(best), "{joins:#?}");
    let statuses = statuses(&suite, &Interpreted::for_model(ir(MODEL)));
    assert_eq!(failed(&statuses), Vec::<&str>::new(), "{statuses:#?}");
}

fn assert_wrong_head_fails(rule: WrongHead) {
    let suite = suite_with_picked(MODEL);
    let statuses = statuses(&suite, &Wrong::new(rule));
    assert!(
        failed(&statuses).contains(&PICKED),
        "{rule:?}: {PICKED} passed: {statuses:#?}"
    );
}

#[test]
fn a_target_taking_the_first_inserted_row_fails_a_scenario() {
    assert_wrong_head_fails(WrongHead::FirstInserted);
}

#[test]
fn a_target_taking_the_last_inserted_row_fails_a_scenario() {
    assert_wrong_head_fails(WrongHead::LastInserted);
}

#[test]
fn a_target_reversing_the_first_key_fails_a_scenario() {
    assert_wrong_head_fails(WrongHead::ReversedFirstKey);
}

#[test]
fn a_target_breaking_ties_by_insertion_order_fails_a_scenario() {
    assert_wrong_head_fails(WrongHead::TiesByInsertion);
}

#[test]
fn a_target_reversing_the_tie_break_fails_a_scenario() {
    assert_wrong_head_fails(WrongHead::TieBreakReversed);
}

/// A branch whose guard admits at most one selected row never observes its order: synthesis says
/// so by name rather than witnessing the head on one row.
#[test]
fn a_head_over_at_most_one_row_is_a_named_gap() {
    let text = edit(
        MODEL,
        "      - name: picked\n",
        "      - name: crowded
        when_related:
          entity: demo.ranked.Membership
          where: {all: [slot_id == input.slot_id, state == Present]}
          count: {gt: 1}
        error: demo.ranked.Crowded
      - name: picked\n",
    );
    let text = edit(
        &text,
        "  - {name: demo.ranked.Empty,",
        "  - {name: demo.ranked.Crowded, summary: More than one element waits., fields: []}\n  - {name: demo.ranked.Empty,",
    );
    let synthesis = synthesized(&text);
    let ids: Vec<String> = synthesis
        .suite
        .scenarios
        .keys()
        .map(ToString::to_string)
        .collect();
    assert!(!ids.iter().any(|id| id == PICKED), "{ids:#?}");
    let refusals = format!("{:?}", synthesis.refusals);
    assert!(
        refusals.contains("the ranked head is not observable"),
        "{refusals}"
    );
}

// ---- struct identities and declared-unordered ties (U7) -----------------------------------------

/// The fixture with the membership identified by a struct whose members are declared `seq` first,
/// then `lane`: the order of their names is the other way round.
fn struct_identity() -> String {
    edit(
        MODEL,
        "  - {name: demo.ranked.MembershipId, kind: newtype, of: Uuid}\n",
        "  - name: demo.ranked.MembershipId\n    kind: struct\n    fields:\n      - {name: seq, type: Integer}\n      - {name: lane, type: String}\n",
    )
}

fn key(seq: i64, lane: &str) -> Node {
    Node::Map(BTreeMap::from([
        ("seq".to_owned(), number(seq)),
        ("lane".to_owned(), text(lane)),
    ]))
}

fn join_node(target: &dyn ConformanceTarget, id: Node, slot: &str, key: &str, score: i64) {
    let answer = target
        .execute_command(request(
            "demo.ranked.Join",
            &[
                ("membership_id", id),
                ("slot_id", text(slot)),
                ("element_key", text(key)),
                ("element_score", number(score)),
            ],
        ))
        .unwrap_or_else(|error| panic!("a membership joins: {error}"));
    assert_eq!(outcome(&answer), "joined");
}

/// A struct identity breaks a tie by its members in declaration order, each by its own type's
/// order: `seq` 1 ranks before `seq` 2 whatever the lanes, and `seq` 9 before `seq` 10, where the
/// names' order (`lane` first) and the text of the numbers would each answer the other row.
#[test]
fn a_struct_identity_breaks_a_tie_by_its_members_in_declaration_order() {
    let target = target(&struct_identity());
    join_node(&target, key(2, "a"), "t", "k-seq-2", 30);
    join_node(&target, key(1, "z"), "t", "k-seq-1", 30);
    join_node(&target, key(0, "a"), "t", "k-lower-score", 10);
    let picked = pick(&target, "t");
    assert_eq!(outcome(&picked), "picked", "{picked:?}");
    let event = &picked.direct_events[0].payload;
    assert_eq!(event["element_key"], text("k-seq-1"), "{event:?}");
    assert_eq!(event["membership_id"], key(1, "z"), "{event:?}");

    let target = self::target(&struct_identity());
    join_node(&target, key(10, "a"), "t", "k-seq-10", 30);
    join_node(&target, key(9, "a"), "t", "k-seq-9", 30);
    let picked = pick(&target, "t");
    let event = &picked.direct_events[0].payload;
    assert_eq!(event["element_key"], text("k-seq-9"), "{event:?}");
}

/// The fixture's three head reads ranked by `[element_score desc]` alone, with `ties: unordered`.
fn ties_unordered() -> String {
    // The `sets:` entry sits two columns to the left of the payload's.
    let text = MODEL
        .replace(
            "\n              order_by: [element_score desc, membership_id]\n",
            "\n              order_by: [element_score desc]\n              ties: unordered\n",
        )
        .replace(
            "\n                order_by: [element_score desc, membership_id]\n",
            "\n                order_by: [element_score desc]\n                ties: unordered\n",
        );
    assert_eq!(text.matches("ties: unordered").count(), 3, "{text}");
    text
}

/// Two selected rows that tie on every key of an order declaring `ties: unordered` leave the head
/// to the implementation: the interpreter answers it unsupported, and picks no row. A tie behind a
/// head that ranks first alone decides nothing, and the head is read.
#[test]
fn the_interpreter_answers_a_declared_unordered_tie_unsupported() {
    let model = ir(&ties_unordered());
    let mut store = Store::default();
    for (id, key, score) in [(S1, "k-first", 30), (S2, "k-second", 30), (S3, "k-low", 10)] {
        run(
            &model,
            &mut store,
            "Join",
            &[
                ("membership_id", text(id)),
                ("slot_id", text("a")),
                ("element_key", text(key)),
                ("element_score", number(score)),
            ],
        )
        .unwrap_or_else(|error| panic!("joined: {error:?}"));
    }
    let before = store.clone();
    let answered = run(
        &model,
        &mut store,
        "Pick",
        &[("pair_id", text(PAIR)), ("slot_id", text("a"))],
    );
    let Err(undetermined) = &answered else {
        panic!("a tie at the head picks no row: {answered:?}");
    };
    assert!(undetermined.is_capability_gap(), "{undetermined:?}");
    let message = undetermined.to_string();
    assert!(message.contains("ties: unordered"), "{message}");
    assert!(message.contains("element_score desc"), "{message}");
    assert_eq!(store, before);

    // Through the target the answer is `unsupported`.
    let target = target(&ties_unordered());
    join(&target, S1, "a", "k-first", 30);
    join(&target, S2, "a", "k-second", 30);
    let answered = target.execute_command(request(
        "demo.ranked.Pick",
        &[("pair_id", text(PAIR)), ("slot_id", text("a"))],
    ));
    assert!(
        matches!(&answered, Err(error @ TargetError::Unsupported { .. }) if error.to_string().contains("ties: unordered")),
        "{answered:?}"
    );

    // A head that ranks first alone is read, whatever ties behind it.
    let target = self::target(&ties_unordered());
    join(&target, S1, "a", "k-low-1", 10);
    join(&target, S2, "a", "k-head", 30);
    join(&target, S3, "a", "k-low-2", 10);
    let picked = pick(&target, "a");
    assert_eq!(outcome(&picked), "picked", "{picked:?}");
    assert_eq!(
        picked.direct_events[0].payload["element_key"],
        text("k-head")
    );
}

/// Under `ties: unordered` synthesis arranges no two selected rows with equal keys, and its witness
/// still has a head that is neither the first nor the last inserted: a target breaking ties by
/// insertion order conforms, and one taking the first inserted row does not.
#[test]
fn synthesis_arranges_no_tie_under_ties_unordered() {
    let model = ties_unordered();
    let suite = suite_with_picked(&model);
    let (slot, joins) = joins_before_pick(&suite);
    let in_slot: Vec<&Node> = joins
        .iter()
        .filter(|(held, _)| *held == slot)
        .map(|(_, score)| score)
        .collect();
    assert!(
        in_slot.len() >= 3,
        "three memberships in the slot: {joins:#?}"
    );
    let distinct: std::collections::BTreeSet<&Node> = in_slot.iter().copied().collect();
    assert_eq!(distinct.len(), in_slot.len(), "no two tie: {joins:#?}");
    let best = in_slot.iter().copied().max().expect("a score");
    assert_ne!(in_slot.first().copied(), Some(best), "{joins:#?}");
    assert_ne!(in_slot.last().copied(), Some(best), "{joins:#?}");

    let healthy = statuses(&suite, &Interpreted::for_model(ir(&model)));
    assert_eq!(failed(&healthy), Vec::<&str>::new(), "{healthy:#?}");
    let by_insertion = statuses(
        &suite,
        &Wrong::with_model(WrongHead::TiesByInsertion, &model),
    );
    assert!(
        !failed(&by_insertion).contains(&PICKED),
        "{by_insertion:#?}"
    );
    let first_inserted = statuses(&suite, &Wrong::with_model(WrongHead::FirstInserted, &model));
    assert!(
        failed(&first_inserted).contains(&PICKED),
        "{first_inserted:#?}"
    );
}

/// An authored `demo.ranked` scenario: `arrange` and the timeline's `acts`.
fn authored(arrange: &str, acts: &str) -> String {
    format!(
        "type: ess-scenario/2\ndomain: demo.ranked\nscenario: a-pick\n\
         summary: What this scenario proves, in one line.\n{arrange}timeline:\n{acts}"
    )
}

fn join_act(at: u32, id: &str, score: i64) -> String {
    format!(
        "  - at: 2026-01-05T09:00:{at:02}Z\n    command: demo.ranked.Join\n    \
         input: {{membership_id: {id}, slot_id: a, element_key: k-{at}, element_score: {score}}}\n    \
         outcome: joined\n"
    )
}

fn pick_act(at: u32) -> String {
    format!(
        "  - at: 2026-01-05T09:00:{at:02}Z\n    command: demo.ranked.Pick\n    \
         input: {{pair_id: {PAIR}, slot_id: a}}\n    outcome: picked\n"
    )
}

fn author(model: &str, text: &str) -> ess_conformance::authored::Authoring {
    ess_conformance::authored::compile(
        &ir(model),
        &[ess_conformance::authored::Source::new("a-pick.yaml", text)],
    )
}

/// `ess verify conform author` refuses an authored scenario whose rows tie, on every key of an order
/// declaring `ties: unordered`, for the head an act reads: the head it claims is the implementation's
/// choice. A head ranking first alone, and a total order, are accepted.
#[test]
fn author_refuses_a_scenario_arranging_a_tie_the_order_leaves_unordered() {
    let model = ties_unordered();
    let tied = authored(
        "",
        &[
            join_act(1, S1, 30),
            join_act(2, S2, 30),
            join_act(3, S3, 10),
            pick_act(4),
        ]
        .concat(),
    );
    let compiled = author(&model, &tied);
    assert!(compiled.scenarios.is_empty(), "refused");
    let refusals: Vec<(String, String)> = compiled
        .refusals
        .iter()
        .map(|refusal| (refusal.code().to_string(), refusal.to_string()))
        .collect();
    assert_eq!(refusals.len(), 1, "{refusals:#?}");
    assert_eq!(refusals[0].0, "ESS-AUTHOR-043", "{refusals:#?}");
    for named in [
        "demo.ranked.Pick",
        "demo.ranked.Membership",
        "element_score desc",
        "ties: unordered",
    ] {
        assert!(
            refusals[0].1.contains(named),
            "names {named}: {}",
            refusals[0].1
        );
    }

    // The same tie arranged with `setup:`.
    let setup = |id: &str, score: i64| {
        format!(
            "  - instance: m-{score}-{}\n    entity: demo.ranked.Membership\n    setup:\n      \
             identity: {id}\n      fields: {{slot_id: a, element_key: k-{id}, element_score: {score}}}\n      \
             state: Present\n",
            &id[id.len() - 1..]
        )
    };
    let arranged = authored(
        &format!("arrange:\n{}{}", setup(S1, 30), setup(S2, 30)),
        &pick_act(1),
    );
    let compiled = author(&model, &arranged);
    let codes: Vec<String> = compiled
        .refusals
        .iter()
        .map(|it| it.code().to_string())
        .collect();
    assert_eq!(
        codes,
        vec!["ESS-AUTHOR-043".to_owned()],
        "{:#?}",
        compiled.refusals
    );

    // A head ranking first alone, whatever ties behind it, is accepted.
    let alone = authored(
        "",
        &[
            join_act(1, S1, 10),
            join_act(2, S2, 30),
            join_act(3, S3, 10),
            pick_act(4),
        ]
        .concat(),
    );
    let compiled = author(&model, &alone);
    assert!(compiled.is_complete(), "{:#?}", compiled.refusals);

    // Under a total order the identity breaks the tie, and the scenario is accepted.
    let compiled = author(MODEL, &tied);
    assert!(compiled.is_complete(), "{:#?}", compiled.refusals);
}
