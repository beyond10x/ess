//! A held `now` (suite/50, beyond10x/ess#586) for the runtime parity tests: the job service of
//! `tests/current_time_guard.rs`, whose clock trails the runner's wall by 900 ms, and that suite
//! with a `hold_now 1` before its accepting scenario's command, so `starts_at` lies exactly on the
//! boundary `now - 60s`.

use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;

use ess_compiler::{
    refs::{CommandRef, OutcomeRef},
    resolve::compile,
    source::SourceMap,
};
use ess_conformance::{
    synthesize::synthesize, target::*, ConformanceSuite, ScenarioStep, ScenarioValue,
};
use ess_domain::{
    command::OutcomeName,
    spec::{RawSpecFile, Specification},
    system::Source,
};
use ess_primitives::{node::Node, time::Rfc3339Instant};

const JOBS: &str = include_str!("../fixtures/current-time-guard.yaml");
pub const REFUSAL: &str = "demo.jobs.ScheduleJob/outcome/start-in-past";
pub const SCHEDULED: &str = "demo.jobs.ScheduleJob/outcome/scheduled";

/// The jobs suite with `hold_now 1` before the accepting scenario's `now - 59s` command.
pub fn held_suite() -> ConformanceSuite {
    let raw = RawSpecFile::parse(JOBS).unwrap_or_else(|error| panic!("{error}"));
    let spec = Specification::assemble([(Source::new("jobs.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    let synthesis = synthesize(&compile(&spec, &SourceMap::new()).unwrap());
    assert!(synthesis.refusals.is_empty(), "{:#?}", synthesis.refusals);
    let mut suite = synthesis.suite;
    let (_, scenario) = suite
        .scenarios
        .iter_mut()
        .find(|(id, _)| id.to_string() == SCHEDULED)
        .expect("the accepting scenario");
    let at = scenario
        .steps
        .iter()
        .position(|step| {
            matches!(step, ScenarioStep::ExecuteCommand { input, .. }
                if input.get("starts_at") == Some(&ScenarioValue::NowOffset { seconds: -59 }))
        })
        .expect("its command sent a second inside the boundary");
    scenario
        .steps
        .insert(at, ScenarioStep::HoldNow { seconds: 1 });
    suite.select_fresh_format();
    suite
}

type Row = BTreeMap<String, Node>;

/// A job service that refuses a start more than 60 seconds before its `now`: the held instant
/// when one is held, else its own clock, 900 ms after `wall_ms`.
pub struct Jobs {
    wall_ms: u64,
    /// Whether it offers `hold_now`; without it the trait's answer is `unsupported`.
    pub holding: bool,
    /// Whether it refuses a start exactly 60 seconds old too: the `<=` mutant of the guard.
    pub inclusive: bool,
    held: Cell<Option<Rfc3339Instant>>,
    rows: RefCell<Vec<Row>>,
    minted: Cell<u64>,
}

impl Jobs {
    pub fn new(wall_ms: u64) -> Self {
        Self {
            wall_ms,
            holding: true,
            inclusive: false,
            held: Cell::new(None),
            rows: RefCell::default(),
            minted: Cell::new(0),
        }
    }

    fn now(&self) -> Rfc3339Instant {
        Rfc3339Instant::from_epoch_millis(i64::try_from(self.wall_ms + 900).unwrap()).unwrap()
    }
}

fn outcome(command: &CommandRef, name: &str) -> OutcomeRef {
    OutcomeRef::new(command.clone(), OutcomeName::new(name).unwrap())
}

impl ConformanceTarget for Jobs {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("held-jobs-fixture", "1"))
    }
    fn begin_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        self.rows.replace(Vec::new());
        self.held.set(None);
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
        let now = self.held.take().unwrap_or_else(|| self.now());
        let limit = now.plus_seconds(-60).unwrap();
        let refused = if self.inclusive {
            starts_at <= limit
        } else {
            starts_at < limit
        };
        let result = if refused {
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
    fn hold_now(&self, request: NowHold) -> Result<(), TargetError> {
        if !self.holding {
            return Err(TargetError::unsupported(
                "hold_now",
                "this service reads its own clock",
            ));
        }
        self.held.set(Some(request.instant));
        Ok(())
    }
}
