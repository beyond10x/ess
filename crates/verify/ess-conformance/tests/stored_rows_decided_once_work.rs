//! Which branch a command selects on a stored row for an input is decided once per synthesis, not
//! once per search that advances the row.
//!
//! A search for a row a branch reads advances every row it meets by every move the model declares.
//! For a move of a command that itself reads stored fields, it checks every input it offers for
//! the branch the command selects on that row. Every moving branch of that command asks the same
//! question, and so does every other search whose goal differs, because each branch reading a
//! different field searches with different hints. A downstream specification of 110 commands took
//! more than half an hour to synthesize, nine in ten of these checks repeating one made before.
//! Synthesis of a model of one entity whose fields twelve commands set, each refusing a row whose
//! own field is already set, is held to a bound on the rows and inputs it checks for the branch
//! they select, a work bound rather than a clock.

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::synthesize::{
    branch_selections_tried, entity_view_indexes_built, kept_edges_equal_fresh,
    kept_view_index_equals_fresh, lifecycle_edges_derived, synthesize, Synthesis,
};
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source,
};
use std::fmt::Write as _;

const MODEL: &str = include_str!("fixtures/stored-rows-decided-once.yaml");

/// The rows and inputs the whole synthesis may check for the branch they select: 204 when each
/// command's pick on a row is kept for the synthesis, 348 when every search decides it anew.
const BOUND: u64 = 260;

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("board.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn ids(synthesis: &Synthesis) -> Vec<String> {
    synthesis
        .suite
        .scenarios
        .keys()
        .map(ToString::to_string)
        .collect()
}

fn refusals(synthesis: &Synthesis) -> Vec<String> {
    synthesis.refusals.iter().map(ToString::to_string).collect()
}

#[test]
fn a_row_many_commands_set_is_decided_within_a_bound() {
    let ir = ir(MODEL);
    let (synthesis, tried) = branch_selections_tried(|| synthesize(&ir));
    assert!(
        refusals(&synthesis).is_empty(),
        "refusals: {:#?}",
        refusals(&synthesis)
    );
    assert!(
        tried <= BOUND,
        "rows and inputs were checked {tried} times for the branch they select, more than \
         {BOUND}\nscenarios: {:#?}",
        ids(&synthesis)
    );
}

/// An entity whose lifecycle is a chain of seven states, one command per move, and `spares` further
/// views of the entity that no scenario reads.
fn chain(spares: usize) -> String {
    let mut text = String::from(
        "format: ess/25\nsystem: shop\nversion: v1\ndomain: shop.order\ntypes:\n  \
         - {name: shop.order.Id, kind: newtype, of: Uuid}\nentities:\n  \
         - name: shop.order.Order\n    identity: {name: id, type: shop.order.Id}\n    fields: []\n    \
         lifecycle:\n      initial: State0\n      \
         states: [State0, State1, State2, State3, State4, State5, State6]\n      \
         terminal: [State6]\n      transitions:\n",
    );
    for n in 1..=6 {
        let _ = writeln!(
            text,
            "        - {{name: move-{n}, from: [State{}], to: State{n}}}",
            n - 1
        );
    }
    text.push_str(
        "errors: [{name: shop.order.Conflict, fields: []}]\ncommands:\n  - name: shop.order.Create\n    \
         outcomes:\n      - name: created\n        creates: shop.order.Order\n        instance: id\n        \
         emits: [shop.order.Created]\n        payload: {shop.order.Created: {id: {generated: true}}}\n",
    );
    for n in 1..=6 {
        let _ = writeln!(
            text,
            "  - name: shop.order.Move{n}\n    input: [{{name: id, type: shop.order.Id}}]\n    outcomes:\n      \
             - {{name: moved, moves: shop.order.Order.move-{n}, instance: id, emits: [shop.order.Moved], \
             payload: {{shop.order.Moved: {{id: input.id}}}}}}\n      \
             - {{name: wrong-state, wrong_state: true, error: shop.order.Conflict}}"
        );
    }
    text.push_str(
        "events:\n  - {name: shop.order.Created, fields: [{name: id, type: shop.order.Id}]}\n  \
         - {name: shop.order.Moved, fields: [{name: id, type: shop.order.Id}]}\nviews:\n  \
         - {name: shop.order.Orders, source: shop.order.Order, consistency: read_your_writes, \
         fields: [{name: id, type: shop.order.Id}, {name: state, type: shop.order.Order.State}]}\n",
    );
    for n in 0..spares {
        let _ = writeln!(
            text,
            "  - {{name: shop.order.Spares{n}, source: shop.order.Order, consistency: eventual, \
             fields: [{{name: id, type: shop.order.Id}}]}}"
        );
    }
    text
}

/// The seven-state chain has six edges. 46 searches rebuilt the graph each time (276 derivations);
/// it is derived once per lifecycle for the synthesis, and equals a fresh derivation.
#[test]
fn a_lifecycle_graph_is_derived_once_per_synthesis() {
    let ir = ir(&chain(0));
    let (synthesis, derived) = lifecycle_edges_derived(|| synthesize(&ir));
    assert!(
        refusals(&synthesis).is_empty(),
        "refusals: {:#?}",
        refusals(&synthesis)
    );
    assert_eq!(derived, 6, "edges derived for a chain of six moves");
    assert!(kept_edges_equal_fresh(&ir));
}

/// Eleven scenario bodies each rebuilt the entity-to-view index (six builds on this model, each
/// costing in proportion to the views of the model, 80 unrelated ones here); it is built once per
/// synthesis, and equals a fresh one.
#[test]
fn the_entity_to_view_index_is_built_once_per_synthesis() {
    let ir = ir(&chain(80));
    let (synthesis, built) = entity_view_indexes_built(|| synthesize(&ir));
    assert!(
        refusals(&synthesis).is_empty(),
        "refusals: {:#?}",
        refusals(&synthesis)
    );
    assert_eq!(built, 1, "entity-to-view indexes built");
    assert!(kept_view_index_equals_fresh(&ir));
}
