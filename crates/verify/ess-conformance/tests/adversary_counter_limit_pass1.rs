//! Adversary pass 1 for beyond10x/ess#226 (a branch selected by a stored counter at its limit).
//!
//! Each case builds a model whose counter is raised by `RetryTask` (never guarded) and read by
//! `StartTask`, so the arranging raises never read the limit and only the synthesized rows can
//! decide it. A hand-written target answers the model; a mutant target moves one comparison. Every
//! mutant here is distinguishable on a row the model reaches, so a clean suite must fail it.
#![allow(
    clippy::too_many_lines,
    clippy::type_complexity,
    clippy::single_match_else
)]

use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;

use ess_compiler::{ir::EssIr, refs::OutcomeRef, resolve::compile, source::SourceMap};
use ess_conformance::{
    report::{ConformanceStatus, Status},
    synthesize::{synthesize, Synthesis},
    target::*,
    AdmittedSuite, Runner,
};
use ess_domain::{
    command::OutcomeName,
    spec::{RawSpecFile, Specification},
    system::Source,
};
use ess_primitives::{consistency::ConsistencyToken, node::Node};

/// `{start}` is where `retries` is created, `{step}` what `RetryTask` moves it by, `{outcomes}` the
/// guarded branches of `StartTask` before its default `started`.
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
      - {name: restarts, type: Integer}
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
errors:
  - {name: work.tasks.Blocked, fields: []}
  - {name: work.tasks.Over, fields: []}
events:
  - name: work.tasks.TaskCreated
    fields: [{name: task_id, type: work.tasks.TaskId}]
  - name: work.tasks.TaskRetried
    fields: [{name: task_id, type: work.tasks.TaskId}]
  - name: work.tasks.TaskRestarted
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
        sets: {label: input.label, retries: START, restarts: 0}
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
        sets: {retries: {increment: STEP}}
  - name: work.tasks.RestartTask
    input:
      - {name: task_id, type: work.tasks.TaskId}
    outcomes:
      - name: restarted
        updates: work.tasks.Task
        instance: task_id
        emits: [work.tasks.TaskRestarted]
        payload:
          work.tasks.TaskRestarted: {task_id: input.task_id}
        sets: {restarts: {increment: 1}}
  - name: work.tasks.StartTask
    input:
      - {name: task_id, type: work.tasks.TaskId}
      - {name: label, type: String}
    outcomes:
OUTCOMES
      - name: started
        updates: work.tasks.Task
        instance: task_id
        emits: [work.tasks.TaskStarted]
        payload:
          work.tasks.TaskStarted: {task_id: input.task_id}
        sets: {label: input.label}
views:
  - name: work.tasks.Tasks
    source: work.tasks.Task
    consistency: read_your_writes
    fields:
      - {name: task_id, type: work.tasks.TaskId}
      - {name: state, type: work.tasks.Task.State}
      - {name: label, type: String}
      - {name: retries, type: Integer}
      - {name: restarts, type: Integer}
";

fn model(start: i64, step: i64, outcomes: &str) -> String {
    MODEL
        .replace("START", &start.to_string())
        .replace("STEP", &step.to_string())
        .replace("OUTCOMES", outcomes.trim_matches('\n'))
}

/// One guarded error branch of `StartTask`.
fn branch(name: &str, error: &str, guard: &str) -> String {
    format!(
        "      - name: {name}\n        when_subject: {{predicate: {guard}}}\n        error: work.tasks.{error}\n"
    )
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

/// How `StartTask` decides for a task holding `(retries, restarts)`: the error branch and its error,
/// or `None` for `started`.
type Decide = fn(i64, i64) -> Option<(&'static str, &'static str)>;

struct Tasks {
    start: i64,
    step: i64,
    decide: Decide,
    rows: RefCell<Vec<BTreeMap<String, Node>>>,
    minted: Cell<u64>,
}

fn integer(node: &Node) -> i64 {
    let Node::Number(number) = node else {
        panic!("an integer: {node:?}")
    };
    #[allow(clippy::cast_possible_truncation)]
    let whole = number.get() as i64;
    whole
}

fn number(value: i64) -> Node {
    serde_json::from_str::<Node>(&value.to_string()).unwrap()
}

impl ConformanceTarget for Tasks {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("tasks-adversary", "1"))
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
        let result = match command.to_string().as_str() {
            "work.tasks.CreateTask" => {
                self.minted.set(self.minted.get() + 1);
                let id = Node::Text(format!("00000000-0000-4000-8000-{:012}", self.minted.get()));
                let mut row = BTreeMap::new();
                row.insert("task_id".to_owned(), id.clone());
                row.insert("state".to_owned(), Node::Text("Open".into()));
                row.insert("label".to_owned(), request.input["label"].clone());
                row.insert("retries".to_owned(), number(self.start));
                row.insert("restarts".to_owned(), number(0));
                self.rows.borrow_mut().push(row);
                took("created").emitting(event("TaskCreated", id))
            }
            name => {
                let id = request.input["task_id"].clone();
                let mut rows = self.rows.borrow_mut();
                let Some(row) = rows.iter_mut().find(|row| row["task_id"] == id) else {
                    self.minted.set(self.minted.get() + 1);
                    return Ok(SemanticCommandResult::undeclared().with_consistency(
                        ConsistencyToken::new(format!("seq:{}", self.minted.get())).unwrap(),
                    ));
                };
                let retries = integer(&row["retries"]);
                let restarts = integer(&row["restarts"]);
                match name {
                    "work.tasks.RetryTask" => {
                        row.insert("retries".to_owned(), number(retries + self.step));
                        took("retried").emitting(event("TaskRetried", id))
                    }
                    "work.tasks.RestartTask" => {
                        row.insert("restarts".to_owned(), number(restarts + 1));
                        took("restarted").emitting(event("TaskRestarted", id))
                    }
                    "work.tasks.StartTask" => match (self.decide)(retries, restarts) {
                        Some((outcome, error)) => took(outcome).with_error(
                            DeclaredErrorValue::new(format!("work.tasks.{error}").parse().unwrap()),
                        ),
                        None => {
                            row.insert("label".to_owned(), request.input["label"].clone());
                            took("started").emitting(event("TaskStarted", id))
                        }
                    },
                    other => return Err(TargetError::unsupported("command", other)),
                }
            }
        };
        self.minted.set(self.minted.get() + 1);
        Ok(result
            .with_consistency(ConsistencyToken::new(format!("seq:{}", self.minted.get())).unwrap()))
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

/// The scenarios of the suite for `text` failing against a target deciding with `decide`.
fn failing(text: &str, start: i64, step: i64, decide: Decide) -> Vec<String> {
    let synthesis = compiled(text);
    assert!(
        synthesis.refusals.is_empty(),
        "{:#?}\n{text}",
        refusals(&synthesis)
    );
    let suite = synthesis.suite;
    let admitted = AdmittedSuite::from_suite(&suite).unwrap();
    let target = Tasks {
        start,
        step,
        decide,
        rows: RefCell::default(),
        minted: Cell::new(0),
    };
    let report = Runner::for_suite(&suite)
        .run_admitted(&admitted, &target)
        .into_report();
    let failed: Vec<String> = report
        .scenarios
        .iter()
        .filter(|scenario| scenario.status != Status::Passed)
        .map(|scenario| scenario.scenario.to_string())
        .collect();
    assert_eq!(
        report.status == ConformanceStatus::Passed,
        failed.is_empty()
    );
    failed
}

/// The model's own target passes; every mutant (each distinguishable on a reached row) fails.
fn kills(text: &str, start: i64, step: i64, right: Decide, mutants: &[(&str, Decide)]) {
    let passed = failing(text, start, step, right);
    assert!(passed.is_empty(), "the faithful target failed: {passed:?}");
    let survived: Vec<&str> = mutants
        .iter()
        .filter(|(_, decide)| failing(text, start, step, *decide).is_empty())
        .map(|(name, _)| *name)
        .collect();
    assert!(survived.is_empty(), "mutants survived: {survived:?}");
}

/// `{increment: 2}` from 0 against `retries >= 3`: rows 0, 2, 4, … . The row one step short of the
/// limit is `2`, but the side row pinned is `limit - step = 1`, which no run holds, so it is dropped
/// and `started` is witnessed only at 0. A target blocking from 1 or 2 differs at the reached row 2.
#[test]
fn an_even_step_toward_an_odd_limit_witnesses_the_reached_row_short_of_it() {
    let text = model(0, 2, &branch("blocked", "Blocked", "retries >= 3"));
    kills(
        &text,
        0,
        2,
        |retries, _| (retries >= 3).then_some(("blocked", "Blocked")),
        &[
            ("limit >= 2", |retries, _| {
                (retries >= 2).then_some(("blocked", "Blocked"))
            }),
            ("limit >= 1", |retries, _| {
                (retries >= 1).then_some(("blocked", "Blocked"))
            }),
        ],
    );
}

/// `retries == 3` read by a command that does not raise it: the row past the limit (4) is reached by
/// one more raise, and a target reading `>= 3` differs there.
#[test]
fn an_equality_limit_read_by_another_command_witnesses_the_row_past_it() {
    let text = model(0, 1, &branch("blocked", "Blocked", "retries == 3"));
    kills(
        &text,
        0,
        1,
        |retries, _| (retries == 3).then_some(("blocked", "Blocked")),
        &[
            ("limit >= 3", |retries, _| {
                (retries >= 3).then_some(("blocked", "Blocked"))
            }),
            ("limit == 2", |retries, _| {
                (retries == 2).then_some(("blocked", "Blocked"))
            }),
            ("limit == 4", |retries, _| {
                (retries == 4).then_some(("blocked", "Blocked"))
            }),
        ],
    );
}

/// Every comparison operator, read by a command that does not raise the counter, each against the
/// two off-by-one mutants of its literal.
#[test]
fn every_operator_read_by_another_command_is_decided_on_both_sides() {
    let cases: [(&str, Decide, [(&str, Decide); 2]); 4] = [
        (
            "retries > 2",
            |r, _| (r > 2).then_some(("blocked", "Blocked")),
            [
                ("> 1", |r, _| (r > 1).then_some(("blocked", "Blocked"))),
                ("> 3", |r, _| (r > 3).then_some(("blocked", "Blocked"))),
            ],
        ),
        (
            "retries < 3",
            |r, _| (r < 3).then_some(("blocked", "Blocked")),
            [
                ("< 2", |r, _| (r < 2).then_some(("blocked", "Blocked"))),
                ("< 4", |r, _| (r < 4).then_some(("blocked", "Blocked"))),
            ],
        ),
        (
            "retries <= 2",
            |r, _| (r <= 2).then_some(("blocked", "Blocked")),
            [
                ("<= 1", |r, _| (r <= 1).then_some(("blocked", "Blocked"))),
                ("<= 3", |r, _| (r <= 3).then_some(("blocked", "Blocked"))),
            ],
        ),
        (
            "retries != 3",
            |r, _| (r != 3).then_some(("blocked", "Blocked")),
            [
                ("!= 2", |r, _| (r != 2).then_some(("blocked", "Blocked"))),
                ("!= 4", |r, _| (r != 4).then_some(("blocked", "Blocked"))),
            ],
        ),
    ];
    let mut survived = Vec::new();
    for (guard, right, mutants) in cases {
        let text = model(0, 1, &branch("blocked", "Blocked", guard));
        let synthesis = compiled(&text);
        if !synthesis.refusals.is_empty() {
            survived.push(format!("{guard}: refused {:?}", refusals(&synthesis)));
            continue;
        }
        let passed = failing(&text, 0, 1, right);
        if !passed.is_empty() {
            survived.push(format!("{guard}: faithful target failed {passed:?}"));
        }
        for (name, decide) in mutants {
            if failing(&text, 0, 1, decide).is_empty() {
                survived.push(format!("{guard}: mutant `{name}` survived"));
            }
        }
    }
    assert!(survived.is_empty(), "{survived:#?}");
}

/// Two guarded siblings on one counter, disjoint: `over` from 5, `blocked` from 3 to 4. At 4 the
/// command answers `blocked`; a target whose `over` limit is 4 answers `over` there. The default's
/// side row for `over` (`retries == 4`) is answered by `blocked`, not `started`, so it is not
/// witnessed as a row of `started`, and nothing else witnesses 4.
#[test]
fn a_guarded_sibling_answering_one_step_short_is_witnessed() {
    let outcomes = format!(
        "{}{}",
        branch("over", "Over", "retries >= 5"),
        branch("blocked", "Blocked", "{all: [retries >= 3, retries < 5]}")
    );
    let text = model(0, 1, &outcomes);
    kills(
        &text,
        0,
        1,
        |r, _| {
            if r >= 5 {
                Some(("over", "Over"))
            } else if r >= 3 {
                Some(("blocked", "Blocked"))
            } else {
                None
            }
        },
        &[("over >= 4", |r, _| {
            if r >= 4 {
                Some(("over", "Over"))
            } else if r >= 3 {
                Some(("blocked", "Blocked"))
            } else {
                None
            }
        })],
    );
}

/// `not: {retries < 3}` selects exactly `retries >= 3`. The refusal is at 3 (the first row holding
/// it) but no row one step short is witnessed for `started`, so a target blocking from 2 survives.
#[test]
fn a_limit_under_not_is_witnessed_one_step_short() {
    let text = model(0, 1, &branch("blocked", "Blocked", "{not: retries < 3}"));
    kills(
        &text,
        0,
        1,
        |r, _| (r >= 3).then_some(("blocked", "Blocked")),
        &[("limit >= 2", |r, _| {
            (r >= 2).then_some(("blocked", "Blocked"))
        })],
    );
}

/// Two counters in one guard, each raised by its own command, limit 5 each: the row holding both is
/// ten raises away, well within the counter reach, and must be witnessed or refused naming a bound.
#[test]
fn two_counters_in_one_guard_are_reached_or_refused_naming_the_bound() {
    let text = model(
        0,
        1,
        &branch("blocked", "Blocked", "{all: [retries >= 5, restarts >= 5]}"),
    );
    let synthesis = compiled(&text);
    let refused = refusals(&synthesis);
    assert!(
        refused.is_empty() || refused.iter().all(|refusal| refusal.contains("within 16")),
        "{refused:#?}"
    );
}

/// The owner-row model of `tests/counter_limit.rs`: `ReserveSlot` raises `open_cards`, `AddCard`
/// reads it through `when_related:`.
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
fn board_failing(text: &str, limit: fn(i64) -> bool) -> BTreeMap<String, String> {
    let synthesis = compiled(text);
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

/// `open_cards == 3` on the owner row read through `when_related:`: `ReserveSlot` raises the counter
/// without limit, so 4 is reached. A board reading `>= 3` answers `full` at 4 where the model adds the
/// card; the owner rows witness only the row one step short (2), so that board passes.
#[test]
fn an_owner_equality_limit_witnesses_the_row_past_it() {
    let text = BOARD.replace("open_cards >= 3", "open_cards == 3");
    assert_ne!(text, BOARD);
    let passed = board_failing(&text, |open| open == 3);
    assert!(passed.is_empty(), "the faithful board failed: {passed:#?}");
    let failed = board_failing(&text, |open| open >= 3);
    assert!(
        !failed.is_empty(),
        "a board reading `>= 3` passed; {FULL} and {ADDED} witness no row past the limit"
    );
}
