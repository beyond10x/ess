//! `when_related:` (beyond10x/ess#211, `ess/18`) on a command that moves an existing subject: the
//! refusal naming no subject is sent for the row its sibling moves, arranged where that move starts,
//! so the related row is the only thing that can refuse it; the move is sent for a row that exists.
//! The illegal-move family, which has no related row to point the command at, refuses by name
//! rather than sending the command for a row nobody arranged.
use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;

use ess_compiler::ir::EssIr;
use ess_compiler::{resolve::compile, source::SourceMap};
use ess_conformance::report::Status;
use ess_conformance::scenario::{CommandRef, ErrorRef, EventRef, OutcomeRef};
use ess_conformance::target::*;
use ess_conformance::{
    synthesize::Synthesis, AdmittedSuite, ConformanceScenario, Runner, ScenarioStep, ScenarioValue,
};
use ess_domain::{
    command::OutcomeName,
    spec::{RawSpecFile, Specification},
    system::Source,
};
use ess_primitives::node::Node;

const SHIP: &str = "format: ess/18
system: demo
version: v1
domain: demo.ship
types:
  - {name: demo.ship.OrderId, kind: newtype, of: Uuid}
  - {name: demo.ship.CarrierId, kind: newtype, of: Uuid}
entities:
  - name: demo.ship.Carrier
    identity: {name: carrier_id, type: demo.ship.CarrierId}
    fields: []
    lifecycle: {initial: Active, states: [Active], terminal: [Active]}
  - name: demo.ship.Order
    identity: {name: order_id, type: demo.ship.OrderId}
    fields: []
    lifecycle:
      initial: Placed
      states: [Placed, Shipped]
      terminal: [Shipped]
      transitions:
        - {name: ship, from: [Placed], to: Shipped}
errors:
  - {name: demo.ship.NoCarrier, summary: No carrier carries that identity., fields: []}
events:
  - name: demo.ship.CarrierRegistered
    fields: [{name: carrier_id, type: demo.ship.CarrierId}]
  - name: demo.ship.OrderPlaced
    fields: [{name: order_id, type: demo.ship.OrderId}]
  - name: demo.ship.OrderShipped
    fields: [{name: order_id, type: demo.ship.OrderId}]
actors:
  - name: demo.ship.Clerk
    may: [demo.ship.RegisterCarrier, demo.ship.PlaceOrder, demo.ship.ShipOrder]
commands:
  - name: demo.ship.RegisterCarrier
    input: []
    outcomes:
      - name: registered
        creates: demo.ship.Carrier
        instance: carrier_id
        emits: [demo.ship.CarrierRegistered]
        payload: {demo.ship.CarrierRegistered: {carrier_id: {generated: true}}}
  - name: demo.ship.PlaceOrder
    input: []
    outcomes:
      - name: placed
        creates: demo.ship.Order
        instance: order_id
        emits: [demo.ship.OrderPlaced]
        payload: {demo.ship.OrderPlaced: {order_id: {generated: true}}}
  - name: demo.ship.ShipOrder
    input:
      - {name: order_id, type: demo.ship.OrderId}
      - {name: carrier, type: demo.ship.CarrierId}
    outcomes:
      - name: no-carrier
        when_related: {via: input.carrier, exists: false}
        error: demo.ship.NoCarrier
      - name: shipped
        moves: demo.ship.Order.ship
        instance: order_id
        emits: [demo.ship.OrderShipped]
        payload: {demo.ship.OrderShipped: {order_id: input.order_id}}
views:
  - name: demo.ship.Carriers
    source: demo.ship.Carrier
    consistency: read_your_writes
    fields:
      - {name: carrier_id, type: demo.ship.CarrierId}
  - name: demo.ship.Orders
    source: demo.ship.Order
    consistency: read_your_writes
    fields:
      - {name: order_id, type: demo.ship.OrderId}
      - {name: state, type: demo.ship.Order.State}
";

const NO_CARRIER: &str = "demo.ship.ShipOrder/outcome/no-carrier";
const SHIPPED: &str = "demo.ship.ShipOrder/outcome/shipped";
const ILLEGAL: &str = "demo.ship.Order/state/Shipped/refuses/demo.ship.ShipOrder";

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("ship.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn synthesis() -> Synthesis {
    ess_conformance::synthesize::synthesize(&ir(SHIP))
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
                    "no scenario {id}\n refusals: {:#?}",
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

/// The input of the last `ShipOrder` the scenario sends.
fn shipping(scenario: &ConformanceScenario) -> BTreeMap<String, ScenarioValue> {
    scenario
        .steps
        .iter()
        .rev()
        .find_map(|step| match step {
            ScenarioStep::ExecuteCommand { command, input, .. }
                if command.to_string() == "demo.ship.ShipOrder" =>
            {
                Some(input.clone())
            }
            _ => None,
        })
        .expect("the scenario ships an order")
}

#[test]
fn issue_211_the_subjectless_refusal_is_sent_for_an_order_the_move_starts_from() {
    let result = synthesis();
    let refused = shipping(scenario(&result, NO_CARRIER));
    assert!(
        matches!(
            refused.get("order_id"),
            Some(ScenarioValue::Instance { .. })
        ),
        "the order is one the scenario placed: {refused:?}"
    );
    assert!(
        matches!(refused.get("carrier"), Some(ScenarioValue::Literal { .. })),
        "the carrier is one no row carries: {refused:?}"
    );
    let shipped = shipping(scenario(&result, SHIPPED));
    for field in ["order_id", "carrier"] {
        assert!(
            matches!(shipped.get(field), Some(ScenarioValue::Instance { .. })),
            "{field} names an arranged row: {shipped:?}"
        );
    }
}

#[test]
fn issue_211_the_illegal_move_family_refuses_by_name_without_a_related_row() {
    let result = synthesis();
    assert!(
        !result
            .suite
            .scenarios
            .keys()
            .any(|key| key.to_string() == ILLEGAL),
        "no illegal-move scenario is sent for a carrier nobody arranged"
    );
    let named: Vec<String> = result
        .refusals
        .iter()
        .map(|refusal| format!("{refusal:?} {}", refusal.cause))
        .filter(|text| text.contains("Shipped") && text.contains("ShipOrder"))
        .collect();
    assert!(
        named
            .iter()
            .any(|text| text.contains("arrange_related_row")),
        "the refusal names the strategy: {named:#?}"
    );
}

// ---- the suite, run ------------------------------------------------------------------------

fn text(node: Option<&Node>) -> String {
    match node {
        Some(Node::Text(value)) => value.clone(),
        other => panic!("expected text, got {other:?}"),
    }
}

fn branch(command: &CommandRef, name: &str) -> OutcomeRef {
    OutcomeRef::new(command.clone(), OutcomeName::new(name).unwrap())
}

#[derive(Default)]
struct Store {
    minted: Cell<u64>,
    carriers: RefCell<Vec<String>>,
    orders: RefCell<BTreeMap<String, String>>,
    published: RefCell<Vec<ObservedEvent>>,
}

/// A shipping service that checks the carrier, or — as a mutant — does not.
struct Shipping {
    checks_carrier: bool,
    store: Store,
}

impl Shipping {
    fn new(checks_carrier: bool) -> Self {
        Self {
            checks_carrier,
            store: Store::default(),
        }
    }

    fn mint(&self) -> String {
        self.store.minted.set(self.store.minted.get() + 1);
        format!("00000000-0000-4000-8000-{:012}", self.store.minted.get())
    }
}

impl ConformanceTarget for Shipping {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("shipping-fixture", "1"))
    }
    fn begin_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        self.store.carriers.replace(Vec::new());
        self.store.orders.replace(BTreeMap::new());
        self.store.published.replace(Vec::new());
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
        let event = |name: &str, field: &str, id: &str| {
            ObservedEvent::new(name.parse::<EventRef>().unwrap())
                .with(field, Node::Text(id.to_owned()))
        };
        let result = match command.to_string().as_str() {
            "demo.ship.RegisterCarrier" => {
                let id = self.mint();
                self.store.carriers.borrow_mut().push(id.clone());
                SemanticCommandResult::took(branch(&command, "registered")).emitting(event(
                    "demo.ship.CarrierRegistered",
                    "carrier_id",
                    &id,
                ))
            }
            "demo.ship.PlaceOrder" => {
                let id = self.mint();
                self.store
                    .orders
                    .borrow_mut()
                    .insert(id.clone(), "Placed".to_owned());
                SemanticCommandResult::took(branch(&command, "placed")).emitting(event(
                    "demo.ship.OrderPlaced",
                    "order_id",
                    &id,
                ))
            }
            "demo.ship.ShipOrder" => {
                let id = text(request.input.get("order_id"));
                let carrier = text(request.input.get("carrier"));
                let held = self.store.orders.borrow().get(&id).cloned();
                if self.checks_carrier && !self.store.carriers.borrow().contains(&carrier) {
                    SemanticCommandResult::took(branch(&command, "no-carrier")).with_error(
                        DeclaredErrorValue::new("demo.ship.NoCarrier".parse::<ErrorRef>().unwrap()),
                    )
                } else if held.as_deref() == Some("Placed") {
                    self.store
                        .orders
                        .borrow_mut()
                        .insert(id.clone(), "Shipped".to_owned());
                    SemanticCommandResult::took(branch(&command, "shipped")).emitting(event(
                        "demo.ship.OrderShipped",
                        "order_id",
                        &id,
                    ))
                } else {
                    SemanticCommandResult::undeclared()
                }
            }
            _ => SemanticCommandResult::undeclared(),
        };
        for published in &result.direct_events {
            self.store.published.borrow_mut().push(published.clone());
        }
        self.store.minted.set(self.store.minted.get() + 1);
        Ok(result.with_consistency(
            ess_primitives::consistency::ConsistencyToken::new(format!(
                "seq:{}",
                self.store.minted.get()
            ))
            .unwrap(),
        ))
    }
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        let rows: Vec<BTreeMap<String, Node>> = match request.view.to_string().as_str() {
            "demo.ship.Carriers" => self
                .store
                .carriers
                .borrow()
                .iter()
                .map(|id| {
                    BTreeMap::from([
                        ("carrier_id".to_owned(), Node::Text(id.clone())),
                        ("state".to_owned(), Node::Text("Active".to_owned())),
                    ])
                })
                .collect(),
            "demo.ship.Orders" => self
                .store
                .orders
                .borrow()
                .iter()
                .map(|(id, state)| {
                    BTreeMap::from([
                        ("order_id".to_owned(), Node::Text(id.clone())),
                        ("state".to_owned(), Node::Text(state.clone())),
                    ])
                })
                .collect(),
            other => panic!("no view {other}"),
        };
        Ok(SemanticViewResult::of(rows))
    }
    fn observe_events(
        &self,
        request: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        Ok(self
            .store
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

fn statuses(result: &Synthesis, target: &Shipping) -> BTreeMap<String, String> {
    let admitted =
        AdmittedSuite::from_suite(&result.suite).unwrap_or_else(|error| panic!("{error}"));
    Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, target)
        .into_report()
        .scenarios
        .into_iter()
        .filter(|run| run.scenario.to_string().contains("ShipOrder"))
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

#[test]
fn issue_211_a_target_checking_the_carrier_passes_and_one_ignoring_it_fails() {
    let result = synthesis();
    let correct = statuses(&result, &Shipping::new(true));
    for id in [NO_CARRIER, SHIPPED] {
        assert_eq!(
            correct.get(id).map(String::as_str),
            Some("passed"),
            "{correct:#?}"
        );
    }
    assert!(
        correct.values().all(|status| status == "passed"),
        "{correct:#?}"
    );
    let ignoring = statuses(&result, &Shipping::new(false));
    assert_ne!(
        ignoring.get(NO_CARRIER).map(String::as_str),
        Some("passed"),
        "{ignoring:#?}"
    );
}

// ---- related predicate refusal beside wrong_state (beyond10x/ess#282, `ess/22`) -------------

const ISSUE_282_RELATED: &str = "demo.release.PublishRelease/outcome/not-accepted";
const ISSUE_282_WRONG: &str =
    "demo.release.Release/state/Published/refuses/demo.release.PublishRelease";

fn issue_282_source() -> String {
    let source = include_str!("fixtures/related-guard-release.yaml")
        .replace("format: ess/20", "format: ess/22")
        .replace(
            "  - {name: demo.release.CandidateNotAccepted, summary: The candidate is not accepted., fields: []}\n",
            "  - {name: demo.release.CandidateNotAccepted, summary: The candidate is not accepted., fields: []}\n  - {name: demo.release.ReleaseStateConflict, summary: The release cannot move from its held state., fields: []}\n",
        )
        .replace(
            "      - name: published\n",
            "      - {name: wrong-state, wrong_state: true, error: demo.release.ReleaseStateConflict}\n      - name: published\n",
        );
    assert!(source.contains("format: ess/22"));
    assert!(source.contains("wrong_state: true"));
    source
}

fn issue_282_acceptance_first_source() -> String {
    let source = issue_282_source();
    let published = "      - name: published\n        moves: demo.release.Release.publish\n        instance: release_id\n        emits: [demo.release.ReleasePublished]\n        payload: {demo.release.ReleasePublished: {release_id: input.release_id}}\n";
    let source = source.replacen(published, "", 1);
    source.replacen(
        "      - name: not-accepted\n",
        "      - name: published\n        when: true\n        moves: demo.release.Release.publish\n        instance: release_id\n        emits: [demo.release.ReleasePublished]\n        payload: {demo.release.ReleasePublished: {release_id: input.release_id}}\n      - name: not-accepted\n",
        1,
    )
}

fn issue_282_synthesis() -> (EssIr, Synthesis) {
    let model = ir(&issue_282_acceptance_first_source());
    let result = ess_conformance::synthesize::synthesize(&model);
    (model, result)
}

fn issue_282_statuses(
    result: &Synthesis,
    target: &impl ConformanceTarget,
) -> BTreeMap<String, Status> {
    let admitted =
        AdmittedSuite::from_suite(&result.suite).unwrap_or_else(|error| panic!("{error}"));
    Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, target)
        .into_report()
        .scenarios
        .into_iter()
        .map(|run| (run.scenario.to_string(), run.status))
        .collect()
}

#[test]
fn issue_282_synthesis_witnesses_both_lifecycle_and_related_refusals() {
    let (model, result) = issue_282_synthesis();
    for id in [ISSUE_282_RELATED, ISSUE_282_WRONG] {
        scenario(&result, id);
    }
    let statuses = issue_282_statuses(
        &result,
        &ess_conformance::interpret::Interpreted::for_model(model),
    );
    for id in [ISSUE_282_RELATED, ISSUE_282_WRONG] {
        assert_eq!(
            statuses.get(id),
            Some(&Status::Passed),
            "{id}: {statuses:#?}"
        );
    }
}

/// The model interpreter with exactly the disputed order swapped: whenever the addressed row would
/// answer `wrong_state`, this target answers the present-related predicate refusal instead.
struct RelatedBeforeWrongState {
    inner: ess_conformance::interpret::Interpreted,
}

impl ConformanceTarget for RelatedBeforeWrongState {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new(
            "related-before-wrong-state",
            "1",
        ))
    }

    fn begin_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.inner.begin_scenario(scenario)
    }

    fn end_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.inner.end_scenario(scenario)
    }

    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        let command = request.command.clone();
        let mut result = self.inner.execute_command(request)?;
        if result
            .outcome
            .as_ref()
            .is_some_and(|outcome| outcome.to_string().ends_with("/wrong-state"))
        {
            result.outcome = Some(branch(&command, "not-accepted"));
            result.error = Some(DeclaredErrorValue::new(
                "demo.release.CandidateNotAccepted"
                    .parse::<ErrorRef>()
                    .unwrap(),
            ));
        }
        Ok(result)
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
        request: ExternalOutcomeControl,
    ) -> Result<(), TargetError> {
        self.inner.configure_external_outcome(request)
    }

    fn redeliver_event(&self, request: RedeliveryRequest) -> Result<(), TargetError> {
        self.inner.redeliver_event(request)
    }
}

#[test]
fn issue_282_wrong_precedence_target_fails_its_suite() {
    let (model, result) = issue_282_synthesis();
    let statuses = issue_282_statuses(
        &result,
        &RelatedBeforeWrongState {
            inner: ess_conformance::interpret::Interpreted::for_model(model),
        },
    );
    assert_eq!(
        statuses.get(ISSUE_282_WRONG),
        Some(&Status::Failed),
        "the wrong-state scenario distinguishes the swapped order: {statuses:#?}"
    );
}
