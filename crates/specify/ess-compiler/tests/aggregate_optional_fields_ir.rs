//! `skip_absent` in the IR, and the stable codes the absent-value refusals compile to
//! (beyond10x/ess#148).
//!
//! `docs/design/aggregate-views.md`, "Absent values (ess/15)": `ResolvedAggregate` gains
//! `skip_absent`, omitted when false, so every model that does not write it keeps its IR bytes.

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::{compile, diagnose};
use ess_compiler::source::SourceMap;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

const ORDERS: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/aggregate-optional-fields.yaml");
const METRICS: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/aggregate-views.yaml");

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}"));
    let spec = Specification::assemble([(Source::new("model.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|errors| panic!("{errors:?}"))
}

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

fn view(body: &str) -> String {
    let head = ORDERS.split_once("views:\n").unwrap().0;
    format!("{head}views:\n  - name: demo.orders.V\n    source: demo.orders.Order\n{body}")
}

#[test]
fn a_skipping_aggregate_carries_skip_absent_and_renders_it() {
    let ir = ir(ORDERS);
    let by_customer = &ir.views()[&"demo.orders.DurationByCustomer".parse().unwrap()];
    let aggregation = by_customer.aggregation.as_ref().unwrap();
    let total = &aggregation.functions["total"];
    assert!(total.skip_absent);
    assert_eq!(total.to_string(), "sum(duration, skip_absent)");
    assert_eq!(
        total.describe(),
        "sum of `duration`, skipping absent values"
    );
    let orders = &aggregation.functions["orders"];
    assert!(!orders.skip_absent);
    assert_eq!(orders.to_string(), "count()");

    let json = ir.to_canonical_json();
    assert!(json.contains(r#""skip_absent": true"#), "{json}");
    assert!(!json.contains(r#""skip_absent": false"#), "{json}");
}

#[test]
fn a_model_that_does_not_write_skip_absent_has_no_such_key() {
    let json = ir(METRICS).to_canonical_json();
    assert!(!json.contains("skip_absent"), "{json}");
}

#[test]
fn the_absent_value_refusals_compile_to_the_stable_codes_the_page_names() {
    let cases: Vec<(&str, String, &str)> = vec![
        (
            "V9 without skip_absent",
            view("    fields:\n      - {name: n, type: Optional<Integer>, aggregate: {sum: duration}}\n"),
            "ESS-VIEW-009",
        ),
        (
            "skip_absent over a field every row holds",
            view("    fields:\n      - {name: n, type: Integer, aggregate: {count_distinct: customer, skip_absent: true}}\n"),
            "ESS-VIEW-004",
        ),
        (
            "skip_absent below ess/15",
            ORDERS.replace("format: ess/15", "format: ess/14"),
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
