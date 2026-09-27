//! Adversary pass 2 (story:concurrent-history-format): the draft-04 `oneOf` in
//! `schemas/ess-history.schema.json` against the reader's completion rule.
//!
//! The reader (`ess_conformance::history::read`) admits an operation exactly when it is `Returned`
//! with an integer `returned_at` and a string `outcome`, or `Indeterminate` with each of the two
//! absent or `null`. Its half of this matrix is pinned by
//! `crates/verify/ess-conformance/tests/history_completion_matrix.rs`, which uses the same predicate;
//! ess-domain cannot call the reader directly.

use serde_json::{json, Map, Value};

fn validator() -> jsonschema::Validator {
    let text = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../schemas/ess-history.schema.json"),
    )
    .expect("the history schema is committed");
    let schema: Value = serde_json::from_str(&text).expect("the history schema is JSON");
    jsonschema::validator_for(&schema).expect("a usable schema")
}

fn reader_admits(completion: &str, returned_at: &str, outcome: &str) -> bool {
    match completion {
        "Returned" => returned_at == "value" && outcome == "value",
        "Indeterminate" => returned_at != "value" && outcome != "value",
        other => panic!("no completion {other}"),
    }
}

#[test]
fn the_schema_admits_exactly_the_completions_the_reader_admits() {
    let validator = validator();
    let mut disagreements = Vec::new();
    for completion in ["Returned", "Indeterminate"] {
        for returned_slot in ["absent", "null", "value"] {
            for outcome_slot in ["absent", "null", "value"] {
                let mut operation = Map::new();
                operation.insert(
                    "operation_id".into(),
                    json!("00000000-0000-4000-8000-00000000000a"),
                );
                operation.insert("client".into(), json!(0));
                operation.insert("command".into(), json!("billing.invoice.IssueInvoice"));
                operation.insert("subject_key".into(), json!("invoice-1"));
                operation.insert("invoked_at".into(), json!(10));
                operation.insert("completion".into(), json!(completion));
                match returned_slot {
                    "null" => {
                        operation.insert("returned_at".into(), Value::Null);
                    }
                    "value" => {
                        operation.insert("returned_at".into(), json!(20));
                    }
                    _ => {}
                }
                match outcome_slot {
                    "null" => {
                        operation.insert("outcome".into(), Value::Null);
                    }
                    "value" => {
                        operation.insert("outcome".into(), json!("issued"));
                    }
                    _ => {}
                }
                let document = json!({
                    "format": "ess-history/1",
                    "history_id": "00000000-0000-4000-8000-000000000001",
                    "spec_digest": "ab".repeat(32),
                    "seed": 7,
                    "clients": 2,
                    "operations": [Value::Object(operation)],
                });
                let schema = validator.is_valid(&document);
                let reader = reader_admits(completion, returned_slot, outcome_slot);
                if schema != reader {
                    disagreements.push(format!(
                        "{completion} returned_at={returned_slot} outcome={outcome_slot}: \
                         schema {schema}, reader {reader}"
                    ));
                }
            }
        }
    }
    assert!(
        disagreements.is_empty(),
        "the schema and the reader disagree: {disagreements:#?}"
    );
}
