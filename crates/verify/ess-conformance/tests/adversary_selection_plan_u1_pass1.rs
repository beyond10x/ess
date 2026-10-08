//! Adversary pass 1 against `story:selection-plan-design-and-type` (wave 2, unit U1): does the
//! precedence plan give the answer the model interpreter gives today, on a command that validates?
//!
//! Each case compiles a model (so it validates), asks the interpreter for one request, and asks the
//! plan the same question: of the branches whose question holds for that request — named by the
//! case, from the request it builds — the one the plan reads first. `docs/design/selection-plan.md`
//! states the plan reproduces the interpreter; a case where the two answers differ is a command the
//! plan orders differently from `interpret/execute.rs` `responding_core`/`select`.
//!
//! The models are the committed `related-guard-stored-reference.yaml` and
//! `filtered-related-reads.yaml` fixtures with one thing added: a refusal written `when: true`
//! (`Predicate::Always`), which the plan placed first in `HeldState` and the interpreter read at the
//! head of `select`. Validation refuses that shape since beyond10x/ess#489, so C1 and C2 now assert
//! the refusal: no command that validates carries it.
use std::collections::BTreeMap;

use ess_compiler::ir::{EssIr, PrecedencePlan};
use ess_compiler::{resolve::compile, source::SourceMap};
use ess_conformance::interpret::execute::{execute, Externals, Step, Store};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_primitives::node::Node;

const STORED: &str = include_str!("fixtures/related-guard-stored-reference.yaml");
const ROW_SETS: &str = include_str!("fixtures/filtered-related-reads.yaml");
const DANGLING: &str = "00000000-0000-4000-8000-777777777777";

fn replace(text: &str, from: &str, to: &str) -> String {
    assert!(text.contains(from), "the model holds {from:?}");
    text.replacen(from, to, 1)
}

fn ir_of(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("adversary.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("the model validates: {errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

/// Asserts validation refuses `text` because its `declined` refusal's `when:` always holds.
fn refused_as_always_holding(text: &str) {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let errors = Specification::assemble([(Source::new("adversary.yaml"), raw)])
        .err()
        .unwrap_or_else(|| panic!("a `when: true` refusal is refused (#489)\n{text}"))
        .to_string();
    assert!(
        errors.contains("`declined` is a refusal whose `when:` always holds"),
        "{errors}"
    );
}

fn step(ir: &EssIr, store: &Store, command: &str, input: &[(&str, Node)]) -> Step {
    let input: BTreeMap<String, Node> = input
        .iter()
        .map(|(name, value)| ((*name).to_owned(), value.clone()))
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

/// The branch name the interpreter answered with.
fn answered(step: &Step) -> String {
    step.outcome.as_ref().map_or_else(
        || "none".to_owned(),
        |outcome| {
            let text = outcome.to_string();
            text.rsplit_once('/')
                .map_or(text.clone(), |(_, name)| name.to_owned())
        },
    )
}

/// The identity the first payload text of `event` carries.
fn minted(step: &Step, event: &str) -> String {
    step.events
        .iter()
        .find(|published| published.event.to_string() == event)
        .and_then(|published| published.payload.values().find_map(Node::as_text))
        .unwrap_or_else(|| panic!("{event} carries the identity"))
        .to_owned()
}

/// The plan's answer: of `holding`, the branches whose question the request answers yes, the one
/// `command`'s precedence plan reads first.
fn plan_answer(ir: &EssIr, command: &str, holding: &[&str]) -> String {
    let command = &ir.commands()[&command.parse().unwrap()];
    let read: Vec<String> = PrecedencePlan::new(command, ir.format())
        .iter()
        .map(|(_, outcome)| outcome.name.to_string())
        .collect();
    for name in holding {
        assert!(
            read.contains(&(*name).to_owned()),
            "`{name}` is a branch: {read:?}"
        );
    }
    read.into_iter()
        .find(|name| holding.contains(&name.as_str()))
        .expect("a holding branch is planned")
}

fn text(value: &str) -> Node {
    Node::Text(value.to_owned())
}

// ---- control: the committed stored-reference fixture -------------------------------------------

/// The harness agrees with the interpreter where the plan's page says it does: a dangling stored
/// reference on an open task answers `exists: false`.
#[test]
fn adv_u1_control_a_dangling_stored_reference_answers_exists_false_in_both() {
    let ir = ir_of(STORED);
    let added = step(
        &ir,
        &Store::default(),
        "demo.tasks.AddTask",
        &[("blocked_by", text(DANGLING))],
    );
    let task = minted(&added, "demo.tasks.TaskAdded");
    let interpreter = answered(&step(
        &ir,
        &added.next,
        "demo.tasks.CompleteTask",
        &[("task_id", text(&task))],
    ));
    let plan = plan_answer(
        &ir,
        "demo.tasks.CompleteTask",
        &["blocker-missing", "completed"],
    );
    assert_eq!(interpreter, "blocker-missing");
    assert_eq!(plan, interpreter);
}

// ---- C1: a stored reference beside a `when: true` refusal -------------------------------------

/// `CompleteTask` with `completed` guarded by an input flag and a `when: true` refusal declared
/// last, as the default refusal is written the long way.
fn stored_with_when_true() -> String {
    let text = replace(
        STORED,
        "  - {name: demo.tasks.TaskStateConflict, summary: The task cannot move from its held state., fields: []}\n",
        "  - {name: demo.tasks.TaskStateConflict, summary: The task cannot move from its held state., fields: []}\n  - {name: demo.tasks.Declined, summary: The completion is declined., fields: []}\n",
    );
    let text = replace(
        &text,
        "  - name: demo.tasks.CompleteTask\n    input:\n      - {name: task_id, type: demo.tasks.TaskId}\n",
        "  - name: demo.tasks.CompleteTask\n    input:\n      - {name: task_id, type: demo.tasks.TaskId}\n      - {name: force, type: Boolean}\n",
    );
    let text = replace(
        &text,
        "      - name: completed\n        moves: demo.tasks.Task.complete\n",
        "      - name: completed\n        when: force == true\n        moves: demo.tasks.Task.complete\n",
    );
    replace(
        &text,
        "        payload: {demo.tasks.TaskCompleted: {task_id: input.task_id}}\nviews:\n",
        "        payload: {demo.tasks.TaskCompleted: {task_id: input.task_id}}\n      - name: declined\n        when: true\n        error: demo.tasks.Declined\nviews:\n",
    )
}

/// The stored reference beside a `when: true` refusal no longer validates (#489), so the plan and
/// the interpreter are never asked to order it.
#[test]
fn adv_u1_c1_stored_reference_beside_a_when_true_refusal_is_refused() {
    refused_as_always_holding(&stored_with_when_true());
}

// ---- C2: a row set beside a `when: true` refusal and `wrong_state` ---------------------------

/// `Close` with `closed` guarded by an input flag and a `when: true` refusal declared last, and a
/// `Shut` command that closes an attempt with no guard, so a closed attempt can be arranged.
fn row_set_with_when_true() -> String {
    let text = replace(
        ROW_SETS,
        "  - {name: demo.jobs.AlreadyClosed, summary: The attempt is closed already., fields: []}\n",
        "  - {name: demo.jobs.AlreadyClosed, summary: The attempt is closed already., fields: []}\n  - {name: demo.jobs.Declined, summary: The close is declined., fields: []}\n",
    );
    let text = replace(
        &text,
        "  - name: demo.jobs.Close\n    input:\n      - {name: attempt_id, type: demo.jobs.AttemptId}\n",
        "  - name: demo.jobs.Close\n    input:\n      - {name: attempt_id, type: demo.jobs.AttemptId}\n      - {name: confirm, type: Boolean}\n",
    );
    let text = replace(
        &text,
        "      - name: closed\n        moves: demo.jobs.Attempt.close\n",
        "      - name: closed\n        when: confirm == true\n        moves: demo.jobs.Attempt.close\n",
    );
    replace(
        &text,
        "          demo.jobs.AttemptClosed: {attempt_id: input.attempt_id}\nviews:\n",
        concat!(
            "          demo.jobs.AttemptClosed: {attempt_id: input.attempt_id}\n",
            "      - name: declined\n",
            "        when: true\n",
            "        error: demo.jobs.Declined\n",
            "  - name: demo.jobs.Shut\n",
            "    input:\n",
            "      - {name: attempt_id, type: demo.jobs.AttemptId}\n",
            "    outcomes:\n",
            "      - name: shut\n",
            "        moves: demo.jobs.Attempt.close\n",
            "        instance: attempt_id\n",
            "        emits: [demo.jobs.AttemptClosed]\n",
            "        payload:\n",
            "          demo.jobs.AttemptClosed: {attempt_id: input.attempt_id}\n",
            "views:\n",
        ),
    )
}

/// The row set beside a `when: true` refusal and `wrong_state` no longer validates (#489).
#[test]
fn adv_u1_c2_row_set_beside_a_when_true_refusal_is_refused() {
    refused_as_always_holding(&row_set_with_when_true());
}
