//! Adversary cases for unit E-U8 (row sets and filtered reads, beyond10x/ess#228, #299).
//!
//! `docs/design/filtered-related-reads.md`, "Subject borrowing and precedence": addressed-row
//! existence and held-state checks keep their priority over every row-set test, and "a missing
//! addressed row never supplies a fabricated subject or turns a subject-dependent selector
//! empty". The fixture's `Close` reaches that rule only through an unguarded accepting branch;
//! here its accepting branch is itself guarded by the row set its refusal reads.

mod support_row_sets;

use std::collections::BTreeMap;

use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::synthesize::synthesize;
use ess_conformance::target::ConformanceTarget;
use ess_conformance::{AdmittedSuite, AdvancingClock, Ids, Runner, RunnerConfig};
use support_row_sets::*;

const UNKNOWN: &str = "00000000-0000-4000-8000-000000000000";

/// The attempts fixture with `Close`'s accepting branch taken only while at most one attempt of
/// the subject's worker and batch is recorded — the complement of its `crowded` refusal, so the
/// two branches answer every count and no default is declared.
fn guarded_close() -> String {
    const FROM: &str = "      - name: closed\n        moves: demo.jobs.Attempt.close\n";
    assert!(
        READS.contains(FROM),
        "the fixture's accepting `Close` branch"
    );
    READS.replacen(
        FROM,
        "      - name: closed
        when_related:
          entity: demo.jobs.Attempt
          where: {all: [worker_id == subject.worker_id, batch_id == subject.batch_id]}
          count: {lte: 1}
        moves: demo.jobs.Attempt.close
",
        1,
    )
}

/// An identity no attempt carries: the addressed row's existence answers `unknown-attempt` before
/// any row set is read, whether the accepting branch is unguarded or guarded by a row set.
#[test]
fn a_missing_subject_answers_unknown_instance_before_a_row_set_guarded_accepting_branch() {
    let target = target(&guarded_close());
    let other = record(&target, "w1", "b1", 1, None);
    // The model is live: its accepting branch closes an attempt alone with its worker and batch.
    let closed = target
        .execute_command(close(&other))
        .unwrap_or_else(|error| panic!("an existing attempt is closed: {error}"));
    assert_eq!(outcome(&closed), "closed");
    let answered = target.execute_command(close(UNKNOWN));
    match answered {
        Ok(result) => assert_eq!(outcome(&result), "unknown-attempt"),
        Err(error) => panic!(
            "an identity no attempt carries is answered by `unknown_instance`, not refused \
             as undecidable: {error}"
        ),
    }
}

/// The same command's synthesized suite, run by the reference runner against the healthy
/// interpreter: every scenario passes.
#[test]
fn the_healthy_interpreter_passes_the_suite_of_a_guarded_close() {
    let ir = model(&guarded_close());
    let synthesis = synthesize(&ir);
    let suite = synthesis.suite;
    let admitted = AdmittedSuite::from_suite(&suite).unwrap_or_else(|error| panic!("{error}"));
    let target = Interpreted::for_model(ir);
    let report = Runner::new(
        RunnerConfig::default(),
        AdvancingClock::default(),
        Ids::for_suite(&suite),
    )
    .run_admitted(&admitted, &target)
    .into_report();
    let statuses: BTreeMap<String, Status> = report
        .scenarios
        .iter()
        .map(|scenario| (scenario.scenario.to_string(), scenario.status))
        .collect();
    let not_passed: Vec<String> = report
        .scenarios
        .iter()
        .filter(|scenario| scenario.status != Status::Passed)
        .map(|scenario| format!("{} {:?}", scenario.scenario, scenario))
        .collect();
    assert!(
        statuses.contains_key("demo.jobs.Close/outcome/unknown-attempt"),
        "the existence scenario is synthesized: {:#?}\nrefusals: {:#?}",
        statuses.keys().collect::<Vec<_>>(),
        synthesis.refusals
    );
    assert_eq!(
        not_passed.len(),
        0,
        "the healthy interpreter fails its own suite:\n{}",
        not_passed.join("\n")
    );
}
