//! Row sets in a recorded history (`docs/design/filtered-related-reads.md`, "Decisive acceptance":
//! history checks preserve generated-value uncertainty and actual observed authority;
//! beyond10x/ess#228, #299).
//!
//! The linearizability checker executes the same row-set decisions the interpreter makes, over
//! every order it explores: two clients retrying the same worker and batch leave a history that
//! one order of the operations explains — the first retries, the second reads its row too and is
//! refused as ambiguous — and a recording corrupted at its outcome so that both retried is a
//! violation no order explains.
//!
//! A history records no inputs, and the checker explains each answer by the candidate inputs
//! synthesis builds (`src/linearize.rs`): two concurrent binds of one set of claims under two user
//! identities are no pair of those candidates, so uniqueness is not checked from a history here.

mod support_row_sets;

use std::collections::BTreeMap;

use ess_compiler::ir::EssIr;
use ess_conformance::history::QualifiedName;
use ess_conformance::history::{self, History, Verdict};
use ess_conformance::linearize;
use ess_conformance::record::{self, Atomic, Call, Subject, Workload};
use ess_conformance::scenario::SuiteProvenance;
use support_row_sets::*;

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

fn recorded(text: &str, workload: &Workload, seed: u64) -> (EssIr, History) {
    let ir = model(text);
    let target = ess_conformance::interpret::Interpreted::for_model(ir.clone());
    begin(&target);
    let history = record::record(&ir, &Atomic(&target), workload, seed).expect("recorded");
    (ir, history)
}

fn outcomes(history: &History, command: &str) -> Vec<String> {
    let mut found: Vec<String> = history
        .operations
        .iter()
        .filter(|operation| operation.command.as_str() == command)
        .map(|operation| {
            operation
                .outcome
                .as_ref()
                .map_or_else(String::new, |outcome| outcome.as_str().to_owned())
        })
        .collect();
    found.sort();
    found
}

/// Every operation of `command` answered `from`, answered `to` instead.
fn rewritten(history: &History, command: &str, from: &str, to: &str) -> History {
    let mut changed = history.clone();
    for operation in &mut changed.operations {
        if operation.command.as_str() == command
            && operation
                .outcome
                .as_ref()
                .is_some_and(|outcome| outcome.as_str() == from)
        {
            operation.outcome = Some(QualifiedName::new(to).expect("an outcome name"));
        }
    }
    changed
}

fn record_call(delay: i64) -> Call {
    Call::new(
        "demo.jobs.Record",
        BTreeMap::from([
            ("worker_id".to_owned(), text("w1")),
            ("batch_id".to_owned(), text("b1")),
            ("delay".to_owned(), number(delay)),
        ]),
        Subject::Creates,
    )
}

fn retry_call() -> Call {
    Call::new(
        "demo.jobs.Retry",
        BTreeMap::from([
            ("worker_id".to_owned(), text("w1")),
            ("batch_id".to_owned(), text("b1")),
        ]),
        Subject::Creates,
    )
}

#[test]
fn a_second_retry_reads_the_first_one_s_row() {
    let workload = Workload {
        prefix: vec![record_call(17)],
        clients: vec![vec![retry_call()], vec![retry_call()]],
    };
    for seed in 0..4 {
        let (ir, history) = recorded(READS, &workload, seed);
        assert_eq!(
            outcomes(&history, "demo.jobs.Retry"),
            ["ambiguous", "retried"],
            "seed {seed}"
        );
        let history = written(&ir, &history);
        assert_eq!(verdict(&ir, &history), Verdict::Linearizable, "seed {seed}");
        // Both retried: the second read the store without the first's row, which no order explains.
        let past = rewritten(&history, "demo.jobs.Retry", "ambiguous", "retried");
        assert_eq!(
            verdict(&ir, &written(&ir, &past)),
            Verdict::Violation,
            "seed {seed}"
        );
    }
}
