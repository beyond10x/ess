//! The generated Go runtime gives the reference verdicts for `{related: {via, field}}` values
//! (ess/16, beyond10x/ess#166), which need no suite vocabulary of their own (beyond10x/ess#188).
//!
//! The target is `tests/related_values.rs`'s, recorded once and replayed to the Go runtime.

mod support_go;

use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;

use ess_compiler::ir::EssIr;
use ess_compiler::refs::{CommandRef, OutcomeRef};
use ess_compiler::{resolve::compile, source::SourceMap};
use ess_conformance::target::*;
use ess_domain::{command::OutcomeName, spec::RawSpecFile, system::Source, Specification};
use ess_primitives::node::Node;

/// The #166 shape: dispatching a shipment emits (and stores) the region of the customer the
/// shipment was packed for; packing emits the region of the customer the input names.
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

const DISPATCH: &str = "demo.shipping.Dispatch/outcome/dispatched";
const PACK: &str = "demo.shipping.Pack/outcome/packed";

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("shipping.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

#[test]
fn go_gives_the_reference_verdict_for_every_related_read() {
    let synthesis = ess_conformance::synthesize::synthesize(&ir(SHIPPING));
    assert!(synthesis.refusals.is_empty(), "{:#?}", synthesis.refusals);
    let suite = synthesis.suite;
    for (mode, expected) in [
        (Mode::Correct, vec![]),
        (Mode::LatestCustomer, vec![DISPATCH, PACK]),
        (Mode::OtherCustomer, vec![DISPATCH, PACK]),
    ] {
        let verdicts = support_go::assert_parity(
            &format!("related-{mode:?}").to_lowercase(),
            &suite,
            Shipping::new(mode),
        );
        assert_eq!(support_go::not_passed(&verdicts), expected, "{mode:?}");
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Mode {
    /// Reads the region of the customer the shipment names.
    Correct,
    /// Reads the region of the customer registered last, whichever the shipment names.
    LatestCustomer,
    /// Reads the region of a customer the shipment does not name, where there is one.
    OtherCustomer,
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

    /// The region this implementation reads for the customer `id`.
    fn region(&self, world: &World, id: &Node) -> Option<Node> {
        let named = |(customer, _): &&(String, Node)| Node::Text(customer.clone()) == *id;
        let found = match self.mode {
            Mode::Correct => world.customers.iter().find(named),
            Mode::LatestCustomer => world.customers.last(),
            Mode::OtherCustomer => world
                .customers
                .iter()
                .find(|customer| !named(customer))
                .or_else(|| world.customers.iter().find(named)),
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
