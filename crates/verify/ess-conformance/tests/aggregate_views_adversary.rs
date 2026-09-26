//! Adversarial cases for aggregate views (beyond10x/ess#96, wave unit D2).
//!
//! Each case states a claim `docs/design/aggregate-views.md` makes about the synthesized aggregate
//! scenario and checks it on a model the page's own rules admit. The expected numbers are computed
//! here from the inputs the scenario sends, not through `ess_conformance::aggregate`.
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
use std::collections::BTreeMap;

const METRICS: &str = include_str!("fixtures/aggregate-views.yaml");

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("metrics.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap()
}

fn synthesis(text: &str) -> Synthesis {
    synthesize(&ir(text))
}

fn with_views(text: &str, views: &str) -> String {
    let head = text.split_once("views:\n").unwrap().0;
    format!("{head}views:\n{views}")
}

fn find<'a>(result: &'a Synthesis, id: &str) -> Option<&'a ConformanceScenario> {
    result
        .suite
        .scenarios
        .iter()
        .find(|(key, _)| key.to_string() == id)
        .map(|(_, scenario)| scenario)
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

fn records(scenario: &ConformanceScenario) -> Vec<BTreeMap<String, ScenarioValue>> {
    scenario
        .steps
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::ExecuteCommand { command, input, .. }
                if command.to_string() == "metrics.session.Record" =>
            {
                Some(input.clone())
            }
            _ => None,
        })
        .collect()
}

fn integer(value: &ScenarioValue) -> i128 {
    match value {
        ScenarioValue::Literal {
            value: Node::Number(number),
        } => number.exact_text().parse().expect("an integer input"),
        other => panic!("not an integer literal: {other:?}"),
    }
}

fn number(text: &str) -> ScenarioValue {
    match FactValue::parse_literal(text) {
        FactValue::Number(number) => ScenarioValue::literal(Node::Number(number)),
        other => panic!("{other:?}"),
    }
}

/// `units × 10⁻⁶`, spelled as a decimal.
fn micro(units: i128) -> ScenarioValue {
    let whole = units / 1_000_000;
    let fraction = units % 1_000_000;
    number(&format!("{whole}.{fraction:06}"))
}

/// The A group's mean of `field` over its first `m` rows, truncated to 6 places — what the
/// page's "`avg` truncates instead of rounding" mutant reports — beside what the scenario asserts.
fn truncated_and_asserted(
    model: &str,
    view: &str,
    field: &str,
    output: &str,
) -> (ScenarioValue, ScenarioValue) {
    let result = synthesis(model);
    let scenario = find(&result, &format!("{view}/aggregate"))
        .unwrap_or_else(|| panic!("no aggregate scenario: {:?}", result.refusals));
    let rows = contains(scenario, view);
    let a = &rows[0];
    let sessions = integer(&a["sessions"]);
    let sent = records(scenario);
    let sum: i128 = sent
        .iter()
        .take(usize::try_from(sessions).unwrap())
        .map(|input| integer(&input[field]))
        .sum();
    (micro(sum * 1_000_000 / sessions), a[output].clone())
}

/// The page's mutant table: "`avg` truncates instead of rounding" is killed by the A row. It is
/// killed only when the arranged mean's seventh decimal is 5 or more. Averaging the second input
/// of the page's own example view (`wait_seconds`: 101, 101, 101, 103, 107, 113, mean
/// 104.3333…) asserts the value a truncating implementation also returns.
#[test]
fn avg_over_the_second_input_separates_rounding_from_truncation() {
    let view = "metrics.session.WaitByAgent";
    let model = with_views(
        METRICS,
        "  - name: metrics.session.WaitByAgent\n    source: metrics.session.Session\n    consistency: read_your_writes\n    filter: state == Completed\n    group_by: [agent_id]\n    fields:\n      - {name: agent_id, type: String}\n      - {name: sessions, type: Integer, aggregate: {count: {}}}\n      - {name: talk_seconds, type: Integer, aggregate: {sum: talk_seconds}}\n      - {name: mean_wait, type: Optional<Decimal>, aggregate: {avg: wait_seconds}}\n      - {name: distinct_callers, type: Integer, aggregate: {count_distinct: caller}}\n",
    );
    let (truncated, asserted) = truncated_and_asserted(&model, view, "wait_seconds", "mean_wait");
    assert_ne!(
        asserted, truncated,
        "the A row asserts `mean_wait` = {asserted:?}, which a truncating `avg` also reports"
    );
}

/// The same mutant at a group of seven, the implementor's own `Wide` view: 78 / 7 = 11.142857142…,
/// whose truncation is the asserted 11.142857.
#[test]
fn avg_over_a_group_of_seven_separates_rounding_from_truncation() {
    let view = "metrics.session.Wide";
    let model = with_views(
        METRICS,
        "  - name: metrics.session.Wide\n    source: metrics.session.Session\n    consistency: read_your_writes\n    filter: state == Completed\n    group_by: [agent_id]\n    fields:\n      - {name: agent_id, type: String}\n      - {name: sessions, type: Integer, aggregate: {count: {}}}\n      - {name: mean_talk, type: Optional<Decimal>, aggregate: {avg: talk_seconds}}\n      - {name: longest_wait, type: Optional<Integer>, aggregate: {max: wait_seconds}}\n      - {name: callers, type: Integer, aggregate: {count_distinct: caller}}\n      - {name: queues, type: Integer, aggregate: {count_distinct: queue_id}}\n      - {name: channels, type: Integer, aggregate: {count_distinct: channel}}\n",
    );
    let (truncated, asserted) = truncated_and_asserted(&model, view, "talk_seconds", "mean_talk");
    assert_ne!(
        asserted, truncated,
        "the A row asserts `mean_talk` = {asserted:?}, which a truncating `avg` also reports"
    );
}

/// `min`/`max` admit `String`, and an aggregate argument may be the identity. A generated identity
/// is chosen by the target, so no expected value can name it; the scenario must not assert one.
#[test]
fn min_over_a_generated_string_identity_asserts_no_invented_value() {
    let model = METRICS
        .replace(
            "fields: [{name: session_id, type: Uuid}]",
            "fields: [{name: session_id, type: String}]",
        )
        .replace(
            "identity: {name: session_id, type: Uuid}",
            "identity: {name: session_id, type: String}",
        )
        .replace(
            "input: [{name: session_id, type: Uuid}]",
            "input: [{name: session_id, type: String}]",
        );
    let view = "metrics.session.FirstSession";
    let model = with_views(
        &model,
        "  - name: metrics.session.FirstSession\n    source: metrics.session.Session\n    consistency: read_your_writes\n    filter: state == Completed\n    group_by: [agent_id]\n    fields:\n      - {name: agent_id, type: String}\n      - {name: first_session, type: Optional<String>, aggregate: {min: session_id}}\n",
    );
    let result = synthesis(&model);
    let Some(scenario) = find(&result, &format!("{view}/aggregate")) else {
        return; // Refused: no number is asserted.
    };
    for row in contains(scenario, view) {
        let value = &row["first_session"];
        assert!(
            !matches!(
                value,
                ScenarioValue::Literal { value: Node::Text(text) } if text.starts_with("row-")
            ),
            "the scenario asserts `first_session` = {value:?}, a placeholder no target generates"
        );
    }
}

/// Scoping: a group's key is `<view>/<g>`, a value no other scenario produces. A declared move
/// that the arrangement drives each row through may rewrite the key; the asserted groups must still
/// be scoped, or the scenario refused.
#[test]
fn a_move_that_rewrites_the_group_key_leaves_no_unscoped_group_asserted() {
    let model = METRICS
        .replace(
            "input: [{name: session_id, type: Uuid}]",
            "input: [{name: session_id, type: Uuid}, {name: handler, type: String}]",
        )
        .replace(
            "        moves: metrics.session.Session.complete\n        instance: session_id\n",
            "        moves: metrics.session.Session.complete\n        instance: session_id\n        sets: {agent_id: input.handler}\n",
        );
    let view = "metrics.session.TalkTimeByAgent";
    let result = synthesis(&model);
    let Some(scenario) = find(&result, &format!("{view}/aggregate")) else {
        return;
    };
    for row in contains(scenario, view) {
        let key = &row["agent_id"];
        assert!(
            matches!(
                key,
                ScenarioValue::Literal { value: Node::Text(text) } if text.starts_with(&format!("{view}/"))
            ),
            "an exact aggregate is asserted for the unscoped group `agent_id` = {key:?}: {row:?}"
        );
    }
}

/// "Group tuples": every key must be observably a key, so that an implementation ignoring it
/// merges two asserted groups. Bₖ is arranged only for keys after the first, so when the first key
/// is the non-scoped one (`group_by: [channel, agent_id]`) no two asserted groups share
/// `agent_id` and differ in `channel`, and ignoring `channel` changes no asserted row.
#[test]
fn ignoring_a_non_scoped_first_key_merges_two_asserted_groups() {
    let view = "metrics.session.ByChannelAgent";
    let model = with_views(
        METRICS,
        "  - name: metrics.session.ByChannelAgent\n    source: metrics.session.Session\n    consistency: read_your_writes\n    filter: state == Completed\n    group_by: [channel, agent_id]\n    fields:\n      - {name: channel, type: metrics.session.Channel}\n      - {name: agent_id, type: String}\n      - {name: sessions, type: Integer, aggregate: {count: {}}}\n",
    );
    let result = synthesis(&model);
    let scenario = find(&result, &format!("{view}/aggregate"))
        .unwrap_or_else(|| panic!("no aggregate scenario: {:?}", result.refusals));
    let rows = contains(scenario, view);
    let separated = rows.iter().enumerate().any(|(i, left)| {
        rows.iter().skip(i + 1).any(|right| {
            left["agent_id"] == right["agent_id"] && left["channel"] != right["channel"]
        })
    });
    assert!(
        separated,
        "no two asserted groups differ only in `channel`, so a target that ignores it passes: {rows:?}"
    );
}
