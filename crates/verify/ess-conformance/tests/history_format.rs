//! `ess-history/1`: what a concurrent history reader admits, and every way it refuses.
//!
//! The document is declared by `models/concurrent-history/`; the Rust type is held to that model by
//! `crates/edge/ess-xtask/tests/history_model.rs`. This file decides the reader's behaviour: a
//! history recorded against another specification is refused before anything reads its operations,
//! and an operation whose completion and instants disagree is refused by name.

use std::path::{Path, PathBuf};

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_conformance::history::{self, Completion, HistoryRefusal, ReturnBound, HISTORY_FORMAT};
use ess_conformance::scenario::SuiteProvenance;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_primitives::evidence::SpecDigest;
use serde_json::{json, Value};

fn example() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../examples/billing")
        .canonicalize()
        .expect("the billing example exists")
}

fn yaml_files(base: &Path) -> Vec<String> {
    let mut found = Vec::new();
    let mut pending = vec![base.to_path_buf()];
    while let Some(directory) = pending.pop() {
        for entry in std::fs::read_dir(&directory).expect("the example is readable") {
            let path = entry.expect("an entry").path();
            if path.is_dir() {
                pending.push(path);
            } else if path.extension().is_some_and(|it| it == "yaml") {
                found.push(
                    path.strip_prefix(base)
                        .expect("inside the example")
                        .display()
                        .to_string(),
                );
            }
        }
    }
    assert!(!found.is_empty(), "the billing example holds no files");
    found.sort();
    found
}

fn billing() -> EssIr {
    let base = example();
    let mut sources = SourceMap::new();
    let mut parsed = Vec::new();
    for label in yaml_files(&base) {
        let text = std::fs::read_to_string(base.join(&label)).expect("readable");
        let raw = RawSpecFile::parse(&text).expect("well formed");
        sources.insert(label.clone(), text);
        parsed.push((Source::new(label), raw));
    }
    let specification = Specification::assemble(parsed).expect("the billing example validates");
    compile(&specification, &sources).expect("the billing example resolves")
}

/// The digest of the compiled billing IR — the one a history recorded against it carries.
fn billing_digest() -> SpecDigest {
    SuiteProvenance::of(&billing()).spec_digest
}

/// A digest that is well formed and is not the billing IR's.
fn other_digest() -> SpecDigest {
    let digest = SpecDigest::new("0".repeat(64)).expect("well formed");
    assert_ne!(digest, billing_digest(), "the stand-in digest collides");
    digest
}

const HISTORY_ID: &str = "00000000-0000-4000-8000-000000000001";
const FIRST: &str = "00000000-0000-4000-8000-00000000000a";
const SECOND: &str = "00000000-0000-4000-8000-00000000000b";

/// A returned operation on client 0, invoked at 10 and returned at 20.
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

/// An operation that never answered, on client 1.
fn indeterminate(operation_id: &str) -> Value {
    json!({
        "operation_id": operation_id,
        "client": 1,
        "command": "billing.invoice.PayInvoice",
        "subject_key": "invoice-1",
        "invoked_at": 15,
        "completion": "Indeterminate",
    })
}

fn document(digest: &SpecDigest, operations: &[Value]) -> Value {
    json!({
        "format": HISTORY_FORMAT,
        "history_id": HISTORY_ID,
        "spec_digest": digest.as_str(),
        "seed": 7,
        "clients": 2,
        "operations": operations,
    })
}

fn bytes(value: &Value) -> Vec<u8> {
    serde_json::to_vec(value).expect("a JSON value serializes")
}

fn refusal(value: &Value, expected: &SpecDigest) -> HistoryRefusal {
    history::read(&bytes(value), expected).expect_err("the history is refused")
}

#[test]
fn a_history_recorded_against_the_compiled_ir_is_admitted() {
    let digest = billing_digest();
    let value = document(&digest, &[returned(FIRST), indeterminate(SECOND)]);
    let history = history::read(&bytes(&value), &digest).expect("the history is admitted");

    assert_eq!(history.spec_digest, digest);
    assert_eq!(history.seed, 7);
    assert_eq!(history.clients, 2);
    assert_eq!(history.history_id.as_str(), HISTORY_ID);
    assert_eq!(history.operations.len(), 2);
    let first = &history.operations[0];
    assert_eq!(first.operation_id.as_str(), FIRST);
    assert_eq!(first.client, 0);
    assert_eq!(first.command.as_str(), "billing.invoice.IssueInvoice");
    assert_eq!(first.subject_key, "invoice-1");
    assert_eq!(first.invoked_at, 10);
    assert_eq!(first.returned_at, Some(20));
    assert_eq!(first.completion, Completion::Returned);
    assert_eq!(
        first.outcome.as_ref().map(history::QualifiedName::as_str),
        Some("issued")
    );
}

#[test]
fn a_history_whose_spec_digest_differs_from_the_compiled_ir_is_refused_by_name() {
    let expected = billing_digest();
    let found = other_digest();
    let value = document(&found, &[returned(FIRST)]);
    let refused = refusal(&value, &expected);

    assert_eq!(
        refused,
        HistoryRefusal::SpecDigestMismatch {
            expected: expected.clone(),
            found,
        }
    );
    assert_eq!(refused.code(), "history.spec-digest-mismatch");
}

/// The digest is decided before any operation is read: a foreign history whose operations are
/// also wrong reports the digest, not the first bad operation.
#[test]
fn the_spec_digest_is_refused_before_any_operation_check_runs() {
    let expected = billing_digest();
    let mut broken = returned(FIRST);
    broken["returned_at"] = json!(1);
    let value = document(&other_digest(), &[broken]);

    assert_eq!(
        refusal(&value, &expected).code(),
        "history.spec-digest-mismatch"
    );
}

#[test]
fn a_returned_operation_with_no_return_instant_is_refused_by_name() {
    let digest = billing_digest();
    let mut operation = returned(FIRST);
    operation
        .as_object_mut()
        .expect("an object")
        .remove("returned_at");
    let refused = refusal(&document(&digest, &[operation]), &digest);

    assert_eq!(
        refused,
        HistoryRefusal::ReturnedWithoutReturnInstant {
            operation_id: FIRST.to_owned(),
        }
    );
    assert_eq!(refused.code(), "history.returned-without-return-instant");
}

#[test]
fn a_returned_operation_with_a_null_return_instant_is_refused_by_name() {
    let digest = billing_digest();
    let mut operation = returned(FIRST);
    operation["returned_at"] = Value::Null;
    let refused = refusal(&document(&digest, &[operation]), &digest);

    assert_eq!(refused.code(), "history.returned-without-return-instant");
}

#[test]
fn an_operation_whose_return_precedes_its_invoke_is_refused_by_name() {
    let digest = billing_digest();
    let mut operation = returned(FIRST);
    operation["returned_at"] = json!(9);
    let refused = refusal(&document(&digest, &[operation]), &digest);

    assert_eq!(
        refused,
        HistoryRefusal::ReturnBeforeInvoke {
            operation_id: FIRST.to_owned(),
            invoked_at: 10,
            returned_at: 9,
        }
    );
    assert_eq!(refused.code(), "history.return-before-invoke");
}

#[test]
fn an_operation_that_returns_at_its_invoke_instant_is_admitted() {
    let digest = billing_digest();
    let mut operation = returned(FIRST);
    operation["returned_at"] = json!(10);
    history::read(&bytes(&document(&digest, &[operation])), &digest)
        .expect("a zero-length operation is admitted");
}

#[test]
fn an_indeterminate_operation_with_a_return_instant_is_refused_by_name() {
    let digest = billing_digest();
    let mut operation = indeterminate(FIRST);
    operation["returned_at"] = json!(30);
    let refused = refusal(&document(&digest, &[operation]), &digest);

    assert_eq!(
        refused,
        HistoryRefusal::IndeterminateWithReturnInstant {
            operation_id: FIRST.to_owned(),
        }
    );
    assert_eq!(refused.code(), "history.indeterminate-with-return-instant");
}

#[test]
fn an_indeterminate_operation_with_an_outcome_is_refused_by_name() {
    let digest = billing_digest();
    let mut operation = indeterminate(FIRST);
    operation["outcome"] = json!("paid");
    let refused = refusal(&document(&digest, &[operation]), &digest);

    assert_eq!(refused.code(), "history.indeterminate-with-outcome");
}

#[test]
fn an_operation_on_a_client_the_history_does_not_count_is_refused_by_name() {
    let digest = billing_digest();
    let mut operation = returned(FIRST);
    operation["client"] = json!(2);
    let refused = refusal(&document(&digest, &[operation]), &digest);

    assert_eq!(
        refused,
        HistoryRefusal::ClientOutOfRange {
            operation_id: FIRST.to_owned(),
            client: 2,
            clients: 2,
        }
    );
    assert_eq!(refused.code(), "history.client-out-of-range");
}

#[test]
fn a_history_with_no_clients_is_refused_by_name() {
    let digest = billing_digest();
    let mut value = document(&digest, &[]);
    value["clients"] = json!(0);

    assert_eq!(refusal(&value, &digest).code(), "history.no-clients");
}

#[test]
fn two_operations_with_one_identity_are_refused_by_name() {
    let digest = billing_digest();
    let refused = refusal(
        &document(&digest, &[returned(FIRST), returned(FIRST)]),
        &digest,
    );

    assert_eq!(
        refused,
        HistoryRefusal::DuplicateOperation {
            operation_id: FIRST.to_owned(),
        }
    );
    assert_eq!(refused.code(), "history.duplicate-operation");
}

#[test]
fn another_format_is_refused_by_name() {
    let digest = billing_digest();
    let mut value = document(&digest, &[]);
    value["format"] = json!("ess-history/2");

    assert_eq!(
        refusal(&value, &digest),
        HistoryRefusal::UnsupportedFormat {
            found: Some("ess-history/2".to_owned()),
        }
    );
    value.as_object_mut().expect("an object").remove("format");
    assert_eq!(
        refusal(&value, &digest).code(),
        "history.unsupported-format"
    );
}

#[test]
fn an_unknown_field_is_refused_at_either_level() {
    let digest = billing_digest();
    let mut value = document(&digest, &[]);
    value["retries"] = json!(1);
    assert_eq!(refusal(&value, &digest).code(), "history.malformed");

    let mut operation = returned(FIRST);
    operation["latency"] = json!(3);
    let value = document(&digest, &[operation]);
    assert_eq!(refusal(&value, &digest).code(), "history.malformed");
}

#[test]
fn a_missing_field_a_bad_identity_and_an_empty_name_are_malformed() {
    let digest = billing_digest();

    let mut operation = returned(FIRST);
    operation
        .as_object_mut()
        .expect("an object")
        .remove("subject_key");
    let value = document(&digest, &[operation]);
    assert_eq!(refusal(&value, &digest).code(), "history.malformed");

    let value = document(&digest, &[returned("not-a-uuid")]);
    assert_eq!(refusal(&value, &digest).code(), "history.malformed");

    let mut operation = returned(FIRST);
    operation["command"] = json!("");
    let value = document(&digest, &[operation]);
    assert_eq!(refusal(&value, &digest).code(), "history.malformed");

    let mut operation = returned(FIRST);
    operation["completion"] = json!("TimedOut");
    let value = document(&digest, &[operation]);
    assert_eq!(refusal(&value, &digest).code(), "history.malformed");

    assert_eq!(
        history::read(b"not json", &digest)
            .expect_err("refused")
            .code(),
        "history.malformed"
    );
}

/// An unanswered operation's return instant is read as after every other operation (decision 4).
#[test]
fn an_indeterminate_operation_returns_after_every_other_operation() {
    let digest = billing_digest();
    let value = document(&digest, &[returned(FIRST), indeterminate(SECOND)]);
    let history = history::read(&bytes(&value), &digest).expect("admitted");

    assert_eq!(history.operations[0].return_bound(), ReturnBound::At(20));
    assert_eq!(
        history.operations[1].return_bound(),
        ReturnBound::AfterEveryOther
    );
    assert!(ReturnBound::At(u64::MAX) < ReturnBound::AfterEveryOther);
}

/// One document reads to one value, whatever the order of its keys.
#[test]
fn reading_is_deterministic_and_ignores_key_order() {
    let digest = billing_digest();
    let value = document(&digest, &[returned(FIRST), indeterminate(SECOND)]);
    let forward = history::read(&bytes(&value), &digest).expect("admitted");
    let again = history::read(&bytes(&value), &digest).expect("admitted");
    assert_eq!(forward, again);

    let text = r#"{"operations":[],"clients":1,"seed":3,"spec_digest":"DIGEST","history_id":"ID","format":"ess-history/1"}"#
        .replace("DIGEST", digest.as_str())
        .replace("ID", HISTORY_ID);
    let reordered = history::read(text.as_bytes(), &digest).expect("admitted");
    assert_eq!(reordered.seed, 3);
    assert!(reordered.operations.is_empty());
}

/// A `Returned` operation carries the outcome it answered with (correction round 1, item 3).
#[test]
fn a_returned_operation_with_no_outcome_is_refused_by_name() {
    let digest = billing_digest();
    for absent in [None, Some(Value::Null)] {
        let mut operation = returned(FIRST);
        match absent {
            None => {
                operation
                    .as_object_mut()
                    .expect("an object")
                    .remove("outcome");
            }
            Some(null) => operation["outcome"] = null,
        }
        let refused = refusal(&document(&digest, &[operation]), &digest);
        assert_eq!(
            refused,
            HistoryRefusal::ReturnedWithoutOutcome {
                operation_id: FIRST.to_owned(),
            }
        );
        assert_eq!(refused.code(), "history.returned-without-outcome");
    }
}

/// One UUID has one spelling: lower-case hexadecimal (correction round 1, item 2).
#[test]
fn an_upper_case_identity_is_refused_by_name() {
    let digest = billing_digest();
    let upper = FIRST.to_uppercase();
    let refused = refusal(&document(&digest, &[returned(&upper)]), &digest);
    assert_eq!(
        refused,
        HistoryRefusal::NonCanonicalUuid {
            value: upper.clone(),
        }
    );
    assert_eq!(refused.code(), "history.non-canonical-uuid");

    let mut value = document(&digest, &[]);
    value["history_id"] = json!(FIRST.to_uppercase());
    assert_eq!(
        refusal(&value, &digest).code(),
        "history.non-canonical-uuid"
    );
}

/// Every integer is at most 2^53 − 1, the largest a JSON reader holding numbers as doubles reads
/// exactly (correction round 2, J1). Larger is refused by name, naming the field.
#[test]
fn an_integer_above_two_to_the_fifty_three_is_refused_by_name() {
    let digest = billing_digest();
    let limit = history::MAX_INTEGER;
    assert_eq!(limit, 9_007_199_254_740_991);

    let mut value = document(&digest, &[returned(FIRST)]);
    value["seed"] = json!(limit);
    history::read(&bytes(&value), &digest).expect("2^53 - 1 is admitted");

    value["seed"] = json!(limit + 1);
    let refused = refusal(&value, &digest);
    assert_eq!(
        refused,
        HistoryRefusal::IntegerOutOfRange {
            field: "seed".to_owned(),
            value: limit + 1,
        }
    );
    assert_eq!(refused.code(), "history.integer-out-of-range");

    value["seed"] = json!(u64::MAX);
    assert_eq!(
        refusal(&value, &digest).code(),
        "history.integer-out-of-range"
    );

    for field in ["client", "invoked_at", "returned_at"] {
        let mut operation = returned(FIRST);
        operation["invoked_at"] = json!(0);
        operation[field] = json!(limit + 1);
        let refused = refusal(&document(&digest, &[operation]), &digest);
        assert_eq!(
            refused,
            HistoryRefusal::IntegerOutOfRange {
                field: format!("operations[0].{field}"),
                value: limit + 1,
            },
            "{field}"
        );
    }

    let mut value = document(&digest, &[]);
    value["clients"] = json!(limit + 1);
    assert_eq!(
        refusal(&value, &digest).code(),
        "history.integer-out-of-range"
    );
}
