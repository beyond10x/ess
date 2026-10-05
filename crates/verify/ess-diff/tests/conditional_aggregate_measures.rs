//! A measure's condition is behaviour, and a change to it is a semantic change
//! (beyond10x/ess#363, `docs/design/conditional-aggregate-measures.md`, "Semantic diff and
//! rendered authority").
use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_diff::diff;
use ess_domain::{spec::RawSpecFile, system::Source, Specification};

const CASES: &str =
    include_str!("../../ess-conformance/tests/fixtures/conditional-aggregate-measures.yaml");

fn ir(text: &str) -> EssIr {
    let spec = Specification::assemble([(
        Source::new("cases.yaml"),
        RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}")),
    )])
    .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap()
}

/// The `field-aggregate-changed` entries of a diff, as `(field, before, after)`.
fn aggregate_changes(before: &str, after: &str) -> Vec<(String, String, String)> {
    let delta = diff(&ir(before), &ir(after)).unwrap();
    let json: serde_json::Value = serde_json::from_str(&delta.to_canonical_json()).unwrap();
    let mut out = Vec::new();
    collect(&json, &mut out);
    out
}

/// Every object anywhere in the delta whose `kind` is `field-aggregate-changed`.
fn collect(value: &serde_json::Value, out: &mut Vec<(String, String, String)>) {
    match value {
        serde_json::Value::Object(map) => {
            if map.get("kind").and_then(serde_json::Value::as_str)
                == Some("field-aggregate-changed")
            {
                let text = |key: &str| {
                    map.get(key)
                        .and_then(serde_json::Value::as_str)
                        .unwrap_or("")
                        .to_owned()
                };
                out.push((text("field"), text("before"), text("after")));
            }
            for inner in map.values() {
                collect(inner, out);
            }
        }
        serde_json::Value::Array(items) => {
            for inner in items {
                collect(inner, out);
            }
        }
        _ => {}
    }
}

const CONDITIONED: &str =
    "{name: escalated_cost, type: Integer, aggregate: {sum: cents, where: escalated == true}}";

#[test]
fn conditional_measure_changes_are_behavioral_diffs() {
    let plain = CASES.replace(
        CONDITIONED,
        "{name: escalated_cost, type: Integer, aggregate: {sum: cents}}",
    );
    // Added and removed.
    assert_eq!(
        aggregate_changes(&plain, CASES),
        [(
            "escalated_cost".to_owned(),
            "sum(cents)".to_owned(),
            "sum(cents) where escalated == true".to_owned()
        )]
    );
    assert_eq!(
        aggregate_changes(CASES, &plain),
        [(
            "escalated_cost".to_owned(),
            "sum(cents) where escalated == true".to_owned(),
            "sum(cents)".to_owned()
        )]
    );
    // Changed, and narrowed.
    let changed = CASES.replacen(
        "where: escalated == true}}",
        "where: escalated == false}}",
        1,
    );
    assert_eq!(aggregate_changes(CASES, &changed).len(), 1);
    let narrowed = CASES.replace(
        CONDITIONED,
        "{name: escalated_cost, type: Integer, aggregate: {sum: cents, where: [escalated == true, cents > 10]}}",
    );
    let found = aggregate_changes(CASES, &narrowed);
    assert_eq!(found.len(), 1, "{found:?}");
    assert_eq!(found[0].0, "escalated_cost");
    // Explicit `true` is not an omitted condition.
    let explicit = CASES.replace(
        CONDITIONED,
        "{name: escalated_cost, type: Integer, aggregate: {sum: cents, where: true}}",
    );
    assert_eq!(aggregate_changes(&plain, &explicit).len(), 1);
    // Two spellings of one resolved condition are no change.
    let mapped = CASES.replace(
        CONDITIONED,
        "{name: escalated_cost, type: Integer, aggregate: {sum: cents, where: {all: [escalated == true]}}}",
    );
    assert_eq!(aggregate_changes(CASES, &mapped), []);
    // And a model with no condition anywhere diffs as before.
    assert_eq!(aggregate_changes(&plain, &plain), []);
}
