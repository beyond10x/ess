//! A literal fallback inside a nested mapping beside a generated leaf (beyond10x/ess#163 with
//! beyond10x/ess#179): `lead: {rank: {generated: true}, tier: {input: tier, else: Standard}}`.
//!
//! The struct is never determined whole, because `rank` is generated, so it is asserted leaf by
//! leaf. The run that sends `tier` asserts `lead.tier` as the value it sent; the run that leaves
//! `tier` out (the literal-fallback invocation) asserts `lead.tier: Standard`, on the event and on
//! the row, so an implementation storing any other default inside the struct fails the suite.
use std::collections::BTreeMap;

use ess_compiler::{resolve::compile, source::SourceMap};
use ess_conformance::{
    scenario::ViewExpectation, ConformanceSuite as Suite, ScenarioStep, ScenarioValue,
};
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source,
};
use ess_primitives::node::Node;

const MODEL: &str = "format: ess/16
system: demo
version: v1
domain: demo.orders
types:
  - {name: demo.orders.OrderId, kind: newtype, of: String}
  - {name: demo.orders.Tier, kind: enum, variants: [Express, Standard]}
  - name: demo.orders.Lead
    kind: struct
    fields:
      - {name: rank, type: Integer}
      - {name: tier, type: demo.orders.Tier}
entities:
  - name: demo.orders.Order
    identity: {name: order_id, type: demo.orders.OrderId}
    fields:
      - {name: lead, type: demo.orders.Lead}
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
events:
  - name: demo.orders.Opened
    fields:
      - {name: order_id, type: demo.orders.OrderId}
      - {name: lead, type: demo.orders.Lead}
actors:
  - {name: demo.orders.Clerk, may: [demo.orders.Open]}
commands:
  - name: demo.orders.Open
    input:
      - {name: tier, type: Optional<demo.orders.Tier>}
    outcomes:
      - name: opened
        creates: demo.orders.Order
        instance: order_id
        emits: [demo.orders.Opened]
        payload:
          demo.orders.Opened:
            order_id: {generated: true}
            lead: {rank: {generated: true}, tier: {input: tier, else: Standard}}
        sets:
          lead: {rank: {generated: true}, tier: {input: tier, else: Standard}}
views:
  - name: demo.orders.OrderRow
    source: demo.orders.Order
    consistency: read_your_writes
    fields:
      - {name: order_id, type: demo.orders.OrderId}
      - {name: lead, type: demo.orders.Lead}
";

const OPENED: &str = "demo.orders.Open/outcome/opened";

fn suite(body: &str) -> Suite {
    let raw = RawSpecFile::parse(body).unwrap_or_else(|error| panic!("{error}"));
    let spec = Specification::assemble([(Source::new("orders.yaml"), raw)])
        .unwrap_or_else(|error| panic!("{error}"));
    let ir = compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"));
    let synthesis = ess_conformance::synthesize::synthesize(&ir);
    assert!(
        synthesis.refusals.is_empty(),
        "nothing is refused: {:#?}",
        synthesis.refusals
    );
    synthesis.suite
}

/// One invocation of `Open` and what follows it up to the next one.
#[derive(Debug)]
struct Invocation {
    /// What it sent for `tier`, `None` where it left it out.
    sent: Option<Node>,
    /// The `Opened` expectation's values and its shape's leaf paths.
    event: BTreeMap<String, Node>,
    leaves: Vec<String>,
    /// Every row the view is required to contain.
    rows: Vec<BTreeMap<String, ScenarioValue>>,
}

fn invocations(suite: &Suite) -> Vec<Invocation> {
    let (_, scenario) = suite
        .scenarios
        .iter()
        .find(|(id, _)| id.to_string() == OPENED)
        .unwrap_or_else(|| panic!("no scenario {OPENED}"));
    let mut out: Vec<Invocation> = Vec::new();
    for step in &scenario.steps {
        match step {
            ScenarioStep::ExecuteCommand { command, input, .. }
                if command.to_string() == "demo.orders.Open" =>
            {
                out.push(Invocation {
                    sent: match input.get("tier") {
                        None | Some(ScenarioValue::Literal { value: Node::Null }) => None,
                        Some(ScenarioValue::Literal { value }) => Some(value.clone()),
                        Some(other) => panic!("`tier` is sent as a literal: {other:?}"),
                    },
                    event: BTreeMap::new(),
                    leaves: Vec::new(),
                    rows: Vec::new(),
                });
            }
            ScenarioStep::ExpectEvent {
                event,
                payload,
                shape,
            } if event.to_string() == "demo.orders.Opened" => {
                let last = out.last_mut().expect("an event after an invocation");
                last.event = payload.clone();
                last.leaves = shape.leaves().keys().cloned().collect();
            }
            ScenarioStep::ExpectView {
                expectation: ViewExpectation::Contains { fields },
                ..
            }
            | ScenarioStep::EventuallyView {
                expectation: ViewExpectation::Contains { fields },
                ..
            } => {
                if let Some(last) = out.last_mut() {
                    last.rows.push(fields.clone());
                }
            }
            _ => {}
        }
    }
    out
}

fn text(value: &str) -> Node {
    Node::Text(value.to_owned())
}

fn literal(value: Node) -> ScenarioValue {
    ScenarioValue::Literal { value }
}

/// The value every required row carrying `lead.tier` asserts, and how many rows carried it.
fn row_tier(invocation: &Invocation) -> Vec<ScenarioValue> {
    invocation
        .rows
        .iter()
        .filter_map(|row| row.get("lead.tier").cloned())
        .collect()
}

#[test]
fn the_omission_run_asserts_the_literal_inside_the_struct_and_the_full_run_the_sent_value() {
    let suite = suite(MODEL);
    let runs = invocations(&suite);
    let full = runs
        .iter()
        .find(|run| run.sent.is_some())
        .unwrap_or_else(|| panic!("a run sends `tier`: {runs:#?}"));
    let omitted = runs
        .iter()
        .find(|run| run.sent.is_none())
        .unwrap_or_else(|| panic!("a run leaves `tier` out: {runs:#?}"));

    let sent = full.sent.clone().expect("sent");
    assert_ne!(
        sent,
        text("Standard"),
        "the full run must send a value the literal does not equal, or it decides nothing"
    );
    // Sent: the payload and the row assert what was sent, under the leaf's dotted path.
    assert_eq!(full.event.get("lead.tier"), Some(&sent), "{full:#?}");
    assert!(!full.event.contains_key("lead"), "{full:#?}");
    assert!(
        full.leaves.iter().any(|leaf| leaf == "lead.rank"),
        "{full:#?}"
    );
    let rows = row_tier(full);
    assert!(!rows.is_empty(), "the row asserts `lead.tier`: {full:#?}");
    assert!(
        rows.iter().all(|value| *value == literal(sent.clone())),
        "{rows:?}"
    );

    // Left out: the payload and the row assert the literal.
    assert_eq!(
        omitted.event.get("lead.tier"),
        Some(&text("Standard")),
        "{omitted:#?}"
    );
    assert!(!omitted.event.contains_key("lead.rank"), "{omitted:#?}");
    assert!(
        omitted.leaves.iter().any(|leaf| leaf == "lead.rank"),
        "the generated leaf stays covered by the shape: {omitted:#?}"
    );
    let rows = row_tier(omitted);
    assert!(
        !rows.is_empty(),
        "the row asserts `lead.tier`: {omitted:#?}"
    );
    assert!(
        rows.iter().all(|value| *value == literal(text("Standard"))),
        "{rows:?}"
    );
}

#[test]
fn a_mutant_storing_another_default_inside_the_struct_is_killed() {
    // The mutant stores `Express` where the specification says `Standard`, in `sets:` only. The
    // original's omission run asks for a row with `lead.tier: Standard`; the mutant's implementation
    // stores `Express` there and fails it.
    let mutated = MODEL.replacen(
        "        sets:\n          lead: {rank: {generated: true}, tier: {input: tier, else: Standard}}",
        "        sets:\n          lead: {rank: {generated: true}, tier: {input: tier, else: Express}}",
        1,
    );
    assert_ne!(mutated, MODEL, "the mutation applies");
    let original = invocations(&suite(MODEL));
    let mutant = invocations(&suite(&mutated));
    let omitted = |runs: &[Invocation]| {
        runs.iter()
            .find(|run| run.sent.is_none())
            .map(row_tier)
            .unwrap_or_default()
    };
    let (expected, stored) = (omitted(&original), omitted(&mutant));
    assert!(!expected.is_empty() && !stored.is_empty());
    assert!(expected
        .iter()
        .all(|value| *value == literal(text("Standard"))));
    assert!(stored
        .iter()
        .all(|value| *value == literal(text("Express"))));
}
