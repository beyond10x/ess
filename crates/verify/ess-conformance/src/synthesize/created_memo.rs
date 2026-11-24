//! The row a creating branch makes ([`super::created_owned`]), made once per model and context and
//! answered from memory after that.
//!
//! Every scenario that acts on a row starts by creating one, and most ask exactly what the one
//! before them asked: the same creator, the same input, the same owner. Each invariant scenario of
//! an entity runs another of its branches on a row arranged afresh, and the stored-row searches
//! create the same first row under every goal; with a creator guarded by a related row, each of
//! those creations searched for that related row again. The row depends on nothing but the
//! arguments, the model and the thread's arrangement context (the keys [`super::made`] uses), so it
//! is kept per model, keyed on all of those, for as long as the model is being synthesized.
//!
//! Answering from memory does not change what a search decides: the beside searches the creation
//! cut ([`super::related_guard::cuts`]) count as cut again.
use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;

use super::{
    ActorRef, Arrangement, Distinction, Driver, EntityHandle, EssIr, Node, QualifiedName,
    Unreachable,
};

/// The rows created for one model.
struct Kept {
    ir: *const EssIr,
    rows: BTreeMap<String, Entry>,
}

/// One creation's result, and how many searches for a row beside it cut.
#[derive(Clone)]
struct Entry {
    made: Result<Arrangement, Unreachable>,
    cuts: usize,
}

thread_local! {
    /// The rows created per model, innermost model last ([`remember`]).
    static KEPT: RefCell<Vec<Kept>> = const { RefCell::new(Vec::new()) };
    /// How many creations were asked on this thread, and how many of them were made.
    static ASKED: Cell<u64> = const { Cell::new(0) };
    static MADE: Cell<u64> = const { Cell::new(0) };
    /// Set while every creation is made afresh ([`unkept`]).
    static UNKEPT: Cell<bool> = const { Cell::new(false) };
}

/// Keeps the rows created for one model until dropped. See [`remember`].
pub(super) struct Remembered {
    pushed: bool,
}

impl Drop for Remembered {
    fn drop(&mut self) {
        if self.pushed {
            KEPT.with(|kept| {
                kept.borrow_mut().pop();
            });
        }
    }
}

/// Remembers the rows created for `ir` until the returned guard drops. A model already remembered
/// innermost keeps its memory.
pub(super) fn remember(ir: &EssIr) -> Remembered {
    let at = std::ptr::from_ref(ir);
    let pushed = KEPT.with(|kept| {
        let mut kept = kept.borrow_mut();
        if kept.last().is_some_and(|top| std::ptr::eq(top.ir, at)) {
            return false;
        }
        kept.push(Kept {
            ir: at,
            rows: BTreeMap::new(),
        });
        true
    });
    Remembered { pushed }
}

/// What `run` returns, how many creations it asked and how many of them were made rather than
/// answered from memory.
pub(super) fn counting<R>(run: impl FnOnce() -> R) -> (R, u64, u64) {
    let asked = ASKED.with(Cell::get);
    let made = MADE.with(Cell::get);
    let out = run();
    (
        out,
        ASKED.with(Cell::get) - asked,
        MADE.with(Cell::get) - made,
    )
}

/// The count of [`ASKED`] on this thread.
pub(super) fn taken_asked() -> u64 {
    ASKED.with(std::cell::Cell::get)
}

/// The count of [`MADE`] on this thread.
pub(super) fn taken_made() -> u64 {
    MADE.with(std::cell::Cell::get)
}

/// Adds `more` to the count of [`MADE`] on this thread.
pub(super) fn add_made(more: u64) {
    MADE.with(|it| it.set(it.get() + more));
}

/// Adds `more` to the count of [`ASKED`] on this thread.
pub(super) fn add_asked(more: u64) {
    ASKED.with(|it| it.set(it.get() + more));
}

/// What `run` returns with every creation made afresh, as before the memory existed.
pub(super) fn unkept<R>(run: impl FnOnce() -> R) -> R {
    let previous = UNKEPT.with(|it| it.replace(true));
    let out = run();
    UNKEPT.with(|it| it.set(previous));
    out
}

/// Whether every one is made afresh on this thread ([`unkept`]).
pub(super) fn is_unkept() -> bool {
    UNKEPT.with(Cell::get)
}

/// The key a creation is kept under: every argument and every thread-local the creation reads.
/// `None` for a creator the model does not declare as such (one projected for a related guard),
/// which has no stable name to be kept by.
#[allow(clippy::too_many_arguments)]
fn key(
    ir: &EssIr,
    entity: &EntityHandle,
    creator: &Driver<'_>,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    distinction: Distinction,
    arranging: &[&EntityHandle],
    owner: Option<&(String, Arrangement)>,
    input: Option<&BTreeMap<String, Node>>,
) -> Option<String> {
    let declared = ir
        .commands()
        .get(&creator.command.name)
        .is_some_and(|command| std::ptr::eq(command, creator.command));
    let outcome = creator
        .command
        .outcomes
        .iter()
        .position(|outcome| std::ptr::eq(outcome, creator.outcome))
        .filter(|_| declared)?;
    Some(format!(
        "{entity:?}|{}|{outcome}|{:?}|{actors:?}|{distinction:?}|{arranging:?}|{owner:?}|{input:?}|{}|{:?}|{}|{}|{}",
        creator.command.name,
        creator.effect,
        super::subject_fact::held_key(ir),
        super::earlier_runs::captured(),
        crate::witness::under_test_key(),
        super::related_guard::open_key(),
        super::dropped_key(),
    ))
}

/// `make`'s row, from memory where this model asked the same creation in the same context before.
/// A model not remembered innermost is answered by `make` every time.
#[allow(clippy::too_many_arguments)]
pub(super) fn created_owned(
    ir: &EssIr,
    entity: &EntityHandle,
    creator: &Driver<'_>,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    distinction: Distinction,
    arranging: &[&EntityHandle],
    owner: Option<&(String, Arrangement)>,
    input: Option<&BTreeMap<String, Node>>,
    make: impl FnOnce() -> Result<Arrangement, Unreachable>,
) -> Result<Arrangement, Unreachable> {
    ASKED.with(|count| count.set(count.get() + 1));
    let held = !UNKEPT.with(Cell::get)
        && KEPT.with(|kept| {
            kept.borrow()
                .last()
                .is_some_and(|top| std::ptr::eq(top.ir, ir))
        });
    let key = held
        .then(|| {
            key(
                ir,
                entity,
                creator,
                actors,
                distinction,
                arranging,
                owner,
                input,
            )
        })
        .flatten();
    let Some(key) = key else {
        MADE.with(|count| count.set(count.get() + 1));
        return make();
    };
    let known = KEPT.with(|kept| {
        kept.borrow()
            .last()
            .and_then(|top| top.rows.get(&key).cloned())
    });
    if let Some(known) = known {
        super::related_guard::add_cuts(known.cuts);
        return known.made;
    }
    // No borrow is held while making: the creations below ask the memory too.
    let before = super::related_guard::cuts();
    MADE.with(|count| count.set(count.get() + 1));
    let created = make();
    let cuts = super::related_guard::cuts().saturating_sub(before);
    KEPT.with(|kept| {
        if let Some(top) = kept.borrow_mut().last_mut() {
            if std::ptr::eq(top.ir, ir) {
                top.rows.insert(
                    key,
                    Entry {
                        made: created.clone(),
                        cuts,
                    },
                );
            }
        }
    });
    created
}
