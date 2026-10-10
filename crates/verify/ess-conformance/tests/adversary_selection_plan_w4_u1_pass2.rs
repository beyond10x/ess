//! Adversary pass 2 of `story:interpreter-reads-selection-plan` (wave 4, unit U1).
//!
//! Pass 2 defers the pre-`select` input-refusal step once the walk passes a phase `select` reads,
//! and moves held-row gathering (`held_rows`) to the plan's `Existence` phase. Two exchanges the
//! plan answers one way and the interpreter another, neither covered by "related-row and existence
//! answer before `select`":
//!
//! 1. Exchanging two phases the command holds nothing that answers in (`InputAbsent` and
//!    `Accepting`, whose one branch does not hold) leaves `InputRefusal` before `Existence` in the
//!    plan, yet the interpreter now answers `unknown_instance:` before the input refusal.
//! 2. `held_rows` reads every input-named related row at `Existence`. Read before `RelatedRow`, a
//!    missing related row is an Undetermined error instead of its `exists: false` branch.
use std::collections::BTreeMap;

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::interpret::execute::{execute, Externals, Store};
use ess_domain::{
    command::precedence::{with_phase_order, Phase},
    name::QualifiedName,
    spec::{RawSpecFile, Specification},
    system::Source,
};
use ess_primitives::{facts::Number, node::Node};

const PICK: &str = r"format: ess/20
system: demo
version: v1
domain: demo.desk
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
      - {name: rush, type: Boolean}
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
      - name: rushed
        when: rush == true
        moves: demo.desk.Pick.accept
        instance: pick_id
        emits: [demo.desk.PickAccepted]
        payload:
          demo.desk.PickAccepted: {pick_id: input.pick_id}
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

const MULTIPLE: &str = include_str!("fixtures/related-guard-multiple.yaml");

fn compiled(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("m.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("the model validates: {errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn input(pairs: &[(&str, Node)]) -> BTreeMap<String, Node> {
    pairs
        .iter()
        .map(|(name, value)| ((*name).to_owned(), value.clone()))
        .collect()
}

/// The branches `command` answers, by name, or the interpreter's error.
fn answer(ir: &EssIr, store: &Store, command: &str, sent: &[(&str, Node)]) -> String {
    match execute(
        ir,
        store,
        &QualifiedName::new(command).unwrap(),
        &input(sent),
        &Externals::Withheld,
    ) {
        Ok(steps) => steps
            .iter()
            .map(|step| {
                step.outcome
                    .as_ref()
                    .map_or("none".to_owned(), ToString::to_string)
            })
            .collect::<Vec<_>>()
            .join(" + "),
        Err(why) => format!("Undetermined: {why}"),
    }
}

fn next(ir: &EssIr, store: &Store, command: &str, sent: &[(&str, Node)]) -> Store {
    let steps = execute(
        ir,
        store,
        &QualifiedName::new(command).unwrap(),
        &input(sent),
        &Externals::Withheld,
    )
    .unwrap_or_else(|why| panic!("{command}: {why}"));
    steps[0].next.clone()
}

fn exchanged(first: Phase, second: Phase) -> [Phase; 8] {
    let mut order = Phase::PRECEDENCE;
    let at = |phase| order.iter().position(|held| *held == phase).unwrap();
    let (a, b) = (at(first), at(second));
    order.swap(a, b);
    order
}

fn int(value: i64) -> Node {
    Node::Number(Number::from(value))
}

/// `InputAbsent` and `Accepting` exchanged: the plan still reads `InputRefusal` (position 2) before
/// `Existence` (position 3), and `rushed`, the one accepting branch, does not hold. A pick nobody
/// holds, sent `revision: 200`, is `too-big` under both orders. The interpreter, having passed
/// `Accepting`, leaves the input refusals to `select` and answers `unknown` from `held_rows` first.
#[test]
fn exchanging_two_phases_that_do_not_answer_keeps_the_input_refusal_before_existence() {
    let ir = compiled(PICK);
    let made = next(
        &ir,
        &Store::default(),
        "demo.desk.MakePick",
        &[("revision", int(1))],
    );
    let (_, pick, _) = made.instances().next().expect("MakePick creates a pick");
    let sent = [
        ("pick_id", pick.clone()),
        ("revision", int(200)),
        ("rush", Node::Bool(false)),
    ];
    let ask = || answer(&ir, &Store::default(), "demo.desk.CheckPick", &sent);
    assert_eq!(
        ask(),
        "demo.desk.CheckPick/too-big",
        "control: the real plan"
    );
    let order = exchanged(Phase::InputAbsent, Phase::Accepting);
    assert!(
        Phase::InputRefusal.position() < Phase::Existence.position()
            && with_phase_order(order, || {
                Phase::InputRefusal.position() < Phase::Existence.position()
            }),
        "both orders read InputRefusal before Existence"
    );
    assert_eq!(
        with_phase_order(order, ask),
        "demo.desk.CheckPick/too-big",
        "InputAbsent <-> Accepting exchanged: InputRefusal still precedes Existence in the plan"
    );
}

/// `RelatedRow` and `Existence` exchanged on `related-guard-multiple.yaml`'s `StartRun`, whose
/// `Existence` phase holds no branch (`existing_instance:` is not declared). A capability nobody
/// holds is answered `no-such-capability` under both orders; the interpreter reads the related
/// rows in `held_rows` at `Existence` and answers Undetermined.
#[test]
fn existence_read_before_related_row_still_answers_the_missing_row() {
    let ir = compiled(MULTIPLE);
    let installed = next(&ir, &Store::default(), "demo.run.InstallSwitch", &[]);
    let (_, switch, _) = installed.instances().next().expect("a switch");
    let switch = switch.clone();
    let missing = Node::Text("00000000-0000-4000-8000-000000000077".to_owned());
    let sent = [("switch", switch), ("capability", missing)];
    let ask = || answer(&ir, &installed, "demo.run.StartRun", &sent);
    assert_eq!(
        ask(),
        "demo.run.StartRun/no-such-capability",
        "control: the real plan"
    );
    assert_eq!(
        with_phase_order(exchanged(Phase::RelatedRow, Phase::Existence), ask),
        "demo.run.StartRun/no-such-capability",
        "RelatedRow <-> Existence exchanged: Existence holds no branch of StartRun"
    );
}
