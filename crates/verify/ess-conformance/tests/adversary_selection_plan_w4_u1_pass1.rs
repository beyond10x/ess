//! Adversary pass 1 of `story:interpreter-reads-selection-plan` (wave 4, unit U1).
//!
//! The story's contract: the interpreter takes its branch order from the precedence plan, so an
//! order exchanged through `ess-domain`'s `with_phase_order` seam moves the answer in every phase
//! pair it exchanges — not only held state (step 4) against accepting (step 6).
//!
//! `responding_core` answers `refused_by_input` (step 2) in its pre-`select` loop, and the
//! addressed row's `unknown_instance:` (step 3) in the held-subject loop after that loop. Neither
//! reads the plan's order against the phases answered elsewhere, so exchanging step 2 with step 4,
//! or step 2 with step 3, leaves the answer where the real plan puts it.
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

/// A pick checked against its revision: an input refusal (`too-big`, step 2), a held-state branch
/// (`stale`, step 4), the default move, and the two markers.
const MODEL: &str = r"format: ess/20
system: demo
version: v1
domain: demo.desk
summary: A pick checked against the current revision before it is accepted.
types:
  - {name: demo.desk.PickId, kind: newtype, of: Uuid}
entities:
  - name: demo.desk.Pick
    identity: {name: pick_id, type: demo.desk.PickId}
    fields:
      - {name: revision, type: Integer}
    lifecycle:
      initial: Picked
      states: [Picked, Accepted, Refused]
      terminal: [Accepted, Refused]
      transitions:
        - {name: accept, from: [Picked], to: Accepted}
        - {name: refuse, from: [Picked], to: Refused}
actors:
  - name: demo.desk.Clerk
    may: [demo.desk.MakePick, demo.desk.CheckPick]
commands:
  - name: demo.desk.MakePick
    input:
      - {name: revision, type: Integer}
    outcomes:
      - name: picked
        creates: demo.desk.Pick
        instance: pick_id
        sets: {revision: input.revision}
        emits: [demo.desk.Picked]
        payload:
          demo.desk.Picked: {pick_id: {generated: true}}
  - name: demo.desk.CheckPick
    input:
      - {name: pick_id, type: demo.desk.PickId}
      - {name: revision, type: Integer}
    outcomes:
      - name: too-big
        when: revision > 100
        error: demo.desk.TooBig
      - name: stale
        when_subject:
          predicate: revision != input.revision
        moves: demo.desk.Pick.refuse
        instance: pick_id
        emits: [demo.desk.PickStale]
        payload:
          demo.desk.PickStale: {pick_id: input.pick_id}
      - name: accepted
        moves: demo.desk.Pick.accept
        instance: pick_id
        emits: [demo.desk.PickAccepted]
        payload:
          demo.desk.PickAccepted: {pick_id: input.pick_id}
      - name: wrong-state
        wrong_state: true
        error: demo.desk.NotPicked
      - name: unknown
        unknown_instance: true
        error: demo.desk.NoPick
events:
  - name: demo.desk.Picked
    fields: [{name: pick_id, type: demo.desk.PickId}]
  - name: demo.desk.PickStale
    fields: [{name: pick_id, type: demo.desk.PickId}]
  - name: demo.desk.PickAccepted
    fields: [{name: pick_id, type: demo.desk.PickId}]
errors:
  - name: demo.desk.NotPicked
  - name: demo.desk.NoPick
  - name: demo.desk.TooBig
views:
  - name: demo.desk.Picks
    source: demo.desk.Pick
    consistency: read_your_writes
    fields:
      - {name: pick_id, type: demo.desk.PickId}
      - {name: revision, type: Integer}
      - {name: state, type: demo.desk.Pick.State}
";

fn ir() -> EssIr {
    let raw = RawSpecFile::parse(MODEL).unwrap_or_else(|error| panic!("{error}"));
    let spec = Specification::assemble([(Source::new("model.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("the model validates: {errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn name(text: &str) -> QualifiedName {
    QualifiedName::new(text).unwrap()
}

fn integer(value: i64) -> Node {
    Node::Number(Number::from(value))
}

/// The branches `CheckPick` answers for `revision: 200`, on a pick holding revision 1 where `held`,
/// else on a store holding no pick.
fn answered(ir: &EssIr, held: bool) -> BTreeSet<String> {
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
        ("revision".to_owned(), integer(200)),
    ]);
    let store = if held {
        made.next.clone()
    } else {
        Store::default()
    };
    execute(
        ir,
        &store,
        &name("demo.desk.CheckPick"),
        &sent,
        &Externals::Withheld,
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

fn exchanged(first: Phase, second: Phase) -> [Phase; 8] {
    let mut order = Phase::PRECEDENCE;
    let at = |phase| order.iter().position(|held| *held == phase).unwrap();
    let (a, b) = (at(first), at(second));
    order.swap(a, b);
    order
}

/// Step 2 and step 4 exchanged: the held state reads before the input refusals, so a pick whose
/// revision differs is `stale` before `too-big` is read.
#[test]
fn held_state_before_input_refusal_answers_the_held_state() {
    let ir = ir();
    assert_eq!(
        answered(&ir, true),
        one("too-big"),
        "control: the real plan"
    );
    let moved = with_phase_order(exchanged(Phase::InputRefusal, Phase::HeldState), || {
        answered(&ir, true)
    });
    assert_eq!(
        moved,
        one("stale"),
        "with HeldState read before InputRefusal the plan answers `stale`; the interpreter's \
         `refused_by_input` still answers first"
    );
}

/// Step 2 and step 3 exchanged: existence reads before the input refusals, so a pick nobody holds
/// is `unknown` before `too-big` is read.
#[test]
fn existence_before_input_refusal_answers_unknown_instance() {
    let ir = ir();
    assert_eq!(
        answered(&ir, false),
        one("too-big"),
        "control: the real plan"
    );
    let moved = with_phase_order(exchanged(Phase::InputRefusal, Phase::Existence), || {
        answered(&ir, false)
    });
    assert_eq!(
        moved,
        one("unknown"),
        "with Existence read before InputRefusal the plan answers `unknown`; the interpreter's \
         held-subject loop answers it only after `refused_by_input`"
    );
}
