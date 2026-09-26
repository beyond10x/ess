//! An aggregate view is synthesized as one scenario over rows the scenario made (beyond10x/ess#96).
//!
//! `docs/design/aggregate-views.md`, "Conformance" and "Deciding checks" 4, 6, 8, 10 and 11. Every
//! number asserted here is the page's own worked example, read off the page and not off the
//! implementation.
use std::collections::BTreeMap;

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::{
    aggregate,
    coverage::{Origins, Scope},
    coverage_build,
    scenario::{ScenarioId, SuiteFormat, ViewExpectation},
    synthesize::{synthesize, Synthesis},
    AdmittedSuite, ConformanceScenario, ConformanceSuite, ScenarioStep, ScenarioValue,
};
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source,
};
use ess_primitives::{facts::FactValue, node::Node};

const METRICS: &str = include_str!("fixtures/aggregate-views.yaml");
const BY_AGENT: &str = "metrics.session.TalkTimeByAgent";
const TOTALS: &str = "metrics.session.QueueTotals";
const BY_CHANNEL: &str = "metrics.session.SessionsByAgentChannel";

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("metrics.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap()
}

fn synthesis(text: &str) -> Synthesis {
    synthesize(&ir(text))
}

fn scenario<'a>(suite: &'a ConformanceSuite, id: &str) -> &'a ConformanceScenario {
    suite
        .scenarios
        .iter()
        .find(|(key, _)| key.to_string() == id)
        .map_or_else(|| panic!("no scenario {id}"), |(_, scenario)| scenario)
}

fn text(value: &str) -> ScenarioValue {
    ScenarioValue::literal(Node::Text(value.to_owned()))
}

fn number(value: &str) -> ScenarioValue {
    match FactValue::parse_literal(value) {
        FactValue::Number(number) => ScenarioValue::literal(Node::Number(number)),
        other => panic!("{other:?}"),
    }
}

fn null() -> ScenarioValue {
    ScenarioValue::literal(Node::Null)
}

fn row(entries: &[(&str, ScenarioValue)]) -> BTreeMap<String, ScenarioValue> {
    entries
        .iter()
        .map(|(name, value)| ((*name).to_owned(), value.clone()))
        .collect()
}

/// Every read of `view` in step order: the parameters bound and the expectation.
fn reads(
    scenario: &ConformanceScenario,
    view: &str,
) -> Vec<(BTreeMap<String, ScenarioValue>, ViewExpectation)> {
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
                expectation,
            } if read.to_string() == view => out.push((params.clone(), expectation.clone())),
            ScenarioStep::EventuallyView {
                view: read,
                params: bound,
                expectation,
            } if read.to_string() == view => out.push((bound.clone(), expectation.clone())),
            _ => {}
        }
    }
    out
}

fn contains(scenario: &ConformanceScenario, view: &str) -> Vec<BTreeMap<String, ScenarioValue>> {
    reads(scenario, view)
        .into_iter()
        .filter_map(|(_, expectation)| match expectation {
            ViewExpectation::Contains { fields } => Some(fields),
            _ => None,
        })
        .collect()
}

fn excludes(scenario: &ConformanceScenario, view: &str) -> Vec<BTreeMap<String, ScenarioValue>> {
    reads(scenario, view)
        .into_iter()
        .filter_map(|(_, expectation)| match expectation {
            ViewExpectation::Excludes { fields } => Some(fields),
            _ => None,
        })
        .collect()
}

/// The inputs every `Record` of the scenario sent, in step order.
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

fn scoped(view: &str, group: &str) -> ScenarioValue {
    text(&format!("{view}/{group}"))
}

#[test]
fn the_example_yields_one_aggregate_scenario_per_view_and_selects_suite_16() {
    let result = synthesis(METRICS);
    let ids: Vec<String> = result
        .suite
        .scenarios
        .keys()
        .map(ToString::to_string)
        .filter(|id| id.ends_with("/aggregate"))
        .collect();
    assert_eq!(
        ids,
        [
            "metrics.session.QueueTotals/aggregate",
            "metrics.session.SessionsByAgentChannel/aggregate",
            "metrics.session.TalkTimeByAgent/aggregate",
        ]
    );
    let aggregate_refusals: Vec<String> = result
        .refusals
        .iter()
        .filter(|refusal| aggregate::is_aggregate_refusal(&refusal.code().to_string()))
        .map(ToString::to_string)
        .collect();
    assert!(aggregate_refusals.is_empty(), "{aggregate_refusals:?}");
    assert_eq!(
        result.suite.provenance.suite_version,
        SuiteFormat::parse("ess-conformance/16").unwrap()
    );
    assert!(aggregate::used_by(&result.suite));
    assert_eq!(
        ScenarioId::parse("metrics.session.TalkTimeByAgent/aggregate")
            .unwrap()
            .to_string(),
        "metrics.session.TalkTimeByAgent/aggregate"
    );
}

#[test]
fn the_page_example_asserts_the_page_numbers() {
    let suite = synthesis(METRICS).suite;
    let by_agent = scenario(&suite, &format!("{BY_AGENT}/aggregate"));
    let rows = contains(by_agent, BY_AGENT);
    assert_eq!(
        rows,
        vec![
            row(&[
                ("agent_id", scoped(BY_AGENT, "A")),
                ("sessions", number("6")),
                ("talk_seconds", number("46")),
                ("longest_wait", number("113")),
                ("distinct_callers", number("3")),
                ("mean_talk", number("7.666667")),
            ]),
            row(&[
                ("agent_id", scoped(BY_AGENT, "B")),
                ("sessions", number("1")),
                ("talk_seconds", number("85")),
                ("longest_wait", number("185")),
                ("distinct_callers", number("1")),
                ("mean_talk", number("85")),
            ]),
        ]
    );
    assert_eq!(
        excludes(by_agent, BY_AGENT),
        vec![row(&[("agent_id", scoped(BY_AGENT, "C"))])]
    );

    // The rows, in creation order: six in A, then b, x and c.
    let sent = records(by_agent);
    assert_eq!(sent.len(), 9, "{sent:?}");
    let column = |field: &str| -> Vec<ScenarioValue> {
        sent.iter().map(|input| input[field].clone()).collect()
    };
    assert_eq!(
        column("talk_seconds"),
        ["1", "1", "3", "7", "13", "21", "85", "97", "93"].map(number)
    );
    assert_eq!(
        column("wait_seconds"),
        ["101", "101", "101", "103", "107", "113", "185", "197", "193"].map(number)
    );
    let caller = |n: &str| text(&format!("{BY_AGENT}/caller/{n}"));
    assert_eq!(
        column("caller"),
        ["201", "201", "201", "201", "203", "207", "285", "297", "293"].map(caller)
    );
    assert_eq!(
        column("agent_id"),
        ["A", "A", "A", "A", "A", "A", "B", "A", "C"].map(|g| scoped(BY_AGENT, g))
    );
    // Only the admitted rows are completed: x and c are left `Open`.
    let completes = by_agent
        .steps
        .iter()
        .filter(|step| {
            matches!(step, ScenarioStep::ExecuteCommand { command, .. }
                if command.to_string() == "metrics.session.Complete")
        })
        .count();
    assert_eq!(completes, 7);
}

#[test]
fn an_ungrouped_view_is_read_twice_once_for_its_rows_and_once_empty() {
    let suite = synthesis(METRICS).suite;
    let totals = scenario(&suite, &format!("{TOTALS}/aggregate"));
    let reads = reads(totals, TOTALS);
    let one = ViewExpectation::Counts {
        at_least: Some(1),
        at_most: Some(1),
    };
    assert_eq!(
        reads,
        vec![
            (
                row(&[("queue_id", scoped(TOTALS, "in"))]),
                ViewExpectation::Contains {
                    fields: row(&[("sessions", number("3")), ("longest_wait", number("3"))]),
                }
            ),
            (row(&[("queue_id", scoped(TOTALS, "in"))]), one.clone()),
            (
                row(&[("queue_id", scoped(TOTALS, "empty"))]),
                ViewExpectation::Contains {
                    fields: row(&[("sessions", number("0")), ("longest_wait", null())]),
                }
            ),
            (row(&[("queue_id", scoped(TOTALS, "empty"))]), one),
        ]
    );
    // Three A rows and x: x keeps the scoped queue and is refuted by its state.
    let sent = records(totals);
    assert_eq!(sent.len(), 4);
    assert!(sent
        .iter()
        .all(|input| input["queue_id"] == scoped(TOTALS, "in")));
}

#[test]
fn a_second_key_gets_a_row_that_differs_from_b_in_that_key() {
    let suite = synthesis(METRICS).suite;
    let by_channel = scenario(&suite, &format!("{BY_CHANNEL}/aggregate"));
    let rows = contains(by_channel, BY_CHANNEL);
    assert_eq!(
        rows,
        vec![
            row(&[
                ("agent_id", scoped(BY_CHANNEL, "A")),
                ("channel", text("Voice")),
                ("sessions", number("3")),
                ("talk_seconds", number("5")),
            ]),
            row(&[
                ("agent_id", scoped(BY_CHANNEL, "B")),
                ("channel", text("Chat")),
                ("sessions", number("1")),
                ("talk_seconds", number("85")),
            ]),
            row(&[
                ("agent_id", scoped(BY_CHANNEL, "B")),
                ("channel", text("Voice")),
                ("sessions", number("1")),
                ("talk_seconds", number("87")),
            ]),
        ]
    );
    assert_eq!(
        excludes(by_channel, BY_CHANNEL),
        vec![row(&[
            ("agent_id", scoped(BY_CHANNEL, "C")),
            ("channel", text("Voice")),
        ])]
    );
}

/// The fixture with its `views:` replaced.
fn with_views(views: &str) -> String {
    let head = METRICS.split_once("views:\n").unwrap().0;
    format!("{head}views:\n{views}")
}

#[test]
fn an_enum_only_keyed_view_is_refused_as_unscoped() {
    let model = with_views(
        "  - name: metrics.session.ByChannel\n    source: metrics.session.Session\n    group_by: [channel]\n    fields:\n      - {name: channel, type: metrics.session.Channel}\n      - {name: sessions, type: Integer, aggregate: {count: {}}}\n",
    );
    let result = synthesis(&model);
    let refused: Vec<String> = result
        .refusals
        .iter()
        .filter(|refusal| refusal.code().to_string() == "ESS-SYNTH-016")
        .map(|refusal| refusal.scenario.as_ref().unwrap().to_string())
        .collect();
    assert_eq!(refused, ["metrics.session.ByChannel/aggregate"]);
    assert!(!result
        .suite
        .scenarios
        .keys()
        .any(|id| id.to_string().starts_with("metrics.session.ByChannel")));
}

#[test]
fn no_row_level_scenario_reads_an_aggregate_view() {
    // Grouped by the identity and `state`: every row-level picker's field checks would pass.
    let model = with_views(
        "  - name: metrics.session.PerSession\n    source: metrics.session.Session\n    consistency: read_your_writes\n    group_by: [session_id, state, agent_id]\n    fields:\n      - {name: session_id, type: Uuid}\n      - {name: state, type: metrics.session.Session.State}\n      - {name: agent_id, type: String}\n      - {name: sessions, type: Integer, aggregate: {count: {}}}\n",
    );
    let suite = synthesis(&model).suite;
    for (id, scenario) in &suite.scenarios {
        if id.to_string() == "metrics.session.PerSession/aggregate" {
            continue;
        }
        let named = scenario.steps.iter().any(|step| {
            serde_json::to_string(step)
                .unwrap()
                .contains("metrics.session.PerSession")
        });
        assert!(
            !named,
            "`{id}` reads the aggregate view: {:?}",
            scenario.steps
        );
        assert!(
            !scenario
                .source
                .iter()
                .any(|source| source.to_string().contains("metrics.session.PerSession")),
            "`{id}` depends on the aggregate view"
        );
    }
}

#[test]
fn a_boolean_second_key_gets_the_negated_value_and_a_one_variant_key_gets_no_row() {
    let model = METRICS
        .replace(
            "      - {name: wait_seconds, type: Integer}\n    lifecycle:",
            "      - {name: wait_seconds, type: Integer}\n      - {name: priority, type: Boolean}\n      - {name: tier, type: metrics.session.Tier}\n    lifecycle:",
        )
        .replace(
            "      - {name: wait_seconds, type: Integer}\n    outcomes:",
            "      - {name: wait_seconds, type: Integer}\n      - {name: priority, type: Boolean}\n      - {name: tier, type: metrics.session.Tier}\n    outcomes:",
        )
        .replace(
            "          wait_seconds: input.wait_seconds\n",
            "          wait_seconds: input.wait_seconds\n          priority: input.priority\n          tier: input.tier\n",
        )
        .replace(
            "types:\n",
            "types:\n  - name: metrics.session.Tier\n    kind: enum\n    variants: [Gold]\n",
        );
    let model = with_views_in(
        &model,
        "  - name: metrics.session.ByPriority\n    source: metrics.session.Session\n    consistency: read_your_writes\n    filter: state == Completed\n    group_by: [agent_id, priority, tier]\n    fields:\n      - {name: agent_id, type: String}\n      - {name: priority, type: Boolean}\n      - {name: tier, type: metrics.session.Tier}\n      - {name: sessions, type: Integer, aggregate: {count: {}}}\n",
    );
    let suite = synthesis(&model).suite;
    let view = "metrics.session.ByPriority";
    let rows = contains(scenario(&suite, &format!("{view}/aggregate")), view);
    let bool_value = |value: bool| ScenarioValue::literal(Node::Bool(value));
    assert_eq!(
        rows,
        vec![
            row(&[
                ("agent_id", scoped(view, "A")),
                ("priority", bool_value(false)),
                ("tier", text("Gold")),
                ("sessions", number("3")),
            ]),
            row(&[
                ("agent_id", scoped(view, "B")),
                ("priority", bool_value(true)),
                ("tier", text("Gold")),
                ("sessions", number("1")),
            ]),
            row(&[
                ("agent_id", scoped(view, "B")),
                ("priority", bool_value(false)),
                ("tier", text("Gold")),
                ("sessions", number("1")),
            ]),
        ],
        "B₂ negates B's `priority`; `tier` has one value, so no row is arranged for it"
    );
}

fn with_views_in(text: &str, views: &str) -> String {
    let head = text.split_once("views:\n").unwrap().0;
    format!("{head}views:\n{views}")
}

#[test]
fn an_aggregate_over_a_group_key_takes_the_keys_value() {
    let model = with_views(
        "  - name: metrics.session.Agents\n    source: metrics.session.Session\n    consistency: read_your_writes\n    filter: state == Completed\n    group_by: [agent_id]\n    fields:\n      - {name: agent_id, type: String}\n      - {name: agents, type: Integer, aggregate: {count_distinct: agent_id}}\n      - {name: sessions, type: Integer, aggregate: {count: {}}}\n",
    );
    let suite = synthesis(&model).suite;
    let view = "metrics.session.Agents";
    let rows = contains(scenario(&suite, &format!("{view}/aggregate")), view);
    assert_eq!(rows.len(), 2);
    for held in rows {
        assert_eq!(held["agents"], number("1"), "{held:?}");
    }
    assert_eq!(
        contains(scenario(&suite, &format!("{view}/aggregate")), view)[0]["sessions"],
        number("3"),
        "no numbered input: m = 3"
    );
}

#[test]
fn a_filter_made_only_of_the_scoping_equality_is_refuted_through_the_out_value() {
    let model = with_views(
        "  - name: metrics.session.InQueue\n    source: metrics.session.Session\n    consistency: read_your_writes\n    params: [{name: queue_id, type: String}]\n    filter: queue_id == param.queue_id\n    fields:\n      - {name: sessions, type: Integer, aggregate: {count: {}}}\n",
    );
    let result = synthesis(&model);
    let view = "metrics.session.InQueue";
    assert!(
        !result
            .refusals
            .iter()
            .any(|refusal| refusal.code().to_string() == "ESS-SYNTH-017"),
        "{:?}",
        result.refusals
    );
    let sent = records(scenario(&result.suite, &format!("{view}/aggregate")));
    let queues: Vec<ScenarioValue> = sent.iter().map(|input| input["queue_id"].clone()).collect();
    assert_eq!(
        queues,
        [
            scoped(view, "in"),
            scoped(view, "in"),
            scoped(view, "in"),
            scoped(view, "out")
        ]
    );
}

#[test]
fn a_parameter_read_inside_a_disjunction_is_refused_as_unwitnessed() {
    let model = with_views(
        "  - name: metrics.session.Either\n    source: metrics.session.Session\n    params: [{name: queue_id, type: String}]\n    filter: {any: [queue_id == param.queue_id, state == Completed]}\n    fields:\n      - {name: sessions, type: Integer, aggregate: {count: {}}}\n",
    );
    let result = synthesis(&model);
    assert!(
        result
            .refusals
            .iter()
            .any(|refusal| refusal.code().to_string() == "ESS-SYNTH-017"
                && refusal.to_string().contains("metrics.session.Either")),
        "{:?}",
        result.refusals
    );
}

#[test]
fn a_scoped_value_the_model_writes_as_a_literal_takes_the_next_suffix() {
    let model = METRICS.replace(
        "    filter: state == Completed\n    group_by: [agent_id]\n",
        "    filter: [state == Completed, agent_id != \"metrics.session.TalkTimeByAgent/A\"]\n    group_by: [agent_id]\n",
    );
    let suite = synthesis(&model).suite;
    let rows = contains(scenario(&suite, &format!("{BY_AGENT}/aggregate")), BY_AGENT);
    assert_eq!(rows[0]["agent_id"], scoped(BY_AGENT, "A#2"));
    assert_eq!(rows[0]["sessions"], number("6"));
}

#[test]
fn a_mean_is_moved_until_rounding_and_truncation_differ() {
    // Five inputs: m = 7, and `talk_seconds` in A is 1, 1, 3, 7, 13, 21, 31 — a sum of 77, whose
    // mean 11 ends. The last A value is raised by the smallest δ that leaves a seventh decimal of 5
    // or more: 35, and 81 / 7 = 11.5714285…, which rounds to 11.571429 and truncates to 11.571428.
    let model = with_views(
        "  - name: metrics.session.Wide\n    source: metrics.session.Session\n    consistency: read_your_writes\n    filter: state == Completed\n    group_by: [agent_id]\n    fields:\n      - {name: agent_id, type: String}\n      - {name: mean_talk, type: Optional<Decimal>, aggregate: {avg: talk_seconds}}\n      - {name: longest_wait, type: Optional<Integer>, aggregate: {max: wait_seconds}}\n      - {name: callers, type: Integer, aggregate: {count_distinct: caller}}\n      - {name: queues, type: Integer, aggregate: {count_distinct: queue_id}}\n      - {name: channels, type: Integer, aggregate: {count_distinct: channel}}\n",
    );
    let suite = synthesis(&model).suite;
    let view = "metrics.session.Wide";
    let wide = scenario(&suite, &format!("{view}/aggregate"));
    let talk: Vec<ScenarioValue> = records(wide)
        .iter()
        .take(7)
        .map(|input| input["talk_seconds"].clone())
        .collect();
    assert_eq!(talk, ["1", "1", "3", "7", "13", "21", "35"].map(number));
    let rows = contains(wide, view);
    assert_eq!(rows[0]["mean_talk"], number("11.571429"));
    assert_eq!(rows[0]["callers"], number("4"), "d₂ = m − 1 − 2");
    assert_eq!(rows[0]["queues"], number("3"), "d₃ = m − 1 − 3");
    // Ordinals 401 and 403 are both odd: an enum input collapses to the values its type has.
    assert_eq!(rows[0]["channels"], number("1"));
}

#[test]
fn the_conformance_catalog_names_each_views_grouping_and_aggregates() {
    let ir = ir(METRICS);
    let suite = synthesize(&ir).suite;
    let emitted = ess_conformance::web::emit(&ir, &suite).unwrap_or_else(|error| panic!("{error}"));
    let model: serde_json::Value = serde_json::from_str(&emitted["model.json"].contents).unwrap();
    let views = model["views"].as_array().unwrap();
    let by_agent = views.iter().find(|view| view["name"] == BY_AGENT).unwrap();
    assert_eq!(by_agent["group_by"], serde_json::json!(["agent_id"]));
    assert_eq!(by_agent["aggregates"]["sessions"], "count()");
    assert_eq!(by_agent["aggregates"]["mean_talk"], "avg(talk_seconds)");
    let totals = views.iter().find(|view| view["name"] == TOTALS).unwrap();
    assert_eq!(totals["group_by"], serde_json::json!([]));
}

/// The `ESS-SYNTH-017` refusals of `view`, rendered.
fn unwitnessed(model: &str, view: &str) -> Vec<String> {
    synthesis(model)
        .refusals
        .iter()
        .filter(|refusal| refusal.code().to_string() == "ESS-SYNTH-017")
        .map(ToString::to_string)
        .filter(|rendered| rendered.contains(view))
        .collect()
}

#[test]
fn an_extreme_or_mean_over_the_identity_is_refused_and_its_distinct_count_is_not() {
    let model = with_views(
        "  - name: metrics.session.Ids\n    source: metrics.session.Session\n    consistency: read_your_writes\n    filter: state == Completed\n    group_by: [agent_id]\n    fields:\n      - {name: agent_id, type: String}\n      - {name: ids, type: Integer, aggregate: {count_distinct: session_id}}\n",
    );
    assert!(unwitnessed(&model, "metrics.session.Ids").is_empty());
    let rows = contains(
        scenario(&synthesis(&model).suite, "metrics.session.Ids/aggregate"),
        "metrics.session.Ids",
    );
    assert_eq!(rows[0]["ids"], number("3"), "one identity per row");
    let text = METRICS.replace(
        "{name: session_id, type: Uuid}",
        "{name: session_id, type: String}",
    );
    let model = with_views_in(
        &text,
        "  - name: metrics.session.First\n    source: metrics.session.Session\n    filter: state == Completed\n    group_by: [agent_id]\n    fields:\n      - {name: agent_id, type: String}\n      - {name: first, type: Optional<String>, aggregate: {min: session_id}}\n",
    );
    let refused = unwitnessed(&model, "metrics.session.First");
    assert_eq!(refused.len(), 1, "{refused:?}");
    assert!(refused[0].contains("the target generates"), "{refused:?}");
}

#[test]
fn a_move_that_rewrites_a_group_key_refuses_the_view() {
    let model = METRICS
        .replace(
            "input: [{name: session_id, type: Uuid}]",
            "input: [{name: session_id, type: Uuid}, {name: handler, type: String}]",
        )
        .replace(
            "        moves: metrics.session.Session.complete\n        instance: session_id\n",
            "        moves: metrics.session.Session.complete\n        instance: session_id\n        sets: {agent_id: input.handler}\n",
        );
    let refused = unwitnessed(&model, BY_AGENT);
    assert!(
        refused.iter().any(|it| it.contains("rewrites `agent_id`")),
        "{refused:?}"
    );
}

#[test]
fn a_mean_over_a_key_every_row_shares_is_refused_rather_than_asserted() {
    let model = with_views(
        "  - name: metrics.session.Level\n    source: metrics.session.Session\n    consistency: read_your_writes\n    filter: state == Completed\n    group_by: [agent_id, talk_seconds]\n    fields:\n      - {name: agent_id, type: String}\n      - {name: talk_seconds, type: Integer}\n      - {name: mean, type: Optional<Decimal>, aggregate: {avg: talk_seconds}}\n",
    );
    let refused = unwitnessed(&model, "metrics.session.Level");
    assert!(
        refused
            .iter()
            .any(|it| it.contains("truncating `avg` also reports")),
        "{refused:?}"
    );
}

#[test]
fn a_non_scoped_first_key_gets_its_own_b_row() {
    let model = with_views(
        "  - name: metrics.session.ByChannelAgent\n    source: metrics.session.Session\n    consistency: read_your_writes\n    filter: state == Completed\n    group_by: [channel, agent_id]\n    fields:\n      - {name: channel, type: metrics.session.Channel}\n      - {name: agent_id, type: String}\n      - {name: sessions, type: Integer, aggregate: {count: {}}}\n",
    );
    let view = "metrics.session.ByChannelAgent";
    let rows = contains(
        scenario(&synthesis(&model).suite, &format!("{view}/aggregate")),
        view,
    );
    let tuples: Vec<(ScenarioValue, ScenarioValue)> = rows
        .iter()
        .map(|held| (held["channel"].clone(), held["agent_id"].clone()))
        .collect();
    assert_eq!(
        tuples,
        [
            (text("Voice"), scoped(view, "A")),
            (text("Chat"), scoped(view, "B")),
            (text("Voice"), scoped(view, "B")),
            (text("Chat"), scoped(view, "B2")),
        ]
    );
}

#[test]
fn an_explicitly_pinned_older_suite_with_an_aggregate_scenario_is_refused() {
    let mut suite = synthesis(METRICS).suite;
    suite.provenance.suite_version = SuiteFormat::parse("ess-conformance/15").unwrap();
    let error = ess_conformance::admission::suite(&suite).expect_err("refused");
    assert_eq!(error.issues[0].reason, "UnsupportedVocabulary");
    assert!(error.to_string().contains("suite/16"), "{error}");
    assert!(aggregate::admit_suite(&suite).is_err());
    suite.provenance.suite_version = SuiteFormat::parse("ess-conformance/16").unwrap();
    assert!(aggregate::admit_suite(&suite).is_ok());
}

#[test]
fn a_refusal_only_suite_carrying_an_aggregate_refusal_is_written_at_coverage_17() {
    let model = with_views(
        "  - name: metrics.session.ByChannel\n    source: metrics.session.Session\n    group_by: [channel]\n    fields:\n      - {name: channel, type: metrics.session.Channel}\n      - {name: sessions, type: Integer, aggregate: {count: {}}}\n",
    );
    let input = coverage_build::build(&ir(&model), &[], Scope::System, Origins::Generated)
        .unwrap_or_else(|error| panic!("{error:?}"));
    assert_eq!(
        input.selected().suite().provenance.suite_version.major(),
        aggregate::COVERAGE
    );
    let original = input.selected().original_json();
    assert!(original.contains("ESS-SYNTH-016"), "{original}");
    // The same document labelled with the coverage major before the construct is refused.
    let older = original.replace("\"ess-conformance/17\"", "\"ess-conformance/15\"");
    assert_ne!(older, original);
    let error = AdmittedSuite::from_json(&older).expect_err("an aggregate refusal needs suite/17");
    assert!(error.to_string().contains("suite/17"), "{error}");
}
