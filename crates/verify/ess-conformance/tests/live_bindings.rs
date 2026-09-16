//! The same actual-response and nested-event counterexamples as the generated Go evaluator.
use ess_conformance::target::{
    ConformanceTarget, EventObservationRequest, ExternalOutcomeControl, ImplementationIdentity,
    ObservedEvent, RedeliveryRequest, ScenarioContext, SemanticCommandRequest,
    SemanticCommandResult, SemanticViewRequest, SemanticViewResult, TargetError,
};
use ess_conformance::{
    AdmittedSuite, AdvancingClock, CountReport, CountStatus, Ids, Runner, RunnerConfig,
};
use ess_primitives::node::Node;
use std::cell::Cell;

const INPUT: &str = include_str!("fixtures/live-binding-suite.json");

struct Target {
    mode: &'static str,
    ended: Cell<usize>,
    observed: Cell<usize>,
}

fn event(call: &str, agent: &str) -> ObservedEvent {
    let payload: Node = serde_json::from_value(serde_json::json!({
        "id": call, "agent": {"id": agent}, "state": "bridged"
    }))
    .unwrap();
    ObservedEvent::new("fixture.routing.CallStateChanged".parse().unwrap()).with("item", payload)
}

impl ConformanceTarget for Target {
    fn open_observations(
        &self,
        _: std::collections::BTreeSet<ess_conformance::scenario::EventRef>,
    ) -> Result<String, TargetError> {
        Ok("subscribed".into())
    }
    fn observe_occurrences(
        &self,
        request: ess_conformance::live_trace::Request,
    ) -> Result<ess_conformance::temporal::Batch, TargetError> {
        use ess_conformance::temporal::{Batch, Occurrence};
        if self.mode == "gap" {
            return Err(TargetError::unavailable("observe", "disconnected"));
        }
        let mut rows = vec![
            (
                100,
                "Agent",
                serde_json::json!({"id":"owned", "state":"wrap-up"}),
            ),
            (
                120,
                "Agent",
                serde_json::json!({"id":"owned", "state":"available"}),
            ),
            (
                121,
                "Agent",
                serde_json::json!({"id":"owned", "state":"selected"}),
            ),
            (
                200,
                "Metrics",
                serde_json::json!({"account":"owned", "waiting":3}),
            ),
            (
                205,
                "Metrics",
                serde_json::json!({"account":"owned", "waiting":3}),
            ),
            (
                209,
                "Metrics",
                serde_json::json!({"account":"owned", "waiting":3}),
            ),
        ];
        if self.mode == "early-offer" {
            rows[2].0 = 119;
            rows.swap(1, 2);
        }
        if self.mode == "inverted" {
            rows[1].0 = 99;
            rows.swap(0, 1);
        }
        if self.mode == "transient-zero" {
            rows[4].2["waiting"] = 0.into();
        }
        let occurrences = if request.after == 0 {
            rows.into_iter()
                .enumerate()
                .map(|(i, (at_ms, event, payload))| Occurrence {
                    sequence: i as u64 + 1,
                    at_ms,
                    event: format!("fixture.routing.{event}").parse().unwrap(),
                    payload: serde_json::from_value(payload).unwrap(),
                })
                .collect()
        } else {
            vec![]
        };
        Ok(Batch {
            lifetime: request.lifetime,
            after: request.after,
            complete_before_ms: if self.mode == "truncated" { 209 } else { 211 },
            occurrences,
        })
    }
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("fixture", "1"))
    }
    fn begin_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        Ok(())
    }
    fn end_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        self.ended.set(self.ended.get() + 1);
        Ok(())
    }
    fn execute_command(
        &self,
        _: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        let mut result = SemanticCommandResult::undeclared();
        if self.mode != "missing-response" {
            result.response = Some(
                [
                    (
                        "agent_id".into(),
                        if self.mode == "malformed-response" {
                            serde_json::from_str("7").unwrap()
                        } else {
                            Node::Text("real-agent".into())
                        },
                    ),
                    ("call_id".into(), Node::Text("real-call".into())),
                ]
                .into(),
            );
        }
        if self.mode == "stale-direct-event" {
            result.direct_events.push(event("real-call", "real-agent"));
        }
        Ok(result)
    }
    fn observe_events(
        &self,
        _: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        self.observed.set(self.observed.get() + 1);
        match self.mode {
            "wrong-call" | "stale-direct-event" => Ok(vec![event("unrelated", "real-agent")]),
            "wrong-agent" => Ok(vec![event("real-call", "unrelated")]),
            "gap" => Err(TargetError::unavailable("observe", "source disconnected")),
            _ => Ok(vec![
                event("unrelated", "real-agent"),
                event("real-call", "unrelated"),
                event("real-call", "real-agent"),
            ]),
        }
    }
    fn query_view(&self, _: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        unreachable!()
    }
    fn configure_external_outcome(&self, _: ExternalOutcomeControl) -> Result<(), TargetError> {
        unreachable!()
    }
    fn redeliver_event(&self, _: RedeliveryRequest) -> Result<(), TargetError> {
        unreachable!()
    }
}

#[test]
fn native_rust_executes_complete_windows_and_refuses_temporal_counterexamples() {
    let suite = AdmittedSuite::from_json(include_str!("fixtures/live-trace-suite.json")).unwrap();
    for mode in [
        "pass",
        "early-offer",
        "inverted",
        "transient-zero",
        "truncated",
        "gap",
    ] {
        let target = Target {
            mode,
            ended: Cell::new(0),
            observed: Cell::new(0),
        };
        let run = Runner::new(
            RunnerConfig::new(100),
            AdvancingClock::new(0, 10),
            Ids::for_suite(suite.suite()),
        )
        .with_failed_setup_cleanup()
        .run_admitted(&suite, &target);
        let report = CountReport::from_run(&run, &suite).unwrap();
        assert_eq!(
            report.execution_status() == CountStatus::Passed,
            mode == "pass",
            "{mode}: {report:?}"
        );
        assert_eq!(target.ended.get(), 1);
    }
}

#[test]
fn actual_responses_and_nested_event_paths_agree_with_go() {
    let suite = AdmittedSuite::from_json(INPUT).unwrap();
    for mode in [
        "pass",
        "wrong-call",
        "wrong-agent",
        "missing-response",
        "malformed-response",
        "stale-direct-event",
        "gap",
    ] {
        let target = Target {
            mode,
            ended: Cell::new(0),
            observed: Cell::new(0),
        };
        let run = Runner::new(
            RunnerConfig::new(100),
            AdvancingClock::new(0, 10),
            Ids::for_suite(suite.suite()),
        )
        .with_failed_setup_cleanup()
        .run_admitted(&suite, &target);
        let report = CountReport::from_run(&run, &suite).unwrap();
        assert_eq!(
            report.execution_status() == CountStatus::Passed,
            mode == "pass",
            "{mode}: {report:?}"
        );
        assert_eq!(target.ended.get(), 1, "{mode}");
        if mode.contains("response") {
            assert_eq!(target.observed.get(), 0, "{mode}");
        }
    }
}

#[test]
fn unknown_bindings_and_downgraded_vocabulary_are_refused() {
    let trace = include_str!("fixtures/live-trace-suite.json");
    assert!(
        AdmittedSuite::from_json(&trace.replace("ess-conformance/10", "ess-conformance/8"))
            .is_err()
    );
    for changed in [
        INPUT.replace("ess-conformance/10", "ess-conformance/9"),
        INPUT.replacen("\"instance\": \"call\"}", "\"instance\": \"missing\"}", 1),
        INPUT.replacen("\"field\": \"call_id\"", "\"field\": \"missing\"", 1),
        INPUT.replacen(
            "\"instance\": \"call\", \"field\"",
            "\"instance\": \"agent\", \"field\"",
            1,
        ),
        INPUT.replacen(
            "\"command\": \"fixture.routing.Agent\", \"instance\"",
            "\"command\": \"fixture.routing.Other\", \"instance\"",
            1,
        ),
    ] {
        assert!(AdmittedSuite::from_json(&changed).is_err(), "{changed}");
    }
}
