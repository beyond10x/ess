//! How the documentation reads a set effect (ess/16, beyond10x/ess#167, #175,
//! `docs/design/set-effects-over-filtered-instances.md`): which rows a branch changes, by the
//! filter that selects them, and what each comes to hold.
//!
//! The other artifacts carry no set effect: `OpenAPI` and `AsyncAPI` publish the request and the events
//! as for a branch that names no instance, which is what the wire of such a branch is.

use ess_compiler::ir::{
    EssIr, ResolvedAffect, ResolvedEffect, ResolvedOutcome, ResolvedPayloadField,
    ResolvedSetSubject,
};

use crate::document::Inline;

/// What a set subject does, as a sentence.
pub(crate) fn set_sentence(ir: &EssIr, set: &ResolvedSetSubject) -> Vec<Inline> {
    let entity = ir.entity(&set.entity);
    let mut out = match &set.effect {
        ResolvedEffect::Moves { transition } => vec![
            Inline::text("It moves every "),
            Inline::code(entity.name.to_string()),
            Inline::text(" its filter "),
            Inline::code(set.filter.to_string()),
            Inline::text(" selects to "),
            Inline::code(transition.to.to_string()),
            Inline::text(", along the declared move "),
            Inline::code(transition.name.clone()),
            Inline::text(
                "; a selected row resting outside the move's starting states is left as it is.",
            ),
        ],
        _ => vec![
            Inline::text("It changes every "),
            Inline::code(entity.name.to_string()),
            Inline::text(" its filter "),
            Inline::code(set.filter.to_string()),
            Inline::text(" selects, without moving it along its lifecycle."),
        ],
    };
    out.push(Inline::text(
        " No selected row at all is an accepted answer.",
    ));
    out
}

/// The `affects:` entries of a branch, one sentence each; nothing for a branch declaring none.
pub(crate) fn affects_sentences(ir: &EssIr, outcome: &ResolvedOutcome) -> Vec<Inline> {
    let mut out = Vec::new();
    for affect in &outcome.affects {
        out.extend(affect_sentence(ir, affect));
    }
    out
}

fn affect_sentence(ir: &EssIr, affect: &ResolvedAffect) -> Vec<Inline> {
    let entity = ir.entity(&affect.entity);
    // From ess/22 an entry may move its rows (beyond10x/ess#229); an entry that only sets fields
    // reads as it always did.
    let Some(transition) = &affect.moves else {
        let mut out = vec![
            Inline::text(" Beside its subject, it changes every "),
            Inline::code(entity.name.to_string()),
            Inline::text(" the filter "),
            Inline::code(affect.filter.to_string()),
            Inline::text(" selects, the subject itself excepted"),
        ];
        out.extend(assignments(&affect.sets));
        out.push(Inline::text("."));
        return out;
    };
    let mut out = vec![
        Inline::text(" Beside its subject, it moves every "),
        Inline::code(entity.name.to_string()),
        Inline::text(" the filter "),
        Inline::code(affect.filter.to_string()),
        Inline::text(" selects to "),
        Inline::code(transition.to.to_string()),
        Inline::text(", along the declared move "),
        Inline::code(transition.name.clone()),
        Inline::text(", the subject itself excepted"),
    ];
    out.extend(assignments(&affect.sets));
    out.push(Inline::text(
        "; a selected row resting outside the move's starting states is left as it is.",
    ));
    out
}

fn assignments(sets: &[ResolvedPayloadField]) -> Vec<Inline> {
    let mut out = Vec::new();
    for (index, field) in sets.iter().enumerate() {
        out.push(Inline::text(if index == 0 { ": " } else { ", " }));
        out.push(Inline::code(field.target.clone()));
        out.push(Inline::text(format!(" becomes {}", field.value.describe())));
    }
    out
}
