//! Adversary pass 1 on the state-scoped unit (beyond10x/ess#201, #204), `ess/18`.
//!
//! Each case synthesizes a valid model and asks what the suite witnesses: a listed held-state guard
//! on an accepting branch must be witnessed in every state it lists, as the split spelling is; and
//! two refusals of one held state told apart by the input must each keep their scenarios.
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

fn replaced(text: &str, from: &str, to: &str) -> String {
    let out = text.replace(from, to);
    assert_ne!(out, text, "`{from}` is in the fixture");
    out
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
        .map(|refusal| format!("{}: {} ({refusal:?})", refusal.cause.code(), refusal.cause))
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

/// How the hand-written service answers `ShipOrder` on a row resting in a given state.
type Answer = fn(&str) -> (&'static str, bool);

/// A shipping service: `PlaceOrder`, `DeliverOrder` and `CancelOrder` as the model has them, and
/// `ShipOrder` answered by `answer` for every state but `Placed`, which ships.
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
        Ok(ImplementationIdentity::new("adversary-shipping", "1"))
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
                    let (name, refused) = (self.answer)(held);
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

/// A listed guard on the accepting branch: `already-shipped` answers `Delivered` and `Shipped`,
/// `gone` answers `Cancelled` alone.
fn listed_accepting() -> String {
    replaced(
        SHIP,
        ACCEPT_AND_GONE,
        "      - name: already-shipped\n        when_subject_state: [Shipped, Delivered]\n        preserves: demo.ship.Order\n        instance: order_id\n      - name: gone\n        when_subject_state: Cancelled\n        error: demo.ship.Gone\n",
    )
}

fn correct(held: &str) -> (&'static str, bool) {
    match held {
        "Cancelled" => ("gone", true),
        _ => ("already-shipped", false),
    }
}

/// Refuses a shipped order as gone: the listed branch answered in `Delivered` only.
fn misses_shipped(held: &str) -> (&'static str, bool) {
    match held {
        "Cancelled" | "Shipped" => ("gone", true),
        _ => ("already-shipped", false),
    }
}

/// Refuses a delivered order as gone: the listed branch answered in `Shipped` only.
fn misses_delivered(held: &str) -> (&'static str, bool) {
    match held {
        "Cancelled" | "Delivered" => ("gone", true),
        _ => ("already-shipped", false),
    }
}

#[test]
fn a_listed_accepting_guard_is_witnessed_in_every_state_it_lists() {
    let result = synthesis(&listed_accepting());
    assert_eq!(
        refusals_about(&result, "demo.ship.ShipOrder"),
        Vec::<String>::new()
    );
    let faithful = failures(&result, &Shipping::new(correct));
    assert!(
        faithful.is_empty(),
        "the faithful target fails: {faithful:#?}"
    );
    for (mutant, answer) in [
        ("misses_shipped", misses_shipped as Answer),
        ("misses_delivered", misses_delivered as Answer),
    ] {
        assert!(
            !failures(&result, &Shipping::new(answer)).is_empty(),
            "{mutant} passes every ShipOrder scenario: a state `when_subject_state: [Shipped, \
             Delivered]` lists is never arranged for `already-shipped`"
        );
    }
}

/// Two refusals of one held state, told apart by the input: each is a declared branch the model
/// selects, so each is witnessed and synthesis refuses neither.
#[test]
fn two_refusals_of_one_held_state_split_by_the_input_both_synthesize() {
    let text = replaced(
        &replaced(
            &replaced(
                SHIP,
                "  - {name: demo.ship.OrderId, kind: newtype, of: Uuid}\n",
                "  - {name: demo.ship.OrderId, kind: newtype, of: Uuid}\n  - {name: demo.ship.Reason, kind: enum, variants: [Late, Lost]}\n",
            ),
            "  - name: demo.ship.ShipOrder\n    input: [{name: order_id, type: demo.ship.OrderId}]\n",
            "  - name: demo.ship.ShipOrder\n    input: [{name: order_id, type: demo.ship.OrderId}, {name: reason, type: demo.ship.Reason}]\n",
        ),
        "      - name: gone\n        when_subject_state: [Delivered, Cancelled]\n        error: demo.ship.Gone\n",
        "      - name: gone\n        when_subject_state: [Delivered, Cancelled]\n        when: reason == Late\n        error: demo.ship.Gone\n      - name: gone-quietly\n        when_subject_state: [Delivered, Cancelled]\n        when: reason == Lost\n        error: demo.ship.Gone\n",
    );
    let result = synthesis(&text);
    assert_eq!(
        refusals_about(&result, "demo.ship.ShipOrder"),
        Vec::<String>::new()
    );
}

/// The #204 spelling of the same accepting branch: `state` read in a `when_subject` disjunction.
/// Each disjunct is a state the branch answers, so each is witnessed.
fn state_disjunction() -> String {
    replaced(
        SHIP,
        ACCEPT_AND_GONE,
        "      - name: already-shipped\n        when_subject:\n          predicate:\n            any:\n              - state == Shipped\n              - state == Delivered\n        preserves: demo.ship.Order\n        instance: order_id\n      - name: gone\n        when_subject:\n          predicate: state == Cancelled\n        error: demo.ship.Gone\n",
    )
}

/// Guards reading `state` answer every state the move does not start from, and the command
/// declares no `wrong_state:` branch: no wrong-state row is left, so the wrong-state family has
/// nothing to witness and must not refuse.
#[test]
fn state_guards_covering_every_wrong_state_leave_no_wrong_state_refusal() {
    let result = synthesis(&state_disjunction());
    assert_eq!(
        refusals_about(&result, "demo.ship.ShipOrder"),
        Vec::<String>::new()
    );
}

#[test]
fn a_state_disjunction_in_when_subject_is_witnessed_in_every_state_it_names() {
    let result = synthesis(&state_disjunction());
    let faithful = failures(&result, &Shipping::new(correct));
    assert!(
        faithful.is_empty(),
        "the faithful target fails: {faithful:#?}"
    );
    for (mutant, answer) in [
        ("misses_shipped", misses_shipped as Answer),
        ("misses_delivered", misses_delivered as Answer),
    ] {
        assert!(
            !failures(&result, &Shipping::new(answer)).is_empty(),
            "{mutant} passes every ShipOrder scenario"
        );
    }
}

/// The order an author lists held states in is not part of the model: both orders compile to one
/// canonical IR, and a scalar keeps the bytes it had before `ess/18`.
#[test]
fn a_listed_guard_compiles_to_one_ir_whatever_order_it_is_written_in() {
    let reordered = replaced(SHIP, "[Delivered, Cancelled]", "[Cancelled, Delivered]");
    assert_eq!(
        ir(SHIP).to_canonical_json(),
        ir(&reordered).to_canonical_json()
    );
    let canonical: String = ir(SHIP).to_canonical_json().split_whitespace().collect();
    assert!(canonical.contains(r#""state":"Shipped""#), "{canonical}");
    assert!(
        canonical.contains(r#""state":["Cancelled","Delivered"]"#),
        "{canonical}"
    );
}
