//! Shared source and occurrence batches exercise both native evaluator implementations.
#[path = "support/live_metrics.rs"]
mod fixture;

use ess_conformance::target::*;
use ess_conformance::temporal::{Batch, Occurrence};
use ess_conformance::{
    AdmittedSuite, AdvancingClock, CountReport, CountStatus, Ids, Runner, RunnerConfig,
};
use serde_json::Value;
use std::cell::Cell;

struct Target<'a> {
    case: &'a Value,
    next: Cell<usize>,
    ended: Cell<usize>,
}
impl ConformanceTarget for Target<'_> {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("fixture", "1"))
    }
    fn begin_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        Ok(())
    }
    fn end_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        self.ended.set(self.ended.get() + 1);
        if self.case["cleanup_failure"] == true {
            Err(TargetError::unavailable("cleanup", "failed"))
        } else {
            Ok(())
        }
    }
    fn open_observations(
        &self,
        _: std::collections::BTreeSet<ess_conformance::scenario::EventRef>,
    ) -> Result<String, TargetError> {
        Ok("subscribed".into())
    }
    fn observe_occurrences(
        &self,
        request: ess_conformance::live_trace::Request,
    ) -> Result<Batch, TargetError> {
        let index = self.next.get();
        self.next.set(index + 1);
        if self.case["error_at"].as_u64() == Some(index as u64) {
            return Err(TargetError::unavailable("observe", "cancelled"));
        }
        let batches = self.case["batches"].as_array().unwrap();
        let Some(batch) = batches.get(index) else {
            return Ok(Batch {
                lifetime: request.lifetime,
                after: request.after,
                complete_before_ms: batches.last().unwrap()["complete_before_ms"]
                    .as_u64()
                    .unwrap(),
                occurrences: vec![],
            });
        };
        Ok(Batch {
            lifetime: request.lifetime,
            after: batch["after"].as_u64().unwrap(),
            complete_before_ms: batch["complete_before_ms"].as_u64().unwrap(),
            occurrences: batch["occurrences"]
                .as_array()
                .unwrap()
                .iter()
                .map(|row| Occurrence {
                    sequence: row["sequence"].as_u64().unwrap(),
                    at_ms: row["at_ms"].as_u64().unwrap(),
                    event: row["event"].as_str().unwrap().parse().unwrap(),
                    payload: serde_json::from_value(row["payload"].clone()).unwrap(),
                })
                .collect(),
        })
    }
    fn execute_command(
        &self,
        _: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        unreachable!()
    }
    fn observe_events(
        &self,
        _: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        unreachable!()
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
fn rust_observes_first_status_fresh_quiet_baselines_and_every_stable_frame() {
    let compilation = fixture::compilation(fixture::SOURCE).unwrap();
    let suite = compilation.input.selected();
    assert_eq!(suite.suite().provenance.suite_version.major(), 13);
    for case in fixture::cases().as_array().unwrap() {
        let target = Target {
            case,
            next: Cell::new(0),
            ended: Cell::new(0),
        };
        let run = Runner::new(
            RunnerConfig::new(100),
            AdvancingClock::new(0, 1),
            Ids::for_suite(suite.suite()),
        )
        .with_failed_setup_cleanup()
        .run_admitted(suite, &target);
        let report = CountReport::from_run(&run, suite).unwrap();
        assert_eq!(
            report.execution_status() == CountStatus::Passed,
            case["pass"].as_bool().unwrap(),
            "{}: {report:?}",
            case["name"]
        );
        assert_eq!(target.ended.get(), 1);
    }
}

#[test]
fn captures_are_typed_unique_present_and_versioned_before_execution() {
    let valid = fixture::compilation(fixture::SOURCE).unwrap();
    let again = fixture::compilation(fixture::SOURCE).unwrap();
    assert_eq!(valid.input.document(), again.input.document());
    for source in [
        fixture::SOURCE.replace("ess-scenario/4","ess-scenario/3"),
        fixture::SOURCE.replace("capture: baseline, plus: 3","capture: future, plus: 3"),
        fixture::SOURCE.replace("min: 0, max: 10000","min: 10001, max: 10000"),
        fixture::SOURCE.replace("anchor: first-live","anchor: first-live\n    capture:\n      baseline: {path: catchup_ms}"),
        fixture::SOURCE.replace("anchor: first-live","anchor: first-live\n    capture:\n      baseline: {path: catchup_ms, require_present: true}"),
        fixture::SOURCE.replace("anchor: first-live","anchor: first-live\n    capture:\n      word: {path: status}").replace("capture: baseline, plus: 3","capture: word, plus: 3"),
    ] {
        assert!(fixture::compilation(&source).is_err(),"unexpected admission: {source}");
    }
    let original = valid.input.selected().original_json();
    assert!(AdmittedSuite::from_json(
        &original.replace("ess-conformance/13", "ess-conformance/11")
    )
    .is_err());
    let mut ordinary: Value = serde_json::from_str(original).unwrap();
    ordinary["provenance"]["suite_version"] = "ess-conformance/12".into();
    ordinary.as_object_mut().unwrap().remove("coverage");
    assert!(AdmittedSuite::from_json(&ordinary.to_string()).is_ok());
    ordinary["provenance"]["suite_version"] = "ess-conformance/10".into();
    assert!(AdmittedSuite::from_json(&ordinary.to_string()).is_err());
}
