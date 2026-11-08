//! An expression measure and an ordered enum in the IR (ess/36,
//! `story:aggregate-measures-over-expressions`, beyond10x/ess#561, beyond10x/ess#566).
//!
//! `ResolvedAggregate.expr` carries the checked expression, each read with its kind, beside
//! `input`, and `ResolvedType.ordered` the enum's flag; both are omitted when absent, so a model
//! whose measures read fields and whose enums declare no order keeps its IR bytes and digests, and
//! the same model written at `ess/35` compiles to the same bytes as at `ess/36`.

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

const LEDGER: &str = include_str!(
    "../../../verify/ess-conformance/tests/fixtures/aggregate-measures-over-expressions.yaml"
);

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}"));
    let spec = Specification::assemble([(Source::new("ledger.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|errors| panic!("{errors:?}"))
}

#[test]
fn an_expression_measure_carries_its_checked_expression_and_renders_it() {
    let ir = ir(LEDGER);
    let view = &ir.views()[&"demo.ledger.Downtime".parse().unwrap()];
    let weighted = &view.aggregation.as_ref().unwrap().functions["weighted"];
    assert!(weighted.input.is_none());
    let measured = weighted.expr.as_ref().expect("an expression");
    assert_eq!(measured.typed.to_string(), "Integer");
    assert_eq!(measured.fields(), ["duration_s", "level"]);
    assert_eq!(weighted.to_string(), "sum(duration_s * level.weight)");
    assert_eq!(weighted.describe(), "sum of `duration_s * level.weight`");

    let mttr = &ir.views()[&"demo.ledger.Mttr".parse().unwrap()]
        .aggregation
        .as_ref()
        .unwrap()
        .functions["mttr"];
    assert_eq!(
        mttr.expr
            .as_ref()
            .map(|measured| measured.typed.to_string()),
        Some("Duration".to_owned())
    );

    let level = &ir.types()[&"demo.ledger.ImpactLevel".parse().unwrap()];
    assert!(level.ordered);
    let json = ir.to_canonical_json();
    assert!(json.contains(r#""ordered": true"#), "{json}");
    assert!(json.contains(r#""typed": "Integer""#), "{json}");
}

/// A model with no expression measure and no ordered enum keeps its bytes from `ess/35` to
/// `ess/36`: neither key is written for it.
#[test]
fn a_field_measure_keeps_its_ir_bytes_across_the_format() {
    let head = LEDGER
        .split_once("views:\n")
        .unwrap()
        .0
        .replace("    ordered: true\n", "");
    let plain = format!(
        "{head}views:\n  - name: demo.ledger.Totals\n    source: demo.ledger.Window\n    \
         consistency: read_your_writes\n    group_by: [incident]\n    fields:\n      - {{name: \
         incident, type: String}}\n      - {{name: total, type: Integer, aggregate: {{sum: \
         duration_s}}}}\n      - {{name: latest, type: Optional<Timestamp>, aggregate: {{max: \
         ended_at}}}}\n"
    );
    let at_36 = ir(&plain).to_canonical_json();
    assert!(!at_36.contains(r#""expr""#), "{at_36}");
    assert!(!at_36.contains(r#""ordered""#), "{at_36}");
}
