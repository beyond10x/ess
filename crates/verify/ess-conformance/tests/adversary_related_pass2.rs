//! Adversary, pass 2, on `{related: {via: <field>, field: <field>}}` (ess/16, beyond10x/ess#166,
//! `docs/design/value-expressions.md` E8).
//!
//! The pass-1 fixture (the unit's #166 document) with two variations: the read field typed as a
//! three-valued enum, and a command that changes the customer's region.

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

/// `Region` as a closed set of three names. The witness cycles enum variants by distinction
/// (`variants[n % 3]`), and the referenced row sits at `256` with its decoys at `257` and `259`:
/// `259 % 3 == 256 % 3`, so the second decoy holds the referenced row's region.
fn three_regions() -> String {
    let before = "{name: demo.shipping.Region, kind: newtype, of: String}";
    assert!(SHIPPING.contains(before));
    SHIPPING.replace(
        before,
        "{name: demo.shipping.Region, kind: enum, variants: [North, South, West]}",
    )
}

/// The #166 document with a `Relocate` command that changes a customer's region — the ordinary way
/// the value `{related: …}` reads "as it was immediately before the outcome" can differ from what
/// it was when the shipment was packed.
fn relocatable() -> String {
    let mut text = SHIPPING.replace(
        "    may: [demo.shipping.Register, demo.shipping.Pack, demo.shipping.Dispatch]",
        "    may: [demo.shipping.Register, demo.shipping.Pack, demo.shipping.Dispatch, demo.shipping.Relocate]",
    );
    text = text.replace(
        "  - name: demo.shipping.ShipmentPacked\n",
        "  - name: demo.shipping.CustomerRelocated
    fields:
      - {name: customer_id, type: demo.shipping.CustomerId}
  - name: demo.shipping.ShipmentPacked\n",
    );
    text = text.replace(
        "  - name: demo.shipping.Pack\n",
        "  - name: demo.shipping.Relocate
    input:
      - {name: customer_id, type: demo.shipping.CustomerId}
      - {name: region, type: demo.shipping.Region}
    outcomes:
      - name: relocated
        updates: demo.shipping.Customer
        instance: customer_id
        emits: [demo.shipping.CustomerRelocated]
        payload:
          demo.shipping.CustomerRelocated: {customer_id: input.customer_id}
        sets:
          region: input.region
  - name: demo.shipping.Pack\n",
    );
    assert!(text.contains("demo.shipping.Relocate\n    input"));
    text
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
    /// Reads the region of the customer the shipment names, when it is asked.
    Correct,
    /// Reads the region of the customer registered last, whichever the shipment names.
    LatestCustomer,
    /// Copies the customer's region onto the shipment when it is packed, and `Dispatch` emits that
    /// copy: the right customer, read at the wrong time.
    SnapshotAtPack,
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

/// Shipment columns the view does not publish.
const HIDDEN: [&str; 2] = ["customer_id", "packed_region"];

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
        world.customers.iter().find(named)?;
        let found = match self.mode {
            Mode::Correct | Mode::SnapshotAtPack => world.customers.iter().find(named),
            Mode::LatestCustomer => world.customers.last(),
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
            "demo.shipping.Relocate" => {
                let id = request.input["customer_id"].clone();
                let Some(row) = world
                    .customers
                    .iter_mut()
                    .find(|(customer, _)| Node::Text(customer.clone()) == id)
                else {
                    return Ok(SemanticCommandResult::undeclared());
                };
                row.1 = request.input["region"].clone();
                SemanticCommandResult::took(outcome(&command, "relocated")).emitting(
                    ObservedEvent::new("demo.shipping.CustomerRelocated".parse().unwrap())
                        .with("customer_id", id),
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
                        ("packed_region".to_owned(), region.clone()),
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
                let Some(stored) = world.shipments.get(&id).cloned() else {
                    return Ok(SemanticCommandResult::undeclared());
                };
                let region = if self.mode == Mode::SnapshotAtPack {
                    stored["packed_region"].clone()
                } else {
                    let Some(region) = self.region(&world, &stored["customer_id"]) else {
                        return Ok(SemanticCommandResult::undeclared());
                    };
                    region
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
                    .filter(|(name, _)| !HIDDEN.contains(&name.as_str()))
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

/// The target is sound on both variations: the correct implementation passes each.
#[test]
fn adversary_related_pass2_the_correct_implementation_passes_both_variations() {
    assert_eq!(
        failing(&suite(&three_regions()), Mode::Correct),
        Vec::<String>::new()
    );
    assert_eq!(
        failing(&suite(&relocatable()), Mode::Correct),
        Vec::<String>::new()
    );
}

/// E8 claims the decoys' "odd distinctions against the referenced row's even one" make their
/// fields hold other values. For a three-valued enum the second decoy (`259`) cycles back onto the
/// referenced row's variant (`256`), so an implementation reading the customer registered last
/// emits the right region by accident and passes. Decoys at `first - 1` and `first + 1` differ
/// from `first` under every period of two or more.
#[test]
fn adversary_related_pass2_a_three_valued_enum_still_catches_the_latest_customer() {
    assert_eq!(
        failing(&suite(&three_regions()), Mode::LatestCustomer),
        vec![DISPATCH.to_owned(), PACK.to_owned()]
    );
}

/// The control for the case above: with four variants (`256 % 4`, `257 % 4`, `259 % 4` all
/// differ) the same implementation fails both branches, so the enum region is asserted and what
/// lets the three-valued one through is the decoys' distinctions alone.
#[test]
fn adversary_related_pass2_a_four_valued_enum_catches_the_latest_customer() {
    let four = three_regions().replace("[North, South, West]", "[North, South, West, East]");
    assert_eq!(
        failing(&suite(&four), Mode::LatestCustomer),
        vec![DISPATCH.to_owned(), PACK.to_owned()]
    );
}

/// E8 reads the referenced row "as it was immediately before the outcome". Where the document
/// declares a way to change that field, nothing between the subject's creation and the branch
/// changes it, so an implementation that copied the customer's region at `Pack` and emits the copy
/// at `Dispatch` passes.
#[test]
fn adversary_related_pass2_a_region_copied_at_pack_time_fails_dispatch() {
    assert_eq!(
        failing(&suite(&relocatable()), Mode::SnapshotAtPack),
        vec![DISPATCH.to_owned()]
    );
}
