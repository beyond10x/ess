//! Synthesis witnesses a refusal scoped to some held states (beyond10x/ess#201) and a stored-field
//! guard that also reads the held state (beyond10x/ess#204), both `ess/18`.
//!
//! Each shape is synthesized, its scenarios are read for the arranged state, and the suite is run:
//! against a hand-written target that answers the model, which must pass every scenario of the
//! command under test, and against mutants of it, each of which must fail at least one. The
//! interpreted target does not evaluate a guard over the subject, and says so by name rather than
//! passing or failing.
use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::report::Status;
use ess_conformance::scenario::{CommandRef, ErrorRef, EventRef, OutcomeRef};
use ess_conformance::target::*;
use ess_conformance::{
    synthesize::Synthesis, AdmittedSuite, ConformanceScenario, Runner, ScenarioStep,
};
use ess_domain::{
    command::OutcomeName,
    spec::{RawSpecFile, Specification},
    system::Source,
};
use ess_primitives::node::Node;

const SHIP: &str = include_str!("fixtures/state-scoped-refusals.yaml");
const REPORT: &str = include_str!("fixtures/state-in-subject-predicate.yaml");

const LISTED: &str =
    "      - name: gone\n        when_subject_state: [Delivered, Cancelled]\n        error: demo.ship.Gone\n";
const SPLIT: &str = "      - name: gone-delivered\n        when_subject_state: Delivered\n        error: demo.ship.Gone\n      - name: gone-cancelled\n        when_subject_state: Cancelled\n        error: demo.ship.Gone\n";

const DELIVERED: &str = "demo.ship.Order/state/Delivered/refuses/demo.ship.ShipOrder";
const CANCELLED: &str = "demo.ship.Order/state/Cancelled/refuses/demo.ship.ShipOrder";

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("model.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn synthesis(text: &str) -> Synthesis {
    ess_conformance::synthesize::synthesize(&ir(text))
}

fn split() -> String {
    let out = SHIP.replace(LISTED, SPLIT);
    assert_ne!(out, SHIP);
    out
}

fn refusals_about(result: &Synthesis, command: &str) -> Vec<String> {
    result
        .refusals
        .iter()
        .filter(|refusal| format!("{refusal:?}").contains(command))
        .map(|refusal| format!("{}: {}", refusal.cause.code(), refusal.cause))
        .collect()
}

fn ids(result: &Synthesis) -> Vec<String> {
    result
        .suite
        .scenarios
        .keys()
        .map(ToString::to_string)
        .collect()
}

fn scenario<'a>(result: &'a Synthesis, id: &str) -> &'a ConformanceScenario {
    result
        .suite
        .scenarios
        .iter()
        .find(|(key, _)| key.to_string() == id)
        .map_or_else(
            || {
                panic!(
                    "no scenario {id}\n ids: {:#?}\n refusals: {:#?}",
                    ids(result),
                    result
                        .refusals
                        .iter()
                        .map(|refusal| format!("{refusal:?}"))
                        .collect::<Vec<_>>()
                )
            },
            |(_, scenario)| scenario,
        )
}

/// The short name of every command the scenario sends, in step order.
fn sent(scenario: &ConformanceScenario) -> Vec<String> {
    scenario
        .steps
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::ExecuteCommand { command, .. } => {
                Some(command.to_string().rsplit('.').next().unwrap().to_owned())
            }
            _ => None,
        })
        .collect()
}

fn requires_outcome(scenario: &ConformanceScenario, outcome: &str) -> bool {
    scenario.steps.iter().any(|step| {
        matches!(step, ScenarioStep::ExpectOutcome { outcome: expected }
            if expected.to_string() == outcome)
    })
}

fn requires_error(scenario: &ConformanceScenario, error: &str) -> bool {
    scenario.steps.iter().any(|step| {
        matches!(step, ScenarioStep::ExpectError { error: expected, .. }
            if expected.to_string() == error)
    })
}

/// Whether a view step before the last invocation of `command` requires the row in `state`.
fn observes_state_before(scenario: &ConformanceScenario, command: &str, state: &str) -> bool {
    let last = scenario
        .steps
        .iter()
        .rposition(|step| {
            matches!(step, ScenarioStep::ExecuteCommand { command: sent, .. }
                if sent.to_string() == command)
        })
        .expect("the command is sent");
    scenario.steps[..last].iter().any(|step| {
        matches!(
            step,
            ScenarioStep::ExpectView { .. } | ScenarioStep::EventuallyView { .. }
        ) && format!("{step:?}").contains(&format!("\"{state}\""))
    })
}

fn branch(command: &CommandRef, name: &str) -> OutcomeRef {
    OutcomeRef::new(command.clone(), OutcomeName::new(name).unwrap())
}

fn text(node: Option<&Node>) -> String {
    match node {
        Some(Node::Text(value)) => value.clone(),
        other => panic!("expected text, got {other:?}"),
    }
}

/// An in-memory store of rows keyed by identity, and the events published in this scenario.
#[derive(Default)]
struct Store {
    minted: Cell<u64>,
    rows: RefCell<BTreeMap<String, BTreeMap<String, Node>>>,
    published: RefCell<Vec<ObservedEvent>>,
}

impl Store {
    fn begin(&self) {
        self.rows.replace(BTreeMap::new());
        self.published.replace(Vec::new());
    }

    fn token(&self) -> ess_primitives::consistency::ConsistencyToken {
        self.minted.set(self.minted.get() + 1);
        ess_primitives::consistency::ConsistencyToken::new(format!("seq:{}", self.minted.get()))
            .unwrap()
    }

    fn mint(&self) -> String {
        self.minted.set(self.minted.get() + 1);
        format!("00000000-0000-4000-8000-{:012}", self.minted.get())
    }

    fn state(&self, id: &str) -> Option<String> {
        self.rows.borrow().get(id).map(|row| text(row.get("state")))
    }

    fn set(&self, id: &str, field: &str, value: &str) {
        self.rows
            .borrow_mut()
            .get_mut(id)
            .unwrap()
            .insert(field.to_owned(), Node::Text(value.to_owned()));
    }

    fn finish(&self, result: SemanticCommandResult) -> SemanticCommandResult {
        for event in &result.direct_events {
            self.published.borrow_mut().push(event.clone());
        }
        result.with_consistency(self.token())
    }

    fn events(&self, request: &EventObservationRequest) -> Vec<ObservedEvent> {
        self.published
            .borrow()
            .iter()
            .filter(|event| event.event == request.event)
            .cloned()
            .collect()
    }
}

fn event(name: &str, id: &str) -> ObservedEvent {
    ObservedEvent::new(name.parse::<EventRef>().unwrap()).with("order_id", Node::Text(id.into()))
}

fn gone(command: &CommandRef, name: &str) -> SemanticCommandResult {
    SemanticCommandResult::took(branch(command, name)).with_error(DeclaredErrorValue::new(
        "demo.ship.Gone".parse::<ErrorRef>().unwrap(),
    ))
}

// ---- #201 ------------------------------------------------------------------------------------

/// How the hand-written shipping service answers `ShipOrder`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Ship {
    /// Ships a placed order, accepts a shipped one unchanged, refuses a delivered or cancelled one.
    Correct,
    /// `Gone` for every state the move does not start from: the over-claim `wrong_state:` forced.
    GoneWhenShipped,
    /// Accepts a cancelled order as if it were shipped.
    AcceptsCancelled,
    /// Refuses a delivered order and cancels it anyway.
    RefusesWithEffect,
}

struct Shipping {
    mode: Ship,
    /// The refusal branch names, per state: one listed branch, or the issue's one per state.
    split: bool,
    store: Store,
}

impl Shipping {
    fn new(mode: Ship, split: bool) -> Self {
        Self {
            mode,
            split,
            store: Store::default(),
        }
    }

    fn refusal(&self, state: &str) -> &'static str {
        match (self.split, state) {
            (false, _) => "gone",
            (true, "Delivered") => "gone-delivered",
            (true, _) => "gone-cancelled",
        }
    }
}

impl ConformanceTarget for Shipping {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("shipping-fixture", "1"))
    }
    fn begin_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        self.store.begin();
        Ok(())
    }
    fn end_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        Ok(())
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        let command = request.command.clone();
        let store = &self.store;
        let result = if command.to_string() == "demo.ship.PlaceOrder" {
            let id = store.mint();
            store.rows.borrow_mut().insert(
                id.clone(),
                BTreeMap::from([
                    ("order_id".to_owned(), Node::Text(id.clone())),
                    ("state".to_owned(), Node::Text("Placed".to_owned())),
                ]),
            );
            SemanticCommandResult::took(branch(&command, "placed"))
                .emitting(event("demo.ship.OrderPlaced", &id))
        } else {
            let id = text(request.input.get("order_id"));
            let Some(state) = store.state(&id) else {
                return Ok(store.finish(SemanticCommandResult::undeclared()));
            };
            match (command.to_string().as_str(), state.as_str()) {
                ("demo.ship.ShipOrder", "Placed") => {
                    store.set(&id, "state", "Shipped");
                    SemanticCommandResult::took(branch(&command, "shipped"))
                        .emitting(event("demo.ship.OrderShipped", &id))
                }
                ("demo.ship.ShipOrder", "Shipped") if self.mode == Ship::GoneWhenShipped => {
                    gone(&command, self.refusal("Delivered"))
                }
                ("demo.ship.ShipOrder", "Shipped") => {
                    SemanticCommandResult::took(branch(&command, "already-shipped"))
                }
                ("demo.ship.ShipOrder", "Cancelled") if self.mode == Ship::AcceptsCancelled => {
                    SemanticCommandResult::took(branch(&command, "already-shipped"))
                }
                ("demo.ship.ShipOrder", held) => {
                    if self.mode == Ship::RefusesWithEffect && held == "Delivered" {
                        store.set(&id, "state", "Cancelled");
                    }
                    gone(&command, self.refusal(held))
                }
                ("demo.ship.DeliverOrder", "Shipped") => {
                    store.set(&id, "state", "Delivered");
                    SemanticCommandResult::took(branch(&command, "delivered"))
                        .emitting(event("demo.ship.OrderDelivered", &id))
                }
                ("demo.ship.CancelOrder", "Placed" | "Shipped") => {
                    store.set(&id, "state", "Cancelled");
                    SemanticCommandResult::took(branch(&command, "cancelled"))
                        .emitting(event("demo.ship.OrderCancelled", &id))
                }
                _ => SemanticCommandResult::undeclared(),
            }
        };
        Ok(store.finish(result))
    }
    fn query_view(&self, _: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        Ok(SemanticViewResult::of(
            self.store
                .rows
                .borrow()
                .values()
                .cloned()
                .collect::<Vec<_>>(),
        ))
    }
    fn observe_events(
        &self,
        request: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        Ok(self.store.events(&request))
    }
    fn configure_external_outcome(&self, _: ExternalOutcomeControl) -> Result<(), TargetError> {
        panic!("nothing here is externally decided")
    }
    fn redeliver_event(&self, _: RedeliveryRequest) -> Result<(), TargetError> {
        panic!("no bindings")
    }
}

/// Every scenario about `command` and its status against `target`.
fn statuses<T: ConformanceTarget>(
    result: &Synthesis,
    target: &T,
    command: &str,
) -> BTreeMap<String, String> {
    let admitted =
        AdmittedSuite::from_suite(&result.suite).unwrap_or_else(|error| panic!("{error}"));
    Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, target)
        .into_report()
        .scenarios
        .into_iter()
        .filter(|run| run.scenario.to_string().contains(command))
        .map(|run| {
            (
                run.scenario.to_string(),
                if run.status == Status::Passed {
                    "passed".to_owned()
                } else {
                    format!("{:?}: {:?}", run.status, run.checks)
                },
            )
        })
        .collect()
}

fn failed(statuses: &BTreeMap<String, String>) -> Vec<&String> {
    statuses
        .iter()
        .filter(|(_, status)| *status != "passed")
        .map(|(id, _)| id)
        .collect()
}

#[test]
fn issue_201_each_refused_state_is_witnessed_on_a_row_arranged_in_it() {
    for (text, gone) in [
        (SHIP.to_owned(), ["gone", "gone"]),
        (split(), ["gone-delivered", "gone-cancelled"]),
    ] {
        let result = synthesis(&text);
        assert_eq!(
            refusals_about(&result, "demo.ship.ShipOrder"),
            Vec::<String>::new()
        );
        for (id, state, branch, arranges) in [
            (DELIVERED, "Delivered", gone[0], "DeliverOrder"),
            (CANCELLED, "Cancelled", gone[1], "CancelOrder"),
        ] {
            let witness = scenario(&result, id);
            assert!(
                requires_outcome(witness, &format!("demo.ship.ShipOrder/{branch}")),
                "{id} requires `{branch}`: {witness:#?}"
            );
            assert!(requires_error(witness, "demo.ship.Gone"), "{id}");
            assert!(
                sent(witness).contains(&arranges.to_owned()),
                "{id} arranges the row with {arranges}: {:?}",
                sent(witness)
            );
            assert!(
                observes_state_before(witness, "demo.ship.ShipOrder", state),
                "{id} observes the row in {state} before it sends ShipOrder"
            );
        }
        // No refusal scenario claims the states a branch answers otherwise.
        for state in ["Placed", "Shipped"] {
            let id = format!("demo.ship.Order/state/{state}/refuses/demo.ship.ShipOrder");
            assert!(!ids(&result).contains(&id), "{id} is not written");
        }
        for branch in gone {
            scenario(&result, &format!("demo.ship.ShipOrder/outcome/{branch}"));
        }
        let shipped = scenario(&result, "demo.ship.ShipOrder/outcome/already-shipped");
        assert!(observes_state_before(
            shipped,
            "demo.ship.ShipOrder",
            "Shipped"
        ));
    }
}

#[test]
fn issue_201_a_target_answering_the_model_passes_every_ship_order_scenario() {
    for split_form in [false, true] {
        let text = if split_form { split() } else { SHIP.to_owned() };
        let result = synthesis(&text);
        let statuses = statuses(
            &result,
            &Shipping::new(Ship::Correct, split_form),
            "ShipOrder",
        );
        assert!(statuses.len() >= 5, "{statuses:#?}");
        assert!(failed(&statuses).is_empty(), "{statuses:#?}");
    }
}

#[test]
fn issue_201_each_mutant_fails_a_ship_order_scenario() {
    for split_form in [false, true] {
        let text = if split_form { split() } else { SHIP.to_owned() };
        let result = synthesis(&text);
        for mode in [
            Ship::GoneWhenShipped,
            Ship::AcceptsCancelled,
            Ship::RefusesWithEffect,
        ] {
            let statuses = statuses(&result, &Shipping::new(mode, split_form), "ShipOrder");
            assert!(
                !failed(&statuses).is_empty(),
                "{mode:?} (split: {split_form}) passes: {statuses:#?}"
            );
        }
    }
}

#[test]
fn issue_201_the_interpreted_target_executes_held_state_refusals_and_snapshots() {
    let model = ir(SHIP);
    let result = ess_conformance::synthesize::synthesize(&model);
    let target = ess_conformance::interpret::Interpreted::for_model(model);
    let admitted = AdmittedSuite::from_suite(&result.suite).unwrap();
    let report = Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &target)
        .into_report();
    for id in [DELIVERED, CANCELLED] {
        let run = report
            .scenarios
            .iter()
            .find(|run| run.scenario.to_string() == id)
            .unwrap_or_else(|| panic!("{id} is run"));
        assert_eq!(run.status, Status::Passed, "{id}: {:?}", run.checks);
        assert!(run
            .checks
            .iter()
            .all(|check| check.status == Status::Passed));
        assert_eq!(
            run.checks
                .iter()
                .filter(|check| check.about == "subject snapshot demo.ship.Orders")
                .count(),
            2,
            "{id}: before and after snapshots must both execute: {:?}",
            run.checks
        );
        assert!(run
            .checks
            .iter()
            .any(|check| check.about == "no direct events"));
    }
}

// ---- #204 ------------------------------------------------------------------------------------

const KEPT: &str = "demo.shop.ReportPending/outcome/kept-ready";
const PENDING: &str = "demo.shop.ReportPending/outcome/pending";

/// How the hand-written shop answers `ReportPending`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Report {
    /// Keeps a ready order carrying a hold note; moves every other one to pending.
    Correct,
    /// Keeps any order carrying a hold note, whatever its state.
    IgnoresState,
    /// Keeps any ready order, with or without a hold note.
    IgnoresNote,
}

struct Shop {
    mode: Report,
    store: Store,
}

impl ConformanceTarget for Shop {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("shop-fixture", "1"))
    }
    fn begin_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        self.store.begin();
        Ok(())
    }
    fn end_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        Ok(())
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        let command = request.command.clone();
        let store = &self.store;
        let changed = |id: &str| event("demo.shop.OrderChanged", id);
        let result = if command.to_string() == "demo.shop.PlaceOrder" {
            let id = store.mint();
            store.rows.borrow_mut().insert(
                id.clone(),
                BTreeMap::from([
                    ("order_id".to_owned(), Node::Text(id.clone())),
                    ("state".to_owned(), Node::Text("Draft".to_owned())),
                    (
                        "hold_note".to_owned(),
                        Node::Text(text(request.input.get("hold_note"))),
                    ),
                ]),
            );
            SemanticCommandResult::took(branch(&command, "placed"))
                .emitting(event("demo.shop.OrderPlaced", &id))
        } else {
            let id = text(request.input.get("order_id"));
            let Some(state) = store.state(&id) else {
                return Ok(store.finish(SemanticCommandResult::undeclared()));
            };
            let noted = store
                .rows
                .borrow()
                .get(&id)
                .is_some_and(|row| !text(row.get("hold_note")).is_empty());
            match command.to_string().as_str() {
                "demo.shop.MarkReady" => {
                    store.set(&id, "state", "Ready");
                    SemanticCommandResult::took(branch(&command, "readied")).emitting(changed(&id))
                }
                "demo.shop.ReportPending" => {
                    let ready = state == "Ready";
                    let kept = match self.mode {
                        Report::Correct => ready && noted,
                        Report::IgnoresState => noted,
                        Report::IgnoresNote => ready,
                    };
                    if kept {
                        SemanticCommandResult::took(branch(&command, "kept-ready"))
                            .emitting(changed(&id))
                    } else {
                        store.set(&id, "state", "Pending");
                        SemanticCommandResult::took(branch(&command, "pending"))
                            .emitting(changed(&id))
                    }
                }
                other => panic!("unexpected command {other}"),
            }
        };
        Ok(store.finish(result))
    }
    fn query_view(&self, _: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        Ok(SemanticViewResult::of(
            self.store
                .rows
                .borrow()
                .values()
                .cloned()
                .collect::<Vec<_>>(),
        ))
    }
    fn observe_events(
        &self,
        request: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        Ok(self.store.events(&request))
    }
    fn configure_external_outcome(&self, _: ExternalOutcomeControl) -> Result<(), TargetError> {
        panic!("nothing here is externally decided")
    }
    fn redeliver_event(&self, _: RedeliveryRequest) -> Result<(), TargetError> {
        panic!("no bindings")
    }
}

fn shop(mode: Report) -> Shop {
    Shop {
        mode,
        store: Store::default(),
    }
}

#[test]
fn issue_204_kept_ready_is_witnessed_on_a_ready_row_carrying_a_note() {
    let result = synthesis(REPORT);
    assert_eq!(
        refusals_about(&result, "demo.shop.ReportPending"),
        Vec::<String>::new()
    );
    let kept = scenario(&result, KEPT);
    assert!(
        sent(kept).contains(&"MarkReady".to_owned()),
        "the row is moved to Ready: {:?}",
        sent(kept)
    );
    assert!(
        observes_state_before(kept, "demo.shop.ReportPending", "Ready"),
        "the row is observed in Ready before ReportPending"
    );
    scenario(&result, PENDING);
}

#[test]
fn issue_204_a_target_answering_the_model_passes_every_report_pending_scenario() {
    let result = synthesis(REPORT);
    let statuses = statuses(&result, &shop(Report::Correct), "ReportPending");
    assert!(statuses.len() >= 2, "{statuses:#?}");
    assert!(failed(&statuses).is_empty(), "{statuses:#?}");
}

#[test]
fn issue_204_a_target_ignoring_either_half_of_the_guard_fails() {
    let result = synthesis(REPORT);
    for mode in [Report::IgnoresState, Report::IgnoresNote] {
        let statuses = statuses(&result, &shop(mode), "ReportPending");
        assert!(
            !failed(&statuses).is_empty(),
            "{mode:?} passes: {statuses:#?}"
        );
    }
}

/// [`REPORT`] with a terminal `Closed` no move of `ReportPending` starts from, answered by a
/// `wrong_state:` refusal — except a closed order carrying a hold note, which a guarded refusal
/// reading `state` answers first (the #192 ruling: guarded branches select before `wrong_state`).
fn closing() -> String {
    let edits = [
        ("      terminal: []\n", "      terminal: [Closed]\n"),
        (
            "      states: [Draft, Pending, Ready]\n",
            "      states: [Draft, Pending, Ready, Closed]\n",
        ),
        (
            "        - {name: back_to_pending, from: [Draft, Pending, Ready], to: Pending}\n",
            "        - {name: back_to_pending, from: [Draft, Pending, Ready], to: Pending}\n        - {name: close, from: [Draft, Pending, Ready], to: Closed}\n",
        ),
        (
            "demo.shop.ReportPending]\n",
            "demo.shop.ReportPending, demo.shop.CloseOrder]\nerrors:\n  - {name: demo.shop.Held, summary: A closed order is held., fields: []}\n  - {name: demo.shop.Gone, summary: The order is closed., fields: []}\n",
        ),
        (
            "      - name: pending\n",
            "      - name: closed-held\n        when_subject:\n          predicate:\n            all:\n              - state == Closed\n              - hold_note != \"\"\n        error: demo.shop.Held\n      - name: gone\n        wrong_state: true\n        error: demo.shop.Gone\n      - name: pending\n",
        ),
        (
            "views:\n",
            "  - name: demo.shop.CloseOrder\n    input: [{name: order_id, type: demo.shop.OrderId}]\n    outcomes:\n      - name: closed\n        moves: demo.shop.Order.close\n        instance: order_id\n        emits: [demo.shop.OrderChanged]\n        payload: {demo.shop.OrderChanged: {order_id: input.order_id}}\nviews:\n",
        ),
    ];
    let mut text = REPORT.to_owned();
    for (from, to) in edits {
        let next = text.replace(from, to);
        assert_ne!(next, text, "`{from}` is in the fixture");
        text = next;
    }
    text
}

const HELD: &str = "demo.shop.ReportPending/outcome/closed-held";
const CLOSED: &str = "demo.shop.Order/state/Closed/refuses/demo.shop.ReportPending";

#[test]
fn issue_204_a_state_reading_guard_selects_before_wrong_state_and_both_are_witnessed() {
    let result = synthesis(&closing());
    assert_eq!(
        refusals_about(&result, "demo.shop.ReportPending"),
        Vec::<String>::new()
    );
    let held = scenario(&result, HELD);
    assert!(
        sent(held).contains(&"CloseOrder".to_owned()),
        "{:?}",
        sent(held)
    );
    assert!(requires_error(held, "demo.shop.Held"));
    assert!(observes_state_before(
        held,
        "demo.shop.ReportPending",
        "Closed"
    ));
    let closed = scenario(&result, CLOSED);
    assert!(requires_error(closed, "demo.shop.Gone"));
    assert!(requires_outcome(closed, "demo.shop.ReportPending/gone"));
    assert!(
        requires_outcome(closed, "demo.shop.ReportPending/closed-held"),
        "the Closed state's scenario also witnesses the guard that answers a noted row there"
    );
}

/// The shop of [`closing`]: `Held` for a closed order carrying a note, `Gone` for any other closed
/// order — or, with `wrong_state_first`, `Gone` for every closed order.
struct ClosingShop {
    wrong_state_first: bool,
    inner: Shop,
}

impl ConformanceTarget for ClosingShop {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        self.inner.identity()
    }
    fn begin_scenario(&self, context: &ScenarioContext) -> Result<(), TargetError> {
        self.inner.begin_scenario(context)
    }
    fn end_scenario(&self, context: &ScenarioContext) -> Result<(), TargetError> {
        self.inner.end_scenario(context)
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        let command = request.command.clone();
        let store = &self.inner.store;
        let name = command.to_string();
        if name == "demo.shop.PlaceOrder" || name == "demo.shop.MarkReady" {
            return self.inner.execute_command(request);
        }
        let id = text(request.input.get("order_id"));
        // The wrong-state branch answers an identity no row carries, as the unknown-instance rule
        // has it (`docs/design/typed-literals-and-unknown-instances.md`).
        let state = store.state(&id).unwrap_or_default();
        let result = match (name.as_str(), state.as_str()) {
            ("demo.shop.ReportPending", "") => {
                SemanticCommandResult::took(branch(&command, "gone")).with_error(
                    DeclaredErrorValue::new("demo.shop.Gone".parse::<ErrorRef>().unwrap()),
                )
            }
            (_, "") | ("demo.shop.CloseOrder", "Closed") => SemanticCommandResult::undeclared(),
            ("demo.shop.CloseOrder", _) => {
                store.set(&id, "state", "Closed");
                SemanticCommandResult::took(branch(&command, "closed"))
                    .emitting(event("demo.shop.OrderChanged", &id))
            }
            ("demo.shop.ReportPending", "Closed") => {
                let noted = store
                    .rows
                    .borrow()
                    .get(&id)
                    .is_some_and(|row| !text(row.get("hold_note")).is_empty());
                let refuse = |branch_name: &str, error: &str| {
                    SemanticCommandResult::took(branch(&command, branch_name))
                        .with_error(DeclaredErrorValue::new(error.parse::<ErrorRef>().unwrap()))
                };
                if noted && !self.wrong_state_first {
                    refuse("closed-held", "demo.shop.Held")
                } else {
                    refuse("gone", "demo.shop.Gone")
                }
            }
            _ => return self.inner.execute_command(request),
        };
        Ok(store.finish(result))
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

#[test]
fn issue_204_the_guard_before_wrong_state_passes_a_faithful_target_and_fails_the_reverse_order() {
    let result = synthesis(&closing());
    let faithful = ClosingShop {
        wrong_state_first: false,
        inner: shop(Report::Correct),
    };
    let passed = statuses(&result, &faithful, "ReportPending");
    assert!(failed(&passed).is_empty(), "{passed:#?}");
    let reversed = ClosingShop {
        wrong_state_first: true,
        inner: shop(Report::Correct),
    };
    let reversed = statuses(&result, &reversed, "ReportPending");
    // `closed-held` is witnessed by its own scenario and, beside `gone`, by the Closed state's.
    assert_eq!(failed(&reversed), vec![CLOSED, HELD], "{reversed:#?}");
}

#[test]
fn issue_204_the_interpreted_target_names_the_stored_field_guard_it_does_not_evaluate() {
    let model = ir(REPORT);
    let result = ess_conformance::synthesize::synthesize(&model);
    let target = ess_conformance::interpret::Interpreted::for_model(model);
    let admitted = AdmittedSuite::from_suite(&result.suite).unwrap();
    let report = Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &target)
        .into_report();
    let run = report
        .scenarios
        .iter()
        .find(|run| run.scenario.to_string() == KEPT)
        .expect("kept-ready is run");
    assert_eq!(run.status, Status::Unsupported, "{:?}", run.checks);
}

/// Two refusals of one held state told apart by the input share the state's one refusal id, and
/// that scenario witnesses both, each on its own row.
#[test]
fn issue_201_refusals_of_one_state_split_by_the_input_are_both_witnessed_under_its_id() {
    let edits = [
        (
            "  - {name: demo.ship.OrderId, kind: newtype, of: Uuid}\n",
            "  - {name: demo.ship.OrderId, kind: newtype, of: Uuid}\n  - {name: demo.ship.Reason, kind: enum, variants: [Late, Lost]}\n",
        ),
        (
            "  - name: demo.ship.ShipOrder\n    input: [{name: order_id, type: demo.ship.OrderId}]\n",
            "  - name: demo.ship.ShipOrder\n    input: [{name: order_id, type: demo.ship.OrderId}, {name: reason, type: demo.ship.Reason}]\n",
        ),
        (
            LISTED,
            "      - name: gone\n        when_subject_state: [Delivered, Cancelled]\n        when: reason == Late\n        error: demo.ship.Gone\n      - name: gone-quietly\n        when_subject_state: [Delivered, Cancelled]\n        when: reason == Lost\n        error: demo.ship.Gone\n",
        ),
    ];
    let mut text = SHIP.to_owned();
    for (from, to) in edits {
        let next = text.replace(from, to);
        assert_ne!(next, text, "`{from}` is in the fixture");
        text = next;
    }
    let result = synthesis(&text);
    assert_eq!(
        refusals_about(&result, "demo.ship.ShipOrder"),
        Vec::<String>::new()
    );
    for id in [DELIVERED, CANCELLED] {
        let witness = scenario(&result, id);
        for branch in ["gone", "gone-quietly"] {
            assert!(
                requires_outcome(witness, &format!("demo.ship.ShipOrder/{branch}")),
                "{id} witnesses `{branch}`"
            );
        }
        let rows = sent(witness)
            .iter()
            .filter(|sent| *sent == "PlaceOrder")
            .count();
        assert_eq!(rows, 2, "{id}: one row per refusal: {:?}", sent(witness));
    }
}
