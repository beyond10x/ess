//! The key a memory of this module keeps a computation under, as a 128-bit digest of its
//! rendering rather than the rendering itself.
//!
//! The memories ([`super::made`], [`super::created_memo`], [`super::run_memo`],
//! [`super::first_kept`]) are keyed on every argument and every thread-local a computation reads,
//! written with `Debug`. Those include whole arrangements, and an arrangement beside a row-set
//! refusal carries dozens of steps (0.99.8: 67 on average, up to 281), so each key was a string of
//! many kilobytes, built, compared and kept for every question asked. A model of a few hundred
//! scenarios spent a fifth of its synthesis writing those strings, and nearly half of the 17 GB it
//! held went with them.
//!
//! The rendering is streamed into two hashers and never kept. What most keys repeat is written
//! once and named by its own digest after that: the actor each command is sent as ([`Actors`]),
//! one entry per granted command, a row held for the runs advancing it
//! ([`super::subject_fact::holding`]), and the steps of an arrangement (`super::Steps`).
//!
//! Two keys are the same exactly when their renderings are, up to a collision of both 64-bit
//! hashes at once.
use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;
use std::fmt;
use std::hash::{DefaultHasher, Hasher};

use ess_compiler::refs::ActorRef;
use ess_domain::name::QualifiedName;

/// The digest of one key's rendering ([`of`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(super) struct Key(u64, u64);

impl fmt::Display for Key {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:016x}{:016x}", self.0, self.1)
    }
}

/// How many actor maps [`Actors`] keeps the digest of on one thread. A synthesis sends its
/// commands as one map per caller reading, so a few are asked again and again.
const ACTOR_MAPS: usize = 8;

thread_local! {
    /// How many bytes of key renderings were written on this thread.
    static RENDERED: Cell<u64> = const { Cell::new(0) };
    /// How many renderings of a key are open on this thread ([`of`]), for [`keying`].
    static KEYING: Cell<usize> = const { Cell::new(0) };
    /// The actor maps written into a key on this thread, most recent last, each with the digest
    /// of its `Debug` rendering ([`Actors`]).
    static ACTOR_DIGESTS: RefCell<Vec<(BTreeMap<QualifiedName, ActorRef>, Key)>> =
        const { RefCell::new(Vec::new()) };
}

/// The bytes a rendering gathers before they are hashed: the pieces `Debug` writes are a few
/// bytes each, and a hasher asked per piece spent most of its time starting and ending.
const BLOCK: usize = 512;

/// A rendering in progress: every byte goes to both hashers, none is kept.
struct Digesting {
    first: DefaultHasher,
    second: DefaultHasher,
    block: [u8; BLOCK],
    held: usize,
    bytes: u64,
}

impl Digesting {
    fn flush(&mut self) {
        let held = &self.block[..self.held];
        self.first.write(held);
        self.second.write(held);
        self.held = 0;
    }
}

impl fmt::Write for Digesting {
    fn write_str(&mut self, text: &str) -> fmt::Result {
        let bytes = text.as_bytes();
        self.bytes += bytes.len() as u64;
        if self.held + bytes.len() > BLOCK {
            self.flush();
        }
        if bytes.len() >= BLOCK {
            self.first.write(bytes);
            self.second.write(bytes);
        } else {
            self.block[self.held..self.held + bytes.len()].copy_from_slice(bytes);
            self.held += bytes.len();
        }
        Ok(())
    }
}

/// The digest of what `rendering` writes.
pub(super) fn of(rendering: fmt::Arguments<'_>) -> Key {
    let mut second = DefaultHasher::new();
    // Told apart from the first hasher by one leading byte.
    second.write_u8(0x5a);
    let mut digesting = Digesting {
        first: DefaultHasher::new(),
        second,
        block: [0; BLOCK],
        held: 0,
        bytes: 0,
    };
    KEYING.with(|open| open.set(open.get() + 1));
    // Writing into a hasher cannot fail; a `Debug` that does is a defect of that type.
    let written = fmt::write(&mut digesting, rendering);
    KEYING.with(|open| open.set(open.get() - 1));
    written.expect("a key renders");
    digesting.flush();
    RENDERED.with(|it| it.set(it.get() + digesting.bytes));
    Key(digesting.first.finish(), digesting.second.finish())
}

/// Whether a key is being rendered on this thread ([`of`]): a value that writes a digest of itself
/// there rather than its whole rendering asks this.
pub(super) fn keying() -> bool {
    KEYING.with(|open| open.get() > 0)
}

/// The actor each command is sent as, written into a key as the digest of its `Debug` rendering.
///
/// The map holds one entry per granted command, and nearly every key of a synthesis names it, so
/// it was most of what the keys wrote. The digest of a map equal to one of the last
/// [`ACTOR_MAPS`] written on this thread is read back: equal maps render alike.
pub(super) struct Actors<'a>(pub(super) &'a BTreeMap<QualifiedName, ActorRef>);

impl fmt::Display for Actors<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let known = ACTOR_DIGESTS.with(|kept| {
            let mut kept = kept.borrow_mut();
            let at = kept.iter().rposition(|(map, _)| map == self.0)?;
            // The most recent last, so the maps asked most stay.
            let entry = kept.remove(at);
            let key = entry.1;
            kept.push(entry);
            Some(key)
        });
        let key = known.unwrap_or_else(|| {
            let map = self.0;
            let key = of(format_args!("{map:?}"));
            ACTOR_DIGESTS.with(|kept| {
                let mut kept = kept.borrow_mut();
                if kept.len() == ACTOR_MAPS {
                    kept.remove(0);
                }
                kept.push((map.clone(), key));
            });
            key
        });
        write!(f, "actors={key}")
    }
}

/// What `run` returns, and how many bytes of key renderings were written on this thread while it
/// ran, counting those of caller readings run on other threads. The work a test bounds; not part
/// of the suite.
#[doc(hidden)]
pub fn key_bytes_rendered<R>(run: impl FnOnce() -> R) -> (R, u64) {
    let before = RENDERED.with(Cell::get);
    let out = run();
    (out, RENDERED.with(Cell::get) - before)
}

/// The count of [`RENDERED`] on this thread.
pub(super) fn taken_rendered() -> u64 {
    RENDERED.with(Cell::get)
}

/// Adds `more` to the count of [`RENDERED`] on this thread.
pub(super) fn add_rendered(more: u64) {
    RENDERED.with(|it| it.set(it.get() + more));
}
