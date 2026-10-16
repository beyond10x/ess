//! The interpreted target decides a `now` guard at the instant of the step the runner is executing
//! (<https://github.com/beyond10x/ess/issues/510>).
//!
//! [`Runner::command_clock`] hands a target a command clock over the runner's step instant: one
//! wall reading per step, rounded up to a whole second, which is also what that step's
//! `now_offset` values resolve against. The interpreter reads it once per decision. A target built
//! without it still answers such a decision `unsupported`, and the report says which guard.

use std::cell::Cell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::interpret::Interpreted;
use ess_conformance::now_offset::WithWall;
use ess_conformance::occurrence_clock::{CommandClock, DecisionInstant};
use ess_conformance::report::Status;
use ess_conformance::{
    synthesize::synthesize, AdmittedSuite, AdvancingClock, ConformanceSuite, CountRun, Ids, Runner,
    RunnerConfig, ScenarioStep, ScenarioValue,
};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_primitives::time::{Rfc3339Instant, Timestamp};

/// The issue's reproducer: `Present` is `expired` while `exp < now - 10m`, else `accepted`.
const TOKENS: &str = include_str!("fixtures/now-guard-interpreted.yaml");

/// 2026-10-04T12:00:00.250Z: a wall that is not on a whole second.
const WALL_MS: u64 = 1_791_115_200_250;

fn ir() -> EssIr {
    let raw = RawSpecFile::parse(TOKENS).unwrap_or_else(|error| panic!("{error}"));
    let specification = Specification::assemble([(Source::new("system.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("the fixture validates:\n{errors}"));
    compile(&specification, &SourceMap::new())
        .unwrap_or_else(|diagnostics| panic!("the fixture resolves:\n{diagnostics}"))
}

fn suite(ir: &EssIr) -> (ConformanceSuite, AdmittedSuite) {
    let mut synthesis = synthesize(ir);
    // The scenario on the boundary is exercised by the tests of `hold_now` below.
    synthesis
        .suite
        .scenarios
        .retain(|id, _| !id.to_string().contains("/held/"));
    assert_eq!(
        synthesis.suite.scenarios.len(),
        2,
        "one scenario per branch: {:#?}",
        synthesis.refusals
    );
    let admitted = AdmittedSuite::from_suite(&synthesis.suite).expect("the suite is admitted");
    (synthesis.suite, admitted)
}

/// A command clock counting its reads and keeping what each answered.
struct Counted<C> {
    inner: C,
    reads: Arc<Mutex<Vec<Option<DecisionInstant>>>>,
}

impl<C: CommandClock> CommandClock for Counted<C> {
    fn read(&self) -> Option<DecisionInstant> {
        let reading = self.inner.read();
        self.reads.lock().unwrap().push(reading);
        reading
    }
}

/// A wall answering `start_ms` and then `step_ms` later on every read, counting its reads.
fn wall(start_ms: u64, step_ms: u64, reads: &Rc<Cell<usize>>) -> impl FnMut() -> Timestamp {
    let reads = Rc::clone(reads);
    move || {
        let n = reads.get();
        reads.set(n + 1);
        Timestamp::from_epoch_millis(start_ms + step_ms * u64::try_from(n).unwrap())
    }
}

/// How many commands the suite sends: one decision each.
fn commands(suite: &ConformanceSuite) -> usize {
    suite
        .scenarios
        .values()
        .flat_map(|scenario| &scenario.steps)
        .filter(|step| matches!(step, ScenarioStep::ExecuteCommand { .. }))
        .count()
}

fn ceil(ms: u64) -> DecisionInstant {
    DecisionInstant::checked(
        Rfc3339Instant::from_epoch_millis(i64::try_from(ms).unwrap())
            .unwrap()
            .ceil_to_second(),
    )
    .unwrap()
}

/// A clock of the target's own, answering the same instant on every read.
struct Fixed;

impl CommandClock for Fixed {
    fn read(&self) -> Option<DecisionInstant> {
        Some(ceil(WALL_MS))
    }
}

#[test]
fn issue_510_the_step_instant_decides_every_scenario_once_per_decision() {
    let ir = ir();
    let (suite, admitted) = suite(&ir);
    let wall_reads = Rc::new(Cell::new(0));
    let mut runner = Runner::new(
        RunnerConfig::default(),
        WithWall::new(AdvancingClock::default(), wall(WALL_MS, 0, &wall_reads)),
        Ids::for_suite(&suite),
    );
    let reads = Arc::new(Mutex::new(Vec::new()));
    let target = Interpreted::for_model(ir).with_command_clock(Counted {
        inner: runner.command_clock(),
        reads: Arc::clone(&reads),
    });
    let report = runner.run_admitted(&admitted, &target).into_report();
    for scenario in &report.scenarios {
        assert_eq!(scenario.status, Status::Passed, "{scenario:#?}");
    }
    let commands = commands(&suite);
    let steps: usize = suite
        .scenarios
        .values()
        .map(|scenario| scenario.steps.len())
        .sum();
    // One reading per decision, each the step's instant: the wall rounded up to a whole second.
    assert_eq!(*reads.lock().unwrap(), vec![Some(ceil(WALL_MS)); commands]);
    // One wall reading per step.
    assert_eq!(wall_reads.get(), steps);
}

/// The decision follows the step: a wall that moves a day per reading gives every step another
/// instant, and each scenario still passes because its values and its decision share one.
#[test]
fn issue_510_each_step_decides_at_its_own_instant() {
    const DAY_MS: u64 = 86_400_000;
    let ir = ir();
    let (suite, admitted) = suite(&ir);
    let wall_reads = Rc::new(Cell::new(0));
    let mut runner = Runner::new(
        RunnerConfig::default(),
        WithWall::new(
            AdvancingClock::default(),
            wall(WALL_MS, DAY_MS, &wall_reads),
        ),
        Ids::for_suite(&suite),
    );
    let reads = Arc::new(Mutex::new(Vec::new()));
    let target = Interpreted::for_model(ir).with_command_clock(Counted {
        inner: runner.command_clock(),
        reads: Arc::clone(&reads),
    });
    let report = runner.run_admitted(&admitted, &target).into_report();
    for scenario in &report.scenarios {
        assert_eq!(scenario.status, Status::Passed, "{scenario:#?}");
    }
    let mut reads = reads.lock().unwrap().clone();
    assert_eq!(reads.len(), commands(&suite), "{reads:?}");
    reads.dedup();
    assert_eq!(
        reads.len(),
        commands(&suite),
        "every step its own instant: {reads:?}"
    );
}

/// A runner no command clock is taken from reads its wall only where a step names a `now_offset`
/// no earlier step fixed, as before the bridge existed.
#[test]
fn issue_510_a_runner_without_a_command_clock_reads_its_wall_as_before() {
    let ir = ir();
    let (suite, admitted) = suite(&ir);
    // The steps that name a `now_offset` no earlier step of their scenario fixed.
    let naming: usize = suite
        .scenarios
        .values()
        .map(|scenario| {
            let mut fixed = Vec::new();
            scenario
                .steps
                .iter()
                .filter(|step| match step {
                    ScenarioStep::ExecuteCommand { input, .. } => {
                        input.values().any(|value| match value {
                            ScenarioValue::NowOffset { seconds } if !fixed.contains(seconds) => {
                                fixed.push(*seconds);
                                true
                            }
                            _ => false,
                        })
                    }
                    _ => false,
                })
                .count()
        })
        .sum();
    let steps: usize = suite
        .scenarios
        .values()
        .map(|scenario| scenario.steps.len())
        .sum();
    assert!(naming < steps, "{naming} of {steps} steps name a new value");
    // The target decides by a clock of its own, so every step runs.
    let wall_reads = Rc::new(Cell::new(0));
    let runner = Runner::new(
        RunnerConfig::default(),
        WithWall::new(AdvancingClock::default(), wall(WALL_MS, 0, &wall_reads)),
        Ids::for_suite(&suite),
    );
    let report = runner
        .run_admitted(
            &admitted,
            &Interpreted::for_model(ir).with_command_clock(Fixed),
        )
        .into_report();
    for scenario in &report.scenarios {
        assert_eq!(scenario.status, Status::Passed, "{scenario:#?}");
    }
    assert_eq!(wall_reads.get(), naming);
}

/// A target with no command clock still answers the decision `unsupported`, and both the text and
/// the JSON report name the guard it could not decide.
#[test]
fn issue_510_an_unsupported_scenario_names_the_guard_in_text_and_json() {
    let ir = ir();
    let (suite, admitted) = suite(&ir);
    let run = Runner::for_suite(&suite).run_admitted(&admitted, &Interpreted::for_model(ir));
    let reason = "the guard of `demo.tok.Present/expired` is Unknown";
    let text = run.report().to_string();
    for scenario in &run.report().scenarios {
        assert_eq!(scenario.status, Status::Unsupported, "{scenario:#?}");
    }
    assert_eq!(text.matches(reason).count(), 2, "{text}");
    let json = CountRun::from_run(&run, &admitted)
        .expect("a count run")
        .to_canonical_json()
        .expect("canonical JSON");
    assert_eq!(json.matches(reason).count(), 2, "{json}");
}

#[test]
fn issue_510_every_clock_taken_from_one_runner_reads_the_same_instant() {
    let ir = ir();
    let (suite, admitted) = suite(&ir);
    let mut runner = Runner::new(
        RunnerConfig::default(),
        WithWall::new(
            AdvancingClock::default(),
            wall(WALL_MS, 0, &Rc::new(Cell::new(0))),
        ),
        Ids::for_suite(&suite),
    );
    let first = runner.command_clock();
    let second = runner.command_clock();
    assert_eq!(first.read(), None, "no step has run");
    let reads = Arc::new(Mutex::new(Vec::new()));
    let target = Interpreted::for_model(ir).with_command_clock(Counted {
        inner: first,
        reads: Arc::clone(&reads),
    });
    let report = runner.run_admitted(&admitted, &target).into_report();
    assert!(report
        .scenarios
        .iter()
        .all(|scenario| scenario.status == Status::Passed));
    assert_eq!(reads.lock().unwrap().len(), commands(&suite));
    assert_eq!(second.read(), Some(ceil(WALL_MS)));
}

// ---- a held `now` (beyond10x/ess#586) ------------------------------------------------------

/// The issue's reproducer: `Issue` stores a code's `expiry`, and `Redeem` takes it while
/// `expiry >= now`, else refuses it as expired. `{strict}` is the guard's comparison, so `>` is the
/// boundary mutant that refuses a code expiring exactly now.
const CODES: &str = r"format: ess/23
system: demo
version: v1
domain: demo.codes
types:
  - {name: demo.codes.CodeId, kind: newtype, of: Uuid}
entities:
  - name: demo.codes.Code
    identity: {name: code_id, type: demo.codes.CodeId}
    fields:
      - {name: expiry, type: Timestamp}
    lifecycle:
      initial: Issued
      states: [Issued, Redeemed]
      terminal: [Redeemed]
      transitions:
        - {name: redeem, from: [Issued], to: Redeemed}
errors:
  - {name: demo.codes.Expired, summary: The code expired., fields: []}
  - {name: demo.codes.Spent, summary: The code was redeemed already., fields: []}
  - {name: demo.codes.UnknownCode, summary: No code carries the identity., fields: []}
events:
  - name: demo.codes.Issued
    fields:
      - {name: code_id, type: demo.codes.CodeId}
  - name: demo.codes.Redeemed
    fields:
      - {name: code_id, type: demo.codes.CodeId}
commands:
  - name: demo.codes.Issue
    input:
      - {name: expiry, type: Timestamp}
    outcomes:
      - name: issued
        creates: demo.codes.Code
        instance: code_id
        sets: {expiry: input.expiry}
        emits: [demo.codes.Issued]
        payload:
          demo.codes.Issued: {code_id: {generated: true}}
  - name: demo.codes.Redeem
    input:
      - {name: code_id, type: demo.codes.CodeId}
    outcomes:
      - name: redeemed
        when_subject: {predicate: expiry {strict} now}
        moves: demo.codes.Code.redeem
        instance: code_id
        emits: [demo.codes.Redeemed]
        payload:
          demo.codes.Redeemed: {code_id: input.code_id}
      - name: expired
        error: demo.codes.Expired
      - name: spent
        wrong_state: true
        error: demo.codes.Spent
      - name: unknown-code
        unknown_instance: true
        error: demo.codes.UnknownCode
";

const REDEEMED: &str = "demo.codes.Redeem/outcome/redeemed";

fn codes(comparison: &str) -> EssIr {
    let text = CODES.replace("{strict}", comparison);
    let raw = RawSpecFile::parse(&text).unwrap_or_else(|error| panic!("{error}"));
    let specification = Specification::assemble([(Source::new("system.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("the reproducer validates:\n{errors}"));
    compile(&specification, &SourceMap::new())
        .unwrap_or_else(|diagnostics| panic!("the reproducer resolves:\n{diagnostics}"))
}

/// The suite of the correct model with its accepting scenario rewritten to the boundary: `Issue`
/// sends `expiry` as `now_offset 0`, and, when `held`, a `hold_now 0` stands before `Redeem`.
fn redeem_at_the_boundary(held: bool) -> ConformanceSuite {
    let mut suite = synthesize(&codes(">=")).suite;
    let (_, scenario) = suite
        .scenarios
        .iter_mut()
        .find(|(id, _)| id.to_string() == REDEEMED)
        .unwrap_or_else(|| panic!("a redeemed scenario"));
    for step in &mut scenario.steps {
        if let ScenarioStep::ExecuteCommand { command, input, .. } = step {
            if command.to_string() == "demo.codes.Issue" {
                input.insert("expiry".to_owned(), ScenarioValue::NowOffset { seconds: 0 });
            }
        }
    }
    if held {
        let at = scenario
            .steps
            .iter()
            .position(|step| {
                matches!(step, ScenarioStep::ExecuteCommand { command, .. }
                    if command.to_string() == "demo.codes.Redeem")
            })
            .expect("the command under test");
        scenario
            .steps
            .insert(at, ScenarioStep::HoldNow { seconds: 0 });
    }
    suite.select_fresh_format();
    suite
}

/// The redeemed scenario's verdict for `model` on the interpreted target over the runner's command
/// clock, under a wall that moves a day on every reading: unheld, `Redeem` is decided a day after
/// the `expiry` `Issue` stored.
fn redeemed(suite: &ConformanceSuite, model: EssIr) -> Status {
    const DAY_MS: u64 = 86_400_000;
    let admitted = AdmittedSuite::from_suite(suite).unwrap_or_else(|error| panic!("{error}"));
    let mut runner = Runner::new(
        RunnerConfig::default(),
        WithWall::new(
            AdvancingClock::default(),
            wall(WALL_MS, DAY_MS, &Rc::new(Cell::new(0))),
        ),
        Ids::for_suite(suite),
    );
    let target = Interpreted::for_model(model).with_command_clock(runner.command_clock());
    let report = runner.run_admitted(&admitted, &target).into_report();
    report
        .scenarios
        .iter()
        .find(|scenario| scenario.scenario.to_string() == REDEEMED)
        .map(|scenario| scenario.status)
        .expect("the redeemed scenario ran")
}

/// The interpreted target offers `hold_now` by construction: the runner writes the held instant into
/// the command clock it handed out, so `Redeem` is decided at the instant `Issue`'s `expiry` was
/// sent relative to, however far the wall has moved, and passes on the boundary.
#[test]
fn issue_586_the_interpreted_target_decides_a_held_command_at_the_held_instant() {
    let held = redeem_at_the_boundary(true);
    assert_eq!(
        held.provenance.suite_version.to_string(),
        "ess-conformance/50"
    );
    assert_eq!(redeemed(&held, codes(">=")), Status::Passed);
    // Unheld, the same boundary value is decided at the next step's instant, a day later.
    assert_eq!(
        redeemed(&redeem_at_the_boundary(false), codes(">=")),
        Status::Failed
    );
}

/// The held boundary kills the guard-boundary mutant, `expiry > now`, which refuses a code that
/// expires exactly now.
#[test]
fn issue_586_a_held_boundary_kills_the_strict_mutant_on_the_interpreted_target() {
    assert_eq!(
        redeemed(&redeem_at_the_boundary(true), codes(">")),
        Status::Failed
    );
}
