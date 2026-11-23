//! A further row of an entity arranged at the cheapest of some states ([`super::arrange_first`]),
//! made once per model and context ([`super::made`]) rather than once per scenario that asks.
//!
//! The same row is asked for by every scenario whose subject is the entity a ranked view shows
//! (`arrange_beside`) and by every arrangement of a command reading a related row of the entity
//! (`related_guard::row_at`), at the same distinction: nothing it is made from differs between
//! them. Its key holds everything the arrangement reads beside its arguments, as every key of
//! [`super::made`] does: the rows held, the instances earlier runs captured, the under-test scope,
//! the searches for a row beside open, and the dropped write of a mutant.

use std::collections::BTreeMap;

use crate::scenario::EntityRef;
use ess_compiler::ir::{EntityHandle, EssIr};
use ess_domain::entity::StateName;
use ess_domain::name::QualifiedName;

use super::{
    arrange_first, ActorRef, Arrangement, Distinction, InstanceNeed, RefusalCause, Unreachable,
};

thread_local! {
    /// How many rows were asked for and how many were arranged on this thread, and whether every
    /// one is arranged afresh ([`first_rows_made`]).
    static WORK: std::cell::Cell<(u64, u64, bool)> = const { std::cell::Cell::new((0, 0, false)) };
}

/// `run`'s result, with how many further rows were asked for ([`arranged_first`]) and how many
/// were arranged rather than answered from memory on this thread while it ran. Where `fresh` is
/// set, every one is arranged, as before. The work a test bounds; not part of the suite.
#[doc(hidden)]
pub fn first_rows_made<R>(fresh: bool, run: impl FnOnce() -> R) -> (R, u64, u64) {
    let (asked, made, was) = WORK.with(std::cell::Cell::get);
    WORK.with(|work| work.set((asked, made, fresh)));
    let out = run();
    let (after_asked, after_made, _) = WORK.with(std::cell::Cell::get);
    WORK.with(|work| work.set((after_asked, after_made, was)));
    (out, after_asked - asked, after_made - made)
}

/// How many further rows were asked for and how many were arranged on this thread ([`WORK`]).
pub(super) fn taken_work() -> (u64, u64) {
    let (asked, made, _) = WORK.with(std::cell::Cell::get);
    (asked, made)
}

/// Adds `more`, the rows asked for and the rows arranged, to the counts of [`WORK`] on this
/// thread.
pub(super) fn add_work(more: (u64, u64)) {
    WORK.with(|work| {
        let (asked, made, fresh) = work.get();
        work.set((asked + more.0, made + more.1, fresh));
    });
}

/// Whether every row is arranged afresh on this thread ([`first_rows_made`]).
pub(super) fn fresh() -> bool {
    WORK.with(std::cell::Cell::get).2
}

/// [`arrange_first`] of `entity` at `targets`, under `distinction`, arranged for no other entity,
/// from memory where this model asked the same in the same context before.
pub(super) fn arranged_first(
    ir: &EssIr,
    entity: &EntityHandle,
    targets: &[StateName],
    actors: &BTreeMap<QualifiedName, ActorRef>,
    distinction: Distinction,
) -> Result<Arrangement, Unreachable> {
    let (_, _, fresh) = WORK.with(|work| {
        let (asked, made, fresh) = work.get();
        work.set((asked + 1, made, fresh));
        work.get()
    });
    let make = || {
        WORK.with(|work| {
            let (asked, made, fresh) = work.get();
            work.set((asked, made + 1, fresh));
        });
        arrange_first(ir, entity, targets, actors, distinction, &[])
    };
    if fresh {
        return make();
    }
    let key = || {
        format!(
            "arrange_first|{entity:?}|{targets:?}|{actors:?}|{distinction:?}|{}|{:?}|{}|{}|{}",
            super::subject_fact::held_key(ir),
            super::earlier_runs::captured(),
            crate::witness::under_test_key(),
            super::related_guard::open_key(),
            super::dropped_key(),
        )
    };
    // The memory holds rows and refusals: an unreachable row is kept as the refusal carrying it.
    let kept = super::made::made(ir, key, || {
        make()
            .map(|row| vec![row])
            .map_err(|reason| RefusalCause::InstanceRequired {
                entity: EntityRef::from(entity),
                need: InstanceNeed::Updates,
                reason,
            })
    });
    match kept {
        Ok(mut rows) if rows.len() == 1 => Ok(rows.remove(0)),
        Err(RefusalCause::InstanceRequired { reason, .. }) => Err(reason),
        // Nothing else is kept under this key.
        _ => make(),
    }
}
