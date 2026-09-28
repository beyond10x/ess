//! Adversary, pass 2: admission of a `changed_by` amount (story
//! `ungrouped-aggregate-views-are-witnessed`, beyond10x/ess#148).
//!
//! The design page: "Each amount is a number, compared exactly", and the runner compares through
//! `aggregate::change` / `aggregate::same_number`, which answer nothing for a number with no exact
//! decimal spelling. An amount the runner can never compare is no claim, and admission refuses the
//! other kinds of no-claim (`InvalidSuite`).
use ess_compiler::{resolve::compile, source::SourceMap};
use ess_conformance::{synthesize::synthesize, AdmittedSuite};
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source,
};

const ORDERS: &str = include_str!("fixtures/aggregate-optional-fields.yaml");

fn suite_json() -> String {
    let raw = RawSpecFile::parse(ORDERS).unwrap();
    let spec = Specification::assemble([(Source::new("orders.yaml"), raw)]).unwrap();
    let ir = compile(&spec, &SourceMap::new()).unwrap();
    synthesize(&ir).suite.to_canonical_json().unwrap()
}

#[test]
fn an_amount_with_no_exact_decimal_spelling_is_refused_as_invalid_suite() {
    let original = suite_json();
    for raw in ["1e40", "123456789012345678901234567890123456789012"] {
        let mut document: serde_json::Value = serde_json::from_str(&original).unwrap();
        let steps = document["scenarios"]["demo.orders.DurationTotal/aggregate"]["steps"]
            .as_array_mut()
            .unwrap();
        let change = steps
            .iter_mut()
            .find(|step| step["expectation"]["expect"] == "changed_by")
            .expect("the scenario asserts a change");
        change["expectation"]["fields"]["total"] = serde_json::json!(987_654_321_u64);
        let text = serde_json::to_string_pretty(&document)
            .unwrap()
            .replacen("987654321", raw, 1);
        assert!(text.contains(raw), "{raw} is in the document");
        match AdmittedSuite::from_json(&text) {
            Ok(_) => panic!("`{raw}` is admitted as a change the runner can never compare"),
            Err(error) => assert_eq!(error.issues[0].reason, "InvalidSuite", "{raw}: {error}"),
        }
    }
}
