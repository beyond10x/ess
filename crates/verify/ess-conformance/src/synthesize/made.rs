//! The rows a creating branch leaves ([`super::subject_fact`]'s `creations`), made once per model
//! and context and answered from memory after that (beyond10x/ess#633).
//!
//! One arrangement asks for the same creations again and again: a row-set branch trying one
//! creator input after another arranges the same related row under each, and every such row
//! arranges its own related rows below it. Since a creating branch also leaves rows taken toward
//! the guards of the moves on the way, each of those levels creates several rows, and asking anew
//! multiplied the work by the depth of the chain: one adopter's specification ran for more than
//! thirteen minutes where 0.68.0 took under one. The rows depend on nothing but the arguments, the
//! model and the thread's arrangement context, so they are kept per model, keyed on all of those,
//! for as long as the model is being synthesized.
//!
//! Answering from memory does not change what a search decides: the beside searches a computation
//! cut ([`super::related_guard::cuts`]) count as cut again. The work counters a test reads
//! (`directed_steps`, `row_set_arrangements_tried`) count only the work done.
use std::cell::RefCell;
use std::collections::BTreeMap;

use super::{Arrangement, EssIr, RefusalCause};

thread_local! {
    /// The rows made per model, innermost model last ([`remember`]).
    static MADE: RefCell<Vec<Made>> = const { RefCell::new(Vec::new()) };
}

/// The rows made for one model.
struct Made {
    /// The model, compared by address only. It is borrowed for as long as this entry is on the
    /// stack ([`Remembered`]), so the address cannot be reused by another model meanwhile.
    ir: *const EssIr,
    rows: BTreeMap<String, Entry>,
}

/// What one computation answered, and how many beside searches it cut.
#[derive(Clone)]
struct Entry {
    made: Result<Vec<Arrangement>, RefusalCause>,
    cuts: usize,
}

thread_local! {
    /// How many times rows were made on this thread rather than answered from memory.
    static MAKES: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
}

/// What `run` returns, and how many times rows were made on this thread while it ran, rather than
/// answered from memory.
pub(super) fn counting<R>(run: impl FnOnce() -> R) -> (R, u64) {
    let before = MAKES.with(std::cell::Cell::get);
    let out = run();
    (out, MAKES.with(std::cell::Cell::get) - before)
}

/// `make`'s rows, counted as made ([`counting`]).
fn making(
    make: impl FnOnce() -> Result<Vec<Arrangement>, RefusalCause>,
) -> Result<Vec<Arrangement>, RefusalCause> {
    MAKES.with(|count| count.set(count.get() + 1));
    make()
}

/// Keeps the rows made for one model until dropped. See [`remember`].
pub(super) struct Remembered {
    pushed: bool,
}

impl Drop for Remembered {
    fn drop(&mut self) {
        if self.pushed {
            MADE.with(|made| {
                made.borrow_mut().pop();
            });
        }
    }
}

/// Remembers the rows made for `ir` until the returned guard drops. A model already remembered
/// innermost keeps its memory.
pub(super) fn remember(ir: &EssIr) -> Remembered {
    let at = std::ptr::from_ref(ir);
    let pushed = MADE.with(|made| {
        let mut made = made.borrow_mut();
        if made.last().is_some_and(|top| std::ptr::eq(top.ir, at)) {
            return false;
        }
        made.push(Made {
            ir: at,
            rows: BTreeMap::new(),
        });
        true
    });
    Remembered { pushed }
}

/// `make`'s rows, from memory where this model asked under the same `key` before. A model not
/// remembered innermost is answered by `make` every time.
pub(super) fn made(
    ir: &EssIr,
    key: impl FnOnce() -> String,
    make: impl FnOnce() -> Result<Vec<Arrangement>, RefusalCause>,
) -> Result<Vec<Arrangement>, RefusalCause> {
    let held = MADE.with(|made| {
        made.borrow()
            .last()
            .is_some_and(|top| std::ptr::eq(top.ir, ir))
    });
    if !held {
        return making(make);
    }
    let key = key();
    let known = MADE.with(|made| {
        made.borrow()
            .last()
            .and_then(|top| top.rows.get(&key).cloned())
    });
    if let Some(known) = known {
        super::related_guard::add_cuts(known.cuts);
        return known.made;
    }
    // No borrow is held while making: the rows below ask the memory too.
    let before = super::related_guard::cuts();
    let rows = making(make);
    let cuts = super::related_guard::cuts().saturating_sub(before);
    MADE.with(|memory| {
        if let Some(top) = memory.borrow_mut().last_mut() {
            if std::ptr::eq(top.ir, ir) {
                top.rows.insert(
                    key,
                    Entry {
                        made: rows.clone(),
                        cuts,
                    },
                );
            }
        }
    });
    rows
}
