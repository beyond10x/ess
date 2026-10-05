//! Adversary pass 1 against beyond10x/ess#304 slice 2: a `when_related:` guard reading a stored
//! field of the addressed subject (`via: <field>`, `ess/22`).
//!
//! Each model below is the committed `related-guard-stored-reference.yaml` fixture with one thing
//! added that the fixture cannot reach: a task cancelled while its blocker is open (held state that
//! also blocks), a field the branch itself `sets:`, a field an earlier command rewrote or cleared,
//! both declaration orders of `wrong_state`, and an accepting `when_related:` branch that moves the
//! subject. The interpreter is asked directly, the synthesized suite is run against the
//! interpreter (agreement), and against targets that get the precedence wrong.

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};

use ess_compiler::ir::EssIr;
use ess_compiler::refs::{CommandRef, OutcomeRef};
use ess_compiler::{resolve::compile, source::SourceMap};
use ess_conformance::interpret::execute::{execute, Externals, Step, Store};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::scenario::{InstanceName, ScenarioStep};
use ess_conformance::target::*;
use ess_conformance::{synthesize::Synthesis, AdmittedSuite, Runner, ScenarioValue};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_primitives::node::Node;

const FIXTURE: &str = include_str!("fixtures/related-guard-stored-reference.yaml");
const COMPLETE: &str = "demo.tasks.CompleteTask";

/// The fixture's lifecycle, with a `cancel` move out of `Open` that reads no related row.
fn with_cancel(text: &str) -> String {
    let text = replace(
        text,
        "      states: [Open, Done]\n      terminal: [Done]\n      transitions:\n        - {name: complete, from: [Open], to: Done}\n",
        "      states: [Open, Done, Cancelled]\n      terminal: [Done, Cancelled]\n      transitions:\n        - {name: complete, from: [Open], to: Done}\n        - {name: cancel, from: [Open], to: Cancelled}\n",
    );
    let text = replace(
        &text,
        "  - name: demo.tasks.TaskCompleted\n    fields: [{name: task_id, type: demo.tasks.TaskId}]\n",
        "  - name: demo.tasks.TaskCompleted\n    fields: [{name: task_id, type: demo.tasks.TaskId}]\n  - name: demo.tasks.TaskCancelled\n    fields: [{name: task_id, type: demo.tasks.TaskId}]\n",
    );
    let text = replace(
        &text,
        "    may: [demo.tasks.AddTask, demo.tasks.CompleteTask]\n",
        "    may: [demo.tasks.AddTask, demo.tasks.CompleteTask, demo.tasks.CancelTask]\n",
    );
    let text = replace(
        &text,
        "views:\n",
        "  - name: demo.tasks.CancelTask\n    input:\n      - {name: task_id, type: demo.tasks.TaskId}\n    outcomes:\n      - name: cancelled\n        moves: demo.tasks.Task.cancel\n        instance: task_id\n        emits: [demo.tasks.TaskCancelled]\n        payload: {demo.tasks.TaskCancelled: {task_id: input.task_id}}\nviews:\n",
    );
    let text = replace(
        &text,
        "        - demo.tasks.CompleteTask\n    publishes:\n",
        "        - demo.tasks.CompleteTask\n        - demo.tasks.CancelTask\n    publishes:\n",
    );
    replace(
        &text,
        "        - demo.tasks.TaskCompleted\n",
        "        - demo.tasks.TaskCompleted\n        - demo.tasks.TaskCancelled\n",
    )
}

/// [`with_cancel`]'s model with a `close` move into `Done` that reads no related row, so a blocker
/// is done without `CompleteTask`.
fn with_close(text: &str) -> String {
    let text = replace(
        text,
        "        - {name: cancel, from: [Open], to: Cancelled}\n",
        "        - {name: cancel, from: [Open], to: Cancelled}\n        - {name: close, from: [Open], to: Done}\n",
    );
    let text = replace(
        &text,
        "demo.tasks.CompleteTask, demo.tasks.CancelTask]\n",
        "demo.tasks.CompleteTask, demo.tasks.CancelTask, demo.tasks.CloseTask]\n",
    );
    let text = replace(
        &text,
        "views:\n",
        "  - name: demo.tasks.CloseTask\n    input:\n      - {name: task_id, type: demo.tasks.TaskId}\n    outcomes:\n      - name: closed\n        moves: demo.tasks.Task.close\n        instance: task_id\n        emits: [demo.tasks.TaskCancelled]\n        payload: {demo.tasks.TaskCancelled: {task_id: input.task_id}}\nviews:\n",
    );
    replace(
        &text,
        "        - demo.tasks.CancelTask\n    publishes:\n",
        "        - demo.tasks.CancelTask\n        - demo.tasks.CloseTask\n    publishes:\n",
    )
}

/// The fixture with `SetBlocker` (rewrites the stored reference), `Unblock` (clears it) and
/// `Reassign` (reads the stored reference and `sets:` it in the same branch).
fn with_writers(text: &str) -> String {
    let text = replace(
        text,
        "  - name: demo.tasks.TaskCompleted\n    fields: [{name: task_id, type: demo.tasks.TaskId}]\n",
        "  - name: demo.tasks.TaskCompleted\n    fields: [{name: task_id, type: demo.tasks.TaskId}]\n  - name: demo.tasks.BlockerSet\n    fields: [{name: task_id, type: demo.tasks.TaskId}]\n  - name: demo.tasks.Unblocked\n    fields: [{name: task_id, type: demo.tasks.TaskId}]\n  - name: demo.tasks.Reassigned\n    fields: [{name: task_id, type: demo.tasks.TaskId}]\n",
    );
    let text = replace(
        &text,
        "    may: [demo.tasks.AddTask, demo.tasks.CompleteTask]\n",
        "    may: [demo.tasks.AddTask, demo.tasks.CompleteTask, demo.tasks.SetBlocker, demo.tasks.Unblock, demo.tasks.Reassign]\n",
    );
    let text = replace(
        &text,
        "views:\n",
        concat!(
            "  - name: demo.tasks.SetBlocker\n",
            "    input:\n",
            "      - {name: task_id, type: demo.tasks.TaskId}\n",
            "      - {name: blocker, type: Optional<demo.tasks.TaskId>}\n",
            "    outcomes:\n",
            "      - name: set\n",
            "        updates: demo.tasks.Task\n",
            "        instance: task_id\n",
            "        sets: {blocked_by: input.blocker}\n",
            "        emits: [demo.tasks.BlockerSet]\n",
            "        payload: {demo.tasks.BlockerSet: {task_id: input.task_id}}\n",
            "  - name: demo.tasks.Unblock\n",
            "    input:\n",
            "      - {name: task_id, type: demo.tasks.TaskId}\n",
            "    outcomes:\n",
            "      - name: unblocked\n",
            "        updates: demo.tasks.Task\n",
            "        instance: task_id\n",
            "        sets: {blocked_by: {cleared: true}}\n",
            "        emits: [demo.tasks.Unblocked]\n",
            "        payload: {demo.tasks.Unblocked: {task_id: input.task_id}}\n",
            "  - name: demo.tasks.Reassign\n",
            "    input:\n",
            "      - {name: task_id, type: demo.tasks.TaskId}\n",
            "      - {name: blocker, type: Optional<demo.tasks.TaskId>}\n",
            "    outcomes:\n",
            "      - name: reassign-missing\n",
            "        when_related: {via: blocked_by, exists: false}\n",
            "        error: demo.tasks.BlockerMissing\n",
            "      - name: still-blocked\n",
            "        when_related: {via: blocked_by, predicate: state != Done}\n",
            "        error: demo.tasks.Blocked\n",
            "      - name: reassigned\n",
            "        updates: demo.tasks.Task\n",
            "        instance: task_id\n",
            "        sets: {blocked_by: input.blocker}\n",
            "        emits: [demo.tasks.Reassigned]\n",
            "        payload: {demo.tasks.Reassigned: {task_id: input.task_id}}\n",
            "views:\n",
        ),
    );
    let text = replace(
        &text,
        "        - demo.tasks.CompleteTask\n    publishes:\n",
        "        - demo.tasks.CompleteTask\n        - demo.tasks.SetBlocker\n        - demo.tasks.Unblock\n        - demo.tasks.Reassign\n    publishes:\n",
    );
    replace(
        &text,
        "        - demo.tasks.TaskCompleted\n",
        "        - demo.tasks.TaskCompleted\n        - demo.tasks.BlockerSet\n        - demo.tasks.Unblocked\n        - demo.tasks.Reassigned\n",
    )
}

const FIXTURE_OUTCOMES: &str = "      - name: blocker-missing\n        when_related: {via: blocked_by, exists: false}\n        error: demo.tasks.BlockerMissing\n      - name: blocked\n        when_related: {via: blocked_by, predicate: state != Done}\n        error: demo.tasks.Blocked\n      - {name: wrong-state, wrong_state: true, error: demo.tasks.TaskStateConflict}\n";

/// `wrong_state` declared before the related branches instead of after them.
fn wrong_state_first(text: &str) -> String {
    replace(
        text,
        FIXTURE_OUTCOMES,
        "      - {name: wrong-state, wrong_state: true, error: demo.tasks.TaskStateConflict}\n      - name: blocked\n        when_related: {via: blocked_by, predicate: state != Done}\n        error: demo.tasks.Blocked\n      - name: blocker-missing\n        when_related: {via: blocked_by, exists: false}\n        error: demo.tasks.BlockerMissing\n",
    )
}

/// The accepting branch is the `when_related:` one, and moves the subject; the default refuses.
fn accepting_related(text: &str) -> String {
    replace(
        text,
        concat!(
            "      - name: blocker-missing\n        when_related: {via: blocked_by, exists: false}\n        error: demo.tasks.BlockerMissing\n",
            "      - name: blocked\n        when_related: {via: blocked_by, predicate: state != Done}\n        error: demo.tasks.Blocked\n",
            "      - {name: wrong-state, wrong_state: true, error: demo.tasks.TaskStateConflict}\n",
            "      - name: completed\n        moves: demo.tasks.Task.complete\n",
        ),
        concat!(
            "      - name: blocker-missing\n        when_related: {via: blocked_by, exists: false}\n        error: demo.tasks.BlockerMissing\n",
            "      - {name: wrong-state, wrong_state: true, error: demo.tasks.TaskStateConflict}\n",
            "      - name: completed\n        when_related: {via: blocked_by, predicate: state == Done}\n        moves: demo.tasks.Task.complete\n",
        ),
    )
    .replace(
        "        payload: {demo.tasks.TaskCompleted: {task_id: input.task_id}}\nviews:\n",
        "        payload: {demo.tasks.TaskCompleted: {task_id: input.task_id}}\n      - name: blocked\n        error: demo.tasks.Blocked\nviews:\n",
    )
}

fn replace(text: &str, from: &str, to: &str) -> String {
    assert!(text.contains(from), "the model holds {from:?}");
    text.replacen(from, to, 1)
}

fn ir_of(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("stored-reference.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

// ---- the interpreter, asked directly --------------------------------------------------------------

fn step(ir: &EssIr, store: &Store, command: &str, input: &[(&str, Option<&str>)]) -> Step {
    let input: BTreeMap<String, Node> = input
        .iter()
        .filter_map(|(name, value)| {
            value.map(|value| ((*name).to_owned(), Node::Text(value.to_owned())))
        })
        .collect();
    let mut steps = execute(
        ir,
        store,
        &command.parse().unwrap(),
        &input,
        &Externals::Withheld,
    )
    .unwrap_or_else(|error| panic!("{command} {input:?} is interpreted: {error}"));
    assert_eq!(steps.len(), 1, "{command} has one selected outcome");
    steps.remove(0)
}

fn outcome(step: &Step) -> String {
    step.outcome
        .as_ref()
        .map_or_else(|| "none".to_owned(), ToString::to_string)
}

fn add(ir: &EssIr, store: &Store, blocked_by: Option<&str>) -> (Store, String) {
    let taken = step(
        ir,
        store,
        "demo.tasks.AddTask",
        &[("blocked_by", blocked_by)],
    );
    let identity = taken
        .events
        .iter()
        .find(|published| published.event.to_string() == "demo.tasks.TaskAdded")
        .and_then(|published| published.payload.values().find_map(Node::as_text))
        .expect("TaskAdded carries the identity")
        .to_owned();
    (taken.next, identity)
}

fn on(ir: &EssIr, store: &Store, command: &str, task: &str) -> Step {
    step(ir, store, command, &[("task_id", Some(task))])
}

const DANGLING: &str = "00000000-0000-4000-8000-777777777777";

#[test]
fn adv304s_held_state_answers_before_an_open_stored_blocker() {
    let ir = ir_of(&with_cancel(FIXTURE));
    let (store, blocker) = add(&ir, &Store::default(), None);
    let (store, task) = add(&ir, &store, Some(&blocker));
    let store = on(&ir, &store, "demo.tasks.CancelTask", &task).next;
    assert_eq!(
        outcome(&on(&ir, &store, COMPLETE, &task)),
        "demo.tasks.CompleteTask/wrong-state",
        "a cancelled task is answered by its held state before its open blocker is read"
    );
    let (store, dangling) = add(&ir, &store, Some(DANGLING));
    let store = on(&ir, &store, "demo.tasks.CancelTask", &dangling).next;
    assert_eq!(
        outcome(&on(&ir, &store, COMPLETE, &dangling)),
        "demo.tasks.CompleteTask/wrong-state",
        "a cancelled task is answered by its held state before its dangling blocker is read"
    );
}

#[test]
fn adv304s_the_branch_reads_the_value_before_its_own_sets() {
    let ir = ir_of(&with_writers(FIXTURE));
    let (store, open) = add(&ir, &Store::default(), None);
    let (store, done) = add(&ir, &store, None);
    let store = on(&ir, &store, COMPLETE, &done).next;
    let (store, task) = add(&ir, &store, Some(&open));
    let refused = step(
        &ir,
        &store,
        "demo.tasks.Reassign",
        &[("task_id", Some(&task)), ("blocker", Some(&done))],
    );
    assert_eq!(
        outcome(&refused),
        "demo.tasks.Reassign/still-blocked",
        "the stored blocker before the branch is open, whatever the branch would write"
    );
    let (store, other) = add(&ir, &store, Some(&done));
    let moved = step(
        &ir,
        &store,
        "demo.tasks.Reassign",
        &[("task_id", Some(&other)), ("blocker", Some(&open))],
    );
    assert_eq!(outcome(&moved), "demo.tasks.Reassign/reassigned");
}

#[test]
fn adv304s_an_earlier_rewrite_or_clear_is_what_is_read() {
    let ir = ir_of(&with_writers(FIXTURE));
    let (store, open) = add(&ir, &Store::default(), None);
    let (store, done) = add(&ir, &store, None);
    let store = on(&ir, &store, COMPLETE, &done).next;
    // Created blocked by the open task, then pointed at the done one.
    let (store, task) = add(&ir, &store, Some(&open));
    let store = step(
        &ir,
        &store,
        "demo.tasks.SetBlocker",
        &[("task_id", Some(&task)), ("blocker", Some(&done))],
    )
    .next;
    assert_eq!(
        outcome(&on(&ir, &store, COMPLETE, &task)),
        "demo.tasks.CompleteTask/completed"
    );
    // Created unblocked, then pointed at the open one.
    let (store, later) = add(&ir, &store, None);
    let store = step(
        &ir,
        &store,
        "demo.tasks.SetBlocker",
        &[("task_id", Some(&later)), ("blocker", Some(&open))],
    )
    .next;
    assert_eq!(
        outcome(&on(&ir, &store, COMPLETE, &later)),
        "demo.tasks.CompleteTask/blocked"
    );
    // Blocked by the open task, then cleared: absent, so no related branch.
    let (store, cleared) = add(&ir, &store, Some(&open));
    let store = on(&ir, &store, "demo.tasks.Unblock", &cleared).next;
    assert_eq!(
        outcome(&on(&ir, &store, COMPLETE, &cleared)),
        "demo.tasks.CompleteTask/completed",
        "a cleared Optional reference is absent"
    );
}

/// Coordinator decision F1 (correction round 1): a stored via beside `wrong_state`, where an
/// accepting `when_related:` branch moves the subject, is refused with the `conflicting_declaration`
/// an input via gets, until a precedence for it is designed. The model is unchanged; this case
/// asserted the interpreter's held-state answer for it before the decision.
fn refused_as_unordered(text: &str) {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let errors = Specification::assemble([(Source::new("stored-reference.yaml"), raw)])
        .err()
        .unwrap_or_else(|| panic!("an accepting related move beside `wrong_state` is refused"));
    assert!(
        errors.as_slice().iter().any(|error| {
            error.code == ess_primitives::error::ValidationCode::ConflictingDeclaration
                && error
                    .to_string()
                    .contains("which of the two answers first is not stated")
        }),
        "{errors}"
    );
}

#[test]
fn adv304s_an_accepting_related_move_still_answers_held_state() {
    refused_as_unordered(&with_close(&with_cancel(&accepting_related(FIXTURE))));
}

// ---- synthesis against the interpreter ------------------------------------------------------------

fn statuses<T: ConformanceTarget>(result: &Synthesis, target: &T) -> BTreeMap<String, Status> {
    let admitted =
        AdmittedSuite::from_suite(&result.suite).unwrap_or_else(|error| panic!("{error}"));
    Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, target)
        .into_report()
        .scenarios
        .into_iter()
        .map(|run| (run.scenario.to_string(), run.status))
        .collect()
}

/// Every scenario synthesized for `text` passes against the interpreter of `text`, and every
/// branch of every command reading a stored reference is witnessed or refused with a reason.
fn agreement(label: &str, text: &str) -> Vec<String> {
    let ir = ir_of(text);
    let result = ess_conformance::synthesize::synthesize(&ir);
    let mut problems = Vec::new();
    for (id, status) in statuses(&result, &Interpreted::for_model(ir.clone())) {
        if status != Status::Passed {
            problems.push(format!(
                "{label}: {id} is {status:?} against the interpreter"
            ));
        }
    }
    let refusals: Vec<String> = result.refusals.iter().map(ToString::to_string).collect();
    for command in ["demo.tasks.CompleteTask", "demo.tasks.Reassign"] {
        let Some(declared) = ir.commands().get(&command.parse().unwrap()) else {
            continue;
        };
        for branch in &declared.outcomes {
            let id = format!("{command}/outcome/{}", branch.name);
            let witnessed = result
                .suite
                .scenarios
                .keys()
                .any(|key| key.to_string() == id);
            let noted = refusals.iter().find(|text| text.contains(&id));
            if !witnessed {
                problems.push(format!(
                    "{label}: {id} has no scenario; refusal: {}",
                    noted.map_or("none", String::as_str)
                ));
            }
        }
    }
    problems
}

#[test]
fn adv304s_synthesis_and_interpreter_agree_on_the_refusing_variants() {
    let mut problems = Vec::new();
    for (label, text) in [
        ("fixture", FIXTURE.to_owned()),
        ("cancel", with_cancel(FIXTURE)),
        ("writers", with_writers(FIXTURE)),
        ("wrong-state-first", wrong_state_first(FIXTURE)),
        ("no-wrong-state", replace(FIXTURE, "      - {name: wrong-state, wrong_state: true, error: demo.tasks.TaskStateConflict}\n", "")),
    ] {
        problems.extend(agreement(label, &text));
    }
    assert!(problems.is_empty(), "{problems:#?}");
}

// ---- a target that reads the stored reference before the held state ------------------------------

/// Answers `CompleteTask` from the stored blocker first: an open blocker is `blocked`, a dangling
/// one `blocker-missing`, before the task's own state is looked at. Otherwise the interpreter.
struct ReferenceFirst {
    inner: Interpreted,
    blocker: RefCell<BTreeMap<String, Option<String>>>,
    done: RefCell<BTreeSet<String>>,
}

impl ConformanceTarget for ReferenceFirst {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        self.inner.identity()
    }
    fn begin_scenario(&self, context: &ScenarioContext) -> Result<(), TargetError> {
        self.blocker.borrow_mut().clear();
        self.done.borrow_mut().clear();
        self.inner.begin_scenario(context)
    }
    fn end_scenario(&self, context: &ScenarioContext) -> Result<(), TargetError> {
        self.inner.end_scenario(context)
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        let command = request.command.to_string();
        if command == COMPLETE {
            if let Some(task) = request.input.get("task_id").and_then(Node::as_text) {
                let stored = self.blocker.borrow().get(task).cloned().flatten();
                if let Some(blocker) = stored {
                    let known = self.blocker.borrow().contains_key(&blocker);
                    let refusal = if !known {
                        Some(("blocker-missing", "demo.tasks.BlockerMissing"))
                    } else if !self.done.borrow().contains(&blocker) {
                        Some(("blocked", "demo.tasks.Blocked"))
                    } else {
                        None
                    };
                    if let Some((name, error)) = refusal {
                        let mut result = SemanticCommandResult::took(OutcomeRef::new(
                            CommandRef::new(COMPLETE.parse().unwrap()),
                            name.parse().unwrap(),
                        ));
                        result.error = Some(DeclaredErrorValue::new(error.parse().unwrap()));
                        return Ok(result);
                    }
                }
            }
        }
        let blocked_by = request
            .input
            .get("blocked_by")
            .and_then(Node::as_text)
            .map(str::to_owned);
        let task = request
            .input
            .get("task_id")
            .and_then(Node::as_text)
            .map(str::to_owned);
        let result = self.inner.execute_command(request)?;
        let taken = result
            .outcome
            .as_ref()
            .map(|outcome| outcome.outcome.to_string())
            .unwrap_or_default();
        if command == "demo.tasks.AddTask" {
            for event in &result.direct_events {
                if let Some(identity) = event.payload.get("task_id").and_then(Node::as_text) {
                    self.blocker
                        .borrow_mut()
                        .insert(identity.to_owned(), blocked_by.clone());
                }
            }
        }
        if command == COMPLETE && taken == "completed" {
            if let Some(task) = task {
                self.done.borrow_mut().insert(task);
            }
        }
        Ok(result)
    }
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        self.inner.query_view(request)
    }
    fn observe_events(
        &self,
        request: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        self.inner.observe_events(request)
    }
    fn configure_external_outcome(
        &self,
        control: ExternalOutcomeControl,
    ) -> Result<(), TargetError> {
        self.inner.configure_external_outcome(control)
    }
    fn redeliver_event(&self, request: RedeliveryRequest) -> Result<(), TargetError> {
        self.inner.redeliver_event(request)
    }
}

#[test]
fn adv304s_a_target_reading_the_reference_before_held_state_fails_the_suite() {
    let text = with_cancel(FIXTURE);
    let ir = ir_of(&text);
    let result = ess_conformance::synthesize::synthesize(&ir);
    let honest = statuses(&result, &Interpreted::for_model(ir.clone()));
    assert!(
        honest.values().all(|status| *status == Status::Passed),
        "the interpreter passes first: {honest:#?}"
    );
    let faulty = statuses(
        &result,
        &ReferenceFirst {
            inner: Interpreted::for_model(ir),
            blocker: RefCell::new(BTreeMap::new()),
            done: RefCell::new(BTreeSet::new()),
        },
    );
    // The mutant executes: it answers the related branches as the interpreter does.
    for id in [
        "demo.tasks.CompleteTask/outcome/blocked",
        "demo.tasks.CompleteTask/outcome/blocker-missing",
    ] {
        assert_eq!(faulty.get(id), Some(&Status::Passed), "{id}: {faulty:#?}");
    }
    assert!(
        faulty.values().any(|status| *status != Status::Passed),
        "some scenario sends `CompleteTask` to a cancelled task whose stored blocker is open or \
         dangling, so a target that reads the reference before the held state fails: {faulty:#?}"
    );
}

/// What each `Reassign` the suite sends would answer if its stored blocker were read before the
/// branch (`pre`) and if the blocker the branch writes were read instead (`post`).
fn reassign_reads(steps: &[ScenarioStep]) -> Vec<(String, String, String)> {
    let mut pending: Option<(String, BTreeMap<String, ScenarioValue>)> = None;
    let mut added: Option<Option<ScenarioValue>> = None;
    let mut stored: BTreeMap<InstanceName, Option<ScenarioValue>> = BTreeMap::new();
    let mut done: BTreeSet<InstanceName> = BTreeSet::new();
    let mut found = Vec::new();
    let read = |value: &Option<ScenarioValue>, done: &BTreeSet<InstanceName>| match value {
        None => "absent".to_owned(),
        Some(ScenarioValue::Instance { instance }) if done.contains(instance) => "done".to_owned(),
        Some(ScenarioValue::Instance { .. }) => "open".to_owned(),
        Some(_) => "dangling".to_owned(),
    };
    for step in steps {
        match step {
            ScenarioStep::ExecuteCommand { command, input, .. } => {
                pending = Some((command.to_string(), input.clone()));
            }
            ScenarioStep::ExpectOutcome { outcome } => {
                let Some((command, input)) = pending.take() else {
                    continue;
                };
                let taken = outcome.outcome.to_string();
                let task = match input.get("task_id") {
                    Some(ScenarioValue::Instance { instance }) => Some(instance.clone()),
                    _ => None,
                };
                match (command.as_str(), taken.as_str()) {
                    ("demo.tasks.AddTask", "added") => {
                        added = Some(input.get("blocked_by").cloned());
                    }
                    ("demo.tasks.CompleteTask", "completed") => {
                        done.extend(task);
                    }
                    ("demo.tasks.SetBlocker", "set") | ("demo.tasks.Reassign", "reassigned") => {
                        let pre = task
                            .as_ref()
                            .and_then(|task| stored.get(task).cloned())
                            .flatten();
                        if command == "demo.tasks.Reassign" {
                            found.push((
                                taken.clone(),
                                read(&pre, &done),
                                read(&input.get("blocker").cloned(), &done),
                            ));
                        }
                        if let Some(task) = task {
                            stored.insert(task, input.get("blocker").cloned());
                        }
                    }
                    ("demo.tasks.Unblock", "unblocked") => {
                        if let Some(task) = task {
                            stored.insert(task, None);
                        }
                    }
                    ("demo.tasks.Reassign", _) => {
                        let pre = task
                            .as_ref()
                            .and_then(|task| stored.get(task).cloned())
                            .flatten();
                        found.push((
                            taken.clone(),
                            read(&pre, &done),
                            read(&input.get("blocker").cloned(), &done),
                        ));
                    }
                    _ => {}
                }
            }
            ScenarioStep::CaptureInstance { instance, .. } => {
                if let Some(blocked_by) = added.take() {
                    stored.insert(instance.clone(), blocked_by);
                }
            }
            _ => {}
        }
    }
    found
}

#[test]
fn adv304s_the_suite_tells_the_pre_branch_value_from_the_one_the_branch_writes() {
    let ir = ir_of(&with_writers(FIXTURE));
    let result = ess_conformance::synthesize::synthesize(&ir);
    let reads: Vec<(String, String, String)> = result
        .suite
        .scenarios
        .values()
        .flat_map(|scenario| reassign_reads(&scenario.steps))
        .collect();
    // Under the post-branch reading, `open` and `dangling` refuse and `absent`/`done` accept.
    let answer = |read: &str| match read {
        "open" => "still-blocked",
        "dangling" => "reassign-missing",
        _ => "reassigned",
    };
    assert!(
        reads
            .iter()
            .any(|(_, pre, post)| answer(pre) != answer(post)),
        "some `Reassign` scenario writes a blocker that would answer otherwise than the stored \
         one, so a target reading the post-branch value fails: {reads:#?}"
    );
}

/// Model D: the accepting branch is the `when_related:` one and moves the subject, the default
/// refuses, and `wrong_state` is declared. Before coordinator decision F1 the domain admitted it and
/// its suite failed against the interpreter; it is now refused as an input via is
/// ([`refused_as_unordered`]).
#[test]
fn adv304s_an_accepting_related_move_is_synthesized_as_the_interpreter_answers() {
    // Decision F1: the model is refused, so there is no suite to run against the interpreter.
    refused_as_unordered(&with_close(&with_cancel(&accepting_related(FIXTURE))));
}

/// `AddTask` refuses an identity no task carries before storing it, and `SetBlocker` stores one
/// unchecked: a task holding a dangling blocker is reachable (add, then `SetBlocker` to an identity
/// no task carries), so `blocker-missing` is witnessed — or, where it is not, the refusal does not
/// say no run reaches it.
#[test]
fn adv304s_a_dangling_reference_a_later_unchecked_writer_stores_is_not_called_unreachable() {
    let checked = replace(
        &with_writers(FIXTURE),
        "    outcomes:\n      - name: added\n",
        "    outcomes:\n      - name: no-such-blocker\n        when_related: {via: input.blocked_by, exists: false}\n        error: demo.tasks.BlockerMissing\n      - name: added\n",
    );
    let ir = ir_of(&checked);
    // Reachable: the interpreter takes `blocker-missing` after an unchecked rewrite.
    let (store, task) = add(&ir, &Store::default(), None);
    let store = step(
        &ir,
        &store,
        "demo.tasks.SetBlocker",
        &[("task_id", Some(&task)), ("blocker", Some(DANGLING))],
    )
    .next;
    assert_eq!(
        outcome(&on(&ir, &store, COMPLETE, &task)),
        "demo.tasks.CompleteTask/blocker-missing"
    );
    let result = ess_conformance::synthesize::synthesize(&ir);
    let id = "demo.tasks.CompleteTask/outcome/blocker-missing";
    let witnessed = result
        .suite
        .scenarios
        .keys()
        .any(|key| key.to_string() == id);
    let noted: Vec<String> = result
        .refusals
        .iter()
        .map(|refusal| format!("{} {refusal}", refusal.code()))
        .filter(|text| text.contains(id))
        .collect();
    assert!(
        witnessed || !noted.iter().any(|text| text.contains("unreachable")),
        "{id} is reachable through `SetBlocker`; it is not refused as unreachable: {noted:#?}"
    );
}

/// A required stored reference to another entity: a task belongs to a milestone and is completed
/// only once the milestone is reached. The domain admits a required stored via
/// (`a_stored_reference_via_validates_under_ess_22`); every branch of `CompleteTask` is reachable
/// (the interpreter takes each below), so the suite witnesses each, and agrees with the
/// interpreter.
const MILESTONES: &str = r"format: ess/22
system: demo
version: v1
domain: demo.tasks
summary: A task belongs to a milestone and is completed once the milestone is reached.
types:
  - {name: demo.tasks.TaskId, kind: newtype, of: Uuid}
  - {name: demo.tasks.MilestoneId, kind: newtype, of: Uuid}
entities:
  - name: demo.tasks.Milestone
    identity: {name: milestone_id, type: demo.tasks.MilestoneId}
    lifecycle:
      initial: Planned
      states: [Planned, Reached]
      terminal: [Reached]
      transitions:
        - {name: reach, from: [Planned], to: Reached}
  - name: demo.tasks.Task
    identity: {name: task_id, type: demo.tasks.TaskId}
    fields:
      - {name: milestone, type: demo.tasks.MilestoneId}
    lifecycle:
      initial: Open
      states: [Open, Done]
      terminal: [Done]
      transitions:
        - {name: complete, from: [Open], to: Done}
errors:
  - {name: demo.tasks.MilestoneMissing, summary: No milestone carries the stored identity., fields: []}
  - {name: demo.tasks.TooEarly, summary: The milestone is not reached., fields: []}
  - {name: demo.tasks.TaskStateConflict, summary: The task cannot move from its held state., fields: []}
events:
  - name: demo.tasks.MilestonePlanned
    fields: [{name: milestone_id, type: demo.tasks.MilestoneId}]
  - name: demo.tasks.MilestoneReached
    fields: [{name: milestone_id, type: demo.tasks.MilestoneId}]
  - name: demo.tasks.TaskAdded
    fields: [{name: task_id, type: demo.tasks.TaskId}]
  - name: demo.tasks.TaskCompleted
    fields: [{name: task_id, type: demo.tasks.TaskId}]
actors:
  - name: demo.tasks.Planner
    may: [demo.tasks.PlanMilestone, demo.tasks.ReachMilestone, demo.tasks.AddTask, demo.tasks.CompleteTask]
commands:
  - name: demo.tasks.PlanMilestone
    outcomes:
      - name: planned
        creates: demo.tasks.Milestone
        instance: milestone_id
        emits: [demo.tasks.MilestonePlanned]
        payload: {demo.tasks.MilestonePlanned: {milestone_id: {generated: true}}}
  - name: demo.tasks.ReachMilestone
    input:
      - {name: milestone_id, type: demo.tasks.MilestoneId}
    outcomes:
      - name: reached
        moves: demo.tasks.Milestone.reach
        instance: milestone_id
        emits: [demo.tasks.MilestoneReached]
        payload: {demo.tasks.MilestoneReached: {milestone_id: input.milestone_id}}
  - name: demo.tasks.AddTask
    input:
      - {name: milestone, type: demo.tasks.MilestoneId}
    outcomes:
      - name: added
        creates: demo.tasks.Task
        instance: task_id
        sets: {milestone: input.milestone}
        emits: [demo.tasks.TaskAdded]
        payload: {demo.tasks.TaskAdded: {task_id: {generated: true}}}
  - name: demo.tasks.CompleteTask
    input:
      - {name: task_id, type: demo.tasks.TaskId}
    outcomes:
      - name: milestone-missing
        when_related: {via: milestone, exists: false}
        error: demo.tasks.MilestoneMissing
      - name: too-early
        when_related: {via: milestone, predicate: state != Reached}
        error: demo.tasks.TooEarly
      - {name: wrong-state, wrong_state: true, error: demo.tasks.TaskStateConflict}
      - name: completed
        moves: demo.tasks.Task.complete
        instance: task_id
        emits: [demo.tasks.TaskCompleted]
        payload: {demo.tasks.TaskCompleted: {task_id: input.task_id}}
views:
  - name: demo.tasks.Tasks
    source: demo.tasks.Task
    consistency: read_your_writes
    fields:
      - {name: task_id, type: demo.tasks.TaskId}
      - {name: milestone, type: demo.tasks.MilestoneId}
      - {name: state, type: demo.tasks.Task.State}
components:
  - component: task-service
    reached_by: network
    owns:
      domains: [demo.tasks]
    accepts:
      commands:
        - demo.tasks.PlanMilestone
        - demo.tasks.ReachMilestone
        - demo.tasks.AddTask
        - demo.tasks.CompleteTask
    publishes:
      events:
        - demo.tasks.MilestonePlanned
        - demo.tasks.MilestoneReached
        - demo.tasks.TaskAdded
        - demo.tasks.TaskCompleted
";

#[test]
fn adv304s_a_required_stored_reference_has_every_branch_witnessed() {
    let ir = ir_of(MILESTONES);
    // Every branch is reachable.
    let planned = step(&ir, &Store::default(), "demo.tasks.PlanMilestone", &[]);
    let milestone = planned
        .events
        .iter()
        .find_map(|published| published.payload.values().find_map(Node::as_text))
        .expect("the milestone identity")
        .to_owned();
    let added = step(
        &ir,
        &planned.next,
        "demo.tasks.AddTask",
        &[("milestone", Some(&milestone))],
    );
    let task = added
        .events
        .iter()
        .find_map(|published| published.payload.values().find_map(Node::as_text))
        .expect("the task identity")
        .to_owned();
    assert_eq!(
        outcome(&on(&ir, &added.next, COMPLETE, &task)),
        "demo.tasks.CompleteTask/too-early"
    );
    let reached = step(
        &ir,
        &added.next,
        "demo.tasks.ReachMilestone",
        &[("milestone_id", Some(&milestone))],
    );
    let completed = on(&ir, &reached.next, COMPLETE, &task);
    assert_eq!(outcome(&completed), "demo.tasks.CompleteTask/completed");
    assert_eq!(
        outcome(&on(&ir, &completed.next, COMPLETE, &task)),
        "demo.tasks.CompleteTask/wrong-state"
    );

    let mut problems = agreement("milestones", MILESTONES);
    let result = ess_conformance::synthesize::synthesize(&ir);
    for id in [
        "demo.tasks.CompleteTask/outcome/milestone-missing",
        "demo.tasks.CompleteTask/outcome/too-early",
        "demo.tasks.CompleteTask/outcome/completed",
        "demo.tasks.CompleteTask/outcome/wrong-state",
    ] {
        if !result
            .suite
            .scenarios
            .keys()
            .any(|key| key.to_string() == id)
        {
            problems.push(format!("{id} has no scenario"));
        }
    }
    let refusals: Vec<String> = result.refusals.iter().map(ToString::to_string).collect();
    assert!(
        problems.is_empty(),
        "{problems:#?}\nrefusals: {refusals:#?}"
    );
}
