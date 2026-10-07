//! The precedence plan gives the model interpreter's answer where a marker's question is decided by
//! what the interpreter reads first (`docs/design/selection-plan.md`, "The markers answer"; adversary
//! pass 2 of `story:selection-plan-design-and-type`, R1 and R3).
//!
//! Each case compiles a fixture, asks the interpreter for one request, and asks the plan which of
//! the branches whose question holds for that request it reads first. Which branches hold is the
//! page's rule, applied by hand and stated beside each case.
//!
//! The `wrong_state:` cases bound where that marker can sit. On
//! `precedence-row-set-updating-branch.yaml`, a closed attempt alone, sent with a delay, is answered
//! by `annotated`; a closed attempt beside a second one, sent with none, by `already-closed` before
//! the row-set refusal `crowded`; an open attempt beside a second one, sent with a delay, by
//! `crowded` before `annotated`. So `already-closed` reads before `crowded`, and `crowded` before
//! `annotated`: no order reads `annotated` before `already-closed` where both are counted as
//! holding, and the first request is answered alike only because `already-closed`'s question does
//! not hold there.
use std::collections::BTreeMap;

use ess_compiler::ir::{EssIr, PrecedencePlan};
use ess_compiler::{resolve::compile, source::SourceMap};
use ess_conformance::interpret::execute::{execute, Externals, Step, Store};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_primitives::facts::Number;
use ess_primitives::node::Node;

const UPSERT_UNKNOWN: &str = include_str!(
    "../../../specify/ess-compiler/tests/fixtures/precedence-upsert-unknown-instance.yaml"
);
const UPDATING: &str = include_str!(
    "../../../specify/ess-compiler/tests/fixtures/precedence-row-set-updating-branch.yaml"
);

fn ir_of(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}"));
    let spec = Specification::assemble([(Source::new("model.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("the model validates: {errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
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
    let outcome = step
        .outcome
        .as_ref()
        .expect("a declared outcome")
        .to_string();
    outcome
        .rsplit_once('/')
        .map_or(outcome.clone(), |(_, name)| name.to_owned())
}

/// The identity the first text field of `event`'s payload carries.
fn minted(step: &Step, event: &str) -> String {
    step.events
        .iter()
        .find(|published| published.event.to_string() == event)
        .and_then(|published| published.payload.values().find_map(Node::as_text))
        .unwrap_or_else(|| panic!("{event} carries the identity"))
        .to_owned()
}

/// Of `holding`, the branch `command`'s plan reads first.
fn plan_first(ir: &EssIr, command: &str, holding: &[&str]) -> String {
    let command = &ir.commands()[&command.parse().unwrap()];
    let read: Vec<String> = PrecedencePlan::new(command, ir.format())
        .iter()
        .map(|(_, outcome)| outcome.name.to_string())
        .collect();
    for name in holding {
        assert!(read.contains(&(*name).to_owned()), "`{name}` is a branch");
    }
    read.into_iter()
        .find(|name| holding.contains(&name.as_str()))
        .expect("a holding branch is planned")
}

fn text(value: &str) -> Node {
    Node::Text(value.to_owned())
}

// ---- `unknown_instance:` beside a row-set upsert ----------------------------------------------

/// A library whose curator is `the-curator`.
fn opened(ir: &EssIr) -> Store {
    let opened = step(
        ir,
        &Store::default(),
        "demo.shelf.OpenLibrary",
        &[("library", text("main")), ("curator", text("the-curator"))],
    );
    assert_eq!(answered(&opened), "opened");
    opened.next
}

/// A book no row carries, sent by the curator of an open library and not as new: the row set
/// `refused` holds; `unknown-book` holds, since the branch selected after the row sets, the
/// default `replaced`, addresses the absent row; `replaced` holds. The row sets answer first.
#[test]
fn a_row_set_refusal_answers_before_unknown_instance_on_an_upsert() {
    let ir = ir_of(UPSERT_UNKNOWN);
    let store = opened(&ir);
    let interpreter = answered(&step(
        &ir,
        &store,
        "demo.shelf.Shelve",
        &[
            ("book", text("absent-book")),
            ("curator", text("the-curator")),
            ("mode", text("old")),
        ],
    ));
    assert_eq!(interpreter, "refused");
    assert_eq!(
        plan_first(
            &ir,
            "demo.shelf.Shelve",
            &["refused", "unknown-book", "replaced"]
        ),
        interpreter
    );
}

/// The same book sent by a curator with no library: the row set does not hold, and the absent row
/// the default addresses is answered by `unknown-book`, before the default.
#[test]
fn unknown_instance_answers_the_branch_selected_after_the_row_sets_on_an_upsert() {
    let ir = ir_of(UPSERT_UNKNOWN);
    let store = opened(&ir);
    let interpreter = answered(&step(
        &ir,
        &store,
        "demo.shelf.Shelve",
        &[
            ("book", text("absent-book")),
            ("curator", text("someone-else")),
            ("mode", text("old")),
        ],
    ));
    assert_eq!(interpreter, "unknown-book");
    assert_eq!(
        plan_first(&ir, "demo.shelf.Shelve", &["unknown-book", "replaced"]),
        interpreter
    );
}

// ---- `wrong_state:` beside an accepting branch that updates -----------------------------------

/// An attempt of worker `w` and batch `b`, recorded into `store`, and its identity.
fn recorded(ir: &EssIr, store: &Store) -> (Store, String) {
    let recorded = step(
        ir,
        store,
        "demo.jobs.Record",
        &[
            ("worker_id", text("w")),
            ("batch_id", text("b")),
            ("delay", Node::Number(Number::from(1_i64))),
        ],
    );
    let attempt = minted(&recorded, "demo.jobs.AttemptRecorded");
    (recorded.next, attempt)
}

fn shut(ir: &EssIr, store: &Store, attempt: &str) -> Store {
    let shut = step(
        ir,
        store,
        "demo.jobs.Shut",
        &[("attempt_id", text(attempt))],
    );
    assert_eq!(answered(&shut), "shut");
    shut.next
}

fn close(ir: &EssIr, store: &Store, attempt: &str, delay: i64) -> String {
    answered(&step(
        ir,
        store,
        "demo.jobs.Close",
        &[
            ("attempt_id", text(attempt)),
            ("delay", Node::Number(Number::from(delay))),
        ],
    ))
}

/// A closed attempt alone, sent with a delay: `annotated` updates and moves nothing, so not every
/// accepting branch acting on the row moves and `already-closed` does not answer at once; the
/// branch selected, `annotated`, moves nothing, so it does not answer for it either. `annotated`
/// and the default `closed` hold.
#[test]
fn an_updating_branch_answers_on_a_closed_row_where_wrong_state_does_not_hold() {
    let ir = ir_of(UPDATING);
    let (store, attempt) = recorded(&ir, &Store::default());
    let store = shut(&ir, &store, &attempt);
    let interpreter = close(&ir, &store, &attempt, 5);
    assert_eq!(interpreter, "annotated");
    assert_eq!(
        plan_first(&ir, "demo.jobs.Close", &["annotated", "closed"]),
        interpreter
    );
}

/// A closed attempt beside a second one, sent with no delay: the branch selected with the row-set
/// refusal left out is the default `closed`, which moves from `Open`, so `already-closed` holds;
/// `crowded` holds (two rows); `closed` holds. `wrong_state:` answers before the row sets.
#[test]
fn wrong_state_answers_before_the_row_set_refusal_for_the_branch_selected() {
    let ir = ir_of(UPDATING);
    let (store, attempt) = recorded(&ir, &Store::default());
    let (store, _) = recorded(&ir, &store);
    let store = shut(&ir, &store, &attempt);
    let interpreter = close(&ir, &store, &attempt, 0);
    assert_eq!(interpreter, "already-closed");
    assert_eq!(
        plan_first(
            &ir,
            "demo.jobs.Close",
            &["already-closed", "crowded", "closed"]
        ),
        interpreter
    );
}

/// An open attempt beside a second one, sent with a delay: `crowded`, `annotated` and `closed`
/// hold, and the row-set refusal answers before the accepting branch.
#[test]
fn the_row_set_refusal_answers_before_the_updating_branch() {
    let ir = ir_of(UPDATING);
    let (store, attempt) = recorded(&ir, &Store::default());
    let (store, _) = recorded(&ir, &store);
    let interpreter = close(&ir, &store, &attempt, 5);
    assert_eq!(interpreter, "crowded");
    assert_eq!(
        plan_first(&ir, "demo.jobs.Close", &["crowded", "annotated", "closed"]),
        interpreter
    );
}
