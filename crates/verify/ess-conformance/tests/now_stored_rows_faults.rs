//! The implementation faults of A3, each failing a synthesized scenario
//! (`docs/design/expression-family-source22.md`, "A3: current time over stored and related rows";
//! beyond10x/ess#244 part a, unit U5): a decision by the wrong related row, by the row as its own
//! effect would leave it, by comparing instants as bytes, and by a `now` predicate read before the
//! input and existence refusals declared ahead of it.
//!
//! Each fault is the interpreter steered to the branch the faulty reading takes
//! (`support_now_stored::Steered`), run by the reference runner against the suite
//! `tests/fixtures/now-stored-rows.yaml` synthesizes, with the wall clock at `WALL_MS` and the
//! decision instant `T1` just after it. `tests/now_stored_rows_runtimes.rs` holds the Go and
//! TypeScript runtimes to the same verdicts.

mod support_now_stored;
mod support_occurrence_clock;

use std::collections::{BTreeMap, BTreeSet};

use ess_conformance::report::Status;
use ess_conformance::synthesize::synthesize;
use ess_conformance::{now_offset, AdmittedSuite, AdvancingClock, Ids, Runner, RunnerConfig};
use ess_primitives::time::Timestamp;
use support_now_stored::{Fault, Steered, LEASES, WALL_MS};
use support_occurrence_clock::model;

fn run(fault: Option<Fault>) -> BTreeMap<String, Status> {
    let suite = synthesize(&model(LEASES)).suite;
    let admitted = AdmittedSuite::from_suite(&suite).unwrap();
    let runner = Runner::new(
        RunnerConfig::default(),
        now_offset::WithWall::new(AdvancingClock::default(), || {
            Timestamp::from_epoch_millis(WALL_MS)
        }),
        Ids::for_suite(&suite),
    );
    runner
        .run_admitted(&admitted, &Steered::new(model(LEASES), fault))
        .into_report()
        .scenarios
        .into_iter()
        .map(|scenario| (scenario.scenario.to_string(), scenario.status))
        .collect()
}

fn failed(statuses: &BTreeMap<String, Status>) -> BTreeSet<&str> {
    statuses
        .iter()
        .filter(|(_, status)| **status != Status::Passed)
        .map(|(id, _)| id.as_str())
        .collect()
}

#[test]
fn a3_the_steered_interpreter_unfaulted_passes_every_scenario() {
    let statuses = run(None);
    assert_eq!(failed(&statuses), BTreeSet::new(), "{statuses:#?}");
}

#[test]
fn a3_faults_fail_synthesized_scenarios() {
    for (fault, expected) in support_now_stored::FAULT_KILLS {
        let statuses = run(Some(*fault));
        let failed = failed(&statuses);
        for id in *expected {
            assert!(failed.contains(id), "{fault:?}: {id} passed: {failed:#?}");
        }
    }
}

/// The finding this suite cannot close (`support_now_stored::UNCAUGHT_BYTES`): a byte comparison
/// against the decision's instant spelled in UTC, or east of it, decides every sound witness as
/// the instant comparison does. Held here so a change that starts catching it is seen.
#[test]
fn a3_a_byte_comparison_spelled_in_utc_or_east_of_it_is_caught_by_no_scenario() {
    for fault in support_now_stored::UNCAUGHT_BYTES {
        let statuses = run(Some(fault));
        assert_eq!(failed(&statuses), BTreeSet::new(), "{fault:?}");
    }
}
