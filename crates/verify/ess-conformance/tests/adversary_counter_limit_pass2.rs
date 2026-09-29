//! Adversary pass 2 for beyond10x/ess#226, after correction 1.
//!
//! Each case builds a model whose counter `retries` is raised by commands that never read it and
//! read by `StartTask`, so only synthesized rows decide the limit. A hand-written target answers the
//! model; mutant targets move one limit. Every mutant is distinguishable on a row the model reaches.
#![allow(
    clippy::too_many_lines,
    clippy::type_complexity,
    clippy::single_match_else,
    clippy::format_push_string
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

/// The shape of one model: which commands exist and what they do to `retries`.
#[derive(Clone, Copy)]
struct Shape {
    /// Where `CreateTask` sets `retries`; `None` takes it from `input.start`.
    start: Option<i64>,
    /// `RetryTask`'s increment.
    step: i64,
    /// `BumpTask`'s increment, where the model has one.
    bump: Option<i64>,
    /// The literal `ResetTask` sets, where the model has one.
    reset: Option<i64>,
    /// The literal `AImportTask` creates a task at, where the model has one.
    import: Option<i64>,
}

const BASE: Shape = Shape {
    start: Some(0),
    step: 1,
    bump: None,
    reset: None,
    import: None,
};

fn command(name: &str, event: &str, effect: &str, sets: &str) -> String {
    format!(
        "  - name: work.tasks.{name}
    input:
      - {{name: task_id, type: work.tasks.TaskId}}
    outcomes:
      - name: done
        updates: work.tasks.Task
        instance: task_id
        emits: [work.tasks.{event}]
        payload:
          work.tasks.{event}: {{task_id: input.task_id}}
        sets: {{retries: {effect}}}
{sets}"
    )
}

fn event(name: &str) -> String {
    format!(
        "  - name: work.tasks.{name}\n    fields: [{{name: task_id, type: work.tasks.TaskId}}]\n"
    )
}

fn model(shape: Shape, outcomes: &str) -> String {
    let mut events = String::new();
    let mut commands = String::new();
    if let Some(import) = shape.import {
        events.push_str(&event("TaskImported"));
        commands.push_str(&format!(
            "  - name: work.tasks.AImportTask
    input: [{{name: label, type: String}}]
    outcomes:
      - name: imported
        creates: work.tasks.Task
        instance: task_id
        emits: [work.tasks.TaskImported]
        payload:
          work.tasks.TaskImported: {{task_id: {{generated: true}}}}
        sets: {{label: input.label, retries: {import}}}
"
        ));
    }
    if let Some(bump) = shape.bump {
        events.push_str(&event("TaskBumped"));
        commands.push_str(&command(
            "BumpTask",
            "TaskBumped",
            &format!("{{increment: {bump}}}"),
            "",
        ));
    }
    if let Some(reset) = shape.reset {
        events.push_str(&event("TaskReset"));
        commands.push_str(&command("ResetTask", "TaskReset", &reset.to_string(), ""));
    }
    let (create_input, create_start) = match shape.start {
        Some(start) => (String::new(), start.to_string()),
        None => (
            "\n      - {name: start, type: Integer}".to_owned(),
            "input.start".to_owned(),
        ),
    };
    format!(
        "format: ess/18
system: work
version: v1
domain: work.tasks
types:
  - {{name: work.tasks.TaskId, kind: newtype, of: Uuid}}
entities:
  - name: work.tasks.Task
    identity: {{name: task_id, type: work.tasks.TaskId}}
    fields:
      - {{name: label, type: String}}
      - {{name: retries, type: Integer}}
    lifecycle: {{initial: Open, states: [Open], terminal: [Open]}}
errors:
  - {{name: work.tasks.Blocked, fields: []}}
events:
  - name: work.tasks.TaskCreated
    fields: [{{name: task_id, type: work.tasks.TaskId}}]
  - name: work.tasks.TaskRetried
    fields: [{{name: task_id, type: work.tasks.TaskId}}]
  - name: work.tasks.TaskStarted
    fields: [{{name: task_id, type: work.tasks.TaskId}}]
{events}commands:
  - name: work.tasks.CreateTask
    input:
      - {{name: label, type: String}}{create_input}
    outcomes:
      - name: created
        creates: work.tasks.Task
        instance: task_id
        emits: [work.tasks.TaskCreated]
        payload:
          work.tasks.TaskCreated: {{task_id: {{generated: true}}}}
        sets: {{label: input.label, retries: {create_start}}}
{commands}{retry}  - name: work.tasks.StartTask
    input:
      - {{name: task_id, type: work.tasks.TaskId}}
      - {{name: label, type: String}}
    outcomes:
{outcomes}      - name: started
        updates: work.tasks.Task
        instance: task_id
        emits: [work.tasks.TaskStarted]
        payload:
          work.tasks.TaskStarted: {{task_id: input.task_id}}
        sets: {{label: input.label}}
views:
  - name: work.tasks.Tasks
    source: work.tasks.Task
    consistency: read_your_writes
    fields:
      - {{name: task_id, type: work.tasks.TaskId}}
      - {{name: state, type: work.tasks.Task.State}}
      - {{name: label, type: String}}
      - {{name: retries, type: Integer}}
",
        retry = command(
            "RetryTask",
            "TaskRetried",
            &format!("{{increment: {}}}", shape.step),
            ""
        ),
    )
}

fn blocked(guard: &str) -> String {
    format!(
        "      - name: blocked\n        when_subject: {{predicate: {guard}}}\n        error: work.tasks.Blocked\n"
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

/// Whether `StartTask` blocks a task holding `(retries, label)`.
type Decide = fn(i64, &str) -> bool;

struct Tasks {
    shape: Shape,
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
        Ok(ImplementationIdentity::new("tasks-adversary-2", "1"))
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
        let name = command.to_string();
        let created = match name.as_str() {
            "work.tasks.CreateTask" => Some((
                self.shape
                    .start
                    .unwrap_or_else(|| integer(&request.input["start"])),
                "created",
                "TaskCreated",
            )),
            "work.tasks.AImportTask" => {
                Some((self.shape.import.unwrap(), "imported", "TaskImported"))
            }
            _ => None,
        };
        let result = if let Some((retries, outcome, emitted)) = created {
            self.minted.set(self.minted.get() + 1);
            let id = Node::Text(format!("00000000-0000-4000-8000-{:012}", self.minted.get()));
            let mut row = BTreeMap::new();
            row.insert("task_id".to_owned(), id.clone());
            row.insert("state".to_owned(), Node::Text("Open".into()));
            row.insert("label".to_owned(), request.input["label"].clone());
            row.insert("retries".to_owned(), number(retries));
            self.rows.borrow_mut().push(row);
            took(outcome).emitting(event(emitted, id))
        } else {
            let id = request.input["task_id"].clone();
            let mut rows = self.rows.borrow_mut();
            let Some(row) = rows.iter_mut().find(|row| row["task_id"] == id) else {
                self.minted.set(self.minted.get() + 1);
                return Ok(SemanticCommandResult::undeclared().with_consistency(
                    ConsistencyToken::new(format!("seq:{}", self.minted.get())).unwrap(),
                ));
            };
            let retries = integer(&row["retries"]);
            let mut set = |value: i64, emitted: &str| {
                row.insert("retries".to_owned(), number(value));
                took("done").emitting(event(emitted, id.clone()))
            };
            match name.as_str() {
                "work.tasks.RetryTask" => set(retries + self.shape.step, "TaskRetried"),
                "work.tasks.BumpTask" => set(retries + self.shape.bump.unwrap(), "TaskBumped"),
                "work.tasks.ResetTask" => set(self.shape.reset.unwrap(), "TaskReset"),
                "work.tasks.StartTask" => {
                    let label = match &row["label"] {
                        Node::Text(text) => text.clone(),
                        other => panic!("a label: {other:?}"),
                    };
                    if (self.decide)(retries, &label) {
                        took("blocked").with_error(DeclaredErrorValue::new(
                            "work.tasks.Blocked".parse().unwrap(),
                        ))
                    } else {
                        row.insert("label".to_owned(), request.input["label"].clone());
                        took("started").emitting(event("TaskStarted", id.clone()))
                    }
                }
                other => return Err(TargetError::unsupported("command", other)),
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

/// The scenarios of `synthesis` failing against a target deciding with `decide`.
fn failing(synthesis: &Synthesis, shape: Shape, decide: Decide) -> Vec<String> {
    let suite = &synthesis.suite;
    let admitted = AdmittedSuite::from_suite(suite).unwrap();
    let target = Tasks {
        shape,
        decide,
        rows: RefCell::default(),
        minted: Cell::new(0),
    };
    let report = Runner::for_suite(suite)
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

/// Synthesis is clean, the faithful target passes, and every mutant fails some scenario.
fn kills(shape: Shape, outcomes: &str, right: Decide, mutants: &[(&str, Decide)]) {
    let text = model(shape, outcomes);
    let synthesis = compiled(&text);
    assert!(synthesis.refusals.is_empty(), "{:#?}", refusals(&synthesis));
    let passed = failing(&synthesis, shape, right);
    assert!(passed.is_empty(), "the faithful target failed: {passed:?}");
    let survived: Vec<&str> = mutants
        .iter()
        .filter(|(_, decide)| failing(&synthesis, shape, *decide).is_empty())
        .map(|(name, _)| *name)
        .collect();
    assert!(survived.is_empty(), "mutants survived: {survived:?}");
}

/// Two creations: `AImportTask` (searched first, by name) at 40, `CreateTask` at 0, raised by one,
/// `retries >= 30`. The side rows 29 and 30 lie 29 and 30 raises from 0, past the reach, exactly as
/// with `CreateTask` alone, which is refused naming the bound. With the import in front, the
/// search's refusal is the import's (every row above the limit, moving away), which is not the
/// bound, so both side rows are dropped silently and a target off by one either way passes.
#[test]
fn a_second_creation_past_the_limit_does_not_hide_the_bound() {
    let shape = Shape {
        import: Some(40),
        ..BASE
    };
    let text = model(shape, &blocked("retries >= 30"));
    let synthesis = compiled(&text);
    let refused = refusals(&synthesis);
    if refused.iter().any(|refusal| refusal.contains("within 16")) {
        return;
    }
    let survived: Vec<&str> = [
        ("limit >= 29", (|r, _| r >= 29) as Decide),
        ("limit >= 31", |r, _| r >= 31),
    ]
    .iter()
    .filter(|(_, decide)| failing(&synthesis, shape, *decide).is_empty())
    .map(|(name, _)| *name)
    .collect();
    assert!(
        survived.is_empty(),
        "neither refused naming the bound nor decided: survived {survived:?}, refusals {refused:#?}"
    );
}

/// `retries` created from an input: where a run starts is unknown, so the one-step rule applies.
#[test]
fn a_counter_started_from_an_input_is_decided_on_both_sides() {
    kills(
        Shape {
            start: None,
            ..BASE
        },
        &blocked("retries >= 3"),
        |r, _| r >= 3,
        &[("limit >= 2", |r, _| r >= 2), ("limit >= 4", |r, _| r >= 4)],
    );
}

/// Raised by three from 0, reset to 1: runs hold 0, 1, 3, 4, 6, 7. `retries >= 5`: the side rows are
/// 4 and 6, and a target blocking from 4 or from 7 differs on one of them.
#[test]
fn a_reset_to_a_literal_mid_run_pins_the_rows_it_reaches() {
    kills(
        Shape {
            step: 3,
            reset: Some(1),
            ..BASE
        },
        &blocked("retries >= 5"),
        |r, _| r >= 5,
        &[("limit >= 4", |r, _| r >= 4), ("limit >= 7", |r, _| r >= 7)],
    );
}

/// Two raising commands, by three and by one, toward `retries >= 10`.
#[test]
fn two_raising_commands_with_different_steps_are_decided_on_both_sides() {
    kills(
        Shape {
            step: 3,
            bump: Some(1),
            ..BASE
        },
        &blocked("retries >= 10"),
        |r, _| r >= 10,
        &[
            ("limit >= 9", |r, _| r >= 9),
            ("limit >= 11", |r, _| r >= 11),
        ],
    );
}

/// Raised by two, lowered by one, toward `retries >= 5`.
#[test]
fn an_increment_and_a_decrement_on_one_counter_are_decided_on_both_sides() {
    kills(
        Shape {
            step: 2,
            bump: Some(-1),
            ..BASE
        },
        &blocked("retries >= 5"),
        |r, _| r >= 5,
        &[("limit >= 4", |r, _| r >= 4), ("limit >= 6", |r, _| r >= 6)],
    );
}

/// A `not:` over an `any:` mixing the counter with a non-counter leaf, inside an `all:`:
/// `retries >= 3 and label != "x"` written negatively.
#[test]
fn a_nested_not_mixing_a_counter_with_another_leaf_is_decided_on_both_sides() {
    kills(
        BASE,
        &blocked("{all: [{not: {any: [retries < 3, label == \"x\"]}}]}"),
        |r, label| r >= 3 && label != "x",
        &[
            ("limit >= 2", |r, label| r >= 2 && label != "x"),
            ("limit >= 4", |r, label| r >= 4 && label != "x"),
        ],
    );
}
