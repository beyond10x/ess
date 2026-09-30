//! Adversary pass 2 on beyond10x/ess#257: an aggregate group key copied from a related row.
//!
//! Pass 2 attacks correction 1: rows sharing an owner reuse it as their related row, the `Bₖ`
//! tuple of a key read from a shared, grouped-by owner is dropped, the creating branch of a related
//! row is chosen once per row and input, and an optional related key leaves its input out. Every
//! case builds a minimal brand-free model and runs the synthesized aggregate scenario against a
//! hand-written target and its mutants.

use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;

use ess_compiler::{
    ir::EssIr,
    refs::{CommandRef, OutcomeRef},
    resolve::compile,
    source::SourceMap,
};
use ess_conformance::{
    report::Status, synthesize::Synthesis, target::*, AdmittedSuite, ConformanceSuite, Runner,
    ScenarioStep, ScenarioValue,
};
use ess_domain::{
    command::OutcomeName,
    spec::{RawSpecFile, Specification},
    system::Source,
};
use ess_primitives::{facts::FactValue, node::Node};

const BASE: &str = "format: ess/16
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
      - {name: country, type: String}
      - {name: tier, type: Integer}
    lifecycle: {initial: Active, states: [Active], terminal: [Active]}
  - name: demo.orders.Order
    identity: {name: order_id, type: demo.orders.OrderId}
    fields:
      - {name: customer_id, type: demo.orders.CustomerId}
      - {name: region, type: String}
      - {name: country, type: String}
      - {name: tier, type: Integer}
      - {name: channel, type: String}
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
      - {name: country, type: String}
      - {name: tier, type: Integer}
    outcomes:
      - name: registered
        creates: demo.orders.Customer
        instance: customer_id
        emits: [demo.orders.CustomerRegistered]
        payload:
          demo.orders.CustomerRegistered: {customer_id: {generated: true}}
        sets:
          region: input.region
          country: input.country
          tier: input.tier
  - name: demo.orders.Place
    input:
      - {name: customer_id, type: demo.orders.CustomerId}
      - {name: channel, type: String}
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
          country: {related: {via: input.customer_id, field: country}}
          tier: {related: {via: input.customer_id, field: tier}}
          channel: input.channel
          amount: input.amount
views:
";

const VIEW: &str = "demo.orders.ByKey";
const AGGREGATE: &str = "demo.orders.ByKey/aggregate";

fn view(keys: &[(&str, &str)], extra: &str) -> String {
    let names: Vec<&str> = keys.iter().map(|(name, _)| *name).collect();
    let mut out = format!(
        "  - name: {VIEW}\n    source: demo.orders.Order\n    consistency: read_your_writes\n    \
         group_by: [{}]\n",
        names.join(", ")
    );
    out.push_str(extra);
    out.push_str("    fields:\n");
    for (name, type_ref) in keys {
        out.extend(["      - {name: ", name, ", type: ", type_ref, "}\n"]);
    }
    out.push_str("      - {name: orders, type: Integer, aggregate: {count: {}}}\n");
    out.push_str("      - {name: total, type: Integer, aggregate: {sum: amount}}\n");
    out
}

fn model(base: &str, keys: &[(&str, &str)], extra: &str) -> String {
    format!("{base}{}", view(keys, extra))
}

fn replaced(text: &str, from: &str, to: &str) -> String {
    assert!(text.contains(from), "fixture edit misses: {from}");
    text.replacen(from, to, 1)
}

/// Order owned by Customer through `customer_id`, the input every related key is read through.
fn owned() -> String {
    replaced(
        BASE,
        "      - {name: tier, type: Integer}\n    lifecycle: {initial: Active",
        "      - {name: tier, type: Integer}\n    relations:\n      - {name: orders, kind: owns, \
         target: demo.orders.Order, cardinality: many, via: customer_id}\n    lifecycle: \
         {initial: Active",
    )
}

/// A second related entity: the order copies its `city` from the store `store_id` names.
fn with_store() -> String {
    let text = replaced(
        BASE,
        "  - {name: demo.orders.OrderId, kind: newtype, of: String}\n",
        "  - {name: demo.orders.OrderId, kind: newtype, of: String}\n  - {name: \
         demo.orders.StoreId, kind: newtype, of: String}\n",
    );
    let text = replaced(
        &text,
        "entities:\n",
        "entities:\n  - name: demo.orders.Store\n    identity: {name: store_id, type: \
         demo.orders.StoreId}\n    fields:\n      - {name: city, type: String}\n      - {name: \
         rank, type: Integer}\n    lifecycle: {initial: Open, states: [Open], terminal: \
         [Open]}\n",
    );
    let text = replaced(
        &text,
        "      - {name: channel, type: String}\n      - {name: amount, type: Integer}\n    \
         lifecycle: {initial: Placed",
        "      - {name: store_id, type: demo.orders.StoreId}\n      - {name: city, type: \
         String}\n      - {name: rank, type: Integer}\n      - {name: channel, type: String}\n      \
         - {name: amount, type: Integer}\n    lifecycle: {initial: Placed",
    );
    let text = replaced(
        &text,
        "events:\n",
        "events:\n  - name: demo.orders.StoreOpened\n    fields:\n      - {name: store_id, type: \
         demo.orders.StoreId}\n",
    );
    let text = replaced(
        &text,
        "may: [demo.orders.Register, demo.orders.Place]",
        "may: [demo.orders.OpenStore, demo.orders.Register, demo.orders.Place]",
    );
    let text = replaced(
        &text,
        "commands:\n",
        "commands:\n  - name: demo.orders.OpenStore\n    input:\n      - {name: city, type: \
         String}\n      - {name: rank, type: Integer}\n    outcomes:\n      - name: opened\n        \
         creates: demo.orders.Store\n        instance: store_id\n        emits: \
         [demo.orders.StoreOpened]\n        payload:\n          demo.orders.StoreOpened: \
         {store_id: {generated: true}}\n        sets:\n          city: input.city\n          \
         rank: input.rank\n",
    );
    let text = replaced(
        &text,
        "      - {name: customer_id, type: demo.orders.CustomerId}\n      - {name: channel, type: \
         String}\n      - {name: amount, type: Integer}\n    outcomes:",
        "      - {name: customer_id, type: demo.orders.CustomerId}\n      - {name: store_id, type: \
         demo.orders.StoreId}\n      - {name: channel, type: String}\n      - {name: amount, type: \
         Integer}\n    outcomes:",
    );
    replaced(
        &text,
        "          channel: input.channel\n",
        "          store_id: input.store_id\n          city: {related: {via: input.store_id, \
         field: city}}\n          rank: {related: {via: input.store_id, field: rank}}\n          \
         channel: input.channel\n",
    )
}

/// A second input naming the same entity: the order copies `seller_region` and `seller_tier` from
/// the customer `seller_id` names, beside the buyer's `region` and `tier`.
fn with_seller() -> String {
    let text = replaced(
        BASE,
        "      - {name: channel, type: String}\n      - {name: amount, type: Integer}\n    \
         lifecycle: {initial: Placed",
        "      - {name: seller_id, type: demo.orders.CustomerId}\n      - {name: seller_region, \
         type: String}\n      - {name: seller_tier, type: Integer}\n      - {name: channel, type: \
         String}\n      - {name: amount, type: Integer}\n    lifecycle: {initial: Placed",
    );
    let text = replaced(
        &text,
        "      - {name: customer_id, type: demo.orders.CustomerId}\n      - {name: channel, type: \
         String}\n      - {name: amount, type: Integer}\n    outcomes:",
        "      - {name: customer_id, type: demo.orders.CustomerId}\n      - {name: seller_id, \
         type: demo.orders.CustomerId}\n      - {name: channel, type: String}\n      - {name: \
         amount, type: Integer}\n    outcomes:",
    );
    replaced(
        &text,
        "          channel: input.channel\n",
        "          seller_id: input.seller_id\n          seller_region: {related: {via: \
         input.seller_id, field: region}}\n          seller_tier: {related: {via: \
         input.seller_id, field: tier}}\n          channel: input.channel\n",
    )
}

/// The customer's region is optional. `Register` requires it; only `Import`, declared after it,
/// takes it as an `Optional` input, so only `Import` can register a customer without one.
fn optional_through_one_branch() -> String {
    let text = replaced(
        BASE,
        "      - {name: region, type: String}\n",
        "      - {name: region, type: Optional<String>}\n",
    );
    let text = replaced(
        &text,
        "      - {name: region, type: String}\n",
        "      - {name: region, type: Optional<String>}\n",
    );
    let text = replaced(
        &text,
        "may: [demo.orders.Register, demo.orders.Place]",
        "may: [demo.orders.Register, demo.orders.Import, demo.orders.Place]",
    );
    replaced(
        &text,
        "  - name: demo.orders.Place\n",
        "  - name: demo.orders.Import
    input:
      - {name: region, type: Optional<String>}
      - {name: country, type: String}
      - {name: tier, type: Integer}
    outcomes:
      - name: imported
        creates: demo.orders.Customer
        instance: customer_id
        emits: [demo.orders.CustomerRegistered]
        payload:
          demo.orders.CustomerRegistered: {customer_id: {generated: true}}
        sets:
          region: input.region
          country: input.country
          tier: input.tier
  - name: demo.orders.Place
",
    )
}

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("orders.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn synthesis(text: &str) -> Synthesis {
    ess_conformance::synthesize::synthesize(&ir(text))
}

fn refusals_of_view(result: &Synthesis) -> Vec<String> {
    result
        .refusals
        .iter()
        .map(ToString::to_string)
        .filter(|rendered| rendered.contains(VIEW))
        .collect()
}

fn aggregate_suite(text: &str) -> ConformanceSuite {
    let result = synthesis(text);
    let mut suite = result.suite.clone();
    suite.scenarios.retain(|id, _| id.to_string() == AGGREGATE);
    assert_eq!(
        suite.scenarios.len(),
        1,
        "no scenario {AGGREGATE}: {:#?}",
        refusals_of_view(&result)
    );
    suite
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mutant {
    None,
    /// Every order is a group of its own.
    NoGrouping,
    /// Groups by these keys only; reports the others from the group's first order.
    GroupsOnly(&'static [&'static str]),
    /// Copies the buyer's fields from the customer registered last.
    CustomerLatest,
    /// Copies the store's fields from the store opened last.
    StoreLatest,
    /// Copies the store's fields from the store opened first.
    StoreFirst,
    /// Copies the seller's fields from the buyer.
    SellerIsBuyer,
    /// Leaves out every order whose region is absent.
    DropsAbsentRegion,
}

type Row = BTreeMap<String, Node>;

struct Shop {
    mutant: Mutant,
    keys: Vec<String>,
    customers: RefCell<Vec<(Node, Row)>>,
    stores: RefCell<Vec<(Node, Row)>>,
    placed: RefCell<Vec<Row>>,
    minted: Cell<u64>,
}

impl Shop {
    fn new(mutant: Mutant, keys: &[(&str, &str)]) -> Self {
        Self {
            mutant,
            keys: keys.iter().map(|(name, _)| (*name).to_owned()).collect(),
            customers: RefCell::default(),
            stores: RefCell::default(),
            placed: RefCell::default(),
            minted: Cell::new(0),
        }
    }

    fn view(&self, region: Option<&Node>) -> Vec<Row> {
        let grouped: Vec<String> = match self.mutant {
            Mutant::GroupsOnly(only) => only.iter().map(|key| (*key).to_owned()).collect(),
            _ => self.keys.clone(),
        };
        let mut groups: Vec<(Vec<Node>, Vec<Row>)> = Vec::new();
        for (index, order) in self.placed.borrow().iter().enumerate() {
            if region.is_some_and(|wanted| order.get("region") != Some(wanted))
                || (self.mutant == Mutant::DropsAbsentRegion && !order.contains_key("region"))
            {
                continue;
            }
            let mut held: Vec<Node> = grouped
                .iter()
                .map(|key| order.get(key).cloned().unwrap_or(Node::Null))
                .collect();
            if self.mutant == Mutant::NoGrouping {
                held.push(number(index as i128));
            }
            match groups.iter_mut().find(|(group, _)| *group == held) {
                Some((_, members)) => members.push(order.clone()),
                None => groups.push((held, vec![order.clone()])),
            }
        }
        groups
            .into_iter()
            .map(|(_, members)| {
                let mut row = Row::new();
                for key in &self.keys {
                    row.insert(
                        key.clone(),
                        members[0].get(key).cloned().unwrap_or(Node::Null),
                    );
                }
                let sum: i128 = members.iter().map(|member| int(&member["amount"])).sum();
                row.insert("orders".to_owned(), number(members.len() as i128));
                row.insert("total".to_owned(), number(sum));
                row
            })
            .collect()
    }
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

/// Copy `fields` from `held` into `row` under `prefix`; an absent or null field stays absent.
fn copy(row: &mut Row, held: &Row, fields: &[&str], prefix: &str) {
    for field in fields {
        if let Some(value) = held.get(*field).filter(|value| **value != Node::Null) {
            row.insert(format!("{prefix}{field}"), value.clone());
        }
    }
}

impl ConformanceTarget for Shop {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("shop-fixture-2", "1"))
    }
    fn begin_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        self.customers.replace(Vec::new());
        self.stores.replace(Vec::new());
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
            name @ ("demo.orders.Register" | "demo.orders.Import") => {
                let id = Node::Text(format!("customer-{n}"));
                self.customers
                    .borrow_mut()
                    .push((id.clone(), request.input.clone()));
                let branch = if name == "demo.orders.Register" {
                    "registered"
                } else {
                    "imported"
                };
                SemanticCommandResult::took(outcome(&command, branch)).emitting(
                    ObservedEvent::new("demo.orders.CustomerRegistered".parse().unwrap())
                        .with("customer_id", id),
                )
            }
            "demo.orders.OpenStore" => {
                let id = Node::Text(format!("store-{n}"));
                self.stores
                    .borrow_mut()
                    .push((id.clone(), request.input.clone()));
                SemanticCommandResult::took(outcome(&command, "opened")).emitting(
                    ObservedEvent::new("demo.orders.StoreOpened".parse().unwrap())
                        .with("store_id", id),
                )
            }
            "demo.orders.Place" => {
                let customers = self.customers.borrow();
                let stores = self.stores.borrow();
                let customer = request.input["customer_id"].clone();
                let buyer = match self.mutant {
                    Mutant::CustomerLatest => customers.last(),
                    _ => customers.iter().find(|(id, _)| *id == customer),
                };
                let Some((_, buyer)) = buyer else {
                    return Ok(SemanticCommandResult::undeclared().with_consistency(token));
                };
                let id = Node::Text(format!("order-{n}"));
                let mut row = Row::from([
                    ("order_id".to_owned(), id.clone()),
                    ("customer_id".to_owned(), customer),
                ]);
                copy(&mut row, buyer, &["region", "country", "tier"], "");
                if let Some(seller) = request.input.get("seller_id") {
                    let held = match self.mutant {
                        Mutant::SellerIsBuyer => Some(buyer),
                        _ => customers
                            .iter()
                            .find(|(id, _)| id == seller)
                            .map(|(_, held)| held),
                    };
                    let Some(held) = held else {
                        return Ok(SemanticCommandResult::undeclared().with_consistency(token));
                    };
                    row.insert("seller_id".to_owned(), seller.clone());
                    copy(&mut row, held, &["region", "tier"], "seller_");
                }
                if let Some(store) = request.input.get("store_id") {
                    let held = match self.mutant {
                        Mutant::StoreLatest => stores.last(),
                        Mutant::StoreFirst => stores.first(),
                        _ => stores.iter().find(|(id, _)| id == store),
                    };
                    let Some((_, held)) = held else {
                        return Ok(SemanticCommandResult::undeclared().with_consistency(token));
                    };
                    row.insert("store_id".to_owned(), store.clone());
                    copy(&mut row, held, &["city", "rank"], "");
                }
                for own in ["channel", "amount"] {
                    if let Some(value) = request.input.get(own) {
                        row.insert(own.to_owned(), value.clone());
                    }
                }
                self.placed.borrow_mut().push(row);
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
        Ok(SemanticViewResult::of(
            self.view(request.params.get("region")),
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

fn status(suite: &ConformanceSuite, keys: &[(&str, &str)], mutant: Mutant) -> Status {
    let admitted = AdmittedSuite::from_suite(suite).unwrap();
    let target = Shop::new(mutant, keys);
    let report = Runner::for_suite(suite)
        .run_admitted(&admitted, &target)
        .into_report();
    report
        .scenarios
        .iter()
        .find(|scenario| scenario.scenario.to_string() == AGGREGATE)
        .map_or_else(|| panic!("{report:?}"), |scenario| scenario.status)
}

/// The correct target passes and every listed mutant fails.
fn discriminates(text: &str, keys: &[(&str, &str)], mutants: &[Mutant]) {
    let suite = aggregate_suite(text);
    assert_eq!(
        status(&suite, keys, Mutant::None),
        Status::Passed,
        "the correct target fails"
    );
    let survived: Vec<Mutant> = mutants
        .iter()
        .copied()
        .filter(|mutant| status(&suite, keys, *mutant) == Status::Passed)
        .collect();
    assert!(survived.is_empty(), "survived: {survived:?}");
}

const CUSTOMER: (&str, &str) = ("customer_id", "demo.orders.CustomerId");
const TIER: (&str, &str) = ("tier", "Integer");
const REGION: (&str, &str) = ("region", "String");
const CHANNEL: (&str, &str) = ("channel", "String");

/// Grouped by the owner link, then by an `Integer` key read from that owner. The link is scoped and
/// first, so it has no `B₁`, and the tier's `B₂` is dropped: no two owners ever hold one tier, and a
/// target that ignores the owner link and groups by the tier alone forms the same groups. Before
/// the drop, `B₂` held A's tier under B's owner, and that target merged it with A. A tuple that
/// keeps B's tier under another owner is realisable and would catch it.
#[test]
fn a_target_ignoring_the_owner_link_beside_a_walked_related_key_fails() {
    let keys = [CUSTOMER, TIER];
    discriminates(
        &model(&owned(), &keys, ""),
        &keys,
        &[Mutant::NoGrouping, Mutant::GroupsOnly(&["tier"])],
    );
}

/// The same view with the keys the other way round: the link's `B₂` keeps B's tier under another
/// owner, and the tier-only target fails. Only the order of `group_by` differs from the case above.
#[test]
fn the_same_keys_in_the_other_order_catch_the_tier_only_target() {
    let keys = [TIER, CUSTOMER];
    discriminates(
        &model(&owned(), &keys, ""),
        &keys,
        &[Mutant::NoGrouping, Mutant::GroupsOnly(&["tier"])],
    );
}

/// Grouped by the owner link and filtered by the owner's region through a parameter. The refuted
/// rows are moved out of the scope by another region, while their owner link names an owner an
/// earlier row created in the scope: a refuted row is not in the view, so it may be created under
/// an owner of its own, and the view is witnessable.
#[test]
fn a_refuted_row_under_a_shared_owner_that_needs_another_value_is_arranged() {
    let keys = [CUSTOMER];
    let extra = "    params: [{name: region, type: String}]\n    filter: region == param.region\n";
    discriminates(&model(&owned(), &keys, extra), &keys, &[Mutant::NoGrouping]);
}

/// Only `Import` can register a customer without a region; `Register`, declared first, requires
/// one. The absent group is arranged through `Import`, no step sends a region as `null`, and a
/// target that drops the absent group fails.
#[test]
fn an_optional_related_key_that_only_one_branch_leaves_out_is_arranged_through_it() {
    let keys = [("region", "Optional<String>")];
    let text = model(&optional_through_one_branch(), &keys, "");
    let suite = aggregate_suite(&text);
    let scenario = suite.scenarios.values().next().unwrap();
    let mut imported_without = 0;
    for step in &scenario.steps {
        if let ScenarioStep::ExecuteCommand { command, input, .. } = step {
            for (field, value) in input {
                assert_ne!(
                    *value,
                    ScenarioValue::literal(Node::Null),
                    "{command} sends `{field}: null`"
                );
            }
            if command.to_string() == "demo.orders.Register" {
                assert!(
                    input.contains_key("region"),
                    "Register requires its region: {input:?}"
                );
            }
            if command.to_string() == "demo.orders.Import" && !input.contains_key("region") {
                imported_without += 1;
            }
        }
    }
    assert!(
        imported_without > 0,
        "no customer is imported without a region"
    );
    discriminates(
        &text,
        &keys,
        &[
            Mutant::NoGrouping,
            Mutant::CustomerLatest,
            Mutant::DropsAbsentRegion,
        ],
    );
}

/// Two keys, each copied from a row of another entity: the customer's region and the store's city.
#[test]
fn two_related_entities_feeding_two_keys_are_each_witnessed() {
    let keys = [REGION, ("city", "String")];
    discriminates(
        &model(&with_store(), &keys, ""),
        &keys,
        &[
            Mutant::NoGrouping,
            Mutant::CustomerLatest,
            Mutant::StoreLatest,
            Mutant::StoreFirst,
            Mutant::GroupsOnly(&["region"]),
            Mutant::GroupsOnly(&["city"]),
        ],
    );
}

/// The same with walked keys, the customer's tier and the store's rank, beside a scoped input key
/// that keeps the rows apart from other scenarios'.
#[test]
fn two_related_entities_feeding_two_walked_keys_are_each_witnessed() {
    let keys = [TIER, ("rank", "Integer"), CHANNEL];
    discriminates(
        &model(&with_store(), &keys, ""),
        &keys,
        &[
            Mutant::NoGrouping,
            Mutant::CustomerLatest,
            Mutant::StoreLatest,
            Mutant::StoreFirst,
            Mutant::GroupsOnly(&["tier"]),
            Mutant::GroupsOnly(&["rank"]),
        ],
    );
}

/// Two inputs naming one entity: the buyer's region and the seller's, both a Customer's `region`.
/// A target that copies the seller's region from the buyer fails.
#[test]
fn two_inputs_naming_one_entity_are_each_read_from_their_own_row() {
    let keys = [REGION, ("seller_region", "String")];
    discriminates(
        &model(&with_seller(), &keys, ""),
        &keys,
        &[
            Mutant::NoGrouping,
            Mutant::SellerIsBuyer,
            Mutant::GroupsOnly(&["region"]),
            Mutant::GroupsOnly(&["seller_region"]),
        ],
    );
}

/// The same with walked keys, the buyer's tier and the seller's, beside a scoped input key.
#[test]
fn two_inputs_naming_one_entity_with_walked_keys_are_each_read_from_their_own_row() {
    let keys = [TIER, ("seller_tier", "Integer"), CHANNEL];
    discriminates(
        &model(&with_seller(), &keys, ""),
        &keys,
        &[
            Mutant::NoGrouping,
            Mutant::SellerIsBuyer,
            Mutant::GroupsOnly(&["tier"]),
            Mutant::GroupsOnly(&["seller_tier"]),
        ],
    );
}

/// Pass 1's optional case asserted only that a scenario exists. The absent group it arranges is
/// asserted here: a target that leaves out the orders of a customer without a region fails.
#[test]
fn the_absent_group_of_an_optional_related_key_is_asserted() {
    let text = replaced(
        BASE,
        "      - {name: region, type: String}\n",
        "      - {name: region, type: Optional<String>}\n",
    );
    let text = replaced(
        &text,
        "      - {name: region, type: String}\n",
        "      - {name: region, type: Optional<String>}\n",
    );
    let text = replaced(
        &text,
        "      - {name: region, type: String}\n",
        "      - {name: region, type: Optional<String>}\n",
    );
    let keys = [("region", "Optional<String>")];
    discriminates(
        &model(&text, &keys, ""),
        &keys,
        &[Mutant::DropsAbsentRegion, Mutant::CustomerLatest],
    );
}

/// An optional key read from the shared owner, grouped beside the owner link: its absent group is
/// an owner of its own without a region.
#[test]
fn an_optional_key_read_from_a_shared_owner_has_its_absent_group() {
    let text = replaced(
        &owned(),
        "      - {name: region, type: String}\n",
        "      - {name: region, type: Optional<String>}\n",
    );
    let text = replaced(
        &text,
        "      - {name: region, type: String}\n",
        "      - {name: region, type: Optional<String>}\n",
    );
    let text = replaced(
        &text,
        "      - {name: region, type: String}\n",
        "      - {name: region, type: Optional<String>}\n",
    );
    let keys = [CUSTOMER, ("region", "Optional<String>")];
    discriminates(
        &model(&text, &keys, ""),
        &keys,
        &[
            Mutant::NoGrouping,
            Mutant::DropsAbsentRegion,
            Mutant::GroupsOnly(&["region"]),
        ],
    );
}

/// Reported downstream on 0.48.0: the related key is read through the created row's own field
/// (`via: customer_id`, a subject field the branch fills from `input.customer_id`), not through
/// `input.customer_id`, and the view groups by that key declared `Optional<String>`. The scenario
/// is synthesized with no ESS-SYNTH-017, and a target that never groups fails it.
#[test]
fn an_optional_key_read_through_the_created_rows_own_field_is_witnessed() {
    let mut text = BASE.to_owned();
    for _ in 0..3 {
        text = replaced(
            &text,
            "      - {name: region, type: String}\n",
            "      - {name: region, type: Optional<String>}\n",
        );
    }
    let text = replaced(
        &text,
        "region: {related: {via: input.customer_id, field: region}}",
        "region: {related: {via: customer_id, field: region}}",
    );
    let keys = [("region", "Optional<String>")];
    let text = model(&text, &keys, "");
    let result = synthesis(&text);
    let unwitnessed: Vec<String> = refusals_of_view(&result)
        .into_iter()
        .filter(|refusal| refusal.contains("ESS-SYNTH-017"))
        .collect();
    assert_eq!(unwitnessed, Vec::<String>::new());
    discriminates(
        &text,
        &keys,
        &[Mutant::NoGrouping, Mutant::DropsAbsentRegion],
    );
}
