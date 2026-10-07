//! A branch the held state selects is declared before every accepting and external branch of its
//! command (beyond10x/ess#486).
//!
//! The held state selects at step 4 of the precedence order and the accepting and external
//! branches at step 6, whatever their declaration order
//! (`docs/design/cross-record-and-stored-field-guards.md`, "The precedence order"). The model
//! interpreter read the three in one declaration-order pass, so a command declaring an accepting or
//! external branch first answered differently here than in the Rust and Go targets. Validation now
//! refuses that order, and on every order it admits the interpreter answers as the design does.
//!
//! `tests/fixtures/external-beside-held-guard.yaml`: `CheckPick` refuses a pick whose stored
//! `revision` differs from the input's (`stale`, a `when_subject` guard) and declares the external
//! `unlisted` after it.
use std::collections::{BTreeMap, BTreeSet};

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::interpret::execute::{execute, Externals, Store};
use ess_domain::{
    command::OutcomeName,
    name::QualifiedName,
    spec::{RawSpecFile, Specification},
    system::Source,
};
use ess_primitives::{facts::Number, node::Node};

const MODEL: &str = include_str!("fixtures/external-beside-held-guard.yaml");

const STALE_BRANCH: &str = "      - name: stale\n";
const UNLISTED_BRANCH: &str = "      - name: unlisted\n";
const ACCEPTED_BRANCH: &str = "      - name: accepted\n";

/// `CheckPick`'s input with a Boolean `rush`, and `rushed`, an accepting branch taking it.
const INPUT: &str = "      - {name: pick_id, type: demo.desk.PickId}\n      - {name: revision, type: Integer}\n    outcomes:\n";
const RUSH_INPUT: &str = "      - {name: pick_id, type: demo.desk.PickId}\n      - {name: revision, type: Integer}\n      - {name: rush, type: Boolean}\n    outcomes:\n";
const RUSHED_BRANCH: &str = "      - name: rushed\n        when: rush == true\n        moves: demo.desk.Pick.accept\n        instance: pick_id\n        emits: [demo.desk.PickAccepted]\n        payload:\n          demo.desk.PickAccepted: {pick_id: input.pick_id}\n";

fn specification(text: &str) -> Result<Specification, String> {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    Specification::assemble([(Source::new("model.yaml"), raw)]).map_err(|errors| errors.to_string())
}

fn ir(text: &str) -> EssIr {
    let spec = specification(text).unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

/// `model` with the branch starting at `moved` declared before the branch starting at `anchor`.
fn declared_before(model: &str, moved: &str, anchor: &str, next: &str) -> String {
    let at_anchor = model.find(anchor).expect("the anchor is declared");
    let at_moved = model.find(moved).expect("the moved branch is declared");
    let at_next = model.find(next).expect("the branch after it is declared");
    assert!(at_anchor < at_moved && at_moved < at_next);
    format!(
        "{}{}{}{}",
        &model[..at_anchor],
        &model[at_moved..at_next],
        &model[at_anchor..at_moved],
        &model[at_next..]
    )
}

/// The fixture with `rush` declared, and `rushed` inserted before the branch starting at `before`.
fn with_rushed_before(before: &str) -> String {
    let model = MODEL.replacen(INPUT, RUSH_INPUT, 1);
    assert_ne!(model, MODEL, "`CheckPick`'s input is declared");
    let rushed = model.replacen(before, &format!("{RUSHED_BRANCH}{before}"), 1);
    assert_ne!(rushed, model, "the anchor is declared");
    rushed
}

fn refusal(model: &str) -> String {
    specification(model).map_or_else(|errors| errors, |_| panic!("the model validates:\n{model}"))
}

fn name(text: &str) -> QualifiedName {
    QualifiedName::new(text).unwrap()
}

fn integer(value: i64) -> Node {
    Node::Number(Number::from(value))
}

/// A store holding one `Picked` pick of `revision`, made by `MakePick`, and its identity.
fn picked(ir: &EssIr, revision: i64) -> (Store, Node) {
    let steps = execute(
        ir,
        &Store::default(),
        &name("demo.desk.MakePick"),
        &BTreeMap::from([("revision".to_owned(), integer(revision))]),
        &Externals::Withheld,
    )
    .expect("the model determines MakePick");
    let [step] = steps.as_slice() else {
        panic!("MakePick answers once: {steps:#?}");
    };
    let (_, identity, _) = step
        .next
        .instances()
        .next()
        .expect("MakePick creates a pick");
    (step.next.clone(), identity.clone())
}

/// The branches `CheckPick` answers on a pick holding revision 1, sent `input` beside its identity.
fn answered(model: &str, input: &[(&str, Node)], externals: &Externals) -> BTreeSet<String> {
    let ir = ir(model);
    let (store, pick) = picked(&ir, 1);
    let mut sent = BTreeMap::from([("pick_id".to_owned(), pick)]);
    sent.extend(
        input
            .iter()
            .map(|(field, value)| ((*field).to_owned(), value.clone())),
    );
    execute(&ir, &store, &name("demo.desk.CheckPick"), &sent, externals)
        .unwrap_or_else(|why| panic!("the model determines CheckPick: {why}"))
        .iter()
        .map(|step| {
            step.outcome
                .as_ref()
                .map_or("none".to_owned(), ToString::to_string)
        })
        .collect()
}

fn one(branch: &str) -> BTreeSet<String> {
    BTreeSet::from([format!("demo.desk.CheckPick/{branch}")])
}

fn forced(branch: &str) -> Externals {
    Externals::Forced(OutcomeName::new(branch).unwrap())
}

#[test]
fn an_accepting_branch_declared_before_a_held_state_branch_is_refused() {
    let errors = refusal(&with_rushed_before(STALE_BRANCH));
    assert!(
        errors.contains(
            "[conflicting_declaration] command.demo.desk.CheckPick.outcomes.stale: `stale` is \
             selected by the held state, which answers before the accepting branch `rushed` \
             declared above it"
        ),
        "{errors}"
    );
    assert!(
        errors.contains("declare `stale` before `rushed`"),
        "{errors}"
    );
}

#[test]
fn an_external_branch_declared_before_a_held_state_branch_is_refused() {
    let errors = refusal(&declared_before(
        MODEL,
        UNLISTED_BRANCH,
        STALE_BRANCH,
        ACCEPTED_BRANCH,
    ));
    assert!(
        errors.contains(
            "[conflicting_declaration] command.demo.desk.CheckPick.outcomes.stale: `stale` is \
             selected by the held state, which answers before the external branch `unlisted` \
             declared above it"
        ),
        "{errors}"
    );
    assert!(
        errors.contains("declare `stale` before `unlisted`"),
        "{errors}"
    );
}

/// Declared before the external branch, the held-state branch answers wherever its guard holds,
/// under every provider.
#[test]
fn declared_first_the_held_state_branch_answers_before_the_external_one() {
    let stale = [("revision", integer(2))];
    assert_eq!(answered(MODEL, &stale, &Externals::Withheld), one("stale"));
    assert_eq!(answered(MODEL, &stale, &forced("unlisted")), one("stale"));
    assert_eq!(answered(MODEL, &stale, &Externals::Open), one("stale"));
    let current = [("revision", integer(1))];
    assert_eq!(
        answered(MODEL, &current, &forced("unlisted")),
        one("unlisted")
    );
    assert_eq!(
        answered(MODEL, &current, &Externals::Withheld),
        one("accepted")
    );
}

/// Declared before the accepting branch, the held-state branch answers where both guards hold, and
/// the accepting branch where only its own does.
#[test]
fn declared_first_the_held_state_branch_answers_before_the_accepting_one() {
    let model = with_rushed_before(UNLISTED_BRANCH);
    let rush =
        |revision: i64, rush: bool| [("revision", integer(revision)), ("rush", Node::Bool(rush))];
    assert_eq!(
        answered(&model, &rush(2, true), &Externals::Withheld),
        one("stale")
    );
    assert_eq!(
        answered(&model, &rush(1, true), &Externals::Withheld),
        one("rushed")
    );
    assert_eq!(
        answered(&model, &rush(1, false), &Externals::Withheld),
        one("accepted")
    );
}
