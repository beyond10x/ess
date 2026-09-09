//! The store, the invocation reservation, the durable claim and the journal grammar.
//!
//! Everything here is about *durability under interruption*, and every guarantee is conditional on
//! the declared filesystem honoring synchronization and atomic publication. No lease, no automatic
//! claim theft, no automatic history reset and no timeout exists in this module, because each of
//! them would turn "I cannot establish what happened" into "nothing happened".
//!
//! The three publication rules are the whole protocol:
//!
//! 1. A record is serialized whole into a newly created private stage, synchronized, read back and
//!    checked, published under its final name **without replacement**, and its parent directory is
//!    then synchronized. A stage is never a record.
//! 2. A directory is exclusively created, synchronized itself, and then its parent is synchronized
//!    to publish its name. Both barriers precede anything that relies on the directory.
//! 3. Nothing that could mutate the target happens before the complete reservation, claim and
//!    `Prepared` durability chain has succeeded.

use std::fs::{self, File, OpenOptions};
use std::io::Write as _;
use std::path::{Path, PathBuf};

use super::model::{
    invalid, read_canonical, write_canonical, Admitted, Digest, Index, InvocationContext,
    InvocationId, JournalEntry, JournalFact, JournalFormat, LockClaim, ObservationPhase, Prepared,
    ProcessDisposition, ProcessOutcome, Refusal, RefusalCode, StoreFormat, StoreHeader, Uuid,
};
use super::{Barrier, FileFacts, Host, Trust};

/// The largest admitted store, claim or journal file.
///
/// Every read here is bounded before allocation. The bound is generous for a canonical record and
/// small enough that a corrupt or hostile file cannot be read into memory unbounded.
pub const RECORD_LIMIT: u64 = 1024 * 1024;

/// How many nonce collisions a reservation retries before refusing.
pub const RESERVATION_ATTEMPTS: usize = 16;

/// The store header's file name under the state root.
pub const STORE_HEADER: &str = "store.json";
/// The target exclusion claim's file name under the state root.
pub const TARGET_LOCK: &str = "target.lock";
/// The preprovisioned directory every invocation reservation is created beneath.
pub const INVOCATIONS: &str = "invocations";

/// Restates a protected-path refusal under the code of the thing being read.
///
/// [`admit_path`] and [`read_protected`] are shared, and they refuse under [`RefusalCode::StoreInvalid`]
/// because the store is what they were written for. Every other reader of a protected file has its
/// own code, and a caller told "the store did not admit" about the Helm artifact, the registry, a
/// kubeconfig or a pinned runtime has been told about the wrong thing. This is the one-line fix at
/// every such call site, so the mapping is a rule rather than a habit.
pub fn under<T>(code: RefusalCode, result: Admitted<T>) -> Admitted<T> {
    result.map_err(|refusal| {
        if refusal.code == RefusalCode::StoreInvalid {
            Refusal::new(code, refusal.detail)
        } else {
            refusal
        }
    })
}

fn store_invalid(detail: impl Into<String>) -> Refusal {
    Refusal::new(RefusalCode::StoreInvalid, detail)
}

fn incomplete(detail: impl Into<String>) -> Refusal {
    Refusal::new(RefusalCode::EvidenceIncomplete, detail)
}

/// Admits every component of `path` under `trust`, following no symbolic link.
///
/// A lexical path comparison is not enough and neither is a check of the leaf: a writable parent
/// component is a path an untrusted account can replace the leaf through, which is exactly the
/// substitution the administrative-file contract exists to exclude.
pub fn admit_path(host: &dyn Host, path: &Path, trust: Trust) -> Admitted<FileFacts> {
    if !path.is_absolute() {
        return Err(store_invalid(format!("{} is not absolute", path.display())));
    }
    let mut walked = PathBuf::from("/");
    let mut leaf = None;
    for component in path.components().skip(1) {
        let std::path::Component::Normal(name) = component else {
            return Err(store_invalid(format!(
                "{} is not a canonical path",
                path.display()
            )));
        };
        walked.push(name);
        let facts = host.facts(&walked)?;
        if facts.symlink {
            return Err(store_invalid(format!(
                "{} is a symbolic link; protected paths resolve through real components only",
                walked.display()
            )));
        }
        let owner_ok = match trust {
            Trust::Administrative => facts.uid == 0,
            Trust::Executor(uid) => facts.uid == 0 || facts.uid == uid,
        };
        if !owner_ok {
            return Err(store_invalid(format!(
                "{} is owned by uid {} and is not admitted under this trust",
                walked.display(),
                facts.uid
            )));
        }
        if facts.shared_writable() {
            return Err(store_invalid(format!(
                "{} is writable by group or other",
                walked.display()
            )));
        }
        leaf = Some(facts);
    }
    leaf.ok_or_else(|| store_invalid("the root directory is not a protected path"))
}

/// Reads a bounded, admitted regular file's exact bytes.
pub fn read_protected(host: &dyn Host, path: &Path, trust: Trust) -> Admitted<String> {
    let facts = admit_path(host, path, trust)?;
    if !facts.regular {
        return Err(store_invalid(format!(
            "{} is not a regular file",
            path.display()
        )));
    }
    if facts.size > RECORD_LIMIT {
        return Err(store_invalid(format!(
            "{} is {} bytes, past the {RECORD_LIMIT}-byte bound",
            path.display(),
            facts.size
        )));
    }
    let bytes = fs::read(path)
        .map_err(|error| store_invalid(format!("{} is unreadable: {error}", path.display())))?;
    String::from_utf8(bytes).map_err(|_| store_invalid(format!("{} is not UTF-8", path.display())))
}

/// One admitted preprovisioned recovery store.
#[derive(Debug, Clone)]
pub struct Store {
    root: PathBuf,
    header: StoreHeader,
    header_digest: Digest,
}

impl Store {
    /// The canonical state root.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// The admitted header.
    pub fn header(&self) -> &StoreHeader {
        &self.header
    }

    /// The digest of the header's exact bytes.
    pub fn header_digest(&self) -> &Digest {
        &self.header_digest
    }

    /// The preprovisioned `invocations/` directory.
    pub fn invocations(&self) -> PathBuf {
        self.root.join(INVOCATIONS)
    }

    /// The target exclusion claim's path.
    pub fn lock_path(&self) -> PathBuf {
        self.root.join(TARGET_LOCK)
    }
}

/// Admits a preprovisioned store, refusing a missing one rather than initializing a replacement.
///
/// "The CLI neither silently creates a replacement store nor treats a missing store as first use"
/// is the sentence this function is: a store that is not there is a store whose history is not
/// there, and a fresh one would make an empty journal look like a clean start.
pub fn open_store(host: &dyn Host, root: &Path) -> Admitted<Store> {
    let trust = Trust::Executor(host.executor_uid());
    let facts = admit_path(host, root, trust)?;
    if !facts.directory {
        return Err(store_invalid(format!(
            "the state root {} is not a directory",
            root.display()
        )));
    }
    let header_path = root.join(STORE_HEADER);
    let text = read_protected(host, &header_path, trust)?;
    let header: StoreHeader = read_canonical(&text).map_err(|error| {
        store_invalid(format!(
            "{} is not a canonical ess-execution-store/1 header: {}",
            header_path.display(),
            error.detail
        ))
    })?;
    if header.format != StoreFormat::V1 {
        return Err(store_invalid("unsupported store format"));
    }
    header.target.validate()?;
    let invocations = root.join(INVOCATIONS);
    let invocation_facts = admit_path(host, &invocations, trust)?;
    if !invocation_facts.directory {
        return Err(store_invalid(format!(
            "{} must be a durably provisioned directory",
            invocations.display()
        )));
    }
    Ok(Store {
        root: root.to_path_buf(),
        header_digest: Digest::of_bytes(text.as_bytes()),
        header,
    })
}

// --- Durable publication ------------------------------------------------------------------------

fn sync_directory(host: &dyn Host, path: &Path, barrier: Barrier, label: &str) -> Admitted<()> {
    host.barrier(barrier, label)?;
    let handle = File::open(path).map_err(|error| {
        incomplete(format!(
            "{} could not be opened for synchronization: {error}",
            path.display()
        ))
    })?;
    handle.sync_all().map_err(|error| {
        incomplete(format!(
            "{} could not be synchronized: {error}",
            path.display()
        ))
    })
}

/// Creates one directory exclusively. `Ok(false)` is a name collision, and only a collision.
///
/// The distinction is load bearing and was wrong once. A failed *barrier* after the directory was
/// created is not a collision, and retrying it under a fresh nonce would report "no fresh identity
/// was available" for a fault that had nothing to do with identity — hiding the failed barrier
/// behind an exhausted retry budget.
fn create_exclusive(host: &dyn Host, path: &Path, label: &str) -> Admitted<bool> {
    host.barrier(Barrier::DirectoryCreate, label)?;
    match fs::create_dir(path) {
        Ok(()) => Ok(true),
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => Ok(false),
        Err(error) => Err(incomplete(format!(
            "{} could not be created: {error}",
            path.display()
        ))),
    }
}

/// Crosses the two publication barriers that make a created directory usable.
fn publish_directory(host: &dyn Host, path: &Path, label: &str) -> Admitted<()> {
    sync_directory(host, path, Barrier::DirectorySync, label)?;
    let parent = path
        .parent()
        .ok_or_else(|| incomplete("a created directory has a parent"))?;
    sync_directory(host, parent, Barrier::DirectoryParentSync, label)
}

/// Publishes exact bytes under `destination` without replacing anything.
///
/// Stage, synchronize, read back and check, hard-link into place, synchronize the destination's
/// directory. The hard link is what makes publication no-replacement: it fails rather than
/// overwriting, so a second writer cannot take a name that is already a fact.
pub fn publish_bytes(
    host: &dyn Host,
    destination: &Path,
    bytes: &str,
    label: &str,
) -> Admitted<()> {
    let parent = destination
        .parent()
        .ok_or_else(|| incomplete("a published record has a parent directory"))?;
    let stage = parent.join(format!(
        ".stage-{}",
        destination
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("record")
    ));
    let _ = fs::remove_file(&stage);

    host.barrier(Barrier::StageWrite, label)?;
    let mut handle = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&stage)
        .map_err(|error| incomplete(format!("a private stage could not be created: {error}")))?;
    handle
        .write_all(bytes.as_bytes())
        .map_err(|error| incomplete(format!("a private stage could not be written: {error}")))?;

    host.barrier(Barrier::StageSync, label)?;
    handle.sync_all().map_err(|error| {
        incomplete(format!(
            "a private stage could not be synchronized: {error}"
        ))
    })?;
    drop(handle);

    host.barrier(Barrier::StageReadback, label)?;
    let read_back = fs::read(&stage)
        .map_err(|error| incomplete(format!("a private stage could not be read back: {error}")))?;
    if read_back != bytes.as_bytes() {
        return Err(incomplete(
            "a private stage did not read back as the bytes written to it",
        ));
    }

    host.barrier(Barrier::Publish, label)?;
    fs::hard_link(&stage, destination).map_err(|error| {
        if error.kind() == std::io::ErrorKind::AlreadyExists {
            incomplete(format!(
                "{} is already published; publication never replaces",
                destination.display()
            ))
        } else {
            incomplete(format!(
                "{} could not be published: {error}",
                destination.display()
            ))
        }
    })?;
    let _ = fs::remove_file(&stage);
    sync_directory(host, parent, Barrier::PublishParentSync, label)
}

// --- Invocation reservation ---------------------------------------------------------------------

/// One exclusively created, durably published invocation directory.
#[derive(Debug, Clone)]
pub struct Reservation {
    id: InvocationId,
    directory: PathBuf,
}

impl Reservation {
    /// The invocation identity.
    pub fn id(&self) -> &InvocationId {
        &self.id
    }

    /// The reserved directory.
    pub fn directory(&self) -> &Path {
        &self.directory
    }
}

/// Reserves a fresh invocation directory beneath the admitted store.
///
/// The nonce comes from the host OS random source, never from a PID, a clock or a counter. A
/// collision retries at most [`RESERVATION_ATTEMPTS`] times and then refuses; a retained
/// reservation is never reused, and a retry is a new invocation identity.
pub fn reserve(host: &dyn Host, store: &Store) -> Admitted<Reservation> {
    let parent = store.invocations();
    for _ in 0..RESERVATION_ATTEMPTS {
        let nonce = host.random_nonce()?;
        let directory = parent.join(nonce.as_str());
        if !create_exclusive(host, &directory, nonce.as_str())? {
            continue;
        }
        publish_directory(host, &directory, nonce.as_str())?;
        return Ok(Reservation {
            id: InvocationId {
                store_epoch: store.header().store_epoch.clone(),
                nonce,
            },
            directory,
        });
    }
    Err(incomplete(
        "no fresh invocation identity was available after the admitted number of attempts",
    ))
}

// --- Target exclusion ---------------------------------------------------------------------------

/// A retained claim read from the store, with the digest of its exact bytes.
#[derive(Debug, Clone)]
pub struct RetainedClaim {
    /// The claim.
    pub claim: LockClaim,
    /// The digest of its exact retained bytes.
    pub digest: Digest,
}

/// Reads the retained target claim, if one is published.
pub fn read_claim(host: &dyn Host, store: &Store) -> Admitted<Option<RetainedClaim>> {
    let path = store.lock_path();
    if !path.exists() {
        return Ok(None);
    }
    let text = read_protected(host, &path, Trust::Executor(host.executor_uid()))?;
    let claim: LockClaim = read_canonical(&text).map_err(|error| {
        Refusal::new(
            RefusalCode::EvidenceIncomplete,
            format!(
                "the retained target claim is not canonical: {}",
                error.detail
            ),
        )
    })?;
    Ok(Some(RetainedClaim {
        digest: Digest::of_bytes(text.as_bytes()),
        claim,
    }))
}

/// Publishes this invocation's claim, refusing rather than replacing a retained one.
///
/// There is no timeout, no PID check and no age. A claim that is there belongs to somebody else,
/// and the only thing that removes it is its own owner's ordinary safe completion or the caller's
/// independently established quiescence procedure.
pub fn publish_claim(host: &dyn Host, store: &Store, claim: &LockClaim) -> Admitted<Digest> {
    let bytes = write_canonical(claim);
    publish_bytes(host, &store.lock_path(), &bytes, "target.lock").map_err(|error| {
        Refusal::new(
            RefusalCode::MutationBlocked,
            format!(
                "the target exclusion claim could not be published: {}",
                error.detail
            ),
        )
    })?;
    Ok(Digest::of_bytes(bytes.as_bytes()))
}

/// Removes this invocation's own claim on ordinary safe completion.
///
/// The claim is re-read and compared before removal, so that an invocation can only release the
/// exclusion it actually holds. The parent is synchronized before the release is reported.
pub fn release_claim(host: &dyn Host, store: &Store, id: &InvocationId) -> Admitted<()> {
    let Some(retained) = read_claim(host, store)? else {
        return Ok(());
    };
    if &retained.claim.invocation != id {
        return Err(Refusal::new(
            RefusalCode::MutationBlocked,
            "the retained claim belongs to another invocation and is never reclaimed here",
        ));
    }
    host.barrier(Barrier::ClaimRelease, "target.lock")?;
    fs::remove_file(store.lock_path())
        .map_err(|error| incomplete(format!("the own claim could not be removed: {error}")))?;
    sync_directory(
        host,
        store.root(),
        Barrier::PublishParentSync,
        "target.lock",
    )
}

// --- Journal --------------------------------------------------------------------------------------

/// The exact file name one sequence number publishes under.
pub fn entry_name(sequence: Index) -> String {
    format!("{:020}.json", sequence.get())
}

/// An open journal for one invocation, appended to entry by entry.
#[derive(Debug)]
pub struct Journal {
    directory: PathBuf,
    invocation: InvocationId,
    next: u64,
    previous: Option<Digest>,
    closed: bool,
}

impl Journal {
    /// Opens a journal on a fresh reservation.
    pub fn open(reservation: &Reservation) -> Self {
        Self {
            directory: reservation.directory().to_path_buf(),
            invocation: reservation.id().clone(),
            next: 0,
            previous: None,
            closed: false,
        }
    }

    /// The sequence the next published entry will take.
    pub fn next_sequence(&self) -> u64 {
        self.next
    }

    /// Whether a terminal fact has been published.
    pub fn is_closed(&self) -> bool {
        self.closed
    }

    /// Publishes one fact durably, and returns the sequence it took.
    ///
    /// A fact after a terminal fact is refused here rather than written and diagnosed later: the
    /// grammar says a closed history has one final fact and no following entry, and this is the
    /// only place that could produce one.
    pub fn append(&mut self, host: &dyn Host, fact: JournalFact) -> Admitted<Index> {
        if self.closed {
            return Err(incomplete("a closed journal admits no further entry"));
        }
        if self.next == 0 && !matches!(fact, JournalFact::Opened(_)) {
            return Err(incomplete(
                "a journal starts with exactly one Opened at sequence zero",
            ));
        }
        if self.next > 0 && matches!(fact, JournalFact::Opened(_)) {
            return Err(incomplete("a journal has exactly one Opened"));
        }
        let sequence = Index::new(self.next)?;
        let terminal = fact.is_terminal();
        let entry = JournalEntry {
            format: JournalFormat::V1,
            invocation: self.invocation.clone(),
            sequence,
            previous_digest: self.previous.clone(),
            fact,
        };
        let bytes = write_canonical(&entry);
        let name = entry_name(sequence);
        publish_bytes(host, &self.directory.join(&name), &bytes, &name)?;
        self.previous = Some(Digest::of_bytes(bytes.as_bytes()));
        self.next += 1;
        self.closed = terminal;
        Ok(sequence)
    }
}

/// How a retained invocation directory classifies.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JournalState {
    /// A reserved directory with no `Opened`. It authorizes no mutation and is not a completion.
    EmptyReservation,
    /// A valid prefix with no terminal fact.
    Incomplete,
    /// A closed history: exactly one final `Stopped` or `Completed`, and nothing after it.
    Closed,
}

/// One retained invocation's admitted history.
#[derive(Debug, Clone)]
pub struct History {
    /// The invocation's nonce, taken from the directory name.
    pub nonce: Uuid,
    /// How the directory classifies.
    pub state: JournalState,
    /// Every published entry, in sequence order.
    pub entries: Vec<JournalEntry>,
}

impl History {
    /// The invocation's context, if it published one.
    pub fn context(&self) -> Option<&InvocationContext> {
        self.entries.first().and_then(|entry| match &entry.fact {
            JournalFact::Opened(context) => Some(context.as_ref()),
            _ => None,
        })
    }

    /// Every `Prepared` fact that has no matching durable disposition.
    ///
    /// This is the indeterminate set. A prepared operation without a disposition may have started;
    /// nothing in the journal can settle it, and no restart may reconstruct `NotLaunched` from an
    /// empty tail.
    pub fn unresolved_preparations(&self) -> Vec<&Prepared> {
        let settled: Vec<Index> = self
            .entries
            .iter()
            .filter_map(|entry| match &entry.fact {
                JournalFact::ProcessOutcome(outcome) => Some(outcome.operation),
                _ => None,
            })
            .collect();
        self.entries
            .iter()
            .filter_map(|entry| match &entry.fact {
                JournalFact::Prepared(prepared) => Some(prepared),
                _ => None,
            })
            .filter(|prepared| !settled.contains(&prepared.operation))
            .collect()
    }

    /// Every operation whose acknowledged disposition has no fresh `After` observation.
    pub fn acknowledged_without_after(&self) -> Vec<Index> {
        let after: Vec<Index> = self
            .entries
            .iter()
            .filter_map(|entry| match &entry.fact {
                JournalFact::Observed(observation)
                    if observation.phase == ObservationPhase::After =>
                {
                    Some(observation.operation)
                }
                _ => None,
            })
            .collect();
        self.entries
            .iter()
            .filter_map(|entry| match &entry.fact {
                JournalFact::ProcessOutcome(outcome)
                    if outcome.disposition == ProcessDisposition::Acknowledged =>
                {
                    Some(outcome.operation)
                }
                _ => None,
            })
            .filter(|operation| !after.contains(operation))
            .collect()
    }
}

/// Reads and admits one retained invocation directory.
///
/// Corrupt evidence refuses and the bytes are preserved: a torn file at a published sequence name,
/// a gap, an invalid predecessor digest, wrong identity or scope, a duplicated terminal or an entry
/// after a terminal is R18 refusal, and quiescence does not repair it. An unpublished stage is not
/// an entry and is skipped rather than read.
pub fn read_history(host: &dyn Host, store: &Store, nonce: &Uuid) -> Admitted<History> {
    let directory = store.invocations().join(nonce.as_str());
    let trust = Trust::Executor(host.executor_uid());
    admit_path(host, &directory, trust)?;
    let mut names: Vec<String> = fs::read_dir(&directory)
        .map_err(|error| incomplete(format!("a reserved invocation is unreadable: {error}")))?
        .filter_map(std::result::Result::ok)
        .filter_map(|entry| entry.file_name().to_str().map(str::to_owned))
        .filter(|name| !name.starts_with('.'))
        .collect();
    names.sort();

    let mut entries: Vec<JournalEntry> = Vec::new();
    let mut previous: Option<Digest> = None;
    for (position, name) in names.iter().enumerate() {
        let sequence = Index::new(u64::try_from(position).map_err(|_| incomplete("sequence"))?)?;
        if name != &entry_name(sequence) {
            return Err(incomplete(format!(
                "{}/{name} is not the contiguous published sequence member {}",
                nonce,
                entry_name(sequence)
            )));
        }
        let text = read_protected(host, &directory.join(name), trust).map_err(|error| {
            Refusal::new(
                RefusalCode::EvidenceIncomplete,
                format!("{nonce}/{name} is not admissible: {}", error.detail),
            )
        })?;
        let entry: JournalEntry = read_canonical(&text).map_err(|error| {
            Refusal::new(
                RefusalCode::EvidenceIncomplete,
                format!(
                    "{nonce}/{name} is not a canonical journal entry: {}",
                    error.detail
                ),
            )
        })?;
        if entry.sequence != sequence {
            return Err(incomplete(format!(
                "{nonce}/{name} carries sequence {} and is published as {sequence}",
                entry.sequence
            )));
        }
        if entry.invocation.nonce != *nonce
            || entry.invocation.store_epoch != store.header().store_epoch
        {
            return Err(incomplete(format!(
                "{nonce}/{name} is scoped to another invocation or store epoch"
            )));
        }
        if entry.previous_digest != previous {
            return Err(incomplete(format!(
                "{nonce}/{name} does not carry its predecessor's digest"
            )));
        }
        if position == 0 && !matches!(entry.fact, JournalFact::Opened(_)) {
            return Err(incomplete(format!(
                "{nonce} publishes {} at sequence zero, not Opened",
                entry.fact.name()
            )));
        }
        if position > 0 && matches!(entry.fact, JournalFact::Opened(_)) {
            return Err(incomplete(format!("{nonce} publishes a second Opened")));
        }
        if entries
            .last()
            .is_some_and(|last: &JournalEntry| last.fact.is_terminal())
        {
            return Err(incomplete(format!(
                "{nonce}/{name} follows a terminal fact"
            )));
        }
        previous = Some(Digest::of_bytes(text.as_bytes()));
        entries.push(entry);
    }

    check_operation_grammar(nonce, &entries)?;
    let state = match entries.last() {
        None => JournalState::EmptyReservation,
        Some(entry) if entry.fact.is_terminal() => JournalState::Closed,
        Some(_) => JournalState::Incomplete,
    };
    Ok(History {
        nonce: nonce.clone(),
        state,
        entries,
    })
}

/// The per-operation grammar every prefix obeys.
///
/// At most one `Prepared` and one disposition per selected index; a `Prepared` names an earlier
/// `Observed` of the same operation in the `Before` phase within the same invocation; a
/// `Completed` accounts for the selected operations.
fn check_operation_grammar(nonce: &Uuid, entries: &[JournalEntry]) -> Admitted<()> {
    let mut prepared: Vec<Index> = Vec::new();
    let mut settled: Vec<Index> = Vec::new();
    for entry in entries {
        match &entry.fact {
            JournalFact::Prepared(fact) => {
                if prepared.contains(&fact.operation) {
                    return Err(incomplete(format!(
                        "{nonce} prepares operation {} twice",
                        fact.operation
                    )));
                }
                let authorizing = entries.iter().find(|candidate| {
                    candidate.sequence == fact.observation_sequence
                        && matches!(
                            &candidate.fact,
                            JournalFact::Observed(observation)
                                if observation.operation == fact.operation
                                    && observation.phase == ObservationPhase::Before
                        )
                });
                if authorizing.is_none() || fact.observation_sequence >= entry.sequence {
                    return Err(incomplete(format!(
                        "{nonce} prepares operation {} against sequence {}, which is not an \
                         earlier Before observation of that operation in this invocation",
                        fact.operation, fact.observation_sequence
                    )));
                }
                prepared.push(fact.operation);
            }
            JournalFact::ProcessOutcome(fact) => {
                if settled.contains(&fact.operation) {
                    return Err(incomplete(format!(
                        "{nonce} settles operation {} twice",
                        fact.operation
                    )));
                }
                if !prepared.contains(&fact.operation) {
                    return Err(incomplete(format!(
                        "{nonce} settles operation {} without a durable Prepared",
                        fact.operation
                    )));
                }
                settled.push(fact.operation);
            }
            JournalFact::Observed(observation) => observation.validate()?,
            JournalFact::Completed(count) => {
                let context = entries.first().and_then(|entry| match &entry.fact {
                    JournalFact::Opened(context) => Some(context.as_ref()),
                    _ => None,
                });
                let selected = context.map_or(0, |context| context.selected.len());
                if usize::try_from(count.get()).unwrap_or(usize::MAX) != selected {
                    return Err(incomplete(format!(
                        "{nonce} claims completion of {count} operations and selected {selected}"
                    )));
                }
            }
            JournalFact::Opened(_) | JournalFact::Stopped(_) => {}
        }
    }
    Ok(())
}

/// Every retained invocation in the store, in nonce order.
///
/// The scan considers all relevant retained reservations and histories. An explicitly named
/// predecessor is additional context, never a selector: omitting `retry_of` cannot hide a
/// retained store, authority revision, ownership history or unresolved preparation.
pub fn scan_store(host: &dyn Host, store: &Store) -> Admitted<Vec<History>> {
    let mut nonces: Vec<Uuid> = Vec::new();
    let listing = fs::read_dir(store.invocations())
        .map_err(|error| store_invalid(format!("the invocation store is unreadable: {error}")))?;
    for entry in listing {
        let entry = entry
            .map_err(|error| store_invalid(format!("an invocation is unreadable: {error}")))?;
        let name = entry
            .file_name()
            .to_str()
            .map(str::to_owned)
            .ok_or_else(|| store_invalid("an invocation name is not UTF-8"))?;
        if name.starts_with('.') {
            continue;
        }
        nonces.push(Uuid::new(name).map_err(|error| {
            store_invalid(format!(
                "the invocation store holds a name that is not an invocation: {}",
                error.detail
            ))
        })?);
    }
    nonces.sort();
    nonces
        .iter()
        .map(|nonce| read_history(host, store, nonce))
        .collect()
}

/// Builds the durable `Prepared` fact naming its authorizing observation.
pub fn prepared(operation: Index, observation_sequence: Index) -> JournalFact {
    JournalFact::Prepared(Prepared {
        operation,
        observation_sequence,
    })
}

/// Builds the durable process outcome for one operation.
pub fn outcome(operation: Index, disposition: ProcessDisposition) -> JournalFact {
    JournalFact::ProcessOutcome(ProcessOutcome {
        operation,
        disposition,
    })
}

/// Refuses a reader that asked for an index past the admitted range.
pub fn index(value: usize) -> Admitted<Index> {
    Index::new(u64::try_from(value).map_err(|_| invalid("an index is out of range"))?)
}
