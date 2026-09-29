//! Adversary pass 2 on beyond10x/ess#217 after correction 1: an overlap of two accepting `when:`
//! branches that needs more than one leaf moved off its base value together, where
//! `witness::exhausts` counts regions per leaf and may call the overlap empty although it is not.
//! Every case asserts the rule of the unit's own design document: an overlap is sent requiring the
//! first-declared branch, or recorded as `Note::UnwitnessedOverlap` — never neither.
#![allow(clippy::too_many_lines)]

use std::collections::BTreeMap;

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::{
    synthesize::{synthesize, Note, Synthesis},
    ConformanceSuite, ScenarioStep, ScenarioValue,
};
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source,
};
use ess_primitives::node::Node;

const HEAD: &str = r"
format: ess/16
system: demo
version: v1
domain: demo.orders
types:
  - {name: demo.orders.OrderId, kind: newtype, of: Uuid}
entities:
  - name: demo.orders.Order
    identity: {name: order_id, type: demo.orders.OrderId}
    fields: []
    lifecycle:
      initial: Placed
      states: [Placed]
      terminal: [Placed]
      transitions: []
actors:
  - name: demo.orders.Clerk
    may: [demo.orders.PlaceOrder]
errors:
  - name: demo.orders.NotPlaced
    summary: No order was placed.
    fields: []
events:
  - name: demo.orders.Placed
    fields:
      - {name: order_id, type: demo.orders.OrderId}
  - name: demo.orders.Flagged
    fields:
      - {name: order_id, type: demo.orders.OrderId}
commands:
  - name: demo.orders.PlaceOrder
";

fn branch(name: &str, when: &str, event: &str) -> String {
    format!(
        "      - name: {name}\n        when: {when}\n        creates: demo.orders.Order\n        instance: order_id\n        emits: [demo.orders.{event}]\n        payload: {{demo.orders.{event}: {{order_id: {{generated: true}}}}}}\n"
    )
}

const DEFAULT: &str = "      - name: refused\n        error: demo.orders.NotPlaced\n";

fn model(inputs: &str, first: &str, other: &str) -> String {
    format!(
        "{HEAD}    input:\n{inputs}    outcomes:\n{}{}{DEFAULT}",
        branch("small", first, "Placed"),
        branch("flagged", other, "Flagged"),
    )
}

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("orders.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|errors| panic!("{errors:?}\n{text}"))
}

/// Every invocation of `PlaceOrder` a scenario sends, with the branch it requires.
fn sent(suite: &ConformanceSuite) -> Vec<(BTreeMap<String, ScenarioValue>, String)> {
    let mut out = Vec::new();
    for scenario in suite.scenarios.values() {
        let mut steps = scenario.steps.iter().peekable();
        while let Some(step) = steps.next() {
            let ScenarioStep::ExecuteCommand {
                command: invoked,
                input,
                ..
            } = step
            else {
                continue;
            };
            if invoked.to_string() != "demo.orders.PlaceOrder" {
                continue;
            }
            if let Some(ScenarioStep::ExpectOutcome { outcome }) = steps.peek() {
                out.push((input.clone(), outcome.outcome.to_string()));
            }
        }
    }
    out
}

fn overlap_notes(synthesis: &Synthesis) -> Vec<String> {
    synthesis
        .notes
        .iter()
        .filter(|note| matches!(note, Note::UnwitnessedOverlap { .. }))
        .map(ToString::to_string)
        .collect()
}

fn literal<'a>(input: &'a BTreeMap<String, ScenarioValue>, field: &str) -> Option<&'a Node> {
    match input.get(field) {
        Some(ScenarioValue::Literal { value }) => Some(value),
        _ => None,
    }
}

fn num(input: &BTreeMap<String, ScenarioValue>, field: &str) -> Option<f64> {
    match literal(input, field) {
        Some(Node::Number(number)) => Some(number.get()),
        _ => None,
    }
}

fn text_at<'a>(input: &'a BTreeMap<String, ScenarioValue>, field: &str) -> Option<&'a str> {
    match literal(input, field) {
        Some(Node::Text(value)) => Some(value.as_str()),
        _ => None,
    }
}

/// Asserts the overlap of `small` (declared first) and `flagged` is sent requiring `small`, or is
/// noted naming both.
fn assert_sent_or_noted(
    label: &str,
    synthesis: &Synthesis,
    inside: impl Fn(&BTreeMap<String, ScenarioValue>) -> bool,
) {
    let invocations = sent(&synthesis.suite);
    let witnessed = invocations
        .iter()
        .any(|(input, required)| required == "small" && inside(input));
    let noted = overlap_notes(synthesis)
        .iter()
        .any(|note| note.contains("`small`") && note.contains("`flagged`"));
    assert!(
        witnessed || noted,
        "{label}: the overlap of `small` and `flagged` is neither sent nor noted.\nsent: {:?}\nrefusals: {:?}\nnotes: {:?}",
        invocations
            .iter()
            .map(|(input, required)| format!(
                "{required} <- {:?}",
                input
                    .iter()
                    .filter_map(|(k, v)| match v {
                        ScenarioValue::Literal { value } => Some(format!("{k}={value:?}")),
                        _ => None,
                    })
                    .collect::<Vec<_>>()
            ))
            .collect::<Vec<_>>(),
        synthesis
            .refusals
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>(),
        overlap_notes(synthesis)
    );
}

/// Two Decimal leaves, each overlapping only between two literals less than two apart: the overlap
/// is `11 < x < 12 and 11 < y < 12`, reached only with both leaves at their midpoint together.
#[test]
fn an_overlap_needing_two_decimal_midpoints_together_is_sent_or_noted() {
    let inputs = "      - {name: x, type: Decimal}\n      - {name: y, type: Decimal}\n";
    let text = model(
        inputs,
        "{all: [x > 10, x < 12, y > 10, y < 12]}",
        "{all: [x > 11, x < 13, y > 11, y < 13]}",
    );
    let synthesis = synthesize(&ir(&text));
    assert_sent_or_noted("decimal x/y", &synthesis, |input| {
        num(input, "x").is_some_and(|x| x > 11.0 && x < 12.0)
            && num(input, "y").is_some_and(|y| y > 11.0 && y < 12.0)
    });
}

/// Three Integer leaves: `small: a == 7 and b == 7`, `flagged: b == 7 and c == 7`. The overlap is
/// `a == b == c == 7`, three leaves off their base at once.
#[test]
fn an_overlap_needing_three_integer_leaves_together_is_sent_or_noted() {
    let inputs = "      - {name: a, type: Integer}\n      - {name: b, type: Integer}\n      - {name: c, type: Integer}\n";
    let text = model(inputs, "{all: [a == 7, b == 7]}", "{all: [b == 7, c == 7]}");
    let synthesis = synthesize(&ir(&text));
    assert_sent_or_noted("integer a/b/c", &synthesis, |input| {
        ["a", "b", "c"]
            .iter()
            .all(|field| num(input, field).is_some_and(|v| (v - 7.0).abs() < f64::EPSILON))
    });
}

/// The same three-leaf overlap written through `not` over `any`, which `exhausts` accepts as plain.
#[test]
fn an_overlap_written_through_negated_disjunctions_is_sent_or_noted() {
    let inputs = "      - {name: a, type: Integer}\n      - {name: b, type: Integer}\n      - {name: c, type: Integer}\n";
    let text = model(
        inputs,
        "{not: {any: [a != 7, b != 7]}}",
        "{not: {any: [b != 7, c != 7]}}",
    );
    let synthesis = synthesize(&ir(&text));
    assert_sent_or_noted("negated a/b/c", &synthesis, |input| {
        ["a", "b", "c"]
            .iter()
            .all(|field| num(input, field).is_some_and(|v| (v - 7.0).abs() < f64::EPSILON))
    });
}

/// Three text leaves, each compared with one literal: the overlap is `s == p, t == q, u == r`.
#[test]
fn an_overlap_needing_three_text_leaves_together_is_sent_or_noted() {
    let inputs = "      - {name: s, type: String}\n      - {name: t, type: String}\n      - {name: u, type: String}\n";
    let text = model(
        inputs,
        "{all: [s == \"p\", t == \"q\"]}",
        "{all: [t == \"q\", u == \"r\"]}",
    );
    let synthesis = synthesize(&ir(&text));
    assert_sent_or_noted("text s/t/u", &synthesis, |input| {
        text_at(input, "s") == Some("p")
            && text_at(input, "t") == Some("q")
            && text_at(input, "u") == Some("r")
    });
}

/// A `Decimal` newtype whose invariant caps it below the literals' midpoint: `value < 11.5`. The
/// overlap `11 < amount < 12` still holds inputs the type admits (`11 < amount < 11.5`), but the
/// midpoint `11.5` the second pass adds is one the type refuses, so no candidate lies there.
#[test]
fn an_overlap_whose_midpoint_the_newtype_refuses_is_sent_or_noted() {
    let text = model(
        "      - {name: amount, type: demo.orders.Capped}\n",
        "{all: [amount > 10, amount < 12]}",
        "{all: [amount > 11, amount < 13]}",
    )
    .replace(
        "types:\n",
        "types:\n  - name: demo.orders.Capped\n    kind: newtype\n    of: Decimal\n    invariants: ['value < 11.5']\n",
    );
    let synthesis = synthesize(&ir(&text));
    assert_sent_or_noted("capped decimal", &synthesis, |input| {
        num(input, "amount").is_some_and(|amount| amount > 11.0 && amount < 11.5)
    });
}

/// The committed examples an adopter copies from: none gains a refusal or a note this unit added
/// (a shadow by an earlier accepting branch, `PrecededExternalEligibility`, an unwitnessed overlap).
#[test]
fn committed_examples_gain_no_precedence_refusal_or_overlap_note() {
    use std::path::{Path, PathBuf};
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let mut report = Vec::new();
    for example in ["billing", "gatepass", "oracle-fixture"] {
        let base = root.join("examples").join(example).canonicalize().unwrap();
        let mut found: Vec<PathBuf> = Vec::new();
        let mut pending = vec![base.clone()];
        while let Some(directory) = pending.pop() {
            for entry in std::fs::read_dir(&directory).unwrap() {
                let path = entry.unwrap().path();
                if path.is_dir() {
                    pending.push(path);
                } else if path.extension().is_some_and(|it| it == "yaml") {
                    found.push(path);
                }
            }
        }
        found.sort();
        let mut sources = SourceMap::new();
        let mut parsed = Vec::new();
        for path in found {
            let label = path.strip_prefix(&base).unwrap().display().to_string();
            let text = std::fs::read_to_string(&path).unwrap();
            let Ok(raw) = RawSpecFile::parse(&text) else {
                continue;
            };
            sources.insert(label.clone(), text);
            parsed.push((Source::new(label), raw));
        }
        let Ok(specification) = Specification::assemble(parsed) else {
            report.push(format!("{example}: does not assemble"));
            continue;
        };
        let Ok(model) = compile(&specification, &sources) else {
            report.push(format!("{example}: does not compile"));
            continue;
        };
        let synthesis = synthesize(&model);
        for refusal in &synthesis.refusals {
            let text = refusal.to_string();
            if text.contains("declared first") || text.contains("PrecededExternalEligibility") {
                report.push(format!("{example}: {text}"));
            }
        }
        for note in overlap_notes(&synthesis) {
            report.push(format!("{example}: {note}"));
        }
    }
    assert!(report.is_empty(), "{report:#?}");
}

/// The overlap over a required leaf of an optional structure: `exhausts` refuses an optional leaf,
/// but `box.amount` is required inside an `Optional` parent, so its leaf is not itself optional.
#[test]
fn an_overlap_under_an_optional_parent_is_sent_or_noted() {
    let text = model(
        "      - {name: box, type: Optional<demo.orders.Box>}\n",
        "{all: [box.amount > 10, box.amount < 12]}",
        "{all: [box.amount > 11, box.amount < 13]}",
    )
    .replace(
        "types:\n",
        "types:\n  - name: demo.orders.Box\n    kind: struct\n    fields:\n      - {name: amount, type: Decimal}\n",
    );
    let synthesis = synthesize(&ir(&text));
    assert_sent_or_noted("optional parent", &synthesis, |input| {
        match input.get("box") {
            Some(ScenarioValue::Literal {
                value: Node::Map(fields),
            }) => fields.iter().any(|(name, value)| {
                name.as_str() == "amount"
                    && matches!(value, Node::Number(n) if n.get() > 11.0 && n.get() < 12.0)
            }),
            _ => false,
        }
    });
}
