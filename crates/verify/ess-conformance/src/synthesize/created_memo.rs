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
//!
//! The setup run of a creating branch arranged as its own scenario
//! ([`super::related_guard`]'s `as_own_scenario`) is kept the same way ([`own_arranged`]): every
//! scenario of every command that needs such a row asks for it again with the same command,
//! outcome, chosen values and bound rows, and before it was kept a model of a few hundred
//! scenarios took about 30 times as long to synthesize as the release before the arrangement took
//! chosen values.
use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, BTreeSet};
use std::rc::Rc;

use super::{
    ActorRef, Arrangement, Distinction, Driver, EntityHandle, EssIr, Invocation, Node,
    QualifiedName, RefusalCause, ResolvedCommand, Unreachable,
};

/// The rows created for one model, and its setup runs arranged as their own scenario.
struct Kept {
    ir: *const EssIr,
    rows: BTreeMap<String, Entry>,
    own: BTreeMap<String, OwnEntry>,
}

/// One creation's result, and how many searches for a row beside it cut.
#[derive(Clone)]
struct Entry {
    made: Result<Arrangement, Unreachable>,
    cuts: usize,
}

/// One setup run arranged as its own scenario, and how many searches for a row beside it cut.
#[derive(Clone)]
struct OwnEntry {
    made: Result<Invocation, RefusalCause>,
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
            own: BTreeMap::new(),
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

/// The key a setup run arranged as its own scenario is kept under: every argument and every
/// thread-local the arrangement reads, the open arrangements and the whole caller-reading context
/// ([`super::threads::Context`]) among them. `None` for a command the model does not declare as
/// such (one projected for a related guard), which has no stable name to be kept by.
///
/// Of the rows held for the runs advancing them ([`super::subject_fact::holding`]), only those of
/// an entity the arrangement can reach ([`reach`]) are in the key: the arrangement reads a held row
/// only for an entity it arranges or the owner of one.
///
/// Two thread-locals a scenario sets around its runs are left out, as no arrangement reads them:
/// the names the scenario's earlier runs captured ([`super::earlier_runs`]), read only where an
/// outcome scenario arranges its view companions, and the branch under test where it copies no
/// `Optional` input, for which [`crate::witness::copied_under_test`] answers nothing for every
/// command ([`crate::witness::under_test_read_key`]). Every scenario of every command that needs
/// the row asks under a scope of its own, so with either in the key no arrangement is found
/// again; the committed suite bytes are the proof that the arrangement reads neither.
fn own_key(
    ir: &EssIr,
    driver: &Driver<'_>,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    distinction: Distinction,
    (bound, chosen): (
        &BTreeMap<String, crate::scenario::InstanceName>,
        Option<&BTreeMap<String, Node>>,
    ),
) -> Option<String> {
    let declared = ir
        .commands()
        .get(&driver.command.name)
        .is_some_and(|command| std::ptr::eq(command, driver.command));
    let outcome = driver
        .command
        .outcomes
        .iter()
        .position(|outcome| std::ptr::eq(outcome, driver.outcome))
        .filter(|_| declared)?;
    Some(format!(
        "{}|{outcome}|{:?}|{actors:?}|{distinction:?}|{bound:?}|{chosen:?}|{}|{}|{}|{}|{:?}",
        driver.command.name,
        driver.effect,
        super::subject_fact::held_key_within(ir, &reach(ir, driver.command)),
        crate::witness::under_test_read_key(),
        super::related_guard::open_key(),
        super::dropped_key(),
        super::threads::Context::current(),
    ))
}

/// Every qualified name written in `text`, a `Debug` rendering of the model, as
/// `QualifiedName(<name>)`.
fn names_in(text: &str) -> BTreeSet<&str> {
    text.split("QualifiedName(")
        .skip(1)
        .filter_map(|rest| rest.split(')').next())
        .collect()
}

/// The entities whose rows an arrangement of `command` can arrange or read, a superset: every
/// entity the command names, by its name or its identity type, and from each such entity its
/// owner, every entity its declaration names, and every entity named by a command that creates,
/// moves or deletes its rows, until nothing more is added. Kept per model.
fn reach(ir: &EssIr, command: &ResolvedCommand) -> Rc<BTreeSet<QualifiedName>> {
    let make = || {
        let identities: Vec<(&QualifiedName, String)> = ir
            .entities()
            .iter()
            .map(|(name, entity)| (name, format!("{:?}", entity.identity.type_ref)))
            .collect();
        let named = |text: &str| -> Vec<QualifiedName> {
            let names = names_in(text);
            identities
                .iter()
                .filter(|(name, identity)| {
                    names.contains(name.to_string().as_str())
                        || names_in(identity).iter().any(|held| names.contains(held))
                })
                .map(|(name, _)| (*name).clone())
                .collect()
        };
        let mut reached: BTreeSet<QualifiedName> = BTreeSet::new();
        let mut open = named(&format!("{command:?}"));
        while let Some(entity) = open.pop() {
            if !reached.insert(entity.clone()) {
                continue;
            }
            // Its owner, and itself for a reference it carries.
            open.extend(
                ir.relations_carried_by(&entity)
                    .values()
                    .map(|carried| carried.source.clone()),
            );
            if let Some(declared) = ir.entities().get(&entity) {
                open.extend(named(&format!("{declared:?}")));
            }
            for writer in ir.commands().values() {
                let writes = writer.outcomes.iter().any(|outcome| {
                    outcome
                        .subject
                        .as_ref()
                        .is_some_and(|subject| *subject.entity.name() == entity)
                        || outcome
                            .instances
                            .as_ref()
                            .is_some_and(|instances| *instances.entity.name() == entity)
                        || outcome
                            .affects
                            .iter()
                            .any(|affect| *affect.entity.name() == entity)
                });
                if writes {
                    open.extend(named(&format!("{writer:?}")));
                }
            }
        }
        reached
    };
    super::made::kept(ir, &format!("own-scenario reach|{}", command.name), make)
        .unwrap_or_else(|| Rc::new(make()))
}

/// `make`'s setup run of `driver` arranged as its own scenario, from memory where this model asked
/// the same arrangement in the same context before. A model not remembered innermost, and a thread
/// on which every creation is made afresh ([`unkept`]), is answered by `make` every time.
pub(super) fn own_arranged(
    ir: &EssIr,
    driver: &Driver<'_>,
    actors: &BTreeMap<QualifiedName, ActorRef>,
    distinction: Distinction,
    asked: (
        &BTreeMap<String, crate::scenario::InstanceName>,
        Option<&BTreeMap<String, Node>>,
    ),
    make: impl FnOnce() -> Result<Invocation, RefusalCause>,
) -> Result<Invocation, RefusalCause> {
    let held = !UNKEPT.with(Cell::get)
        && KEPT.with(|kept| {
            kept.borrow()
                .last()
                .is_some_and(|top| std::ptr::eq(top.ir, ir))
        });
    let Some(key) = held
        .then(|| own_key(ir, driver, actors, distinction, asked))
        .flatten()
    else {
        return make();
    };
    let known = KEPT.with(|kept| {
        kept.borrow()
            .last()
            .and_then(|top| top.own.get(&key).cloned())
    });
    if let Some(known) = known {
        super::related_guard::add_cuts(known.cuts);
        return known.made;
    }
    // No borrow is held while making: the arrangement asks the memory too.
    let before = super::related_guard::cuts();
    let arranged = make();
    let cuts = super::related_guard::cuts().saturating_sub(before);
    KEPT.with(|kept| {
        if let Some(top) = kept.borrow_mut().last_mut() {
            if std::ptr::eq(top.ir, ir) {
                top.own.insert(
                    key,
                    OwnEntry {
                        made: arranged.clone(),
                        cuts,
                    },
                );
            }
        }
    });
    arranged
}
