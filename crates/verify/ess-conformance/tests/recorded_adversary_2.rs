//! Second adversarial pass against `ess_conformance::recorded`
//! (`story:recorded-history-validation`), aimed at the correction of pass 1: duplicate identities
//! refused naming every line, and non-canonical identities refused as `import.field-malformed`
//! before the history reader sees them.

use ess_conformance::history::SpecDigest;
use ess_conformance::recorded::{self, Adapter, ImportRefusal};
use serde_json::json;

const ADAPTER: &str = include_str!("fixtures/recorded/adapter.yaml");

fn digest() -> SpecDigest {
    SpecDigest::new("0123456789abcdef0123456789abcdef").expect("a digest")
}

fn adapter() -> Adapter {
    recorded::adapter(ADAPTER).expect("the committed adapter is admitted")
}

fn call(correlation: &str, invoked: u64) -> String {
    json!({
        "correlation": correlation,
        "request": {
            "client": "worker-0",
            "command": "billing.invoice.Issue",
            "subject": "inv-1",
            "at_ms": invoked,
        },
        "response": { "status": "ok", "outcome": "Ok", "at_ms": invoked + 1 },
    })
    .to_string()
}

/// The correction says a duplicate identity is refused "naming every line". The importer returns
/// at the second occurrence, so a third line carrying the same identity is never named: an
/// operator who fixes lines 1 and 2 re-runs and is told about line 3 only then.
#[test]
fn a_duplicate_identity_on_three_lines_is_refused_naming_all_three() {
    let id = "5f0c8a2e-3b7d-4c11-9e2a-7d4b1c0e9f31";
    let log = format!("{}\n{}\n{}\n", call(id, 10), call(id, 20), call(id, 30));
    let refusal = recorded::import(log.as_bytes(), &adapter(), &digest())
        .expect_err("one identity on three lines is refused");
    match &refusal {
        ImportRefusal::DuplicateOperation { lines, .. } => {
            assert_eq!(lines, &vec![1, 2, 3], "every line is named: {refusal}");
        }
        other => panic!("refused as a duplicate, not {other:?}"),
    }
}

/// The correction says a non-canonical identity is refused by the importer as
/// `import.field-malformed`, naming the line and `operation_id`. An upper-case identity passes the
/// importer's own check (`Uuid::new` admits either case), is written into the document verbatim,
/// and is refused later by the history reader under its own code.
#[test]
fn an_upper_case_identity_is_refused_by_the_importer_as_a_malformed_field() {
    let log = format!("{}\n", call("5F0C8A2E-3B7D-4C11-9E2A-7D4B1C0E9F31", 10));
    let refusal = recorded::import(log.as_bytes(), &adapter(), &digest())
        .expect_err("an upper-case identity is refused");
    assert_eq!(
        refusal.code(),
        "import.field-malformed",
        "refused before conversion, naming line and field: {refusal}"
    );
}
