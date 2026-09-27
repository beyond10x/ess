//! Adversary cases for beyond10x/ess#148: what the synthesized aggregate scenarios witness.
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

fn places(scenario: &ConformanceScenario) -> Vec<BTreeMap<String, ScenarioValue>> {
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

fn contains(scenario: &ConformanceScenario, view: &str) -> Vec<BTreeMap<String, ScenarioValue>> {
    scenario
        .steps
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::ExpectView {
                view: read,
                expectation,
            }
            | ScenarioStep::EventuallyView {
                view: read,
                expectation,
                ..
            } if read.to_string() == view => match expectation {
                ViewExpectation::Contains { fields } => Some(fields.clone()),
                _ => None,
            },
            _ => None,
        })
        .collect()
}

fn n(text: &str) -> ScenarioValue {
    match FactValue::parse_literal(text) {
        FactValue::Number(number) => ScenarioValue::literal(Node::Number(number)),
        other => panic!("{other:?}"),
    }
}

fn null() -> ScenarioValue {
    ScenarioValue::literal(Node::Null)
}

/// The fixture with a second optional integer `wait`, filled from the command's optional input.
fn with_wait() -> String {
    ORDERS
        .replace(
            "      - {name: duration, type: Optional<Integer>}\n",
            "      - {name: duration, type: Optional<Integer>}\n      - {name: wait, type: Optional<Integer>}\n",
        )
        .replace(
            "          duration: input.duration\n",
            "          duration: input.duration\n          wait: input.wait\n",
        )
}

/// Two skipping aggregates over two different optional fields are skipped column by column (SQL:
/// `SUM(a)` skips a row where `a` is `NULL` whatever `b` holds). The arrangement's only absent row
/// lacks *every* skipping input at once, so a target that drops a row lacking any of them
/// (`WHERE duration IS NOT NULL AND wait IS NOT NULL`, or `if r.Duration == nil || r.Wait == nil
/// { continue }`) computes every asserted value exactly as SQL does. Some arranged row must lack one
/// skipping input and hold the other.
#[test]
fn adversary_two_skipping_inputs_are_witnessed_absent_one_at_a_time() {
    let text = with_wait();
    let head = text.split_once("views:\n").unwrap().0;
    let model = format!(
        "{head}views:\n  - name: demo.orders.Both\n    source: demo.orders.Order\n    group_by: [customer]\n    fields:\n      - {{name: customer, type: String}}\n      - {{name: total, type: Optional<Integer>, aggregate: {{sum: duration, skip_absent: true}}}}\n      - {{name: waited, type: Optional<Integer>, aggregate: {{sum: wait, skip_absent: true}}}}\n"
    );
    let result = synthesis(&model);
    let both = scenario(&result, "demo.orders.Both/aggregate");
    let rows: Vec<(bool, bool)> = places(both)
        .iter()
        .map(|input| (input.contains_key("duration"), input.contains_key("wait")))
        .collect();
    assert!(
        rows.iter().any(|(duration, wait)| duration != wait),
        "every arranged row holds both skipping inputs or neither (duration, wait present): \
         {rows:?}; a target that drops a row lacking either passes every assertion: {:?}",
        contains(both, "demo.orders.Both")
    );
}

/// `count_distinct` over the optional key itself, skipping absent values: in the key's null group
/// there is no present value, so it is 0; in every other group it is 1.
#[test]
fn adversary_a_skipping_distinct_count_over_an_optional_key_is_zero_in_its_null_group() {
    let head = ORDERS.split_once("views:\n").unwrap().0;
    let model = format!(
        "{head}views:\n  - name: demo.orders.Channels\n    source: demo.orders.Order\n    group_by: [customer, channel]\n    fields:\n      - {{name: customer, type: String}}\n      - {{name: channel, type: Optional<demo.orders.Channel>}}\n      - {{name: orders, type: Integer, aggregate: {{count: {{}}}}}}\n      - {{name: channels, type: Integer, aggregate: {{count_distinct: channel, skip_absent: true}}}}\n"
    );
    let result = synthesis(&model);
    let channels = scenario(&result, "demo.orders.Channels/aggregate");
    let rows = contains(channels, "demo.orders.Channels");
    let null_group: Vec<_> = rows
        .iter()
        .filter(|row| row.get("channel") == Some(&null()))
        .collect();
    assert_eq!(null_group.len(), 1, "{rows:?}");
    assert_eq!(null_group[0].get("channels"), Some(&n("0")), "{rows:?}");
    for row in rows
        .iter()
        .filter(|row| row.get("channel") != Some(&null()))
    {
        assert_eq!(row.get("channels"), Some(&n("1")), "{rows:?}");
    }
}

/// A parameter-scoped view grouped by an optional key: the scope keeps every group to this
/// scenario's rows, so the null group is asserted exactly, with its count.
#[test]
fn adversary_a_parameter_scoped_view_asserts_its_null_group_exactly() {
    let head = ORDERS.split_once("views:\n").unwrap().0;
    let model = format!(
        "{head}views:\n  - name: demo.orders.ByChannelFor\n    source: demo.orders.Order\n    params: [{{name: customer, type: String}}]\n    filter: customer == param.customer\n    group_by: [channel]\n    fields:\n      - {{name: channel, type: Optional<demo.orders.Channel>}}\n      - {{name: orders, type: Integer, aggregate: {{count: {{}}}}}}\n      - {{name: total, type: Optional<Integer>, aggregate: {{sum: duration, skip_absent: true}}}}\n"
    );
    let result = synthesis(&model);
    let by_channel = scenario(&result, "demo.orders.ByChannelFor/aggregate");
    let rows = contains(by_channel, "demo.orders.ByChannelFor");
    let null_group: Vec<_> = rows
        .iter()
        .filter(|row| row.get("channel") == Some(&null()))
        .collect();
    assert_eq!(null_group.len(), 1, "{rows:?}");
    assert_eq!(null_group[0].get("orders"), Some(&n("1")), "{rows:?}");
}
