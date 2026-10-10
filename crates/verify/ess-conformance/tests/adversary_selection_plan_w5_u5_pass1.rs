//! Adversary pass 1 against `story:synthesis-reads-selection-plan` unit 5 (`row_set.rs` reads the
//! precedence plan).
//!
//! * A1: the one shape where `input_for`'s refute set moved: an `external:` refusal with
//!   `compensates: true` acting on the supplied instance, with an accepting `when:` declared before
//!   it, on a row-set command with `unknown_instance:`. `synthesize::unknown_instance` sends
//!   `row_set::input_for`'s input for it. The suite must pass the model interpreter.
//! * A2: whole-suite synthesis on every row-set fixture with phases exchanged, run on the
//!   interpreter under the same exchange: the plan and the interpreter read one order.
//! * A3: a recorded history of a command whose answer the exchange moves, checked under the same
//!   exchange.
mod support_row_sets;

use std::collections::BTreeMap;

use ess_compiler::ir::EssIr;
use ess_conformance::history::{self, Verdict};
use ess_conformance::interpret::Interpreted;
use ess_conformance::linearize;
use ess_conformance::record::{self, Atomic, Call, Subject, Workload};
use ess_conformance::report::Status;
use ess_conformance::scenario::{ConformanceSuite, SuiteProvenance};
use ess_conformance::synthesize::{synthesize, Synthesis};
use ess_conformance::{AdmittedSuite, Runner, ScenarioStep, ScenarioValue};
use ess_domain::command::precedence::{with_phase_order, Phase};
use support_row_sets::{begin, model, number, text, READS, UNIQUE};

const UPSERT: &str = include_str!("fixtures/row-set-upsert.yaml");

fn with_commands(base: &str, commands: &str) -> EssIr {
    const VIEWS: &str = "views:\n";
    model(&base.replacen(VIEWS, &format!("{commands}{VIEWS}"), 1))
}

const SHUT: &str = "  - name: demo.jobs.Shut
    input:
      - {name: attempt_id, type: demo.jobs.AttemptId}
      - {name: delay, type: Integer}
    outcomes:
      - name: unknown-attempt
        unknown_instance: true
        error: demo.jobs.UnknownAttempt
      - name: already-closed
        wrong_state: true
        error: demo.jobs.AlreadyClosed
      - name: crowded
        when_related:
          entity: demo.jobs.Attempt
          where: {all: [worker_id == subject.worker_id, batch_id == subject.batch_id]}
          count: {gt: 1}
        error: demo.jobs.Crowded
      - name: rushed
        when: delay > 0
        emits: [demo.jobs.AttemptClosed]
        payload:
          demo.jobs.AttemptClosed: {attempt_id: input.attempt_id}
      - name: failed
        external: the upstream refuses the close
        error: demo.jobs.OverLimit
        compensates: true
        moves: demo.jobs.Attempt.close
        instance: attempt_id
      - name: closed
        moves: demo.jobs.Attempt.close
        instance: attempt_id
        emits: [demo.jobs.AttemptClosed]
        payload:
          demo.jobs.AttemptClosed: {attempt_id: input.attempt_id}
";

const CLAIM: &str = "  - name: demo.jobs.Claim
    input:
      - {name: worker_id, type: String}
      - {name: batch_id, type: String}
    outcomes:
      - name: crowded
        when_related:
          entity: demo.jobs.Attempt
          where: {all: [worker_id == input.worker_id, batch_id == input.batch_id]}
          count: {gte: 1}
        error: demo.jobs.Crowded
      - name: claimed
        when_related:
          entity: demo.jobs.Attempt
          where: {all: [worker_id == input.worker_id, batch_id == input.batch_id]}
          count: {lte: 1}
        emits: [demo.jobs.WithinLimit]
        payload:
          demo.jobs.WithinLimit: {worker_id: input.worker_id}
";

fn exchanged(a: Phase, b: Phase) -> [Phase; 8] {
    Phase::PRECEDENCE.map(|phase| {
        if phase == a {
            b
        } else if phase == b {
            a
        } else {
            phase
        }
    })
}

fn not_passed(ir: EssIr, suite: &ConformanceSuite) -> Vec<String> {
    let admitted = AdmittedSuite::from_suite(suite).unwrap_or_else(|error| panic!("{error}"));
    Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &Interpreted::for_model(ir))
        .into_report()
        .scenarios
        .into_iter()
        .filter(|result| result.status != Status::Passed)
        .map(|result| format!("{}: {:?}", result.scenario, result.status))
        .collect()
}

fn ids(result: &Synthesis, command: &str) -> Vec<String> {
    result
        .suite
        .scenarios
        .keys()
        .map(ToString::to_string)
        .filter(|id| id.starts_with(command))
        .collect()
}

/// A1: the compensating external refusal is the first branch `unknown_instance` reaches; its input
/// comes from `row_set::input_for`, which now refutes `rushed` (declared before it, accepting).
#[test]
fn a1_unknown_instance_through_a_compensating_refusal_passes_the_interpreter() {
    let ir = with_commands(READS, SHUT);
    let result = synthesize(&ir);
    let shut = ids(&result, "demo.jobs.Shut");
    let refusals: Vec<String> = result
        .refusals
        .iter()
        .map(ToString::to_string)
        .filter(|refusal| refusal.contains("demo.jobs.Shut"))
        .collect();
    eprintln!("Shut scenarios: {shut:#?}\nShut refusals: {refusals:#?}");
    let unknown = result
        .suite
        .scenarios
        .iter()
        .find(|(key, _)| key.to_string() == "demo.jobs.Shut/outcome/unknown-attempt")
        .map_or_else(
            || panic!("an unknown-attempt scenario; refusals: {refusals:#?}"),
            |(_, scenario)| scenario,
        );
    let sent: Vec<&BTreeMap<String, ScenarioValue>> = unknown
        .steps
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::ExecuteCommand { command, input, .. }
                if command.name().to_string() == "demo.jobs.Shut" =>
            {
                Some(input)
            }
            _ => None,
        })
        .collect();
    eprintln!("unknown-attempt sends: {sent:?}");
    let failed: Vec<String> = not_passed(ir, &result.suite)
        .into_iter()
        .filter(|id| id.starts_with("demo.jobs.Shut"))
        .collect();
    assert_eq!(failed, Vec::<String>::new());
}

/// A2: every row-set fixture, synthesized and interpreted under one exchanged order.
#[test]
fn a2_exchanged_suites_pass_the_interpreter_under_the_same_exchange() {
    let models: [(&str, EssIr); 4] = [
        (
            "reads+claim+shut",
            with_commands(READS, &format!("{CLAIM}{SHUT}")),
        ),
        ("unique", model(UNIQUE)),
        ("upsert", model(UPSERT)),
        ("reads", model(READS)),
    ];
    let orders = [
        (
            "present_related<->accepting",
            exchanged(Phase::PresentRelated, Phase::Accepting),
        ),
        (
            "input_refusal<->present_related",
            exchanged(Phase::InputRefusal, Phase::PresentRelated),
        ),
        (
            "existence<->present_related",
            exchanged(Phase::Existence, Phase::PresentRelated),
        ),
    ];
    let mut failures = Vec::new();
    for (order_name, order) in orders {
        for (model_name, ir) in &models {
            let failed = with_phase_order(order, || {
                let result = synthesize(ir);
                not_passed(ir.clone(), &result.suite)
            });
            for id in failed {
                failures.push(format!("{order_name} / {model_name}: {id}"));
            }
        }
    }
    assert_eq!(failures, Vec::<String>::new());
}

/// A3: one recorded attempt, then a `Claim`: under the exchange `claimed` answers first; the
/// checker, reading the same exchange, explains the history.
#[test]
fn a3_a_history_recorded_under_the_exchange_is_explained_under_it() {
    let ir = with_commands(READS, CLAIM);
    let order = exchanged(Phase::PresentRelated, Phase::Accepting);
    let record_call = Call::new(
        "demo.jobs.Record",
        BTreeMap::from([
            ("worker_id".to_owned(), text("w1")),
            ("batch_id".to_owned(), text("b1")),
            ("delay".to_owned(), number(3)),
        ]),
        Subject::Creates,
    );
    let claim_call = Call::new(
        "demo.jobs.Claim",
        BTreeMap::from([
            ("worker_id".to_owned(), text("w1")),
            ("batch_id".to_owned(), text("b1")),
        ]),
        Subject::Creates,
    );
    let workload = Workload {
        prefix: vec![record_call],
        clients: vec![vec![claim_call]],
    };
    with_phase_order(order, || {
        let target = Interpreted::for_model(ir.clone());
        begin(&target);
        let recorded = record::record(&ir, &Atomic(&target), &workload, 0).expect("recorded");
        let answered: Vec<String> = recorded
            .operations
            .iter()
            .filter(|operation| operation.command.as_str() == "demo.jobs.Claim")
            .filter_map(|operation| operation.outcome.as_ref().map(|o| o.as_str().to_owned()))
            .collect();
        assert_eq!(
            answered,
            ["claimed"],
            "under the exchange `claimed` answers first"
        );
        let bytes = serde_json::to_vec(&recorded).expect("serializes");
        let read = history::read(&bytes, &SuiteProvenance::of(&ir).spec_digest)
            .unwrap_or_else(|refusal| panic!("admitted: {refusal}"));
        let verdict = linearize::check(&ir, &read, linearize::DEFAULT_BUDGET)
            .unwrap_or_else(|refusal| panic!("checked: {refusal}"))
            .verdict;
        assert_eq!(verdict, Verdict::Linearizable);
    });
}

const GATE: &str = "  - name: demo.jobs.Gate
    input:
      - {name: worker_id, type: String}
      - {name: batch_id, type: String}
      - {name: delay, type: Integer}
    outcomes:
      - name: too-late
        when: delay > 5
        error: demo.jobs.OverLimit
      - name: crowded
        when_related:
          entity: demo.jobs.Attempt
          where: {all: [worker_id == input.worker_id, batch_id == input.batch_id]}
          count: {gte: 2}
        error: demo.jobs.Crowded
      - name: fast
        when: delay < 1
        emits: [demo.jobs.WithinLimit]
        payload:
          demo.jobs.WithinLimit: {worker_id: input.worker_id}
      - name: claimed
        when_related:
          entity: demo.jobs.Attempt
          where: {all: [worker_id == input.worker_id, batch_id == input.batch_id]}
          count: {lte: 1}
        emits: [demo.jobs.WithinLimit]
        payload:
          demo.jobs.WithinLimit: {worker_id: input.worker_id}
      - name: waited
        emits: [demo.jobs.WithinLimit]
        payload:
          demo.jobs.WithinLimit: {worker_id: input.worker_id}
";

/// A4: the row-set branches of `Gate` under exchanges that move an input-guarded phase relative
/// to the row-set phases. Only the scenarios `row_set::prepare` writes (row-set
/// branches, accepting `when:`, the default) are asserted; the rest are printed.
#[test]
fn a4_row_set_witnesses_under_input_and_default_exchanges() {
    let ir = with_commands(READS, GATE);
    let orders = [
        ("precedence", Phase::PRECEDENCE),
        (
            "input_refusal<->accepting",
            exchanged(Phase::InputRefusal, Phase::Accepting),
        ),
        (
            "input_refusal<->present_related",
            exchanged(Phase::InputRefusal, Phase::PresentRelated),
        ),
        (
            "present_related<->accepting",
            exchanged(Phase::PresentRelated, Phase::Accepting),
        ),
    ];
    let routed = ["crowded", "fast", "claimed", "waited"];
    let mut failures = Vec::new();
    for (name, order) in orders {
        let (failed, refused) = with_phase_order(order, || {
            let result = synthesize(&ir);
            let refused: Vec<String> = result
                .refusals
                .iter()
                .map(ToString::to_string)
                .filter(|refusal| refusal.contains("demo.jobs.Gate"))
                .collect();
            (not_passed(ir.clone(), &result.suite), refused)
        });
        eprintln!("{name}: not passed {failed:#?}\n{name}: refused {refused:#?}");
        for id in failed {
            if routed
                .iter()
                .any(|branch| id.starts_with(&format!("demo.jobs.Gate/outcome/{branch}:")))
            {
                failures.push(format!("{name}: {id}"));
            }
        }
    }
    assert_eq!(failures, Vec::<String>::new());
}

/// A5: with the default read before the accepting phase, `waited` (`otherwise:`) answers every
/// request the earlier phases leave, so `fast` and `claimed` have no witness. `input_for` keeps only
/// the plan's `when:` members (`filter_map(when)`) and `consistent` only those with a row set
/// (`takes` is `None` otherwise), so the `otherwise:` the plan reads first is neither refuted nor a
/// refusal, and synthesis writes witnesses the interpreter, reading the same order, answers
/// `waited`. Reached only through the `with_phase_order` test seam.
#[test]
fn a5_a_plan_member_no_search_can_refute_refuses_the_witness() {
    let ir = with_commands(READS, GATE);
    let order = exchanged(Phase::Accepting, Phase::Default);
    let failed: Vec<String> = with_phase_order(order, || {
        let result = synthesize(&ir);
        not_passed(ir.clone(), &result.suite)
    })
    .into_iter()
    .filter(|id| id.starts_with("demo.jobs.Gate/"))
    .collect();
    assert_eq!(
        failed,
        Vec::<String>::new(),
        "a witness is refused where a branch the plan reads first cannot be refuted"
    );
}
