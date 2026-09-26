//! A generated suite pins the behaviour a specification declares (beyond10x/ess#111).
//!
//! A mutation audit of a real implementation found eight mutants that changed no verdict, six of
//! them behaviour the specification declares, each through one of four generator gaps. This file
//! reproduces all four on the issue's minimal specification — extended with an enum written at
//! creation (`tier`), an enum nobody writes before an update (`colour`) and a compound guard
//! (`Restock`) — and runs each surviving mutant against the generated suite:
//!
//! | gap | mutant | fails |
//! |---|---|---|
//! | a transition never asserts the state it is named for | `close` leaves the order where it was | the `close` transition scenario |
//! | an update writes the value the row already holds | `Relabel` drops `tier` from `sets:`, or `colour` | the `Relabel` outcome scenario |
//! | a multi-source transition runs from one source | `close` from `Open` only | the `close` transition scenario |
//! | guard boundaries are not probed | `items >= 0` → `> 0`; `n >= 1` → `>= 0` | the `set` success; the `Restock` refusal |
use ess_compiler::{
    ir::EssIr,
    refs::{CommandRef, OutcomeRef},
    resolve::compile,
    source::SourceMap,
};
use ess_conformance::{
    report::{ConformanceStatus, Status},
    scenario::ViewExpectation,
    target::*,
    AdmittedSuite, ConformanceScenario, ConformanceSuite, Runner, ScenarioStep, ScenarioValue,
};
use ess_domain::{
    command::OutcomeName,
    spec::{RawSpecFile, Specification},
    system::Source,
};
use ess_primitives::node::Node;
use std::{
    cell::{Cell, RefCell},
    collections::BTreeMap,
};

const SHOP: &str = include_str!("fixtures/declared-behaviour.yaml");
const CLOSE: &str = "shop.orders.Order/transition/close/by/shop.orders.CloseOrder/closed";
const HOLD: &str = "shop.orders.Order/transition/hold/by/shop.orders.HoldOrder/held";
const RELABEL: &str = "shop.orders.Relabel/outcome/relabelled";
const ITEMS_SET: &str = "shop.orders.SetItems/outcome/set";
const ITEMS_REFUSED: &str = "shop.orders.SetItems/outcome/refused";
const RESTOCKED: &str = "shop.orders.Restock/outcome/restocked";
const RESTOCK_REFUSED: &str = "shop.orders.Restock/outcome/refused";

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}"));
    let spec = Specification::assemble([(Source::new("shop.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap()
}

fn suite() -> ConformanceSuite {
    let synthesis = ess_conformance::synthesize::synthesize(&ir(SHOP));
    assert!(synthesis.refusals.is_empty(), "{:?}", synthesis.refusals);
    synthesis.suite
}

fn scenario<'a>(suite: &'a ConformanceSuite, id: &str) -> &'a ConformanceScenario {
    suite
        .scenarios
        .iter()
        .find(|(key, _)| key.to_string() == id)
        .map_or_else(|| panic!("no scenario {id}"), |(_, scenario)| scenario)
}

fn literal(text: &str) -> ScenarioValue {
    ScenarioValue::literal(Node::Text(text.into()))
}

fn number(value: f64) -> ScenarioValue {
    ScenarioValue::literal(Node::Number(
        ess_primitives::facts::Number::new(value).unwrap(),
    ))
}

fn instance(name: &str) -> ScenarioValue {
    ScenarioValue::instance(name.parse().unwrap())
}

/// The input every `ExecuteCommand` of `command` sent, in step order.
fn inputs(scenario: &ConformanceScenario, command: &str) -> Vec<BTreeMap<String, ScenarioValue>> {
    scenario
        .steps
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::ExecuteCommand {
                command: sent,
                input,
                ..
            } if sent.to_string() == command => Some(input.clone()),
            _ => None,
        })
        .collect()
}

/// Every `Contains` row asserted, in step order.
fn rows(scenario: &ConformanceScenario) -> Vec<BTreeMap<String, ScenarioValue>> {
    scenario
        .steps
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::ExpectView {
                expectation: ViewExpectation::Contains { fields },
                ..
            }
            | ScenarioStep::EventuallyView {
                expectation: ViewExpectation::Contains { fields },
                ..
            } => Some(fields.clone()),
            _ => None,
        })
        .collect()
}

/// Every outcome the scenario requires of `command`, in step order.
fn required(scenario: &ConformanceScenario, command: &str) -> Vec<String> {
    scenario
        .steps
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::ExpectOutcome { outcome } if outcome.command.to_string() == command => {
                Some(outcome.outcome.to_string())
            }
            _ => None,
        })
        .collect()
}

#[test]
fn a_transition_scenario_asserts_the_state_it_moves_to() {
    let suite = suite();
    for (id, to) in [(CLOSE, "Closed"), (HOLD, "Held")] {
        let rows = rows(scenario(&suite, id));
        assert!(
            rows.iter()
                .any(|row| row.get("order_id") == Some(&instance("order"))
                    && row.get("state") == Some(&literal(to))),
            "{id}: the view projects `state`, so the row is required in `{to}`: {rows:?}"
        );
    }
}

#[test]
fn an_update_witness_differs_from_what_the_row_already_holds() {
    let suite = suite();
    let relabel = scenario(&suite, RELABEL);
    let placed = inputs(relabel, "shop.orders.PlaceOrder");
    assert_eq!(
        placed,
        [BTreeMap::from([("tier".to_owned(), literal("Low"))])]
    );
    let sent = inputs(relabel, "shop.orders.Relabel");
    assert_eq!(sent.len(), 1, "{sent:?}");
    assert_eq!(
        sent[0].get("tier"),
        Some(&literal("High")),
        "the setup wrote `Low`, so the update is witnessed at the other variant"
    );
    assert_eq!(
        sent[0].get("colour"),
        Some(&literal("Red")),
        "nothing wrote `colour`, so the update is witnessed off its first variant"
    );
    let rows = rows(relabel);
    assert!(
        rows.iter()
            .any(|row| row.get("tier") == Some(&literal("High"))
                && row.get("colour") == Some(&literal("Red"))),
        "{rows:?}"
    );
}

#[test]
fn a_multi_source_transition_is_exercised_from_every_source() {
    let suite = suite();
    let close = scenario(&suite, CLOSE);
    let held = inputs(close, "shop.orders.HoldOrder");
    let closed = inputs(close, "shop.orders.CloseOrder");
    assert_eq!(
        closed
            .iter()
            .map(|input| input.get("order_id").cloned())
            .collect::<Vec<_>>(),
        [Some(instance("order")), Some(instance("order-2"))],
        "one close from `Open` and one from `Held`, each on its own order"
    );
    assert_eq!(
        held.iter()
            .map(|input| input.get("order_id").cloned())
            .collect::<Vec<_>>(),
        [Some(instance("order-2"))],
        "the second order is held before it is closed"
    );
    assert_eq!(
        required(close, "shop.orders.CloseOrder"),
        ["closed", "closed"]
    );
    let rows = rows(close);
    assert!(
        rows.iter()
            .any(|row| row.get("order_id") == Some(&instance("order-2"))
                && row.get("state") == Some(&literal("Closed"))),
        "{rows:?}"
    );
    let single_source = scenario(&suite, HOLD);
    assert_eq!(
        inputs(single_source, "shop.orders.HoldOrder").len(),
        1,
        "`hold` has one source and runs once"
    );
}

#[test]
fn an_integer_guard_is_witnessed_at_its_literal_and_the_neighbour_across_it() {
    let suite = suite();
    let sent = |id: &str, command: &str, field: &str| -> Vec<Option<ScenarioValue>> {
        inputs(scenario(&suite, id), command)
            .iter()
            .map(|input| input.get(field).cloned())
            .collect()
    };
    assert!(
        sent(ITEMS_SET, "shop.orders.SetItems", "items").contains(&Some(number(0.0))),
        "`items >= 0` is accepted at `0`"
    );
    assert!(
        sent(ITEMS_REFUSED, "shop.orders.SetItems", "items").contains(&Some(number(-1.0))),
        "and refused at `-1`"
    );
    let refused: Vec<_> = inputs(scenario(&suite, RESTOCK_REFUSED), "shop.orders.Restock")
        .into_iter()
        .map(|input| (input.get("n").cloned(), input.get("limit").cloned()))
        .collect();
    assert!(
        refused.contains(&(Some(number(0.0)), Some(number(1.0)))),
        "`n >= 1` is refused at `0` while `limit > 0` holds: {refused:?}"
    );
    assert!(
        refused.contains(&(Some(number(1.0)), Some(number(0.0)))),
        "`limit > 0` is refused at `0` while `n >= 1` holds: {refused:?}"
    );
    let restocked: Vec<_> = inputs(scenario(&suite, RESTOCKED), "shop.orders.Restock")
        .into_iter()
        .map(|input| (input.get("n").cloned(), input.get("limit").cloned()))
        .collect();
    assert_eq!(
        restocked,
        [(Some(number(1.0)), Some(number(1.0)))],
        "the plain witness is already both accepting boundaries, so it is not sent twice"
    );
    assert_eq!(
        required(scenario(&suite, RESTOCK_REFUSED), "shop.orders.Restock"),
        ["refused", "refused"]
    );
}

#[test]
fn the_suite_needs_no_new_vocabulary_and_every_lane_admits_it() {
    let suite = suite();
    assert_eq!(
        suite.provenance.suite_version.to_string(),
        "ess-conformance/12",
        "every assertion added is an existing step"
    );
    AdmittedSuite::from_suite(&suite).unwrap();
    ess_conformance::go::emit(&suite).unwrap();
    ess_conformance::ts::emit(&suite).unwrap();
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Mutant {
    /// The specification as declared.
    None,
    /// `close` answers `closed` and leaves the order where it was: `B → B` for `B → C`.
    CloseStays,
    /// `close` runs from `Open` only: the second source dropped.
    CloseFromOpenOnly,
    /// `Relabel` with `tier` removed from its `sets:`.
    RelabelDropsTier,
    /// `Relabel` with `colour` removed from its `sets:`.
    RelabelDropsColour,
    /// `items >= 0` moved to `items > 0`.
    ItemsStrict,
    /// `n >= 1` moved to `n >= 0`.
    RestockFromZero,
}

struct Shop {
    mutant: Mutant,
    rows: RefCell<BTreeMap<String, BTreeMap<String, Node>>>,
    minted: Cell<u64>,
}

impl Shop {
    fn new(mutant: Mutant) -> Self {
        Self {
            mutant,
            rows: RefCell::default(),
            minted: Cell::new(0),
        }
    }
}

fn outcome(command: &CommandRef, name: &str) -> OutcomeRef {
    OutcomeRef::new(command.clone(), OutcomeName::new(name).unwrap())
}

fn event(name: &str, order: &str) -> ObservedEvent {
    ObservedEvent::new(format!("shop.orders.{name}").parse().unwrap())
        .with("order_id", Node::Text(order.to_owned()))
}

fn integer(node: Option<&Node>) -> f64 {
    match node {
        Some(Node::Number(number)) => number.get(),
        other => panic!("an integer input, not {other:?}"),
    }
}

impl ConformanceTarget for Shop {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("shop-fixture", "1"))
    }
    fn begin_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        self.rows.replace(BTreeMap::new());
        Ok(())
    }
    fn end_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        Ok(())
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        self.minted.set(self.minted.get() + 1);
        let token = ess_primitives::consistency::ConsistencyToken::new(format!(
            "seq:{}",
            self.minted.get()
        ))
        .unwrap();
        self.answer(&request)
            .map(|result| result.with_consistency(token))
    }
    fn query_view(&self, _: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        Ok(SemanticViewResult::of(self.rows.borrow().values().cloned()))
    }
    fn observe_events(
        &self,
        _: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        Ok(Vec::new())
    }
    fn configure_external_outcome(&self, _: ExternalOutcomeControl) -> Result<(), TargetError> {
        panic!("nothing here is externally decided")
    }
    fn redeliver_event(&self, _: RedeliveryRequest) -> Result<(), TargetError> {
        panic!("no bindings")
    }
}

impl Shop {
    /// Whether the command's input guard admits the request: `items >= 0` and `n >= 1 and
    /// limit > 0`, each as its mutant moves it.
    fn admits(&self, command: &str, request: &SemanticCommandRequest) -> bool {
        match command {
            "shop.orders.SetItems" => {
                let items = integer(request.input.get("items"));
                if self.mutant == Mutant::ItemsStrict {
                    items > 0.0
                } else {
                    items >= 0.0
                }
            }
            "shop.orders.Restock" => {
                let floor = if self.mutant == Mutant::RestockFromZero {
                    0.0
                } else {
                    1.0
                };
                integer(request.input.get("n")) >= floor
                    && integer(request.input.get("limit")) > 0.0
            }
            _ => true,
        }
    }

    fn answer(
        &self,
        request: &SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        let mut rows = self.rows.borrow_mut();
        let command = request.command.clone();
        let name = command.to_string();
        if name == "shop.orders.PlaceOrder" {
            self.minted.set(self.minted.get() + 1);
            let id = format!("00000000-0000-4000-8000-{:012}", self.minted.get());
            rows.insert(
                id.clone(),
                BTreeMap::from([
                    ("order_id".to_owned(), Node::Text(id.clone())),
                    ("state".to_owned(), Node::Text("Open".into())),
                    (
                        "items".to_owned(),
                        Node::Number(ess_primitives::facts::Number::new(0.0).unwrap()),
                    ),
                    ("tier".to_owned(), request.input["tier"].clone()),
                    // An enum no act has written starts at its first variant.
                    ("colour".to_owned(), Node::Text("Green".into())),
                ]),
            );
            return Ok(SemanticCommandResult::took(outcome(&command, "placed"))
                .emitting(event("OrderPlaced", &id)));
        }
        // Guards over the input are decided before the subject is looked up, as the refusal
        // scenarios send an identity no row carries.
        if !self.admits(&name, request) {
            return Ok(
                SemanticCommandResult::took(outcome(&command, "refused")).with_error(
                    DeclaredErrorValue::new("shop.orders.InvalidItems".parse().unwrap()),
                ),
            );
        }
        let Some(Node::Text(id)) = request.input.get("order_id") else {
            return Ok(SemanticCommandResult::undeclared());
        };
        let id = id.clone();
        let Some(row) = rows.get_mut(&id) else {
            return Ok(SemanticCommandResult::undeclared());
        };
        let state = match &row["state"] {
            Node::Text(state) => state.clone(),
            other => panic!("a state is text, not {other:?}"),
        };
        let conflict = || {
            SemanticCommandResult::took(outcome(&command, "wrong-state")).with_error(
                DeclaredErrorValue::new("shop.orders.OrderStateConflict".parse().unwrap())
                    .with("state", Node::Text(state.clone())),
            )
        };
        match name.as_str() {
            "shop.orders.HoldOrder" => {
                if state != "Open" {
                    return Ok(conflict());
                }
                row.insert("state".into(), Node::Text("Held".into()));
                Ok(SemanticCommandResult::took(outcome(&command, "held"))
                    .emitting(event("OrderHeld", &id)))
            }
            "shop.orders.CloseOrder" => {
                let sources: &[&str] = if self.mutant == Mutant::CloseFromOpenOnly {
                    &["Open"]
                } else {
                    &["Open", "Held"]
                };
                if !sources.contains(&state.as_str()) {
                    return Ok(conflict());
                }
                if self.mutant != Mutant::CloseStays {
                    row.insert("state".into(), Node::Text("Closed".into()));
                }
                Ok(SemanticCommandResult::took(outcome(&command, "closed"))
                    .emitting(event("OrderClosed", &id)))
            }
            "shop.orders.SetItems" => {
                row.insert("items".into(), request.input["items"].clone());
                Ok(SemanticCommandResult::took(outcome(&command, "set"))
                    .emitting(event("ItemsSet", &id).with("items", request.input["items"].clone())))
            }
            "shop.orders.Relabel" => {
                if self.mutant != Mutant::RelabelDropsTier {
                    row.insert("tier".into(), request.input["tier"].clone());
                }
                if self.mutant != Mutant::RelabelDropsColour {
                    row.insert("colour".into(), request.input["colour"].clone());
                }
                Ok(SemanticCommandResult::took(outcome(&command, "relabelled"))
                    .emitting(event("Relabelled", &id)))
            }
            "shop.orders.Restock" => {
                Ok(SemanticCommandResult::took(outcome(&command, "restocked"))
                    .emitting(event("Restocked", &id)))
            }
            other => Err(TargetError::unsupported("command", other)),
        }
    }
}

fn failing(mutant: Mutant) -> Vec<String> {
    let suite = suite();
    let admitted = AdmittedSuite::from_suite(&suite).unwrap();
    let report = Runner::for_suite(&suite)
        .run_admitted(&admitted, &Shop::new(mutant))
        .into_report();
    let failed: Vec<String> = report
        .scenarios
        .iter()
        .filter(|scenario| scenario.status != Status::Passed)
        .map(|scenario| scenario.scenario.to_string())
        .collect();
    assert_eq!(
        report.status == ConformanceStatus::Passed,
        failed.is_empty(),
        "{report:?}"
    );
    failed
}

fn fails(mutant: Mutant, id: &str) {
    let failed = failing(mutant);
    assert!(
        failed.contains(&id.to_owned()),
        "{mutant:?} must fail {id}; failed: {failed:?}"
    );
}

#[test]
fn the_specification_as_declared_passes_its_own_suite() {
    assert_eq!(failing(Mutant::None), Vec::<String>::new());
}

#[test]
fn a_close_that_leaves_the_order_where_it_was_fails_the_transition() {
    fails(Mutant::CloseStays, CLOSE);
}

#[test]
fn a_close_from_one_source_only_fails_the_transition() {
    fails(Mutant::CloseFromOpenOnly, CLOSE);
}

#[test]
fn an_update_that_drops_a_field_the_setup_wrote_fails_it() {
    fails(Mutant::RelabelDropsTier, RELABEL);
}

#[test]
fn an_update_that_drops_a_field_nobody_wrote_fails_it() {
    fails(Mutant::RelabelDropsColour, RELABEL);
}

#[test]
fn a_boundary_moved_past_the_literal_fails_the_success() {
    fails(Mutant::ItemsStrict, ITEMS_SET);
}

#[test]
fn a_boundary_moved_below_one_conjunct_fails_the_refusal() {
    fails(Mutant::RestockFromZero, RESTOCK_REFUSED);
}
