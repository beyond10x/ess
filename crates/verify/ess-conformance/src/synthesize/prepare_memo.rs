//! The branches the row-set strategy prepares ([`super::row_set::prepare`]), prepared once per
//! model, branch, actors and context while the model is remembered, and answered from memory after
//! that.
//!
//! One branch is prepared for its own scenario family, for the existence scenario of the row its
//! creating branch creates, for a set guard's binding effects and beside a sibling's `before_row`
//! refusal, and again per caller reading; each request searched every arrangement of its
//! selectors' counts afresh. A preparation reads nothing but the model, the branch, the actors and
//! the thread's context, every one of them a part of [`key`]: the context a caller reading carries
//! to another thread ([`Context`]) and the scopes open on this one (`threads.rs`, `PREPARE_KEY`).
//!
//! Answering from memory does not change what a search decides: the beside searches the
//! preparation cut ([`super::related_guard::cuts`]) count as cut again, and the result is shared,
//! not copied.
use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, BTreeSet};
use std::rc::Rc;

use super::row_set::{Prepared, Preparing};
use super::threads::Context;
use super::{EssIr, RefusalCause};

/// A preparation's result, shared by every request answered from it.
pub(super) type Shared = Rc<Result<Prepared, RefusalCause>>;

/// The preparations kept for one model.
struct Kept {
    ir: *const EssIr,
    prepared: BTreeMap<String, Entry>,
}

/// One preparation's result, and how many searches for a row beside it cut.
struct Entry {
    prepared: Shared,
    cuts: usize,
}

thread_local! {
    /// The preparations kept per model, innermost model last ([`remember`]).
    static KEPT: RefCell<Vec<Kept>> = const { RefCell::new(Vec::new()) };
    /// How many preparations were asked on this thread, and how many of them were made.
    static ASKED: Cell<u64> = const { Cell::new(0) };
    static MADE: Cell<u64> = const { Cell::new(0) };
    /// Set while every preparation is made afresh ([`unkept`]).
    static UNKEPT: Cell<bool> = const { Cell::new(false) };
    /// The key of every preparation made on this thread while [`counting`] runs.
    static KEYS: RefCell<Option<Vec<String>>> = const { RefCell::new(None) };
}

/// Keeps the preparations made for one model until dropped. See [`remember`].
pub(super) struct Remembered;

impl Drop for Remembered {
    fn drop(&mut self) {
        KEPT.with(|kept| {
            kept.borrow_mut().pop();
        });
    }
}

/// Remembers the preparations [`prepare`] makes for `ir` until the returned guard drops.
pub(super) fn remember(ir: &EssIr) -> Remembered {
    KEPT.with(|kept| {
        kept.borrow_mut().push(Kept {
            ir: std::ptr::from_ref(ir),
            prepared: BTreeMap::new(),
        });
    });
    Remembered
}

/// What `run` returns, how many preparations it asked, how many of them were made rather than
/// answered from memory, and the distinct keys of the ones made, on this thread.
pub(super) fn counting<R>(run: impl FnOnce() -> R) -> (R, u64, u64, BTreeSet<String>) {
    let asked = ASKED.with(Cell::get);
    let made = MADE.with(Cell::get);
    let outer = KEYS.with(|keys| keys.replace(Some(Vec::new())));
    let out = run();
    let keys = KEYS.with(|keys| keys.replace(outer)).unwrap_or_default();
    KEYS.with(|outer| {
        if let Some(outer) = outer.borrow_mut().as_mut() {
            outer.extend(keys.iter().cloned());
        }
    });
    (
        out,
        ASKED.with(Cell::get) - asked,
        MADE.with(Cell::get) - made,
        keys.into_iter().collect(),
    )
}

/// The count of [`ASKED`] on this thread.
pub(super) fn taken_asked() -> u64 {
    ASKED.with(Cell::get)
}

/// The count of [`MADE`] on this thread.
pub(super) fn taken_made() -> u64 {
    MADE.with(Cell::get)
}

/// Adds `more` to the count of [`ASKED`] on this thread.
pub(super) fn add_asked(more: u64) {
    ASKED.with(|it| it.set(it.get() + more));
}

/// Adds `more` to the count of [`MADE`] on this thread.
pub(super) fn add_made(more: u64) {
    MADE.with(|it| it.set(it.get() + more));
}

/// What `run` returns with every preparation made afresh, as before the memory existed.
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

/// The key of a preparation of `preparing`'s branch, the `position`th of its command: every input
/// [`super::row_set::prepare`] reads. Both structs are taken apart field by field, so a field
/// added to either does not compile until it is a part of the key or named here as none.
fn key(preparing: &Preparing<'_>, position: usize, context: &Context) -> String {
    let Preparing {
        arrangement,
        command,
        // By its position in the command, as `run_memo` keys a branch.
        outcome: _,
        actors,
    } = preparing;
    let Context {
        scope,
        dropped_write,
        dropped_grant,
        added_grant,
        phase_order,
        created_unkept,
        runs_unkept,
        first_fresh,
        attempts_fresh,
        companion_fresh,
        // None: while it is set nothing is kept or read.
        prepares_unkept: _,
    } = context;
    format!(
        "{}|{position}|{actors:?}|model={:p}|scope={scope:?}|dropped_write={dropped_write:?}|\
         dropped_grant={dropped_grant:?}|added_grant={added_grant:?}|phase_order={phase_order:?}|\
         created_unkept={created_unkept}|runs_unkept={runs_unkept}|first_fresh={first_fresh}|\
         attempts_fresh={attempts_fresh}|companion_fresh={companion_fresh}|held={}|earlier={:?}|\
         under_test={}|open={}",
        command.name,
        std::ptr::from_ref(*arrangement),
        super::subject_fact::held_key(arrangement),
        super::earlier_runs::captured(),
        crate::witness::under_test_key(),
        super::related_guard::open_key(),
    )
}

/// `make`'s preparation of `preparing`'s branch, from memory where the model remembered innermost
/// prepared it under the same key before. Anything else is made every time.
pub(super) fn prepare(
    preparing: &Preparing<'_>,
    make: impl FnOnce() -> Result<Prepared, RefusalCause>,
) -> Shared {
    ASKED.with(|count| count.set(count.get() + 1));
    let ir = preparing.arrangement;
    let held = !UNKEPT.with(Cell::get)
        && KEPT.with(|kept| {
            kept.borrow()
                .last()
                .is_some_and(|top| std::ptr::eq(top.ir, ir))
        });
    let counted = KEYS.with(|keys| keys.borrow().is_some());
    let declared = ir
        .commands()
        .get(&preparing.command.name)
        .is_some_and(|declared| std::ptr::eq(declared, preparing.command));
    let position = preparing
        .command
        .outcomes
        .iter()
        .position(|branch| std::ptr::eq(branch, preparing.outcome))
        .filter(|_| declared);
    let key = position
        .filter(|_| held || counted)
        .map(|position| key(preparing, position, &Context::current()));
    let known = key.as_ref().filter(|_| held).and_then(|key| {
        KEPT.with(|kept| {
            kept.borrow().last().and_then(|top| {
                top.prepared
                    .get(key)
                    .map(|entry| (Rc::clone(&entry.prepared), entry.cuts))
            })
        })
    });
    if let Some((known, cuts)) = known {
        super::related_guard::add_cuts(cuts);
        return known;
    }
    // No borrow is held while making: the preparation asks the memory too.
    let before = super::related_guard::cuts();
    MADE.with(|count| count.set(count.get() + 1));
    let prepared = Rc::new(make());
    let cuts = super::related_guard::cuts().saturating_sub(before);
    if counted {
        let named = key.clone().unwrap_or_else(|| {
            format!(
                "{}/{}|undeclared",
                preparing.command.name, preparing.outcome.name
            )
        });
        KEYS.with(|keys| {
            if let Some(keys) = keys.borrow_mut().as_mut() {
                keys.push(named);
            }
        });
    }
    if let Some(key) = key.filter(|_| held) {
        KEPT.with(|kept| {
            if let Some(top) = kept.borrow_mut().last_mut() {
                if std::ptr::eq(top.ir, ir) {
                    top.prepared.insert(
                        key,
                        Entry {
                            prepared: Rc::clone(&prepared),
                            cuts,
                        },
                    );
                }
            }
        });
    }
    prepared
}
