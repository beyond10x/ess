//! Adversary, w1fix pass 2: the literal fallback inside a struct and the corrected omission rule
//! (`without_literal_fallbacks` in `synthesize.rs`, `Read::Nullable`), in variants neither the
//! unit's suites nor pass 1 drive: a Decimal fallback beside a generated leaf, a plain copy of the
//! omitted input into an `Optional` leaf of the same struct, and a view whose filter reads the
//! `Optional` field the omission run now leaves absent.
use std::collections::BTreeMap;

use ess_compiler::{resolve::compile, source::SourceMap};
use ess_conformance::{
    scenario::ViewExpectation, ConformanceSuite as Suite, ScenarioStep, ScenarioValue,
};
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source,
};
use ess_primitives::{facts::Number, node::Node};

struct Model<'a> {
    lead_fields: &'a str,
    input: &'a str,
    mapping: &'a str,
    extra_sets: &'a str,
    extra_fields: &'a str,
    extra_views: &'a str,
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
  - name: demo.orders.Lead
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
            lead: {mapping}
        sets:
          lead: {mapping}
{extra_sets}views:
  - name: demo.orders.OrderRow
    source: demo.orders.Order
    consistency: read_your_writes
    fields:
      - {{name: order_id, type: demo.orders.OrderId}}
      - {{name: lead, type: demo.orders.Lead}}
{extra_fields_view}{extra_views}",
            lead_fields = self.lead_fields,
            extra_fields = self.extra_fields,
            input = self.input,
            mapping = self.mapping,
            extra_sets = self.extra_sets,
            extra_fields_view = self.extra_fields,
            extra_views = self.extra_views,
        )
    }
}

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
    /// `(view, expectation kind, fields)` for every view step after the invocation.
    views: Vec<(String, &'static str, BTreeMap<String, ScenarioValue>)>,
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
                    views: Vec::new(),
                });
            }
            ScenarioStep::ExpectEvent { event, payload, .. }
                if event.to_string() == "demo.orders.Opened" =>
            {
                out.last_mut().expect("after an invocation").event = payload.clone();
            }
            ScenarioStep::ExpectView {
                view, expectation, ..
            }
            | ScenarioStep::EventuallyView {
                view, expectation, ..
            } => {
                let (kind, fields) = match expectation {
                    ViewExpectation::Contains { fields } => ("contains", fields.clone()),
                    ViewExpectation::Excludes { fields } => ("excludes", fields.clone()),
                    _ => ("other", BTreeMap::new()),
                };
                if let Some(last) = out.last_mut() {
                    last.views.push((view.to_string(), kind, fields));
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

fn omission<'a>(runs: &'a [Invocation], input: &str) -> &'a Invocation {
    runs.iter()
        .find(|run| left_out(run, input))
        .unwrap_or_else(|| panic!("a run leaves `{input}` out: {runs:#?}"))
}

fn asserts_literal(runs: &[Invocation], input: &str, leaf: &str, expected: &Node) {
    let run = omission(runs, input);
    assert_eq!(run.event.get(leaf), Some(expected), "payload: {run:#?}");
    let rows: Vec<_> = run
        .views
        .iter()
        .filter(|(view, kind, _)| view == "demo.orders.OrderRow" && *kind == "contains")
        .filter_map(|(_, _, row)| row.get(leaf))
        .collect();
    assert!(!rows.is_empty(), "a row asserts `{leaf}`: {run:#?}");
    assert!(
        rows.iter().all(|value| **value
            == ScenarioValue::Literal {
                value: expected.clone()
            }),
        "{rows:?}"
    );
}

const TIER_INPUT: &str = "      - {name: tier, type: Optional<demo.orders.Tier>}\n";

/// A Decimal fallback beside a generated leaf: the omission run asserts `lead.fee: 2.5` read at the
/// leaf's type, in the payload and the row.
#[test]
fn adversary_w1fix_pass2_a_decimal_fallback_inside_the_struct_is_asserted_on_omission() {
    let model = Model {
        lead_fields: "      - {name: rank, type: Integer}\n      - {name: fee, type: Decimal}\n",
        input: "      - {name: fee, type: Optional<Decimal>}\n",
        mapping: "{rank: {generated: true}, fee: {input: fee, else: 2.5}}",
        extra_sets: "",
        extra_fields: "",
        extra_views: "",
    };
    let runs = invocations(&suite(&model.render()));
    let expected = Node::Number(Number::decimal_literal("2.5").expect("a decimal"));
    asserts_literal(&runs, "fee", "lead.fee", &expected);
}

/// `lead.asked: input.tier` copies the omitted input plainly into an `Optional` leaf of the same
/// struct. The corrected rule reads that as `Nullable`, so the omission run still exists: it must
/// assert `lead.tier: Standard`, and must not assert a value for `lead.asked` other than absent.
#[test]
fn adversary_w1fix_pass2_a_plain_copy_into_an_optional_leaf_of_the_same_struct_keeps_the_omission_run(
) {
    let model = Model {
        lead_fields: "      - {name: rank, type: Integer}\n      - {name: tier, type: demo.orders.Tier}\n      - {name: asked, type: Optional<demo.orders.Tier>}\n",
        input: TIER_INPUT,
        mapping: "{rank: {generated: true}, tier: {input: tier, else: Standard}, asked: input.tier}",
        extra_sets: "",
        extra_fields: "",
        extra_views: "",
    };
    let runs = invocations(&suite(&model.render()));
    asserts_literal(&runs, "tier", "lead.tier", &Node::Text("Standard".into()));
    let run = omission(&runs, "tier");
    for (view, _, row) in &run.views {
        if let Some(value) = row.get("lead.asked") {
            assert_eq!(
                value,
                &ScenarioValue::Literal { value: Node::Null },
                "{view}: `lead.asked` holds nothing when `tier` is left out"
            );
        }
    }
    if let Some(value) = run.event.get("lead.asked") {
        assert_eq!(value, &Node::Null, "payload `lead.asked`");
    }
}

/// `asked: input.tier` into an `Optional` entity field, and a second view that shows only orders
/// with `defined(asked)`. The omission run leaves `tier` out, so `asked` is absent and the order is
/// not in `AskedRow`: the run must not require a row for it there, and the full run (which sends
/// `tier`) must.
#[test]
fn adversary_w1fix_pass2_the_omission_run_does_not_require_a_row_the_filter_now_excludes() {
    let model = Model {
        lead_fields:
            "      - {name: rank, type: Integer}\n      - {name: tier, type: demo.orders.Tier}\n",
        input: TIER_INPUT,
        mapping: "{rank: {generated: true}, tier: {input: tier, else: Standard}}",
        extra_sets: "          asked: input.tier\n",
        extra_fields: "      - {name: asked, type: Optional<demo.orders.Tier>}\n",
        extra_views: "  - name: demo.orders.AskedRow
    source: demo.orders.Order
    consistency: read_your_writes
    filter: defined(asked)
    fields:
      - {name: order_id, type: demo.orders.OrderId}
      - {name: asked, type: Optional<demo.orders.Tier>}
",
    };
    let runs = invocations(&suite(&model.render()));
    let run = omission(&runs, "tier");
    assert!(
        !run.views
            .iter()
            .any(|(view, kind, _)| view == "demo.orders.AskedRow" && *kind == "contains"),
        "the omission run requires a row `defined(asked)` excludes: {run:#?}"
    );
    let full = runs
        .iter()
        .find(|run| !left_out(run, "tier"))
        .expect("a run sends `tier`");
    assert!(
        full.views
            .iter()
            .any(|(view, kind, _)| view == "demo.orders.AskedRow" && *kind == "contains"),
        "the full run requires the row: {full:#?}"
    );
}

/// `asked: input.tier` into an `Optional` entity field that an entity invariant requires to be
/// present. The omission run leaves `tier` out, so the row it asserts holds `asked: null`; a
/// `satisfies: defined(asked)` check in the same run then contradicts that row, and no
/// implementation can pass the scenario. Either the omission run keeps `tier` (the invariant
/// needs it), or it does not demand the invariant over a row it itself asserts violates it.
#[test]
fn adversary_w1fix_pass2_the_omission_run_does_not_contradict_an_invariant_over_the_nullable_field()
{
    let body = Model {
        lead_fields:
            "      - {name: rank, type: Integer}\n      - {name: tier, type: demo.orders.Tier}\n",
        input: TIER_INPUT,
        mapping: "{rank: {generated: true}, tier: {input: tier, else: Standard}}",
        extra_sets: "          asked: input.tier\n",
        extra_fields: "      - {name: asked, type: Optional<demo.orders.Tier>}\n",
        extra_views: "",
    }
    .render()
    .replacen(
        "    lifecycle: {initial: Open",
        "    invariants:\n      - \"defined(asked)\"\n    lifecycle: {initial: Open",
        1,
    );
    assert!(body.contains("defined(asked)"), "{body}");
    let suite = suite(&body);
    for scenario in suite.scenarios.values() {
        let mut omitted = false;
        for step in &scenario.steps {
            match step {
                ScenarioStep::ExecuteCommand { input, .. } => {
                    omitted = matches!(
                        input.get("tier"),
                        None | Some(ScenarioValue::Literal { value: Node::Null })
                    );
                }
                ScenarioStep::ExpectView {
                    expectation: ViewExpectation::Satisfies { predicate },
                    ..
                }
                | ScenarioStep::EventuallyView {
                    expectation: ViewExpectation::Satisfies { predicate },
                    ..
                } if omitted => {
                    assert!(
                    !predicate.to_string().contains("defined(asked)"),
                    "the omission run leaves `asked` absent and still demands `{predicate}`: {:#?}",
                    scenario.steps
                );
                }
                _ => {}
            }
        }
    }
}

/// The literal fallback inside a struct beside a generated leaf, under an `ess/15` header: refused
/// as `unsupported_format_version` in `sets:` and in the payload, as a top-level one is.
#[test]
fn adversary_w1fix_pass2_a_fallback_inside_a_struct_is_refused_under_ess_15() {
    let body = Model {
        lead_fields:
            "      - {name: rank, type: Integer}\n      - {name: tier, type: demo.orders.Tier}\n",
        input: TIER_INPUT,
        mapping: "{rank: {generated: true}, tier: {input: tier, else: Standard}}",
        extra_sets: "",
        extra_fields: "",
        extra_views: "",
    }
    .render()
    .replacen("format: ess/16", "format: ess/15", 1);
    let raw = RawSpecFile::parse(&body).unwrap_or_else(|error| panic!("{error}"));
    let error = match Specification::assemble([(Source::new("orders.yaml"), raw)]) {
        Err(errors) => errors.to_string(),
        Ok(spec) => match compile(&spec, &SourceMap::new()) {
            Err(error) => format!("{error:?}"),
            Ok(_) => panic!("an ess/15 source with `else: <literal>` inside a struct compiles"),
        },
    };
    assert!(
        error.contains("requires specification format ess/16"),
        "{error}"
    );
    assert!(
        error
            .matches("requires specification format ess/16")
            .count()
            >= 2,
        "both the payload and `sets:` are refused: {error}"
    );
}
