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
//! The rows a stored-row search reaches, depth by depth, are kept the same way ([`reached`]): every
//! branch of a command reading stored fields asks the same search and differs only in what it
//! accepts, and advancing every row by every move again for each of them made a row several
//! commands move cost minutes to synthesize.
//!
//! The branch a command selects on a stored row for an input is kept too ([`row_picks`]): it reads
//! nothing but the model, the command, the row's state and fields and the input, and every
//! search that advances that row asks it again — once per branch of the moving command, and once
//! per search whose goal differs — which made a model of many such commands spend most of its
//! synthesis deciding the same guards over the same rows.
//!
//! The branch a command selects on a related row for an input is kept the same way, under a key
//! of its own (`related_guard::row_picks`): every branch of a command guarding on that row searches
//! the same rows with the same inputs and differs only in the branch it accepts, so a command of
//! many `when_related:` refusals decided the same guards once per refusal.
//!
//! Answering from memory does not change what a search decides: the beside searches a computation
//! cut ([`super::related_guard::cuts`]) count as cut again. The work counters a test reads
//! (`directed_steps`, `row_set_arrangements_tried`, `branch_selections_tried`,
//! `related_selections_tried`) count only the work done.
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use super::memo_key::Key;
use super::subject_fact::{Reached, RowPicks, Selections};
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
    rows: BTreeMap<Key, Entry>,
    /// What each stored-row search reached, by its key ([`reached`]).
    searches: BTreeMap<Key, Reached>,
    /// The branches a command's inputs select on a row, by their key ([`selections`]).
    selections: BTreeMap<Key, Rc<Selections>>,
    /// The branch a command selects on a row for each input asked so far, by command and row
    /// ([`row_picks`]).
    picks: BTreeMap<Key, Rc<RowPicks>>,
    /// Values derived from the model alone and owning what they hold, by their key ([`kept`]).
    kept: BTreeMap<String, Rc<dyn std::any::Any>>,
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

/// The count of [`MAKES`] on this thread.
pub(super) fn taken_makes() -> u64 {
    MAKES.with(std::cell::Cell::get)
}

/// Adds `more` to the count of [`MAKES`] on this thread.
pub(super) fn add_makes(more: u64) {
    MAKES.with(|it| it.set(it.get() + more));
}

thread_local! {
    /// How many times a row and an input were checked on this thread for the branch they select.
    static PICKS: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
}

/// What `run` returns, and how many times a row and an input were checked on this thread for the
/// branch they select while it ran, rather than answered from memory.
pub(super) fn picks<R>(run: impl FnOnce() -> R) -> (R, u64) {
    let before = PICKS.with(std::cell::Cell::get);
    let out = run();
    (out, PICKS.with(std::cell::Cell::get) - before)
}

/// The count of [`PICKS`] on this thread.
pub(super) fn taken_picks() -> u64 {
    PICKS.with(std::cell::Cell::get)
}

/// Adds `more` to the count of [`PICKS`] on this thread.
pub(super) fn add_picks(more: u64) {
    PICKS.with(|it| it.set(it.get() + more));
}

/// Counts one row and input checked for the branch they select ([`picks`]).
pub(super) fn picking() {
    PICKS.with(|count| count.set(count.get() + 1));
}

thread_local! {
    /// How many times a related row and an input were checked on this thread for the branch they
    /// select.
    static RELATED_PICKS: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
}

/// What `run` returns, and how many times a related row and an input were checked on this thread
/// for the branch they select while it ran, rather than answered from memory.
pub(super) fn related_picks<R>(run: impl FnOnce() -> R) -> (R, u64) {
    let before = RELATED_PICKS.with(std::cell::Cell::get);
    let out = run();
    (out, RELATED_PICKS.with(std::cell::Cell::get) - before)
}

/// The count of [`RELATED_PICKS`] on this thread.
pub(super) fn taken_related_picks() -> u64 {
    RELATED_PICKS.with(std::cell::Cell::get)
}

/// Adds `more` to the count of [`RELATED_PICKS`] on this thread.
pub(super) fn add_related_picks(more: u64) {
    RELATED_PICKS.with(|it| it.set(it.get() + more));
}

/// Counts one related row and input checked for the branch they select ([`related_picks`]).
pub(super) fn related_picking() {
    RELATED_PICKS.with(|count| count.set(count.get() + 1));
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
            let top = MADE.with(|made| made.borrow_mut().pop());
            if let Some(top) = top {
                park(top);
            }
        }
    }
}

/// Remembers the rows made for `ir` until the returned guard drops. A model already remembered
/// innermost keeps its memory, and a model [`Keeping`] holds gets back what it made before.
pub(super) fn remember(ir: &EssIr) -> Remembered {
    if let Some(recalled) = recall(ir) {
        return recalled;
    }
    let at = std::ptr::from_ref(ir);
    let pushed = MADE.with(|made| {
        let mut made = made.borrow_mut();
        if made.last().is_some_and(|top| std::ptr::eq(top.ir, at)) {
            return false;
        }
        made.push(Made {
            ir: at,
            rows: BTreeMap::new(),
            searches: BTreeMap::new(),
            selections: BTreeMap::new(),
            picks: BTreeMap::new(),
            kept: BTreeMap::new(),
        });
        true
    });
    Remembered { pushed }
}

/// The value under `key`, derived by `make` the first time this model asks and kept for as long as
/// the model is remembered. `None` where `ir` is not the model remembered innermost: the caller
/// derives it afresh then, as it did before.
pub(super) fn kept<T: 'static>(ir: &EssIr, key: &str, make: impl FnOnce() -> T) -> Option<Rc<T>> {
    let held = MADE.with(|memory| {
        let memory = memory.borrow();
        let top = memory.last().filter(|top| std::ptr::eq(top.ir, ir))?;
        Some(top.kept.get(key).cloned())
    })?;
    if let Some(held) = held {
        return held.downcast::<T>().ok();
    }
    let value = Rc::new(make());
    MADE.with(|made| {
        if let Some(top) = made
            .borrow_mut()
            .last_mut()
            .filter(|top| std::ptr::eq(top.ir, ir))
        {
            top.kept.insert(key.to_owned(), value.clone());
        }
    });
    Some(value)
}

/// Whether `ir` is the model remembered innermost, whose searches are kept ([`reached`]).
pub(super) fn remembers(ir: &EssIr) -> bool {
    MADE.with(|made| {
        made.borrow()
            .last()
            .is_some_and(|top| std::ptr::eq(top.ir, ir))
    })
}

/// What the search under `key` reached when this model asked it before ([`Reached`]), if it did.
pub(super) fn reached(ir: &EssIr, key: Key) -> Option<Reached> {
    MADE.with(|made| {
        made.borrow()
            .last()
            .filter(|top| std::ptr::eq(top.ir, ir))
            .and_then(|top| top.searches.get(&key).cloned())
    })
}

/// The branches a command's inputs select on a row under `key` ([`Selections`]): the ones this
/// model kept, else `make`'s, kept where `ir` is the model remembered innermost. They are filled
/// in as later branches ask, through the shared handle.
pub(super) fn selections(
    ir: &EssIr,
    key: Key,
    make: impl FnOnce() -> Selections,
) -> Rc<Selections> {
    let known = MADE.with(|made| {
        made.borrow()
            .last()
            .filter(|top| std::ptr::eq(top.ir, ir))
            .and_then(|top| top.selections.get(&key).cloned())
    });
    if let Some(known) = known {
        return known;
    }
    // No borrow is held while making: the inputs below may ask the memory too.
    let fresh = Rc::new(make());
    MADE.with(|memory| {
        if let Some(top) = memory.borrow_mut().last_mut() {
            if std::ptr::eq(top.ir, ir) {
                top.selections.insert(key, Rc::clone(&fresh));
            }
        }
    });
    fresh
}

/// The branches a command selects on one row under `key` ([`RowPicks`]), as far as the model
/// remembered innermost has asked; `None` where `ir` is not that model. They are filled in as
/// later searches ask, through the shared handle.
pub(super) fn row_picks(ir: &EssIr, key: Key) -> Option<Rc<RowPicks>> {
    MADE.with(|made| {
        let mut made = made.borrow_mut();
        let top = made.last_mut().filter(|top| std::ptr::eq(top.ir, ir))?;
        Some(Rc::clone(top.picks.entry(key).or_default()))
    })
}

/// Keeps what the search under `key` has reached so far, for the searches after it.
pub(super) fn reach(ir: &EssIr, key: Key, reached: Reached) {
    MADE.with(|made| {
        if let Some(top) = made.borrow_mut().last_mut() {
            if std::ptr::eq(top.ir, ir) {
                top.searches.insert(key, reached);
            }
        }
    });
}

/// `make`'s rows, from memory where this model asked under the same `key` before. A model not
/// remembered innermost is answered by `make` every time.
pub(super) fn made(
    ir: &EssIr,
    key: impl FnOnce() -> Key,
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

thread_local! {
    /// The models whose memory outlasts each synthesis of them ([`Keeping`]), innermost scope
    /// last, by address: the memory while it is parked, `None` while the model is remembered.
    static KEPT: RefCell<Vec<BTreeMap<usize, Option<Made>>>> = const { RefCell::new(Vec::new()) };
}

/// Keeps what is made for each model it holds past every synthesis of that model, until dropped.
///
/// A model whose actors carry attributes is read once per caller assignment, and every reading is
/// a model of its own (`super::caller`); one reading is synthesized whole and again for each
/// command focused on, and each time asked every search again. A model held here is the same model
/// each time, so the next synthesis of it is answered from what the earlier ones made. The memory
/// is keyed as before, so nothing a search decides changes. The guard holds the models, so no
/// address it keeps can be taken by another model while their memory is parked.
pub(super) struct Keeping {
    models: Vec<std::sync::Arc<EssIr>>,
    /// A Keeping pops the stack of the thread that opened it, so it is not sent to another thread.
    _thread: std::marker::PhantomData<*const ()>,
}

impl Keeping {
    /// A new innermost scope, holding no model yet.
    pub(super) fn open() -> Self {
        KEPT.with(|kept| kept.borrow_mut().push(BTreeMap::new()));
        Self {
            models: Vec::new(),
            _thread: std::marker::PhantomData,
        }
    }

    /// Keeps what `ir` makes for as long as this guard lives.
    pub(super) fn keep(&mut self, ir: &std::sync::Arc<EssIr>) {
        KEPT.with(|kept| {
            if let Some(scope) = kept.borrow_mut().last_mut() {
                scope.entry(address(ir)).or_insert(None);
            }
        });
        self.models.push(std::sync::Arc::clone(ir));
    }
}

impl Drop for Keeping {
    fn drop(&mut self) {
        // The memories go before the models they are about.
        KEPT.with(|kept| {
            kept.borrow_mut().pop();
        });
    }
}

fn address(ir: &EssIr) -> usize {
    std::ptr::from_ref(ir) as usize
}

/// `ir`'s parked memory, remembered innermost again, where [`Keeping`] holds it.
fn recall(ir: &EssIr) -> Option<Remembered> {
    let parked = KEPT.with(|kept| {
        kept.borrow_mut()
            .last_mut()
            .and_then(|scope| scope.get_mut(&address(ir)))
            .and_then(Option::take)
    })?;
    MADE.with(|made| made.borrow_mut().push(parked));
    Some(Remembered { pushed: true })
}

/// Parks `made` where [`Keeping`] holds its model; drops it otherwise.
fn park(made: Made) {
    KEPT.with(|kept| {
        if let Some(slot) = kept
            .borrow_mut()
            .last_mut()
            .and_then(|scope| scope.get_mut(&(made.ir as usize)))
        {
            *slot = Some(made);
        }
    });
}
