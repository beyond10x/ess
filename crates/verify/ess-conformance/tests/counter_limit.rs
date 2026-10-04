//! A branch selected by a stored counter at its limit is witnessed (beyond10x/ess#226).
//!
//! `retries >= 3` on a task whose `RetryTask` raises `retries` by one is reached only by repeating
//! the raising command: the row a creation leaves holds `0`, and every row from `0` to `2` decides
//! the guard alike. The stored-row search follows the counter's value, so it repeats the command up
//! to the limit, and each side of the limit is witnessed — at the limit and one step before it — so
//! a target whose limit is off by one either way fails the suite. A limit beyond the search's bound
//! is refused as `ESS-SYNTH-003` naming the bound.
//!
//! The suites are run against a hand-written target answering the model, which must pass every
//! scenario, and against targets whose limit is moved one step either way, each of which must fail.
mod support_versions;

use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;

use ess_compiler::{ir::EssIr, refs::OutcomeRef, resolve::compile, source::SourceMap};
use ess_conformance::{
    report::{ConformanceStatus, Status},
    scenario::{ScenarioId, ScenarioStep, ScenarioValue},
    synthesize::{synthesize, Synthesis},
    target::*,
    AdmittedSuite, ConformanceScenario, Runner,
};
use ess_domain::{
    command::OutcomeName,
    spec::{RawSpecFile, Specification},
    system::Source,
};
use ess_primitives::{consistency::ConsistencyToken, node::Node};

/// A task holding a counter, created at `{start}`, and the view every scenario reads it through.
/// `{commands}` is the commands raising the counter and reading it.
const MODEL: &str = r"
format: ess/18
system: work
version: v1
domain: work.tasks
types:
  - {name: work.tasks.TaskId, kind: newtype, of: Uuid}
entities:
  - name: work.tasks.Task
    identity: {name: task_id, type: work.tasks.TaskId}
    fields:
      - {name: label, type: String}
      - {name: retries, type: Integer}
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
errors:
  - {name: work.tasks.Exhausted, fields: []}
  - {name: work.tasks.Blocked, fields: []}
events:
  - name: work.tasks.TaskCreated
    fields: [{name: task_id, type: work.tasks.TaskId}]
  - name: work.tasks.TaskRetried
    fields: [{name: task_id, type: work.tasks.TaskId}]
  - name: work.tasks.TaskStarted
    fields: [{name: task_id, type: work.tasks.TaskId}]
commands:
  - name: work.tasks.CreateTask
    input: [{name: label, type: String}]
    outcomes:
      - name: created
        creates: work.tasks.Task
        instance: task_id
        emits: [work.tasks.TaskCreated]
        payload:
          work.tasks.TaskCreated: {task_id: {generated: true}}
        sets: {label: input.label, retries: {start}}
{commands}
views:
  - name: work.tasks.Tasks
    source: work.tasks.Task
    consistency: read_your_writes
    fields:
      - {name: task_id, type: work.tasks.TaskId}
      - {name: state, type: work.tasks.Task.State}
      - {name: label, type: String}
      - {name: retries, type: Integer}
";

/// `RetryTask` refuses at the limit and otherwise moves the counter by `{step}`: the guarded command
/// is the one raising the counter.
const RETRY_GUARDED: &str = r"
  - name: work.tasks.RetryTask
    input:
      - {name: task_id, type: work.tasks.TaskId}
{max}
    outcomes:
      - name: at-limit
        when_subject: {predicate: {guard}}
        error: work.tasks.Exhausted
      - name: retried
        updates: work.tasks.Task
        instance: task_id
        emits: [work.tasks.TaskRetried]
        payload:
          work.tasks.TaskRetried: {task_id: input.task_id}
        sets: {retries: {increment: {step}}}
";

/// `RetryTask` always moves the counter; `StartTask` is the command refusing at the limit.
const START_GUARDED: &str = r"
  - name: work.tasks.RetryTask
    input:
      - {name: task_id, type: work.tasks.TaskId}
    outcomes:
      - name: retried
        updates: work.tasks.Task
        instance: task_id
        emits: [work.tasks.TaskRetried]
        payload:
          work.tasks.TaskRetried: {task_id: input.task_id}
        sets: {retries: {increment: {step}}}
  - name: work.tasks.StartTask
    input:
      - {name: task_id, type: work.tasks.TaskId}
      - {name: label, type: String}
{max}
    outcomes:
      - name: blocked
        when_subject: {predicate: {guard}}
        error: work.tasks.Blocked
      - name: started
        updates: work.tasks.Task
        instance: task_id
        emits: [work.tasks.TaskStarted]
        payload:
          work.tasks.TaskStarted: {task_id: input.task_id}
        sets: {label: input.label}
";

const MAX_INPUT: &str = "      - {name: max, type: Integer}";

const AT_LIMIT: &str = "work.tasks.RetryTask/outcome/at-limit";
const RETRIED: &str = "work.tasks.RetryTask/outcome/retried";
const BLOCKED: &str = "work.tasks.StartTask/outcome/blocked";
const STARTED: &str = "work.tasks.StartTask/outcome/started";

/// One counter model: which command reads the limit, the guard, where the counter starts and by
/// how much each raise moves it, and whether the command takes a `max` input.
#[derive(Debug, Clone, Copy)]
struct Shape {
    commands: &'static str,
    guard: &'static str,
    start: i64,
    step: i64,
    max: bool,
}

impl Shape {
    const fn retry(guard: &'static str) -> Self {
        Self {
            commands: RETRY_GUARDED,
            guard,
            start: 0,
            step: 1,
            max: false,
        }
    }

    const fn start(guard: &'static str) -> Self {
        Self {
            commands: START_GUARDED,
            guard,
            start: 0,
            step: 1,
            max: false,
        }
    }

    fn text(self) -> String {
        let commands = self
            .commands
            .replace("{guard}", self.guard)
            .replace("{step}", &self.step.to_string())
            .replace("{max}\n", if self.max { MAX_INPUT } else { "" })
            .replace(MAX_INPUT, &format!("{MAX_INPUT}\n"));
        MODEL
            .replace("{start}", &self.start.to_string())
            .replace("{commands}\n", commands.trim_start_matches('\n'))
    }
}

fn compiled(text: &str) -> Synthesis {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("tasks.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    let ir: EssIr = compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"));
    synthesize(&ir)
}

fn refusals(synthesis: &Synthesis) -> Vec<String> {
    synthesis
        .refusals
        .iter()
        .map(|refusal| format!("{}: {refusal}", refusal.cause.code()))
        .collect()
}

fn scenario<'a>(synthesis: &'a Synthesis, id: &str) -> &'a ConformanceScenario {
    synthesis
        .suite
        .scenarios
        .get(&ScenarioId::parse(id).unwrap())
        .unwrap_or_else(|| panic!("no scenario {id}: {:#?}", refusals(synthesis)))
}

/// How many times a scenario sends `name` for a task it created: a branch naming no subject of its
/// own is first sent for a task nobody created, which is not counted.
fn sent(scenario: &ConformanceScenario, name: &str) -> usize {
    scenario
        .steps
        .iter()
        .filter(|step| {
            matches!(step, ScenarioStep::ExecuteCommand { command, input, .. }
                if command.to_string() == name
                    && matches!(input.get("task_id"), Some(ScenarioValue::Instance { .. })))
        })
        .count()
}

/// Whether the limit holds for a task holding `retries`, with the `max` the command was sent.
type Limit = fn(i64, Option<i64>) -> bool;

/// The task store, implemented here and not by the synthesizer, reading its limit through `limit`.
struct Tasks {
    start: i64,
    step: i64,
    retry_guarded: bool,
    /// Whether a task labelled `frozen` is refused whatever its counter: the guard's other disjunct.
    frozen: bool,
    limit: Limit,
    rows: RefCell<Vec<BTreeMap<String, Node>>>,
    minted: Cell<u64>,
}

impl Tasks {
    fn new(shape: Shape, limit: Limit) -> Self {
        Self {
            start: shape.start,
            step: shape.step,
            retry_guarded: shape.commands.contains("name: at-limit"),
            frozen: shape.guard.contains("frozen"),
            limit,
            rows: RefCell::default(),
            minted: Cell::new(0),
        }
    }

    fn token(&self) -> ConsistencyToken {
        ConsistencyToken::new(format!("seq:{}", self.minted.get())).unwrap()
    }
}

fn integer(node: &Node) -> i64 {
    let Node::Number(number) = node else {
        panic!("an integer: {node:?}")
    };
    let value = number.get();
    assert!(value.fract() == 0.0, "an integer: {node:?}");
    #[allow(clippy::cast_possible_truncation)]
    let whole = value as i64;
    whole
}

fn number(value: i64) -> Node {
    serde_json::from_str::<Node>(&value.to_string()).unwrap()
}

impl ConformanceTarget for Tasks {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("tasks-fixture", "1"))
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
        let command = request.command.clone();
        let took = |name: &str| {
            SemanticCommandResult::took(OutcomeRef::new(
                command.clone(),
                OutcomeName::new(name).unwrap(),
            ))
        };
        let event = |name: &str, id: Node| {
            ObservedEvent::new(format!("work.tasks.{name}").parse().unwrap()).with("task_id", id)
        };
        let max = request.input.get("max").map(integer);
        let result = match command.to_string().as_str() {
            "work.tasks.CreateTask" => {
                self.minted.set(self.minted.get() + 1);
                let id = Node::Text(format!("00000000-0000-4000-8000-{:012}", self.minted.get()));
                let mut row = BTreeMap::new();
                row.insert("task_id".to_owned(), id.clone());
                row.insert("state".to_owned(), Node::Text("Open".into()));
                row.insert("label".to_owned(), request.input["label"].clone());
                row.insert("retries".to_owned(), number(self.start));
                self.rows.borrow_mut().push(row);
                took("created").emitting(event("TaskCreated", id))
            }
            name @ ("work.tasks.RetryTask" | "work.tasks.StartTask") => {
                let id = request.input["task_id"].clone();
                let mut rows = self.rows.borrow_mut();
                let Some(row) = rows.iter_mut().find(|row| row["task_id"] == id) else {
                    return Ok(SemanticCommandResult::undeclared().with_consistency(self.token()));
                };
                let retries = integer(&row["retries"]);
                if name == "work.tasks.RetryTask" {
                    let frozen = self.frozen && row["label"] == Node::Text("frozen".into());
                    if self.retry_guarded && ((self.limit)(retries, max) || frozen) {
                        took("at-limit").with_error(DeclaredErrorValue::new(
                            "work.tasks.Exhausted".parse().unwrap(),
                        ))
                    } else {
                        row.insert("retries".to_owned(), number(retries + self.step));
                        took("retried").emitting(event("TaskRetried", id))
                    }
                } else if (self.limit)(retries, max) {
                    took("blocked").with_error(DeclaredErrorValue::new(
                        "work.tasks.Blocked".parse().unwrap(),
                    ))
                } else {
                    row.insert("label".to_owned(), request.input["label"].clone());
                    took("started").emitting(event("TaskStarted", id))
                }
            }
            other => return Err(TargetError::unsupported("command", other)),
        };
        Ok(result.with_consistency(self.token()))
    }
    fn query_view(&self, _: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        Ok(SemanticViewResult::of(self.rows.borrow().clone()))
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

/// The synthesis of `shape`, which must refuse nothing.
fn clean(shape: Shape) -> Synthesis {
    let synthesis = compiled(&shape.text());
    assert!(
        synthesis.refusals.is_empty(),
        "{:#?}\n{}",
        refusals(&synthesis),
        shape.text()
    );
    synthesis
}

/// Every scenario of the suite of `shape` that does not pass against a target reading `limit`.
fn failing(shape: Shape, limit: Limit) -> BTreeMap<String, String> {
    let suite = clean(shape).suite;
    let admitted = AdmittedSuite::from_suite(&suite).unwrap();
    let report = Runner::for_suite(&suite)
        .run_admitted(&admitted, &Tasks::new(shape, limit))
        .into_report();
    let failed: BTreeMap<String, String> = report
        .scenarios
        .iter()
        .filter(|scenario| scenario.status != Status::Passed)
        .map(|scenario| {
            (
                scenario.scenario.to_string(),
                format!("{:#?}", scenario.diagnostics().collect::<Vec<_>>()),
            )
        })
        .collect();
    assert_eq!(
        report.status == ConformanceStatus::Passed,
        failed.is_empty(),
        "{report:?}"
    );
    failed
}

/// The target as specified passes; each off-by-one target fails the scenario of the branch on the
/// side of the limit it moved: a limit moved toward the counter's start fails the other branch's
/// scenario, which is witnessed one step before the limit, and one moved away fails the limit's.
fn decides_the_limit(shape: Shape, right: Limit, wrong: &[(&str, Limit, &str)]) {
    let passed = failing(shape, right);
    assert!(passed.is_empty(), "{shape:?}: {passed:#?}");
    let mut survived = Vec::new();
    for (name, limit, expected) in wrong {
        let failed = failing(shape, *limit);
        if !failed.contains_key(*expected) {
            survived.push(format!(
                "{name}: {expected} passed; failed {:?}",
                failed.keys()
            ));
        }
    }
    assert!(survived.is_empty(), "{shape:?}\n{}", survived.join("\n"));
}

#[test]
fn a_refusal_at_a_counter_limit_is_synthesized_by_repeating_the_raise() {
    let synthesis = clean(Shape::retry("retries >= 3"));
    let at_limit = scenario(&synthesis, AT_LIMIT);
    // Three raises before the refused one: the row holds 3.
    assert_eq!(sent(at_limit, "work.tasks.RetryTask"), 4, "{at_limit:#?}");
}

/// The branch on the other side of the limit is witnessed one raise before it, so a target whose
/// limit is one lower fails; the refusal at the limit fails a target whose limit is one higher.
#[test]
fn a_target_with_the_limit_off_by_one_fails() {
    decides_the_limit(
        Shape::retry("retries >= 3"),
        |retries, _| retries >= 3,
        &[
            ("limit 2", |retries, _| retries >= 2, RETRIED),
            ("limit 4", |retries, _| retries >= 4, AT_LIMIT),
            ("no limit", |_, _| false, AT_LIMIT),
        ],
    );
    let synthesis = clean(Shape::retry("retries >= 3"));
    assert!(
        synthesis
            .suite
            .scenarios
            .contains_key(&ScenarioId::parse(RETRIED).unwrap()),
        "the other side is witnessed"
    );
}

/// A limit that is one disjunct of an `any:`: the refusal is witnessed at the limit with the other
/// disjunct refuted, and the default one step short of it with both refuted, so a target whose
/// limit is off by one fails even though a frozen label alone already selects the refusal.
#[test]
fn a_limit_beside_another_disjunct_is_witnessed_on_both_sides() {
    let shape = Shape::retry(r#"{any: [retries >= 3, label == "frozen"]}"#);
    let synthesis = clean(shape);
    let at_limit = scenario(&synthesis, AT_LIMIT);
    assert!(
        sent(at_limit, "work.tasks.RetryTask") >= 4,
        "the limit is reached by raising: {at_limit:#?}"
    );
    decides_the_limit(
        shape,
        |retries, _| retries >= 3,
        &[
            ("limit 2", |retries, _| retries >= 2, RETRIED),
            ("limit 4", |retries, _| retries >= 4, AT_LIMIT),
        ],
    );
}

#[test]
fn a_strict_limit_is_witnessed_on_both_sides() {
    decides_the_limit(
        Shape::retry("retries > 2"),
        |retries, _| retries > 2,
        &[
            ("limit > 1", |retries, _| retries > 1, RETRIED),
            ("limit > 3", |retries, _| retries > 3, AT_LIMIT),
            ("limit >= 2", |retries, _| retries >= 2, RETRIED),
        ],
    );
}

/// `retries == 3` refuses only at 3: the raise before it (at 2) is witnessed beside the refusal.
#[test]
fn an_equality_limit_is_witnessed_on_both_sides() {
    decides_the_limit(
        Shape::retry("retries == 3"),
        |retries, _| retries == 3,
        &[
            ("limit == 2", |retries, _| retries == 2, RETRIED),
            ("limit == 4", |retries, _| retries == 4, AT_LIMIT),
            ("limit >= 2", |retries, _| retries >= 2, RETRIED),
        ],
    );
}

/// A counter lowered toward its limit: created at 3, each raise moves it by -1, refused at 0.
#[test]
fn a_decrement_toward_the_limit_is_witnessed_on_both_sides() {
    let shape = Shape {
        start: 3,
        step: -1,
        ..Shape::retry("retries <= 0")
    };
    decides_the_limit(
        shape,
        |retries, _| retries <= 0,
        &[
            ("limit <= 1", |retries, _| retries <= 1, RETRIED),
            ("limit <= -1", |retries, _| retries <= -1, AT_LIMIT),
        ],
    );
}

/// A counter lowered by two from 5 toward `retries <= 0` holds 5, 3, 1, -1: the rows either side of
/// the limit are the ones a run holds (1 and -1), not the literal one step either side (1 and 0,
/// or 2), so a target whose limit is one off either way still fails.
#[test]
fn an_odd_decrement_toward_the_limit_pins_rows_the_run_holds() {
    let shape = Shape {
        start: 5,
        step: -2,
        ..Shape::retry("retries <= 0")
    };
    decides_the_limit(
        shape,
        |retries, _| retries <= 0,
        &[
            ("limit <= 1", |retries, _| retries <= 1, RETRIED),
            ("limit <= -2", |retries, _| retries <= -2, AT_LIMIT),
        ],
    );
}

/// `retries == 20` raised by twenty from 0: the row past the limit is 40, farther from it than the
/// search follows the counter. That row alone is refused, naming the step and the limit; the
/// default's own witness at 0 and the refusal's at 20 stand.
#[test]
fn a_side_past_the_reach_refuses_only_that_row() {
    let synthesis = compiled(
        &Shape {
            step: 20,
            ..Shape::start("retries == 20")
        }
        .text(),
    );
    scenario(&synthesis, STARTED);
    scenario(&synthesis, BLOCKED);
    let refused = refusals(&synthesis);
    assert!(!refused.is_empty(), "the row at 40 is not witnessed");
    assert!(
        refused
            .iter()
            .all(|refusal| refusal.starts_with("ESS-SYNTH-003")
                && refusal.contains("work.tasks.StartTask/started")
                && refusal.contains("moves by 20")
                && refusal.contains("limit 20")
                && refusal.contains("within 16")),
        "{refused:#?}"
    );
}

/// `retries == 3` raised by two from 0 never holds 3, and the raise is unbounded: the search leaves
/// every row within reach of the limit and the rows past it only move farther off, so the refusal
/// of `blocked` does not claim the counter's bound, and `started` is witnessed with nothing refused.
#[test]
fn an_unreachable_limit_under_an_unbounded_raise_does_not_claim_the_bound() {
    let synthesis = compiled(
        &Shape {
            step: 2,
            ..Shape::start("retries == 3")
        }
        .text(),
    );
    scenario(&synthesis, STARTED);
    let refused = refusals(&synthesis);
    assert!(!refused.is_empty(), "no run holds 3");
    assert!(
        refused
            .iter()
            .all(|refusal| refusal.contains("work.tasks.StartTask/blocked")
                && !refusal.contains("within 16")),
        "{refused:#?}"
    );
}

/// A band written as a `not:` over an `any:` (`3 <= retries <= 10`): the `not:` is pushed onto
/// both comparisons, so each limit of the band has the rows its positive form has — 2 and 11
/// answered by `started` — and a target with either end one off fails.
#[test]
fn a_band_under_not_is_witnessed_at_both_ends() {
    decides_the_limit(
        Shape::start("{not: {any: [retries < 3, retries > 10]}}"),
        |retries, _| (3..=10).contains(&retries),
        &[
            ("lower 2", |retries, _| (2..=10).contains(&retries), STARTED),
            ("lower 4", |retries, _| (4..=10).contains(&retries), BLOCKED),
            ("upper 9", |retries, _| (3..=9).contains(&retries), BLOCKED),
            (
                "upper 11",
                |retries, _| (3..=11).contains(&retries),
                STARTED,
            ),
        ],
    );
}

/// The counter is raised by `RetryTask` and read by `StartTask`.
#[test]
fn a_counter_raised_by_another_command_is_witnessed_on_both_sides() {
    let synthesis = clean(Shape::start("retries >= 3"));
    let blocked = scenario(&synthesis, BLOCKED);
    assert_eq!(sent(blocked, "work.tasks.RetryTask"), 3, "{blocked:#?}");
    scenario(&synthesis, STARTED);
    decides_the_limit(
        Shape::start("retries >= 3"),
        |retries, _| retries >= 3,
        &[
            ("limit 2", |retries, _| retries >= 2, STARTED),
            ("limit 4", |retries, _| retries >= 4, BLOCKED),
        ],
    );
}

/// A limit the command's input names: the counter is raised to the value the input is sent at.
#[test]
fn a_limit_compared_with_an_input_is_witnessed_on_both_sides() {
    let shape = Shape {
        max: true,
        ..Shape::start("retries >= input.max")
    };
    decides_the_limit(
        shape,
        |retries, max| retries >= max.expect("max sent"),
        &[
            (
                "limit max - 1",
                |retries, max| retries >= max.expect("max sent") - 1,
                STARTED,
            ),
            (
                "limit max + 1",
                |retries, max| retries > max.expect("max sent"),
                BLOCKED,
            ),
        ],
    );
}

/// A limit farther from where the counter starts than the search follows it is refused as
/// `ESS-SYNTH-003`, naming the bound — never dropped.
#[test]
fn a_limit_beyond_the_bound_is_refused_naming_the_bound() {
    let synthesis = compiled(&Shape::retry("retries >= 1000").text());
    let refused = refusals(&synthesis);
    let at_limit: Vec<&String> = refused
        .iter()
        .filter(|refusal| refusal.contains("work.tasks.RetryTask/at-limit"))
        .collect();
    assert!(
        at_limit
            .iter()
            .any(|refusal| refusal.starts_with("ESS-SYNTH-003")
                && refusal.contains("counter")
                && refusal.contains("within 16")),
        "{refused:#?}"
    );
}

/// A counter on the owner row another command reads through `when_related:` (ess/18): a project's
/// `open_cards`, raised by `ReserveSlot`, and `AddCard` refusing once it reaches the limit.
const BOARD: &str = r"
format: ess/18
system: work
version: v1
domain: work.board
types:
  - {name: work.board.ProjectId, kind: newtype, of: Uuid}
  - {name: work.board.CardId, kind: newtype, of: Uuid}
entities:
  - name: work.board.Project
    identity: {name: project_id, type: work.board.ProjectId}
    fields:
      - {name: open_cards, type: Integer}
    lifecycle: {initial: Active, states: [Active], terminal: [Active]}
  - name: work.board.Card
    identity: {name: card_id, type: work.board.CardId}
    fields:
      - {name: project_id, type: work.board.ProjectId}
    lifecycle: {initial: Added, states: [Added], terminal: [Added]}
errors:
  - {name: work.board.NoProject, fields: []}
  - {name: work.board.BoardFull, fields: []}
events:
  - name: work.board.ProjectOpened
    fields: [{name: project_id, type: work.board.ProjectId}]
  - name: work.board.SlotReserved
    fields: [{name: project_id, type: work.board.ProjectId}]
  - name: work.board.CardAdded
    fields: [{name: card_id, type: work.board.CardId}]
commands:
  - name: work.board.OpenProject
    input: []
    outcomes:
      - name: opened
        creates: work.board.Project
        instance: project_id
        emits: [work.board.ProjectOpened]
        payload:
          work.board.ProjectOpened: {project_id: {generated: true}}
        sets: {open_cards: 0}
  - name: work.board.ReserveSlot
    input:
      - {name: project_id, type: work.board.ProjectId}
    outcomes:
      - name: reserved
        updates: work.board.Project
        instance: project_id
        emits: [work.board.SlotReserved]
        payload:
          work.board.SlotReserved: {project_id: input.project_id}
        sets: {open_cards: {increment: 1}}
  - name: work.board.AddCard
    input:
      - {name: project_id, type: work.board.ProjectId}
    outcomes:
      - name: no-project
        when_related: {via: input.project_id, exists: false}
        error: work.board.NoProject
      - name: full
        when_related: {via: input.project_id, predicate: open_cards >= 3}
        error: work.board.BoardFull
      - name: added
        creates: work.board.Card
        instance: card_id
        emits: [work.board.CardAdded]
        payload:
          work.board.CardAdded: {card_id: {generated: true}}
        sets: {project_id: input.project_id}
views:
  - name: work.board.Projects
    source: work.board.Project
    consistency: read_your_writes
    fields:
      - {name: project_id, type: work.board.ProjectId}
      - {name: open_cards, type: Integer}
  - name: work.board.Cards
    source: work.board.Card
    consistency: read_your_writes
    fields:
      - {name: card_id, type: work.board.CardId}
      - {name: project_id, type: work.board.ProjectId}
";

const FULL: &str = "work.board.AddCard/outcome/full";
const ADDED: &str = "work.board.AddCard/outcome/added";

/// The board, implemented here, reading its limit through `limit`.
struct Board {
    limit: fn(i64) -> bool,
    projects: RefCell<Vec<(Node, i64)>>,
    cards: RefCell<Vec<(Node, Node)>>,
    minted: Cell<u64>,
}

impl Board {
    fn mint(&self) -> Node {
        self.minted.set(self.minted.get() + 1);
        Node::Text(format!("00000000-0000-4000-8000-{:012}", self.minted.get()))
    }
}

impl ConformanceTarget for Board {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("board-fixture", "1"))
    }
    fn begin_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        self.projects.replace(Vec::new());
        self.cards.replace(Vec::new());
        Ok(())
    }
    fn end_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        Ok(())
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        let command = request.command.clone();
        let took = |name: &str| {
            SemanticCommandResult::took(OutcomeRef::new(
                command.clone(),
                OutcomeName::new(name).unwrap(),
            ))
        };
        let event = |name: &str, field: &str, id: Node| {
            ObservedEvent::new(format!("work.board.{name}").parse().unwrap()).with(field, id)
        };
        let error =
            |name: &str| DeclaredErrorValue::new(format!("work.board.{name}").parse().unwrap());
        let result = match command.to_string().as_str() {
            "work.board.OpenProject" => {
                let id = self.mint();
                self.projects.borrow_mut().push((id.clone(), 0));
                took("opened").emitting(event("ProjectOpened", "project_id", id))
            }
            "work.board.ReserveSlot" => {
                let id = request.input["project_id"].clone();
                let mut projects = self.projects.borrow_mut();
                let Some(row) = projects.iter_mut().find(|(project, _)| *project == id) else {
                    return Ok(SemanticCommandResult::undeclared());
                };
                row.1 += 1;
                took("reserved").emitting(event("SlotReserved", "project_id", id))
            }
            "work.board.AddCard" => {
                let id = request.input["project_id"].clone();
                let open = self
                    .projects
                    .borrow()
                    .iter()
                    .find(|(project, _)| *project == id)
                    .map(|(_, open)| *open);
                match open {
                    None => took("no-project").with_error(error("NoProject")),
                    Some(open) if (self.limit)(open) => took("full").with_error(error("BoardFull")),
                    Some(_) => {
                        let card = self.mint();
                        self.cards.borrow_mut().push((card.clone(), id));
                        took("added").emitting(event("CardAdded", "card_id", card))
                    }
                }
            }
            other => return Err(TargetError::unsupported("command", other)),
        };
        let token = ConsistencyToken::new(format!("seq:{}", self.minted.get())).unwrap();
        Ok(result.with_consistency(token))
    }
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        let rows: Vec<BTreeMap<String, Node>> = match request.view.to_string().as_str() {
            "work.board.Projects" => self
                .projects
                .borrow()
                .iter()
                .map(|(id, open)| {
                    BTreeMap::from([
                        ("project_id".to_owned(), id.clone()),
                        ("open_cards".to_owned(), number(*open)),
                    ])
                })
                .collect(),
            "work.board.Cards" => self
                .cards
                .borrow()
                .iter()
                .map(|(card, project)| {
                    BTreeMap::from([
                        ("card_id".to_owned(), card.clone()),
                        ("project_id".to_owned(), project.clone()),
                    ])
                })
                .collect(),
            other => panic!("no view {other}"),
        };
        Ok(SemanticViewResult::of(rows))
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

/// The scenarios of the board's suite that do not pass against a board reading `limit`.
fn board_failing(limit: fn(i64) -> bool) -> BTreeMap<String, String> {
    let synthesis = compiled(BOARD);
    assert!(synthesis.refusals.is_empty(), "{:#?}", refusals(&synthesis));
    let suite = synthesis.suite;
    let admitted = AdmittedSuite::from_suite(&suite).unwrap();
    let board = Board {
        limit,
        projects: RefCell::default(),
        cards: RefCell::default(),
        minted: Cell::new(0),
    };
    let report = Runner::for_suite(&suite)
        .run_admitted(&admitted, &board)
        .into_report();
    report
        .scenarios
        .iter()
        .filter(|scenario| scenario.status != Status::Passed)
        .map(|scenario| {
            (
                scenario.scenario.to_string(),
                format!("{:#?}", scenario.diagnostics().collect::<Vec<_>>()),
            )
        })
        .collect()
}

/// The owner's counter is raised to the limit for the refusal, and — as a further row of the
/// refusal's scenario, the related-row boundaries' layout (beyond10x/ess#211) — held one step short
/// of it for a card that is added, so a board whose limit is off by one either way fails that
/// scenario. The arrangement's raises read no limit, so only the further row fails a lower one.
#[test]
fn a_counter_on_an_owner_row_is_witnessed_on_both_sides() {
    let synthesis = compiled(BOARD);
    assert!(synthesis.refusals.is_empty(), "{:#?}", refusals(&synthesis));
    let full = scenario(&synthesis, FULL);
    let reserved = full
        .steps
        .iter()
        .filter(|step| {
            matches!(step, ScenarioStep::ExecuteCommand { command, .. }
                if command.to_string() == "work.board.ReserveSlot")
        })
        .count();
    assert!(reserved >= 3, "{full:#?}");
    let added = ADDED.replace("/outcome/", "/");
    assert!(
        full.steps.iter().any(|step| matches!(step,
            ScenarioStep::ExpectOutcome { outcome } if outcome.to_string() == added)),
        "the further row is answered by `added`: {full:#?}"
    );
    let passed = board_failing(|open| open >= 3);
    assert!(passed.is_empty(), "{passed:#?}");
    let mut survived = Vec::new();
    for (name, limit, expected) in [
        ("limit 2", (|open| open >= 2) as fn(i64) -> bool, FULL),
        ("limit 4", |open| open >= 4, FULL),
        ("no limit", |_| false, FULL),
    ] {
        let failed = board_failing(limit);
        if !failed.contains_key(expected) {
            survived.push(format!(
                "{name}: {expected} passed; failed {:?}",
                failed.keys()
            ));
        }
    }
    assert!(survived.is_empty(), "{}", survived.join("\n"));
}

/// An owner's counter limit beyond the bound is refused naming the bound, as the subject's is.
#[test]
fn an_owner_limit_beyond_the_bound_is_refused_naming_the_bound() {
    let text = BOARD.replace("open_cards >= 3", "open_cards >= 1000");
    assert_ne!(text, BOARD);
    let refused = refusals(&compiled(&text));
    assert!(
        refused
            .iter()
            .any(|refusal| refusal.starts_with("ESS-SYNTH-003")
                && refusal.contains("work.board.AddCard/full")
                && refusal.contains("within 16")),
        "{refused:#?}"
    );
}

// ESS413A's finite control is deliberately ordinary command arrangement, not a state seed.
fn counter413a_cas_model() -> String {
    Shape::retry("retries >= 2")
        .text()
        .replace("errors:\n", "errors:\n  - {name: work.tasks.Stale, fields: []}\n")
        .replace(
            "      - {name: task_id, type: work.tasks.TaskId}\n    outcomes:",
            "      - {name: task_id, type: work.tasks.TaskId}\n      - {name: expected_revision, type: Integer}\n    outcomes:",
        )
        .replace(
            "    outcomes:\n      - name: at-limit",
            "    outcomes:\n      - name: stale\n        when_subject: {predicate: retries != input.expected_revision}\n        error: work.tasks.Stale\n      - name: at-limit",
        )
}

#[derive(Debug)]
struct Counter413aCall {
    scenario: String,
    command: String,
    subject: Option<Node>,
    expected: Option<i64>,
    before: Option<i64>,
    after: Option<i64>,
    outcome: Option<String>,
}

struct Counter413aCas {
    tasks: Tasks,
    ignore_cas: bool,
    scenario: RefCell<String>,
    calls: RefCell<Vec<Counter413aCall>>,
}
impl Counter413aCas {
    fn new(limit: Limit, ignore_cas: bool) -> Self {
        Self {
            tasks: Tasks::new(Shape::retry("retries >= 2"), limit),
            ignore_cas,
            scenario: RefCell::default(),
            calls: RefCell::default(),
        }
    }
}
impl ConformanceTarget for Counter413aCas {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        self.tasks.identity()
    }
    fn begin_scenario(&self, context: &ScenarioContext) -> Result<(), TargetError> {
        self.scenario.replace(context.scenario.to_string());
        self.tasks.begin_scenario(context)
    }
    fn end_scenario(&self, context: &ScenarioContext) -> Result<(), TargetError> {
        self.tasks.end_scenario(context)
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        let command = request.command.clone();
        let subject = request.input.get("task_id").cloned();
        let expected = request.input.get("expected_revision").map(integer);
        let before = self
            .tasks
            .rows
            .borrow()
            .iter()
            .find(|row| subject.as_ref() == Some(&row["task_id"]))
            .map(|row| integer(&row["retries"]));
        let result = if command.to_string() == "work.tasks.RetryTask"
            && before.is_some()
            && before != expected
            && !self.ignore_cas
        {
            SemanticCommandResult::took(OutcomeRef::new(
                command.clone(),
                OutcomeName::new("stale").unwrap(),
            ))
            .with_error(DeclaredErrorValue::new("work.tasks.Stale".parse().unwrap()))
            .with_consistency(self.tasks.token())
        } else {
            self.tasks.execute_command(request)?
        };
        let rows = self.tasks.rows.borrow();
        let held = if subject.is_some() {
            rows.iter()
                .find(|row| subject.as_ref() == Some(&row["task_id"]))
        } else {
            rows.last()
        };
        self.calls.borrow_mut().push(Counter413aCall {
            scenario: self.scenario.borrow().clone(),
            command: command.to_string(),
            subject: held.map(|row| row["task_id"].clone()),
            expected,
            before,
            after: held.map(|row| integer(&row["retries"])),
            outcome: result.outcome.as_ref().map(ToString::to_string),
        });
        Ok(result)
    }
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        self.tasks.query_view(request)
    }
    fn observe_events(
        &self,
        request: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        self.tasks.observe_events(request)
    }
    fn configure_external_outcome(
        &self,
        request: ExternalOutcomeControl,
    ) -> Result<(), TargetError> {
        self.tasks.configure_external_outcome(request)
    }
    fn redeliver_event(&self, request: RedeliveryRequest) -> Result<(), TargetError> {
        self.tasks.redeliver_event(request)
    }
}

#[test]
fn counter413a_finite_two_cas_executes_both_advances_stale_and_exhaustion_without_setup() {
    let synthesis = compiled(&counter413a_cas_model());
    assert!(
        synthesis.refusals.is_empty(),
        "finite CAS must be independently classified if refused: {:#?}",
        refusals(&synthesis)
    );
    assert!(
        synthesis
            .suite
            .scenarios
            .values()
            .flat_map(|scenario| &scenario.steps)
            .all(|step| !matches!(step, ScenarioStep::EstablishEntity { .. })),
        "finite counter arrangement must use ordinary commands, not entity setup"
    );
    let admitted = AdmittedSuite::from_suite(&synthesis.suite).unwrap();
    let target = Counter413aCas::new(|revision, _| revision >= 2, false);
    let report = Runner::for_suite(&synthesis.suite)
        .run_admitted(&admitted, &target)
        .into_report();
    assert_eq!(report.status, ConformanceStatus::Passed, "{report:#?}");
    let calls = target.calls.borrow();
    assert!(
        calls.windows(3).any(|triple| {
            triple.iter().all(|call| {
                call.scenario == triple[0].scenario && call.subject == triple[0].subject
            }) && triple[0].command == "work.tasks.CreateTask"
                && triple[0].after == Some(0)
                && triple[1].command == "work.tasks.RetryTask"
                && triple[1].expected == Some(0)
                && triple[1].before == Some(0)
                && triple[1].after == Some(1)
                && triple[2].command == "work.tasks.RetryTask"
                && triple[2].expected == Some(1)
                && triple[2].before == Some(1)
                && triple[2].after == Some(2)
        }),
        "same ordinary instance must actually advance 0 then 1 and hold 2: {calls:#?}"
    );
    assert!(
        calls.iter().any(
            |call| call.outcome.as_deref() == Some("work.tasks.RetryTask/stale")
                && call.before.is_some()
                && call.before != call.expected
                && call.before == call.after
        ),
        "stale must execute without changing revision: {calls:#?}"
    );
    assert!(
        calls.iter().any(
            |call| call.outcome.as_deref() == Some("work.tasks.RetryTask/at-limit")
                && call.expected == Some(2)
                && call.before == Some(2)
                && call.after == Some(2)
        ),
        "exhaustion at exact revision 2 must execute without mutation: {calls:#?}"
    );
    drop(calls);
    for (name, limit, ignore_cas, required_failure) in [
        (
            "CAS ignored",
            (|revision, _| revision >= 2) as Limit,
            true,
            "work.tasks.RetryTask/outcome/stale",
        ),
        (
            "limit one early",
            (|revision, _| revision >= 1) as Limit,
            false,
            RETRIED,
        ),
        (
            "limit one late",
            (|revision, _| revision >= 3) as Limit,
            false,
            AT_LIMIT,
        ),
    ] {
        let bad = Counter413aCas::new(limit, ignore_cas);
        let report = Runner::for_suite(&synthesis.suite)
            .run_admitted(&admitted, &bad)
            .into_report();
        assert!(
            report
                .scenarios
                .iter()
                .any(|case| case.scenario.to_string() == required_failure
                    && case.status != Status::Passed),
            "{name} survived required scenario {required_failure}: {report:#?}"
        );
    }
}

#[test]
fn counter413a_ordinary_and_shared_related_controls_keep_deterministic_suite_bytes() {
    use sha2::{Digest as _, Sha256};
    use std::fmt::Write as _;
    for (name, text) in [
        ("ordinary-two", Shape::retry("retries >= 2").text()),
        ("shared-related-three", BOARD.to_owned()),
    ] {
        let first = compiled(&text);
        let second = compiled(&text);
        assert!(first.refusals.is_empty(), "{name}: {:#?}", refusals(&first));
        let bytes = first.suite.to_canonical_json().unwrap();
        assert_eq!(bytes, second.suite.to_canonical_json().unwrap());
        let mut hash = String::with_capacity(64);
        for byte in Sha256::digest(bytes.as_bytes()) {
            write!(&mut hash, "{byte:02x}").unwrap();
        }
        println!("counter413a canonical {name} sha256={hash}");
        assert!(first
            .suite
            .scenarios
            .values()
            .flat_map(|scenario| &scenario.steps)
            .all(|step| !matches!(step, ScenarioStep::EstablishEntity { .. })));
    }
    // The full counter_limit target also runs its unchanged healthy, off-by-one and shared
    // related-counter execution controls; these digests supplement those semantic assertions.
}

// Independent public API probe from the #413A final review.
// Independent public API probe: all preceding author cases are retained byte-for-byte.
#[test]
fn independent_extreme_counter_refusals_do_not_claim_a_complete_nearest_value() {
    for (name, shape, limit) in [
        (
            "upper",
            Shape::retry("retries >= 9223372036854775807"),
            (|value, _| value == i64::MAX) as Limit,
        ),
        (
            "lower",
            Shape {
                step: -1,
                ..Shape::retry("retries <= -9223372036854775808")
            },
            (|value, _| value == i64::MIN) as Limit,
        ),
    ] {
        let synthesis = compiled(&shape.text());
        let refused = refusals(&synthesis);
        assert!(
            !refused.is_empty(),
            "{name}: extreme state must not become fake coverage"
        );
        assert!(
            refused.iter().all(|reason| !reason.contains("nearest value it holds")),
            "{name}: incomplete arithmetic must not publish a completeness-derived nearest-value claim: {refused:#?}"
        );
        assert!(
            synthesis
                .suite
                .scenarios
                .values()
                .flat_map(|scenario| &scenario.steps)
                .all(|step| !matches!(step, ScenarioStep::EstablishEntity { .. })),
            "{name}: ordinary arrangement must not be replaced by a seed"
        );
        // A refusal is not a blanket removal of useful ordinary scenarios.
        scenario(&synthesis, RETRIED);
        let admitted =
            AdmittedSuite::from_suite(&synthesis.suite).expect("current suite admission");
        let target = Tasks::new(shape, limit);
        let report = Runner::for_suite(&synthesis.suite)
            .run_admitted(&admitted, &target)
            .into_report();
        assert_eq!(
            report.status,
            ConformanceStatus::Passed,
            "{name}: ordinary executable scenarios regressed: {report:#?}"
        );
        println!(
            "independent extreme {name}: refused={} scenarios={}",
            refused.len(),
            synthesis.suite.scenarios.len()
        );
    }
}

// #413A final review cases: extreme counters moving away from MAX/MIN, a literal past i64,
// a shared related counter at MAX, and the recorded canonical digests.

/// Every "the nearest value it holds <side> the limit <literal> is <value>" claim in a refusal, as
/// `(side, literal, value)`.
fn review2_nearest_claims(refused: &[String]) -> Vec<(String, String, String)> {
    const MARK: &str = "the nearest value it holds ";
    let mut claims = Vec::new();
    for refusal in refused {
        let mut rest = refusal.as_str();
        while let Some(at) = rest.find(MARK) {
            rest = &rest[at + MARK.len()..];
            let Some((side, tail)) = rest.split_once(" the limit ") else {
                break;
            };
            let Some((literal, tail)) = tail.split_once(" is ") else {
                break;
            };
            let value: String = tail
                .chars()
                .take_while(|c| *c == '-' || c.is_ascii_digit() || *c == '.')
                .collect();
            claims.push((side.to_owned(), literal.to_owned(), value));
        }
    }
    claims
}

fn review2_no_entity_setup(synthesis: &Synthesis) -> bool {
    synthesis
        .suite
        .scenarios
        .values()
        .flat_map(|scenario| &scenario.steps)
        .all(|step| !matches!(step, ScenarioStep::EstablishEntity { .. }))
}

fn review2_run<T: ConformanceTarget>(synthesis: &Synthesis, target: &T) -> ConformanceStatus {
    let admitted = AdmittedSuite::from_suite(&synthesis.suite).expect("suite admission");
    Runner::for_suite(&synthesis.suite)
        .run_admitted(&admitted, target)
        .into_report()
        .status
}

/// The two canonical digests the story records as the unchanged baseline
/// (`ordinary-two 6bf91d8f…`, `shared-related-three 8f56c982…`), asserted as literals. The unit's
/// own control only compares two compilations by the same code with each other, so it stays green
/// under any deterministic change to those bytes.
#[test]
fn review2_ordinary_and_shared_related_digests_equal_the_recorded_baseline() {
    use sha2::{Digest as _, Sha256};
    use std::fmt::Write as _;
    for (name, text, expected) in [
        (
            "ordinary-two",
            Shape::retry("retries >= 2").text(),
            "6bf91d8f43da8103fa8e0f0a9898dc9002b292934f8a60d494769c72d0826c5e",
        ),
        (
            "shared-related-three",
            BOARD.to_owned(),
            "8f56c982d3834b1eb7a3290a7a460d79fd2ca8c7fc4d821ad6a5df1748c96958",
        ),
    ] {
        // The bytes as recorded, before beyond10x/ess#273 compared the captured identities events
        // carry: every other byte is still held to the baseline.
        let fresh = compiled(&text).suite.to_canonical_json().unwrap();
        let bytes = serde_json::from_str::<ess_conformance::ConformanceSuite>(
            &support_versions::without_captured_identities(&fresh),
        )
        .unwrap()
        .to_canonical_json()
        .unwrap();
        let mut hash = String::with_capacity(64);
        for byte in Sha256::digest(bytes.as_bytes()) {
            write!(&mut hash, "{byte:02x}").unwrap();
        }
        assert_eq!(hash, expected, "{name}: canonical suite bytes moved");
    }
}

/// A counter that only moves down from 0 never holds anything at or above `i64::MAX`. Every
/// nearest-value claim a refusal makes must name a value the counter holds (at most 0), and the
/// ordinary scenarios must still pass a healthy target.
#[test]
fn review2_decrement_away_from_max_claims_only_values_it_holds() {
    let shape = Shape {
        step: -1,
        ..Shape::retry("retries >= 9223372036854775807")
    };
    let synthesis = compiled(&shape.text());
    let refused = refusals(&synthesis);
    println!("review2 away-from-max refusals: {refused:#?}");
    for (side, literal, value) in review2_nearest_claims(&refused) {
        let held: i128 = value.parse().unwrap_or(i128::MAX);
        assert!(
            held <= 0,
            "claims {value} is the nearest value held {side} {literal}, but the counter only \
             holds values <= 0: {refused:#?}"
        );
    }
    assert!(review2_no_entity_setup(&synthesis));
    scenario(&synthesis, RETRIED);
    let target = Tasks::new(shape, |value, _| value == i64::MAX);
    assert_eq!(review2_run(&synthesis, &target), ConformanceStatus::Passed);
}

/// The at-limit branch of the same counter is unreachable because the counter moves away from the
/// limit, not because the search stopped at its reach; the refusal must not blame the reach
/// (`a_unreachable_limit_under_an_unbounded_raise_does_not_claim_the_bound` is the same rule).
#[test]
fn review2_decrement_away_from_max_does_not_blame_the_reach() {
    let shape = Shape {
        step: -1,
        ..Shape::retry("retries >= 9223372036854775807")
    };
    let refused = refusals(&compiled(&shape.text()));
    assert!(
        refused
            .iter()
            .filter(|refusal| refusal.contains("work.tasks.RetryTask/at-limit"))
            .all(|refusal| !refusal.contains("within 16")),
        "no run holds MAX; the reach is not the cause: {refused:#?}"
    );
}

/// The mirror at `i64::MIN`: a counter that only moves up from 0.
#[test]
fn review2_increment_away_from_min_claims_only_values_it_holds() {
    let shape = Shape::retry("retries <= -9223372036854775808");
    let synthesis = compiled(&shape.text());
    let refused = refusals(&synthesis);
    println!("review2 away-from-min refusals: {refused:#?}");
    for (side, literal, value) in review2_nearest_claims(&refused) {
        let held: i128 = value.parse().unwrap_or(i128::MIN);
        assert!(
            held >= 0,
            "claims {value} is the nearest value held {side} {literal}, but the counter only \
             holds values >= 0: {refused:#?}"
        );
    }
    assert!(review2_no_entity_setup(&synthesis));
    scenario(&synthesis, RETRIED);
    let target = Tasks::new(shape, |value, _| value == i64::MIN);
    assert_eq!(review2_run(&synthesis, &target), ConformanceStatus::Passed);
}

#[test]
fn review2_increment_away_from_min_does_not_blame_the_reach() {
    let refused = refusals(&compiled(
        &Shape::retry("retries <= -9223372036854775808").text(),
    ));
    assert!(
        refused
            .iter()
            .filter(|refusal| refusal.contains("work.tasks.RetryTask/at-limit"))
            .all(|refusal| !refusal.contains("within 16")),
        "no run holds MIN; the reach is not the cause: {refused:#?}"
    );
}

/// A literal one past `i64::MAX`: both windows are unrepresentable. No refusal may claim the
/// counter's nearest value below it is a small number it reached by stopping early.
#[test]
fn review2_literal_past_i64_makes_no_completeness_claim() {
    let text = Shape::retry("retries >= 9223372036854775808").text();
    let Ok(raw) = RawSpecFile::parse(&text) else {
        return;
    };
    let Ok(spec) = Specification::assemble([(Source::new("tasks.yaml"), raw)]) else {
        return;
    };
    let Ok(ir) = compile(&spec, &SourceMap::new()) else {
        return;
    };
    let synthesis = synthesize(&ir);
    let refused = refusals(&synthesis);
    println!("review2 past-i64 refusals: {refused:#?}");
    let claims = review2_nearest_claims(&refused);
    assert!(
        claims.is_empty(),
        "a counter stepping by 1 from 0 has no complete nearest value below 2^63 within the \
         search: {claims:#?}"
    );
    assert!(review2_no_entity_setup(&synthesis));
    let target = Tasks::new(Shape::retry("retries >= 9223372036854775808"), |_, _| false);
    assert_eq!(review2_run(&synthesis, &target), ConformanceStatus::Passed);
}

/// The shared related-counter path (`when_related:`) at `i64::MAX`: the owner's counter, raised by
/// one from 0, must not be described as holding 1 as its nearest value below the limit.
#[test]
fn review2_related_owner_counter_at_max_makes_no_completeness_claim() {
    let text = BOARD.replace("open_cards >= 3", "open_cards >= 9223372036854775807");
    assert_ne!(text, BOARD);
    let synthesis = compiled(&text);
    let refused = refusals(&synthesis);
    println!("review2 related-max refusals: {refused:#?}");
    assert!(
        refused
            .iter()
            .any(|refusal| refusal.contains("work.board.AddCard/full")),
        "the extreme owner state must be refused, not covered: {refused:#?}"
    );
    let claims = review2_nearest_claims(&refused);
    assert!(
        claims.is_empty(),
        "no complete nearest value exists below MAX within the search: {claims:#?}"
    );
    assert!(review2_no_entity_setup(&synthesis));
    let board = Board {
        limit: |open| open == i64::MAX,
        projects: RefCell::default(),
        cards: RefCell::default(),
        minted: Cell::new(0),
    };
    assert_eq!(review2_run(&synthesis, &board), ConformanceStatus::Passed);
}
