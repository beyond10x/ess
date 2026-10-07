//! Real interpreter controls expire by matching invocation, including binding retries.
use std::{collections::BTreeMap, num::NonZeroU32};

use ess_compiler::{resolve::compile, source::SourceMap, EssIr};
use ess_conformance::{
    interpret::Interpreted, report::Status, scenario::OutcomeRef, target::*, AdmittedSuite, Runner,
};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::{ids::CorrelationId, node::Node, time::Timestamp};

const MODEL: &str = include_str!("../../../specify/ess-compiler/tests/fixtures/bounded-retry.yaml");
const RECORD: &str = "demo.ledger.Record";

fn model() -> EssIr {
    let source = MODEL.replace(
        "bindings:\n",
        "  - name: demo.ledger.Unrelated\n    input: []\n    outcomes:\n      - {name: accepted, emits: [demo.ledger.Recorded], payload: {demo.ledger.Recorded: {order_id: unrelated}}}\nbindings:\n",
    );
    let spec = Specification::assemble([(
        Source::new("repeated.yaml"),
        RawSpecFile::parse(&source).unwrap(),
    )])
    .unwrap();
    compile(&spec, &SourceMap::new()).unwrap()
}
fn context() -> ScenarioContext {
    ScenarioContext::new(
        "demo.ledger/authored/repeated".parse().unwrap(),
        CorrelationId::new("repeated").unwrap(),
    )
}
fn target() -> Interpreted {
    let target = Interpreted::for_model(model());
    target.begin_scenario(&context()).unwrap();
    target
}
fn control(command: &str, outcome: &str) -> ExternalOutcomeControl {
    ExternalOutcomeControl {
        force: OutcomeRef::new(command.parse().unwrap(), outcome.parse().unwrap()),
        correlation: context().correlation,
    }
}
fn repeat(target: &Interpreted, times: u32) {
    target
        .configure_external_outcome_repeatedly(
            control(RECORD, "unavailable"),
            NonZeroU32::new(times).unwrap(),
        )
        .unwrap();
}
fn invoke(target: &Interpreted, command: &str) -> SemanticCommandResult {
    target
        .execute_command(SemanticCommandRequest {
            command: command.parse().unwrap(),
            actor: None,
            caller: None,
            input: if command.ends_with("Unrelated") {
                BTreeMap::new()
            } else {
                BTreeMap::from([("order_id".into(), Node::Text("order-1".into()))])
            },
            correlation: context().correlation,
        })
        .unwrap()
}
fn assert_outcome(target: &Interpreted, command: &str, expected: &str) {
    let answer = invoke(target, command);
    assert_eq!(answer.outcome.unwrap().outcome.as_str(), expected);
    assert_eq!(
        answer.error.is_some(),
        matches!(expected, "unavailable" | "rejected")
    );
}

#[test]
fn exactly_n_matching_invocations_are_forced_then_normal_behavior_resumes() {
    for times in [1, 2, 3, 7] {
        let target = target();
        repeat(&target, times);
        for _ in 0..times {
            assert_outcome(&target, "demo.ledger.Unrelated", "accepted");
            assert_outcome(&target, RECORD, "unavailable");
        }
        assert_outcome(&target, RECORD, "recorded");
        assert_outcome(&target, RECORD, "recorded");
    }
}

#[test]
fn maximum_count_is_not_truncated_and_reconfiguration_replaces_the_count() {
    let target = target();
    repeat(&target, u32::MAX);
    assert_outcome(&target, RECORD, "unavailable");
    assert_outcome(&target, RECORD, "unavailable");
    repeat(&target, 2);
    assert_outcome(&target, RECORD, "unavailable");
    assert_outcome(&target, RECORD, "unavailable");
    assert_outcome(&target, RECORD, "recorded");
}

#[test]
fn begin_and_end_scenario_clear_unconsumed_repetitions() {
    let target = target();
    repeat(&target, 3);
    assert_outcome(&target, RECORD, "unavailable");
    target.begin_scenario(&context()).unwrap();
    assert_outcome(&target, RECORD, "recorded");
    repeat(&target, 3);
    target.end_scenario(&context()).unwrap();
    assert_outcome(&target, RECORD, "recorded");
}

#[test]
fn a_single_shot_replaces_repetition_and_still_expires_after_one_match() {
    let target = target();
    repeat(&target, 3);
    target
        .configure_external_outcome(control(RECORD, "rejected"))
        .unwrap();
    assert_outcome(&target, "demo.ledger.Unrelated", "accepted");
    assert_outcome(&target, RECORD, "rejected");
    assert_outcome(&target, RECORD, "recorded");
}

#[test]
fn invalid_external_controls_are_refused_without_replacing_a_valid_control() {
    let target = target();
    repeat(&target, 2);
    for (command, outcome) in [
        (RECORD, "recorded"),
        (RECORD, "unknown"),
        ("demo.ledger.Unknown", "failed"),
    ] {
        let error = target
            .configure_external_outcome_repeatedly(
                control(command, outcome),
                NonZeroU32::new(3).unwrap(),
            )
            .unwrap_err();
        assert!(
            matches!(error, TargetError::Unavailable { .. }),
            "{error:?}"
        );
        assert!(target
            .configure_external_outcome(control(command, outcome))
            .is_err());
    }
    assert_outcome(&target, RECORD, "unavailable");
    assert_outcome(&target, RECORD, "unavailable");
    assert_outcome(&target, RECORD, "recorded");
}

#[test]
fn actual_binding_retries_consume_the_same_counter_as_direct_invocations() {
    let target = target();
    repeat(&target, 3);
    assert_outcome(&target, "demo.ledger.Place", "placed");
    let invocations = target
        .observe_invocations(InvocationObservationRequest {
            binding: "notify-ledger".parse().unwrap(),
            command: RECORD.parse().unwrap(),
            correlation: context().correlation,
            deadline: Deadline::at(Timestamp::from_epoch_millis(0)),
        })
        .unwrap();
    assert_eq!(invocations.len(), 3);
    assert!(invocations
        .iter()
        .all(|invocation| invocation.input["order_id"] == Node::Text("order-1".into())));
    assert_outcome(&target, RECORD, "recorded");
}

#[test]
fn actual_synthesized_bounded_retry_and_final_failure_claims_pass() {
    let ir = model();
    let suite = ess_conformance::synthesize(&ir).suite;
    let admitted = AdmittedSuite::from_suite(&suite).unwrap();
    let report = Runner::for_suite(&suite)
        .run_admitted(&admitted, &Interpreted::for_model(ir))
        .into_report();
    for id in [
        "notify-ledger/binding/on-failure",
        "notify-ledger/binding/final-failure",
    ] {
        let scenario = report
            .scenarios
            .iter()
            .find(|scenario| scenario.scenario.to_string() == id)
            .unwrap();
        assert_eq!(scenario.status, Status::Passed, "{scenario:#?}");
        assert_ne!(scenario.checks.len(), 0);
    }
    assert!(
        report
            .scenarios
            .iter()
            .all(|scenario| scenario.status == Status::Passed),
        "{report:#?}"
    );
}
