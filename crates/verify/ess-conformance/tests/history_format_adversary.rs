//! Adversarial cases against the `ess-history/1` reader (`src/history.rs`).
//!
//! Each case states what the document the unit wrote promises — `schemas/ess-history.schema.json`,
//! the module docs of `history.rs`, or the story's outcome — and drives the reader against it.

use ess_conformance::history::{self, HistoryRefusal, HISTORY_FORMAT};
use ess_primitives::evidence::SpecDigest;
use serde_json::{json, Value};

const HISTORY_ID: &str = "00000000-0000-4000-8000-000000000001";
const FIRST: &str = "00000000-0000-4000-8000-00000000000a";
const FIRST_UPPER: &str = "00000000-0000-4000-8000-00000000000A";

fn digest() -> SpecDigest {
    SpecDigest::new("ab".repeat(32)).expect("well formed")
}

fn returned(operation_id: &str) -> Value {
    json!({
        "operation_id": operation_id,
        "client": 0,
        "command": "billing.invoice.IssueInvoice",
        "subject_key": "invoice-1",
        "invoked_at": 10,
        "returned_at": 20,
        "completion": "Returned",
        "outcome": "issued",
    })
}

fn document(operations: &[Value]) -> Value {
    json!({
        "format": HISTORY_FORMAT,
        "history_id": HISTORY_ID,
        "spec_digest": digest().as_str(),
        "seed": 7,
        "clients": 2,
        "operations": operations,
    })
}

fn read_text(text: &str) -> Result<history::History, HistoryRefusal> {
    history::read(text.as_bytes(), &digest())
}

fn read_value(value: &Value) -> Result<history::History, HistoryRefusal> {
    history::read(&serde_json::to_vec(value).expect("serializes"), &digest())
}

/// The schema says an operation is an `object` with `additionalProperties: false`, and the module
/// docs say the reader refuses "bytes that are not exactly this document". serde's derived struct
/// visitor also accepts a JSON array in field order, so a positional operation must still be
/// refused.
#[test]
fn an_operation_written_as_a_json_array_is_refused() {
    let positional = json!([
        FIRST,
        0,
        "billing.invoice.IssueInvoice",
        "invoice-1",
        10,
        20,
        "Returned",
        "issued"
    ]);
    let refused = read_value(&document(&[positional]));
    assert!(
        matches!(refused, Err(HistoryRefusal::Malformed { .. })),
        "an operation spelled as a JSON array is not an `ess-history/1` operation, got {refused:?}"
    );
}

/// The same, for the whole document: the schema's root is `type: object`.
#[test]
fn a_history_written_as_a_json_array_is_refused() {
    let text = format!(
        r#"["{HISTORY_FORMAT}","{HISTORY_ID}","{}",7,2,[]]"#,
        digest().as_str()
    );
    assert!(
        read_text(&text).is_err(),
        "a history spelled as a JSON array is not an `ess-history/1` document"
    );
}

/// `is_canonical_uuid` (and the schema's `Uuid` pattern) admit either hex case, so one UUID has
/// two spellings. The reader promises to refuse "two operations with one identity"; comparing the
/// strings misses the same identity written twice in different case.
#[test]
fn one_uuid_spelled_in_two_cases_is_one_identity() {
    let refused = read_value(&document(&[returned(FIRST), returned(FIRST_UPPER)]));
    assert!(
        matches!(refused, Err(HistoryRefusal::DuplicateOperation { .. })),
        "`{FIRST}` and `{FIRST_UPPER}` are one UUID, got {refused:?}"
    );
}

/// The story's outcome records "per operation its ... outcome"; the module says an `Indeterminate`
/// operation has none. A `Returned` operation without the outcome it answered leaves the checker
/// nothing to hold against the specification, and nothing refuses it.
#[test]
fn a_returned_operation_with_no_outcome_is_refused() {
    let mut operation = returned(FIRST);
    operation
        .as_object_mut()
        .expect("an object")
        .remove("outcome");
    let refused = read_value(&document(&[operation]));
    assert!(
        refused.is_err(),
        "a `Returned` operation carries the outcome it answered with"
    );
}

/// Every integer form other than a plain non-negative integer within `u64` is refused. These pin
/// the reader's half of the schema comparison in `ess-domain/tests/ess_history_schema_adversary.rs`.
#[test]
fn number_forms_other_than_plain_u64_are_malformed() {
    let base = document(&[returned(FIRST)]);
    let text = serde_json::to_string(&base).expect("serializes");
    for (from, to) in [
        (r#""seed":7"#, r#""seed":7.0"#),
        (r#""seed":7"#, r#""seed":7e0"#),
        (r#""seed":7"#, r#""seed":18446744073709551616"#),
        (r#""seed":7"#, r#""seed":-1"#),
        (r#""invoked_at":10"#, r#""invoked_at":1e1"#),
        (r#""clients":2"#, r#""clients":2.0"#),
    ] {
        assert!(text.contains(from), "the fixture spells {from}");
        let mutated = text.replacen(from, to, 1);
        let refused = read_text(&mutated);
        assert!(
            matches!(refused, Err(HistoryRefusal::Malformed { .. })),
            "{to} is refused as malformed, got {refused:?}"
        );
    }
}

/// A repeated key is refused rather than resolved by first- or last-wins, at both levels.
#[test]
fn a_repeated_key_is_malformed_at_either_level() {
    let text = serde_json::to_string(&document(&[returned(FIRST)])).expect("serializes");
    let top = text.replacen(r#""seed":7"#, r#""seed":7,"seed":8"#, 1);
    assert!(matches!(
        read_text(&top),
        Err(HistoryRefusal::Malformed { .. })
    ));
    let nested = text.replacen(r#""client":0"#, r#""client":0,"client":1"#, 1);
    assert!(matches!(
        read_text(&nested),
        Err(HistoryRefusal::Malformed { .. })
    ));
}
