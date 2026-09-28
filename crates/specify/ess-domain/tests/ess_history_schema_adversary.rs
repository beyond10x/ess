//! `schemas/ess-history.schema.json` against the reader it claims to describe.
//!
//! The Go and TypeScript writers validate against this schema; the Rust reader
//! (`ess_conformance::history::read`) decides what is checked. A document the schema admits and the
//! reader refuses as malformed is a writer that passes its own validation and is then refused.
//! The reader's half of each spelling below is pinned by
//! `crates/verify/ess-conformance/tests/history_format_adversary.rs`
//! (`number_forms_other_than_plain_u64_are_malformed`), which ess-domain cannot call directly.

use serde_json::Value;

fn validator() -> jsonschema::Validator {
    let text = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../schemas/ess-history.schema.json"),
    )
    .expect("the history schema is committed");
    let schema: Value = serde_json::from_str(&text).expect("the history schema is JSON");
    jsonschema::validator_for(&schema).expect("a usable schema")
}

const BASE: &str = r#"{"format":"ess-history/1","history_id":"00000000-0000-4000-8000-000000000001","spec_digest":"abababababababababababababababababababababababababababababababab","seed":7,"clients":2,"operations":[{"operation_id":"00000000-0000-4000-8000-00000000000a","client":0,"command":"billing.invoice.IssueInvoice","subject_key":"invoice-1","invoked_at":10,"returned_at":20,"completion":"Returned","outcome":"issued"}]}"#;

#[test]
fn the_schema_admits_the_base_document() {
    let value: Value = serde_json::from_str(BASE).expect("JSON");
    assert!(validator().is_valid(&value), "the base document is valid");
}

/// Each spelling is refused by the reader as `history.malformed`; the schema must refuse it too.
#[test]
fn the_schema_refuses_every_number_form_the_reader_refuses() {
    let validator = validator();
    let mut admitted = Vec::new();
    for (from, to) in [
        (r#""seed":7"#, r#""seed":7.0"#),
        (r#""seed":7"#, r#""seed":7e0"#),
        (r#""seed":7"#, r#""seed":18446744073709551616"#),
        (r#""seed":7"#, r#""seed":-1"#),
        (r#""invoked_at":10"#, r#""invoked_at":1e1"#),
        (r#""clients":2"#, r#""clients":2.0"#),
    ] {
        assert!(BASE.contains(from), "the fixture spells {from}");
        let value: Value = serde_json::from_str(&BASE.replacen(from, to, 1)).expect("JSON");
        if validator.is_valid(&value) {
            admitted.push(to);
        }
    }
    assert!(
        admitted.is_empty(),
        "the schema admits spellings the reader refuses as malformed: {admitted:?}"
    );
}
