//! What synthesis derives from the model alone, derived once per model rather than once per
//! command or branch that asks.
//!
//! Each answer here is a function of the model and of the key it is kept under, and of nothing
//! else: no clock, no counter, no state carried between calls. [`keep`] holds the answers for as
//! long as the model they were asked of is borrowed, so a second asking reads the first answer and
//! the suite is unchanged by construction. Outside a [`keep`], every answer is derived afresh.

use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;
use std::marker::PhantomData;
use std::rc::Rc;

use ess_compiler::ir::EssIr;
use ess_compiler::refs::EventRef;

thread_local! {
    /// The answers held, innermost model last: a synthesis of one model may synthesize another
    /// inside it (`synthesize::caller`), and an answer about one model is no answer about another.
    static KEPT: RefCell<Vec<Kept>> = const { RefCell::new(Vec::new()) };
    /// How many lists of the events a branch must not publish were built on this thread.
    static ABSENT_BUILT: Cell<u64> = const { Cell::new(0) };
}

/// The answers one model gave.
struct Kept {
    /// The model, compared by address only. It is borrowed for as long as this entry is on the
    /// stack ([`Keeping`]), so the address cannot be reused by another model meanwhile.
    ir: *const EssIr,
    /// The declared events a branch emitting the key's events must not publish.
    absent: BTreeMap<Vec<EventRef>, Rc<Vec<EventRef>>>,
}

/// Holds one model's answers until dropped. See [`keep`].
pub(super) struct Keeping<'ir> {
    pushed: bool,
    model: PhantomData<&'ir EssIr>,
}

impl Drop for Keeping<'_> {
    fn drop(&mut self) {
        if self.pushed {
            KEPT.with(|kept| {
                kept.borrow_mut().pop();
            });
        }
    }
}

/// Keeps each answer about `ir` from its first asking while the result is held. Nested for the
/// same model, the outer holder keeps answering.
pub(super) fn keep(ir: &EssIr) -> Keeping<'_> {
    let at = std::ptr::from_ref(ir);
    let pushed = KEPT.with(|kept| {
        let mut kept = kept.borrow_mut();
        if kept.last().is_some_and(|top| std::ptr::eq(top.ir, at)) {
            return false;
        }
        kept.push(Kept {
            ir: at,
            absent: BTreeMap::new(),
        });
        true
    });
    Keeping {
        pushed,
        model: PhantomData,
    }
}

/// What `run` returns, and how many lists of the events a branch must not publish were built on
/// this thread while it ran rather than read from the ones kept.
pub(super) fn absent_built<R>(run: impl FnOnce() -> R) -> (R, u64) {
    let before = ABSENT_BUILT.with(Cell::get);
    let out = run();
    (out, ABSENT_BUILT.with(Cell::get) - before)
}

/// The count of [`ABSENT_BUILT`] on this thread.
pub(super) fn taken_absent_built() -> u64 {
    ABSENT_BUILT.with(std::cell::Cell::get)
}

/// Adds `more` to the count of [`ABSENT_BUILT`] on this thread.
pub(super) fn add_absent_built(more: u64) {
    ABSENT_BUILT.with(|it| it.set(it.get() + more));
}

/// `f` over the answers kept for `ir`, where `ir` is the model kept innermost.
fn with_kept<R>(ir: &EssIr, f: impl FnOnce(&mut Kept) -> R) -> Option<R> {
    KEPT.with(|kept| {
        let mut kept = kept.borrow_mut();
        kept.last_mut()
            .filter(|top| std::ptr::eq(top.ir, ir))
            .map(f)
    })
}

/// The declared events a branch emitting `emitted` must not publish: `build`'s answer, kept per
/// `emitted` list.
pub(super) fn absent_events(
    ir: &EssIr,
    emitted: &[EventRef],
    build: impl FnOnce() -> Vec<EventRef>,
) -> Rc<Vec<EventRef>> {
    if let Some(Some(known)) = with_kept(ir, |kept| kept.absent.get(emitted).cloned()) {
        return known;
    }
    ABSENT_BUILT.with(|count| count.set(count.get() + 1));
    let absent = Rc::new(build());
    with_kept(ir, |kept| {
        kept.absent.insert(emitted.to_vec(), Rc::clone(&absent));
    });
    absent
}
