//! An aggregate view whose group key the creating command copies from a related row
//! (`{related: …}` in `sets:`) is witnessed (beyond10x/ess#257).
//!
//! Synthesis creates the related row with the key value it wants, then runs the creating command
//! against it. The scenario is run against a hand-written target that answers the model, which
//! must pass it, and against mutants that aggregate under the wrong key, each of which must fail
//! it. `ESS-SYNTH-017` says "does not set" only of a key the creating command sets from nothing.
use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;

use ess_compiler::{
    ir::EssIr,
    refs::{CommandRef, OutcomeRef},
    resolve::compile,
    source::SourceMap,
};
use ess_conformance::{
    report::Status, synthesize::Synthesis, target::*, AdmittedSuite, Runner, ScenarioStep,
    ScenarioValue,
};
use ess_domain::{
    command::OutcomeName,
    spec::{RawSpecFile, Specification},
    system::Source,
};
use ess_primitives::{facts::FactValue, node::Node};

/// Placing an order copies the region of the customer the input names; the view groups orders by
/// that region.
const ORDERS: &str = "format: ess/16
system: demo
version: v1
domain: demo.orders
types:
  - {name: demo.orders.CustomerId, kind: newtype, of: String}
  - {name: demo.orders.OrderId, kind: newtype, of: String}
entities:
  - name: demo.orders.Customer
    identity: {name: customer_id, type: demo.orders.CustomerId}
    fields:
      - {name: region, type: String}
    lifecycle: {initial: Active, states: [Active], terminal: [Active]}
  - name: demo.orders.Order
    identity: {name: order_id, type: demo.orders.OrderId}
    fields:
      - {name: customer_id, type: demo.orders.CustomerId}
      - {name: region, type: String}
      - {name: amount, type: Integer}
    lifecycle: {initial: Placed, states: [Placed], terminal: [Placed]}
events:
  - name: demo.orders.CustomerRegistered
    fields:
      - {name: customer_id, type: demo.orders.CustomerId}
  - name: demo.orders.OrderPlaced
    fields:
      - {name: order_id, type: demo.orders.OrderId}
actors:
  - name: demo.orders.Clerk
    may: [demo.orders.Register, demo.orders.Place]
commands:
  - name: demo.orders.Register
    input:
      - {name: region, type: String}
    outcomes:
      - name: registered
        creates: demo.orders.Customer
        instance: customer_id
        emits: [demo.orders.CustomerRegistered]
        payload:
          demo.orders.CustomerRegistered: {customer_id: {generated: true}}
        sets:
          region: input.region
  - name: demo.orders.Place
    input:
      - {name: customer_id, type: demo.orders.CustomerId}
      - {name: amount, type: Integer}
    outcomes:
      - name: placed
        creates: demo.orders.Order
        instance: order_id
        emits: [demo.orders.OrderPlaced]
        payload:
          demo.orders.OrderPlaced: {order_id: {generated: true}}
        sets:
          customer_id: input.customer_id
          region: {related: {via: input.customer_id, field: region}}
          amount: input.amount
views:
  - name: demo.orders.OrdersByRegion
    source: demo.orders.Order
    consistency: read_your_writes
    group_by: [region]
    fields:
      - {name: region, type: String}
      - {name: orders, type: Integer, aggregate: {count: {}}}
      - {name: total, type: Integer, aggregate: {sum: amount}}
";

const VIEW: &str = "demo.orders.OrdersByRegion";
const AGGREGATE: &str = "demo.orders.OrdersByRegion/aggregate";

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("orders.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn synthesis(text: &str) -> Synthesis {
    ess_conformance::synthesize::synthesize(&ir(text))
}

/// The `ESS-SYNTH-017` refusals of the view, rendered.
fn unwitnessed(result: &Synthesis) -> Vec<String> {
    result
        .refusals
        .iter()
        .filter(|refusal| refusal.code().to_string() == "ESS-SYNTH-017")
        .map(ToString::to_string)
        .filter(|rendered| rendered.contains(VIEW))
        .collect()
}

#[test]
fn a_group_key_copied_from_a_related_row_is_witnessed() {
    let result = synthesis(ORDERS);
    assert_eq!(unwitnessed(&result), Vec::<String>::new());
    let scenario = result
        .suite
        .scenarios
        .iter()
        .find(|(id, _)| id.to_string() == AGGREGATE)
        .map_or_else(
            || panic!("no scenario {AGGREGATE}: {:#?}", result.refusals),
            |(_, scenario)| scenario,
        );
    // Every order names a customer the scenario registered with the key value it wanted, and every
    // customer is registered before the first order.
    let mut registered = Vec::new();
    let mut placed = Vec::new();
    for step in &scenario.steps {
        if let ScenarioStep::ExecuteCommand { command, input, .. } = step {
            match command.to_string().as_str() {
                "demo.orders.Register" => {
                    assert!(placed.is_empty(), "a customer registered after an order");
                    registered.push(input["region"].clone());
                }
                "demo.orders.Place" => {
                    assert!(
                        matches!(input["customer_id"], ScenarioValue::Instance { .. }),
                        "{input:?}"
                    );
                    placed.push(input.clone());
                }
                other => panic!("{other}"),
            }
        }
    }
    assert_eq!(registered.len(), placed.len(), "one customer per order");
    let scoped = |group: &str| ScenarioValue::literal(Node::Text(format!("{VIEW}/{group}")));
    assert!(registered.contains(&scoped("A")), "{registered:?}");
    assert!(registered.contains(&scoped("B")), "{registered:?}");
}

#[test]
fn the_message_says_does_not_set_only_of_a_key_set_from_nothing() {
    // Set from nothing.
    let unset = ORDERS.replace(
        "          region: {related: {via: input.customer_id, field: region}}\n",
        "",
    );
    let refusals = unwitnessed(&synthesis(&unset));
    assert!(
        refusals
            .iter()
            .any(|refusal| refusal.contains("does not set the group key `region`")),
        "{refusals:?}"
    );
    // Set from a related row no arrangement can give a chosen value: the customer's region is a
    // literal.
    let fixed = ORDERS.replace(
        "          region: input.region\n",
        "          region: North\n",
    );
    let refusals = unwitnessed(&synthesis(&fixed));
    assert!(!refusals.is_empty(), "the view is refused");
    assert!(
        refusals
            .iter()
            .all(|refusal| !refusal.contains("does not set")),
        "{refusals:?}"
    );
    assert!(
        refusals.iter().any(|refusal| refusal.contains("`region`")),
        "{refusals:?}"
    );
}

/// One defect an implementation of the view could have.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mutant {
    None,
    /// Groups by the customer the order names rather than the region it copied.
    GroupsByCustomer,
    /// Groups every order into one group.
    IgnoresKey,
    /// Copies the region of the customer registered last, not the one the input names.
    CopiesLatestRegion,
    /// Copies the region of the customer registered first, not the one the input names.
    CopiesFirstRegion,
}

type Row = BTreeMap<String, Node>;

struct Orders {
    mutant: Mutant,
    customers: RefCell<Vec<(Node, Node)>>,
    placed: RefCell<Vec<Row>>,
    minted: Cell<u64>,
}

fn outcome(command: &CommandRef, name: &str) -> OutcomeRef {
    OutcomeRef::new(command.clone(), OutcomeName::new(name).unwrap())
}

fn number(n: i128) -> Node {
    match FactValue::parse_literal(&n.to_string()) {
        FactValue::Number(number) => Node::Number(number),
        other => panic!("{other:?}"),
    }
}

fn int(node: &Node) -> i128 {
    match node {
        Node::Number(number) => i128::from(number.as_i64().expect("an integer")),
        other => panic!("{other:?}"),
    }
}

impl Orders {
    fn view(&self) -> Vec<Row> {
        let key = match self.mutant {
            Mutant::GroupsByCustomer => Some("customer_id"),
            Mutant::IgnoresKey => None,
            _ => Some("region"),
        };
        let mut groups: Vec<(Option<Node>, Vec<Row>)> = Vec::new();
        for order in self.placed.borrow().iter() {
            let held = key.map(|key| order[key].clone());
            match groups.iter_mut().find(|(group, _)| *group == held) {
                Some((_, members)) => members.push(order.clone()),
                None => groups.push((held, vec![order.clone()])),
            }
        }
        groups
            .into_iter()
            .map(|(_, members)| {
                let total: i128 = members.iter().map(|member| int(&member["amount"])).sum();
                Row::from([
                    ("region".to_owned(), members[0]["region"].clone()),
                    ("orders".to_owned(), number(members.len() as i128)),
                    ("total".to_owned(), number(total)),
                ])
            })
            .collect()
    }
}

impl ConformanceTarget for Orders {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("orders-fixture", "1"))
    }
    fn begin_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        self.customers.replace(Vec::new());
        self.placed.replace(Vec::new());
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
        let result = match command.to_string().as_str() {
            "demo.orders.Register" => {
                let id = Node::Text(format!("customer-{n}"));
                self.customers.borrow_mut().push((
                    id.clone(),
                    request.input.get("region").cloned().unwrap_or(Node::Null),
                ));
                SemanticCommandResult::took(outcome(&command, "registered")).emitting(
                    ObservedEvent::new("demo.orders.CustomerRegistered".parse().unwrap())
                        .with("customer_id", id),
                )
            }
            "demo.orders.Place" => {
                let customers = self.customers.borrow();
                let customer = request.input["customer_id"].clone();
                let read = match self.mutant {
                    Mutant::CopiesLatestRegion => customers.last(),
                    Mutant::CopiesFirstRegion => customers.first(),
                    _ => customers.iter().find(|(id, _)| *id == customer),
                };
                let Some((_, region)) = read else {
                    return Ok(SemanticCommandResult::undeclared().with_consistency(token));
                };
                let id = Node::Text(format!("order-{n}"));
                self.placed.borrow_mut().push(Row::from([
                    ("order_id".to_owned(), id.clone()),
                    ("customer_id".to_owned(), customer),
                    ("region".to_owned(), region.clone()),
                    ("amount".to_owned(), request.input["amount"].clone()),
                ]));
                SemanticCommandResult::took(outcome(&command, "placed")).emitting(
                    ObservedEvent::new("demo.orders.OrderPlaced".parse().unwrap())
                        .with("order_id", id),
                )
            }
            other => return Err(TargetError::unsupported("command", other)),
        };
        Ok(result.with_consistency(token))
    }
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        assert_eq!(request.view.to_string(), VIEW);
        Ok(SemanticViewResult::of(self.view()))
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

/// The aggregate scenario's status against a target with `mutant`.
fn aggregate_status(mutant: Mutant) -> Status {
    status_of(ORDERS, mutant)
}

/// The aggregate scenario of `model`'s status against a target with `mutant`.
fn status_of(model: &str, mutant: Mutant) -> Status {
    let mut suite = synthesis(model).suite;
    suite.scenarios.retain(|id, _| id.to_string() == AGGREGATE);
    assert_eq!(
        suite.scenarios.len(),
        1,
        "the aggregate scenario is synthesized"
    );
    let admitted = AdmittedSuite::from_suite(&suite).unwrap();
    let target = Orders {
        mutant,
        customers: RefCell::default(),
        placed: RefCell::default(),
        minted: Cell::new(0),
    };
    let report = Runner::for_suite(&suite)
        .run_admitted(&admitted, &target)
        .into_report();
    report
        .scenarios
        .iter()
        .find(|scenario| scenario.scenario.to_string() == AGGREGATE)
        .map_or_else(|| panic!("{report:?}"), |scenario| scenario.status)
}

#[test]
fn the_view_as_specified_passes_and_every_wrong_key_fails() {
    assert_eq!(aggregate_status(Mutant::None), Status::Passed);
    let mut survived = Vec::new();
    for mutant in [
        Mutant::GroupsByCustomer,
        Mutant::IgnoresKey,
        Mutant::CopiesLatestRegion,
        Mutant::CopiesFirstRegion,
    ] {
        if aggregate_status(mutant) == Status::Passed {
            survived.push(mutant);
        }
    }
    assert!(survived.is_empty(), "survived: {survived:?}");
}

/// An `Optional` region, left out when a customer is registered, gives orders without one: the
/// absent group is arranged by leaving the related row's input out, and the scenario still passes
/// the view as specified and fails a target copying from the wrong customer.
#[test]
fn an_optional_key_copied_from_a_related_row_is_run_with_its_absent_group() {
    let optional = ORDERS.replace(
        "{name: region, type: String}",
        "{name: region, type: Optional<String>}",
    );
    assert_eq!(status_of(&optional, Mutant::None), Status::Passed);
    assert_ne!(
        status_of(&optional, Mutant::CopiesLatestRegion),
        Status::Passed
    );
    let suite = synthesis(&optional).suite;
    let (_, scenario) = suite
        .scenarios
        .iter()
        .find(|(id, _)| id.to_string() == AGGREGATE)
        .expect("synthesized");
    assert!(
        scenario.steps.iter().any(|step| matches!(
            step,
            ScenarioStep::ExecuteCommand { command, input, .. }
                if command.to_string() == "demo.orders.Register" && !input.contains_key("region")
        )),
        "a customer is registered without a region"
    );
}
