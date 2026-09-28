//! A recorded production log is converted into `ess-history/1` through a declared adapter and
//! judged (`story:recorded-history-validation`).
//!
//! The acceptance, case by case:
//!
//! * a history recorded from the billing target with `Fault::LostUpdate` active, written as a log
//!   in a shape of its own and imported through `tests/fixtures/recorded/adapter.yaml`, is a
//!   `Violation`; the same run against the unfaulted reference is `Linearizable`;
//! * a log missing return instants is refused, naming the missing field once per operation.
//!
//! The log is deliberately not `ess-history/1`: each call is one JSON line with a correlation id,
//! nested `request`/`response` objects, epoch-millisecond timestamps, a string client label and the
//! log's own completion words. Nothing but the adapter says how it maps.

use std::path::{Path, PathBuf};

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_conformance::faulty::{self, Fault};
use ess_conformance::history::{Completion, History, QualifiedName, Verdict};
use ess_conformance::linearize::{self, DEFAULT_BUDGET};
use ess_conformance::record::{self, Atomic};
use ess_conformance::recorded::{self, ImportRefusal};
use ess_conformance::reference::Billing;
use ess_conformance::scenario::SuiteProvenance;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use serde_json::json;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

/// `examples/billing`, compiled from the files it lives in.
fn billing_model() -> EssIr {
    let base = root()
        .join("examples/billing")
        .canonicalize()
        .expect("the billing example exists");
    let mut found: Vec<PathBuf> = Vec::new();
    let mut pending = vec![base.clone()];
    while let Some(directory) = pending.pop() {
        for entry in std::fs::read_dir(&directory).expect("the example is readable") {
            let path = entry.expect("an entry").path();
            if path.is_dir() {
                pending.push(path);
            } else if path.extension().is_some_and(|it| it == "yaml") {
                found.push(path);
            }
        }
    }
    found.sort();
    let mut sources = SourceMap::new();
    let mut parsed = Vec::new();
    for path in found {
        let label = path
            .strip_prefix(&base)
            .expect("inside the example")
            .display()
            .to_string();
        let text = std::fs::read_to_string(&path).expect("readable");
        let raw = RawSpecFile::parse(&text).expect("well formed");
        sources.insert(label.clone(), text);
        parsed.push((Source::new(label), raw));
    }
    let specification = Specification::assemble(parsed).expect("billing validates");
    compile(&specification, &sources).expect("billing resolves")
}

const ADAPTER: &str = include_str!("fixtures/recorded/adapter.yaml");

/// The seed whose interleaving overlaps the two payments of the `LostUpdate` workload.
const OVERLAPPING: u64 = 0;

/// Epoch milliseconds of logical instant 0, and how many milliseconds one tick is.
const EPOCH_MS: u64 = 1_790_000_000_000;
const TICK_MS: u64 = 7;

/// The prefix the log's correlation ids carry, in place of the recorder's
/// `00000000-0000-4000-8000-`: a version-4 identity no generated (version-8) one can equal.
const CARRIED_PREFIX: &str = "5f0c8a2e-3b7d-4c11-9e2a-";

/// The correlation id the log carries for the recorder's `n`th operation, counting from 1.
fn carried_id(n: usize) -> String {
    format!("{CARRIED_PREFIX}{n:012x}")
}

/// `history` written as the log a service would keep: one JSON line per call.
fn as_log(history: &History) -> Vec<u8> {
    let mut log = Vec::new();
    for operation in &history.operations {
        let response = match operation.completion {
            Completion::Returned => json!({
                "status": "ok",
                "outcome": operation.outcome.as_ref().map(QualifiedName::as_str),
                "at_ms": operation.returned_at.map(|at| EPOCH_MS + at * TICK_MS),
            }),
            Completion::Indeterminate => json!({ "status": "timeout" }),
        };
        let line = json!({
            "correlation": operation
                .operation_id
                .as_str()
                .replacen("00000000-0000-4000-8000-", CARRIED_PREFIX, 1),
            "request": {
                "client": format!("worker-{}", operation.client),
                "command": operation.command.as_str(),
                "subject": operation.subject_key,
                "at_ms": EPOCH_MS + operation.invoked_at * TICK_MS,
            },
            "response": response,
        });
        log.extend(serde_json::to_vec(&line).expect("a line serializes"));
        log.push(b'\n');
    }
    log
}

fn recorded_log(ir: &EssIr, fault: bool) -> Vec<u8> {
    let workload = faulty::lost_update_workload();
    let history = if fault {
        record::record(
            ir,
            &faulty::billing(Fault::LostUpdate),
            &workload,
            OVERLAPPING,
        )
    } else {
        record::record(ir, &Atomic(&Billing::new()), &workload, OVERLAPPING)
    }
    .expect("the workload names what billing declares");
    as_log(&history)
}

fn imported_verdict(ir: &EssIr, log: &[u8]) -> Verdict {
    let adapter = recorded::adapter(ADAPTER).expect("the committed adapter is admitted");
    let imported = recorded::import_for(log, &adapter, ir)
        .unwrap_or_else(|refusal| panic!("the log is imported: {refusal}"));
    assert!(
        imported.gaps.iter().all(|gap| gap.field == "seed"),
        "the adapter maps every operation field and the log reads no view, so only the \
         document's seed is a gap: {:?}",
        imported.gaps
    );
    linearize::check(ir, &imported.history, DEFAULT_BUDGET)
        .unwrap_or_else(|refusal| panic!("the imported history is checked: {refusal:?}"))
        .verdict
}

#[test]
fn a_recorded_log_of_billing_under_lost_update_imports_to_a_violation() {
    let model = billing_model();
    let log = recorded_log(&model, true);
    assert_eq!(imported_verdict(&model, &log), Verdict::Violation);
}

#[test]
fn the_same_recorded_log_without_the_fault_imports_to_linearizable() {
    let model = billing_model();
    let log = recorded_log(&model, false);
    assert_eq!(imported_verdict(&model, &log), Verdict::Linearizable);
}

#[test]
fn the_imported_history_keeps_the_log_s_clients_instants_and_identities() {
    let model = billing_model();
    let log = recorded_log(&model, true);
    let adapter = recorded::adapter(ADAPTER).expect("admitted");
    let imported = recorded::import_for(&log, &adapter, &model).expect("imported");
    let history = &imported.history;
    assert_eq!(history.clients, 2, "two distinct client labels");
    assert_eq!(
        history
            .operations
            .iter()
            .map(|operation| operation.operation_id.as_str().to_owned())
            .collect::<Vec<_>>(),
        (1..=4).map(carried_id).collect::<Vec<_>>(),
        "every carried identity is kept verbatim"
    );
    let first = &history.operations[0];
    assert_eq!(
        first.client, 0,
        "`worker-0` is the first label the log names"
    );
    assert_eq!(first.invoked_at, EPOCH_MS + TICK_MS);
    assert_eq!(first.returned_at, Some(EPOCH_MS + 2 * TICK_MS));
}

/// The log with `response.at_ms` removed from every line that has one.
fn without_return_instants(log: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    for line in log
        .split(|byte| *byte == b'\n')
        .filter(|line| !line.is_empty())
    {
        let mut value: serde_json::Value = serde_json::from_slice(line).expect("a JSON line");
        if let Some(response) = value
            .get_mut("response")
            .and_then(serde_json::Value::as_object_mut)
        {
            response.remove("at_ms");
        }
        out.extend(serde_json::to_vec(&value).expect("serializes"));
        out.push(b'\n');
    }
    out
}

#[test]
fn a_log_missing_return_instants_is_refused_naming_the_field_per_operation() {
    let model = billing_model();
    let log = without_return_instants(&recorded_log(&model, true));
    let adapter = recorded::adapter(ADAPTER).expect("admitted");
    let refusal = recorded::import(&log, &adapter, &SuiteProvenance::of(&model).spec_digest)
        .expect_err("a log without return instants is refused");
    let ImportRefusal::MissingFields { missing } = &refusal else {
        panic!("refused for the missing fields, not {refusal:?}");
    };
    // Four calls, every one answered: two in the prefix and one payment per client.
    assert_eq!(missing.len(), 4, "{missing:?}");
    for (index, gap) in missing.iter().enumerate() {
        assert_eq!(gap.line, index + 1);
        assert_eq!(gap.field, "returned_at");
        assert_eq!(
            gap.operation_id.as_deref(),
            Some(carried_id(index + 1).as_str())
        );
    }
    let text = refusal.to_string();
    assert_eq!(text.matches("`returned_at`").count(), 4, "{text}");
    assert_eq!(refusal.code(), "import.missing-fields");
}

#[test]
fn an_adapter_declaring_return_instants_absent_is_refused_per_operation_too() {
    let model = billing_model();
    let log = recorded_log(&model, false);
    let declared = ADAPTER.replace(
        "  returned_at:\n    pointer: /response/at_ms\n",
        "  returned_at: absent\n",
    );
    assert_ne!(declared, ADAPTER);
    let adapter = recorded::adapter(&declared).expect("`absent` is a declaration");
    let refusal = recorded::import(&log, &adapter, &SuiteProvenance::of(&model).spec_digest)
        .expect_err("refused");
    let ImportRefusal::MissingFields { missing } = refusal else {
        panic!("refused for the missing fields");
    };
    assert_eq!(missing.len(), 4);
    assert!(missing.iter().all(|gap| gap.field == "returned_at"));
}

#[test]
fn an_optional_field_declared_absent_is_a_coverage_gap_per_operation_not_a_guess() {
    let model = billing_model();
    let log = recorded_log(&model, false);
    let declared = ADAPTER.replace(
        "  operation_id:\n    pointer: /correlation\n",
        "  operation_id: absent\n",
    );
    assert_ne!(declared, ADAPTER);
    let adapter = recorded::adapter(&declared).expect("admitted");
    let imported = recorded::import_for(&log, &adapter, &model)
        .expect("an operation identity is not needed to judge the history");
    let identity_gaps: Vec<_> = imported
        .gaps
        .iter()
        .filter(|gap| gap.field == "operation_id")
        .collect();
    assert_eq!(identity_gaps.len(), 4, "{:?}", imported.gaps);
    assert_eq!(
        identity_gaps.iter().map(|gap| gap.line).collect::<Vec<_>>(),
        vec![Some(1), Some(2), Some(3), Some(4)]
    );
    // A generated identity is version 8 with the RFC 9562 variant: neither the recorder's
    // version-4 spelling nor anything the log carried.
    for (operation, gap) in imported.history.operations.iter().zip(&identity_gaps) {
        let id = operation.operation_id.as_str();
        assert_eq!(&id[14..15], "8", "{id}");
        assert!(matches!(&id[19..20], "8" | "9" | "a" | "b"), "{id}");
        assert!(!id.starts_with("00000000-0000-4000-8000-"), "{id}");
        assert!(gap.written.contains(id), "{gap:?}");
    }
    // One log, one set of generated identities.
    let again = recorded::import_for(&log, &adapter, &model).expect("imported again");
    assert_eq!(again.history, imported.history);
    assert_eq!(
        linearize::check(&model, &imported.history, DEFAULT_BUDGET)
            .expect("checked")
            .verdict,
        Verdict::Linearizable
    );
}

#[test]
fn an_adapter_with_an_unknown_field_or_an_undeclared_one_is_refused() {
    // `rows` is the one optional field (correction F2): declared, it is admitted.
    let rows = format!("{ADAPTER}  rows:\n    pointer: /response/rows\n");
    assert!(recorded::adapter(&rows).is_ok());
    let unknown = format!("{ADAPTER}  seed:\n    pointer: /seed\n");
    assert!(matches!(
        recorded::adapter(&unknown),
        Err(ImportRefusal::Adapter { .. })
    ));
    let undeclared = ADAPTER.replace("  subject_key:\n    pointer: /request/subject\n", "");
    assert_ne!(undeclared, ADAPTER);
    assert!(matches!(
        recorded::adapter(&undeclared),
        Err(ImportRefusal::Adapter { .. })
    ));
}

#[test]
fn a_completion_word_the_adapter_does_not_map_is_refused_naming_the_line() {
    let model = billing_model();
    let log = String::from_utf8(recorded_log(&model, false))
        .expect("utf-8")
        .replacen("\"ok\"", "\"accepted\"", 1);
    let adapter = recorded::adapter(ADAPTER).expect("admitted");
    let refusal = recorded::import(
        log.as_bytes(),
        &adapter,
        &SuiteProvenance::of(&model).spec_digest,
    )
    .expect_err("an unmapped completion is never guessed");
    assert!(
        matches!(&refusal, ImportRefusal::Field { line: 1, field, .. } if *field == "completion"),
        "{refusal:?}"
    );
}

/// Correction F2: `rows` is declarable; against the specification, a `rows` gap is reported on
/// exactly the lines that read one of its views and carry no rows, and carried rows are kept.
#[test]
fn a_view_read_carries_its_rows_or_is_reported_as_a_rows_gap_on_its_line() {
    let model = billing_model();
    let with_rows = format!("{ADAPTER}  rows:\n    pointer: /response/rows\n");
    let adapter = recorded::adapter(&with_rows).expect("admitted");
    let call = |n: usize, command: &str, outcome: &str, rows: Option<Vec<&str>>| {
        let mut response = json!({ "status": "ok", "outcome": outcome, "at_ms": 10 * n + 5 });
        if let Some(rows) = rows {
            response["rows"] = json!(rows);
        }
        json!({
            "correlation": carried_id(n),
            "request": { "client": "worker-0", "command": command, "subject": "", "at_ms": 10 * n },
            "response": response,
        })
        .to_string()
    };
    let log = [
        call(
            1,
            "billing.invoice.OutstandingInvoices",
            "read",
            Some(vec!["inv-1"]),
        ),
        call(2, "billing.invoice.OutstandingInvoices", "read", None),
        call(
            3,
            "billing.invoice.IssueInvoice",
            "billing.invoice.IssueInvoice.issued",
            None,
        ),
    ]
    .join("\n");
    let imported = recorded::import_for(log.as_bytes(), &adapter, &model).expect("imported");
    let rows_gaps: Vec<_> = imported
        .gaps
        .iter()
        .filter(|gap| gap.field == "rows")
        .map(|gap| gap.line)
        .collect();
    assert_eq!(rows_gaps, vec![Some(2)], "{:?}", imported.gaps);
    assert_eq!(
        imported.history.operations[0].rows,
        Some(vec!["inv-1".to_owned()])
    );
    assert_eq!(imported.history.operations[1].rows, None);
}
