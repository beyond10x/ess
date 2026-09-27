//! Adversary pass 2 (story:concurrent-history-format): the reader's completion rule over every
//! absent / `null` / value combination of `returned_at` and `outcome`. The schema's half is
//! `crates/specify/ess-domain/tests/ess_history_schema_completion_matrix.rs`, with the same
//! predicate.

use ess_conformance::history::{self, HISTORY_FORMAT};
use ess_primitives::evidence::SpecDigest;
use serde_json::{json, Map, Value};

fn reader_admits(completion: &str, returned_at: &str, outcome: &str) -> bool {
    match completion {
        "Returned" => returned_at == "value" && outcome == "value",
        "Indeterminate" => returned_at != "value" && outcome != "value",
        other => panic!("no completion {other}"),
    }
}

#[test]
fn the_reader_admits_exactly_the_modelled_completions() {
    let digest = SpecDigest::new("ab".repeat(32)).expect("well formed");
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
                    "format": HISTORY_FORMAT,
                    "history_id": "00000000-0000-4000-8000-000000000001",
                    "spec_digest": digest.as_str(),
                    "seed": 7,
                    "clients": 2,
                    "operations": [Value::Object(operation)],
                });
                let bytes = serde_json::to_vec(&document).expect("serializes");
                let first = history::read(&bytes, &digest);
                let second = history::read(&bytes, &digest);
                assert_eq!(first, second, "one document reads to one result");
                let reader = first.is_ok();
                let expected = reader_admits(completion, returned_slot, outcome_slot);
                if reader != expected {
                    disagreements.push(format!(
                        "{completion} returned_at={returned_slot} outcome={outcome_slot}: \
                         reader {reader}, expected {expected} ({first:?})"
                    ));
                }
            }
        }
    }
    assert!(
        disagreements.is_empty(),
        "the reader disagrees with the completion rule: {disagreements:#?}"
    );
}
