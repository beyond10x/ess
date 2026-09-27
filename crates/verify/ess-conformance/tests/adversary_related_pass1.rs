//! Adversary, pass 1, on `{related: {via: <field>, field: <field>}}` (ess/16, beyond10x/ess#166,
//! `docs/design/value-expressions.md` E8).
//!
//! The fixture and the in-memory target are the unit's own (`tests/related_values.rs`), with two
//! further implementations and one further document.

use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;

use ess_compiler::ir::EssIr;
use ess_compiler::refs::{CommandRef, OutcomeRef};
use ess_compiler::{resolve::compile, source::SourceMap};
use ess_conformance::report::Status;
use ess_conformance::target::*;
use ess_conformance::{AdmittedSuite, ConformanceSuite, Runner};
use ess_domain::{command::OutcomeName, spec::RawSpecFile, system::Source, Specification};
use ess_primitives::node::Node;

/// The unit's #166 fixture, unchanged.
const SHIPPING: &str = "format: ess/16
system: demo
version: v1
domain: demo.shipping
types:
  - {name: demo.shipping.CustomerId, kind: newtype, of: String}
  - {name: demo.shipping.ShipmentId, kind: newtype, of: String}
  - {name: demo.shipping.Region, kind: newtype, of: String}
entities:
  - name: demo.shipping.Customer
    identity: {name: customer_id, type: demo.shipping.CustomerId}
    fields:
      - {name: region, type: demo.shipping.Region}
    lifecycle: {initial: Active, states: [Active], terminal: [Active]}
  - name: demo.shipping.Shipment
    identity: {name: shipment_id, type: demo.shipping.ShipmentId}
    fields:
      - {name: customer_id, type: demo.shipping.CustomerId}
      - {name: region, type: Optional<demo.shipping.Region>}
    lifecycle: {initial: Packed, states: [Packed], terminal: [Packed]}
events:
  - name: demo.shipping.CustomerRegistered
    fields:
      - {name: customer_id, type: demo.shipping.CustomerId}
  - name: demo.shipping.ShipmentPacked
    fields:
      - {name: shipment_id, type: demo.shipping.ShipmentId}
      - {name: region, type: demo.shipping.Region}
  - name: demo.shipping.ShipmentDispatched
    fields:
      - {name: shipment_id, type: demo.shipping.ShipmentId}
      - {name: region, type: demo.shipping.Region}
actors:
  - name: demo.shipping.Clerk
    may: [demo.shipping.Register, demo.shipping.Pack, demo.shipping.Dispatch]
commands:
  - name: demo.shipping.Register
    input:
      - {name: region, type: demo.shipping.Region}
    outcomes:
      - name: registered
        creates: demo.shipping.Customer
        instance: customer_id
        emits: [demo.shipping.CustomerRegistered]
        payload:
          demo.shipping.CustomerRegistered: {customer_id: {generated: true}}
        sets:
          region: input.region
  - name: demo.shipping.Pack
    input:
      - {name: customer_id, type: demo.shipping.CustomerId}
    outcomes:
      - name: packed
        creates: demo.shipping.Shipment
        instance: shipment_id
        emits: [demo.shipping.ShipmentPacked]
        payload:
          demo.shipping.ShipmentPacked:
            shipment_id: {generated: true}
            region: {related: {via: input.customer_id, field: region}}
        sets:
          customer_id: input.customer_id
  - name: demo.shipping.Dispatch
    input:
      - {name: shipment_id, type: demo.shipping.ShipmentId}
    outcomes:
      - name: dispatched
        updates: demo.shipping.Shipment
        instance: shipment_id
        emits: [demo.shipping.ShipmentDispatched]
        payload:
          demo.shipping.ShipmentDispatched:
            shipment_id: input.shipment_id
            region: {related: {via: customer_id, field: region}}
        sets:
          region: {related: {via: customer_id, field: region}}
views:
  - name: demo.shipping.Shipments
    source: demo.shipping.Shipment
    consistency: read_your_writes
    fields:
      - {name: shipment_id, type: demo.shipping.ShipmentId}
      - {name: region, type: Optional<demo.shipping.Region>}
";

/// The same document with the customer declaring that it owns its shipments through
/// `Shipment.customer_id` — the ordinary way to say a shipment cannot exist without its customer.
fn owned() -> String {
    SHIPPING.replace(
        "      - {name: region, type: demo.shipping.Region}
    lifecycle: {initial: Active, states: [Active], terminal: [Active]}",
        "      - {name: region, type: demo.shipping.Region}
    relations:
      - {name: shipments, kind: owns, target: demo.shipping.Shipment, cardinality: many, via: customer_id}
    lifecycle: {initial: Active, states: [Active], terminal: [Active]}",
    )
}

const DISPATCH: &str = "demo.shipping.Dispatch/outcome/dispatched";
const PACK: &str = "demo.shipping.Pack/outcome/packed";

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("shipping.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn suite(text: &str) -> ConformanceSuite {
    let synthesis = ess_conformance::synthesize::synthesize(&ir(text));
    assert!(
        synthesis.refusals.is_empty(),
        "nothing is refused: {:#?}",
        synthesis.refusals
    );
    synthesis.suite
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Mode {
    /// Reads the region of the customer the shipment names.
    Correct,
    /// Reads the region of the customer registered last, whichever the shipment names.
    LatestCustomer,
    /// Reads the region of the customer registered first, whichever the shipment names: the
    /// lookup that lost its `WHERE customer_id = ?` and returns the table's first row.
    FirstCustomer,
}

#[derive(Default)]
struct World {
    customers: Vec<(String, Node)>,
    shipments: BTreeMap<String, BTreeMap<String, Node>>,
}

struct Shipping {
    mode: Mode,
    world: RefCell<World>,
    minted: Cell<u32>,
}

impl Shipping {
    fn new(mode: Mode) -> Self {
        Self {
            mode,
            world: RefCell::default(),
            minted: Cell::new(0),
        }
    }

    fn region(&self, world: &World, id: &Node) -> Option<Node> {
        let named = |(customer, _): &&(String, Node)| Node::Text(customer.clone()) == *id;
        // Every mode still refuses a customer that does not exist, so only the row read differs.
        world.customers.iter().find(named)?;
        let found = match self.mode {
            Mode::Correct => world.customers.iter().find(named),
            Mode::LatestCustomer => world.customers.last(),
            Mode::FirstCustomer => world.customers.first(),
        };
        found.map(|(_, region)| region.clone())
    }
}

fn outcome(command: &CommandRef, name: &str) -> OutcomeRef {
    OutcomeRef::new(command.clone(), OutcomeName::new(name).unwrap())
}

impl ConformanceTarget for Shipping {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("shipping-fixture", "1"))
    }
    fn begin_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        self.world.replace(World::default());
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
        let n = self.minted.get();
        let token = ess_primitives::consistency::ConsistencyToken::new(format!("seq:{n}")).unwrap();
        let command = request.command.clone();
        let mut world = self.world.borrow_mut();
        let result = match command.to_string().as_str() {
            "demo.shipping.Register" => {
                let id = format!("customer-{n}");
                world
                    .customers
                    .push((id.clone(), request.input["region"].clone()));
                SemanticCommandResult::took(outcome(&command, "registered")).emitting(
                    ObservedEvent::new("demo.shipping.CustomerRegistered".parse().unwrap())
                        .with("customer_id", Node::Text(id)),
                )
            }
            "demo.shipping.Pack" => {
                let customer = request.input["customer_id"].clone();
                let Some(region) = self.region(&world, &customer) else {
                    return Ok(SemanticCommandResult::undeclared());
                };
                let id = format!("shipment-{n}");
                world.shipments.insert(
                    id.clone(),
                    BTreeMap::from([
                        ("shipment_id".to_owned(), Node::Text(id.clone())),
                        ("customer_id".to_owned(), customer),
                        ("region".to_owned(), Node::Null),
                    ]),
                );
                SemanticCommandResult::took(outcome(&command, "packed")).emitting(
                    ObservedEvent::new("demo.shipping.ShipmentPacked".parse().unwrap())
                        .with("shipment_id", Node::Text(id))
                        .with("region", region),
                )
            }
            "demo.shipping.Dispatch" => {
                let Some(Node::Text(id)) = request.input.get("shipment_id").cloned() else {
                    return Ok(SemanticCommandResult::undeclared());
                };
                let Some(customer) = world
                    .shipments
                    .get(&id)
                    .map(|row| row["customer_id"].clone())
                else {
                    return Ok(SemanticCommandResult::undeclared());
                };
                let Some(region) = self.region(&world, &customer) else {
                    return Ok(SemanticCommandResult::undeclared());
                };
                let row = world.shipments.get_mut(&id).expect("looked up above");
                row.insert("region".to_owned(), region.clone());
                SemanticCommandResult::took(outcome(&command, "dispatched")).emitting(
                    ObservedEvent::new("demo.shipping.ShipmentDispatched".parse().unwrap())
                        .with("shipment_id", Node::Text(id))
                        .with("region", region),
                )
            }
            other => panic!("unexpected command {other}"),
        };
        Ok(result.with_consistency(token))
    }
    fn query_view(&self, _: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        Ok(SemanticViewResult::of(
            self.world.borrow().shipments.values().map(|row| {
                row.iter()
                    .filter(|(name, _)| name.as_str() != "customer_id")
                    .map(|(name, value)| (name.clone(), value.clone()))
                    .collect()
            }),
        ))
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

fn failing(suite: &ConformanceSuite, mode: Mode) -> Vec<String> {
    let admitted = AdmittedSuite::from_suite(suite).unwrap_or_else(|error| panic!("{error}"));
    let mut failed: Vec<String> = Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &Shipping::new(mode))
        .into_report()
        .scenarios
        .into_iter()
        .filter(|result| result.status != Status::Passed)
        .map(|result| result.scenario.to_string())
        .collect();
    failed.sort();
    failed
}

/// The target is sound on both documents: the correct implementation passes each.
#[test]
fn adversary_related_the_correct_implementation_passes_both_documents() {
    assert_eq!(
        failing(&suite(SHIPPING), Mode::Correct),
        Vec::<String>::new()
    );
    assert_eq!(
        failing(&suite(&owned()), Mode::Correct),
        Vec::<String>::new()
    );
}

/// The acceptance: "fails an implementation that emits the other row's value". The decoy is always
/// arranged *after* the referenced row, so an implementation reading the first row of the table —
/// the lookup with its key predicate dropped — reads the referenced row by accident and passes.
#[test]
fn adversary_related_an_implementation_reading_the_first_customer_fails() {
    assert_eq!(
        failing(&suite(SHIPPING), Mode::FirstCustomer),
        vec![DISPATCH.to_owned(), PACK.to_owned()]
    );
}

/// With `Customer owns Shipment via customer_id`, `Pack`'s input is bound to the arranged owner,
/// `related::point_at` declines to bind it again, and `Pack`'s related read is left undetermined:
/// one customer, no region asserted, and an implementation reading any customer passes `Pack`.
#[test]
fn adversary_related_an_owned_subject_still_asserts_the_input_read() {
    assert_eq!(
        failing(&suite(&owned()), Mode::LatestCustomer),
        vec![DISPATCH.to_owned(), PACK.to_owned()]
    );
}
