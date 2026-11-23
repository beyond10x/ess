//! The runs a value object's wrapped positions read ([`super::value_invariant`]), made once per
//! branch and context while [`super::invariants`] files them, and answered from memory after that.
//!
//! Every wrapped position of a value type is observed on a row some branch acting on the view's
//! source leaves, and each position tried those branches from the first again: the same run of the
//! same branch, arranged afresh, once per position, per view field and per value type. A run
//! depends on nothing but the model, the branch, the actors and the thread's arrangement context
//! (the keys [`super::made`] uses), so it is kept, keyed on all of those, for as long as the value
//! invariants are being filed.
//!
//! Answering from memory does not change what a search decides: the beside searches the run cut
//! ([`super::related_guard::cuts`]) count as cut again.
use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;

use super::{ActorRef, EssIr, QualifiedName, RefusalCause, ResolvedCommand, ResolvedOutcome, Run};

/// The runs kept for one model.
struct Kept {
    ir: *const EssIr,
    runs: BTreeMap<String, Entry>,
}

/// One run's result, and how many searches for a row beside it cut.
#[derive(Clone)]
struct Entry {
    run: Result<Run, RefusalCause>,
    cuts: usize,
}

thread_local! {
    /// The runs kept per model, innermost model last ([`remember`]).
    static KEPT: RefCell<Vec<Kept>> = const { RefCell::new(Vec::new()) };
    /// How many runs were asked on this thread, and how many of them were made.
    static ASKED: Cell<u64> = const { Cell::new(0) };
    static MADE: Cell<u64> = const { Cell::new(0) };
    /// Set while every run is made afresh ([`unkept`]).
    static UNKEPT: Cell<bool> = const { Cell::new(false) };
}

/// Keeps the runs made for one model until dropped. See [`remember`].
pub(super) struct Remembered;

impl Drop for Remembered {
    fn drop(&mut self) {
        KEPT.with(|kept| {
            kept.borrow_mut().pop();
        });
    }
}

/// Remembers the runs [`run`] makes for `ir` until the returned guard drops.
pub(super) fn remember(ir: &EssIr) -> Remembered {
    KEPT.with(|kept| {
        kept.borrow_mut().push(Kept {
            ir: std::ptr::from_ref(ir),
            runs: BTreeMap::new(),
        });
    });
    Remembered
}

/// What `run` returns, how many branch runs it asked and how many of them were made rather than
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

/// What `run` returns with every branch run made afresh, as before the memory existed.
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

/// [`super::run`] of `outcome`, from memory where the model remembered innermost ran it in the same
/// context before. Anything else is run every time.
pub(super) fn run(
    ir: &EssIr,
    command: &ResolvedCommand,
    outcome: &ResolvedOutcome,
    actors: &BTreeMap<QualifiedName, ActorRef>,
) -> Result<Run, RefusalCause> {
    ASKED.with(|count| count.set(count.get() + 1));
    let held = !UNKEPT.with(Cell::get)
        && KEPT.with(|kept| {
            kept.borrow()
                .last()
                .is_some_and(|top| std::ptr::eq(top.ir, ir))
        });
    let declared = ir
        .commands()
        .get(&command.name)
        .is_some_and(|declared| std::ptr::eq(declared, command));
    let position = command
        .outcomes
        .iter()
        .position(|branch| std::ptr::eq(branch, outcome))
        .filter(|_| held && declared);
    let Some(position) = position else {
        MADE.with(|count| count.set(count.get() + 1));
        return super::run(ir, command, outcome, actors);
    };
    let key = format!(
        "{}|{position}|{actors:?}|{}|{:?}|{}|{}|{}",
        command.name,
        super::subject_fact::held_key(ir),
        super::earlier_runs::captured(),
        crate::witness::under_test_key(),
        super::related_guard::open_key(),
        super::dropped_key(),
    );
    let known = KEPT.with(|kept| {
        kept.borrow()
            .last()
            .and_then(|top| top.runs.get(&key).cloned())
    });
    if let Some(known) = known {
        super::related_guard::add_cuts(known.cuts);
        return known.run;
    }
    let before = super::related_guard::cuts();
    MADE.with(|count| count.set(count.get() + 1));
    let made = super::run(ir, command, outcome, actors);
    let cuts = super::related_guard::cuts().saturating_sub(before);
    KEPT.with(|kept| {
        if let Some(top) = kept.borrow_mut().last_mut() {
            if std::ptr::eq(top.ir, ir) {
                top.runs.insert(
                    key,
                    Entry {
                        run: made.clone(),
                        cuts,
                    },
                );
            }
        }
    });
    made
}
