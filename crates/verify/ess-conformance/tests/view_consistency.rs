//! View reads in a recorded history are judged at the consistency their view declares
//! (`story:session-and-eventual-view-checks`).
//!
//! The acceptance, case by case:
//!
//! * `billing.invoice.InvoiceById` is `eventual`: recorded against the reference at
//!   `Billing::DEFAULT_LAG`, every read judged for convergence shows what the writes produced, and
//!   the history passes; recorded against a projection that never catches up, it fails, naming the
//!   client and the read that did not converge;
//! * `billing.invoice.OutstandingInvoices` is `read_your_writes`: a read that leaves out the
//!   client's own last write is a violation naming that client and that read;
//! * a read the checker cannot judge, or judges for less than its level promises, is listed with
//!   the reason, and never turns a history into a pass or a violation.
//!
//! The fault-matrix row (`Fault::StaleReadUnderReadYourWrites`) is in `tests/faults.rs`.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_conformance::faulty;
use ess_conformance::history::{self, History, HistoryRefusal, Operation, Verdict};
use ess_conformance::linearize::{self, Anomaly, DEFAULT_BUDGET, DEFAULT_SETTLE};
use ess_conformance::record::{Atomic, Call, Subject};
use ess_conformance::reference::Billing;
use ess_conformance::scenario::SuiteProvenance;
use ess_conformance::sessions::{self, Act};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_primitives::facts::Number;
use ess_primitives::node::Node;

const INVOICE_BY_ID: &str = "billing.invoice.InvoiceById";
const OUTSTANDING: &str = "billing.invoice.OutstandingInvoices";

/// A lag no history here reads past: a projection that, for every purpose of one run, never
/// converges.
const NEVER: u64 = 1 << 40;

/// `examples/billing`, compiled from the files it lives in.
fn billing_model() -> EssIr {
    let base = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../examples/billing")
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
        let raw = RawSpecFile::parse(&text)
            .unwrap_or_else(|error| panic!("{label} is well formed: {error}"));
        sources.insert(label.clone(), text);
        parsed.push((Source::new(label), raw));
    }
    let specification = Specification::assemble(parsed)
        .unwrap_or_else(|errors| panic!("billing validates:\n{errors}"));
    compile(&specification, &sources)
        .unwrap_or_else(|diagnostics| panic!("billing resolves:\n{diagnostics}"))
}

/// A recorded history, written out and read back, as a runner would hand it to the checker.
fn through_the_reader(ir: &EssIr, recorded: &History) -> History {
    let bytes = serde_json::to_vec(recorded).expect("a history serializes");
    history::read(&bytes, &SuiteProvenance::of(ir).spec_digest)
        .unwrap_or_else(|refusal| panic!("the recorded history is admitted: {refusal}"))
}

fn money(amount: f64) -> Node {
    Node::Map(BTreeMap::from([
        (
            "amount".to_owned(),
            Node::Number(Number::new(amount).expect("finite")),
        ),
        (
            "currency".to_owned(),
            Node::Text("amount.currency".to_owned()),
        ),
    ]))
}

fn create() -> Call {
    Call::new(
        "billing.invoice.CreateInvoice",
        BTreeMap::from([
            ("amount".to_owned(), money(5.0)),
            (
                "account_id".to_owned(),
                Node::Text("00000000-0000-4000-8000-0000000000aa".to_owned()),
            ),
            (
                "customer_email".to_owned(),
                Node::Text("payer@example.com".to_owned()),
            ),
        ]),
        Subject::Creates,
    )
}

fn issue(prefix: usize) -> Call {
    Call::new(
        "billing.invoice.IssueInvoice",
        BTreeMap::new(),
        Subject::Created(prefix),
    )
}

/// `reads` reads of `InvoiceById`, one client's session.
fn invoice_reads(reads: usize) -> Vec<Act> {
    (0..reads)
        .map(|_| Act::Read(INVOICE_BY_ID.to_owned()))
        .collect()
}

/// Every write in the prefix, then two clients that only read `InvoiceById`, twelve times each:
/// each session reads well past [`DEFAULT_SETTLE`] times after the writes stop, so its later reads
/// are judged for convergence, and the first ones are behind at `Billing::DEFAULT_LAG`.
fn eventual_workload() -> sessions::Workload {
    sessions::Workload {
        prefix: vec![create(), issue(0), create()],
        clients: vec![invoice_reads(12), invoice_reads(12)],
    }
}

/// The instant the writes stop: the latest return of any command.
fn writes_stop(history: &History) -> u64 {
    history
        .operations
        .iter()
        .filter(|operation| {
            operation.command.as_str() != INVOICE_BY_ID && operation.command.as_str() != OUTSTANDING
        })
        .filter_map(|operation| operation.returned_at)
        .max()
        .unwrap_or(0)
}

fn reads_of<'h>(history: &'h History, view: &str) -> Vec<&'h Operation> {
    history
        .operations
        .iter()
        .filter(|operation| operation.command.as_str() == view)
        .collect()
}

/// `true` when `read` is judged for convergence under [`DEFAULT_SETTLE`]: it was invoked after the
/// writes stopped, and its session (its client's reads of `InvoiceById`) had already read
/// `DEFAULT_SETTLE` times after the writes stopped. Instants are compared only by order.
fn settled(history: &History, read: &Operation) -> bool {
    let stopped = writes_stop(history);
    let before = reads_of(history, INVOICE_BY_ID)
        .into_iter()
        .filter(|earlier| {
            earlier.client == read.client
                && earlier.invoked_at > stopped
                && earlier.invoked_at < read.invoked_at
        })
        .count() as u64;
    read.invoked_at > stopped && before >= DEFAULT_SETTLE
}

// ---- eventual -------------------------------------------------------------------------------

#[test]
fn the_billing_references_eventual_invoice_by_id_at_its_default_lag_passes() {
    let model = billing_model();
    for seed in 0..8 {
        let reference = Billing::new();
        let recorded = sessions::record(
            &model,
            &Atomic(&reference),
            &reference,
            &eventual_workload(),
            seed,
        )
        .expect("the workload names what billing declares");
        let history = through_the_reader(&model, &recorded);
        let reads = reads_of(&history, INVOICE_BY_ID);
        assert_eq!(
            reads.len(),
            24,
            "seed {seed}: two clients, twelve reads each"
        );
        assert!(
            reads.iter().all(|read| read.rows.is_some()),
            "seed {seed}: every read records the rows it answered"
        );
        assert!(
            reads
                .iter()
                .any(|read| read.rows.as_ref().is_some_and(Vec::is_empty)),
            "seed {seed}: the first read after the writes is behind, or the lag is not exercised"
        );
        assert!(
            reads.iter().any(|read| settled(&history, read)),
            "seed {seed}: some read is judged for convergence"
        );
        let checked = linearize::check(&model, &history, DEFAULT_BUDGET).expect("checked");
        assert_eq!(
            checked.verdict,
            Verdict::Linearizable,
            "seed {seed}: {checked:?}"
        );
        assert!(
            checked
                .not_judged
                .iter()
                .all(|read| read.reason.starts_with(linearize::BEFORE_SETTLE)),
            "seed {seed}: only the reads before each session settled are listed: {:?}",
            checked.not_judged
        );
        assert_eq!(
            checked.not_judged.len() as u64,
            2 * DEFAULT_SETTLE,
            "seed {seed}: each session's first reads after the writes are listed"
        );
        assert_eq!(checked.read, None);
    }
}

#[test]
fn a_projection_that_never_converges_fails_naming_the_client_and_the_read() {
    let model = billing_model();
    for seed in 0..8 {
        let stuck = Billing::with_lag(NEVER);
        let recorded =
            sessions::record(&model, &Atomic(&stuck), &stuck, &eventual_workload(), seed)
                .expect("recorded");
        let history = through_the_reader(&model, &recorded);
        let checked = linearize::check(&model, &history, DEFAULT_BUDGET).expect("checked");
        assert_eq!(checked.verdict, Verdict::Violation, "seed {seed}");
        let read = checked.read.expect("the violation names a read");
        assert_eq!(read.anomaly, Anomaly::NotConverged, "seed {seed}");
        assert_eq!(read.view, INVOICE_BY_ID);
        assert_eq!(read.consistency, "eventual");
        let named = history
            .operations
            .iter()
            .find(|operation| operation.operation_id.as_str() == read.operation_id)
            .expect("the named read is in the history");
        assert_eq!(named.command.as_str(), INVOICE_BY_ID);
        assert_eq!(named.client, read.client, "the client is the read's own");
        assert!(
            settled(&history, named),
            "convergence is judged only on a read after its session's first `DEFAULT_SETTLE` \
             reads after the writes stop"
        );
        assert!(
            reads_of(&history, INVOICE_BY_ID)
                .into_iter()
                .filter(|earlier| earlier.invoked_at < named.invoked_at)
                .all(|earlier| !settled(&history, earlier)),
            "the named read is the first one judged for convergence"
        );

        let report = linearize::report(&model, &history, DEFAULT_BUDGET).expect("reported");
        let text = report.to_text();
        assert!(
            text.contains(&read.operation_id) && text.contains(&format!("client {}", read.client)),
            "the text report names the client and the read:\n{text}"
        );
        let json: serde_json::Value =
            serde_json::from_str(&report.to_json()).expect("the report is JSON");
        assert_eq!(json["read"]["operation_id"], read.operation_id.as_str());
        assert_eq!(json["read"]["client"], read.client);
        assert_eq!(json["read"]["anomaly"], "not-converged");
        let shrunk = report.shrunk.expect("a violation is shrunk");
        let again = linearize::check(&model, &through_the_reader(&model, &shrunk), DEFAULT_BUDGET)
            .expect("checked");
        assert_eq!(
            again.verdict,
            Verdict::Violation,
            "seed {seed}: {shrunk:#?}"
        );
        assert_eq!(
            again.read.map(|it| (it.operation_id, it.anomaly)),
            Some((read.operation_id.clone(), Anomaly::NotConverged)),
            "the shrunk history is the same violation"
        );
        assert!(
            shrunk.operations.len() < history.operations.len(),
            "the shrink removed something: {shrunk:#?}"
        );
    }
}

#[test]
fn a_larger_settle_judges_fewer_reads_for_convergence() {
    // The same never-converging recording, with `settle` as large as a session: no read is judged
    // for convergence, every one of them may be behind, and every one is listed as not judged.
    let model = billing_model();
    let stuck = Billing::with_lag(NEVER);
    let history = through_the_reader(
        &model,
        &sessions::record(&model, &Atomic(&stuck), &stuck, &eventual_workload(), 0)
            .expect("recorded"),
    );
    let checked = linearize::check_settled(&model, &history, DEFAULT_BUDGET, 12).expect("checked");
    assert_eq!(checked.verdict, Verdict::Linearizable, "{checked:?}");
    assert_eq!(checked.not_judged.len(), 24, "{:?}", checked.not_judged);
    assert!(checked
        .not_judged
        .iter()
        .all(|read| read.reason.starts_with(linearize::BEFORE_SETTLE)));
    let checked = linearize::check_settled(&model, &history, DEFAULT_BUDGET, 1).expect("checked");
    assert_eq!(
        checked.read.map(|it| it.anomaly),
        Some(Anomaly::NotConverged)
    );
}

#[test]
fn an_eventual_read_before_the_writes_settle_may_be_behind() {
    // One client writes while the other reads: a read before the writes stop, or among its
    // session's first `DEFAULT_SETTLE` after, may show any earlier state, including none.
    let model = billing_model();
    let workload = sessions::Workload {
        prefix: vec![create()],
        clients: vec![
            vec![Act::Call(issue(0)), Act::Call(create())],
            invoice_reads(10),
        ],
    };
    let mut sides = (0, 0);
    for seed in 0..32 {
        let stuck = Billing::with_lag(NEVER);
        let history = through_the_reader(
            &model,
            &sessions::record(&model, &Atomic(&stuck), &stuck, &workload, seed).expect("recorded"),
        );
        let judged = reads_of(&history, INVOICE_BY_ID)
            .into_iter()
            .any(|read| settled(&history, read));
        let expected = if judged {
            sides.0 += 1;
            Verdict::Violation
        } else {
            sides.1 += 1;
            Verdict::Linearizable
        };
        let checked = linearize::check(&model, &history, DEFAULT_BUDGET).expect("checked");
        assert_eq!(
            checked.verdict, expected,
            "seed {seed}: a read judged for convergence {judged}"
        );
        if !judged {
            assert!(
                !checked.not_judged.is_empty(),
                "seed {seed}: a history in which no read was judged for convergence says so"
            );
        }
    }
    assert!(
        sides.0 > 0 && sides.1 > 0,
        "the seeds exercise both sides: {} with a read judged for convergence, {} without",
        sides.0,
        sides.1
    );
}

#[test]
fn the_verdict_does_not_depend_on_the_clock_a_history_was_written_on() {
    // Instants compare only by order: stretching every instant by a different amount per position,
    // order kept, changes nothing the checker says about the never-converging recording.
    let model = billing_model();
    let stuck = Billing::with_lag(NEVER);
    let logical = through_the_reader(
        &model,
        &sessions::record(&model, &Atomic(&stuck), &stuck, &eventual_workload(), 3)
            .expect("recorded"),
    );
    let mut stretched = logical.clone();
    let stretch = |at: u64| at * at * 1_000 + 7;
    for operation in &mut stretched.operations {
        operation.invoked_at = stretch(operation.invoked_at);
        operation.returned_at = operation.returned_at.map(stretch);
    }
    let stretched = through_the_reader(&model, &stretched);
    let first = linearize::check(&model, &logical, DEFAULT_BUDGET).expect("checked");
    let second = linearize::check(&model, &stretched, DEFAULT_BUDGET).expect("checked");
    assert_eq!(first.verdict, second.verdict);
    assert_eq!(first.read, second.read);
    assert_eq!(first.not_judged, second.not_judged);
}

// ---- read your writes -------------------------------------------------------------------------

/// Two invoices created in the prefix; each client issues one and then reads the outstanding list.
fn session_workload() -> sessions::Workload {
    faulty::stale_read_workload()
}

#[test]
fn a_read_your_writes_read_of_the_reference_is_judged_and_passes() {
    let model = billing_model();
    for seed in 0..16 {
        let reference = Billing::new();
        let history = through_the_reader(
            &model,
            &sessions::record(
                &model,
                &Atomic(&reference),
                &reference,
                &session_workload(),
                seed,
            )
            .expect("recorded"),
        );
        assert_eq!(reads_of(&history, OUTSTANDING).len(), 2);
        let checked = linearize::check(&model, &history, DEFAULT_BUDGET).expect("checked");
        assert_eq!(
            checked.verdict,
            Verdict::Linearizable,
            "seed {seed}: {checked:?}"
        );
        assert!(checked.not_judged.is_empty(), "{:?}", checked.not_judged);
    }
}

#[test]
fn a_read_that_leaves_out_the_clients_own_write_is_a_stale_read_naming_both() {
    let model = billing_model();
    let reference = Billing::new();
    let mut history = through_the_reader(
        &model,
        &sessions::record(
            &model,
            &Atomic(&reference),
            &reference,
            &session_workload(),
            0,
        )
        .expect("recorded"),
    );
    // Client 1's read, with the invoice client 1 itself issued taken out of its answer.
    let issued = history
        .operations
        .iter()
        .find(|operation| {
            operation.client == 1 && operation.command.as_str() == "billing.invoice.IssueInvoice"
        })
        .expect("client 1 issues")
        .subject_key
        .clone();
    let read = history
        .operations
        .iter_mut()
        .find(|operation| operation.client == 1 && operation.command.as_str() == OUTSTANDING)
        .expect("client 1 reads");
    let rows = read.rows.as_mut().expect("the read records its rows");
    assert!(
        rows.contains(&issued),
        "the reference shows the client's own write"
    );
    rows.retain(|row| *row != issued);
    let read_id = read.operation_id.as_str().to_owned();
    let history = through_the_reader(&model, &history);

    let checked = linearize::check(&model, &history, DEFAULT_BUDGET).expect("checked");
    assert_eq!(checked.verdict, Verdict::Violation);
    let violation = checked.read.expect("the violation names a read");
    assert_eq!(violation.operation_id, read_id);
    assert_eq!(violation.client, 1);
    assert_eq!(violation.anomaly, Anomaly::StaleRead);
    assert_eq!(violation.consistency, "read_your_writes");
    assert_eq!(violation.subject_key, issued);
    assert_eq!(checked.subject_key.as_deref(), Some(issued.as_str()));
}

#[test]
fn a_row_no_write_produced_is_a_future_read() {
    let model = billing_model();
    let reference = Billing::new();
    let mut history = sessions::record(
        &model,
        &Atomic(&reference),
        &reference,
        &session_workload(),
        0,
    )
    .expect("recorded");
    let read = history
        .operations
        .iter_mut()
        .find(|operation| operation.command.as_str() == OUTSTANDING)
        .expect("a read");
    read.rows
        .as_mut()
        .expect("rows")
        .push("00000000-0000-4000-8000-0000000fffff".to_owned());
    let history = through_the_reader(&model, &history);
    let checked = linearize::check(&model, &history, DEFAULT_BUDGET).expect("checked");
    assert_eq!(checked.verdict, Verdict::Violation);
    let violation = checked.read.expect("a read violation");
    assert_eq!(violation.anomaly, Anomaly::FutureRead);
    assert_eq!(
        violation.subject_key,
        "00000000-0000-4000-8000-0000000fffff"
    );
}

#[test]
fn a_read_listing_one_instance_twice_is_a_duplicate_row() {
    let model = billing_model();
    let reference = Billing::new();
    let mut history = sessions::record(
        &model,
        &Atomic(&reference),
        &reference,
        &session_workload(),
        0,
    )
    .expect("recorded");
    let read = history
        .operations
        .iter_mut()
        .find(|operation| {
            operation.command.as_str() == OUTSTANDING
                && operation.rows.as_ref().is_some_and(|rows| !rows.is_empty())
        })
        .expect("a read with a row");
    let rows = read.rows.as_mut().expect("rows");
    let twice = rows[0].clone();
    rows.push(twice.clone());
    let read_id = read.operation_id.as_str().to_owned();
    let history = through_the_reader(&model, &history);
    let checked = linearize::check(&model, &history, DEFAULT_BUDGET).expect("checked");
    assert_eq!(checked.verdict, Verdict::Violation);
    let violation = checked.read.expect("a read violation");
    assert_eq!(violation.anomaly, Anomaly::DuplicateRow);
    assert_eq!(violation.subject_key, twice);
    assert_eq!(violation.operation_id, read_id);
}

// ---- what is not judged ------------------------------------------------------------------------

#[test]
fn a_read_that_records_no_rows_is_listed_with_its_reason_and_judges_nothing() {
    let model = billing_model();
    let reference = Billing::new();
    let mut history = sessions::record(
        &model,
        &Atomic(&reference),
        &reference,
        &session_workload(),
        0,
    )
    .expect("recorded");
    for operation in &mut history.operations {
        if operation.command.as_str() == OUTSTANDING {
            operation.rows = None;
        }
    }
    let history = through_the_reader(&model, &history);
    let checked = linearize::check(&model, &history, DEFAULT_BUDGET).expect("checked");
    assert_eq!(checked.verdict, Verdict::Linearizable);
    assert_eq!(checked.not_judged.len(), 2);
    for read in &checked.not_judged {
        assert_eq!(read.consistency, "read_your_writes");
        assert!(
            read.reason.contains("rows"),
            "the reason says what is missing: {}",
            read.reason
        );
    }
}

// ---- the format -------------------------------------------------------------------------------

#[test]
fn a_reads_rows_survive_the_reader_and_a_history_without_them_still_reads() {
    let model = billing_model();
    let reference = Billing::new();
    let recorded = sessions::record(
        &model,
        &Atomic(&reference),
        &reference,
        &session_workload(),
        0,
    )
    .expect("recorded");
    let read = through_the_reader(&model, &recorded);
    assert_eq!(read, recorded, "rows round-trip through the reader");
    let text = serde_json::to_string(&recorded).expect("serializes");
    assert!(text.contains("\"rows\""));
    let issue = recorded
        .operations
        .iter()
        .find(|operation| operation.command.as_str() == "billing.invoice.IssueInvoice")
        .expect("an issue");
    let written = serde_json::to_value(issue).expect("serializes");
    assert!(
        written.get("rows").is_none(),
        "a command writes no `rows`, so an `ess-history/1` document without the field is \
         unchanged: {written}"
    );
}

#[test]
fn rows_on_an_operation_that_never_answered_are_refused_by_name() {
    let model = billing_model();
    let reference = Billing::new();
    let mut history = sessions::record(
        &model,
        &Atomic(&reference),
        &reference,
        &session_workload(),
        0,
    )
    .expect("recorded");
    let read = history
        .operations
        .iter_mut()
        .find(|operation| operation.command.as_str() == OUTSTANDING)
        .expect("a read");
    read.completion = history::Completion::Indeterminate;
    read.returned_at = None;
    read.outcome = None;
    let id = read.operation_id.as_str().to_owned();
    let bytes = serde_json::to_vec(&history).expect("serializes");
    let refused = history::read(&bytes, &SuiteProvenance::of(&model).spec_digest)
        .expect_err("an `Indeterminate` operation carrying `rows` is refused");
    assert_eq!(refused.code(), "history.indeterminate-with-rows");
    assert_eq!(
        refused,
        HistoryRefusal::IndeterminateWithRows { operation_id: id }
    );
}
