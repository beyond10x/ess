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
/// The scenario synthesis puts on the boundary: `starts_at` sent as `now - 60s` exactly, the
/// command decided at a held instant (beyond10x/ess#586).
const HELD: &str = "demo.jobs.ScheduleJob/outcome/scheduled/held/starts_at/-60";

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
    // The scenario on the boundary carries a `hold_now`, so the suite is written in /50
    // (beyond10x/ess#586).
    assert_eq!(
        suite.provenance.suite_version.to_string(),
        "ess-conformance/50"
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

    let legacy = support_versions::legacy_json(&unheld(&json), now_offset::ORDINARY);
    let admitted = AdmittedSuite::from_json(&legacy).expect("historical suite/26 is admitted");
    assert_eq!(admitted.suite().provenance.suite_version.major(), 26);
    assert_eq!(admitted.suite().provenance.scenario_initial_state, None);

    let coverage =
        ess_conformance::coverage_build::build(&ir(JOBS), &[], Scope::System, Origins::Generated)
            .unwrap();
    assert_eq!(
        coverage.selected().suite().provenance.suite_version.major(),
        51
    );
    assert_eq!(
        coverage
            .selected()
            .suite()
            .provenance
            .scenario_initial_state,
        Some(ScenarioInitialState::Empty)
    );
    let historical = support_versions::legacy_json(
        &unheld(coverage.selected().original_json()),
        now_offset::COVERAGE,
    );
    let admitted = AdmittedSuite::from_json(&historical).expect("historical suite/27 is admitted");
    assert_eq!(admitted.suite().provenance.suite_version.major(), 27);
    assert_eq!(admitted.suite().provenance.scenario_initial_state, None);
}

/// `json` without its scenarios on a `now` boundary, the suite synthesis wrote before
/// beyond10x/ess#586: what an older format can still carry.
fn unheld(json: &str) -> String {
    fn strip(value: &mut serde_json::Value) {
        match value {
            serde_json::Value::Object(map) => {
                map.retain(|id, _| !id.contains("/held/"));
                map.values_mut().for_each(strip);
                // An inventory counts its arrays.
                for key in ["generated", "authored"] {
                    let length = map.get(key).and_then(|ids| ids.as_array()).map(Vec::len);
                    if let (Some(length), Some(counts)) = (length, map.get_mut("counts")) {
                        counts[key] = serde_json::json!(length);
                    }
                }
            }
            serde_json::Value::Array(items) => {
                items.retain(|item| !item.as_str().is_some_and(|id| id.contains("/held/")));
                items.iter_mut().for_each(strip);
            }
            _ => {}
        }
    }
    let mut document: serde_json::Value = serde_json::from_str(json).unwrap();
    strip(&mut document);
    serde_json::to_string(&document).unwrap()
}

#[test]
fn an_older_suite_format_refuses_now_offsets() {
    let fresh = suite().to_canonical_json().unwrap();
    let legacy = support_versions::legacy_json(&unheld(&fresh), now_offset::ORDINARY);
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
    /// Whether it refuses a start exactly `tolerance` seconds old too: the `<=` mutant of the guard.
    inclusive: bool,
    /// The instant `hold_now` fixed for the next command, if any.
    held: Cell<Option<Rfc3339Instant>>,
    rows: RefCell<Vec<Row>>,
    minted: Cell<u64>,
}

impl<F: Fn() -> Rfc3339Instant> Jobs<F> {
    fn new(now: F, tolerance: i64) -> Self {
        Self {
            now,
            tolerance,
            inclusive: false,
            held: Cell::new(None),
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
        let now = self.held.take().unwrap_or_else(|| (self.now)());
        let limit = now.plus_seconds(-self.tolerance).unwrap();
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
        self.held.set(Some(request.instant));
        Ok(())
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
    assert_eq!(
        failing(30, 0),
        BTreeSet::from([SCHEDULED.to_owned(), HELD.to_owned()])
    );
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

// ---- a held `now` (beyond10x/ess#586) ------------------------------------------------------

/// [`suite`], with a `hold_now 1` before its accepting scenario's command: that command is decided
/// a second after the instant its `now - 59s` is sent relative to, so `starts_at` lies exactly on
/// the boundary, `now - 60s`, the one value the `<=` mutant answers differently.
fn held_suite() -> ConformanceSuite {
    let mut suite = suite();
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
                if input.get("starts_at") == Some(&offset(-59)))
        })
        .expect("its command sent a second inside the boundary");
    scenario
        .steps
        .insert(at, ScenarioStep::HoldNow { seconds: 1 });
    suite.select_fresh_format();
    suite
}

/// The verdict of every scenario of `suite` against `target`, under a wall a quarter second past
/// a whole second.
fn verdicts<T: ConformanceTarget>(
    suite: &ConformanceSuite,
    target: &T,
) -> BTreeMap<String, Status> {
    let admitted = AdmittedSuite::from_suite(suite).unwrap();
    let runner = Runner::new(
        RunnerConfig::default(),
        FixedWall {
            steps: AdvancingClock::default(),
            wall: Timestamp::from_epoch_millis(WALL_MS),
        },
        Ids::for_suite(suite),
    );
    runner
        .run_admitted(&admitted, target)
        .into_report()
        .scenarios
        .iter()
        .map(|scenario| (scenario.scenario.to_string(), scenario.status))
        .collect()
}

/// The service's own clock trails the runner's by 900 ms: without the held instant the boundary
/// value would be decided in the accepting branch's favour by luck alone.
fn trailing() -> Rfc3339Instant {
    Rfc3339Instant::from_epoch_millis(i64::try_from(WALL_MS + 900).unwrap()).unwrap()
}

#[test]
fn a_held_step_is_written_in_suite_fifty_and_refused_below_it() {
    let suite = held_suite();
    assert_eq!(
        suite.provenance.suite_version.to_string(),
        "ess-conformance/50"
    );
    let json = suite.to_canonical_json().unwrap();
    assert!(
        json.contains("\"step\": \"hold_now\",\n          \"seconds\": 1\n"),
        "{json}"
    );
    AdmittedSuite::from_json(&json).expect("the suite it writes is admitted");
    let older = json.replace("\"ess-conformance/50\"", "\"ess-conformance/48\"");
    let refused = AdmittedSuite::from_json(&older).expect_err("suite/48 has no hold_now");
    assert!(refused.to_string().contains("hold_now"), "{refused}");
}

#[test]
fn a_hold_now_not_followed_by_a_command_is_refused() {
    let mut suite = held_suite();
    let (_, scenario) = suite
        .scenarios
        .iter_mut()
        .find(|(id, _)| id.to_string() == SCHEDULED)
        .unwrap();
    scenario.steps.push(ScenarioStep::HoldNow { seconds: 0 });
    let refused = AdmittedSuite::from_suite(&suite).expect_err("a dangling hold");
    assert!(
        refused.to_string().contains("hold_now") && refused.to_string().contains("execute_command"),
        "{refused}"
    );
}

#[test]
fn fixed_clock_decides_the_input_boundary_at_the_held_instant() {
    let suite = held_suite();
    let correct = verdicts(&suite, &Jobs::new(trailing, 60));
    assert!(
        correct.values().all(|status| *status == Status::Passed),
        "{correct:#?}"
    );
    let mut mutant = Jobs::new(trailing, 60);
    mutant.inclusive = true;
    let killed = verdicts(&suite, &mutant);
    assert_eq!(killed[SCHEDULED], Status::Failed, "{killed:#?}");
    assert_eq!(killed[REFUSAL], Status::Passed, "{killed:#?}");
}

/// `Jobs` without `hold_now`: every other method delegated, so the trait's default body answers.
struct Unheld<'a, F: Fn() -> Rfc3339Instant>(&'a Jobs<F>);

impl<F: Fn() -> Rfc3339Instant> ConformanceTarget for Unheld<'_, F> {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        self.0.identity()
    }
    fn begin_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.0.begin_scenario(scenario)
    }
    fn end_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.0.end_scenario(scenario)
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        self.0.execute_command(request)
    }
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        self.0.query_view(request)
    }
    fn observe_events(
        &self,
        request: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        self.0.observe_events(request)
    }
    fn configure_external_outcome(
        &self,
        control: ExternalOutcomeControl,
    ) -> Result<(), TargetError> {
        self.0.configure_external_outcome(control)
    }
    fn redeliver_event(&self, request: RedeliveryRequest) -> Result<(), TargetError> {
        self.0.redeliver_event(request)
    }
}

/// A target without `hold_now` runs the synthesized suite as before: the scenario on the boundary
/// is reported `unsupported`, naming the capability, and every other scenario passes.
#[test]
fn target_without_the_capability() {
    let jobs = Jobs::new(trailing, 60);
    let suite = suite();
    let admitted = AdmittedSuite::from_suite(&suite).unwrap();
    let report = Runner::new(
        RunnerConfig::default(),
        FixedWall {
            steps: AdvancingClock::default(),
            wall: Timestamp::from_epoch_millis(WALL_MS),
        },
        Ids::for_suite(&suite),
    )
    .run_admitted(&admitted, &Unheld(&jobs))
    .into_report();
    let held = report
        .scenarios
        .iter()
        .find(|scenario| scenario.scenario.to_string() == HELD)
        .unwrap();
    assert_eq!(held.status, Status::Unsupported, "{held:#?}");
    assert!(format!("{held:?}").contains("hold_now"), "{held:#?}");
    let others: BTreeMap<String, Status> = report
        .scenarios
        .iter()
        .filter(|scenario| scenario.scenario.to_string() != HELD)
        .map(|scenario| (scenario.scenario.to_string(), scenario.status))
        .collect();
    assert!(others.contains_key(SCHEDULED) && others.contains_key(REFUSAL));
    assert!(
        others.values().all(|status| *status == Status::Passed),
        "{report:#?}"
    );
}

/// Synthesis adds the boundary itself: `starts_at` sent exactly `now - 60s`, `hold_now 0` before
/// the command, and the branch `<` takes there required. The off-boundary scenarios keep their
/// values; a lower bound `>=` is held on the guarded branch's side.
#[test]
fn input_guard_boundary_on_now() {
    let suite = suite();
    let steps = scenario(&suite, HELD);
    let at = steps
        .iter()
        .position(|step| matches!(step, ScenarioStep::HoldNow { .. }))
        .expect("a held step");
    assert!(
        matches!(steps[at], ScenarioStep::HoldNow { seconds: 0 }),
        "{steps:#?}"
    );
    assert!(
        matches!(&steps[at + 1], ScenarioStep::ExecuteCommand { input, .. }
            if input["starts_at"] == offset(-60)),
        "{steps:#?}"
    );
    assert_eq!(
        invocations(&suite, HELD),
        [(offset(-60), "scheduled".to_owned())]
    );
    assert!(
        invocations(&suite, SCHEDULED).contains(&(offset(-59), "scheduled".to_owned())),
        "the off-boundary scenario keeps its witness"
    );
    assert_eq!(
        invocations(&suite, REFUSAL),
        [(offset(-61), "start-in-past".to_owned())]
    );
    assert_eq!(
        suite
            .scenarios
            .keys()
            .filter(|id| id.to_string().contains("/held/"))
            .count(),
        1
    );

    let lower =
        suite_of(&JOBS.replace("when: starts_at < now - 60s", "when: starts_at >= now + 5m"));
    assert_eq!(
        invocations(
            &lower,
            "demo.jobs.ScheduleJob/outcome/start-in-past/held/starts_at/300"
        ),
        [(offset(300), "start-in-past".to_owned())]
    );
}

/// `Jobs` with no clock of its own: a command it is not held for is `unsupported`, so only the
/// scenarios on a boundary execute, decided at the instant they hold.
struct HeldOnly<F: Fn() -> Rfc3339Instant>(Jobs<F>);

impl<F: Fn() -> Rfc3339Instant> ConformanceTarget for HeldOnly<F> {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        self.0.identity()
    }
    fn begin_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.0.begin_scenario(scenario)
    }
    fn end_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.0.end_scenario(scenario)
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        if self.0.held.get().is_none() {
            return Err(TargetError::unsupported(
                "now",
                "this service decides only a held command",
            ));
        }
        self.0.execute_command(request)
    }
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        self.0.query_view(request)
    }
    fn observe_events(
        &self,
        request: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        self.0.observe_events(request)
    }
    fn configure_external_outcome(
        &self,
        control: ExternalOutcomeControl,
    ) -> Result<(), TargetError> {
        self.0.configure_external_outcome(control)
    }
    fn redeliver_event(&self, request: RedeliveryRequest) -> Result<(), TargetError> {
        self.0.redeliver_event(request)
    }
    fn hold_now(&self, request: NowHold) -> Result<(), TargetError> {
        self.0.hold_now(request)
    }
}

/// The mutation audit's `guard-boundary` mutant `starts_at <= now - 60s` answers apart from the
/// guard only on the boundary. Its own suite holds that boundary and requires the refusal there,
/// which the unchanged service, decided at the held instant, does not give: the mutant is killed by
/// the held scenario.
#[test]
fn fixed_clock_kills_the_boundary_mutant() {
    use ess_conformance::mutate::{self, MutantClass, Verdict};

    let label = "jobs.yaml".to_owned();
    let raw = RawSpecFile::parse(JOBS).unwrap_or_else(|error| panic!("{error}"));
    let mut texts = SourceMap::new();
    texts.insert(label.clone(), JOBS.to_owned());
    let documents = vec![(Source::new(label), raw)];
    let report = mutate::audit(&documents, &texts, &[MutantClass::GuardBoundary], || {
        HeldOnly(Jobs::new(trailing, 60))
    })
    .unwrap_or_else(|refusal| panic!("{refusal}"));
    let mutant = report
        .mutants
        .iter()
        .find(|mutant| mutant.change.contains("starts_at <= now - 60s"))
        .unwrap_or_else(|| panic!("the strictness mutant: {report:#?}"));
    assert_eq!(mutant.verdict, Verdict::Killed, "{mutant:#?}");
    assert_eq!(
        mutant.killers.as_deref(),
        Some(&["demo.jobs.ScheduleJob/outcome/start-in-past/held/starts_at/-60".to_owned()][..]),
        "{mutant:#?}"
    );
}
