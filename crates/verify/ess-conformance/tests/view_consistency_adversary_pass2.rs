//! Second adversary pass on `story:session-and-eventual-view-checks`: the reachable-state judge,
//! the settle interval, and what `ess-history/1` promises about its instants.
//!
//! Every history here is the reference implementation's (`Billing`) or a never-converging
//! `Billing::with_lag`, recorded by the session recorder, and edited only the way a different writer
//! would write it (another clock) or a faulty projection would answer (a row twice).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_conformance::faulty;
use ess_conformance::history::{self, History, Verdict};
use ess_conformance::linearize::{self, DEFAULT_BUDGET};
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
    Call::new(ISSUE, BTreeMap::new(), Subject::Created(prefix))
}

fn invoice_reads(reads: usize) -> Vec<Act> {
    (0..reads)
        .map(|_| Act::Read(INVOICE_BY_ID.to_owned()))
        .collect()
}

/// The acceptance's own eventual workload (`tests/view_consistency.rs`): the prefix writes, then two
/// clients reading `InvoiceById` twelve times each.
fn eventual_workload() -> sessions::Workload {
    sessions::Workload {
        prefix: vec![create(), issue(0), create()],
        clients: vec![invoice_reads(12), invoice_reads(12)],
    }
}

// ---- instants are compared only by order ------------------------------------------------------

/// `ess-history/1` says of its instants: "the checker compares instants only by order, so
/// milliseconds, nanoseconds or a logical counter all serve" (`src/history.rs:28-30`, and the schema's
/// `invoked_at` description). The settle interval subtracts instants, so the same history written on
/// a finer clock — every instant multiplied by one million, order unchanged — is judged differently:
/// the reference at `Billing::DEFAULT_LAG`, `Linearizable` on the logical clock, becomes a
/// not-converged violation on a nanosecond clock.
#[test]
fn the_same_history_on_a_finer_clock_gets_the_same_verdict() {
    let model = billing_model();
    let mut changed = Vec::new();
    for seed in 0..8 {
        let reference = Billing::new();
        let logical = through_the_reader(
            &model,
            &sessions::record(
                &model,
                &Atomic(&reference),
                &reference,
                &eventual_workload(),
                seed,
            )
            .expect("recorded"),
        );
        let mut finer = logical.clone();
        for operation in &mut finer.operations {
            operation.invoked_at *= 1_000_000;
            operation.returned_at = operation.returned_at.map(|at| at * 1_000_000);
        }
        let finer = through_the_reader(&model, &finer);
        let on_logical = linearize::check(&model, &logical, DEFAULT_BUDGET).expect("checked");
        let on_finer = linearize::check(&model, &finer, DEFAULT_BUDGET).expect("checked");
        if on_logical.verdict != on_finer.verdict {
            changed.push((
                seed,
                on_logical.verdict,
                on_finer.verdict,
                on_finer.read.map(|it| it.anomaly),
            ));
        }
    }
    assert!(
        changed.is_empty(),
        "an order-preserving change of clock changes the verdict at \
         (seed, logical, nanosecond, anomaly) {changed:?}"
    );
}

// ---- a convergence nobody judged ----------------------------------------------------------------

/// A projection that never converges, read once per session after the writes stop. No read is
/// invoked `DEFAULT_SETTLE` instants after the writes, so convergence is never judged — and the
/// report says `Linearizable` with nothing listed as not judged, the same report a converging
/// projection gets. "Unknown is not a pass" (module docs) is not applied to the one claim the
/// history could not decide.
#[test]
fn a_never_converging_projection_whose_convergence_was_never_judged_is_not_a_silent_pass() {
    let model = billing_model();
    let workload = sessions::Workload {
        prefix: vec![create(), issue(0), create()],
        clients: vec![invoice_reads(1), invoice_reads(1)],
    };
    let mut silent = Vec::new();
    for seed in 0..8 {
        let stuck = Billing::with_lag(1 << 40);
        let history = through_the_reader(
            &model,
            &sessions::record(&model, &Atomic(&stuck), &stuck, &workload, seed).expect("recorded"),
        );
        let checked = linearize::check(&model, &history, DEFAULT_BUDGET).expect("checked");
        if checked.verdict == Verdict::Linearizable && checked.not_judged.is_empty() {
            silent.push(seed);
        }
    }
    assert!(
        silent.is_empty(),
        "a projection that never converges is reported Linearizable, nothing not judged, at seeds \
         {silent:?}"
    );
}

// ---- rows --------------------------------------------------------------------------------------

/// A read that lists one invoice twice. A view over instances has one row per instance; the judge
/// folds `rows` into a set (`src/linearize.rs`, `ViewRead::rows`), so a projection that duplicates a
/// row is judged as if it had not. The reader admits it too. Either refusing it or judging it a
/// violation would do; accepting it silently does not.
#[test]
fn a_read_listing_one_invoice_twice_is_not_linearizable() {
    let model = billing_model();
    let mut accepted = Vec::new();
    for seed in 0..16 {
        let reference = Billing::new();
        let mut recorded = sessions::record(
            &model,
            &Atomic(&reference),
            &reference,
            &faulty::stale_read_workload(),
            seed,
        )
        .expect("recorded");
        let Some(read) = recorded.operations.iter_mut().find(|operation| {
            operation.command.as_str() == OUTSTANDING
                && operation.rows.as_ref().is_some_and(|rows| !rows.is_empty())
        }) else {
            continue;
        };
        let rows = read.rows.as_mut().expect("rows");
        rows.push(rows[0].clone());
        let bytes = serde_json::to_vec(&recorded).expect("serializes");
        let Ok(admitted) = history::read(&bytes, &SuiteProvenance::of(&model).spec_digest) else {
            continue;
        };
        let checked = linearize::check(&model, &admitted, DEFAULT_BUDGET).expect("checked");
        if checked.verdict == Verdict::Linearizable {
            accepted.push(seed);
        }
    }
    assert!(
        accepted.is_empty(),
        "a read listing one invoice twice is admitted and judged Linearizable at seeds {accepted:?}"
    );
}

// ---- the session, not everyone ------------------------------------------------------------------

/// Kills the mutant that drops `own.client == operation.client` from the read-your-writes judge
/// (turning the session guarantee into "every write returned before the read"): a read that leaves
/// out **another** client's invoice, issued and returned before the read was invoked, is allowed.
/// Green on the tree; nothing else in the suite has a read miss another client's returned write.
#[test]
fn a_read_that_leaves_out_another_clients_returned_write_is_not_a_stale_read() {
    let model = billing_model();
    let mut exercised = 0;
    for seed in 0..32 {
        let reference = Billing::new();
        let mut recorded = sessions::record(
            &model,
            &Atomic(&reference),
            &reference,
            &faulty::stale_read_workload(),
            seed,
        )
        .expect("recorded");
        let operations = recorded.operations.clone();
        let Some(read_index) = recorded.operations.iter().position(|read| {
            read.command.as_str() == OUTSTANDING
                && operations.iter().any(|other| {
                    other.command.as_str() == ISSUE
                        && other.client != read.client
                        && other.returned_at.is_some_and(|at| at < read.invoked_at)
                        && read
                            .rows
                            .as_ref()
                            .is_some_and(|rows| rows.contains(&other.subject_key))
                })
        }) else {
            continue;
        };
        let read = &operations[read_index];
        let others: Vec<String> = operations
            .iter()
            .filter(|other| {
                other.command.as_str() == ISSUE
                    && other.client != read.client
                    && other.returned_at.is_some_and(|at| at < read.invoked_at)
            })
            .map(|other| other.subject_key.clone())
            .collect();
        recorded.operations[read_index]
            .rows
            .as_mut()
            .expect("rows")
            .retain(|row| !others.contains(row));
        exercised += 1;
        let history = through_the_reader(&model, &recorded);
        let checked = linearize::check(&model, &history, DEFAULT_BUDGET).expect("checked");
        assert_eq!(
            checked.verdict,
            Verdict::Linearizable,
            "seed {seed}: read-your-writes promises a client its own writes, not everyone's: {:?}",
            checked.read
        );
    }
    assert!(
        exercised > 0,
        "no seed has a read after another client's returned issue"
    );
}
