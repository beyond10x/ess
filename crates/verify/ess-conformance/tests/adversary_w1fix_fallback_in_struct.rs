//! Adversary, w1fix pass 1: a literal fallback inside a struct beside a generated leaf, in the
//! variants the unit's own `literal_fallback_in_struct.rs` does not drive: a payload-only mutant,
//! an Integer fallback, two fallbacks in one struct, a fallback two levels deep, and an optional
//! input that is also copied elsewhere.
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

/// `extra_types` goes under `types:`, `lead_fields` is `Lead`'s field list, `input` the command
/// input, `mapping` the flow mapping written for `lead` in both `payload` and `sets`
/// (`payload_mapping` overrides the payload's), `extra_entity` extra entity/view fields.
struct Model<'a> {
    extra_types: &'a str,
    lead_fields: &'a str,
    input: &'a str,
    payload_mapping: &'a str,
    sets_mapping: &'a str,
    extra_sets: &'a str,
    extra_fields: &'a str,
}

impl Model<'_> {
    fn render(&self) -> String {
        format!(
            "format: ess/16
system: demo
version: v1
domain: demo.orders
types:
  - {{name: demo.orders.OrderId, kind: newtype, of: String}}
  - {{name: demo.orders.Tier, kind: enum, variants: [Express, Standard]}}
{extra_types}  - name: demo.orders.Lead
    kind: struct
    fields:
{lead_fields}entities:
  - name: demo.orders.Order
    identity: {{name: order_id, type: demo.orders.OrderId}}
    fields:
      - {{name: lead, type: demo.orders.Lead}}
{extra_fields}    lifecycle: {{initial: Open, states: [Open], terminal: [Open]}}
events:
  - name: demo.orders.Opened
    fields:
      - {{name: order_id, type: demo.orders.OrderId}}
      - {{name: lead, type: demo.orders.Lead}}
actors:
  - {{name: demo.orders.Clerk, may: [demo.orders.Open]}}
commands:
  - name: demo.orders.Open
    input:
{input}    outcomes:
      - name: opened
        creates: demo.orders.Order
        instance: order_id
        emits: [demo.orders.Opened]
        payload:
          demo.orders.Opened:
            order_id: {{generated: true}}
            lead: {payload_mapping}
        sets:
          lead: {sets_mapping}
{extra_sets}views:
  - name: demo.orders.OrderRow
    source: demo.orders.Order
    consistency: read_your_writes
    fields:
      - {{name: order_id, type: demo.orders.OrderId}}
      - {{name: lead, type: demo.orders.Lead}}
{extra_fields_view}",
            extra_types = self.extra_types,
            lead_fields = self.lead_fields,
            extra_fields = self.extra_fields,
            input = self.input,
            payload_mapping = self.payload_mapping,
            sets_mapping = self.sets_mapping,
            extra_sets = self.extra_sets,
            extra_fields_view = self.extra_fields,
        )
    }
}

const TIER_LEAD: &str =
    "      - {name: rank, type: Integer}\n      - {name: tier, type: demo.orders.Tier}\n";
const TIER_INPUT: &str = "      - {name: tier, type: Optional<demo.orders.Tier>}\n";
const TIER_MAPPING: &str = "{rank: {generated: true}, tier: {input: tier, else: Standard}}";

const OPENED: &str = "demo.orders.Open/outcome/opened";

fn suite(body: &str) -> Suite {
    let raw = RawSpecFile::parse(body).unwrap_or_else(|error| panic!("{error}\n{body}"));
    let spec = Specification::assemble([(Source::new("orders.yaml"), raw)])
        .unwrap_or_else(|error| panic!("{error}\n{body}"));
    let ir = compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}\n{body}"));
    let synthesis = ess_conformance::synthesize::synthesize(&ir);
    assert!(
        synthesis.refusals.is_empty(),
        "nothing is refused: {:#?}",
        synthesis.refusals
    );
    synthesis.suite
}

#[derive(Debug)]
struct Invocation {
    input: BTreeMap<String, ScenarioValue>,
    event: BTreeMap<String, Node>,
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
                    input: input.clone(),
                    event: BTreeMap::new(),
                    rows: Vec::new(),
                });
            }
            ScenarioStep::ExpectEvent { event, payload, .. }
                if event.to_string() == "demo.orders.Opened" =>
            {
                out.last_mut().expect("after an invocation").event = payload.clone();
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

fn left_out(invocation: &Invocation, input: &str) -> bool {
    matches!(
        invocation.input.get(input),
        None | Some(ScenarioValue::Literal { value: Node::Null })
    )
}

/// The invocation that leaves `input` out.
fn omission<'a>(runs: &'a [Invocation], input: &str) -> &'a Invocation {
    runs.iter()
        .find(|run| left_out(run, input))
        .unwrap_or_else(|| panic!("a run leaves `{input}` out: {runs:#?}"))
}

fn text(value: &str) -> Node {
    Node::Text(value.to_owned())
}

/// The omission run asserts `leaf` as `expected` in the payload and in every required row.
fn asserts_literal(runs: &[Invocation], input: &str, leaf: &str, expected: &Node) {
    let run = omission(runs, input);
    assert_eq!(run.event.get(leaf), Some(expected), "payload: {run:#?}");
    let rows: Vec<_> = run.rows.iter().filter_map(|row| row.get(leaf)).collect();
    assert!(!rows.is_empty(), "a row asserts `{leaf}`: {run:#?}");
    assert!(
        rows.iter().all(|value| **value
            == ScenarioValue::Literal {
                value: expected.clone()
            }),
        "{rows:?}"
    );
}

/// The mutant stores `Express` in the *payload* only; `sets:` keeps `Standard`. The omission run of
/// the original asserts `lead.tier: Standard` on the event, which an implementation of the mutant
/// does not emit.
#[test]
fn adversary_w1fix_a_payload_only_mutant_default_inside_the_struct_is_killed() {
    let original = Model {
        extra_types: "",
        lead_fields: TIER_LEAD,
        input: TIER_INPUT,
        payload_mapping: TIER_MAPPING,
        sets_mapping: TIER_MAPPING,
        extra_sets: "",
        extra_fields: "",
    };
    let mutant = Model {
        payload_mapping: "{rank: {generated: true}, tier: {input: tier, else: Express}}",
        ..original
    };
    let expected = invocations(&suite(&original.render()));
    let mutated = invocations(&suite(&mutant.render()));
    asserts_literal(&expected, "tier", "lead.tier", &text("Standard"));
    assert_eq!(
        omission(&mutated, "tier").event.get("lead.tier"),
        Some(&text("Express")),
        "the mutant's own suite asserts what it emits, so the two differ"
    );
}

/// An `Integer` fallback beside a generated leaf.
#[test]
fn adversary_w1fix_an_integer_fallback_inside_the_struct_is_asserted_on_omission() {
    let mapping = "{rank: {generated: true}, level: {input: level, else: 7}}";
    let model = Model {
        extra_types: "",
        lead_fields: "      - {name: rank, type: Integer}\n      - {name: level, type: Integer}\n",
        input: "      - {name: level, type: Optional<Integer>}\n",
        payload_mapping: mapping,
        sets_mapping: mapping,
        extra_sets: "",
        extra_fields: "",
    };
    let runs = invocations(&suite(&model.render()));
    let seven = Node::Number(ess_primitives::facts::Number::new(7.0).unwrap());
    asserts_literal(&runs, "level", "lead.level", &seven);
}

/// Two fallbacks in one struct beside a generated leaf: the omission of each is asserted.
#[test]
fn adversary_w1fix_two_fallbacks_beside_a_generated_leaf_are_each_asserted() {
    let mapping = "{rank: {generated: true}, tier: {input: tier, else: Standard}, backup: {input: backup, else: Express}}";
    let model = Model {
        extra_types: "",
        lead_fields: "      - {name: rank, type: Integer}\n      - {name: tier, type: demo.orders.Tier}\n      - {name: backup, type: demo.orders.Tier}\n",
        input: "      - {name: tier, type: Optional<demo.orders.Tier>}\n      - {name: backup, type: Optional<demo.orders.Tier>}\n",
        payload_mapping: mapping,
        sets_mapping: mapping,
        extra_sets: "",
        extra_fields: "",
    };
    let runs = invocations(&suite(&model.render()));
    asserts_literal(&runs, "tier", "lead.tier", &text("Standard"));
    asserts_literal(&runs, "backup", "lead.backup", &text("Express"));
}

/// A fallback two struct levels down, beside a generated leaf at the inner level.
#[test]
fn adversary_w1fix_a_fallback_two_levels_deep_is_asserted_on_omission() {
    let mapping = "{inner: {rank: {generated: true}, tier: {input: tier, else: Standard}}}";
    let model = Model {
        extra_types: "  - name: demo.orders.Inner\n    kind: struct\n    fields:\n      - {name: rank, type: Integer}\n      - {name: tier, type: demo.orders.Tier}\n",
        lead_fields: "      - {name: inner, type: demo.orders.Inner}\n",
        input: TIER_INPUT,
        payload_mapping: mapping,
        sets_mapping: mapping,
        extra_sets: "",
        extra_fields: "",
    };
    let runs = invocations(&suite(&model.render()));
    asserts_literal(&runs, "tier", "lead.inner.tier", &text("Standard"));
}

/// `tier` is also copied, as sent, into a second `Optional` field: leaving it out is still a run
/// the suite makes, and the literal is still asserted inside the struct.
///
/// Red against `without_literal_fallbacks` (`synthesize.rs`), which keeps any input another value
/// of the outcome reads plainly, even when that value's target is `Optional` and would take the
/// omission as `null`. The fallback's literal is then asserted nowhere, and an implementation
/// storing another default passes. Pre-existing (the rule is unchanged from the base).
#[test]
fn adversary_w1fix_an_optional_input_also_copied_elsewhere_keeps_the_omission_run() {
    let model = Model {
        extra_types: "",
        lead_fields: TIER_LEAD,
        input: TIER_INPUT,
        payload_mapping: TIER_MAPPING,
        sets_mapping: TIER_MAPPING,
        extra_sets: "          asked: input.tier\n",
        extra_fields: "      - {name: asked, type: Optional<demo.orders.Tier>}\n",
    };
    let runs = invocations(&suite(&model.render()));
    asserts_literal(&runs, "tier", "lead.tier", &text("Standard"));
}
