//! Adversary, pass 1, story:current-time-guard-operand (beyond10x/ess#171).
//!
//! Every case here runs a synthesized suite against a service that implements the specification
//! correctly, with its own clock reading the runner's wall clock (the harness the unit's own
//! `current_time_guard.rs` uses). A correct service must pass every scenario synthesis emits: a
//! scenario it fails is a suite claim the specification does not make.
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
    ConformanceSuite, Ids, Runner, RunnerConfig, ScenarioStep, ScenarioValue,
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

/// The rule a correct service applies to one request, given its clock: the outcome it takes and
/// the declared error it names, or `None` for `scheduled`.
type Rule = dyn Fn(&BTreeMap<String, Node>, Rfc3339Instant) -> Option<(&'static str, &'static str)>;

/// A job service whose clock is `time` (epoch ms, shared with the runner's wall), which advances
/// `handling_ms` after every command it handles.
struct Jobs {
    time: Rc<Cell<u64>>,
    handling_ms: u64,
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
        Ok(ImplementationIdentity::new("adversary-jobs", "1"))
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
        self.time.set(self.time.get() + self.handling_ms);
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

/// The runner's clock, whose wall is the shared time the service reads.
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

/// Synthesizes `text` and runs every emitted scenario against a correct service; the scenarios
/// that do not pass, with their reports.
fn failing(text: &str, rule: Box<Rule>, handling_ms: u64) -> (BTreeSet<String>, String) {
    let synthesis = synthesize(&ir(text));
    let suite: ConformanceSuite = synthesis.suite;
    assert!(
        !suite.scenarios.is_empty(),
        "synthesis emitted scenarios: {:#?}",
        synthesis.refusals
    );
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
        handling_ms,
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
    let sent: Vec<String> = suite
        .scenarios
        .iter()
        .flat_map(|(id, scenario)| {
            scenario.steps.iter().filter_map(move |step| match step {
                ScenarioStep::ExecuteCommand { input, .. } => Some(format!("{id}: {input:?}")),
                _ => None,
            })
        })
        .collect();
    writeln!(detail, "sent:\n{}", sent.join("\n")).unwrap();
    (failed, detail)
}

fn text_instant(input: &BTreeMap<String, Node>, field: &str) -> Rfc3339Instant {
    match input.get(field) {
        Some(Node::Text(text)) => Rfc3339Instant::parse_rfc3339(text)
            .unwrap_or_else(|| panic!("{field} = {text} is no instant")),
        other => panic!("{field} is {other:?}"),
    }
}

fn cutoff() -> Rfc3339Instant {
    Rfc3339Instant::parse_rfc3339("2030-01-01T00:00:00Z").unwrap()
}

/// The #171 rule beside a second, fixed-instant rule over the same input: no job may start after
/// the programme ends on 2030-01-01. At the service's clock (2026-09-27) both rules are
/// satisfiable, and the service below applies exactly what the specification says.
///
/// Synthesis rewrites every value it chose for `starts_at` as a `now_offset` from its 2019
/// reference, including the values it chose for the fixed 2030 bound — so the refuting boundary
/// row for `starts_at > "2030-01-01T00:00:00Z"`, written at the bound, is sent about seven years
/// after it, and a correct service takes `too-late` where the suite requires `scheduled`.
#[test]
fn adversary_now_a_fixed_instant_rule_beside_now_is_decided_as_written() {
    let text = edited(
        JOBS,
        "      - name: scheduled\n",
        "      - name: too-late\n        when: starts_at > '2030-01-01T00:00:00Z'\n        error: demo.jobs.TooLate\n      - name: scheduled\n",
    );
    let text = edited(
        &text,
        "commands:\n",
        "  - name: demo.jobs.TooLate\n    summary: The start is after the programme ends.\n    fields: []\ncommands:\n",
    );
    let rule: Box<Rule> = Box::new(|input, now| {
        let starts_at = text_instant(input, "starts_at");
        if starts_at < now.plus_seconds(-60).unwrap() {
            Some(("start-in-past", "demo.jobs.StartInPast"))
        } else if starts_at > cutoff() {
            Some(("too-late", "demo.jobs.TooLate"))
        } else {
            None
        }
    });
    let (failed, detail) = failing(&text, rule, 0);
    assert_eq!(failed, BTreeSet::new(), "{detail}");
}

/// The same pairing inside one guard: a start is refused when it is more than a minute in the past
/// or after the programme ends.
#[test]
fn adversary_now_a_fixed_instant_disjunct_beside_now_is_decided_as_written() {
    let text = edited(
        JOBS,
        "when: starts_at < now - 60s",
        "when: {any: [starts_at < now - 60s, starts_at > '2030-01-01T00:00:00Z']}",
    );
    let rule: Box<Rule> = Box::new(|input, now| {
        let starts_at = text_instant(input, "starts_at");
        (starts_at < now.plus_seconds(-60).unwrap() || starts_at > cutoff())
            .then_some(("start-in-past", "demo.jobs.StartInPast"))
    });
    let (failed, detail) = failing(&text, rule, 0);
    assert_eq!(failed, BTreeSet::new(), "{detail}");
}

/// A list of starts, refused when any of them is more than five minutes ahead. `ess-domain`
/// admits the operand in the quantifier body (the path is the binder `s`, a `Timestamp`), and
/// `now_offset::now_paths` returns the binder, which names no input field: nothing is rewritten
/// as a `now_offset` and nothing is refused, so every element is sent as a literal instant beside
/// the 2019 reference. At the service's clock every such instant is years in the past, so a
/// correct service accepts the list the suite requires it to refuse (and the direction a
/// `< now - 60s` body takes passes only because the stale instant happens to lie on its side).
#[test]
fn adversary_now_a_quantified_list_of_timestamps_is_witnessed_or_refused_by_name() {
    let text = edited(
        JOBS,
        "      - {name: starts_at, type: Timestamp}\n    outcomes:",
        "      - {name: starts, type: List<Timestamp>}\n    outcomes:",
    );
    let text = edited(
        &text,
        "when: starts_at < now - 60s",
        "when: {exists: {in: starts, as: s, that: s > now + 5m}}",
    );
    let text = edited(&text, "        sets: {starts_at: input.starts_at}\n", "");
    let text = edited(
        &text,
        "{job_id: {generated: true}, starts_at: input.starts_at}",
        "{job_id: {generated: true}}",
    );
    let text = edited(
        &text,
        "      - {name: job_id, type: demo.jobs.JobId}\n      - {name: starts_at, type: Timestamp}\nviews:",
        "      - {name: job_id, type: demo.jobs.JobId}\nviews:",
    );
    let synthesis = synthesize(&ir(&text));
    let refused_by_name = format!("{:?}", synthesis.refusals).contains("now_offset");
    let sends_command = synthesis.suite.scenarios.values().any(|scenario| {
        scenario.steps.iter().any(|step| {
            matches!(step, ScenarioStep::ExecuteCommand { command, .. }
                if command.to_string() == "demo.jobs.ScheduleJob")
        })
    });
    if refused_by_name && !sends_command {
        return;
    }
    let rule: Box<Rule> = Box::new(|input, now| {
        let limit = now.plus_seconds(300).unwrap();
        let Some(Node::Seq(starts)) = input.get("starts") else {
            panic!("starts is {:?}", input.get("starts"));
        };
        starts
            .iter()
            .any(|start| match start {
                Node::Text(text) => Rfc3339Instant::parse_rfc3339(text).unwrap() > limit,
                other => panic!("{other:?}"),
            })
            .then_some(("start-in-past", "demo.jobs.StartInPast"))
    });
    let (failed, detail) = failing(&text, rule, 0);
    assert_eq!(
        failed,
        BTreeSet::new(),
        "a correct service fails a synthesized scenario; refusals: {:?}\n{detail}",
        synthesis.refusals
    );
}

/// A service that takes a second and a half to handle each command. A `now_offset` is resolved
/// when the step first naming it runs; the design bounds the claim by the latency between that
/// resolution and the service reading its clock. A value resolved once and sent again by a later
/// step is sent after that bound has passed.
#[test]
fn adversary_now_a_slow_service_passes_when_every_send_is_freshly_resolved() {
    let rule: Box<Rule> = Box::new(|input, now| {
        (text_instant(input, "starts_at") < now.plus_seconds(-60).unwrap())
            .then_some(("start-in-past", "demo.jobs.StartInPast"))
    });
    let (failed, detail) = failing(JOBS, rule, 1_500);
    assert_eq!(failed, BTreeSet::new(), "{detail}");
    // And the property directly: no scenario sends one `now_offset` in two commands.
    let suite = synthesize(&ir(JOBS)).suite;
    for (id, scenario) in &suite.scenarios {
        let mut seen = BTreeSet::new();
        for step in &scenario.steps {
            if let ScenarioStep::ExecuteCommand { input, .. } = step {
                for value in input.values() {
                    if let ScenarioValue::NowOffset { seconds } = value {
                        assert!(
                            seen.insert(*seconds),
                            "{id} sends now_offset {seconds} in two commands"
                        );
                    }
                }
            }
        }
    }
}

/// The #171 rule written with the operand on the left, `now - 60s > starts_at`. The design keeps
/// it out of scope ("`now` on the left of an ordering"); either `ess-domain` refuses it, or what
/// it admits is witnessed so that a correct service passes.
#[test]
fn adversary_now_the_operand_on_the_left_is_refused_or_witnessed() {
    let text = edited(
        JOBS,
        "when: starts_at < now - 60s",
        "when: now - 60s > starts_at",
    );
    let admitted = RawSpecFile::parse(&text)
        .ok()
        .and_then(|raw| Specification::assemble([(Source::new("jobs.yaml"), raw)]).ok());
    if admitted.is_none() {
        return;
    }
    let rule: Box<Rule> = Box::new(|input, now| {
        (text_instant(input, "starts_at") < now.plus_seconds(-60).unwrap())
            .then_some(("start-in-past", "demo.jobs.StartInPast"))
    });
    let (failed, detail) = failing(&text, rule, 0);
    assert_eq!(failed, BTreeSet::new(), "{detail}");
}

/// Every ordering, at the smallest and the largest offsets the operand admits: a correct service
/// passes the suite synthesized for each.
#[test]
fn adversary_now_every_ordering_at_zero_and_the_largest_offset_is_witnessed() {
    let mut failures = Vec::new();
    for (op, spelled) in [("<", "lt"), ("<=", "le"), (">", "gt"), (">=", "ge")] {
        for (operand, seconds) in [
            ("now", 0_i64),
            ("now - 876600h", -3_155_760_000),
            ("now + 876600h", 3_155_760_000),
        ] {
            let guard = format!("when: starts_at {op} {operand}");
            let text = edited(JOBS, "when: starts_at < now - 60s", &guard);
            let op = op.to_owned();
            let rule: Box<Rule> = Box::new(move |input, now| {
                let starts_at = text_instant(input, "starts_at");
                let bound = now.plus_seconds(seconds).unwrap();
                let holds = match op.as_str() {
                    "<" => starts_at < bound,
                    "<=" => starts_at <= bound,
                    ">" => starts_at > bound,
                    _ => starts_at >= bound,
                };
                holds.then_some(("start-in-past", "demo.jobs.StartInPast"))
            });
            let (failed, detail) = failing(&text, rule, 0);
            if !failed.is_empty() {
                failures.push(format!("{guard} ({spelled}): {failed:?}\n{detail}"));
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// An `Optional` start, stored and read back: an absent start is not in the past.
#[test]
fn adversary_now_an_optional_start_is_witnessed() {
    let text = edited(
        JOBS,
        "      - {name: starts_at, type: Timestamp}\n    outcomes:",
        "      - {name: starts_at, type: Optional<Timestamp>}\n    outcomes:",
    );
    let text = text
        .replace(
            "    fields:\n      - {name: starts_at, type: Timestamp}\n    lifecycle",
            "    fields:\n      - {name: starts_at, type: Optional<Timestamp>}\n    lifecycle",
        )
        .replace(
            "      - {name: job_id, type: demo.jobs.JobId}\n      - {name: starts_at, type: Timestamp}\nviews:",
            "      - {name: job_id, type: demo.jobs.JobId}\n      - {name: starts_at, type: Optional<Timestamp>}\nviews:",
        )
        .replace(
            "      - {name: job_id, type: demo.jobs.JobId}\n      - {name: starts_at, type: Timestamp}\n",
            "      - {name: job_id, type: demo.jobs.JobId}\n      - {name: starts_at, type: Optional<Timestamp>}\n",
        );
    let rule: Box<Rule> = Box::new(|input, now| match input.get("starts_at") {
        Some(Node::Text(text)) => (Rfc3339Instant::parse_rfc3339(text).unwrap()
            < now.plus_seconds(-60).unwrap())
        .then_some(("start-in-past", "demo.jobs.StartInPast")),
        _ => None,
    });
    let (failed, detail) = failing(&text, rule, 0);
    assert_eq!(failed, BTreeSet::new(), "{detail}");
}
