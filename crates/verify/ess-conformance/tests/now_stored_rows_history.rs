//! `now` over a stored row, recorded in `ess-history/2` and checked
//! (`docs/design/expression-family-source22.md`, "A3: current time over stored and related rows",
//! "Recorded history format 2"; beyond10x/ess#244 part a, unit U5).
//!
//! Two leases are opened, then each is renewed by its own client. The scripted provider answers
//! the setup readings, then [`EARLY`] — an instant every stored expiry the checker can explain is
//! less than an hour behind, so the decision renews — then [`LATE`], an instant every one is far
//! behind, so the decision lapses. Each operation records the instant its own decision observed,
//! and the checker reads that instant as `now` for every alternative of that operation, over the
//! stored row its search arranged. The recording is checked healthy, then corrupted at the
//! recording seam — rule 18: history verdicts check recorder corruption — and the verdict must
//! change.

mod support_occurrence_clock;

use std::collections::BTreeMap;
use std::sync::Arc;

use ess_compiler::ir::EssIr;
use ess_conformance::history::{self, History, HistoryFormat, Verdict};
use ess_conformance::linearize;
use ess_conformance::record::{self, Atomic, Call, Subject, Workload};
use ess_conformance::scenario::SuiteProvenance;
use support_occurrence_clock::{instant, model, text, Scripted, T0};

const LEASES: &str = include_str!("fixtures/now-stored-rows.yaml");

/// The second setup reading.
const T0_LATER: &str = "2000-06-01T00:00:01Z";
/// A decision every stored expiry the checker's candidates hold outlives by less than an hour.
const EARLY: &str = "2000-07-01T00:00:00.5Z";
/// A decision every one of them is far behind.
const LATE: &str = "2026-10-04T12:00:00.123456789Z";
/// What each lease is opened with, and what the recorder sends.
const OPENED_AT: &str = "2020-01-01T00:00:00Z";

fn leases_clock() -> Arc<Scripted> {
    Scripted::new(&[T0, T0_LATER, EARLY, LATE])
}

fn open_call() -> Call {
    Call::new(
        "demo.leases.OpenLease",
        BTreeMap::from([
            ("expires_at".to_owned(), text(OPENED_AT)),
            ("grace_until".to_owned(), text(OPENED_AT)),
        ]),
        Subject::Creates,
    )
}

fn renew_call(prefix: usize) -> Call {
    Call::new(
        "demo.leases.RenewLease",
        BTreeMap::from([
            ("new_expires_at".to_owned(), text(OPENED_AT)),
            ("note".to_owned(), text("renewal")),
        ]),
        Subject::Created(prefix),
    )
}

fn workload() -> Workload {
    Workload {
        prefix: vec![open_call(), open_call()],
        clients: vec![vec![renew_call(0)], vec![renew_call(1)]],
    }
}

fn verdict(ir: &EssIr, history: &History) -> Verdict {
    linearize::check(ir, history, linearize::DEFAULT_BUDGET)
        .unwrap_or_else(|refusal| panic!("checked: {refusal}"))
        .verdict
}

fn written(ir: &EssIr, history: &History) -> History {
    let bytes = serde_json::to_vec(history).expect("a recorded history serializes");
    history::read(&bytes, &SuiteProvenance::of(ir).spec_digest)
        .unwrap_or_else(|refusal| panic!("the recorded history is admitted: {refusal}"))
}

/// One recording of [`workload`] against the interpreter deciding with [`leases_clock`].
fn recorded(seed: u64) -> (EssIr, History) {
    let ir = model(LEASES);
    let clock = leases_clock();
    let target = ess_conformance::interpret::Interpreted::for_model(ir.clone())
        .with_command_clock(clock.clone());
    support_occurrence_clock::begin(&target);
    let history = record::record(&ir, &Atomic(&target), &workload(), seed).expect("recorded");
    assert_eq!(clock.reads(), 4, "one reading per decision");
    (ir, history)
}

/// The renewals of a history, with what each answered and the instant it recorded.
fn renewals(history: &History) -> Vec<(String, Option<String>)> {
    history
        .operations
        .iter()
        .filter(|operation| operation.command.as_str() == "demo.leases.RenewLease")
        .map(|operation| {
            (
                operation
                    .outcome
                    .as_ref()
                    .map_or_else(String::new, |outcome| outcome.as_str().to_owned()),
                operation
                    .decision_time
                    .map(ess_conformance::occurrence_clock::DecisionInstant::to_rfc3339),
            )
        })
        .collect()
}

#[test]
fn a3_a_stored_row_decided_by_its_recorded_instant_is_linearizable() {
    for seed in 0..4 {
        let (ir, history) = recorded(seed);
        assert_eq!(history.format, HistoryFormat::EssHistory2);
        let mut seen = renewals(&history);
        seen.sort();
        assert_eq!(
            seen,
            vec![
                ("lapsed".to_owned(), Some(LATE.to_owned())),
                ("renewed".to_owned(), Some(EARLY.to_owned())),
            ],
            "seed {seed}"
        );
        let history = written(&ir, &history);
        assert_eq!(verdict(&ir, &history), Verdict::Linearizable, "seed {seed}");
    }
}

#[test]
fn a3_a_corrupted_recorded_instant_changes_the_verdict() {
    let (ir, history) = recorded(0);
    let renewed = history
        .operations
        .iter()
        .position(|operation| {
            operation
                .outcome
                .as_ref()
                .is_some_and(|outcome| outcome.as_str() == "renewed")
        })
        .expect("a renewal renewed");
    // The recorder wrote the other decision's instant for the renewal: no candidate stored expiry
    // is less than an hour behind it.
    let mut swapped = history.clone();
    swapped.operations[renewed].decision_time = Some(instant(LATE));
    assert_eq!(verdict(&ir, &written(&ir, &swapped)), Verdict::Violation);
    // The recorder wrote the setup instant: every lease is still renewable then, and so is the one
    // that lapsed.
    let lapsed = history
        .operations
        .iter()
        .position(|operation| {
            operation
                .outcome
                .as_ref()
                .is_some_and(|outcome| outcome.as_str() == "lapsed")
        })
        .expect("a renewal lapsed");
    let mut setup = history.clone();
    setup.operations[lapsed].decision_time = Some(instant(T0));
    assert_eq!(verdict(&ir, &written(&ir, &setup)), Verdict::Violation);
    // The recorder dropped the instant: the stored row's guard needs one, so no verdict at all.
    let mut dropped = history.clone();
    dropped.operations[lapsed].decision_time = None;
    match linearize::check(&ir, &written(&ir, &dropped), linearize::DEFAULT_BUDGET) {
        Ok(checked) => panic!(
            "a verdict without the instant it needs: {:?}",
            checked.verdict
        ),
        Err(refusal) => {
            assert_eq!(refusal.code(), "check.model-undetermined", "{refusal}");
            assert!(refusal.to_string().contains("\"now"), "{refusal}");
        }
    }
}
