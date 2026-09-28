//! Adversarial cases against `ess_conformance::recorded` (`story:recorded-history-validation`).
//!
//! The story's rule is that a field the log does not carry is reported as a coverage gap and never
//! guessed, and the brief's rule is that a refusal names the field and the log line. Each case
//! below builds a small hand-written log and holds the importer to one of those two rules.

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

fn line(
    correlation: Option<&str>,
    command: &str,
    invoked: u64,
    response: serde_json::Value,
) -> String {
    let mut value = json!({
        "request": {
            "client": "worker-0",
            "command": command,
            "subject": "inv-1",
            "at_ms": invoked,
        },
    });
    value["response"] = response;
    if let Some(correlation) = correlation {
        value["correlation"] = json!(correlation);
    }
    value.to_string()
}

fn ok(at: u64) -> serde_json::Value {
    json!({ "status": "ok", "outcome": "Ok", "at_ms": at })
}

/// A line without a correlation id is given `00000000-0000-4000-8000-<line>`. That is the exact
/// spelling the recorder in `record.rs` mints for its own operations, so a log that carries such
/// an id on another line collides with the fabricated one. The log names two distinct calls; the
/// importer refuses it as one operation appearing twice.
#[test]
fn a_numbered_identity_never_collides_with_one_the_log_carries() {
    let log = format!(
        "{}\n{}\n",
        line(
            Some("00000000-0000-4000-8000-000000000002"),
            "billing.invoice.Issue",
            10,
            ok(20),
        ),
        line(None, "billing.invoice.Issue", 30, ok(40)),
    );
    let imported =
        recorded::import(log.as_bytes(), &adapter(), &digest()).unwrap_or_else(|refusal| {
            panic!("two distinct calls, one carrying its identity and one not, import: {refusal}")
        });
    assert_eq!(imported.history.operations.len(), 2);
    assert_ne!(
        imported.history.operations[0].operation_id,
        imported.history.operations[1].operation_id
    );
}

/// A read of a view is judged only from its `rows`. The adapter has no way to declare where rows
/// sit or that they are absent, and the importer reports nothing about them: the read arrives at
/// `check-history` without rows and silently becomes "not judged", with no coverage gap raised at
/// import, even though the log line carries the rows.
#[test]
fn a_view_read_whose_rows_are_not_imported_is_reported_as_a_coverage_gap() {
    let log = format!(
        "{}\n",
        line(
            Some("00000000-0000-4000-8000-00000000000a"),
            "billing.invoice.OutstandingInvoices",
            10,
            json!({ "status": "ok", "outcome": "Ok", "at_ms": 20, "rows": ["inv-1"] }),
        ),
    );
    let imported =
        recorded::import(log.as_bytes(), &adapter(), &digest()).expect("the read is imported");
    assert!(
        imported
            .gaps
            .iter()
            .any(|gap| gap.field == "rows" && gap.line == Some(1)),
        "the read's rows are not carried into the history and no gap says so: {:?}",
        imported.gaps
    );
}

/// Epoch-nanosecond instants (the common `UnixNano` shape) exceed 2^53 and are refused by the
/// history reader. The refusal names `operations[0]`, an index into a document the operator never
/// sees, not the log line; with a blank line first the index and the line differ.
#[test]
fn an_out_of_range_instant_is_refused_naming_its_log_line() {
    let log = format!(
        "\n{}\n",
        line(
            Some("00000000-0000-4000-8000-00000000000b"),
            "billing.invoice.Issue",
            1_790_000_000_000_000_000,
            ok(1_790_000_000_000_000_001),
        ),
    );
    let refusal = recorded::import(log.as_bytes(), &adapter(), &digest())
        .expect_err("an instant above 2^53 is refused");
    let text = refusal.to_string();
    assert!(
        text.contains("line 2"),
        "the refusal names the log line the instant is on: {text}"
    );
}

/// A timed-out call whose log line still records when the client gave up (`response.at_ms`) is
/// refused by the history reader as `Indeterminate` with `returned_at`, naming the operation id
/// only. The brief requires a refusal to name the field and the line.
#[test]
fn an_indeterminate_line_carrying_a_return_instant_is_refused_naming_its_log_line() {
    let log = format!(
        "{}\n{}\n",
        line(
            Some("00000000-0000-4000-8000-00000000000c"),
            "billing.invoice.Issue",
            10,
            ok(20),
        ),
        line(
            Some("00000000-0000-4000-8000-00000000000d"),
            "billing.invoice.Issue",
            30,
            json!({ "status": "timeout", "at_ms": 90 }),
        ),
    );
    let Err(refusal) = recorded::import(log.as_bytes(), &adapter(), &digest()) else {
        return;
    };
    let text = refusal.to_string();
    assert!(
        text.contains("line 2") && text.contains("returned_at"),
        "a refused line is named with its field: {text}"
    );
}

/// Two lines carrying one correlation id are refused as a duplicate, naming the id but not the
/// two lines that carry it.
#[test]
fn a_duplicate_carried_identity_is_refused_naming_both_lines() {
    let id = "00000000-0000-4000-8000-00000000000e";
    let log = format!(
        "{}\n{}\n",
        line(Some(id), "billing.invoice.Issue", 10, ok(20)),
        line(Some(id), "billing.invoice.Issue", 30, ok(40)),
    );
    let refusal = recorded::import(log.as_bytes(), &adapter(), &digest())
        .expect_err("one identity on two lines is refused");
    let text = refusal.to_string();
    assert!(
        text.contains("line 1") && text.contains("line 2"),
        "the refusal names both lines: {text}"
    );
}

/// Boundaries the importer already holds; kept so a regression is caught. An RFC 6901 escape in
/// the pointer resolves, a carried identity that is not the recorder's spelling is kept verbatim
/// (the committed fixture's identities coincide with the numbered fallback, so no other case tells
/// the two apart), and a float or a string instant is refused rather than truncated or parsed.
#[test]
fn escaped_pointers_resolve_carried_identities_are_kept_and_float_instants_are_refused() {
    let escaped = ADAPTER.replace("pointer: /request/client", "pointer: /request/cli~1ent~0x");
    let adapter_escaped = recorded::adapter(&escaped).expect("admitted");
    let id = "5f0c8a2e-3b7d-4c11-9e2a-7d4b1c0e9f31";
    let text = json!({
        "correlation": id,
        "request": { "cli/ent~x": 7, "command": "billing.invoice.Issue", "subject": "s", "at_ms": 1 },
        "response": { "status": "ok", "outcome": "Ok", "at_ms": 2 },
    })
    .to_string();
    let imported =
        recorded::import(text.as_bytes(), &adapter_escaped, &digest()).expect("imported");
    assert_eq!(imported.history.operations[0].operation_id.as_str(), id);
    assert!(imported.gaps.iter().all(|gap| gap.field != "operation_id"));

    let float = json!({
        "correlation": id,
        "request": { "client": "w", "command": "billing.invoice.Issue", "subject": "s", "at_ms": 1.0 },
        "response": { "status": "ok", "outcome": "Ok", "at_ms": 2 },
    })
    .to_string();
    assert!(
        matches!(
            recorded::import(float.as_bytes(), &adapter(), &digest()),
            Err(ImportRefusal::Field {
                field: "invoked_at",
                ..
            })
        ),
        "{float}"
    );
}
