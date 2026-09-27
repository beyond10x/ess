//! Adversary pass 2 for beyond10x/ess#148: what the synthesized aggregate scenarios witness after
//! correction round 1 (one absent row per skipping input). Every expected value here is recomputed
//! from the `Place` inputs the scenario sends, without `ess_conformance::aggregate`.
use std::collections::BTreeMap;

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::{
    scenario::ViewExpectation,
    synthesize::{synthesize, Synthesis},
    ConformanceScenario, ScenarioStep, ScenarioValue,
};
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source,
};
use ess_primitives::{facts::FactValue, node::Node};

const ORDERS: &str = include_str!("fixtures/aggregate-optional-fields.yaml");

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("orders.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap()
}

fn synthesis(text: &str) -> Synthesis {
    synthesize(&ir(text))
}

fn scenario<'a>(synthesis: &'a Synthesis, id: &str) -> &'a ConformanceScenario {
    synthesis
        .suite
        .scenarios
        .iter()
        .find(|(key, _)| key.to_string() == id)
        .map_or_else(
            || {
                panic!(
                    "no scenario {id}; refusals: {:?}",
                    synthesis
                        .refusals
                        .iter()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                )
            },
            |(_, scenario)| scenario,
        )
}

type Fields = BTreeMap<String, ScenarioValue>;

fn places(scenario: &ConformanceScenario) -> Vec<Fields> {
    scenario
        .steps
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::ExecuteCommand { command, input, .. }
                if command.to_string() == "demo.orders.Place" =>
            {
                Some(input.clone())
            }
            _ => None,
        })
        .collect()
}

/// Every `Contains` read of `view`, with the parameters it was read under.
fn reads(scenario: &ConformanceScenario, view: &str) -> Vec<(Fields, Fields)> {
    let mut params = BTreeMap::new();
    let mut out = Vec::new();
    for step in &scenario.steps {
        match step {
            ScenarioStep::QueryView {
                view: read,
                params: bound,
            } if read.to_string() == view => params.clone_from(bound),
            ScenarioStep::ExpectView {
                view: read,
                expectation: ViewExpectation::Contains { fields },
            } if read.to_string() == view => out.push((params.clone(), fields.clone())),
            ScenarioStep::EventuallyView {
                view: read,
                params: bound,
                expectation: ViewExpectation::Contains { fields },
            } if read.to_string() == view => out.push((bound.clone(), fields.clone())),
            _ => {}
        }
    }
    out
}

fn node(value: Option<&ScenarioValue>) -> Node {
    value.map_or(Node::Null, |value| {
        value.as_literal().cloned().expect("a literal")
    })
}

fn int(node: &Node) -> i128 {
    match node {
        Node::Number(number) => i128::from(number.as_i64().expect("an integer")),
        other => panic!("{other:?}"),
    }
}

fn n(text: &str) -> Node {
    match FactValue::parse_literal(text) {
        FactValue::Number(number) => Node::Number(number),
        other => panic!("{other:?}"),
    }
}

fn num(value: i128) -> Node {
    n(&value.to_string())
}

/// The mean of integers to six places, half-even, spelled without trailing zeroes.
fn mean(values: &[i128]) -> Node {
    if values.is_empty() {
        return Node::Null;
    }
    let (sum, count) = (values.iter().sum::<i128>(), values.len() as i128);
    let scaled = sum * 1_000_000;
    let (mut q, r) = (scaled / count, scaled % count);
    if 2 * r > count || (2 * r == count && q % 2 == 1) {
        q += 1;
    }
    let text = format!("{}.{:06}", q / 1_000_000, q % 1_000_000);
    n(text.trim_end_matches('0').trim_end_matches('.'))
}

/// The present values of `field` over `rows`.
fn present(rows: &[&Fields], field: &str) -> Vec<Node> {
    rows.iter()
        .map(|row| node(row.get(field)))
        .filter(|value| *value != Node::Null)
        .collect()
}

fn extreme(values: Vec<Node>, max: bool) -> Node {
    let key = |value: &Node| match value {
        Node::Number(_) => format!("{:020}", int(value)),
        Node::Text(text) => text.clone(),
        other => panic!("{other:?}"),
    };
    let chosen = if max {
        values.into_iter().max_by_key(key)
    } else {
        values.into_iter().min_by_key(key)
    };
    chosen.unwrap_or(Node::Null)
}

/// The fixture with further optional inputs, each filled from the command's `Optional` input of the
/// same name, and a required integer `amount`.
fn model(views: &str) -> String {
    let fields = "      - {name: duration, type: Optional<Integer>}\n";
    let sets = "          duration: input.duration\n";
    let text = ORDERS
        .replace(
            fields,
            &format!(
                "{fields}      - {{name: wait, type: Optional<Integer>}}\n      - {{name: cost, type: Optional<Decimal>}}\n      - {{name: started, type: Optional<Timestamp>}}\n      - {{name: amount, type: Integer}}\n"
            ),
        )
        .replace(
            sets,
            &format!(
                "{sets}          wait: input.wait\n          cost: input.cost\n          started: input.started\n          amount: input.amount\n"
            ),
        );
    let head = text.split_once("views:\n").unwrap().0;
    format!("{head}views:\n{views}")
}

/// A parameter-scoped, ungrouped view whose only aggregates are skipping `min` and `max` over two
/// different optional fields. A target that filters the query to rows holding every skipping input
/// (`WHERE duration IS NOT NULL AND wait IS NOT NULL`, which SQL does not do: `MIN(duration)` keeps
/// a row whose `wait` is `NULL`) drops exactly the absent rows. Each absent row holds the other input
/// at its lowest A value, which moves neither a minimum nor a maximum, and the view has no `count`
/// or `sum` for the dropped row to move — so every asserted value is the same under both targets.
#[test]
fn adversary_a_min_max_only_view_catches_a_target_that_drops_rows_lacking_any_skipping_input() {
    let view = "demo.orders.Span";
    let text = model(&format!(
        "  - name: {view}\n    source: demo.orders.Order\n    params: [{{name: customer, type: String}}]\n    filter: customer == param.customer\n    fields:\n      - {{name: shortest, type: Optional<Integer>, aggregate: {{min: duration, skip_absent: true}}}}\n      - {{name: longest_wait, type: Optional<Integer>, aggregate: {{max: wait, skip_absent: true}}}}\n"
    ));
    let result = synthesis(&text);
    let span = scenario(&result, &format!("{view}/aggregate"));
    let sent = places(span);
    let mut told_apart = false;
    let mut seen = Vec::new();
    for (params, fields) in reads(span, view) {
        let scope = node(params.get("customer"));
        let rows: Vec<&Fields> = sent
            .iter()
            .filter(|row| node(row.get("customer")) == scope)
            .collect();
        let sql = (
            extreme(present(&rows, "duration"), false),
            extreme(present(&rows, "wait"), true),
        );
        let holding_both: Vec<&Fields> = rows
            .iter()
            .copied()
            .filter(|row| row.contains_key("duration") && row.contains_key("wait"))
            .collect();
        let wrong = (
            extreme(present(&holding_both, "duration"), false),
            extreme(present(&holding_both, "wait"), true),
        );
        let asserted = (
            node(fields.get("shortest")),
            node(fields.get("longest_wait")),
        );
        assert_eq!(asserted, sql, "the suite disagrees with SQL for {scope:?}");
        told_apart |= asserted != wrong;
        seen.push((scope, asserted, wrong));
    }
    assert!(!seen.is_empty(), "no Contains read of {view}");
    assert!(
        told_apart,
        "a target dropping rows that lack any skipping input reports every asserted value \
         (scope, asserted, dropping target): {seen:?}; rows sent: {sent:?}"
    );
}

/// Three skipping inputs of three kinds (`min` over `Decimal`, `max` over `Timestamp`, `sum` over
/// `Integer`), a required `avg` beside them, a `count`, and an optional group key, in one view. Every
/// fully asserted group is recomputed from the rows sent with SQL's treatment.
#[test]
fn adversary_three_skipping_kinds_beside_a_required_mean_and_an_optional_key_match_sql() {
    let view = "demo.orders.Mixed";
    let text = model(&format!(
        "  - name: {view}\n    source: demo.orders.Order\n    group_by: [customer, channel]\n    fields:\n      - {{name: customer, type: String}}\n      - {{name: channel, type: Optional<demo.orders.Channel>}}\n      - {{name: orders, type: Integer, aggregate: {{count: {{}}}}}}\n      - {{name: cheapest, type: Optional<Decimal>, aggregate: {{min: cost, skip_absent: true}}}}\n      - {{name: latest, type: Optional<Timestamp>, aggregate: {{max: started, skip_absent: true}}}}\n      - {{name: waited, type: Optional<Integer>, aggregate: {{sum: wait, skip_absent: true}}}}\n      - {{name: typical, type: Optional<Decimal>, aggregate: {{avg: amount}}}}\n"
    ));
    let result = synthesis(&text);
    let mixed = scenario(&result, &format!("{view}/aggregate"));
    let sent = places(mixed);
    let asserted = reads(mixed, view);
    assert!(asserted.len() >= 3, "{asserted:?}");
    let mut null_channel = false;
    for (_, fields) in &asserted {
        let key = (node(fields.get("customer")), node(fields.get("channel")));
        null_channel |= key.1 == Node::Null;
        if !fields.contains_key("orders") {
            continue;
        }
        let rows: Vec<&Fields> = sent
            .iter()
            .filter(|row| (node(row.get("customer")), node(row.get("channel"))) == key)
            .collect();
        let amounts: Vec<i128> = present(&rows, "amount").iter().map(int).collect();
        let waits = present(&rows, "wait");
        let expected = [
            ("orders", num(rows.len() as i128)),
            ("cheapest", extreme(present(&rows, "cost"), false)),
            ("latest", extreme(present(&rows, "started"), true)),
            (
                "waited",
                if waits.is_empty() {
                    Node::Null
                } else {
                    num(waits.iter().map(int).sum())
                },
            ),
            ("typical", mean(&amounts)),
        ];
        for (field, value) in expected {
            assert_eq!(
                node(fields.get(field)),
                value,
                "`{field}` of group {key:?}; rows: {rows:?}"
            );
        }
    }
    assert!(null_channel, "no null channel group: {asserted:?}");
}
