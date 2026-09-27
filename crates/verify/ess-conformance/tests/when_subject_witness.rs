//! A `when_subject:` command with a guard-less fallback is witnessed branch by branch, and refused
//! in the states it does not run from, whatever consistency its views declare (beyond10x/ess#172,
//! beyond10x/ess#173).
//!
//! Two fixtures, one model: `when-subject-fallback-refusal.yaml` reads the job through a
//! `read_your_writes` view, `when-subject-eventual.yaml` through an `eventual` one. Each is
//! synthesized with no `ESS-SYNTH-001`, `-004` or `-008` refusal, and the suite it produces is run
//! against the rule as specified and against the implementations each family exists to catch.
use ess_compiler::{
    ir::EssIr,
    refs::{CommandRef, OutcomeRef},
    resolve::compile,
    source::SourceMap,
};
use ess_conformance::{
    report::{ConformanceStatus, Status},
    synthesize::Synthesis,
    target::*,
    AdmittedSuite, ConformanceScenario, ConformanceSuite, Runner, ScenarioStep,
};
use ess_domain::{
    command::OutcomeName,
    spec::{RawSpecFile, Specification},
    system::Source,
};
use ess_primitives::node::Node;
use std::{
    cell::{Cell, RefCell},
    collections::BTreeMap,
};

const IMMEDIATE: &str = include_str!("fixtures/when-subject-fallback-refusal.yaml");
const EVENTUAL: &str = include_str!("fixtures/when-subject-eventual.yaml");

const STARTED: &str = "demo.jobs.AdvanceJob/outcome/started";
const DIRECT: &str = "demo.jobs.AdvanceJob/outcome/direct";
const START: &str = "demo.jobs.Job/transition/start/by/demo.jobs.AdvanceJob/started";
const GO_DIRECT: &str = "demo.jobs.Job/transition/go-direct/by/demo.jobs.AdvanceJob/direct";
const REFUSED_RUNNING: &str = "demo.jobs.Job/state/Running/refuses/demo.jobs.AdvanceJob";
const REFUSED_DIRECT: &str = "demo.jobs.Job/state/Direct/refuses/demo.jobs.AdvanceJob";

fn synthesis(text: &str) -> Synthesis {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}"));
    let spec = Specification::assemble([(Source::new("jobs.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    let ir: EssIr = compile(&spec, &SourceMap::new()).unwrap();
    ess_conformance::synthesize::synthesize(&ir)
}

fn scenario<'a>(suite: &'a ConformanceSuite, id: &str) -> &'a ConformanceScenario {
    suite
        .scenarios
        .iter()
        .find(|(key, _)| key.to_string() == id)
        .map_or_else(
            || {
                panic!(
                    "no scenario {id} in {:?}",
                    suite
                        .scenarios
                        .keys()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                )
            },
            |(_, scenario)| scenario,
        )
}

/// Every refusal as `<code> <scenario>`, and its full text beside it.
fn refusals(result: &Synthesis) -> Vec<(String, String)> {
    result
        .refusals
        .iter()
        .map(|refusal| {
            (
                format!(
                    "{} {}",
                    refusal.cause.code(),
                    refusal
                        .scenario
                        .as_ref()
                        .map_or_else(String::new, ToString::to_string)
                ),
                format!("{} / {}", refusal.cause, refusal.cause.hint()),
            )
        })
        .collect()
}

fn assert_no_witness_gap(result: &Synthesis) {
    let all = refusals(result);
    let gaps: Vec<_> = all
        .iter()
        .filter(|(code, _)| {
            ["ESS-SYNTH-001 ", "ESS-SYNTH-004 ", "ESS-SYNTH-008 "]
                .iter()
                .any(|banned| code.starts_with(banned))
        })
        .collect();
    assert!(gaps.is_empty(), "{gaps:#?}");
    for (code, text) in &all {
        assert!(
            !text.contains("drifted apart"),
            "an internal drift message reached the author: {code}: {text}"
        );
    }
}

/// The views a scenario reads before it sends `AdvanceJob`, by step kind.
fn observed_before_advance(scenario: &ConformanceScenario) -> Vec<&'static str> {
    let mut kinds = Vec::new();
    for step in &scenario.steps {
        match step {
            ScenarioStep::ExecuteCommand { command, .. }
                if command.to_string() == "demo.jobs.AdvanceJob" =>
            {
                break;
            }
            ScenarioStep::EventuallyView { .. } => kinds.push("eventually"),
            ScenarioStep::QueryView { .. } | ScenarioStep::ExpectView { .. } => {
                kinds.push("expect");
            }
            _ => {}
        }
    }
    kinds
}

#[test]
fn a_guard_less_fallback_gets_its_refusal_scenarios_in_every_state_it_does_not_run_from() {
    let result = synthesis(IMMEDIATE);
    assert_no_witness_gap(&result);
    for id in [
        STARTED,
        DIRECT,
        START,
        GO_DIRECT,
        REFUSED_RUNNING,
        REFUSED_DIRECT,
    ] {
        scenario(&result.suite, id);
    }
}

#[test]
fn the_refusal_scenario_sends_the_command_to_the_arranged_row_and_forbids_every_event() {
    let result = synthesis(IMMEDIATE);
    for id in [REFUSED_RUNNING, REFUSED_DIRECT] {
        let refused = scenario(&result.suite, id);
        let text = serde_json::to_string(&refused.steps).unwrap();
        let sent = refused
            .steps
            .iter()
            .filter(|step| {
                matches!(step, ScenarioStep::ExecuteCommand { command, .. }
                    if command.to_string() == "demo.jobs.AdvanceJob")
            })
            .count();
        assert!(sent >= 1, "{id}: {text}");
        for event in ["demo.jobs.JobMoved", "demo.jobs.JobSubmitted"] {
            assert!(
                refused.steps.iter().any(|step| matches!(
                    step,
                    ScenarioStep::ExpectNoEvent { event: forbidden } if forbidden.to_string() == event
                )),
                "{id} does not forbid {event}: {text}"
            );
        }
    }
}

#[test]
fn only_eventual_views_still_witness_both_branches_and_every_state_after_them() {
    let result = synthesis(EVENTUAL);
    assert_no_witness_gap(&result);
    for id in [
        STARTED,
        DIRECT,
        START,
        GO_DIRECT,
        REFUSED_RUNNING,
        REFUSED_DIRECT,
    ] {
        scenario(&result.suite, id);
    }
}

#[test]
fn an_eventual_view_observes_the_arranged_fact_in_its_own_block_before_the_command() {
    let result = synthesis(EVENTUAL);
    for id in [STARTED, DIRECT] {
        let kinds = observed_before_advance(scenario(&result.suite, id));
        assert!(
            kinds.contains(&"eventually"),
            "{id} does not wait for the arranged row: {kinds:?}"
        );
        assert!(
            !kinds.contains(&"expect"),
            "{id} asserts an eventual view once, racing its projection: {kinds:?}"
        );
    }
    let immediate = synthesis(IMMEDIATE);
    for id in [STARTED, DIRECT] {
        let kinds = observed_before_advance(scenario(&immediate.suite, id));
        assert!(
            !kinds.contains(&"eventually"),
            "{id}: an immediate view keeps its expect block: {kinds:?}"
        );
    }
}

#[test]
fn without_a_view_projecting_the_flag_the_refusal_names_either_consistency() {
    let unprojected = EVENTUAL
        .strip_suffix("      - {name: fast, type: Boolean}\n      - {name: state, type: demo.jobs.Job.State}\n")
        .map(|head| format!("{head}      - {{name: state, type: demo.jobs.Job.State}}\n"))
        .expect("the view projects the flag before the state, last");
    let result = synthesis(&unprojected);
    let about: Vec<_> = refusals(&result)
        .into_iter()
        .filter(|(code, _)| code == &format!("ESS-SYNTH-001 {STARTED}"))
        .collect();
    assert_eq!(about.len(), 1, "{:#?}", refusals(&result));
    let (_, text) = &about[0];
    assert!(text.contains("`demo.jobs.Job.fast`"), "{text}");
    assert!(
        text.contains("`read_your_writes` or `eventual`"),
        "the help names the view the author can declare, of either consistency: {text}"
    );
}

#[test]
fn a_subject_fact_branch_asked_for_an_input_is_not_reported_as_drift() {
    let cause = ess_conformance::synthesize::RefusalCause::StrategyWithoutGuard {
        strategy: ess_domain::command::TestStrategy::ObserveSubjectFact,
    };
    let text = format!("{cause} / {}", cause.hint());
    assert!(!text.contains("drifted apart"), "{text}");
    assert!(text.contains("stored fields"), "{text}");
    assert!(text.contains("authored scenario"), "{text}");
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Behavior {
    /// The rule as specified.
    Good,
    /// Sends every job direct: the stored flag is never read.
    IgnoresFlag,
    /// Starts every job: the stored flag is never read.
    AlwaysStarts,
    /// Moves a job that has already left the queue.
    MovesFromAnyState,
}

struct Jobs {
    behavior: Behavior,
    rows: RefCell<BTreeMap<String, BTreeMap<String, Node>>>,
    minted: Cell<u64>,
}

fn outcome(command: &CommandRef, name: &str) -> OutcomeRef {
    OutcomeRef::new(command.clone(), OutcomeName::new(name).unwrap())
}

impl ConformanceTarget for Jobs {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("jobs-fixture", "1"))
    }
    fn begin_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        self.rows.replace(BTreeMap::new());
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
        self.answer(&request)
            .map(|result| result.with_consistency(token))
    }
    fn query_view(&self, _: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        Ok(SemanticViewResult::of(self.rows.borrow().values().cloned()))
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

impl Jobs {
    fn new(behavior: Behavior) -> Self {
        Self {
            behavior,
            rows: RefCell::default(),
            minted: Cell::new(0),
        }
    }

    fn answer(
        &self,
        request: &SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        let mut rows = self.rows.borrow_mut();
        let command = request.command.clone();
        match command.to_string().as_str() {
            "demo.jobs.SubmitJob" => {
                self.minted.set(self.minted.get() + 1);
                let id = format!("00000000-0000-4000-8000-{:012}", self.minted.get());
                let fast = request.input["fast"].clone();
                rows.insert(
                    id.clone(),
                    BTreeMap::from([
                        ("job_id".to_owned(), Node::Text(id.clone())),
                        ("state".to_owned(), Node::Text("Queued".into())),
                        ("fast".to_owned(), fast.clone()),
                    ]),
                );
                Ok(
                    SemanticCommandResult::took(outcome(&command, "submitted")).emitting(
                        ObservedEvent::new("demo.jobs.JobSubmitted".parse().unwrap())
                            .with("job_id", Node::Text(id))
                            .with("fast", fast),
                    ),
                )
            }
            "demo.jobs.AdvanceJob" => {
                let Node::Text(id) = &request.input["job_id"] else {
                    return Ok(SemanticCommandResult::undeclared());
                };
                let Some(row) = rows.get_mut(id) else {
                    return Ok(SemanticCommandResult::undeclared());
                };
                if row["state"] != Node::Text("Queued".into())
                    && self.behavior != Behavior::MovesFromAnyState
                {
                    return Ok(SemanticCommandResult::undeclared());
                }
                let fast = row["fast"] == Node::Bool(true);
                let start = match self.behavior {
                    Behavior::Good | Behavior::MovesFromAnyState => fast,
                    Behavior::IgnoresFlag => false,
                    Behavior::AlwaysStarts => true,
                };
                let (branch, state) = if start {
                    ("started", "Running")
                } else {
                    ("direct", "Direct")
                };
                row.insert("state".into(), Node::Text(state.into()));
                Ok(
                    SemanticCommandResult::took(outcome(&command, branch)).emitting(
                        ObservedEvent::new("demo.jobs.JobMoved".parse().unwrap())
                            .with("job_id", Node::Text(id.clone()))
                            .with("state", Node::Text(state.into())),
                    ),
                )
            }
            other => Err(TargetError::unsupported("command", other)),
        }
    }
}

fn failing(text: &str, behavior: Behavior) -> Vec<String> {
    let suite = synthesis(text).suite;
    let admitted = AdmittedSuite::from_suite(&suite).unwrap();
    let report = Runner::for_suite(&suite)
        .run_admitted(&admitted, &Jobs::new(behavior))
        .into_report();
    let failed: Vec<String> = report
        .scenarios
        .iter()
        .filter(|scenario| scenario.status != Status::Passed)
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
fn the_rule_as_specified_passes_both_suites() {
    for text in [IMMEDIATE, EVENTUAL] {
        assert_eq!(failing(text, Behavior::Good), Vec::<String>::new());
    }
}

#[test]
fn an_implementation_that_never_reads_the_flag_fails_the_guarded_branch() {
    for text in [IMMEDIATE, EVENTUAL] {
        let ignored = failing(text, Behavior::IgnoresFlag);
        assert!(ignored.contains(&STARTED.to_owned()), "{ignored:?}");
        let started = failing(text, Behavior::AlwaysStarts);
        assert!(started.contains(&DIRECT.to_owned()), "{started:?}");
    }
}

#[test]
fn an_implementation_that_moves_a_job_out_of_a_terminal_state_fails_its_refusal() {
    for text in [IMMEDIATE, EVENTUAL] {
        let failed = failing(text, Behavior::MovesFromAnyState);
        for id in [REFUSED_RUNNING, REFUSED_DIRECT] {
            assert!(failed.contains(&id.to_owned()), "{id} passed: {failed:?}");
        }
    }
}

/// A refusal carried by the stored flag, where the guarded field is projected only by an
/// `eventual` view and an immediate view shows the identity and the state.
fn eventual_refusal() -> String {
    IMMEDIATE
        .replace(
            "      states: [Queued, Running, Direct]\n      terminal: [Running, Direct]\n      transitions:\n        - {name: start, from: [Queued], to: Running}\n",
            "      states: [Queued, Direct]\n      terminal: [Direct]\n      transitions:\n",
        )
        .replace(
            "      - name: started\n        when_subject:\n          predicate: fast == true\n        moves: demo.jobs.Job.start\n        instance: job_id\n        emits: [demo.jobs.JobMoved]\n        payload:\n          demo.jobs.JobMoved: {job_id: input.job_id, state: Running}\n",
            "      - name: held\n        when_subject:\n          predicate: fast == true\n        error: demo.jobs.JobHeld\n",
        )
        .replace(
            "events:\n",
            "errors:\n  - name: demo.jobs.JobHeld\n    summary: The job is held.\nevents:\n",
        )
        .replace(
            "    consistency: read_your_writes\n    fields:\n      - {name: job_id, type: demo.jobs.JobId}\n      - {name: fast, type: Boolean}\n      - {name: state, type: demo.jobs.Job.State}\n",
            "    consistency: read_your_writes\n    fields:\n      - {name: job_id, type: demo.jobs.JobId}\n      - {name: state, type: demo.jobs.Job.State}\n  - name: demo.jobs.JobFeed\n    source: demo.jobs.Job\n    consistency: eventual\n    fields:\n      - {name: job_id, type: demo.jobs.JobId}\n      - {name: fast, type: Boolean}\n      - {name: state, type: demo.jobs.Job.State}\n",
        )
}

#[test]
fn a_refused_row_is_never_asserted_unchanged_through_an_eventual_view() {
    let text = eventual_refusal();
    assert!(text.contains("demo.jobs.JobFeed") && text.contains("name: held"));
    let result = synthesis(&text);
    let held = scenario(&result.suite, "demo.jobs.AdvanceJob/outcome/held");
    let sent = held
        .steps
        .iter()
        .rposition(|step| {
            matches!(step, ScenarioStep::ExecuteCommand { command, .. }
                if command.to_string() == "demo.jobs.AdvanceJob")
        })
        .expect("the refusal is sent");
    let after: Vec<_> = held.steps[sent..]
        .iter()
        .filter(|step| matches!(step, ScenarioStep::EventuallyView { .. }))
        .collect();
    assert!(
        after.is_empty(),
        "an eventual read after a refusal passes on a projection that has not caught up: {}",
        serde_json::to_string(&held.steps).unwrap()
    );
    assert!(
        held.steps[..sent]
            .iter()
            .any(|step| matches!(step, ScenarioStep::EventuallyView { .. })),
        "the arranged flag is still awaited before the command: {}",
        serde_json::to_string(&held.steps).unwrap()
    );
}
