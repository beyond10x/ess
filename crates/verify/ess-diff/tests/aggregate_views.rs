//! A changed grouping or aggregate is a semantic change of its own, at `ess-diff/7` (ess#96).
//!
//! `docs/design/aggregate-views.md`, "Projections": `grouping-changed` and
//! `field-aggregate-changed`, rendered `sum(talk_seconds)` / `count()`, `None` meaning "not an
//! aggregate", emitted after `filter-changed` and needing the diff format that names them.
use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_diff::{diff, EssDelta, RawEssDelta};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::error::ValidationCode;

const METRICS: &str = include_str!("../../ess-conformance/tests/fixtures/aggregate-views.yaml");

fn ir(text: &str) -> EssIr {
    let spec = Specification::assemble([(
        Source::new("metrics.yaml"),
        RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}")),
    )])
    .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap()
}

#[test]
fn regrouping_and_changing_an_aggregate_are_reported_at_diff_7() {
    let before = ir(METRICS);
    let after = ir(&METRICS
        .replace(
            "    group_by: [agent_id]\n    fields:\n      - {name: agent_id, type: String}\n",
            "    group_by: [agent_id, channel]\n    fields:\n      - {name: agent_id, type: String}\n      - {name: channel, type: metrics.session.Channel}\n",
        )
        .replace(
            "      - {name: longest_wait, type: Optional<Integer>, aggregate: {max: wait_seconds}}\n      - {name: distinct_callers",
            "      - {name: longest_wait, type: Optional<Integer>, aggregate: {min: wait_seconds}}\n      - {name: distinct_callers",
        ));
    let delta = diff(&before, &after).unwrap();
    let json = delta.to_canonical_json();
    assert_eq!(delta.format.to_string(), "ess-diff/7", "{json}");
    assert!(json.contains(r#""kind": "grouping-changed""#), "{json}");
    assert!(
        json.contains(r#""kind": "field-aggregate-changed""#),
        "{json}"
    );
    assert!(json.contains(r#""before": "max(wait_seconds)""#), "{json}");
    assert!(json.contains(r#""after": "min(wait_seconds)""#), "{json}");
    let ids: Vec<String> = delta.changes().iter().map(|c| c.id().to_string()).collect();
    assert!(
        ids.iter()
            .any(|id| id
                == "view/metrics.session.TalkTimeByAgent/field-aggregate-changed/longest_wait"),
        "{ids:?}"
    );
    assert!(
        ids.iter()
            .any(|id| id == "view/metrics.session.TalkTimeByAgent/grouping-changed"),
        "{ids:?}"
    );
    // It reads back at its own format.
    let raw: RawEssDelta = serde_json::from_str(&json).unwrap();
    assert_eq!(EssDelta::try_from(raw).unwrap(), delta);
}

#[test]
fn a_field_becoming_an_aggregate_renders_count_and_absence() {
    let before = ir(METRICS);
    let after = ir(&METRICS.replace(
        "      - {name: sessions, type: Integer, aggregate: {count: {}}}\n      - {name: talk_seconds, type: Integer, aggregate: {sum: talk_seconds}}\n      - {name: longest_wait",
        "      - {name: sessions, type: Integer, aggregate: {count_distinct: session_id}}\n      - {name: talk_seconds, type: Integer, aggregate: {sum: talk_seconds}}\n      - {name: longest_wait",
    ));
    let json = diff(&before, &after).unwrap().to_canonical_json();
    assert!(json.contains(r#""before": "count()""#), "{json}");
    assert!(
        json.contains(r#""after": "count_distinct(session_id)""#),
        "{json}"
    );
}

#[test]
fn an_older_delta_format_carrying_an_aggregate_change_is_refused() {
    let before = ir(METRICS);
    let after = ir(&METRICS.replace(
        "aggregate: {max: wait_seconds}}\n      - {name: distinct_callers",
        "aggregate: {min: wait_seconds}}\n      - {name: distinct_callers",
    ));
    let json = diff(&before, &after).unwrap().to_canonical_json();
    let mut document: serde_json::Value = serde_json::from_str(&json).unwrap();
    document["format"] = serde_json::json!("ess-diff/6");
    let raw: RawEssDelta = serde_json::from_value(document).unwrap();
    let errors = EssDelta::try_from(raw).expect_err("refused");
    assert!(
        errors.contains(ValidationCode::UnsupportedFormatVersion),
        "{errors}"
    );
}

#[test]
fn a_model_without_aggregate_views_keeps_its_diff_format() {
    let text = include_str!("../../ess-conformance/tests/fixtures/stored-field-guards.yaml");
    let before = ir(text);
    let after = ir(&text.replace("weight_kg > 20", "weight_kg > 30"));
    assert_eq!(
        diff(&before, &after).unwrap().format.to_string(),
        "ess-diff/2"
    );
}
