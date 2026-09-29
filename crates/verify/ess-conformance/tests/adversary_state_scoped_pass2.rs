//! Adversary pass 2 on the state-scoped unit (beyond10x/ess#201, #204), `ess/18`.
//!
//! The correction skips a wrong state from the wrong-state family when a `state`-reading guard is
//! true there with `state` bound alone, and witnesses every refusal claiming one held state under
//! that state's one id. These cases ask whether the skipped state is then witnessed by anything,
//! whether guards that together cover a wrong state still synthesize, and whether a target that
//! answers the wrong branch among several refusals claiming one state fails.
use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;

use ess_compiler::source::SourceMap;
use ess_compiler::{ir::EssIr, resolve::compile};
use ess_conformance::report::Status;
use ess_conformance::scenario::{CommandRef, ErrorRef, EventRef, OutcomeRef};
use ess_conformance::target::*;
use ess_conformance::{synthesize::Synthesis, AdmittedSuite, Runner};
use ess_domain::{
    command::OutcomeName,
    spec::{RawSpecFile, Specification},
    system::Source,
};
use ess_primitives::node::Node;

const SHIP: &str = include_str!("fixtures/state-scoped-refusals.yaml");

const ACCEPT_AND_GONE: &str = "      - name: already-shipped\n        when_subject_state: Shipped\n        preserves: demo.ship.Order\n        instance: order_id\n      - name: gone\n        when_subject_state: [Delivered, Cancelled]\n        error: demo.ship.Gone\n";

const REASON_TYPE: (&str, &str) = (
    "  - {name: demo.ship.OrderId, kind: newtype, of: Uuid}\n",
    "  - {name: demo.ship.OrderId, kind: newtype, of: Uuid}\n  - {name: demo.ship.Reason, kind: enum, variants: [Late, Lost]}\n",
);
const REASON_INPUT: (&str, &str) = (
    "  - name: demo.ship.ShipOrder\n    input: [{name: order_id, type: demo.ship.OrderId}]\n",
    "  - name: demo.ship.ShipOrder\n    input: [{name: order_id, type: demo.ship.OrderId}, {name: reason, type: demo.ship.Reason}]\n",
);

fn edited(edits: &[(&str, &str)]) -> String {
    let mut text = SHIP.to_owned();
    for (from, to) in edits {
        let next = text.replace(from, to);
        assert_ne!(next, text, "`{from}` is in the fixture");
        text = next;
    }
    text
}

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("model.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn synthesis(text: &str) -> Synthesis {
    ess_conformance::synthesize::synthesize(&ir(text))
}

fn refusals_about(result: &Synthesis, command: &str) -> Vec<String> {
    result
        .refusals
        .iter()
        .filter(|refusal| format!("{refusal:?}").contains(command))
        .map(|refusal| format!("{}: {}", refusal.cause.code(), refusal.cause))
        .collect()
}

fn text(node: Option<&Node>) -> String {
    match node {
        Some(Node::Text(value)) => value.clone(),
        other => panic!("expected text, got {other:?}"),
    }
}

fn branch(command: &CommandRef, name: &str) -> OutcomeRef {
    OutcomeRef::new(command.clone(), OutcomeName::new(name).unwrap())
}

fn event(name: &str, id: &str) -> ObservedEvent {
    ObservedEvent::new(name.parse::<EventRef>().unwrap()).with("order_id", Node::Text(id.into()))
}

/// How the hand-written service answers `ShipOrder` on a row resting in a held state other than
/// `Placed`, given the `reason` input where the command has one: the branch it reports and whether
/// it refuses with `Gone`.
type Answer = fn(&str, Option<&str>) -> (&'static str, bool);

struct Shipping {
    answer: Answer,
    minted: Cell<u64>,
    rows: RefCell<BTreeMap<String, BTreeMap<String, Node>>>,
    published: RefCell<Vec<ObservedEvent>>,
}

impl Shipping {
    fn new(answer: Answer) -> Self {
        Self {
            answer,
            minted: Cell::new(0),
            rows: RefCell::new(BTreeMap::new()),
            published: RefCell::new(Vec::new()),
        }
    }

    fn state(&self, id: &str) -> Option<String> {
        self.rows.borrow().get(id).map(|row| text(row.get("state")))
    }

    fn set(&self, id: &str, value: &str) {
        self.rows
            .borrow_mut()
            .get_mut(id)
            .unwrap()
            .insert("state".to_owned(), Node::Text(value.to_owned()));
    }

    fn finish(&self, result: SemanticCommandResult) -> SemanticCommandResult {
        for event in &result.direct_events {
            self.published.borrow_mut().push(event.clone());
        }
        self.minted.set(self.minted.get() + 1);
        result.with_consistency(
            ess_primitives::consistency::ConsistencyToken::new(format!(
                "seq:{}",
                self.minted.get()
            ))
            .unwrap(),
        )
    }
}

impl ConformanceTarget for Shipping {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("adversary-shipping-2", "1"))
    }
    fn begin_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        self.rows.replace(BTreeMap::new());
        self.published.replace(Vec::new());
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
        let result = if command.to_string() == "demo.ship.PlaceOrder" {
            self.minted.set(self.minted.get() + 1);
            let id = format!("00000000-0000-4000-8000-{:012}", self.minted.get());
            self.rows.borrow_mut().insert(
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
            let reason = match request.input.get("reason") {
                Some(Node::Text(value)) => Some(value.clone()),
                _ => None,
            };
            let Some(state) = self.state(&id) else {
                return Ok(self.finish(SemanticCommandResult::undeclared()));
            };
            match (command.to_string().as_str(), state.as_str()) {
                ("demo.ship.ShipOrder", "Placed") => {
                    self.set(&id, "Shipped");
                    SemanticCommandResult::took(branch(&command, "shipped"))
                        .emitting(event("demo.ship.OrderShipped", &id))
                }
                ("demo.ship.ShipOrder", held) => {
                    let (name, refused) = (self.answer)(held, reason.as_deref());
                    let took = SemanticCommandResult::took(branch(&command, name));
                    if refused {
                        took.with_error(DeclaredErrorValue::new(
                            "demo.ship.Gone".parse::<ErrorRef>().unwrap(),
                        ))
                    } else {
                        took
                    }
                }
                ("demo.ship.DeliverOrder", "Shipped") => {
                    self.set(&id, "Delivered");
                    SemanticCommandResult::took(branch(&command, "delivered"))
                        .emitting(event("demo.ship.OrderDelivered", &id))
                }
                ("demo.ship.CancelOrder", "Placed" | "Shipped") => {
                    self.set(&id, "Cancelled");
                    SemanticCommandResult::took(branch(&command, "cancelled"))
                        .emitting(event("demo.ship.OrderCancelled", &id))
                }
                _ => SemanticCommandResult::undeclared(),
            }
        };
        Ok(self.finish(result))
    }
    fn query_view(&self, _: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        Ok(SemanticViewResult::of(
            self.rows.borrow().values().cloned().collect::<Vec<_>>(),
        ))
    }
    fn observe_events(
        &self,
        request: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        Ok(self
            .published
            .borrow()
            .iter()
            .filter(|event| event.event == request.event)
            .cloned()
            .collect())
    }
    fn configure_external_outcome(&self, _: ExternalOutcomeControl) -> Result<(), TargetError> {
        panic!("nothing here is externally decided")
    }
    fn redeliver_event(&self, _: RedeliveryRequest) -> Result<(), TargetError> {
        panic!("no bindings")
    }
}

/// The `ShipOrder` scenarios `target` does not pass, with their checks.
fn failures(result: &Synthesis, target: &Shipping) -> Vec<String> {
    let admitted =
        AdmittedSuite::from_suite(&result.suite).unwrap_or_else(|error| panic!("{error}"));
    Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, target)
        .into_report()
        .scenarios
        .into_iter()
        .filter(|run| run.scenario.to_string().contains("ShipOrder"))
        .filter(|run| run.status != Status::Passed)
        .map(|run| format!("{}: {:?} {:?}", run.scenario, run.status, run.checks))
        .collect()
}

fn ship_order_ids(result: &Synthesis) -> Vec<String> {
    result
        .suite
        .scenarios
        .keys()
        .map(ToString::to_string)
        .filter(|id| id.contains("ShipOrder"))
        .collect()
}

// ---------------------------------------------------------------------------------------------
// F3 hole: a wrong state the `state` guard answers is dropped from the wrong-state family, and
// nothing else witnesses it.
// ---------------------------------------------------------------------------------------------

/// The issue's fourth spelling: one refusal reading `state != Placed`, no `wrong_state:` branch.
/// `Shipped`, `Delivered` and `Cancelled` are all answered by `gone`.
fn inequality() -> String {
    edited(&[(
        ACCEPT_AND_GONE,
        "      - name: gone\n        when_subject:\n          predicate: state != Placed\n        error: demo.ship.Gone\n",
    )])
}

fn gone_everywhere(_: &str, _: Option<&str>) -> (&'static str, bool) {
    ("gone", true)
}

/// Accepts a re-send in `Shipped` instead of refusing it.
fn accepts_in_shipped(held: &str, _: Option<&str>) -> (&'static str, bool) {
    match held {
        "Shipped" => ("shipped", false),
        _ => ("gone", true),
    }
}

fn accepts_in_delivered(held: &str, _: Option<&str>) -> (&'static str, bool) {
    match held {
        "Delivered" => ("shipped", false),
        _ => ("gone", true),
    }
}

fn accepts_in_cancelled(held: &str, _: Option<&str>) -> (&'static str, bool) {
    match held {
        "Cancelled" => ("shipped", false),
        _ => ("gone", true),
    }
}

/// Every state `state != Placed` answers is one the model refuses in; a target that accepts in
/// any one of them must fail a `ShipOrder` scenario, as it does for the listed spelling
/// `when_subject_state: [Shipped, Delivered, Cancelled]` and did at base for `wrong_state:`.
#[test]
fn a_state_inequality_refusal_is_witnessed_in_every_state_it_answers() {
    let result = synthesis(&inequality());
    assert_eq!(
        refusals_about(&result, "demo.ship.ShipOrder"),
        Vec::<String>::new()
    );
    let faithful = failures(&result, &Shipping::new(gone_everywhere));
    assert!(
        faithful.is_empty(),
        "the faithful target fails: {faithful:#?}"
    );
    let mut survivors = Vec::new();
    for (mutant, answer) in [
        ("accepts_in_shipped", accepts_in_shipped as Answer),
        ("accepts_in_delivered", accepts_in_delivered as Answer),
        ("accepts_in_cancelled", accepts_in_cancelled as Answer),
    ] {
        if failures(&result, &Shipping::new(answer)).is_empty() {
            survivors.push(mutant);
        }
    }
    assert!(
        survivors.is_empty(),
        "mutants passing every ShipOrder scenario: {survivors:?}; scenarios: {:#?}",
        ship_order_ids(&result)
    );
}

/// The same with `not`: `not (state == Placed)`.
#[test]
fn a_negated_state_refusal_is_witnessed_in_every_state_it_answers() {
    let result = synthesis(&edited(&[(
        ACCEPT_AND_GONE,
        "      - name: gone\n        when_subject:\n          predicate:\n            not: state == Placed\n        error: demo.ship.Gone\n",
    )]));
    assert_eq!(
        refusals_about(&result, "demo.ship.ShipOrder"),
        Vec::<String>::new()
    );
    let mut survivors = Vec::new();
    for (mutant, answer) in [
        ("accepts_in_shipped", accepts_in_shipped as Answer),
        ("accepts_in_delivered", accepts_in_delivered as Answer),
        ("accepts_in_cancelled", accepts_in_cancelled as Answer),
    ] {
        if failures(&result, &Shipping::new(answer)).is_empty() {
            survivors.push(mutant);
        }
    }
    assert!(
        survivors.is_empty(),
        "mutants passing every ShipOrder scenario: {survivors:?}; scenarios: {:#?}",
        ship_order_ids(&result)
    );
}

// ---------------------------------------------------------------------------------------------
// F3 residue: guards that cover a wrong state only together.
// ---------------------------------------------------------------------------------------------

/// Two `state`-reading refusals, told apart by the input, together answer every wrong state for
/// every input; there is no `wrong_state:` branch. No wrong-state row is left, as in the case the
/// correction handles, but neither guard alone is true with `state` bound alone.
#[test]
fn state_guards_split_by_the_input_covering_every_wrong_state_synthesize() {
    let text = edited(&[
        REASON_TYPE,
        REASON_INPUT,
        (
            ACCEPT_AND_GONE,
            "      - name: gone\n        when_subject:\n          predicate: state != Placed\n        when: reason == Late\n        error: demo.ship.Gone\n      - name: gone-quietly\n        when_subject:\n          predicate: state != Placed\n        when: reason == Lost\n        error: demo.ship.Gone\n",
        ),
    ]);
    let result = synthesis(&text);
    assert_eq!(
        refusals_about(&result, "demo.ship.ShipOrder"),
        Vec::<String>::new()
    );
}

// ---------------------------------------------------------------------------------------------
// F3 with `or` mixing `state` and an input: true with `state` alone in one state, open in others.
// ---------------------------------------------------------------------------------------------

/// `gone` answers every cancelled row, and any row when the reason is `Lost`; `stale` is the
/// `wrong_state:` answer for every other wrong row. Cancelled is skipped from the wrong-state
/// family (true with `state` alone); Shipped and Delivered keep their wrong-state scenario.
#[test]
fn a_state_or_input_guard_beside_wrong_state_synthesizes() {
    let text = edited(&[
        REASON_TYPE,
        REASON_INPUT,
        (
            ACCEPT_AND_GONE,
            "      - name: gone\n        when_subject:\n          predicate:\n            any:\n              - state == Cancelled\n              - input.reason == Lost\n        error: demo.ship.Gone\n      - name: stale\n        wrong_state: true\n        error: demo.ship.Gone\n",
        ),
    ]);
    let result = synthesis(&text);
    assert_eq!(
        refusals_about(&result, "demo.ship.ShipOrder"),
        Vec::<String>::new()
    );
}

// ---------------------------------------------------------------------------------------------
// F2 mutants: the wrong branch among several refusals claiming one state.
// ---------------------------------------------------------------------------------------------

fn split_by_reason() -> String {
    edited(&[
        REASON_TYPE,
        REASON_INPUT,
        (
            "      - name: gone\n        when_subject_state: [Delivered, Cancelled]\n        error: demo.ship.Gone\n",
            "      - name: gone\n        when_subject_state: [Delivered, Cancelled]\n        when: reason == Late\n        error: demo.ship.Gone\n      - name: gone-quietly\n        when_subject_state: [Delivered, Cancelled]\n        when: reason == Lost\n        error: demo.ship.Gone\n",
        ),
    ])
}

fn split_correct(held: &str, reason: Option<&str>) -> (&'static str, bool) {
    match (held, reason) {
        ("Shipped", _) => ("already-shipped", false),
        (_, Some("Late")) => ("gone", true),
        _ => ("gone-quietly", true),
    }
}

/// Answers `gone` for every refused row, whatever the reason.
fn split_first_only(held: &str, _: Option<&str>) -> (&'static str, bool) {
    match held {
        "Shipped" => ("already-shipped", false),
        _ => ("gone", true),
    }
}

/// Swaps the two refusals in `Cancelled` only.
fn split_swapped_in_cancelled(held: &str, reason: Option<&str>) -> (&'static str, bool) {
    match (held, reason) {
        ("Shipped", _) => ("already-shipped", false),
        ("Cancelled", Some("Late")) => ("gone-quietly", true),
        ("Cancelled", _) | (_, Some("Late")) => ("gone", true),
        _ => ("gone-quietly", true),
    }
}

/// Accepts a delivered order sent with `Lost`.
fn split_accepts_lost_delivered(held: &str, reason: Option<&str>) -> (&'static str, bool) {
    match (held, reason) {
        ("Shipped", _) | ("Delivered", Some("Lost")) => ("already-shipped", false),
        (_, Some("Late")) => ("gone", true),
        _ => ("gone-quietly", true),
    }
}

#[test]
fn a_target_answering_the_wrong_refusal_of_one_state_fails() {
    let result = synthesis(&split_by_reason());
    assert_eq!(
        refusals_about(&result, "demo.ship.ShipOrder"),
        Vec::<String>::new()
    );
    let faithful = failures(&result, &Shipping::new(split_correct));
    assert!(
        faithful.is_empty(),
        "the faithful target fails: {faithful:#?}"
    );
    let mut survivors = Vec::new();
    for (mutant, answer) in [
        ("split_first_only", split_first_only as Answer),
        (
            "split_swapped_in_cancelled",
            split_swapped_in_cancelled as Answer,
        ),
        (
            "split_accepts_lost_delivered",
            split_accepts_lost_delivered as Answer,
        ),
    ] {
        if failures(&result, &Shipping::new(answer)).is_empty() {
            survivors.push(mutant);
        }
    }
    assert!(survivors.is_empty(), "surviving mutants: {survivors:?}");
}

/// Synthesis is a function of the model: two runs give byte-identical suites, and declaring the
/// two refusals in the other order changes only which is witnessed first.
#[test]
fn split_refusals_synthesize_deterministically() {
    assert_eq!(synthesis(&split_by_reason()), synthesis(&split_by_reason()));
}

/// A listed accepting branch of every state a move does not start from, and a list of one.
#[test]
fn a_listed_accepting_guard_of_one_state_or_every_wrong_state_synthesizes() {
    for (label, replacement) in [
        (
            "one",
            "      - name: already-shipped\n        when_subject_state: [Shipped]\n        preserves: demo.ship.Order\n        instance: order_id\n      - name: gone\n        when_subject_state: [Delivered, Cancelled]\n        error: demo.ship.Gone\n",
        ),
        (
            "every",
            "      - name: already-shipped\n        when_subject_state: [Shipped, Delivered, Cancelled]\n        preserves: demo.ship.Order\n        instance: order_id\n",
        ),
    ] {
        let result = synthesis(&edited(&[(ACCEPT_AND_GONE, replacement)]));
        assert_eq!(
            refusals_about(&result, "demo.ship.ShipOrder"),
            Vec::<String>::new(),
            "{label}"
        );
    }
}
