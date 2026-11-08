//! An expression measure and an enum's `ordered: true` (ess/36,
//! `story:aggregate-measures-over-expressions`, beyond10x/ess#561, beyond10x/ess#566): `ess verify
//! diff` reports a measure whose expression moved as the existing `field-aggregate-changed`,
//! naming both expressions, and a flipped `ordered:` as the enum's `ordered-changed`, an
//! `ess-diff/23` change.

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_diff::{diff, SemanticRelation};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};

const LEDGER: &str = include_str!(
    "../../../verify/ess-conformance/tests/fixtures/aggregate-measures-over-expressions.yaml"
);

fn ir(text: &str) -> EssIr {
    let spec = Specification::assemble([(
        Source::new("ledger.yaml"),
        RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}")),
    )])
    .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap()
}

/// The kinds of every change between `before` and `after`, and the delta as JSON.
fn changes(before: &str, after: &str) -> (Vec<String>, serde_json::Value) {
    let delta = diff(&ir(before), &ir(after)).unwrap();
    let kinds = delta
        .changes()
        .iter()
        .map(|change| change.kind().to_owned())
        .collect();
    let json = serde_json::from_str(&delta.to_canonical_json()).unwrap();
    (kinds, json)
}

#[test]
fn an_expression_that_moved_is_a_field_aggregate_change() {
    let after = LEDGER.replace("duration_s * level.weight", "duration_s * level.weight * 2");
    let (kinds, json) = changes(LEDGER, &after);
    assert_eq!(kinds, ["field-aggregate-changed"], "{json:#}");
    let text = json.to_string();
    assert!(
        text.contains("sum(duration_s * level.weight)")
            && text.contains("sum(duration_s * level.weight * 2)"),
        "{json:#}"
    );
    // A respelling that parses to the same tree is no change.
    let respelled = LEDGER.replace("duration_s * level.weight", "(duration_s) * level.weight");
    let (kinds, json) = changes(LEDGER, &respelled);
    assert!(kinds.is_empty(), "{json:#}");
}

#[test]
fn a_field_measure_becoming_an_expression_is_a_field_aggregate_change() {
    let before = LEDGER.replace("avg: {expr: \"ended_at - started_at\"}", "avg: duration_s");
    let (kinds, json) = changes(&before, LEDGER);
    assert_eq!(kinds, ["field-aggregate-changed"], "{json:#}");
}

#[test]
fn a_flipped_order_is_an_ordered_change_in_ess_diff_23() {
    // Dropping the order needs the `max` over the enum gone too, or the later revision is refused.
    let unordered = LEDGER.replace("    ordered: true\n", "").replace(
        "      - {name: worst, type: Optional<demo.ledger.ImpactLevel>, aggregate: {max: level}}\n",
        "      - {name: windows, type: Integer, aggregate: {count: {}}}\n",
    );
    let ordered = unordered.replace(
        "  - name: demo.ledger.ImpactLevel\n    kind: enum\n",
        "  - name: demo.ledger.ImpactLevel\n    kind: enum\n    ordered: true\n",
    );
    let delta = diff(&ir(&unordered), &ir(&ordered)).unwrap();
    let json = delta.to_canonical_json();
    let flipped: Vec<_> = delta
        .changes()
        .iter()
        .filter(|change| change.kind() == "ordered-changed")
        .collect();
    let [change] = flipped.as_slice() else {
        panic!("one ordered change: {json}")
    };
    assert_eq!(change.relation(), SemanticRelation::Changed, "{json}");
    assert_eq!(delta.changes().len(), 1, "{json}");
    assert!(json.contains("\"ess-diff/23\""), "{json}");
    assert!(json.contains("ImpactLevel"), "{json}");
}
