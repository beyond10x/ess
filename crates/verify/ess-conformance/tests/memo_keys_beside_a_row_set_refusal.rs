//! What synthesis writes into the keys of its memories does not grow with the number of commands
//! the model grants (hotfix 0.99.9).
//!
//! The memories of synthesis keep each answer under a key that names every argument and every
//! thread-local the answer reads. 0.99.8 wrote those keys out with `Debug` in full, and nearly
//! every key held the actor each command is sent as: one entry per granted command, written again
//! for every question asked. Once a creating branch was driven beside a row-set refusal (0.99.8),
//! the questions multiplied, and a specification of a few hundred scenarios and many commands took
//! eight times as long to synthesize as with 0.99.7 and held 17 GB. A key now writes that map as
//! a digest, rendered once per thread.
//!
//! The model is the committed fixture of a ticket pinned beside a row-set refusal over the tickets
//! of its shelf (`tests/fixtures/row-set-refusal-beside-a-named-related-row.yaml`), alone and with
//! [`NOTES`] further commands granted to the clerk, each creating a note of its own. The bound is
//! on the bytes of key renderings ([`key_bytes_rendered`]), a count of the work done and no clock.

use std::fmt::Write as _;

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::synthesize::{key_bytes_rendered, synthesize, Synthesis};
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source,
};

const BESIDE: &str = include_str!("fixtures/row-set-refusal-beside-a-named-related-row.yaml");

/// How many note commands the noted model grants beside the fixture's three.
const NOTES: usize = 40;

/// The row-set refusal whose scenario drives the creator beside it.
const CLEAR_PINNED: &str = "shop.store.ClearShelf/outcome/tickets-pinned";

/// `text` with `from`, which it holds once, replaced by `to`.
fn replaced(text: &str, from: &str, to: &str) -> String {
    assert_eq!(text.matches(from).count(), 1, "written once: {from}");
    text.replacen(from, to, 1)
}

/// The fixture with [`NOTES`] note commands granted to the clerk, and the entity they create.
fn noted() -> String {
    let names: Vec<String> = (0..NOTES)
        .map(|at| format!("shop.store.TakeNote{at}"))
        .collect();
    let granted = "    may: [shop.store.BuildShelf, shop.store.PinTicket, shop.store.ClearShelf";
    let mut text = replaced(BESIDE, granted, &format!("{granted}, {}", names.join(", ")));
    text = replaced(
        &text,
        "types:\n",
        "types:\n  - {name: shop.store.NoteId, kind: newtype, of: Uuid}\n",
    );
    text = replaced(
        &text,
        "entities:\n",
        "entities:\n  - name: shop.store.Note\n    identity: {name: note_id, type: \
         shop.store.NoteId}\n    fields:\n      - {name: body, type: String}\n    lifecycle: \
         {initial: Taken, states: [Taken], terminal: [Taken]}\n",
    );
    text = replaced(
        &text,
        "events:\n",
        "events:\n  - name: shop.store.NoteTaken\n    fields: [{name: note_id, type: \
         shop.store.NoteId}]\n",
    );
    let declared = names.iter().fold(String::new(), |mut declared, name| {
        write!(
            declared,
            "  - name: {name}\n    input:\n      - {{name: body, type: String}}\n    \
             outcomes:\n      - name: taken\n        creates: shop.store.Note\n        \
             instance: note_id\n        emits: [shop.store.NoteTaken]\n        payload: \
             {{shop.store.NoteTaken: {{note_id: {{generated: true}}}}}}\n        sets: \
             {{body: input.body}}\n"
        )
        .expect("a String takes every write");
        declared
    });
    replaced(&text, "commands:\n", &format!("commands:\n{declared}"))
}

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("shop.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

/// The bytes of key renderings the synthesis of `text` writes, the steps of its suite, and whether
/// the suite holds [`CLEAR_PINNED`].
fn written(text: &str) -> (u64, u64, bool) {
    let ir = ir(text);
    let (synthesis, bytes): (Synthesis, u64) = key_bytes_rendered(|| synthesize(&ir));
    let steps: u64 = synthesis
        .suite
        .scenarios
        .values()
        .map(|scenario| scenario.steps.len() as u64)
        .sum();
    let pinned = synthesis
        .suite
        .scenarios
        .keys()
        .any(|id| id.to_string() == CLEAR_PINNED);
    (bytes, steps, pinned)
}

/// Forty more granted commands add a quarter at most to the key bytes of the model: they add
/// scenarios of their own, which ask little, and no byte to the keys of the scenarios beside the
/// row-set refusal. 0.99.8's keys wrote 734,034 bytes for the fixture alone and 4,694,014 with
/// the notes; this release's write 461,383 and 464,613.
#[test]
fn the_keys_beside_a_row_set_refusal_do_not_grow_with_the_commands_granted() {
    let (alone, alone_steps, alone_pinned) = written(BESIDE);
    let (noted, noted_steps, noted_pinned) = written(&noted());
    eprintln!("key bytes {alone} alone, {noted} with {NOTES} notes");
    assert!(
        alone_pinned && noted_pinned,
        "{CLEAR_PINNED} is no longer synthesized: the model lost its shape"
    );
    assert!(
        noted_steps > alone_steps,
        "the notes added no step: {alone_steps} and {noted_steps}"
    );
    assert!(
        noted <= alone + alone / 4,
        "{noted} bytes of memo keys with {NOTES} more commands granted, {alone} without them \
         ({noted_steps} and {alone_steps} suite steps): a key writes something once per granted \
         command"
    );
}
