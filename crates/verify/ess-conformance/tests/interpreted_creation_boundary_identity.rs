//! A creating branch that takes its identity from the input is sent each further boundary row under
//! an identity of its own (<https://github.com/beyond10x/ess/issues/471>).
//!
//! The `interpreted` target ended the default branch's scenario `error` whenever a sibling was
//! guarded by a two-condition `all:`. The guard was not the cause: synthesis sends the default once
//! more per conjunct refuted alone, and sent every one of those rows the plain witness's `order_id`.
//! `creates:` never replaces an instance the identity already names, so the second row is a request
//! no branch of the model describes, and the interpreter is right to refuse it. The same rows are
//! sent wherever a creating branch has boundaries: an `all:` of ordering bounds on the branch itself,
//! or each disjunct of its own `any:` holding alone.
//!
//! The second half of the issue: an `error` scenario names its cause in the text a caller reads.
use std::collections::{BTreeMap, BTreeSet};

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::{
    interpret::Interpreted,
    mutate::{self, Document, MutantClass},
    report::Status,
    synthesize::synthesize,
    AdmittedSuite, ConformanceReport, ConformanceSuite, Runner, ScenarioStep, ScenarioValue,
};
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source,
};
use ess_primitives::{facts::Number, node::Node};

/// The issue's model: `placed` creates the order the input names, beside `reserved`, an error
/// guarded by `GUARD`.
const BESIDE: &str = r"
format: ess/22
system: shop
version: v1
domain: shop.orders
types:
  - {name: shop.orders.OrderId, kind: newtype, of: Uuid}
  - {name: shop.orders.Tier, kind: enum, variants: [Basic, Gold, Platinum]}
  - {name: shop.orders.Channel, kind: enum, variants: [Web, Phone]}
entities:
  - name: shop.orders.Order
    identity: {name: order_id, type: shop.orders.OrderId}
    fields:
      - {name: tier, type: shop.orders.Tier}
    lifecycle:
      initial: Placed
      states: [Placed]
      terminal: [Placed]
      transitions: []
actors:
  - name: shop.orders.Clerk
    may: [shop.orders.PlaceOrder]
errors:
  - name: shop.orders.Reserved
    summary: The order is reserved.
    fields:
      - {name: order_id, type: shop.orders.OrderId}
events:
  - name: shop.orders.OrderPlaced
    fields:
      - {name: order_id, type: shop.orders.OrderId}
      - {name: tier, type: shop.orders.Tier}
commands:
  - name: shop.orders.PlaceOrder
    input:
      - {name: order_id, type: shop.orders.OrderId}
      - {name: tier, type: shop.orders.Tier}
      - {name: channel, type: shop.orders.Channel}
    outcomes:
      - name: reserved
        when: GUARD
        error: shop.orders.Reserved
        payload: {shop.orders.Reserved: {order_id: input.order_id}}
      - name: placed
        creates: shop.orders.Order
        instance: order_id
        sets: {tier: input.tier}
        emits: [shop.orders.OrderPlaced]
        payload: {shop.orders.OrderPlaced: {order_id: input.order_id, tier: input.tier}}
views:
  - name: shop.orders.Orders
    source: shop.orders.Order
    consistency: read_your_writes
    fields:
      - {name: order_id, type: shop.orders.OrderId}
      - {name: state, type: shop.orders.Order.State}
      - {name: tier, type: shop.orders.Tier}
";

/// The other side of the same rows: `placed` is the guarded branch, guarded by `GUARD`, and
/// `reserved` is its default.
const CREATING: &str = r"
format: ess/22
system: shop
version: v1
domain: shop.orders
types:
  - {name: shop.orders.OrderId, kind: newtype, of: Uuid}
  - {name: shop.orders.Tier, kind: enum, variants: [Basic, Gold, Platinum]}
  - {name: shop.orders.Channel, kind: enum, variants: [Web, Phone]}
entities:
  - name: shop.orders.Order
    identity: {name: order_id, type: shop.orders.OrderId}
    fields:
      - {name: tier, type: shop.orders.Tier}
    lifecycle:
      initial: Placed
      states: [Placed]
      terminal: [Placed]
      transitions: []
actors:
  - name: shop.orders.Clerk
    may: [shop.orders.PlaceOrder]
errors:
  - name: shop.orders.Reserved
    summary: The order is reserved.
    fields:
      - {name: order_id, type: shop.orders.OrderId}
events:
  - name: shop.orders.OrderPlaced
    fields:
      - {name: order_id, type: shop.orders.OrderId}
      - {name: tier, type: shop.orders.Tier}
commands:
  - name: shop.orders.PlaceOrder
    input:
      - {name: order_id, type: shop.orders.OrderId}
      - {name: tier, type: shop.orders.Tier}
      - {name: channel, type: shop.orders.Channel}
      - {name: amount, type: Integer}
    outcomes:
      - name: placed
        when: GUARD
        creates: shop.orders.Order
        instance: order_id
        sets: {tier: input.tier}
        emits: [shop.orders.OrderPlaced]
        payload: {shop.orders.OrderPlaced: {order_id: input.order_id, tier: input.tier}}
      - name: reserved
        error: shop.orders.Reserved
        payload: {shop.orders.Reserved: {order_id: input.order_id}}
views:
  - name: shop.orders.Orders
    source: shop.orders.Order
    consistency: read_your_writes
    fields:
      - {name: order_id, type: shop.orders.OrderId}
      - {name: state, type: shop.orders.Order.State}
      - {name: tier, type: shop.orders.Tier}
";

/// The issue's sibling guard.
const ALL: &str = "{all: [channel == Phone, tier == Gold]}";
/// The issue's second form: the same conjunction over a membership.
const MEMBERSHIP: &str = "{all: [channel == Phone, {tier: [Gold, Platinum]}]}";
/// The issue's third form. Beside a default it adds no row: the default refutes every disjunct at
/// once, which the plain witness already does.
const ANY: &str = "{any: [channel == Phone, tier == Gold]}";
/// The control the issue reports passing.
const SINGLE: &str = "tier == Gold";
/// An `any:` on the creating branch itself: each disjunct holding alone is a further row.
const ANY_CREATING: &str = "{any: [channel == Web, tier == Basic]}";
/// Ordering bounds on the creating branch: each accepting boundary is a further row.
const RANGE: &str = "{all: [amount >= 0, amount < 100]}";
/// The creating branch's control: one equality has no boundary to move.
const EQUALS: &str = "tier == Basic";

const PLACED: &str = "shop.orders.PlaceOrder/outcome/placed";
const RESERVED: &str = "shop.orders.PlaceOrder/outcome/reserved";

/// The cause the interpreter names when a supplied identity is created twice.
const CAUSE: &str = "creation never replaces it";

fn beside(guard: &str) -> String {
    BESIDE.replace("GUARD", guard)
}

fn creating(guard: &str) -> String {
    CREATING.replace("GUARD", guard)
}

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("orders.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap()
}

fn suite_of(text: &str) -> ConformanceSuite {
    let synthesis = synthesize(&ir(text));
    assert!(
        synthesis.refusals.is_empty(),
        "every scenario is synthesized: {:#?}",
        synthesis.refusals
    );
    synthesis.suite
}

fn run(suite: &ConformanceSuite, text: &str) -> ConformanceReport {
    let admitted = AdmittedSuite::from_suite(suite).unwrap();
    Runner::for_suite(suite)
        .run_admitted(&admitted, &Interpreted::for_model(ir(text)))
        .into_report()
}

fn statuses(report: &ConformanceReport) -> BTreeMap<String, Status> {
    report
        .scenarios
        .iter()
        .map(|result| (result.scenario.to_string(), result.status))
        .collect()
}

fn both_passed() -> BTreeMap<String, Status> {
    BTreeMap::from([
        (PLACED.to_owned(), Status::Passed),
        (RESERVED.to_owned(), Status::Passed),
    ])
}

/// The literal input of every invocation the `placed` scenario requires `placed` of.
fn sent_to_placed(suite: &ConformanceSuite) -> Vec<BTreeMap<String, Node>> {
    let scenario = suite
        .scenarios
        .iter()
        .find(|(id, _)| id.to_string() == PLACED)
        .unwrap_or_else(|| panic!("no scenario {PLACED}"))
        .1;
    let mut out = Vec::new();
    let mut steps = scenario.steps.iter().peekable();
    while let Some(step) = steps.next() {
        let ScenarioStep::ExecuteCommand { command, input, .. } = step else {
            continue;
        };
        if command.to_string() != "shop.orders.PlaceOrder" {
            continue;
        }
        let Some(ScenarioStep::ExpectOutcome { outcome }) = steps.peek() else {
            continue;
        };
        if outcome.outcome.to_string() != "placed" {
            continue;
        }
        out.push(
            input
                .iter()
                .filter_map(|(field, value)| match value {
                    ScenarioValue::Literal { value } => Some((field.clone(), value.clone())),
                    _ => None,
                })
                .collect(),
        );
    }
    out
}

fn variant(value: &str) -> Node {
    Node::Text(value.to_owned())
}

fn number(value: i64) -> Node {
    Node::Number(Number::from(value))
}

/// #471's acceptance: the default beside a sibling guarded by `all:`, by a conjunction over a
/// membership, or by `any:` passes 2 of 2 against the interpreter, as the single condition did.
#[test]
fn issue_471_the_default_beside_a_guarded_sibling_passes_against_interpreted() {
    for guard in [ALL, MEMBERSHIP, ANY, SINGLE] {
        let text = beside(guard);
        let report = run(&suite_of(&text), &text);
        assert_eq!(
            statuses(&report),
            both_passed(),
            "`reserved` guarded by `{guard}`:\n{report}"
        );
    }
}

/// The same rows from the guarded side: a creating branch passes at each of its own boundaries.
#[test]
fn issue_471_a_creating_branch_passes_at_each_of_its_boundary_rows() {
    for guard in [ANY_CREATING, RANGE, EQUALS] {
        let text = creating(guard);
        let report = run(&suite_of(&text), &text);
        assert_eq!(
            statuses(&report),
            both_passed(),
            "`placed` guarded by `{guard}`:\n{report}"
        );
    }
}

/// The class, read off the suite rather than the target: no scenario requires `placed` twice at one
/// `order_id`, and every boundary row is still sent — a fix that dropped the rows would also leave
/// one identity, and would pin nothing the rows were added for.
#[test]
fn no_scenario_requires_one_supplied_identity_to_be_created_twice() {
    let channel_tier = |row: &BTreeMap<String, Node>| (row["channel"].clone(), row["tier"].clone());
    let refuting_each_conjunct = BTreeSet::from([
        (variant("Web"), variant("Gold")),
        (variant("Phone"), variant("Basic")),
    ]);
    for (label, model) in [
        ("beside all", beside(ALL)),
        ("beside membership", beside(MEMBERSHIP)),
        ("creating any", creating(ANY_CREATING)),
        ("creating range", creating(RANGE)),
    ] {
        let sent = sent_to_placed(&suite_of(&model));
        assert_eq!(
            sent.len(),
            3,
            "{label}: the plain witness and two rows: {sent:#?}"
        );
        let identities: BTreeSet<&Node> = sent.iter().map(|row| &row["order_id"]).collect();
        assert_eq!(
            identities.len(),
            sent.len(),
            "{label}: each invocation requiring `placed` creates an order of its own: {sent:#?}"
        );
        if label == "creating range" {
            let amounts: BTreeSet<Node> = sent.iter().map(|row| row["amount"].clone()).collect();
            assert!(
                amounts.contains(&number(0)) && amounts.contains(&number(99)),
                "{label}: both accepting boundaries are still sent: {sent:#?}"
            );
        } else {
            let pairs: BTreeSet<(Node, Node)> = sent.iter().map(channel_tier).collect();
            assert!(
                pairs.is_superset(&refuting_each_conjunct),
                "{label}: each condition alone is still sent: {sent:#?}"
            );
        }
    }
}

/// #471's effect: the mutation audit's baseline passes against the interpreter, so it runs.
#[test]
fn issue_471_the_mutation_audit_runs_against_interpreted() {
    for guard in [ALL, MEMBERSHIP] {
        let model = beside(guard);
        let raw = RawSpecFile::parse(&model).unwrap_or_else(|error| panic!("{error}\n{model}"));
        let mut texts = SourceMap::new();
        texts.insert("orders.yaml".to_owned(), model.clone());
        let files: Vec<Document> = vec![(Source::new("orders.yaml"), raw)];
        let ir = mutate::compile(files.clone(), &texts).expect("the model compiles");
        if let Err(refusal) = mutate::audit(&files, &texts, MutantClass::ALL, || {
            Interpreted::for_model(ir.clone())
        }) {
            panic!("`reserved` guarded by `{guard}`: the audit was refused: {refusal}");
        }
    }
}

/// #471's second acceptance: a scenario that ends `error` names its cause in the text report, on
/// the line after its own, as the detailed result already does.
///
/// The scenario is made to end `error` by sending its first creation again at the identity it has
/// just created: a request no branch of the model describes.
#[test]
fn issue_471_an_error_scenario_names_its_cause_in_the_text_report() {
    let model = beside(SINGLE);
    let mut suite = suite_of(&model);
    let (_, placed) = suite
        .scenarios
        .iter_mut()
        .find(|(id, _)| id.to_string() == PLACED)
        .unwrap_or_else(|| panic!("no scenario {PLACED}"));
    let first = placed
        .steps
        .iter()
        .position(|step| matches!(step, ScenarioStep::ExecuteCommand { .. }))
        .expect("the scenario invokes the command");
    let again = placed.steps[first..=first + 1].to_vec();
    assert!(
        matches!(again[1], ScenarioStep::ExpectOutcome { .. }),
        "the invocation is followed by the outcome it requires: {again:#?}"
    );
    placed.steps.extend(again);

    let report = run(&suite, &model);
    let result = report
        .scenarios
        .iter()
        .find(|result| result.scenario.to_string() == PLACED)
        .expect("the scenario ran");
    assert_eq!(result.status, Status::Error, "{report}");
    assert!(
        result
            .diagnostics()
            .any(|diagnostic| diagnostic.observed.iter().any(|line| line.contains(CAUSE))),
        "the detailed result names the cause: {result:#?}"
    );

    let rendered = report.to_string();
    let lines: Vec<&str> = rendered.lines().collect();
    let at = lines
        .iter()
        .position(|line| line.trim_start().starts_with("error") && line.ends_with(PLACED))
        .unwrap_or_else(|| panic!("no `error` line for {PLACED}:\n{rendered}"));
    assert!(
        lines.get(at + 1).is_some_and(|next| next.contains(CAUSE)),
        "the line after `error {PLACED}` names the cause:\n{rendered}"
    );
}
