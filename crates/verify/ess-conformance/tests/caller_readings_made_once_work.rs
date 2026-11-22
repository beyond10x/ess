//! A model whose actors carry attributes arranges its rows once per distinct caller reading, not
//! once per command each reading is focused on.
//!
//! Such a model is read once per caller assignment: whole as the first caller, whole as the second,
//! and twice more, focused, for every command the two callers send differently, which is every
//! command granted to an actor with attributes. Each reading was rewritten afresh, a model of its
//! own, so each focused run arranged again every row the whole run had arranged. On a model of
//! one entity whose fields several commands set, the caller attributes cost nine to seventeen
//! times the instructions of the same model with the owner sent as an input. A command that reads
//! no caller reads the same under either caller, so those readings are one model, and what one
//! synthesis of it arranged is kept for the next. Synthesis is held to a bound on the creating
//! rows it arranges, against the same model without caller attributes: a work bound, not a clock.

use std::fmt::Write as _;

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::synthesize::{creations_made, synthesize, Synthesis};
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source,
};

/// How many flags the card carries, each set by a command of its own.
const FLAGS: usize = 4;

/// How many times the creating rows the plain model arranges the caller model may arrange: one
/// whole reading per caller, and the readings of the two commands that read the caller, each a
/// model of its own. Arranging afresh in every focused reading took 34 against the plain 5;
/// keeping each reading's rows takes 18.
const TIMES: u64 = 4;

/// A card whose flags several commands set, each refusing a flag already set, and an owner that
/// only the owner may close: with `caller`, the owner is the opening caller's and closing compares
/// the caller; without, both send the owner as an input.
#[allow(clippy::too_many_lines)] // One model, written out in source order.
fn model(caller: bool) -> String {
    let flags = 1..=FLAGS;
    let mut text = String::from(
        "format: ess/37\nsystem: demo\nversion: v1\ndomain: demo.board\n\
         summary: Cards whose flags several commands set, each refusing a flag already set.\n\
         types:\n  - {name: demo.board.CardId, kind: newtype, of: String}\n\
         \x20 - {name: demo.board.OwnerId, kind: newtype, of: String}\n\
         entities:\n  - name: demo.board.Card\n    identity: {name: card_id, type: demo.board.CardId}\n\
         \x20   fields:\n      - {name: owner_id, type: demo.board.OwnerId}\n",
    );
    for i in flags.clone() {
        writeln!(text, "      - {{name: flag_{i}, type: Boolean}}").unwrap();
    }
    text.push_str(
        "    lifecycle:\n      initial: Open\n      states: [Open, Closed]\n      terminal: [Closed]\n\
         \x20     transitions:\n",
    );
    for i in flags.clone() {
        writeln!(text, "        - {{name: mark_{i}, from: [Open], to: Open}}").unwrap();
    }
    text.push_str(
        "        - {name: close, from: [Open], to: Closed}\nactors:\n  - name: demo.board.Clerk\n",
    );
    if caller {
        text.push_str("    attributes:\n      - {name: owner_id, type: demo.board.OwnerId}\n");
    }
    text.push_str("    may: [demo.board.Open, demo.board.Close");
    for i in flags.clone() {
        write!(text, ", demo.board.Mark{i}").unwrap();
    }
    text.push_str("]\nerrors:\n");
    for error in ["NoSuchCard", "AlreadyMarked", "StateConflict", "NotOwner"] {
        writeln!(
            text,
            "  - name: demo.board.{error}\n    fields:\n      - {{name: card_id, type: demo.board.CardId}}"
        )
        .unwrap();
    }
    let owner_input = if caller {
        ""
    } else {
        "      - {name: owner_id, type: demo.board.OwnerId}\n"
    };
    write!(
        text,
        "commands:\n  - name: demo.board.Open\n    input:\n      - {{name: card_id, type: demo.board.CardId}}\n\
         {owner_input}    outcomes:\n      - name: opened\n        creates: demo.board.Card\n\
         \x20       instance: card_id\n        sets:\n          owner_id: {}\n",
        if caller {
            "{caller: owner_id}"
        } else {
            "input.owner_id"
        }
    )
    .unwrap();
    for i in flags.clone() {
        writeln!(text, "          flag_{i}: false").unwrap();
    }
    text.push_str(
        "        emits: [demo.board.Opened]\n        payload:\n          demo.board.Opened: {card_id: input.card_id}\n",
    );
    for i in flags.clone() {
        write!(
            text,
            "  - name: demo.board.Mark{i}\n    input:\n      - {{name: card_id, type: demo.board.CardId}}\n\
             \x20     - {{name: on, type: Boolean}}\n    outcomes:\n      - name: no-card\n\
             \x20       unknown_instance: true\n        error: demo.board.NoSuchCard\n\
             \x20     - name: already-marked\n        when_subject:\n          predicate: flag_{i} == true\n\
             \x20       error: demo.board.AlreadyMarked\n      - name: marked\n\
             \x20       moves: demo.board.Card.mark_{i}\n        instance: card_id\n        sets:\n\
             \x20         flag_{i}: input.on\n        emits: [demo.board.Marked]\n        payload:\n\
             \x20         demo.board.Marked: {{card_id: input.card_id}}\n      - name: wrong-state\n\
             \x20       wrong_state: true\n        error: demo.board.StateConflict\n"
        )
        .unwrap();
    }
    write!(
        text,
        "  - name: demo.board.Close\n    input:\n      - {{name: card_id, type: demo.board.CardId}}\n\
         {owner_input}    outcomes:\n      - name: no-card\n        unknown_instance: true\n\
         \x20       error: demo.board.NoSuchCard\n      - name: not-owner\n        when_subject:\n\
         \x20         predicate: owner_id != {}\n        error: demo.board.NotOwner\n\
         \x20     - name: closed\n        moves: demo.board.Card.close\n        instance: card_id\n\
         \x20       emits: [demo.board.Closed]\n        payload:\n\
         \x20         demo.board.Closed: {{card_id: input.card_id}}\n      - name: wrong-state\n\
         \x20       wrong_state: true\n        error: demo.board.StateConflict\nevents:\n",
        if caller {
            "caller.owner_id"
        } else {
            "input.owner_id"
        }
    )
    .unwrap();
    for event in ["Opened", "Marked", "Closed"] {
        writeln!(
            text,
            "  - name: demo.board.{event}\n    fields:\n      - {{name: card_id, type: demo.board.CardId}}"
        )
        .unwrap();
    }
    text.push_str(
        "views:\n  - name: demo.board.Cards\n    source: demo.board.Card\n    consistency: read_your_writes\n\
         \x20   fields:\n      - {name: card_id, type: demo.board.CardId}\n\
         \x20     - {name: state, type: demo.board.Card.State}\n\
         \x20     - {name: owner_id, type: demo.board.OwnerId}\n",
    );
    for i in flags {
        writeln!(text, "      - {{name: flag_{i}, type: Boolean}}").unwrap();
    }
    text
}

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("board.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn refusals(synthesis: &Synthesis) -> Vec<String> {
    synthesis.refusals.iter().map(ToString::to_string).collect()
}

#[test]
fn caller_readings_arrange_each_row_about_once_per_caller() {
    let (plain, plain_made) = creations_made(|| synthesize(&ir(&model(false))));
    let (caller, caller_made) = creations_made(|| synthesize(&ir(&model(true))));
    assert!(
        refusals(&plain).is_empty(),
        "refusals: {:#?}",
        refusals(&plain)
    );
    assert!(
        refusals(&caller).is_empty(),
        "refusals: {:#?}",
        refusals(&caller)
    );
    assert!(plain_made > 0, "the plain model arranges no creating row");
    assert!(
        caller_made <= TIMES * plain_made,
        "the caller model arranged creating rows {caller_made} times, more than {TIMES} times the \
         {plain_made} of the same model without caller attributes"
    );
}
