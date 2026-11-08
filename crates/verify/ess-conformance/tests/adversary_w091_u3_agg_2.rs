//! Adversary cases for `story:aggregate-measures-over-expressions` (`ess/36`): the Duration and
//! ordered-enum measures the guide (`website/docs/guides/specify/aggregate-views.md`, "Measures over
//! expressions and ordered enums") and design § G3 admit, synthesized and run against the
//! interpreted target.
//!
//! The guide: "A `Duration` field takes `sum`, `avg`, `min` and `max` as an expression of one
//! does", and the only measures it names as getting no scenario are an expression measure beside
//! `skip_absent:` or `where:`.
use std::collections::BTreeMap;

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::{
    interpret::Interpreted, report::Status, synthesize::synthesize, AdmittedSuite,
    ConformanceSuite, Runner,
};
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source,
};

const LEDGER: &str = include_str!("fixtures/aggregate-measures-over-expressions.yaml");

/// The ledger with a `Duration` field `elapsed` on the window, and its views replaced by `views`.
fn model(views: &str) -> String {
    let head = LEDGER
        .replace(
            "      - {name: level, type: demo.ledger.ImpactLevel}\n",
            "      - {name: level, type: demo.ledger.ImpactLevel}\n      - {name: elapsed, type: Duration}\n",
        )
        .replace(
            "          level: input.level\n",
            "          level: input.level\n          elapsed: input.elapsed\n",
        );
    format!("{}views:\n{views}", head.split_once("views:\n").unwrap().0)
}

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}"));
    let spec = Specification::assemble([(Source::new("ledger.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn failing(suite: &ConformanceSuite, target: &Interpreted) -> BTreeMap<String, String> {
    let admitted = AdmittedSuite::from_suite(suite).unwrap_or_else(|error| panic!("{error}"));
    Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, target)
        .into_report()
        .scenarios
        .into_iter()
        .filter(|run| !matches!(run.status, Status::Passed))
        .map(|run| {
            (
                run.scenario.to_string(),
                format!("{:?} {:?}", run.status, run.checks),
            )
        })
        .collect()
}

/// `views` synthesizes with no refusal, carries a scenario for `view`, and the interpreted target
/// of the same model passes every scenario.
fn witnessed_and_passing(views: &str, view: &str) {
    let text = model(views);
    let model = ir(&text);
    let synthesis = synthesize(&model);
    assert!(
        synthesis.refusals.is_empty(),
        "refused:\n{:#?}\nviews:\n{views}",
        synthesis.refusals
    );
    let id = format!("demo.ledger.{view}/aggregate");
    assert!(
        synthesis
            .suite
            .scenarios
            .keys()
            .any(|scenario| scenario.to_string() == id),
        "no `{id}` in {:#?}",
        synthesis.suite.scenarios.keys()
    );
    let failed = failing(&synthesis.suite, &Interpreted::for_model(model));
    assert!(failed.is_empty(), "{failed:#?}");
}

/// The guide's own `downtime`: a grouped `sum` of `ended_at - started_at`, declared `Duration`.
#[test]
fn the_guides_downtime_sum_of_a_difference_is_witnessed() {
    witnessed_and_passing(
        "  - name: demo.ledger.Down\n    source: demo.ledger.Window\n    consistency: read_your_writes\n    group_by: [incident]\n    fields:\n      - {name: incident, type: String}\n      - {name: downtime, type: Duration, aggregate: {sum: {expr: \"ended_at - started_at\"}}}\n",
        "Down",
    );
}

/// A grouped `max` of a Duration field.
#[test]
fn a_grouped_max_of_a_duration_field_is_witnessed() {
    witnessed_and_passing(
        "  - name: demo.ledger.Longest\n    source: demo.ledger.Window\n    consistency: read_your_writes\n    group_by: [incident]\n    fields:\n      - {name: incident, type: String}\n      - {name: longest, type: Optional<Duration>, aggregate: {max: elapsed}}\n",
        "Longest",
    );
}

/// A grouped `avg` of a Duration field, a Decimal of seconds.
#[test]
fn a_grouped_avg_of_a_duration_field_is_witnessed() {
    witnessed_and_passing(
        "  - name: demo.ledger.Mean\n    source: demo.ledger.Window\n    consistency: read_your_writes\n    group_by: [incident]\n    fields:\n      - {name: incident, type: String}\n      - {name: mean, type: Optional<Decimal>, aggregate: {avg: elapsed}}\n",
        "Mean",
    );
}

/// An ungrouped `sum` of a Duration field: the guide names no refusal for it.
#[test]
fn an_ungrouped_sum_of_a_duration_field_is_witnessed() {
    witnessed_and_passing(
        "  - name: demo.ledger.Total\n    source: demo.ledger.Window\n    consistency: read_your_writes\n    fields:\n      - {name: total, type: Duration, aggregate: {sum: elapsed}}\n",
        "Total",
    );
}

/// A grouped `min` over the ordered enum.
#[test]
fn a_grouped_min_over_an_ordered_enum_is_witnessed() {
    witnessed_and_passing(
        "  - name: demo.ledger.Mildest\n    source: demo.ledger.Window\n    consistency: read_your_writes\n    group_by: [incident]\n    fields:\n      - {name: incident, type: String}\n      - {name: mildest, type: Optional<demo.ledger.ImpactLevel>, aggregate: {min: level}}\n",
        "Mildest",
    );
}
