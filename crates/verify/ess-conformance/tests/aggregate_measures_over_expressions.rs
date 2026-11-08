//! An aggregate measure takes an expression of the source row, and `min`/`max` admit an enum whose
//! declaration states an order (`ess/36`, `story:aggregate-measures-over-expressions`,
//! beyond10x/ess#561, beyond10x/ess#566, `docs/design/generic-language-features.md` § G3).
//!
//! The fixture is the incident ledger of the two reports: `Mttr` averages `ended_at - started_at`
//! per incident, `Downtime` sums `duration_s * level.weight`, a product with an attribute of the
//! level, and `Headline` reports the worst level of an incident's windows, by the order
//! `ImpactLevel` declares (`Minor`, `Major`, `Critical`), which alphabetical order reverses.
//!
//! Each test reads the synthesized scenario itself: the rows it creates, the group it asserts, and
//! the figure it asserts there, recomputed here without the shared evaluator. Then the interpreted
//! target passes the suite, and the interpreted target of a model that computes the figure
//! otherwise fails it.
use std::collections::BTreeMap;

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::{
    interpret::Interpreted, report::Status, synthesize::synthesize, AdmittedSuite,
    ConformanceScenario, ConformanceSuite, Runner, ScenarioStep, ScenarioValue, ViewExpectation,
};
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source,
};
use ess_primitives::{facts::Number, node::Node, time::Rfc3339Instant};

const MODEL: &str = include_str!("fixtures/aggregate-measures-over-expressions.yaml");

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}"));
    let spec = Specification::assemble([(Source::new("ledger.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn statuses(suite: &ConformanceSuite, target: &Interpreted) -> BTreeMap<String, String> {
    let admitted = AdmittedSuite::from_suite(suite).unwrap_or_else(|error| panic!("{error}"));
    Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, target)
        .into_report()
        .scenarios
        .into_iter()
        .map(|run| {
            let passed = matches!(run.status, Status::Passed);
            (
                run.scenario.to_string(),
                if passed {
                    "passed".to_owned()
                } else {
                    format!("{:?} {:?}", run.status, run.checks)
                },
            )
        })
        .collect()
}

fn scenario<'s>(suite: &'s ConformanceSuite, view: &str) -> &'s ConformanceScenario {
    let id = format!("demo.ledger.{view}/aggregate");
    suite
        .scenarios
        .iter()
        .find(|(scenario, _)| scenario.to_string() == id)
        .map_or_else(
            || panic!("no `{id}` scenario in {:#?}", suite.scenarios.keys()),
            |(_, scenario)| scenario,
        )
}

/// Every window the scenario records, by incident: the literal input of each `RecordWindow`.
fn windows(scenario: &ConformanceScenario) -> BTreeMap<String, Vec<BTreeMap<String, Node>>> {
    let mut out: BTreeMap<String, Vec<BTreeMap<String, Node>>> = BTreeMap::new();
    for step in &scenario.steps {
        let ScenarioStep::ExecuteCommand { command, input, .. } = step else {
            continue;
        };
        if command.to_string() != "demo.ledger.RecordWindow" {
            continue;
        }
        let input: BTreeMap<String, Node> = input
            .iter()
            .filter_map(|(name, value)| match value {
                ScenarioValue::Literal { value } => Some((name.clone(), value.clone())),
                _ => None,
            })
            .collect();
        let Some(Node::Text(incident)) = input.get("incident") else {
            panic!("a window names its incident: {input:#?}");
        };
        out.entry(incident.clone()).or_default().push(input);
    }
    out
}

/// Each row the scenario requires present, by incident: the value it asserts for `field`.
fn asserted(scenario: &ConformanceScenario, field: &str) -> BTreeMap<String, Node> {
    let mut out = BTreeMap::new();
    for step in &scenario.steps {
        let (ScenarioStep::ExpectView { expectation, .. }
        | ScenarioStep::EventuallyView { expectation, .. }) = step
        else {
            continue;
        };
        let ViewExpectation::Contains { fields } = expectation else {
            continue;
        };
        let (Some(ScenarioValue::Literal { value: incident }), Some(value)) =
            (fields.get("incident"), fields.get(field))
        else {
            continue;
        };
        let (Node::Text(incident), ScenarioValue::Literal { value }) = (incident, value) else {
            continue;
        };
        out.insert(incident.clone(), value.clone());
    }
    out
}

fn instant(node: &Node) -> Rfc3339Instant {
    let Node::Text(text) = node else {
        panic!("an instant is text: {node:?}")
    };
    Rfc3339Instant::parse_rfc3339(text).unwrap_or_else(|| panic!("{text} is an instant"))
}

fn integer(node: &Node) -> i64 {
    let Node::Number(number) = node else {
        panic!("an integer is a number: {node:?}")
    };
    number.as_i64().expect("a whole number")
}

fn text(node: &Node) -> &str {
    let Node::Text(text) = node else {
        panic!("a level is text: {node:?}")
    };
    text
}

/// The mean of `seconds`, six places, ties to even, spelled as the suite spells a number.
fn mean(seconds: &[i64]) -> Node {
    let total: i128 = seconds.iter().copied().map(i128::from).sum();
    let count = i128::try_from(seconds.len()).expect("a count");
    let scaled = total.abs() * 1_000_000;
    let (mut quotient, remainder) = (scaled / count, scaled % count);
    if remainder * 2 > count || (remainder * 2 == count && quotient % 2 == 1) {
        quotient += 1;
    }
    let sign = if total < 0 && quotient != 0 { "-" } else { "" };
    let text = format!("{sign}{}.{:06}", quotient / 1_000_000, quotient % 1_000_000);
    let text = text.trim_end_matches('0').trim_end_matches('.').to_owned();
    Node::Number(Number::decimal_literal(&text).expect("a decimal"))
}

fn weight(level: &str) -> i64 {
    match level {
        "Minor" => 1,
        "Major" => 3,
        "Critical" => 10,
        other => panic!("no level {other}"),
    }
}

fn rank(level: &str) -> usize {
    ["Minor", "Major", "Critical"]
        .iter()
        .position(|declared| *declared == level)
        .unwrap_or_else(|| panic!("no level {level}"))
}

/// Whether two numbers are one value, `5` and `5.0` alike.
fn same_number(left: &Node, right: &Node) -> bool {
    let canonical = |number: &Number| {
        let text = number.exact_text();
        if text.contains('.') {
            text.trim_end_matches('0').trim_end_matches('.').to_owned()
        } else {
            text
        }
    };
    match (left, right) {
        (Node::Number(left), Node::Number(right)) => canonical(left) == canonical(right),
        _ => left == right,
    }
}

/// The suite passes against the interpreted target of `MODEL`, and the scenario of `view` fails
/// against the interpreted target of `faulty`.
fn executed(suite: &ConformanceSuite, view: &str, faulty: &str) {
    let id = format!("demo.ledger.{view}/aggregate");
    let passing = statuses(suite, &Interpreted::for_model(ir(MODEL)));
    let failing: Vec<_> = passing.iter().filter(|(_, s)| *s != "passed").collect();
    assert!(failing.is_empty(), "{failing:#?}");
    let wrong = statuses(suite, &Interpreted::for_model(ir(faulty)));
    let failed: Vec<&str> = wrong
        .iter()
        .filter(|(_, status)| *status != "passed")
        .map(|(scenario, _)| scenario.as_str())
        .collect();
    assert_eq!(
        failed,
        [id.as_str()],
        "a target computing `{view}` otherwise fails {id}, and nothing else: {wrong:#?}"
    );
}

/// MTTR per incident: the mean of `ended_at - started_at`, asserted for a group whose windows last
/// two different durations, so a target reporting any one window's duration, or a figure over one
/// of the two instants, fails.
#[test]
fn avg_over_a_field_difference() {
    let model = ir(MODEL);
    let synthesis = synthesize(&model);
    assert!(synthesis.refusals.is_empty(), "{:#?}", synthesis.refusals);
    let scenario = scenario(&synthesis.suite, "Mttr");
    let windows = windows(scenario);
    let asserted = asserted(scenario, "mttr");
    let mut witnessed = false;
    for (incident, figure) in &asserted {
        let seconds: Vec<i64> = windows[incident]
            .iter()
            .map(|window| {
                instant(&window["ended_at"])
                    .whole_seconds_since(instant(&window["started_at"]))
                    .expect("whole seconds apart")
            })
            .collect();
        assert!(
            same_number(figure, &mean(&seconds)),
            "`{incident}`: asserted {figure:?}, the mean of {seconds:?} is {:?}",
            mean(&seconds)
        );
        let mut distinct = seconds.clone();
        distinct.sort_unstable();
        distinct.dedup();
        if distinct.len() >= 2 {
            witnessed = true;
            for one in &seconds {
                assert!(
                    !same_number(figure, &mean(&[*one])),
                    "`{incident}`'s mean is one window's duration: {seconds:?}"
                );
            }
        }
    }
    assert!(
        witnessed,
        "a group of two different durations is asserted: {asserted:#?} over {windows:#?}"
    );
    executed(
        &synthesis.suite,
        "Mttr",
        &MODEL.replace("avg: {expr: \"ended_at - started_at\"}", "avg: duration_s"),
    );
}

/// Weighted downtime per incident: the sum of `duration_s * level.weight`, asserted for a group
/// holding two levels of different weights, so a target ignoring the weight fails.
#[test]
fn sum_over_a_product_with_an_attribute() {
    let model = ir(MODEL);
    let synthesis = synthesize(&model);
    assert!(synthesis.refusals.is_empty(), "{:#?}", synthesis.refusals);
    let scenario = scenario(&synthesis.suite, "Downtime");
    let windows = windows(scenario);
    let asserted = asserted(scenario, "weighted");
    let mut witnessed = false;
    for (incident, figure) in &asserted {
        let rows = &windows[incident];
        let expected: i64 = rows
            .iter()
            .map(|window| integer(&window["duration_s"]) * weight(text(&window["level"])))
            .sum();
        assert_eq!(
            integer(figure),
            expected,
            "`{incident}`: asserted {figure:?} over {rows:#?}"
        );
        let mut weights: Vec<i64> = rows
            .iter()
            .map(|window| weight(text(&window["level"])))
            .collect();
        weights.sort_unstable();
        weights.dedup();
        let unweighted: i64 = rows
            .iter()
            .map(|window| integer(&window["duration_s"]))
            .sum();
        if weights.len() >= 2 && unweighted != expected {
            witnessed = true;
        }
    }
    assert!(
        witnessed,
        "a group of two levels is asserted: {asserted:#?} over {windows:#?}"
    );
    executed(
        &synthesis.suite,
        "Downtime",
        &MODEL.replace("duration_s * level.weight", "duration_s * 1"),
    );
}

/// The worst level per incident: `max` by the order `ImpactLevel` declares, asserted for a group
/// whose greatest level by declaration is not its greatest word, so a target comparing the
/// variants as text fails.
#[test]
fn max_over_an_ordered_enum() {
    let model = ir(MODEL);
    let synthesis = synthesize(&model);
    assert!(synthesis.refusals.is_empty(), "{:#?}", synthesis.refusals);
    let scenario = scenario(&synthesis.suite, "Headline");
    let windows = windows(scenario);
    let asserted = asserted(scenario, "worst");
    let mut witnessed = false;
    for (incident, figure) in &asserted {
        let levels: Vec<&str> = windows[incident]
            .iter()
            .map(|window| text(&window["level"]))
            .collect();
        let declared = levels
            .iter()
            .max_by_key(|level| rank(level))
            .expect("a level");
        let alphabetical = levels.iter().max().expect("a level");
        assert_eq!(
            text(figure),
            *declared,
            "`{incident}`: asserted {figure:?} over {levels:?}"
        );
        if declared != alphabetical {
            witnessed = true;
        }
    }
    assert!(
        witnessed,
        "a group whose declared and alphabetical maxima differ is asserted: {asserted:#?} over \
         {windows:#?}"
    );
    // The same enum declared alphabetically: a target ordering the variants as words.
    executed(
        &synthesis.suite,
        "Headline",
        &MODEL.replace(
            "      - {name: Minor, attributes: {weight: 1}}\n      - {name: Major, attributes: {weight: 3}}\n      - {name: Critical, attributes: {weight: 10}}\n",
            "      - {name: Critical, attributes: {weight: 10}}\n      - {name: Major, attributes: {weight: 3}}\n      - {name: Minor, attributes: {weight: 1}}\n",
        ),
    );
}
