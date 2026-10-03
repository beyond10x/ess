//! A `Timestamp` guard compared with the current time (beyond10x/ess#171, `ess/16`,
//! `docs/design/current-time-guards.md`).
//!
//! `starts_at < now - 60s` names no instant a suite can carry: the moment it is compared with is the
//! one the implementation handles the request at. So synthesis decides its witnesses against a fixed
//! reference instant, and writes each one into the suite as a `now_offset` — seconds from the moment
//! the runner sends the command — which the runner resolves from its wall clock when it sends it.
//! The target is told nothing new. Both sides are witnessed a second from the boundary, never at it:
//! `now - 61s` requires the refusal and `now - 59s` the accepting branch.
mod support_versions;

use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, BTreeSet};

use ess_compiler::{
    ir::EssIr,
    refs::{CommandRef, OutcomeRef},
    resolve::compile,
    source::SourceMap,
};
use ess_conformance::{
    now_offset,
    report::{ConformanceStatus, Status},
    scenario::ScenarioInitialState,
    synthesize::synthesize,
    target::*,
    AdmittedSuite, AdvancingClock, Clock, ConformanceSuite, Ids, Runner, RunnerConfig,
    ScenarioStep, ScenarioValue, ViewExpectation,
};
use ess_domain::{
    command::OutcomeName,
    spec::{RawSpecFile, Specification},
    system::Source,
};
use ess_primitives::{
    node::Node,
    time::{Rfc3339Instant, Timestamp},
};

const JOBS: &str = include_str!("fixtures/current-time-guard.yaml");
const REFUSAL: &str = "demo.jobs.ScheduleJob/outcome/start-in-past";
const SCHEDULED: &str = "demo.jobs.ScheduleJob/outcome/scheduled";

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("jobs.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap()
}

fn suite_of(text: &str) -> ConformanceSuite {
    let synthesis = synthesize(&ir(text));
    assert!(
        synthesis.refusals.is_empty(),
        "every scenario is synthesized: {:#?}",
        synthesis.refusals
    );
    synthesis.suite
}

fn suite() -> ConformanceSuite {
    suite_of(JOBS)
}

fn scenario<'s>(suite: &'s ConformanceSuite, id: &str) -> &'s [ScenarioStep] {
    &suite
        .scenarios
        .iter()
        .find(|(key, _)| key.to_string() == id)
        .unwrap_or_else(|| {
            panic!(
                "no scenario {id}: {:?}",
                suite.scenarios.keys().collect::<Vec<_>>()
            )
        })
        .1
        .steps
}

/// Every `ScheduleJob` invocation of one scenario: the value sent for `starts_at`, and the branch
/// required of it.
fn invocations(suite: &ConformanceSuite, id: &str) -> Vec<(ScenarioValue, String)> {
    let mut out = Vec::new();
    let mut steps = scenario(suite, id).iter().peekable();
    while let Some(step) = steps.next() {
        let ScenarioStep::ExecuteCommand { input, .. } = step else {
            continue;
        };
        let Some(ScenarioStep::ExpectOutcome { outcome }) = steps.peek() else {
            continue;
        };
        out.push((input["starts_at"].clone(), outcome.outcome.to_string()));
    }
    out
}

fn offset(seconds: i64) -> ScenarioValue {
    ScenarioValue::NowOffset { seconds }
}

#[test]
fn issue_171_the_refusal_is_sent_sixty_one_seconds_ago_and_required() {
    let sent = invocations(&suite(), REFUSAL);
    assert_eq!(
        sent,
        [(offset(-61), "start-in-past".to_owned())],
        "{sent:#?}"
    );
    assert!(
        scenario(&suite(), REFUSAL).iter().any(|step| matches!(
            step,
            ScenarioStep::ExpectError { error, .. } if error.to_string() == "demo.jobs.StartInPast"
        )),
        "the refusal's error is required"
    );
}

#[test]
fn issue_171_the_accepting_branch_is_sent_fifty_nine_seconds_ago() {
    let sent = invocations(&suite(), SCHEDULED);
    assert!(
        sent.contains(&(offset(-59), "scheduled".to_owned())),
        "the far side of the boundary is not witnessed: {sent:#?}"
    );
    assert!(
        sent.iter()
            .all(|(value, _)| matches!(value, ScenarioValue::NowOffset { .. })),
        "a now-guarded input is always sent relative to the moment it is sent: {sent:#?}"
    );
    assert!(
        !sent.iter().any(|(value, _)| *value == offset(-60)),
        "the boundary itself decides nothing a latency cannot flip: {sent:#?}"
    );
}

/// The row the scenario reads back holds the instant the command was sent, resolved once.
#[test]
fn the_stored_start_is_required_as_the_value_that_was_sent() {
    let suite = suite();
    let steps = scenario(&suite, SCHEDULED);
    let sent = match &invocations(&suite, SCHEDULED)[0].0 {
        ScenarioValue::NowOffset { seconds } => *seconds,
        other => panic!("{other:?}"),
    };
    assert!(
        steps.iter().any(|step| matches!(
            step,
            ScenarioStep::ExpectView {
                expectation: ViewExpectation::Contains { fields },
                ..
            } | ScenarioStep::EventuallyView {
                expectation: ViewExpectation::Contains { fields },
                ..
            } if fields.get("starts_at") == Some(&offset(sent))
        )),
        "{steps:#?}"
    );
    // The plain witness: a day and a second after the reference, clear of every whole-unit bound.
    assert_eq!(sent, 86_401);
    // An event payload carries literal nodes, so the copied instant is left to the shape.
    let event = steps
        .iter()
        .find_map(|step| match step {
            ScenarioStep::ExpectEvent { payload, shape, .. } => Some((payload, shape)),
            _ => None,
        })
        .expect("the event is required");
    assert!(!event.0.contains_key("starts_at"), "{event:?}");
    assert!(event.1.leaves().contains_key("starts_at"), "{event:?}");
}

#[test]
fn a_fresh_suite_carrying_now_offsets_declares_empty_state_and_preserves_round_three_compatibility()
{
    use ess_conformance::coverage::{Origins, Scope};

    let suite = suite();
    assert!(now_offset::used_by(&suite));
    assert_eq!(
        suite.provenance.suite_version.to_string(),
        "ess-conformance/34"
    );
    assert_eq!(
        suite.provenance.scenario_initial_state,
        Some(ScenarioInitialState::Empty)
    );
    assert_eq!(now_offset::ORDINARY, 26);
    assert_eq!(now_offset::COVERAGE, 27);
    let json = suite.to_canonical_json().unwrap();
    assert!(json.contains("\"now_offset\""), "{json}");
    assert!(json.contains("-61"), "{json}");
    AdmittedSuite::from_json(&json).expect("the suite it writes is admitted");

    let legacy = support_versions::legacy_json(&json, now_offset::ORDINARY);
    let admitted = AdmittedSuite::from_json(&legacy).expect("historical suite/26 is admitted");
    assert_eq!(admitted.suite().provenance.suite_version.major(), 26);
    assert_eq!(admitted.suite().provenance.scenario_initial_state, None);

    let coverage =
        ess_conformance::coverage_build::build(&ir(JOBS), &[], Scope::System, Origins::Generated)
            .unwrap();
    assert_eq!(
        coverage.selected().suite().provenance.suite_version.major(),
        35
    );
    assert_eq!(
        coverage
            .selected()
            .suite()
            .provenance
            .scenario_initial_state,
        Some(ScenarioInitialState::Empty)
    );
    let historical =
        support_versions::legacy_json(coverage.selected().original_json(), now_offset::COVERAGE);
    let admitted = AdmittedSuite::from_json(&historical).expect("historical suite/27 is admitted");
    assert_eq!(admitted.suite().provenance.suite_version.major(), 27);
    assert_eq!(admitted.suite().provenance.scenario_initial_state, None);
}

#[test]
fn an_older_suite_format_refuses_now_offsets() {
    let fresh = suite().to_canonical_json().unwrap();
    let legacy = support_versions::legacy_json(&fresh, now_offset::ORDINARY);
    AdmittedSuite::from_json(&legacy).expect("the compatibility fixture is valid suite/26");
    let older = support_versions::legacy_json(&legacy, 24);
    assert!(older.contains("\"suite_version\":\"ess-conformance/24\""));
    assert!(!older.contains("scenario_initial_state"));
    let refused = AdmittedSuite::from_json(&older).expect_err("suite/24 has no now_offset");
    assert!(
        refused.to_string().contains("now_offset"),
        "the refusal names the value kind: {refused}"
    );
}

#[test]
fn a_suite_without_now_keeps_its_format() {
    let fixed = JOBS.replace(
        "when: starts_at < now - 60s",
        "when: starts_at < '2020-01-01T00:00:00Z'",
    );
    let suite = suite_of(&fixed);
    assert!(!now_offset::used_by(&suite));
    assert_eq!(
        suite.provenance.suite_version.to_string(),
        "ess-conformance/34"
    );
    assert_eq!(
        suite.provenance.scenario_initial_state,
        Some(ScenarioInitialState::Empty)
    );
}

/// A lower bound is witnessed the same way from the other side: a second above it for the branch it
/// selects, a second below it for the default, and never at it.
#[test]
fn a_lower_bound_is_witnessed_a_second_either_side() {
    let suite =
        suite_of(&JOBS.replace("when: starts_at < now - 60s", "when: starts_at > now + 5m"));
    let guarded = invocations(&suite, REFUSAL);
    let default = invocations(&suite, SCHEDULED);
    assert!(
        guarded.contains(&(offset(301), "start-in-past".to_owned())),
        "{guarded:#?}"
    );
    assert!(
        default.contains(&(offset(299), "scheduled".to_owned())),
        "{default:#?}"
    );
    assert!(
        !guarded
            .iter()
            .chain(&default)
            .any(|(value, _)| *value == offset(300)),
        "the boundary is never sent: {guarded:#?} {default:#?}"
    );
}

/// A `now_offset` replaces a whole input field, so a guard over a `Timestamp` inside a struct is
/// refused by name rather than sent the reference instant as a literal.
#[test]
fn a_timestamp_inside_a_structure_is_refused_by_name() {
    let text = JOBS
        .replace(
            "  - {name: demo.jobs.JobId, kind: newtype, of: Uuid}\n",
            "  - {name: demo.jobs.JobId, kind: newtype, of: Uuid}\n  - name: demo.jobs.Window\n    kind: struct\n    fields:\n      - {name: starts_at, type: Timestamp}\n",
        )
        .replace(
            "      - {name: starts_at, type: Timestamp}\n    outcomes:",
            "      - {name: starts_at, type: Timestamp}\n      - {name: window, type: demo.jobs.Window}\n    outcomes:",
        )
        .replace(
            "when: starts_at < now - 60s",
            "when: window.starts_at < now - 60s",
        );
    let synthesis = synthesize(&ir(&text));
    let rendered = format!("{:?}", synthesis.refusals);
    assert!(
        rendered.contains("window.starts_at") && rendered.contains("now_offset"),
        "{rendered}"
    );
    assert!(
        !now_offset::used_by(&synthesis.suite)
            || synthesis.suite.scenarios.values().all(|scenario| {
                scenario.steps.iter().all(|step| {
                    !matches!(step, ScenarioStep::ExecuteCommand { command, .. }
                        if command.to_string() == "demo.jobs.ScheduleJob")
                })
            }),
        "no scenario sends the command with the reference instant"
    );
}

/// The input `when:` beside a held state is an input guard like a plain one: its now-guarded field
/// is sent as a `now_offset`, a second past the boundary, in the scenario that arranges the state.
#[test]
fn an_input_guard_beside_a_held_state_is_sent_relative_to_the_moment_of_sending() {
    let text = "format: ess/16\nsystem: calls\nversion: v1\ndomain: calls.core\nentities:\n  - name: calls.core.Call\n    identity: {name: call_id, type: Uuid}\n    fields: []\n    lifecycle:\n      initial: Bridged\n      states: [Bridged]\n      terminal: [Bridged]\nactors:\n  - {name: calls.core.Agent, may: [calls.core.Open, calls.core.Enrich]}\nevents:\n  - name: calls.core.Opened\n    fields:\n      - {name: call_id, type: Uuid}\n  - name: calls.core.Observed\n    fields: []\nerrors:\n  - name: calls.core.Stale\n    summary: The reading is more than a minute old.\n    fields: []\ncommands:\n  - name: calls.core.Open\n    input: []\n    outcomes:\n      - name: opened\n        creates: calls.core.Call\n        instance: call_id\n        emits: [calls.core.Opened]\n        payload:\n          calls.core.Opened: {call_id: {generated: true}}\n  - name: calls.core.Enrich\n    input:\n      - {name: call_id, type: Uuid}\n      - {name: read_at, type: Timestamp}\n    outcomes:\n      - name: stale\n        when_subject_state: Bridged\n        when: read_at < now - 60s\n        updates: calls.core.Call\n        instance: call_id\n        emits: [calls.core.Observed]\n      - name: preserved\n        updates: calls.core.Call\n        instance: call_id\n        emits: [calls.core.Observed]\nviews:\n  - name: calls.core.Calls\n    source: calls.core.Call\n    consistency: read_your_writes\n    fields:\n      - {name: call_id, type: Uuid}\n      - {name: state, type: calls.core.Call.State}\n";
    let synthesis = synthesize(&ir(text));
    let (_, stale) = synthesis
        .suite
        .scenarios
        .iter()
        .find(|(id, _)| id.to_string() == "calls.core.Enrich/outcome/stale")
        .unwrap_or_else(|| panic!("{:#?}", synthesis.refusals));
    let sent: Vec<&ScenarioValue> = stale
        .steps
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::ExecuteCommand { command, input, .. }
                if command.to_string() == "calls.core.Enrich" =>
            {
                input.get("read_at")
            }
            _ => None,
        })
        .collect();
    assert_eq!(sent, [&offset(-61)], "{:#?}", stale.steps);
}

// ---- running the suite ---------------------------------------------------------------------

type Row = BTreeMap<String, Node>;

/// A job service whose own clock reads `now()` and that refuses a start more than `tolerance`
/// seconds before it.
struct Jobs<F: Fn() -> Rfc3339Instant> {
    now: F,
    tolerance: i64,
    rows: RefCell<Vec<Row>>,
    minted: Cell<u64>,
}

impl<F: Fn() -> Rfc3339Instant> Jobs<F> {
    fn new(now: F, tolerance: i64) -> Self {
        Self {
            now,
            tolerance,
            rows: RefCell::default(),
            minted: Cell::new(0),
        }
    }
}

fn outcome(command: &CommandRef, name: &str) -> OutcomeRef {
    OutcomeRef::new(command.clone(), OutcomeName::new(name).unwrap())
}

impl<F: Fn() -> Rfc3339Instant> ConformanceTarget for Jobs<F> {
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
        let limit = (self.now)().plus_seconds(-self.tolerance).unwrap();
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

/// The runner's clock, with a wall clock that reads a fixed instant: the moment every
/// `now_offset` of a run is resolved against.
struct FixedWall {
    steps: AdvancingClock,
    wall: Timestamp,
}

impl Clock for FixedWall {
    fn now(&mut self) -> Timestamp {
        self.steps.now()
    }

    fn wall(&mut self) -> Timestamp {
        self.wall
    }
}

/// 2026-09-27T12:00:00.250Z, a quarter second past a whole second.
const WALL_MS: u64 = 1_790_510_400_250;

/// The scenarios that do not pass against a service whose clock reads `latency_ms` after the
/// runner's, refusing starts more than `tolerance` seconds in the past.
fn failing(tolerance: i64, latency_ms: u64) -> BTreeSet<String> {
    let suite = suite();
    let admitted = AdmittedSuite::from_suite(&suite).unwrap();
    let service_now = move || {
        Rfc3339Instant::from_epoch_millis(i64::try_from(WALL_MS + latency_ms).unwrap()).unwrap()
    };
    let runner = Runner::new(
        RunnerConfig::default(),
        FixedWall {
            steps: AdvancingClock::default(),
            wall: Timestamp::from_epoch_millis(WALL_MS),
        },
        Ids::for_suite(&suite),
    );
    let report = runner
        .run_admitted(&admitted, &Jobs::new(service_now, tolerance))
        .into_report();
    let failed: BTreeSet<String> = report
        .scenarios
        .iter()
        .filter(|scenario| scenario.status != Status::Passed)
        .inspect(|scenario| eprintln!("{tolerance}s/{latency_ms}ms: {scenario:#?}"))
        .map(|scenario| scenario.scenario.to_string())
        .collect();
    assert_eq!(
        report.status == ConformanceStatus::Passed,
        failed.is_empty(),
        "{report:?}"
    );
    failed
}

#[test]
fn issue_171_a_service_that_refuses_starts_over_sixty_seconds_old_passes() {
    assert_eq!(failing(60, 0), BTreeSet::new());
}

#[test]
fn a_service_handling_the_request_under_a_second_later_still_passes() {
    assert_eq!(failing(60, 999), BTreeSet::new());
}

#[test]
fn a_service_with_a_tighter_tolerance_fails_the_accepting_boundary() {
    assert_eq!(failing(30, 0), BTreeSet::from([SCHEDULED.to_owned()]));
}

#[test]
fn a_service_with_a_looser_tolerance_fails_the_refusal() {
    assert_eq!(failing(120, 0), BTreeSet::from([REFUSAL.to_owned()]));
}

fn machine_millis() -> u64 {
    let millis = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    u64::try_from(millis).unwrap()
}

/// Against the machine's own clock, as an adopter runs it: the runner is handed the machine's
/// clock as its wall, and the service reads the same clock a moment later.
#[test]
fn a_service_reading_the_real_clock_passes_its_own_suite() {
    let suite = suite();
    let admitted = AdmittedSuite::from_suite(&suite).unwrap();
    let service_now =
        || Rfc3339Instant::from_epoch_millis(i64::try_from(machine_millis()).unwrap()).unwrap();
    let runner = Runner::new(
        RunnerConfig::default(),
        now_offset::WithWall::new(AdvancingClock::default(), || {
            Timestamp::from_epoch_millis(machine_millis())
        }),
        Ids::for_suite(&suite),
    );
    let report = runner
        .run_admitted(&admitted, &Jobs::new(service_now, 60))
        .into_report();
    assert_eq!(report.status, ConformanceStatus::Passed, "{report:#?}");
}
