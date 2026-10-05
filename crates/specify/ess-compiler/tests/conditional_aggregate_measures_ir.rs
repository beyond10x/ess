//! A measure's condition in the IR and in every description of it (beyond10x/ess#363).
//!
//! `docs/design/conditional-aggregate-measures.md`: `ResolvedAggregate.where` is the resolved
//! predicate, omitted when the source writes none, so every model without a condition keeps its IR
//! bytes; the description every projection reads gains one condition clause only where one is
//! written.

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::{compile, diagnose};
use ess_compiler::source::SourceMap;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

const CASES: &str = include_str!(
    "../../../verify/ess-conformance/tests/fixtures/conditional-aggregate-measures.yaml"
);
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

#[test]
fn a_condition_is_carried_resolved_and_described() {
    let ir = ir(CASES);
    let scorecard = &ir.views()[&"demo.cases.Scorecard".parse().unwrap()];
    let functions = &scorecard.aggregation.as_ref().unwrap().functions;
    let completed = &functions["completed"];
    assert_eq!(
        completed
            .r#where
            .as_ref()
            .map(ToString::to_string)
            .as_deref(),
        Some("state == Completed")
    );
    assert_eq!(completed.to_string(), "count() where state == Completed");
    assert_eq!(
        completed.describe(),
        "count of instances, over the rows where state == Completed"
    );
    let total = &functions["total"];
    assert!(total.r#where.is_none());
    assert_eq!(total.to_string(), "count()");
    assert_eq!(total.describe(), "count of instances");

    let json = ir.to_canonical_json();
    assert!(json.contains(r#""where": "state == Completed""#), "{json}");
}

#[test]
fn a_model_that_writes_no_condition_has_no_such_key() {
    let json = ir(METRICS).to_canonical_json();
    assert!(!json.contains(r#""where""#), "{json}");
}

#[test]
fn a_condition_below_ess_22_compiles_to_the_format_code() {
    let older = CASES.replace("format: ess/22", "format: ess/21");
    assert!(
        codes(&older).iter().any(|code| code == "ESS-VIEW-009"),
        "{:?}",
        codes(&older)
    );
}
