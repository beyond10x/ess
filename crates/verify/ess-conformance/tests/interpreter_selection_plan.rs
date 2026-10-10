//! The model interpreter selects through the precedence plan (`story:interpreter-reads-selection-plan`,
//! `docs/design/selection-plan.md`): its branch order is the plan's, phase by phase, so a plan read
//! with two phases exchanged through `ess-domain`'s `with_phase_order` seam changes the answer.
//!
//! `tests/fixtures/external-beside-held-guard.yaml`, with `rushed: when: rush == true` declared after
//! `stale` (the model `tests/selection_precedence.rs` builds): a pick holding revision 1 sent
//! `revision: 2, rush: true` is selected by both the held state (`stale`, step 4) and the accepting
//! branch (`rushed`, step 6).
use std::collections::{BTreeMap, BTreeSet};

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::interpret::execute::{execute, Externals, Store};
use ess_domain::{
    command::precedence::{with_phase_order, Phase},
    name::QualifiedName,
    spec::{RawSpecFile, Specification},
    system::Source,
};
use ess_primitives::{facts::Number, node::Node};

const MODEL: &str = include_str!("fixtures/external-beside-held-guard.yaml");

const UNLISTED_BRANCH: &str = "      - name: unlisted\n";
const INPUT: &str = "      - {name: pick_id, type: demo.desk.PickId}\n      - {name: revision, type: Integer}\n    outcomes:\n";
const RUSH_INPUT: &str = "      - {name: pick_id, type: demo.desk.PickId}\n      - {name: revision, type: Integer}\n      - {name: rush, type: Boolean}\n    outcomes:\n";
const RUSHED_BRANCH: &str = "      - name: rushed\n        when: rush == true\n        moves: demo.desk.Pick.accept\n        instance: pick_id\n        emits: [demo.desk.PickAccepted]\n        payload:\n          demo.desk.PickAccepted: {pick_id: input.pick_id}\n";

/// The fixture with `rush` declared and `rushed` declared after `stale`, before `unlisted`.
fn model() -> String {
    let model = MODEL.replacen(INPUT, RUSH_INPUT, 1);
    assert_ne!(model, MODEL, "`CheckPick`'s input is declared");
    let rushed = model.replacen(
        UNLISTED_BRANCH,
        &format!("{RUSHED_BRANCH}{UNLISTED_BRANCH}"),
        1,
    );
    assert_ne!(rushed, model, "`unlisted` is declared");
    rushed
}

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("model.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("the model validates: {errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn name(text: &str) -> QualifiedName {
    QualifiedName::new(text).unwrap()
}

fn integer(value: i64) -> Node {
    Node::Number(Number::from(value))
}

/// The branches `CheckPick` answers on a pick holding revision 1, sent `revision: 2, rush: true`.
fn answered(ir: &EssIr, externals: &Externals) -> BTreeSet<String> {
    let steps = execute(
        ir,
        &Store::default(),
        &name("demo.desk.MakePick"),
        &BTreeMap::from([("revision".to_owned(), integer(1))]),
        &Externals::Withheld,
    )
    .expect("the model determines MakePick");
    let [made] = steps.as_slice() else {
        panic!("MakePick answers once: {steps:#?}");
    };
    let (_, pick, _) = made
        .next
        .instances()
        .next()
        .expect("MakePick creates a pick");
    let sent = BTreeMap::from([
        ("pick_id".to_owned(), pick.clone()),
        ("revision".to_owned(), integer(2)),
        ("rush".to_owned(), Node::Bool(true)),
    ]);
    execute(
        ir,
        &made.next,
        &name("demo.desk.CheckPick"),
        &sent,
        externals,
    )
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

/// [`Phase::PRECEDENCE`] with the held-state phase (step 4) and the accepting/external phase
/// (step 6) exchanged.
fn exchanged() -> [Phase; 8] {
    let mut order = Phase::PRECEDENCE;
    let at = |phase| order.iter().position(|held| *held == phase).unwrap();
    let (held, accepting) = (at(Phase::HeldState), at(Phase::Accepting));
    order.swap(held, accepting);
    order
}

#[test]
fn the_real_plan_answers_the_held_state_first() {
    let ir = ir(&model());
    assert_eq!(answered(&ir, &Externals::Withheld), one("stale"));
    assert_eq!(answered(&ir, &Externals::Open), one("stale"));
}

#[test]
fn the_exchanged_plan_answers_the_accepting_branch_first() {
    let ir = ir(&model());
    let (withheld, open) = with_phase_order(exchanged(), || {
        (
            answered(&ir, &Externals::Withheld),
            answered(&ir, &Externals::Open),
        )
    });
    assert_eq!(withheld, one("rushed"));
    assert_eq!(open, one("rushed"));
}
