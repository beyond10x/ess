//! Adversary pass 1 on beyond10x/ess#257: an aggregate group key copied from a related row.
//!
//! Each case builds a minimal brand-free model around one boundary of the change: a key read
//! through an owner link, keys read from one related entity that more than one branch creates, an
//! optional related key, a related aggregate input's message, mixed input-set and related keys, the
//! aggregate values themselves, a chain of related copies, and the Go runtime's verdicts.

mod support_go;

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

/// A view grouped by `keys` (name, type) that counts orders and sums their amount, plus `extra`
/// field lines.
fn view(keys: &[(&str, &str)], extra: &str) -> String {
    let names: Vec<&str> = keys.iter().map(|(name, _)| *name).collect();
    let mut out = format!(
        "  - name: {VIEW}\n    source: demo.orders.Order\n    consistency: read_your_writes\n    \
         group_by: [{}]\n    fields:\n",
        names.join(", ")
    );
    for (name, type_ref) in keys {
        out.extend(["      - {name: ", name, ", type: ", type_ref, "}\n"]);
    }
    out.push_str("      - {name: orders, type: Integer, aggregate: {count: {}}}\n");
    out.push_str("      - {name: total, type: Integer, aggregate: {sum: amount}}\n");
    out.push_str(extra);
    out
}

fn model(base: &str, keys: &[(&str, &str)], extra: &str) -> String {
    format!("{base}{}", view(keys, extra))
}

fn replaced(text: &str, from: &str, to: &str) -> String {
    assert!(text.contains(from), "fixture edit misses: {from}");
    text.replace(from, to)
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

fn unwitnessed(result: &Synthesis) -> Vec<String> {
    result
        .refusals
        .iter()
        .filter(|refusal| refusal.code().to_string() == "ESS-SYNTH-017")
        .map(ToString::to_string)
        .filter(|rendered| rendered.contains(VIEW))
        .collect()
}

/// The suite holding only the aggregate scenario, or a panic naming why there is none.
fn aggregate_suite(text: &str) -> ConformanceSuite {
    let result = synthesis(text);
    let mut suite = result.suite.clone();
    suite.scenarios.retain(|id, _| id.to_string() == AGGREGATE);
    assert_eq!(
        suite.scenarios.len(),
        1,
        "no scenario {AGGREGATE}: {:#?}",
        unwitnessed(&result)
    );
    suite
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mutant {
    None,
    /// Never groups: every order is a row of its own.
    NoGrouping,
    /// Copies from the customer registered last, not the one the input names.
    CopiesLatest,
    /// Reports the count where the sum of the amounts belongs.
    TotalIsCount,
}

type Row = BTreeMap<String, Node>;

struct Shop {
    mutant: Mutant,
    keys: Vec<String>,
    customers: RefCell<Vec<(Node, Row)>>,
    placed: RefCell<Vec<Row>>,
    minted: Cell<u64>,
}

impl Shop {
    fn new(mutant: Mutant, keys: &[(&str, &str)]) -> Self {
        Self {
            mutant,
            keys: keys.iter().map(|(name, _)| (*name).to_owned()).collect(),
            customers: RefCell::default(),
            placed: RefCell::default(),
            minted: Cell::new(0),
        }
    }

    fn view(&self) -> Vec<Row> {
        let mut groups: Vec<(Vec<Node>, Vec<Row>)> = Vec::new();
        for (index, order) in self.placed.borrow().iter().enumerate() {
            let mut held: Vec<Node> = self
                .keys
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
                let count = members.len() as i128;
                row.insert("orders".to_owned(), number(count));
                row.insert(
                    "total".to_owned(),
                    number(if self.mutant == Mutant::TotalIsCount {
                        count
                    } else {
                        sum
                    }),
                );
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

impl ConformanceTarget for Shop {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("shop-fixture", "1"))
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
            "demo.orders.Place" => {
                let customers = self.customers.borrow();
                let customer = request.input["customer_id"].clone();
                let read = match self.mutant {
                    Mutant::CopiesLatest => customers.last(),
                    _ => customers.iter().find(|(id, _)| *id == customer),
                };
                let Some((_, held)) = read else {
                    return Ok(SemanticCommandResult::undeclared().with_consistency(token));
                };
                let id = Node::Text(format!("order-{n}"));
                let mut row = Row::from([
                    ("order_id".to_owned(), id.clone()),
                    ("customer_id".to_owned(), customer),
                ]);
                for copied in ["region", "country", "tier"] {
                    if let Some(value) = held.get(copied) {
                        row.insert(copied.to_owned(), value.clone());
                    }
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
    assert_eq!(status(&suite, keys, Mutant::None), Status::Passed);
    let survived: Vec<Mutant> = mutants
        .iter()
        .copied()
        .filter(|mutant| status(&suite, keys, *mutant) == Status::Passed)
        .collect();
    assert!(survived.is_empty(), "survived: {survived:?}");
}

const REGION: (&str, &str) = ("region", "String");

/// Order owned by Customer through the same input the region is copied through, grouped by the
/// owner link and the region: rows sharing a customer share its region, so the pattern's groups of
/// several rows are realisable. Reusing each row's own related row as its owner splits every group
/// into single rows, and a target that never groups then passes.
#[test]
fn an_owner_link_grouped_beside_a_related_key_keeps_groups_of_several_rows() {
    let owned = replaced(
        BASE,
        "      - {name: tier, type: Integer}\n    lifecycle: {initial: Active",
        "      - {name: tier, type: Integer}\n    relations:\n      - {name: orders, kind: owns, \
         target: demo.orders.Order, cardinality: many, via: customer_id}\n    lifecycle: \
         {initial: Active",
    );
    let keys = [("customer_id", "demo.orders.CustomerId"), REGION];
    discriminates(&model(&owned, &keys, ""), &keys, &[Mutant::NoGrouping]);
}

/// Two branches create a Customer: `Import` sets only the region from its input, `Register` sets
/// both keys the view reads. One branch that fills both exists, so the view is witnessed through it;
/// choosing a branch per field picks `Import` for the region and refuses.
#[test]
fn keys_read_from_one_related_row_use_one_branch_that_fills_them_all() {
    let import = "  - name: demo.orders.Import
    input:
      - {name: region, type: String}
    outcomes:
      - name: imported
        creates: demo.orders.Customer
        instance: customer_id
        emits: [demo.orders.CustomerRegistered]
        payload:
          demo.orders.CustomerRegistered: {customer_id: {generated: true}}
        sets:
          region: input.region
  - name: demo.orders.Place
";
    let text = replaced(BASE, "  - name: demo.orders.Place\n", import);
    let text = replaced(
        &text,
        "may: [demo.orders.Register, demo.orders.Place]",
        "may: [demo.orders.Register, demo.orders.Import, demo.orders.Place]",
    );
    let keys = [REGION, ("country", "String")];
    discriminates(
        &model(&text, &keys, ""),
        &keys,
        &[Mutant::NoGrouping, Mutant::CopiesLatest],
    );
}

/// The region is optional on the customer, on its registering input and on the order: a customer
/// registered without one gives an order without one. The view grouped by that region is a key set
/// from a related row, and its absent value is reachable.
#[test]
fn an_optional_group_key_copied_from_a_related_row_is_witnessed() {
    let text = replaced(
        BASE,
        "      - {name: region, type: String}\n",
        "      - {name: region, type: Optional<String>}\n",
    );
    let keys = [("region", "Optional<String>")];
    let result = synthesis(&model(&text, &keys, ""));
    assert!(
        result
            .suite
            .scenarios
            .keys()
            .any(|id| id.to_string() == AGGREGATE),
        "{:#?}",
        unwitnessed(&result)
    );
}

/// An aggregate input copied from a related row is refused (the story is about keys), and the
/// refusal names why: the sentence does not stop after "but".
#[test]
fn a_related_aggregate_input_refusal_says_why() {
    let keys = [REGION];
    let extra = "      - {name: tiers, type: Integer, aggregate: {sum: tier}}\n";
    let refusals = unwitnessed(&synthesis(&model(BASE, &keys, extra)));
    assert!(!refusals.is_empty(), "the view is refused");
    for refusal in &refusals {
        // The rendered refusal is the reason line, then a `help:` line.
        let reason = refusal.lines().nth(1).unwrap_or_default().trim_end();
        assert!(
            !reason.ends_with(", but") && !reason.ends_with(" but"),
            "the reason is missing: {refusal}"
        );
    }
}

/// One key set from input and one copied from a related row.
#[test]
fn an_input_key_beside_a_related_key_is_witnessed_and_discriminating() {
    let keys = [REGION, ("channel", "String")];
    discriminates(
        &model(BASE, &keys, ""),
        &keys,
        &[
            Mutant::NoGrouping,
            Mutant::CopiesLatest,
            Mutant::TotalIsCount,
        ],
    );
}

/// The aggregate values are witnessed, not only the grouping.
#[test]
fn the_aggregate_values_of_a_related_key_view_are_witnessed() {
    let keys = [REGION];
    discriminates(
        &model(BASE, &keys, ""),
        &keys,
        &[
            Mutant::TotalIsCount,
            Mutant::NoGrouping,
            Mutant::CopiesLatest,
        ],
    );
}

/// The customer's region is itself copied from another row: no arrangement chooses it, and the
/// refusal names the related source rather than saying "does not set".
#[test]
fn a_key_copied_through_a_chain_is_refused_naming_the_related_source() {
    let text = replaced(
        BASE,
        "  - {name: demo.orders.OrderId, kind: newtype, of: String}\n",
        "  - {name: demo.orders.OrderId, kind: newtype, of: String}\n  - {name: \
         demo.orders.AreaId, kind: newtype, of: String}\n",
    );
    let text = replaced(
        &text,
        "entities:\n",
        "entities:\n  - name: demo.orders.Area\n    identity: {name: area_id, type: \
         demo.orders.AreaId}\n    fields:\n      - {name: name, type: String}\n    lifecycle: \
         {initial: Open, states: [Open], terminal: [Open]}\n",
    );
    let text = replaced(
        &text,
        "events:\n",
        "events:\n  - name: demo.orders.AreaOpened\n    fields:\n      - {name: area_id, type: \
         demo.orders.AreaId}\n",
    );
    let text = replaced(
        &text,
        "may: [demo.orders.Register, demo.orders.Place]",
        "may: [demo.orders.OpenArea, demo.orders.Register, demo.orders.Place]",
    );
    let text = replaced(
        &text,
        "commands:\n",
        "commands:\n  - name: demo.orders.OpenArea\n    input:\n      - {name: name, type: \
         String}\n    outcomes:\n      - name: opened\n        creates: demo.orders.Area\n        \
         instance: area_id\n        emits: [demo.orders.AreaOpened]\n        payload:\n          \
         demo.orders.AreaOpened: {area_id: {generated: true}}\n        sets:\n          name: \
         input.name\n",
    );
    let text = replaced(
        &text,
        "      - {name: region, type: String}\n      - {name: country, type: String}\n      - \
         {name: tier, type: Integer}\n    outcomes:",
        "      - {name: area_id, type: demo.orders.AreaId}\n      - {name: country, type: \
         String}\n      - {name: tier, type: Integer}\n    outcomes:",
    );
    let text = replaced(
        &text,
        "          region: input.region\n",
        "          region: {related: {via: input.area_id, field: name}}\n",
    );
    let keys = [REGION];
    let refusals = unwitnessed(&synthesis(&model(&text, &keys, "")));
    assert!(!refusals.is_empty(), "the view is refused");
    assert!(
        refusals
            .iter()
            .all(|refusal| !refusal.contains("does not set") && refusal.contains("{related:")),
        "{refusals:#?}"
    );
}

/// The generated Go runtime admits the suite and gives the reference verdicts.
#[test]
fn go_gives_the_reference_verdict_for_a_related_group_key() {
    let keys = [REGION];
    let suite = aggregate_suite(&model(BASE, &keys, ""));
    let correct =
        support_go::assert_parity("agg257-correct", &suite, Shop::new(Mutant::None, &keys));
    assert_eq!(support_go::not_passed(&correct), Vec::<&str>::new());
    let latest = support_go::assert_parity(
        "agg257-latest",
        &suite,
        Shop::new(Mutant::CopiesLatest, &keys),
    );
    assert_eq!(support_go::not_passed(&latest), vec![AGGREGATE]);
}
