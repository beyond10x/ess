//! The generated Go runtime resolves `now_offset` values (suite/26, beyond10x/ess#171) from its
//! wall clock exactly as the reference runner does and gives the reference verdicts
//! (beyond10x/ess#188).
//!
//! Both runners are handed one wall-clock instant, a quarter second past a whole second, so each
//! must round it up to the same whole second and send the same RFC 3339 text: the replay compares
//! every request, and a runtime that resolved `now - 61s` from another instant, or did not round,
//! sends another `starts_at`.

mod support_go;

use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;

use ess_compiler::{
    ir::EssIr,
    refs::{CommandRef, OutcomeRef},
    resolve::compile,
    source::SourceMap,
};
use ess_conformance::{synthesize::synthesize, target::*, ConformanceSuite};
use ess_domain::{
    command::OutcomeName,
    spec::{RawSpecFile, Specification},
    system::Source,
};
use ess_primitives::{node::Node, time::Rfc3339Instant};

const JOBS: &str = include_str!("fixtures/current-time-guard.yaml");
const REFUSAL: &str = "demo.jobs.ScheduleJob/outcome/start-in-past";
const SCHEDULED: &str = "demo.jobs.ScheduleJob/outcome/scheduled";

/// 2026-09-27T12:00:00.250Z, a quarter second past a whole second.
const WALL_MS: u64 = 1_790_510_400_250;

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("jobs.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap()
}

fn suite() -> ConformanceSuite {
    let synthesis = synthesize(&ir(JOBS));
    assert!(synthesis.refusals.is_empty(), "{:#?}", synthesis.refusals);
    assert!(ess_conformance::now_offset::used_by(&synthesis.suite));
    synthesis.suite
}

type Row = BTreeMap<String, Node>;

/// `tests/current_time_guard.rs`'s service: it refuses a start more than `tolerance` seconds before
/// its own clock, which reads `latency_ms` after the runner's wall.
struct Jobs {
    latency_ms: u64,
    tolerance: i64,
    rows: RefCell<Vec<Row>>,
    minted: Cell<u64>,
}

impl Jobs {
    fn new(tolerance: i64, latency_ms: u64) -> Self {
        Self {
            latency_ms,
            tolerance,
            rows: RefCell::default(),
            minted: Cell::new(0),
        }
    }

    fn now(&self) -> Rfc3339Instant {
        Rfc3339Instant::from_epoch_millis(i64::try_from(WALL_MS + self.latency_ms).unwrap())
            .unwrap()
    }
}

fn outcome(command: &CommandRef, name: &str) -> OutcomeRef {
    OutcomeRef::new(command.clone(), OutcomeName::new(name).unwrap())
}

impl ConformanceTarget for Jobs {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("jobs-fixture", "1"))
    }
    fn begin_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        self.rows.replace(Vec::new());
        Ok(())
    }
    fn end_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        Ok(())
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        self.minted.set(self.minted.get() + 1);
        let token = ess_primitives::consistency::ConsistencyToken::new(format!(
            "seq:{}",
            self.minted.get()
        ))
        .unwrap();
        let command = request.command.clone();
        if command.to_string() != "demo.jobs.ScheduleJob" {
            return Err(TargetError::unsupported("command", command.to_string()));
        }
        let Some(Node::Text(text)) = request.input.get("starts_at") else {
            return Err(TargetError::unsupported("input", "starts_at is not text"));
        };
        let starts_at = Rfc3339Instant::parse_rfc3339(text)
            .ok_or_else(|| TargetError::unsupported("input", format!("{text} is no instant")))?;
        let limit = self.now().plus_seconds(-self.tolerance).unwrap();
        let result = if starts_at < limit {
            SemanticCommandResult::took(outcome(&command, "start-in-past")).with_error(
                DeclaredErrorValue::new("demo.jobs.StartInPast".parse().unwrap()),
            )
        } else {
            let id = Node::Text(format!("00000000-0000-4000-8000-{:012}", self.minted.get()));
            self.rows.borrow_mut().push(Row::from([
                ("job_id".to_owned(), id.clone()),
                ("starts_at".to_owned(), Node::Text(text.clone())),
            ]));
            SemanticCommandResult::took(outcome(&command, "scheduled")).emitting(
                ObservedEvent::new("demo.jobs.JobScheduled".parse().unwrap())
                    .with("job_id", id)
                    .with("starts_at", Node::Text(text.clone())),
            )
        };
        Ok(result.with_consistency(token))
    }
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        match request.view.to_string().as_str() {
            "demo.jobs.JobDetails" => Ok(SemanticViewResult::of(self.rows.borrow().clone())),
            other => Err(TargetError::unsupported("view", other)),
        }
    }
    fn observe_events(
        &self,
        _: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        Ok(Vec::new())
    }
    fn configure_external_outcome(&self, _: ExternalOutcomeControl) -> Result<(), TargetError> {
        panic!("nothing here is externally decided")
    }
    fn redeliver_event(&self, _: RedeliveryRequest) -> Result<(), TargetError> {
        panic!("no bindings")
    }
}

#[test]
fn go_resolves_now_offsets_as_the_reference_runner_does_and_gives_its_verdicts() {
    let suite = suite();
    let options = support_go::Options {
        wall_millis: Some(WALL_MS),
        ..support_go::Options::default()
    };
    for (tolerance, latency_ms, expected) in [
        (60, 0, vec![]),
        (60, 999, vec![]),
        (30, 0, vec![SCHEDULED]),
        (120, 0, vec![REFUSAL]),
    ] {
        let verdicts = support_go::assert_parity_with(
            &format!("now-{tolerance}-{latency_ms}"),
            &suite,
            Jobs::new(tolerance, latency_ms),
            &options,
        );
        assert_eq!(
            support_go::not_passed(&verdicts),
            expected,
            "{tolerance}s/{latency_ms}ms"
        );
    }
}
