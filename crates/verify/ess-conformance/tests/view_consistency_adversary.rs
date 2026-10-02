//! Adversary cases for `story:session-and-eventual-view-checks`: view reads judged against the one
//! linearization the search found, quiescence, and the `rows` field of `ess-history/1`.
//!
//! Every history here is the reference implementation's (`Billing`), recorded by the session
//! recorder, or a recorded history edited only where a real runner would write it that way (a call
//! that timed out). A `Violation` on any of them is a false one.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_conformance::faulty;
use ess_conformance::history::{self, Completion, History, Verdict};
use ess_conformance::linearize::{self, Anomaly, DEFAULT_BUDGET};
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
const ISSUE: &str = "billing.invoice.IssueInvoice";

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
        ISSUE,
        BTreeMap::from([(
            "issued_at".to_owned(),
            Node::Text(format!("2026-01-05T09:00:{:02}Z", prefix + 1)),
        )]),
        Subject::Created(prefix),
    )
}

/// A payment of nothing: `rejected` from every state, so it changes nothing and commutes with
/// every other call on the invoice.
fn refused_payment(prefix: usize) -> Call {
    Call::new(
        "billing.invoice.PayInvoice",
        BTreeMap::from([("amount".to_owned(), money(0.0))]),
        Subject::Created(prefix),
    )
}

// ---- one linearization -------------------------------------------------------------------------

/// Client 0 issues the invoice; client 1 makes a refused payment on it and then reads its
/// outstanding list. Client 1's own write moved nothing, so read-your-writes asks nothing of the
/// read; where client 0's issue is still in flight the reference rightly does not list the invoice.
/// Both orders of the two concurrent calls answer what was recorded, and the search, trying moves
/// in invoke order, finds the one with the issue first — against which the read looks stale.
#[test]
fn a_read_after_the_readers_own_refused_write_is_judged_against_every_order_not_one() {
    let model = billing_model();
    let workload = sessions::Workload {
        prefix: vec![create()],
        clients: vec![
            vec![Act::Call(issue(0))],
            vec![
                Act::Call(refused_payment(0)),
                Act::Read(OUTSTANDING.to_owned()),
            ],
        ],
    };
    let mut raced = 0;
    let mut false_violations = Vec::new();
    for seed in 0..256 {
        let reference = Billing::new();
        let history = through_the_reader(
            &model,
            &sessions::record(&model, &Atomic(&reference), &reference, &workload, seed)
                .expect("recorded"),
        );
        let issued = history
            .operations
            .iter()
            .find(|operation| operation.command.as_str() == ISSUE)
            .expect("an issue");
        let read = history
            .operations
            .iter()
            .find(|operation| operation.command.as_str() == OUTSTANDING)
            .expect("a read");
        let paid = history
            .operations
            .iter()
            .find(|operation| operation.command.as_str() == "billing.invoice.PayInvoice")
            .expect("a payment");
        // The search tries moves in invoke order: the issue first, where it was invoked first.
        if issued.invoked_at < paid.invoked_at
            && issued
                .returned_at
                .is_some_and(|at| at > read.returned_at.unwrap_or(0))
        {
            raced += 1;
        }
        let checked = linearize::check(&model, &history, DEFAULT_BUDGET).expect("checked");
        if checked.verdict != Verdict::Linearizable {
            false_violations.push((seed, checked.read.map(|it| it.anomaly)));
        }
    }
    assert!(raced > 0, "no seed read while the issue was in flight");
    assert!(
        false_violations.is_empty(),
        "the reference implementation, recorded, is judged a violation at (seed, anomaly) {false_violations:?}: \
         another order of the concurrent calls explains every read"
    );
}

/// Client 0's issue times out — a runner writes it `Indeterminate` — but took effect, and client
/// 1's later read lists the invoice. The order in which the timed-out call happened explains the
/// read; the search tries "it never happened" first and judges the read against that order.
#[test]
fn a_read_showing_a_timed_out_write_that_took_effect_is_not_a_future_read() {
    let model = billing_model();
    let mut checked_any = false;
    for seed in 0..32 {
        let reference = Billing::new();
        let mut history = sessions::record(
            &model,
            &Atomic(&reference),
            &reference,
            &faulty::stale_read_workload(),
            seed,
        )
        .expect("recorded");
        let issue_index = history
            .operations
            .iter()
            .position(|operation| operation.client == 0 && operation.command.as_str() == ISSUE)
            .expect("client 0 issues");
        let read_index = history
            .operations
            .iter()
            .position(|operation| {
                operation.client == 1 && operation.command.as_str() == OUTSTANDING
            })
            .expect("client 1 reads");
        let (issued_key, issue_returned) = {
            let issue = &history.operations[issue_index];
            (
                issue.subject_key.clone(),
                issue.returned_at.expect("answered"),
            )
        };
        // Only where the issue had returned before client 1 read: it took effect by then.
        if issue_returned >= history.operations[read_index].invoked_at {
            continue;
        }
        let read = &mut history.operations[read_index];
        let rows = read.rows.get_or_insert_with(Vec::new);
        if !rows.contains(&issued_key) {
            rows.push(issued_key.clone());
        }
        // The runner's view of it: the call timed out, and client 0 did nothing after it.
        let issue = &mut history.operations[issue_index];
        issue.completion = Completion::Indeterminate;
        issue.returned_at = None;
        issue.outcome = None;
        history.operations.retain(|operation| {
            !(operation.client == 0 && operation.command.as_str() == OUTSTANDING)
        });
        let history = through_the_reader(&model, &history);
        let checked = linearize::check(&model, &history, DEFAULT_BUDGET).expect("checked");
        checked_any = true;
        assert_eq!(
            checked.verdict,
            Verdict::Linearizable,
            "seed {seed}: a timed-out issue that took effect explains client 1's read of \
             {issued_key}, and the check reports {:?}",
            checked.read
        );
    }
    assert!(checked_any, "no seed read after the issue returned");
}

// ---- quiescence ----------------------------------------------------------------------------------

/// The reference at `Billing::DEFAULT_LAG` is the conformant eventual projection. Two clients each
/// read `InvoiceById` once after every write returned. Nothing in the history says the projection
/// will not catch up; eventual consistency promises convergence, not convergence by the next read.
#[test]
fn one_read_per_session_after_the_writes_stop_does_not_fail_a_conformant_eventual_projection() {
    let model = billing_model();
    let workload = sessions::Workload {
        prefix: vec![create(), issue(0), create()],
        clients: vec![
            vec![Act::Read(INVOICE_BY_ID.to_owned())],
            vec![Act::Read(INVOICE_BY_ID.to_owned())],
        ],
    };
    let mut verdicts = Vec::new();
    for seed in 0..8 {
        let reference = Billing::new();
        let history = through_the_reader(
            &model,
            &sessions::record(&model, &Atomic(&reference), &reference, &workload, seed)
                .expect("recorded"),
        );
        let checked = linearize::check(&model, &history, DEFAULT_BUDGET).expect("checked");
        if checked.verdict != Verdict::Linearizable {
            verdicts.push((seed, checked.read.map(|it| it.anomaly)));
        }
    }
    assert!(
        verdicts.is_empty(),
        "the reference at its default lag is judged not converged at (seed, anomaly) {verdicts:?}"
    );
}

// ---- the judge's upper bound ---------------------------------------------------------------------

/// Kills the mutant that drops `hi` (the reads-show-nothing-not-yet-asked-for bound): a read listing
/// an invoice whose issue was invoked only after the read returned is a future read. Green on the
/// tree; nothing else in the suite asserts `FutureRead` for a subject the history does address.
#[test]
fn a_read_listing_an_invoice_issued_only_after_it_returned_is_a_future_read() {
    let model = billing_model();
    let mut found = false;
    for seed in 0..32 {
        let reference = Billing::new();
        let mut history = sessions::record(
            &model,
            &Atomic(&reference),
            &reference,
            &faulty::stale_read_workload(),
            seed,
        )
        .expect("recorded");
        let Some((read_index, later_key)) =
            history
                .operations
                .iter()
                .enumerate()
                .find_map(|(index, read)| {
                    if read.command.as_str() != OUTSTANDING {
                        return None;
                    }
                    let returned = read.returned_at?;
                    history
                        .operations
                        .iter()
                        .find(|other| {
                            other.command.as_str() == ISSUE
                                && other.client != read.client
                                && other.invoked_at > returned
                        })
                        .map(|other| (index, other.subject_key.clone()))
                })
        else {
            continue;
        };
        history.operations[read_index]
            .rows
            .as_mut()
            .expect("rows")
            .push(later_key.clone());
        let history = through_the_reader(&model, &history);
        let checked = linearize::check(&model, &history, DEFAULT_BUDGET).expect("checked");
        assert_eq!(checked.verdict, Verdict::Violation, "seed {seed}");
        let read = checked.read.expect("a read violation");
        assert_eq!(read.anomaly, Anomaly::FutureRead, "seed {seed}");
        assert_eq!(read.subject_key, later_key);
        found = true;
    }
    assert!(found, "no seed read before the other client's issue");
}

// ---- the format ----------------------------------------------------------------------------------

/// `rows` is documented as written only on a `Returned` read, beside `outcome`, which the reader
/// refuses on an `Indeterminate` operation by name. A read that never answered cannot have answered
/// rows; the reader admits them.
#[test]
fn rows_on_an_operation_that_never_answered_are_refused_like_an_outcome_is() {
    let model = billing_model();
    let reference = Billing::new();
    let mut history = sessions::record(
        &model,
        &Atomic(&reference),
        &reference,
        &faulty::stale_read_workload(),
        0,
    )
    .expect("recorded");
    let read = history
        .operations
        .iter_mut()
        .find(|operation| operation.command.as_str() == OUTSTANDING)
        .expect("a read");
    read.completion = Completion::Indeterminate;
    read.returned_at = None;
    read.outcome = None;
    assert!(read.rows.is_some());
    let bytes = serde_json::to_vec(&history).expect("serializes");
    assert!(
        history::read(&bytes, &SuiteProvenance::of(&model).spec_digest).is_err(),
        "an `Indeterminate` read carrying `rows` is admitted"
    );
}
