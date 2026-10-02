//! `{related: {via: <field>, field: <field>}}` (source format `ess/16`, beyond10x/ess#166,
//! `docs/design/value-expressions.md` E8): the scenario arranges two rows of the referenced entity,
//! points the subject at one of them, and asserts that row's value, so an implementation that reads
//! the other row fails.

use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;

use ess_compiler::ir::{EssIr, ResolvedPayloadValue};
use ess_compiler::refs::{CommandRef, OutcomeRef};
use ess_compiler::{resolve::compile, source::SourceMap};
use ess_conformance::report::Status;
use ess_conformance::scenario::ViewExpectation;
use ess_conformance::target::*;
use ess_conformance::{
    AdmittedSuite, ConformanceScenario, ConformanceSuite, Runner, ScenarioStep, ScenarioValue,
};
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

#[test]
fn the_native_interpreter_executes_the_shipping_suite() {
    let model = ir(SHIPPING);
    let synthesis = ess_conformance::synthesize::synthesize(&model);
    assert!(synthesis.refusals.is_empty(), "{:?}", synthesis.refusals);
    let admitted = AdmittedSuite::from_suite(&synthesis.suite).unwrap();
    let run = Runner::for_suite(admitted.suite()).run_admitted(
        &admitted,
        &ess_conformance::interpret::Interpreted::for_model(model),
    );
    assert!(!run.scenarios.is_empty());
    assert!(
        run.scenarios
            .iter()
            .all(|result| result.status == Status::Passed),
        "{:#?}",
        run.scenarios
    );
}

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

fn scenario<'a>(suite: &'a ConformanceSuite, id: &str) -> &'a ConformanceScenario {
    suite
        .scenarios
        .iter()
        .find(|(key, _)| key.to_string() == id)
        .map_or_else(|| panic!("no scenario {id}"), |(_, scenario)| scenario)
}

/// Every `Register` the scenario runs: the instance it captures and the region it sends, in order.
fn customers(scenario: &ConformanceScenario) -> Vec<(String, Node)> {
    let mut out = Vec::new();
    let mut region = None;
    for step in &scenario.steps {
        match step {
            ScenarioStep::ExecuteCommand { command, input, .. }
                if command.to_string() == "demo.shipping.Register" =>
            {
                region = match input.get("region") {
                    Some(ScenarioValue::Literal { value }) => Some(value.clone()),
                    other => panic!("the region is a literal: {other:?}"),
                };
            }
            ScenarioStep::CaptureInstance {
                instance, entity, ..
            } if entity.to_string() == "demo.shipping.Customer" => {
                out.push((
                    instance.to_string(),
                    region.take().expect("a capture follows its Register"),
                ));
            }
            _ => {}
        }
    }
    out
}

/// The instance the scenario's `Pack` names as its customer.
fn packed_for(scenario: &ConformanceScenario) -> String {
    scenario
        .steps
        .iter()
        .find_map(|step| match step {
            ScenarioStep::ExecuteCommand { command, input, .. }
                if command.to_string() == "demo.shipping.Pack" =>
            {
                match input.get("customer_id") {
                    Some(ScenarioValue::Instance { instance }) => Some(instance.to_string()),
                    other => panic!("Pack names an arranged customer: {other:?}"),
                }
            }
            _ => None,
        })
        .expect("the scenario packs a shipment")
}

/// The value the scenario's expectation of `event` asserts for `region`.
fn asserted_region(scenario: &ConformanceScenario, event: &str) -> Option<Node> {
    scenario
        .steps
        .iter()
        .find_map(|step| match step {
            ScenarioStep::ExpectEvent {
                event: seen,
                payload,
                ..
            } if seen.to_string() == event => Some(payload.get("region").cloned()),
            _ => None,
        })
        .unwrap_or_else(|| panic!("`{event}` is expected"))
}

/// The referenced customer's region, after checking that three customers are arranged, the referenced
/// one in the middle, and that each other holds a region it does not: an implementation reading the
/// first row or the last row of the entity reads a decoy.
fn between_decoys<'a>(customers: &'a [(String, Node)], referenced: &str) -> &'a Node {
    assert_eq!(customers.len(), 3, "three customers: {customers:?}");
    assert_eq!(
        customers[1].0, referenced,
        "the middle one is referenced: {customers:?}"
    );
    let region = &customers[1].1;
    assert_ne!(
        &customers[0].1, region,
        "the first decoy holds another region"
    );
    assert_ne!(
        &customers[2].1, region,
        "the last decoy holds another region"
    );
    region
}

#[test]
fn issue_166_the_compiled_source_names_the_referenced_entity_and_its_field() {
    let ir = ir(SHIPPING);
    let dispatch = ir
        .commands()
        .values()
        .find(|command| command.name.to_string() == "demo.shipping.Dispatch")
        .expect("Dispatch compiles");
    let region = dispatch.outcomes[0].payload[0]
        .fields
        .iter()
        .find(|field| field.target == "region")
        .expect("region is determined");
    let ResolvedPayloadValue::RelatedField { entity, field, .. } = &region.value else {
        panic!("a related source: {:?}", region.value)
    };
    assert_eq!(entity.name().to_string(), "demo.shipping.Customer");
    assert_eq!(field, "region");
}

#[test]
fn issue_166_the_referenced_customer_sits_between_two_decoys_and_its_region_is_asserted() {
    let suite = suite(SHIPPING);
    let dispatch = scenario(&suite, DISPATCH);
    let customers = customers(dispatch);
    let region = between_decoys(&customers, &packed_for(dispatch));
    assert_eq!(
        asserted_region(dispatch, "demo.shipping.ShipmentDispatched").as_ref(),
        Some(region)
    );
    let row = dispatch
        .steps
        .iter()
        .rev()
        .find_map(|step| match step {
            ScenarioStep::ExpectView {
                expectation: ViewExpectation::Contains { fields },
                ..
            } => Some(fields.clone()),
            _ => None,
        })
        .expect("a row is required");
    assert_eq!(
        row.get("region"),
        Some(&ScenarioValue::Literal {
            value: region.clone()
        }),
        "{row:?}"
    );
}

#[test]
fn issue_166_a_related_source_through_the_input_is_asserted_on_creation() {
    let suite = suite(SHIPPING);
    let pack = scenario(&suite, PACK);
    let customers = customers(pack);
    let region = between_decoys(&customers, &packed_for(pack));
    assert_eq!(
        asserted_region(pack, "demo.shipping.ShipmentPacked").as_ref(),
        Some(region)
    );
}

#[test]
fn a_suite_without_a_related_source_keeps_its_arrangement() {
    // Only a branch that reads a related row arranges one: `Register` runs once for `Pack` when
    // `Pack` reads nothing of the customer.
    let text = SHIPPING
        .replace(
            "region: {related: {via: input.customer_id, field: region}}",
            "region: {generated: true}",
        )
        .replace(
            "            region: {related: {via: customer_id, field: region}}\n        sets:\n          region: {related: {via: customer_id, field: region}}",
            "            region: {generated: true}",
        );
    let suite = suite(&text);
    for id in [PACK, DISPATCH] {
        assert!(customers(scenario(&suite, id)).is_empty(), "{id}");
    }
}

// ---- running the suite ------------------------------------------------------------------------

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

/// Every scenario that did not pass against `mode`, by id.
fn failing(suite: &ConformanceSuite, mode: Mode) -> Vec<String> {
    let admitted = AdmittedSuite::from_suite(suite).unwrap_or_else(|error| panic!("{error}"));
    Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &Shipping::new(mode))
        .into_report()
        .scenarios
        .into_iter()
        .filter(|result| result.status != Status::Passed)
        .map(|result| result.scenario.to_string())
        .collect()
}

#[test]
fn issue_166_the_suite_passes_an_implementation_reading_the_referenced_customer() {
    assert_eq!(
        failing(&suite(SHIPPING), Mode::Correct),
        Vec::<String>::new()
    );
}

#[test]
fn issue_166_an_implementation_emitting_the_other_customers_region_fails() {
    let mut failed = failing(&suite(SHIPPING), Mode::OtherCustomer);
    failed.sort();
    assert_eq!(failed, vec![DISPATCH.to_owned(), PACK.to_owned()]);
}

#[test]
fn issue_166_an_implementation_reading_the_latest_customer_fails() {
    let mut failed = failing(&suite(SHIPPING), Mode::LatestCustomer);
    failed.sort();
    assert_eq!(failed, vec![DISPATCH.to_owned(), PACK.to_owned()]);
}
