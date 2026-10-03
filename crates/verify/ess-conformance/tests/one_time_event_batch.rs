//! Ordinary event callbacks obey the same aggregate payload bound before polling or deduplication.
#[allow(dead_code)]
mod support_one_time;
use ess_conformance::{report::Status, target::*, AdmittedSuite, Runner, ScenarioStep};
use ess_primitives::node::Node;
use std::{cell::Cell, collections::BTreeMap};
use support_one_time::{Mode, Service};
struct Batched {
    inner: Service,
    observations: Cell<usize>,
}
impl ConformanceTarget for Batched {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        self.inner.identity()
    }
    fn begin_scenario(&self, s: &ScenarioContext) -> Result<(), TargetError> {
        self.inner.begin_scenario(s)
    }
    fn end_scenario(&self, s: &ScenarioContext) -> Result<(), TargetError> {
        self.inner.end_scenario(s)
    }
    fn execute_command(
        &self,
        r: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        self.inner.execute_command(r)
    }
    fn query_view(&self, r: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        self.inner.query_view(r)
    }
    fn configure_external_outcome(&self, r: ExternalOutcomeControl) -> Result<(), TargetError> {
        self.inner.configure_external_outcome(r)
    }
    fn redeliver_event(&self, r: RedeliveryRequest) -> Result<(), TargetError> {
        self.inner.redeliver_event(r)
    }
    fn observe_invocations(
        &self,
        r: InvocationObservationRequest,
    ) -> Result<Vec<ObservedInvocation>, TargetError> {
        self.inner.observe_invocations(r)
    }
    fn observe_events(
        &self,
        r: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        self.observations.set(self.observations.get() + 1);
        if self.observations.get() != 2 {
            return Ok(Vec::new());
        }
        let mut event = ObservedEvent::new(r.event).in_activity(r.correlation);
        event.payload = BTreeMap::from([("data".into(), Node::Text("x".repeat(600_000)))]);
        Ok(vec![event.clone(), event])
    }
}
#[test]
fn oversized_ordinary_event_batch_stops_before_another_poll() {
    let mut suite = support_one_time::admitted(Mode::WindowHealthy)
        .suite()
        .clone();
    let scenario = suite.scenarios.values_mut().next().unwrap();
    let policy = scenario.one_time_response.as_mut().unwrap();
    for event in &mut policy.events {
        event.within_ms = 0;
    }
    policy.event_windows.retain(|window| window.within_ms == 0);
    let event = policy.events[0].event.clone();
    scenario
        .steps
        .retain(|step| !matches!(step, ScenarioStep::ExpectNoEvent { .. }));
    scenario.steps.push(ScenarioStep::EventuallyEvent {
        event,
        payload: BTreeMap::from([("wanted".into(), Node::Text("yes".into()))]),
        shape: ess_conformance::scenario::PayloadShape::default(),
    });
    let input = AdmittedSuite::from_suite(&suite).unwrap();
    let target = Batched {
        inner: Service::new(Mode::Healthy),
        observations: Cell::new(0),
    };
    let report = Runner::for_suite(input.suite()).run_admitted(&input, &target);
    assert_eq!(report.scenarios[0].status, Status::Unsupported);
    assert_eq!(
        target.observations.get(),
        2,
        "one independent scan then one oversized ordinary batch; no polling after resource refusal"
    );
    let bytes = serde_json::to_string(&report.scenarios).unwrap();
    for secret in target.inner.returned_plaintexts() {
        assert!(!bytes.contains(&secret));
    }
}
