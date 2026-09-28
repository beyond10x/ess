//! Adversary, pass 2, story:current-time-guard-operand (beyond10x/ess#171).
//!
//! Pass 1's correction keeps a value chosen from a fixed instant's boundary literal when the same
//! field is also ordered against `now`. These cases attack that correction and the new value kind:
//! a correct service, whose clock reads the runner's wall, must pass every scenario synthesis
//! emits, and an admitted suite must not carry an offset past the bound admission states.
use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::rc::Rc;

use ess_compiler::{
    ir::EssIr,
    refs::{CommandRef, OutcomeRef},
    resolve::compile,
    source::SourceMap,
};
use ess_conformance::{
    report::Status, synthesize::synthesize, target::*, AdmittedSuite, AdvancingClock, Clock,
    ConformanceSuite, Ids, Runner, RunnerConfig, ScenarioStep,
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

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("jobs.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap()
}

fn edited(base: &str, before: &str, after: &str) -> String {
    assert!(base.contains(before), "fixture holds {before:?}");
    base.replacen(before, after, 1)
}

type Row = BTreeMap<String, Node>;

/// The rule a correct service applies to one request at its clock: the outcome and declared error
/// it takes, or `None` for `scheduled`.
type Rule = dyn Fn(&BTreeMap<String, Node>, Rfc3339Instant) -> Option<(&'static str, &'static str)>;

struct Jobs {
    time: Rc<Cell<u64>>,
    rule: Box<Rule>,
    rows: RefCell<Vec<Row>>,
    minted: Cell<u64>,
}

fn instant_ms(ms: u64) -> Rfc3339Instant {
    Rfc3339Instant::from_epoch_millis(i64::try_from(ms).unwrap()).unwrap()
}

fn outcome(command: &CommandRef, name: &str) -> OutcomeRef {
    OutcomeRef::new(command.clone(), OutcomeName::new(name).unwrap())
}

impl ConformanceTarget for Jobs {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("adversary-pass2-jobs", "1"))
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
        let now = instant_ms(self.time.get());
        let result = if let Some((taken, declared)) = (self.rule)(&request.input, now) {
            SemanticCommandResult::took(outcome(&command, taken))
                .with_error(DeclaredErrorValue::new(declared.parse().unwrap()))
        } else {
            let id = Node::Text(format!("00000000-0000-4000-8000-{:012}", self.minted.get()));
            let mut row = Row::from([("job_id".to_owned(), id.clone())]);
            let mut event =
                ObservedEvent::new("demo.jobs.JobScheduled".parse().unwrap()).with("job_id", id);
            if let Some(starts_at) = request.input.get("starts_at") {
                row.insert("starts_at".to_owned(), starts_at.clone());
                event = event.with("starts_at", starts_at.clone());
            }
            self.rows.borrow_mut().push(row);
            SemanticCommandResult::took(outcome(&command, "scheduled")).emitting(event)
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

struct SharedWall {
    steps: AdvancingClock,
    time: Rc<Cell<u64>>,
}

impl Clock for SharedWall {
    fn now(&mut self) -> Timestamp {
        self.steps.now()
    }
    fn wall(&mut self) -> Timestamp {
        Timestamp::from_epoch_millis(self.time.get())
    }
}

/// 2026-09-27T12:00:00.250Z.
const WALL_MS: u64 = 1_790_510_400_250;

/// Synthesizes `text`, runs it against a correct service reading the runner's wall, and returns
/// the scenarios that do not pass and the synthesis refusals, with a readable detail.
fn run(text: &str, rule: Box<Rule>) -> (BTreeSet<String>, Vec<String>, String) {
    let synthesis = synthesize(&ir(text));
    let refusals: Vec<String> = synthesis
        .refusals
        .iter()
        .map(|refusal| format!("{refusal:?}"))
        .collect();
    let suite: ConformanceSuite = synthesis.suite;
    assert!(!suite.scenarios.is_empty(), "no scenarios: {refusals:#?}");
    let admitted = AdmittedSuite::from_suite(&suite).unwrap();
    let time = Rc::new(Cell::new(WALL_MS));
    let runner = Runner::new(
        RunnerConfig::default(),
        SharedWall {
            steps: AdvancingClock::default(),
            time: Rc::clone(&time),
        },
        Ids::for_suite(&suite),
    );
    let target = Jobs {
        time,
        rule,
        rows: RefCell::default(),
        minted: Cell::new(0),
    };
    let report = runner.run_admitted(&admitted, &target).into_report();
    let mut detail = String::new();
    let failed = report
        .scenarios
        .iter()
        .filter(|scenario| scenario.status != Status::Passed)
        .inspect(|scenario| writeln!(detail, "{scenario:#?}").unwrap())
        .map(|scenario| scenario.scenario.to_string())
        .collect();
    for (id, scenario) in &suite.scenarios {
        for step in &scenario.steps {
            if let ScenarioStep::ExecuteCommand { input, .. } = step {
                writeln!(detail, "sent {id}: {input:?}").unwrap();
            }
        }
    }
    (failed, refusals, detail)
}

fn text_instant(input: &BTreeMap<String, Node>, field: &str) -> Rfc3339Instant {
    match input.get(field) {
        Some(Node::Text(text)) => Rfc3339Instant::parse_rfc3339(text)
            .unwrap_or_else(|| panic!("{field} = {text} is no instant")),
        other => panic!("{field} is {other:?}"),
    }
}

fn instant(text: &str) -> Rfc3339Instant {
    Rfc3339Instant::parse_rfc3339(text).unwrap()
}

/// The #171 rule beside the ordinary companion a spec written in the last few years carries: no job
/// may start before the programme opened on 2025-01-01. Every fixed instant between the synthesis
/// reference (2019-12-30) and the run falls on the other side of `now` at run time than at the
/// reference, so the `scheduled` witness chosen at the fixed bound (2025-01-01, kept literal by the
/// pass-1 correction) is more than a minute in the past when sent, and a correct service refuses
/// it. The `start-in-past` branch, reachable at run time, is unsatisfiable at the reference.
///
/// Coordinator decision (correction round 2, item 1): no value can be decided the same at the
/// reference and at the run, so synthesis refuses the field by name — a `NoWitness` naming
/// `starts_at`, the fixed bound and `now` — and emits none of its scenarios.
#[test]
fn adversary_now_pass2_a_fixed_bound_between_the_reference_and_the_run_is_refused_by_name() {
    let text = edited(
        JOBS,
        "      - name: start-in-past\n",
        "      - name: too-early\n        when: starts_at < '2025-01-01T00:00:00Z'\n        error: demo.jobs.TooEarly\n      - name: start-in-past\n",
    );
    let text = edited(
        &text,
        "commands:\n",
        "  - name: demo.jobs.TooEarly\n    summary: The start is before the programme opened.\n    fields: []\ncommands:\n",
    );
    let synthesis = synthesize(&ir(&text));
    let refusals = format!("{:?}", synthesis.refusals);
    assert!(
        refusals.contains("NoWitness")
            && refusals.contains("starts_at")
            && refusals.contains("2025-01-01T00:00:00Z")
            && refusals.contains("`now`"),
        "correction round 2, item 1: a now-guarded field with a fixed bound after the synthesis \
         reference is refused by name: {refusals}"
    );
    assert!(
        synthesis.suite.scenarios.values().all(|scenario| !scenario
            .steps
            .iter()
            .any(|step| matches!(step, ScenarioStep::ExecuteCommand { .. }))),
        "correction round 2, item 1: no scenario sends the refused command: {:#?}",
        synthesis.suite.scenarios.keys().collect::<Vec<_>>()
    );
}

/// A sentinel instant written as an equality beside the #171 ordering on the same field. The
/// pass-1 correction collects only orderings (`op.needs_ordering()`), so the value chosen for the
/// equality is not recognised as a fixed instant's and is rewritten as a `now_offset` from the
/// 2019 reference: the sentinel is sent years after itself and a correct service takes another
/// branch.
#[test]
fn adversary_now_pass2_an_equality_with_a_fixed_instant_beside_now_is_sent_as_written() {
    let text = edited(
        JOBS,
        "      - name: start-in-past\n",
        "      - name: immediate\n        when: starts_at == '2024-06-01T00:00:00Z'\n        error: demo.jobs.Immediate\n      - name: start-in-past\n",
    );
    let text = edited(
        &text,
        "commands:\n",
        "  - name: demo.jobs.Immediate\n    summary: The sentinel start asks for an immediate run.\n    fields: []\ncommands:\n",
    );
    let rule: Box<Rule> = Box::new(|input, now| {
        let starts_at = text_instant(input, "starts_at");
        if starts_at == instant("2024-06-01T00:00:00Z") {
            Some(("immediate", "demo.jobs.Immediate"))
        } else if starts_at < now.plus_seconds(-60).unwrap() {
            Some(("start-in-past", "demo.jobs.StartInPast"))
        } else {
            None
        }
    });
    let (failed, refusals, detail) = run(&text, rule);
    assert_eq!(
        failed,
        BTreeSet::new(),
        "a correct service fails; refusals: {refusals:#?}\n{detail}"
    );
}

/// Admission bounds a `now_offset` at `now_offset::MAX_SECONDS` either way. `i64::MIN` has no
/// absolute value: the bound check negates it, which overflows (a panic under debug assertions,
/// and `i64::MIN` again — below the bound, so admitted — in a release build).
#[test]
fn adversary_now_pass2_an_offset_without_an_absolute_value_is_refused_without_panicking() {
    let synthesis = synthesize(&ir(JOBS));
    let json = synthesis.suite.to_canonical_json().unwrap();
    let needle = ["\"seconds\":-61", "\"seconds\": -61"]
        .into_iter()
        .find(|needle| json.contains(needle))
        .unwrap_or_else(|| panic!("the suite carries now_offset -61: {json}"));
    let hostile = json.replace(needle, "\"seconds\":-9223372036854775808");
    let admitted = std::panic::catch_unwind(|| AdmittedSuite::from_json(&hostile).is_ok());
    let admitted = admitted.unwrap_or_else(|_| {
        panic!("admission panics on an offset of i64::MIN seconds");
    });
    assert!(
        !admitted,
        "an offset of i64::MIN seconds is admitted past MAX_SECONDS"
    );
}
