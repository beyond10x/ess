//! Aggregate views in the IR, and the stable codes their refusals compile to (beyond10x/ess#96).
//!
//! `docs/design/aggregate-views.md`, "Domain and IR" and the V1–V15 table: `ResolvedView` gains an
//! `aggregation` omitted when absent, so every model without the construct keeps its IR bytes, and
//! each validation rule's `ValidationCode` maps to the `ESS-VIEW-<class>` the page names.

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::{compile, diagnose};
use ess_compiler::source::SourceMap;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_domain::view::AggregateFunction;

const METRICS: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/aggregate-views.yaml");
const PARCELS: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/stored-field-guards.yaml");

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}"));
    let spec = Specification::assemble([(Source::new("model.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|errors| panic!("{errors:?}"))
}

fn view(body: &str) -> String {
    let head = METRICS.split_once("views:\n").unwrap().0;
    format!(
        "{head}views:\n  - name: metrics.session.V\n    source: metrics.session.Session\n{body}"
    )
}

/// The stable codes a document's refusals compile to.
fn codes(text: &str) -> Vec<String> {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}"));
    let errors = Specification::assemble([(Source::new("model.yaml"), raw)])
        .err()
        .unwrap_or_else(|| panic!("must refuse:\n{text}"));
    diagnose(&errors, &SourceMap::new())
        .as_slice()
        .iter()
        .map(|diagnostic| diagnostic.code.to_string())
        .collect()
}

#[test]
fn an_aggregate_view_carries_its_resolved_aggregation() {
    let ir = ir(METRICS);
    let view = &ir.views()[&"metrics.session.TalkTimeByAgent".parse().unwrap()];
    assert!(view.is_aggregate());
    let aggregation = view.aggregation.as_ref().unwrap();
    assert_eq!(aggregation.group_by, ["agent_id"]);
    let sum = &aggregation.functions["talk_seconds"];
    assert_eq!(sum.function, AggregateFunction::Sum);
    let input = sum.input.as_ref().expect("a resolved input");
    assert_eq!(input.name, "talk_seconds");
    assert!(aggregation.functions["sessions"].input.is_none());

    let json = ir.to_canonical_json();
    assert!(json.contains(r#""aggregation": {"#), "{json}");
    assert!(json.contains(r#""function": "count_distinct""#), "{json}");
    // `count` has no input, and it is omitted rather than written as null.
    assert!(!json.contains(r#""input": null"#), "{json}");
    // Declared after `filter` in every view that has one.
    let by_agent = json
        .split(r#""name": "metrics.session.TalkTimeByAgent""#)
        .nth(1)
        .unwrap();
    assert!(
        by_agent.find(r#""filter""#).unwrap() < by_agent.find(r#""aggregation""#).unwrap(),
        "{by_agent}"
    );
}

#[test]
fn a_model_without_an_aggregate_view_has_no_aggregation_key() {
    let json = ir(PARCELS).to_canonical_json();
    assert!(!json.contains("aggregation"), "{json}");
    assert!(!ir(PARCELS)
        .views()
        .values()
        .any(ess_compiler::ir::ResolvedView::is_aggregate));
}

#[test]
fn every_validation_rule_compiles_to_the_stable_code_the_page_names() {
    let group = |key: &str, declared: &str| {
        format!(
            "    group_by: [{key}]\n    fields:\n      - {{name: {key}, type: \"{declared}\"}}\n      - {{name: n, type: Integer, aggregate: {{count: {{}}}}}}\n"
        )
    };
    let optional_field = METRICS.replace(
        "      - {name: wait_seconds, type: Integer}\n    lifecycle:",
        "      - {name: wait_seconds, type: Integer}\n      - {name: note, type: Optional<Integer>}\n      - {name: started, type: Timestamp}\n      - {name: tags, type: List<String>}\n    lifecycle:",
    );
    let typed = |body: &str| {
        let head = optional_field.split_once("views:\n").unwrap().0;
        format!(
            "{head}views:\n  - name: metrics.session.V\n    source: metrics.session.Session\n{body}"
        )
    };
    let cases: Vec<(&str, String, &str)> = vec![
        (
            "V1",
            view("    group_by: [queue_id]\n    fields:\n      - {name: n, type: Integer, aggregate: {count: {}}}\n"),
            "ESS-VIEW-001",
        ),
        (
            "V2",
            view("    group_by: [n]\n    fields:\n      - {name: n, type: Integer, aggregate: {count: {}}}\n"),
            "ESS-VIEW-004",
        ),
        (
            "V3",
            view("    group_by: [agent_id, agent_id]\n    fields:\n      - {name: agent_id, type: String}\n      - {name: n, type: Integer, aggregate: {count: {}}}\n"),
            "ESS-VIEW-006",
        ),
        (
            "V4",
            view("    fields:\n      - {name: agent_id, type: String}\n      - {name: n, type: Integer, aggregate: {count: {}}}\n"),
            "ESS-VIEW-005",
        ),
        (
            "V5",
            view("    group_by: [agent_id]\n    fields:\n      - {name: agent_id, type: String}\n"),
            "ESS-VIEW-005",
        ),
        (
            "V6",
            view("    fields:\n      - {name: n, type: Integer, aggregate: {sum: minutes}}\n"),
            "ESS-VIEW-001",
        ),
        (
            "V7",
            view("    fields:\n      - {name: n, type: Integer, aggregate: {sum: total.amount}}\n"),
            "ESS-VIEW-009",
        ),
        (
            "V8",
            view("    fields:\n      - {name: n, type: Integer, aggregate: {sum: caller}}\n"),
            "ESS-VIEW-002",
        ),
        (
            "V9",
            typed("    fields:\n      - {name: n, type: Integer, aggregate: {sum: note}}\n"),
            "ESS-VIEW-009",
        ),
        (
            "V10",
            view("    fields:\n      - {name: n, type: Decimal, aggregate: {count: {}}}\n"),
            "ESS-VIEW-002",
        ),
        ("V11", typed(&group("started", "Timestamp")), "ESS-VIEW-009"),
        ("V12", typed(&group("tags", "List<String>")), "ESS-VIEW-002"),
        (
            "V13",
            view("    shape: metrics.session.Channel\n    group_by: [agent_id]\n    fields:\n      - {name: agent_id, type: String}\n      - {name: n, type: Integer, aggregate: {count: {}}}\n"),
            "ESS-VIEW-004",
        ),
        (
            "V14",
            view("    group_by: [agent_id]\n    order_by: [agent_id]\n    fields:\n      - {name: agent_id, type: String}\n      - {name: n, type: Integer, aggregate: {count: {}}}\n"),
            "ESS-VIEW-009",
        ),
        (
            "V15",
            METRICS.replace("format: ess/10", "format: ess/9"),
            "ESS-VIEW-009",
        ),
    ];
    for (rule, text, expected) in cases {
        let codes = codes(&text);
        assert!(
            codes.iter().any(|code| code == expected),
            "{rule}: expected {expected}, got {codes:?}"
        );
    }
}
