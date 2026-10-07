//! Adversary pass 2 against `story:selection-plan-design-and-type` (wave 2, unit U1): the marker
//! rules the revised `docs/design/selection-plan.md` states ("The markers answer; they are not read
//! in order"), driven against the model interpreter on commands that validate.
//!
//! Each case compiles a model, asks the interpreter for one request, and asks the plan which of the
//! branches whose question holds — decided by the page's own rule for each marker, quoted in the
//! case — it reads first. Where the two differ, the page's rule is not the interpreter's.
use std::collections::BTreeMap;

use ess_compiler::ir::{EssIr, PrecedencePlan};
use ess_compiler::{resolve::compile, source::SourceMap};
use ess_conformance::interpret::execute::{execute, Externals, Step, Store};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_primitives::facts::Number;
use ess_primitives::node::Node;

const UPSERT: &str = include_str!("fixtures/row-set-upsert.yaml");
const ROW_SETS: &str = include_str!("fixtures/filtered-related-reads.yaml");

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

fn minted(step: &Step, event: &str) -> String {
    step.events
        .iter()
        .find(|published| published.event.to_string() == event)
        .and_then(|published| published.payload.values().find_map(Node::as_text))
        .unwrap_or_else(|| panic!("{event} carries the identity"))
        .to_owned()
}

/// Of `holding`, the one `command`'s precedence plan reads first.
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

// ---- R1: `unknown_instance:` on a row-set command beside an upsert ---------------------------

/// `Shelve` with the ess#462 adversary's input-guarded upsert
/// (`adversary_row_set_upsert_462.rs`, `input_guarded_creation`): `unknown-book`, the row-set
/// refusal `refused`, `added` creating the book under `book` where `mode == "new"`, and the
/// updating default `replaced`. That test runs it as a model that validates.
fn shelve_with_unknown_beside_upsert() -> String {
    let (head, tail) = UPSERT
        .split_once("  - name: demo.shelf.Shelve\n")
        .expect("the Shelve command");
    let (shelve, rest) = tail
        .split_once("  - name: demo.shelf.Restock\n")
        .expect("the next command");
    let shelve = replace(
        shelve,
        "      - {name: curator, type: String}\n",
        "      - {name: curator, type: String}\n      - {name: mode, type: String}\n",
    );
    let shelve = replace(
        &shelve,
        "    outcomes:\n",
        "    outcomes:\n      - name: unknown-book\n        unknown_instance: true\n        error: demo.shelf.UnknownBook\n",
    );
    let shelve = replace(
        &shelve,
        "external: the book is not yet shelved",
        "when: mode == \"new\"",
    );
    format!("{head}  - name: demo.shelf.Shelve\n{shelve}  - name: demo.shelf.Restock\n{rest}")
}

/// A new book no row carries, shelved by a curator whose library is open.
///
/// The page (`selection-plan.md`, "The markers answer"): "`unknown_instance:` answers where the
/// addressed row is not held: at once … on a row-set command (`addressed_row`)". The addressed row
/// (`book`, which `replaced` updates) is not held, so `unknown-book` holds; the row set `refused`
/// holds; `added`'s guard holds. The interpreter's `addressed_row` leaves an absent row a creation
/// takes to the row sets (`creation_takes_absent`, beyond10x/ess#462).
#[test]
fn adv_u1_p2_r1_unknown_instance_on_a_row_set_command_beside_an_upsert() {
    let ir = ir_of(&shelve_with_unknown_beside_upsert());
    let opened = step(
        &ir,
        &Store::default(),
        "demo.shelf.OpenLibrary",
        &[
            ("library", text("matching")),
            ("curator", text("the-curator")),
        ],
    );
    assert_eq!(answered(&opened), "opened");
    let interpreter = answered(&step(
        &ir,
        &opened.next,
        "demo.shelf.Shelve",
        &[
            ("book", text("absent-book")),
            ("curator", text("the-curator")),
            ("mode", text("new")),
        ],
    ));
    let plan = plan_answer(
        &ir,
        "demo.shelf.Shelve",
        &["unknown-book", "refused", "added"],
    );
    assert_eq!(
        plan, interpreter,
        "the page's rule gives `{plan}` where the interpreter answers `{interpreter}`"
    );
}

// ---- R3: `wrong_state:` on a row-set command beside an updating branch -----------------------

/// `Close` with an `annotated` branch that updates the attempt where `delay > 0`, and a `Shut`
/// command that closes an attempt with no guard.
fn close_with_an_updating_branch() -> String {
    let text = replace(
        ROW_SETS,
        "  - name: demo.jobs.AttemptClosed\n    fields:\n      - {name: attempt_id, type: demo.jobs.AttemptId}\n",
        "  - name: demo.jobs.AttemptClosed\n    fields:\n      - {name: attempt_id, type: demo.jobs.AttemptId}\n  - name: demo.jobs.AttemptAnnotated\n    fields:\n      - {name: attempt_id, type: demo.jobs.AttemptId}\n",
    );
    let text = replace(
        &text,
        "  - name: demo.jobs.Close\n    input:\n      - {name: attempt_id, type: demo.jobs.AttemptId}\n",
        "  - name: demo.jobs.Close\n    input:\n      - {name: attempt_id, type: demo.jobs.AttemptId}\n      - {name: delay, type: Integer}\n",
    );
    let text = replace(
        &text,
        "      - name: closed\n        moves: demo.jobs.Attempt.close\n",
        concat!(
            "      - name: annotated\n",
            "        when: delay > 0\n",
            "        updates: demo.jobs.Attempt\n",
            "        instance: attempt_id\n",
            "        sets: {delay: input.delay}\n",
            "        emits: [demo.jobs.AttemptAnnotated]\n",
            "        payload:\n",
            "          demo.jobs.AttemptAnnotated: {attempt_id: input.attempt_id}\n",
            "      - name: closed\n",
            "        moves: demo.jobs.Attempt.close\n",
        ),
    );
    replace(
        &text,
        "          demo.jobs.AttemptClosed: {attempt_id: input.attempt_id}\nviews:\n",
        concat!(
            "          demo.jobs.AttemptClosed: {attempt_id: input.attempt_id}\n",
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

/// A closed attempt, the only one of its worker and batch, sent with `delay: 5`.
///
/// The page, corrected: "On a row-set command it also answers at once where every accepting branch
/// acting on the row moves, and none from the state it holds (`addressed_row`)"; otherwise only where
/// the branch looking ahead selects moves from a state the row does not hold. `annotated` acts and
/// moves nothing, so neither part holds and `already-closed` is not counted; `annotated`'s guard
/// holds; the default `closed` holds; the row set `count: {gt: 1}` does not. Counting
/// `already-closed` here cannot be answered by any order: on this model it must read before
/// `crowded` (a closed row beside a second one, sent with no delay) and `crowded` before
/// `annotated` (`ess-conformance/tests/precedence_plan_answers.rs`).
#[test]
fn adv_u1_p2_r3_wrong_state_on_a_row_set_command_beside_an_updating_branch() {
    let ir = ir_of(&close_with_an_updating_branch());
    let recorded = step(
        &ir,
        &Store::default(),
        "demo.jobs.Record",
        &[
            ("worker_id", text("w")),
            ("batch_id", text("b")),
            ("delay", Node::Number(Number::from(1_i64))),
        ],
    );
    let attempt = minted(&recorded, "demo.jobs.AttemptRecorded");
    let shut = step(
        &ir,
        &recorded.next,
        "demo.jobs.Shut",
        &[("attempt_id", text(&attempt))],
    );
    assert_eq!(answered(&shut), "shut");
    let interpreter = answered(&step(
        &ir,
        &shut.next,
        "demo.jobs.Close",
        &[
            ("attempt_id", text(&attempt)),
            ("delay", Node::Number(Number::from(5_i64))),
        ],
    ));
    let plan = plan_answer(&ir, "demo.jobs.Close", &["annotated", "closed"]);
    assert_eq!(
        plan, interpreter,
        "the page's rule gives `{plan}` where the interpreter answers `{interpreter}`"
    );
}
