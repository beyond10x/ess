//! A complete subject comparison around a system-issued command, in the generated Go and
//! TypeScript runners (beyond10x/ess#542, the 0.91.2 admission fix).
//!
//! For a `system: true` command's refusal synthesis writes `snapshot_complete_subject`, then
//! `expect_not_issued` (or an `await_outcome`) in the place a sent command would take, then
//! `query_view` and `expect_complete_subject_unchanged`. The Rust admission counts the awaited or
//! not-issued step as the one command the comparison brackets
//! (`crate::subject::validate_steps`). The Go runtime counted only a sent command and refused the
//! whole suite, `complete subject comparison lacks its fresh original pair`, so no scenario ran.
//!
//! The model is the downstream reproduction: a record a caller ends and a system timer whose
//! `ended` branch refuses an ended record. Each runner replays the transcript the interpreted
//! target wrote while the Rust runner asked it (`support_go::Recorder`), once as the model and
//! once with the row changed behind the refusal, which the comparison must fail.

mod support_go;
mod support_ts;

use std::cell::Cell;
use std::collections::BTreeMap;
use std::rc::Rc;

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::target::{
    AbsentInputRequest, AwaitAnswer, AwaitRequest, ConformanceTarget, ElapsedObservation,
    ElapsedObservationRequest, EntitySetupRequest, EventDeliveryRequest, EventObservationRequest,
    ExternalOutcomeControl, ImplementationIdentity, InstantMark, InvocationObservationRequest,
    ObservedEvent, ObservedInvocation, RecordedCommandCompletion, RedeliveryRequest,
    ScenarioContext, SemanticCommandRequest, SemanticCommandResult, SemanticViewRequest,
    SemanticViewResult, TargetError,
};
use ess_conformance::{AdmittedSuite, Runner};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::node::Node;

const MODEL: &str = "format: ess/33
system: repro
version: v1
domain: repro.timer
summary: A record a caller ends and a system timer refuses once it has ended.
errors:
  - name: repro.timer.AlreadyEnded
    summary: The record has ended.
    fields: []
events:
  - name: repro.timer.Opened
    fields:
      - {name: id, type: String}
  - name: repro.timer.Ended
    fields:
      - {name: id, type: String}
  - name: repro.timer.Expired
    fields:
      - {name: id, type: String}
entities:
  - name: repro.timer.Thing
    identity:
      name: id
      type: String
    fields:
      - {name: note, type: String}
    lifecycle:
      initial: Live
      states: [Live, Ended]
      terminal: [Ended]
      transitions:
        - {name: end, from: [Live], to: Ended}
actors:
  - name: repro.timer.Caller
    may: [repro.timer.Open, repro.timer.End]
  - name: repro.timer.Clock
    system: true
    may: [repro.timer.Expire]
commands:
  - name: repro.timer.Open
    input:
      - {name: id, type: String}
      - {name: note, type: String}
    outcomes:
      - name: opened
        unknown_instance: true
        creates: repro.timer.Thing
        instance: id
        sets: {note: input.note}
        emits: [repro.timer.Opened]
        payload:
          repro.timer.Opened: {id: input.id}
      - name: already-open
        preserves: repro.timer.Thing
        instance: id
  - name: repro.timer.End
    input:
      - {name: id, type: String}
    outcomes:
      - name: ended
        moves: repro.timer.Thing.end
        instance: id
        emits: [repro.timer.Ended]
        payload:
          repro.timer.Ended: {id: input.id}
  - name: repro.timer.Expire
    input:
      - {name: id, type: String}
    outcomes:
      - name: ended
        when_subject_state: Ended
        error: repro.timer.AlreadyEnded
      - name: expired
        updates: repro.timer.Thing
        instance: id
        sets: {note: expired}
        emits: [repro.timer.Expired]
        payload:
          repro.timer.Expired: {id: input.id}
views:
  - name: repro.timer.Things
    source: repro.timer.Thing
    consistency: read_your_writes
    fields:
      - {name: id, type: String}
      - {name: state, type: repro.timer.Thing.State}
      - {name: note, type: String}
";

/// The two scenarios that bracket the not-issued `Expire` with a complete subject comparison.
const BRACKETED: [&str; 2] = [
    "repro.timer.Expire/outcome/ended",
    "repro.timer.Thing/state/Ended/refuses/repro.timer.Expire",
];

fn ir() -> EssIr {
    let raw = RawSpecFile::parse(MODEL).expect("the model parses");
    let spec = Specification::assemble([(Source::new("timer.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("the model is admitted: {errors}"));
    compile(&spec, &SourceMap::new()).expect("the model compiles")
}

/// The whole suite synthesis writes for the model, which the Rust admission accepts.
fn suite() -> AdmittedSuite {
    let suite = ess_conformance::synthesize::synthesize(&ir()).suite;
    assert_eq!(suite.scenarios.len(), 8, "{:?}", suite.scenarios.keys());
    for id in BRACKETED {
        let steps: Vec<String> = suite
            .scenario(&id.parse().unwrap())
            .unwrap_or_else(|| panic!("`{id}` is synthesized"))
            .steps
            .iter()
            .map(|step| {
                serde_json::to_value(step).unwrap()["step"]
                    .as_str()
                    .unwrap()
                    .to_owned()
            })
            .collect();
        let tail: Vec<&str> = steps[steps.len() - 4..]
            .iter()
            .map(String::as_str)
            .collect();
        assert_eq!(
            tail,
            [
                "snapshot_complete_subject",
                "expect_not_issued",
                "query_view",
                "expect_complete_subject_unchanged"
            ],
            "{id}"
        );
    }
    AdmittedSuite::from_suite(&suite)
        .unwrap_or_else(|error| panic!("the synthesized suite is admitted: {error}"))
}

/// The interpreted model; with `drift`, every row read after `Expire` was not issued carries a
/// changed `note`, which only the complete subject comparison reads.
struct Timer {
    inner: Interpreted,
    drift: bool,
    refused: Cell<bool>,
    told: Rc<Cell<bool>>,
}

impl Timer {
    fn new(drift: bool) -> Self {
        Self {
            inner: Interpreted::for_model(ir()),
            drift,
            refused: Cell::new(false),
            told: Rc::default(),
        }
    }

    fn drifted(&self, mut result: SemanticViewResult) -> SemanticViewResult {
        if self.drift && self.refused.get() {
            for row in &mut result.rows {
                row.insert("note".to_owned(), Node::Text("drifted".to_owned()));
                self.told.set(true);
            }
        }
        result
    }
}

impl ConformanceTarget for Timer {
    fn establish_entity(&self, request: EntitySetupRequest) -> Result<(), TargetError> {
        self.inner.establish_entity(request)
    }
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        self.inner.identity()
    }
    fn begin_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.refused.set(false);
        self.inner.begin_scenario(scenario)
    }
    fn deliver_event(&self, request: EventDeliveryRequest) -> Result<(), TargetError> {
        self.inner.deliver_event(request)
    }
    fn mark_instant(&self, request: InstantMark) -> Result<(), TargetError> {
        self.inner.mark_instant(request)
    }
    fn observe_elapsed(
        &self,
        request: ElapsedObservationRequest,
    ) -> Result<ElapsedObservation, TargetError> {
        self.inner.observe_elapsed(request)
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        self.inner.execute_command(request)
    }
    fn execute_command_recorded(
        &self,
        request: SemanticCommandRequest,
    ) -> RecordedCommandCompletion {
        self.inner.execute_command_recorded(request)
    }
    fn execute_command_without_input(
        &self,
        request: AbsentInputRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        self.inner.execute_command_without_input(request)
    }
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        self.inner
            .query_view(request)
            .map(|result| self.drifted(result))
    }
    fn query_view_as(
        &self,
        request: SemanticViewRequest,
        reader: &ess_conformance::scenario::ActorRef,
    ) -> Result<SemanticViewResult, TargetError> {
        self.inner
            .query_view_as(request, reader)
            .map(|result| self.drifted(result))
    }
    fn query_view_anonymous(
        &self,
        request: SemanticViewRequest,
    ) -> Result<SemanticViewResult, TargetError> {
        self.inner
            .query_view_anonymous(request)
            .map(|result| self.drifted(result))
    }
    fn observe_events(
        &self,
        request: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        self.inner.observe_events(request)
    }
    fn configure_external_outcome(
        &self,
        request: ExternalOutcomeControl,
    ) -> Result<(), TargetError> {
        self.inner.configure_external_outcome(request)
    }
    fn configure_external_outcome_repeatedly(
        &self,
        request: ExternalOutcomeControl,
        times: std::num::NonZeroU32,
    ) -> Result<(), TargetError> {
        self.inner
            .configure_external_outcome_repeatedly(request, times)
    }
    fn redeliver_event(&self, request: RedeliveryRequest) -> Result<(), TargetError> {
        self.inner.redeliver_event(request)
    }
    fn observe_invocations(
        &self,
        request: InvocationObservationRequest,
    ) -> Result<Vec<ObservedInvocation>, TargetError> {
        self.inner.observe_invocations(request)
    }
    fn await_outcome(&self, request: AwaitRequest) -> Result<AwaitAnswer, TargetError> {
        let answer = self.inner.await_outcome(request)?;
        if matches!(answer, AwaitAnswer::NotIssued { .. }) {
            self.refused.set(true);
        }
        Ok(answer)
    }
    fn end_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.inner.end_scenario(scenario)
    }
}

fn written(status: Status) -> String {
    match status {
        Status::Passed => "passed",
        Status::Failed => "failed",
        Status::Error => "error",
        Status::Unsupported => "unsupported",
    }
    .to_owned()
}

/// The Rust runner's verdicts against `target`, and the transcript of what it answered, checked
/// against what the case expects: every scenario passes, or with `drift` exactly the bracketed
/// ones fail.
fn reference(
    drift: bool,
) -> (
    AdmittedSuite,
    BTreeMap<String, String>,
    support_go::Recorder<Timer>,
) {
    let suite = suite();
    let target = Timer::new(drift);
    let told = Rc::clone(&target.told);
    let recorder = support_go::Recorder::new(target);
    let verdicts: BTreeMap<String, String> = Runner::for_suite(suite.suite())
        .run_admitted(&suite, &recorder)
        .into_report()
        .scenarios
        .into_iter()
        .map(|result| (result.scenario.to_string(), written(result.status)))
        .collect();
    assert_eq!(verdicts.len(), 8, "{verdicts:?}");
    assert_eq!(told.get(), drift, "the drift applies exactly when asked");
    for (id, status) in &verdicts {
        let expected = if drift && BRACKETED.contains(&id.as_str()) {
            "failed"
        } else {
            "passed"
        };
        assert_eq!(status, expected, "drift {drift}: {id}: {verdicts:?}");
    }
    (suite, verdicts, recorder)
}

/// The Go runner admits the whole suite and gives every scenario the Rust runner's verdict, as the
/// model and with the row changed behind the refusal.
#[test]
fn the_go_runner_admits_and_judges_a_complete_comparison_around_a_not_issued_command() {
    let directory = support_go::package(
        "complete-not-issued",
        suite().suite(),
        &[support_go::TRANSCRIPT_TARGET],
    );
    for drift in [false, true] {
        let (_, verdicts, recorder) = reference(drift);
        let replayed = support_go::replay(&directory, &recorder, &[]);
        assert!(
            !replayed.go.outcomes.is_empty(),
            "drift {drift}: the Go run published no verdicts:\n{}",
            replayed.go.log
        );
        assert!(
            replayed.divergences.is_empty(),
            "drift {drift}: the Go runtime sent requests the reference runner did not:\n{}\n{}",
            replayed.divergences.join("\n"),
            replayed.go.log
        );
        assert_eq!(
            replayed.go.outcomes, verdicts,
            "drift {drift}\n{}",
            replayed.go.log
        );
    }
}

/// The same in the TypeScript runner.
#[test]
fn the_typescript_runner_admits_and_judges_a_complete_comparison_around_a_not_issued_command() {
    let package = support_ts::Package::new("complete-not-issued", suite().suite());
    for drift in [false, true] {
        let (suite, verdicts, recorder) = reference(drift);
        let replayed = package.replay(&suite, &recorder.transcript(), false);
        assert_ne!(replayed.exit, Some(2), "drift {drift}: {}", replayed.log);
        assert!(
            replayed.divergences.is_empty(),
            "drift {drift}: the TypeScript runtime sent requests the reference runner did not:\n{}\n{}",
            replayed.divergences.join("\n"),
            replayed.log
        );
        assert_eq!(
            replayed.outcomes, verdicts,
            "drift {drift}\n{}",
            replayed.log
        );
    }
}
