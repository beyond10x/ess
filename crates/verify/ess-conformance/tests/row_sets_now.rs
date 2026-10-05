//! `now` inside a row set's `where` and `forall` (`docs/design/expression-family-source22.md`, A3:
//! "the approved row-set selector's `where` and its `forall`"; `docs/design/filtered-related-reads.md`;
//! beyond10x/ess#228, #299): one decision reads every candidate row with the one instant its
//! command clock answered, over the rows as they were before the outcome; without a clock only a
//! decision that needs `now` is Unknown.
//!
//! The model is the attempts fixture with a stored `due_at` and a command `Expire` whose one selector
//! reads `now`: the attempts of the worker and batch due more than half a minute before the decision
//! (`where`). `overdue` while more than one is, `on-time` while every one of them fell due within the
//! hour (`forall`), and `pending` otherwise.

mod support_row_sets;

use std::collections::{BTreeMap, BTreeSet};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use ess_conformance::interpret::Interpreted;
use ess_conformance::occurrence_clock::{CommandClock, DecisionInstant};
use ess_conformance::report::Status;
use ess_conformance::synthesize::synthesize;
use ess_conformance::target::ConformanceTarget;
use ess_conformance::{now_offset, AdmittedSuite, AdvancingClock, Ids, Runner, RunnerConfig};
use ess_primitives::time::Timestamp;
use support_row_sets::*;

/// The attempts fixture with `due_at` stored by `Record`, and `Expire` deciding by it against `now`.
fn timed() -> String {
    let text = READS
        .replace(
            "      - {name: note, type: Optional<String>}\n    lifecycle:",
            "      - {name: note, type: Optional<String>}\n      - {name: due_at, type: Optional<Timestamp>}\n    lifecycle:",
        )
        .replace(
            "      - {name: note, type: Optional<String>}\n    outcomes:\n      - name: recorded",
            "      - {name: note, type: Optional<String>}\n      - {name: due_at, type: Timestamp}\n    outcomes:\n      - name: recorded",
        )
        .replace(
            "sets: {worker_id: input.worker_id, batch_id: input.batch_id, delay: input.delay, note: input.note}",
            "sets: {worker_id: input.worker_id, batch_id: input.batch_id, delay: input.delay, note: input.note, due_at: input.due_at}",
        )
        .replace(
            "  - {name: demo.jobs.UnknownAttempt,",
            "  - {name: demo.jobs.Overdue, summary: An attempt of the worker and batch is overdue., fields: []}\n  - {name: demo.jobs.Pending, summary: An attempt of the worker and batch is due soon., fields: []}\n  - {name: demo.jobs.UnknownAttempt,",
        )
        .replace(
            "  - name: demo.jobs.Close\n",
            "  - name: demo.jobs.Expire
    input:
      - {name: worker_id, type: String}
      - {name: batch_id, type: String}
    outcomes:
      - name: overdue
        when_related:
          entity: demo.jobs.Attempt
          where: {all: [worker_id == input.worker_id, batch_id == input.batch_id, defined(due_at), due_at < now - 30s]}
          count: {gt: 1}
        error: demo.jobs.Overdue
      - name: on-time
        when_related:
          entity: demo.jobs.Attempt
          where: {all: [worker_id == input.worker_id, batch_id == input.batch_id, defined(due_at), due_at < now - 30s]}
          forall: due_at >= now - 1h
        emits: [demo.jobs.WithinLimit]
        payload:
          demo.jobs.WithinLimit: {worker_id: input.worker_id}
      - name: pending
        error: demo.jobs.Pending
  - name: demo.jobs.Close\n",
        );
    assert!(text.contains("demo.jobs.Expire") && text.contains("due_at: input.due_at"));
    text
}

/// The decision instant of the command under test.
const T1: &str = "2026-10-04T12:00:00.5Z";
/// The runner's wall clock, half a second before [`T1`].
const WALL_MS: u64 = 1_791_115_200_000;

/// A command clock answering one instant, counting its reads.
#[derive(Debug)]
struct Fixed {
    at: DecisionInstant,
    reads: AtomicUsize,
}

impl Fixed {
    fn new(text: &str) -> Arc<Self> {
        Arc::new(Self {
            at: DecisionInstant::parse(text).unwrap(),
            reads: AtomicUsize::new(0),
        })
    }
}

impl CommandClock for Fixed {
    fn read(&self) -> Option<DecisionInstant> {
        self.reads.fetch_add(1, Ordering::SeqCst);
        Some(self.at)
    }
}

fn timed_target(clock: Option<Arc<Fixed>>) -> Interpreted {
    let target = Interpreted::for_model(model(&timed()));
    let target = match clock {
        Some(clock) => target.with_command_clock(clock),
        None => target,
    };
    begin(&target);
    target
}

fn record_due(target: &dyn ConformanceTarget, worker: &str, batch: &str, due_at: &str) {
    let answer = target
        .execute_command(request(
            "demo.jobs.Record",
            &[
                ("worker_id", text(worker)),
                ("batch_id", text(batch)),
                ("delay", number(1)),
                ("due_at", text(due_at)),
            ],
        ))
        .unwrap_or_else(|error| panic!("recorded: {error}"));
    assert_eq!(outcome(&answer), "recorded");
}

fn expire() -> ess_conformance::target::SemanticCommandRequest {
    request(
        "demo.jobs.Expire",
        &[("worker_id", text("w1")), ("batch_id", text("b1"))],
    )
}

#[test]
fn now_in_where_and_forall_reads_the_decision_instant() {
    for (dues, expected) in [
        // `due_at < now - 30s` selects: two a nanosecond inside the boundary are two rows.
        (
            [
                "2026-10-04T11:59:30.499999999Z",
                "2026-10-04T11:59:30.499999999Z",
            ],
            "overdue",
        ),
        // One on the boundary is not selected: one row, within the hour.
        (
            ["2026-10-04T11:59:30.5Z", "2026-10-04T11:59:30.499999999Z"],
            "on-time",
        ),
        // `forall: due_at >= now - 1h`: on the boundary, and a nanosecond past it.
        (
            ["2026-10-04T11:00:00.5Z", "2090-01-01T00:00:00Z"],
            "on-time",
        ),
        (
            ["2026-10-04T11:00:00.499999999Z", "2090-01-01T00:00:00Z"],
            "pending",
        ),
    ] {
        let clock = Fixed::new(T1);
        let target = timed_target(Some(clock.clone()));
        // A decoy of another batch, overdue whenever it is read: the selector leaves it out.
        record_due(&target, "w1", "b2", "2000-01-01T00:00:00Z");
        for due in dues {
            record_due(&target, "w1", "b1", due);
        }
        let before = clock.reads.load(Ordering::SeqCst);
        let answer = target.execute_command(expire()).unwrap();
        assert_eq!(outcome(&answer), expected, "{dues:?}");
        assert_eq!(
            clock.reads.load(Ordering::SeqCst) - before,
            1,
            "one reading decides every row"
        );
    }
    // No row is selected: `forall` is true of none.
    let target = timed_target(Some(Fixed::new(T1)));
    record_due(&target, "w2", "b1", "2000-01-01T00:00:00Z");
    assert_eq!(
        outcome(&target.execute_command(expire()).unwrap()),
        "on-time"
    );
}

#[test]
fn without_a_clock_only_a_decision_needing_now_is_unknown() {
    let target = timed_target(None);
    record_due(&target, "w1", "b1", "2090-01-01T00:00:00Z");
    let answered = target.execute_command(expire());
    assert!(
        answered
            .as_ref()
            .map_or(true, |result| result.outcome.is_none()),
        "{answered:?}"
    );
    // A command that reads no `now` is answered.
    assert_eq!(
        outcome(&target.execute_command(retry("w1", "b1")).unwrap()),
        "retried"
    );
}

fn run(clock: Arc<Fixed>) -> BTreeMap<String, Status> {
    let suite = synthesize(&model(&timed())).suite;
    let admitted = AdmittedSuite::from_suite(&suite).unwrap();
    Runner::new(
        RunnerConfig::default(),
        now_offset::WithWall::new(AdvancingClock::default(), || {
            Timestamp::from_epoch_millis(WALL_MS)
        }),
        Ids::for_suite(&suite),
    )
    .run_admitted(&admitted, &timed_target(Some(clock)))
    .into_report()
    .scenarios
    .into_iter()
    .map(|scenario| (scenario.scenario.to_string(), scenario.status))
    .collect()
}

#[test]
fn the_synthesized_row_set_scenarios_reading_now_pass_at_the_decision_instant() {
    let synthesis = synthesize(&model(&timed()));
    let ids: BTreeSet<String> = synthesis
        .suite
        .scenarios
        .keys()
        .map(ToString::to_string)
        .collect();
    for id in [
        "demo.jobs.Expire/outcome/overdue",
        "demo.jobs.Expire/outcome/on-time",
        "demo.jobs.Expire/outcome/pending",
    ] {
        assert!(
            ids.contains(id),
            "{id}: {ids:#?}\n{:#?}",
            synthesis.refusals
        );
    }
    let statuses = run(Fixed::new(T1));
    let failed: BTreeSet<&str> = statuses
        .iter()
        .filter(|(_, status)| **status != Status::Passed)
        .map(|(id, _)| id.as_str())
        .collect();
    assert_eq!(failed, BTreeSet::new(), "{statuses:#?}");
    // A target deciding with an instant a day earlier — the arrangement's, reused — fails.
    let statuses = run(Fixed::new("2026-10-03T12:00:00.5Z"));
    let failed: BTreeSet<&str> = statuses
        .iter()
        .filter(|(_, status)| **status != Status::Passed)
        .map(|(id, _)| id.as_str())
        .collect();
    assert!(
        failed.contains("demo.jobs.Expire/outcome/overdue")
            || failed.contains("demo.jobs.Expire/outcome/pending"),
        "{statuses:#?}"
    );
}
