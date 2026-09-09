//! Finite deployment recovery: the accepted `SingleHostGeneratedHelm1` contract.
//!
//! The binding is `docs/design/review-execution-recovery.md`. This module tree implements it and
//! nothing beyond it: a finite invocation admits desired and baseline intent, admits one caller
//! provisioned authority out of the complete active registry, reserves a durable invocation,
//! observes, decides, attempts at most one mutation per operation, records what it can establish
//! and stops at the first thing it cannot.
//!
//! There is no background watch, no eventual-convergence promise, no automatic retry and no
//! automatic quiescence. Refusal leaves the state *unknown* — not absent, not rolled back, not
//! reconciled.
//!
//! | Module | What it owns |
//! |---|---|
//! | [`model`] | the 44 declared `recovery.execution` types, canonical bytes and reader constraints |
//! | [`journal`] | the store, the invocation reservation, the durable claim and the journal grammar |
//! | [`authority`] | the complete active-registry scan and the admission of one authority |
//! | [`chart`] | OCI proof consumption, bounded archive decoding and the exact five-file projection |
//! | [`observe`] | authenticated observation, the freshness budget and the operation predicates |
//! | [`process`] | the admitted Helm artifact, its restricted invocation and process uncertainty |

pub mod model;

pub mod authority;
pub mod chart;
pub mod journal;
pub mod observe;
pub mod process;

// --- Host seams -------------------------------------------------------------------------------
//
// The production profile expects root-owned administrative files, a nonzero executor UID and a
// filesystem that honors synchronization. An ordinary unprivileged temporary directory cannot
// honestly instantiate that, so the admission *algorithm* is separated from the *evidence* it
// decides on: `Host` supplies the evidence and the faultable boundaries, and the modules around it
// decide. This is a Rust interface, not an escape hatch — the shipped `ess` binary constructs
// `RealHost` unconditionally, with no flag, environment variable or permissive profile selecting
// another one. Injected administrative metadata exercises the decision; it is not proof of root
// provisioning, and injected crash persistence is not proof of hardware power-loss behavior.

use std::fs;
use std::path::Path;
use std::time::Instant;

use model::{invalid, Admitted, Refusal, RefusalCode, Uuid};

/// Every durability and IO boundary a fault control can fail independently.
///
/// C10 requires each of these to fail on its own, for every dynamically introduced directory and
/// every journal entry. They are named rather than numbered so that a control says which barrier
/// it broke, and a report can say which one held.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Barrier {
    /// Exclusive creation of a new directory.
    DirectoryCreate,
    /// Synchronization of a newly created directory itself.
    DirectorySync,
    /// Synchronization of the parent that publishes a new directory's name.
    DirectoryParentSync,
    /// Writing a complete private stage file.
    StageWrite,
    /// `sync_all` on a complete stage file.
    StageSync,
    /// Reading a stage back and checking it before publication.
    StageReadback,
    /// No-replacement publication of a stage under its final name.
    Publish,
    /// Synchronization of the directory that publishes an entry's name.
    PublishParentSync,
    /// Writing the private chart snapshot.
    ChartWrite,
    /// Writing the private values document.
    ValuesWrite,
    /// Removing an own claim on ordinary safe completion.
    ClaimRelease,
}

impl Barrier {
    /// The barrier's exact name, for a diagnostic and for a control that selects one.
    pub fn name(self) -> &'static str {
        match self {
            Self::DirectoryCreate => "DirectoryCreate",
            Self::DirectorySync => "DirectorySync",
            Self::DirectoryParentSync => "DirectoryParentSync",
            Self::StageWrite => "StageWrite",
            Self::StageSync => "StageSync",
            Self::StageReadback => "StageReadback",
            Self::Publish => "Publish",
            Self::PublishParentSync => "PublishParentSync",
            Self::ChartWrite => "ChartWrite",
            Self::ValuesWrite => "ValuesWrite",
            Self::ClaimRelease => "ClaimRelease",
        }
    }
}

/// Every [`Barrier`] variant, asserted exhaustive by this module's own `match`.
pub const BARRIERS: &[Barrier] = &[
    Barrier::DirectoryCreate,
    Barrier::DirectorySync,
    Barrier::DirectoryParentSync,
    Barrier::StageWrite,
    Barrier::StageSync,
    Barrier::StageReadback,
    Barrier::Publish,
    Barrier::PublishParentSync,
    Barrier::ChartWrite,
    Barrier::ValuesWrite,
    Barrier::ClaimRelease,
];

/// What a reader can establish about one filesystem entry without following a link.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FileFacts {
    /// Whether the entry itself is a symbolic link.
    pub symlink: bool,
    /// Whether the entry is a regular file.
    pub regular: bool,
    /// Whether the entry is a directory.
    pub directory: bool,
    /// The owning UID.
    pub uid: u32,
    /// The owning GID.
    pub gid: u32,
    /// The permission bits.
    pub mode: u32,
    /// The device the entry lives on.
    pub device: u64,
    /// The entry's inode.
    pub inode: u64,
    /// The entry's size in bytes.
    pub size: u64,
}

impl FileFacts {
    /// Whether group or other can write this entry.
    pub fn shared_writable(self) -> bool {
        self.mode & 0o022 != 0
    }
}

/// Who a path component must belong to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Trust {
    /// The trusted control account: root-owned, not writable by group or other.
    Administrative,
    /// The non-root executor: owned by the executor UID, not writable by group or other.
    Executor(u32),
}

/// The evidence and the faultable boundaries the recovery engine decides on.
///
/// Every method is evidence or a boundary. None of them is a decision: an implementation that
/// answers "this file is root-owned" does not thereby admit anything, and the algorithm that reads
/// it is the same algorithm in production and under test.
pub trait Host {
    /// Invocation-local monotonic milliseconds. Never a wall clock, never another invocation's.
    fn now_ms(&self) -> u64;

    /// A fresh nonce from the host OS random source.
    fn random_nonce(&self) -> Admitted<Uuid>;

    /// The executing UID.
    fn executor_uid(&self) -> u32;

    /// The independently provisioned local host identity.
    fn host_id(&self) -> Admitted<String>;

    /// Facts about one entry, without following a final symbolic link.
    fn facts(&self, path: &Path) -> Admitted<FileFacts>;

    /// The boundary hook consulted immediately before the real operation it names.
    ///
    /// [`RealHost`] admits every barrier; it exists so that a fault control can fail exactly one
    /// of them, and so that a driver process can be interrupted at exactly one of them.
    fn barrier(&self, barrier: Barrier, label: &str) -> Admitted<()>;
}

/// The production host: real syscalls, no injected evidence, every barrier admitted.
#[derive(Debug)]
pub struct RealHost {
    started: Instant,
    executor_uid: u32,
    host_id_path: std::path::PathBuf,
}

impl RealHost {
    /// The host as the shipped binary constructs it.
    ///
    /// `host_id_path` is the protected `host-id` file the trusted control account provisions; it
    /// is a parameter rather than a constant so the same production code can be pointed at the
    /// same file through a fixture root without the binary gaining a switch.
    pub fn new(host_id_path: impl Into<std::path::PathBuf>) -> Self {
        Self {
            started: Instant::now(),
            executor_uid: rustix::process::getuid().as_raw(),
            host_id_path: host_id_path.into(),
        }
    }
}

impl Host for RealHost {
    fn now_ms(&self) -> u64 {
        u64::try_from(self.started.elapsed().as_millis()).unwrap_or(u64::MAX)
    }

    fn random_nonce(&self) -> Admitted<Uuid> {
        let mut bytes = [0u8; 16];
        getrandom::fill(&mut bytes)
            .map_err(|error| invalid(format!("the host random source is unavailable: {error}")))?;
        Ok(nonce_from_bytes(bytes))
    }

    fn executor_uid(&self) -> u32 {
        self.executor_uid
    }

    fn host_id(&self) -> Admitted<String> {
        let text = fs::read_to_string(&self.host_id_path).map_err(|error| {
            Refusal::new(
                RefusalCode::StoreInvalid,
                format!("the provisioned host identity is unreadable: {error}"),
            )
        })?;
        Ok(text.trim_end_matches('\n').to_owned())
    }

    fn facts(&self, path: &Path) -> Admitted<FileFacts> {
        real_facts(path)
    }

    fn barrier(&self, _: Barrier, _: &str) -> Admitted<()> {
        Ok(())
    }
}

/// Reads one entry's facts without following a final symbolic link.
pub fn real_facts(path: &Path) -> Admitted<FileFacts> {
    use std::os::unix::fs::MetadataExt as _;
    let metadata = fs::symlink_metadata(path).map_err(|error| {
        Refusal::new(
            RefusalCode::StoreInvalid,
            format!("{} is unreadable: {error}", path.display()),
        )
    })?;
    Ok(FileFacts {
        symlink: metadata.file_type().is_symlink(),
        regular: metadata.file_type().is_file(),
        directory: metadata.file_type().is_dir(),
        uid: metadata.uid(),
        gid: metadata.gid(),
        mode: metadata.mode() & 0o7777,
        device: metadata.dev(),
        inode: metadata.ino(),
        size: metadata.size(),
    })
}

/// Formats sixteen random bytes as the version-4 UUID spelling.
pub fn nonce_from_bytes(mut bytes: [u8; 16]) -> Uuid {
    bytes[6] = (bytes[6] & 0x0f) | 0x40;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    let mut hex = String::with_capacity(32);
    for byte in bytes {
        use std::fmt::Write as _;
        let _ = write!(hex, "{byte:02x}");
    }
    Uuid::new(format!(
        "{}-{}-{}-{}-{}",
        &hex[0..8],
        &hex[8..12],
        &hex[12..16],
        &hex[16..20],
        &hex[20..32]
    ))
    .expect("sixteen bytes always render the admitted UUID spelling")
}

// --- The finite invocation --------------------------------------------------------------------

use std::path::PathBuf;

use model::{
    Authority, Digest, Index, InvocationContext, InvocationId, JournalFact, LockClaim, LockFormat,
    Observation, ObservationPhase, ProcessDisposition, Profile, ReleasePermit, ReleaseSnapshot,
    Stopped, Text,
};

/// The production installation prefix for the admitted Helm artifact.
pub const HELM_TOOLS_PREFIX: &str = model::HELM_INSTALL_PREFIX;

/// Everything the shared reconcile handler is given, in both command spellings.
///
/// The surface is the existing one — `--path`, `--current`, `--cache`, `--allow-removals`,
/// `--dry-run` and the timeout — plus `--authority`, which *selects* an entry from the protected
/// registry and never accepts a self-authorizing file, and `--retry-of`, which decodes an existing
/// `InvocationId` and is additional context rather than a selector over retained history.
#[derive(Debug, Clone)]
pub struct ReconcileRequest {
    /// The desired canonical `ess-deployment/1` document.
    pub path: PathBuf,
    /// The admitted baseline desired deployment. Omit for a first deployment.
    pub current: Option<PathBuf>,
    /// The digest-pinned chart cache root.
    pub cache: PathBuf,
    /// Whether the reviewed removal set is authorized.
    pub allow_removals: bool,
    /// Whether to report a local unverified preview and stop.
    pub dry_run: bool,
    /// The Helm wait timeout.
    pub timeout: String,
    /// The authority to select from the protected registry.
    pub authority: Option<Uuid>,
    /// An optional predecessor reference.
    pub retry_of: Option<InvocationId>,
}

/// The roots the shipped binary fixes and a test driver points at a fixture.
///
/// These are construction parameters, not flags. `ess` builds [`Roots::production`] and nothing
/// parses another value into one; a driver that wants a fixture arrangement links the library and
/// constructs its own, which is the whole of the offline qualification's "no shipped bypass".
#[derive(Debug, Clone)]
pub struct Roots {
    /// The protected registry root.
    pub registry: PathBuf,
    /// The admitted Helm installation prefix.
    pub helm_prefix: String,
}

impl Roots {
    /// The roots the shipped `ess` binary uses.
    pub fn production() -> Self {
        Self {
            registry: PathBuf::from(authority::PRODUCTION_ROOT),
            helm_prefix: HELM_TOOLS_PREFIX.to_owned(),
        }
    }
}

/// Opens the bounded authenticated API for one admitted authority and context.
///
/// A factory rather than a client, because the credential comes out of the protected kubeconfig
/// the authority pins, and the engine must not hold it.
pub trait ApiFactory {
    /// Opens the API for the pinned target through the authority's kubeconfig and context.
    fn open(&self, authority: &Authority, context: &str) -> Admitted<Box<dyn observe::Api>>;
}

/// Reads the authenticated Helm release metadata for one address, metadata only.
///
/// Separated from [`observe::Api`] because it runs the admitted Helm artifact rather than the API,
/// and because its result is release *metadata*: a description, a revision, a storage UID, a
/// manifest inventory and a hook count. No Secret value is read or returned.
pub trait HelmReader {
    /// The release's authenticated metadata, or `None` when release storage is absent.
    fn release(
        &self,
        permit: &ReleasePermit,
        context: &str,
    ) -> Admitted<Option<model::HelmIdentity>>;
}

/// What one finite invocation established, in the order it established it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Report {
    /// The invocation identity, when one was reserved.
    pub invocation: Option<InvocationId>,
    /// The selected operations, in execution order.
    pub selected: Vec<String>,
    /// The operations whose required outcome and durable recording were established.
    pub settled: Vec<String>,
    /// The operation the invocation stopped at, if it stopped.
    pub unresolved: Option<String>,
    /// The refusal, when the invocation did not complete.
    pub refusal: Option<Refusal>,
}

impl Report {
    /// Whether every selected operation was accounted for.
    pub fn complete(&self) -> bool {
        self.refusal.is_none() && self.settled.len() == self.selected.len()
    }

    /// The accepted text rendering: the completed prefix and the unresolved remainder.
    ///
    /// It never says "rolled back", "absent" or "successful" about something it could not
    /// establish, and it names the unresolved release rather than reporting a global outcome.
    pub fn render(&self) -> String {
        use std::fmt::Write as _;
        let list = |names: &[String]| {
            if names.is_empty() {
                "(none)".to_owned()
            } else {
                names.join(", ")
            }
        };
        let mut text = String::new();
        let _ = writeln!(text, "selected: {}", list(&self.selected));
        let _ = writeln!(text, "settled: {}", list(&self.settled));
        if let Some(unresolved) = &self.unresolved {
            let _ = writeln!(text, "unresolved: {unresolved}");
        }
        match &self.refusal {
            Some(refusal) => {
                let _ = writeln!(
                    text,
                    "incomplete execution evidence — {}: {}",
                    refusal.code, refusal.detail
                );
            }
            None if self.complete() => {
                text.push_str("complete: every selected operation is accounted for\n");
            }
            None => text.push_str("incomplete execution evidence — accounting is short\n"),
        }
        text
    }
}

/// Reports a local unverified comparison preview and stops.
///
/// This is the whole of dry-run, and the "and stops" is the requirement. It returns before the
/// registry is read, before any executable is probed and before any external call, cache
/// population or recovery write — a preview cannot establish a live no-op, a successful
/// application or an absence, so anything it touched would be a side effect of a claim it is not
/// making.
pub fn preview(
    desired: &ess_deployment::DeploymentIr,
    current: Option<&ess_deployment::DeploymentIr>,
) -> (Vec<String>, Vec<String>) {
    let apply: Vec<String> = desired
        .rollout_order
        .iter()
        .filter(|service| {
            current.and_then(|state| state.releases.get(*service)) != desired.releases.get(*service)
        })
        .map(ToString::to_string)
        .collect();
    let remove: Vec<String> = current
        .map(|state| {
            state
                .rollout_order
                .iter()
                .rev()
                .filter(|service| !desired.releases.contains_key(*service))
                .map(ToString::to_string)
                .collect()
        })
        .unwrap_or_default();
    (apply, remove)
}

/// The early reviewed-removal guard, evaluated before acquisition and before any recovery write.
///
/// R22's whole point: a retirement present with the flag absent rejects *before* both phases, so
/// nothing that would imply execution began has happened by the time the caller is told to review
/// the retirement set.
pub fn admit_removals(remove: &[String], allow_removals: bool) -> Admitted<()> {
    if remove.is_empty() || allow_removals {
        return Ok(());
    }
    Err(Refusal::new(
        RefusalCode::RemovalNotPermitted,
        format!(
            "the deployment removes {}; rerun with --allow-removals after reviewing the \
             retirement set",
            remove.join(", ")
        ),
    ))
}

/// The selected operations of one invocation, in execution order.
///
/// Desired releases first in canonical rollout order, then authorized retirements in **reverse**
/// baseline rollout order. Both halves come from the caller's documents; neither is nominated by
/// a journal or by matching environment names.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Operation {
    /// The service.
    pub service: String,
    /// Whether this operation retires a baseline-only release.
    pub retirement: bool,
}

/// Builds the ordered operation list from the admitted documents.
pub fn operations(
    desired: &ess_deployment::DeploymentIr,
    current: Option<&ess_deployment::DeploymentIr>,
) -> Vec<Operation> {
    let (apply, remove) = preview(desired, current);
    apply
        .into_iter()
        .map(|service| Operation {
            service,
            retirement: false,
        })
        .chain(remove.into_iter().map(|service| Operation {
            service,
            retirement: true,
        }))
        .collect()
}

/// Requires the admitted documents to be exactly the ones the authority pins.
///
/// `--current` remains admitted *intent*: its presence and digest must match the authority,
/// including the complete retirement set. Neither an old journal nor a matching environment name
/// can nominate a baseline in its place.
pub fn admit_documents(
    authority: &Authority,
    desired_bytes: &str,
    current_bytes: Option<&str>,
) -> Admitted<()> {
    let desired = Digest::of_bytes(desired_bytes.as_bytes());
    if desired != authority.desired_digest {
        return Err(Refusal::new(
            RefusalCode::BaselineMismatch,
            "the desired document is not the one the selected authority pins",
        ));
    }
    match (current_bytes, &authority.baseline_digest) {
        (None, None) => Ok(()),
        (Some(bytes), Some(pinned)) if &Digest::of_bytes(bytes.as_bytes()) == pinned => Ok(()),
        (Some(_), Some(_)) => Err(Refusal::new(
            RefusalCode::BaselineMismatch,
            "the baseline document is not the one the selected authority pins",
        )),
        (Some(_), None) => Err(Refusal::new(
            RefusalCode::BaselineMismatch,
            "a baseline was supplied and the selected authority pins none",
        )),
        (None, Some(_)) => Err(Refusal::new(
            RefusalCode::BaselineMismatch,
            "the selected authority pins a baseline and none was supplied",
        )),
    }
}

/// Builds the invocation context published as `Opened` at sequence zero.
pub fn opened(
    authority: &Authority,
    registry: &model::RegistryRef,
    retry_of: Option<InvocationId>,
    selected: &[Operation],
    allow_removals: bool,
) -> Admitted<InvocationContext> {
    if authority.profile != Profile::SingleHostGeneratedHelm1 {
        return Err(Refusal::new(
            RefusalCode::UnsupportedProfile,
            "this implementation admits only SingleHostGeneratedHelm1",
        ));
    }
    Ok(InvocationContext {
        authority: authority.clone(),
        registry: registry.clone(),
        retry_of,
        selected: selected
            .iter()
            .map(|operation| Text::new(operation.service.clone()))
            .collect::<Admitted<Vec<Text>>>()?,
        allow_removals,
    })
}

/// Builds this invocation's target exclusion claim.
pub fn claim(
    invocation: &InvocationId,
    authority: &Authority,
    registry: &model::RegistryRef,
) -> LockClaim {
    LockClaim {
        format: LockFormat::V1,
        invocation: invocation.clone(),
        authority_id: authority.authority_id.clone(),
        authority_revision: authority.revision,
        target: authority.target.clone(),
        registry: registry.clone(),
    }
}

/// Whether a retained predecessor claim has the caller's independently established quiescence.
///
/// The decision has to come out of the authority — the independently controlled source — and it
/// has to name the exact retained claim by its exact retained bytes. A journal record cannot
/// generate one, a successful spawn cannot, a direct-child exit cannot and the passage of time
/// cannot. Without it the invocation may read and report, and may not mutate or claim recovery.
pub fn quiescence_admits(authority: &Authority, retained: &journal::RetainedClaim) -> bool {
    authority.quiescence.iter().any(|decision| {
        decision.claim == retained.claim && decision.claim_digest == retained.digest
    })
}

/// The refusal a retained predecessor claim without quiescence produces.
pub fn blocked_by_predecessor(retained: &journal::RetainedClaim) -> Refusal {
    Refusal::new(
        RefusalCode::MutationBlocked,
        format!(
            "invocation {} still holds the target claim; readings remain point-in-time \
             observations until the caller's quiescence procedure names that exact claim",
            retained.claim.invocation.nonce
        ),
    )
}

/// The durable `Observed` fact for one acquired snapshot.
pub fn observation(
    operation: Index,
    phase: ObservationPhase,
    started_ms: u64,
    finished_ms: u64,
    snapshot: ReleaseSnapshot,
) -> Admitted<Observation> {
    let observation = Observation {
        operation,
        phase,
        started_ms: Index::new(started_ms)?,
        finished_ms: Index::new(finished_ms)?,
        snapshot,
    };
    observation.validate()?;
    Ok(observation)
}

/// The terminal `Stopped` fact for a refusal.
pub fn stopped(operation: Option<Index>, refusal: &Refusal) -> JournalFact {
    JournalFact::Stopped(Stopped {
        operation,
        reason: refusal.code,
    })
}

/// Requires an acknowledged disposition before any later mutation may be attempted.
///
/// Every other disposition — a definite non-launch included — stops later applies and removals.
/// A `NotLaunched` settles that *this* operation did not run; it does not authorize continuing
/// past the refusal that produced it.
pub fn admits_later_mutation(disposition: ProcessDisposition) -> bool {
    disposition == ProcessDisposition::Acknowledged
}

// --- The engine ---------------------------------------------------------------------------------

use chart::PreparedChart;
use model::{HelmIdentity, JournalFormat, RegistryRef, StoreHeader};
use process::Outcome;

/// The bounded Helm operations this profile performs, as a seam a driver can supply.
///
/// Not a convenience. The offline qualification requires the *production* admission, preparation,
/// journal and process code to run in a separate driver process against an independently
/// controlled target, and this is the join between the two: the engine below is the same code in
/// both, and what differs is the arrangement it is pointed at.
pub trait Helm {
    /// The release's authenticated metadata, or `None` when release storage is absent.
    fn release(&self, permit: &ReleasePermit, context: &str) -> Admitted<Option<HelmIdentity>>;

    /// Renders the admitted private chart with the exact private values and namespace.
    fn render(
        &self,
        chart: &PreparedChart,
        permit: &ReleasePermit,
        context: &str,
    ) -> Admitted<String>;

    /// Attempts one apply. A returned [`Outcome`] is what happened, not whether it was permitted.
    fn apply(
        &self,
        chart: &PreparedChart,
        permit: &ReleasePermit,
        context: &str,
        marker: &str,
        timeout: &str,
    ) -> Admitted<Outcome>;

    /// Attempts one removal.
    fn remove(&self, permit: &ReleasePermit, context: &str, timeout: &str) -> Admitted<Outcome>;
}

/// Opens the bounded authenticated API and the admitted Helm artifact for one authority.
pub trait Platform {
    /// The bounded authenticated API.
    fn api(&self, authority: &Authority, context: &str) -> Admitted<Box<dyn observe::Api>>;

    /// The admitted Helm artifact, hashed and probed.
    fn helm(&self, authority: &Authority, prefix: &str) -> Admitted<Box<dyn Helm>>;

    /// The verified original chart payload for one release, through the existing OCI proof.
    fn payload(&self, release: &ess_deployment::DeploymentRelease) -> Admitted<Vec<u8>>;

    /// A private transient directory for one operation's chart and values.
    fn private(&self, label: &str) -> Admitted<PathBuf>;
}

/// The documents one invocation was given, with the exact bytes their digests are taken over.
#[derive(Debug, Clone)]
pub struct Documents {
    /// The desired deployment.
    pub desired: ess_deployment::DeploymentIr,
    /// Its exact bytes.
    pub desired_bytes: String,
    /// The admitted baseline desired deployment.
    pub current: Option<ess_deployment::DeploymentIr>,
    /// Its exact bytes.
    pub current_bytes: Option<String>,
}

/// What one invocation established before it reserved anything.
struct Admission {
    registry: authority::AdmittedRegistry,
    authority: Authority,
    store: journal::Store,
    retained: Option<journal::RetainedClaim>,
    selected: Vec<Operation>,
}

struct Run<'a> {
    host: &'a dyn Host,
    roots: &'a Roots,
    platform: &'a dyn Platform,
    request: &'a ReconcileRequest,
    documents: &'a Documents,
}

/// Runs one finite invocation and reports what it established.
///
/// Every refusal returns a [`Report`] rather than an error, because a refusal is a result: it names
/// the completed prefix, the unresolved operation and the claim that could not be established.
/// What it never does is turn an unknown into a success to obtain a zero exit.
pub fn execute(
    host: &dyn Host,
    roots: &Roots,
    request: &ReconcileRequest,
    documents: &Documents,
    platform: &dyn Platform,
) -> Report {
    let run = Run {
        host,
        roots,
        platform,
        request,
        documents,
    };
    let mut report = Report {
        invocation: None,
        selected: Vec::new(),
        settled: Vec::new(),
        unresolved: None,
        refusal: None,
    };
    if let Err(refusal) = run.perform(&mut report) {
        report.refusal = Some(refusal);
    }
    report
}

impl Run<'_> {
    /// Everything that must be established before an invocation may be reserved at all.
    ///
    /// The order is the contract's: the reviewed-removal guard first, so a retirement with the
    /// flag absent rejects before anything implies execution began; then the whole active
    /// registry; then the documents against what the authority pins; then the store, its complete
    /// retained history, and the retained claim. Nothing here writes.
    fn admitted(&self, report: &mut Report) -> Admitted<Admission> {
        let selected = operations(&self.documents.desired, self.documents.current.as_ref());
        let removals: Vec<String> = selected
            .iter()
            .filter(|operation| operation.retirement)
            .map(|operation| operation.service.clone())
            .collect();
        // Before acquisition, before the registry, before any recovery write: a retirement with
        // the flag absent must reject without anything having happened that implies execution
        // began.
        admit_removals(&removals, self.request.allow_removals)?;
        report.selected = selected
            .iter()
            .map(|operation| operation.service.clone())
            .collect();

        let requested = self.request.authority.clone().ok_or_else(|| {
            Refusal::new(
                RefusalCode::AuthorityMismatch,
                "normal execution requires a caller-provisioned authority selected from the \
                 protected registry; dry-run remains a local preview",
            )
        })?;
        let (registry, authority) = authority::admit(self.host, &self.roots.registry, &requested)?;
        admit_documents(
            &authority,
            &self.documents.desired_bytes,
            self.documents.current_bytes.as_deref(),
        )?;

        let store = journal::open_store(self.host, Path::new(authority.host.state_root.as_str()))?;
        check_store(&store, &authority)?;

        // Every retained reservation and history in the store, whether or not `--retry-of` named
        // one. An omitted predecessor reference is missing context, never a narrower search.
        let histories = journal::scan_store(self.host, &store)?;
        // C05: a generation older than retained execution evidence is refused. Retained evidence
        // names the generation it ran under, and a registry that has gone backwards is not a
        // policy this invocation may adopt.
        for history in &histories {
            if let Some(context) = history.context() {
                authority::admit_generation(
                    context.registry.generation,
                    registry.reference.generation,
                )?;
            }
        }
        // The retained claim is the fence, and the only one. An unresolved `Prepared` is *why* a
        // claim is retained, so it sharpens the refusal rather than adding a second gate — and
        // once the caller's decision names that exact claim, both are satisfied. What quiescence
        // never does is settle the historical attribution: nothing below turns an indeterminate
        // preparation into an applied fact, and the restart observes and decides again.
        let retained = journal::read_claim(self.host, &store)?;
        if let Some(retained) = &retained {
            if !quiescence_admits(&authority, retained) {
                return Err(blocked_by_predecessor(retained));
            }
            // A decision names it, so this invocation may act — but it publishes no claim of its
            // own and releases nothing at the end. The retained one is still the predecessor's,
            // and removing it is the caller's step, not this executor's.
        }
        // The second half of the same fence, and the half a removed lock cannot switch off. C08's
        // procedure archives the claim and removes it, so by the time the next invocation runs
        // there may be nothing on disk to block on — while a retained `Prepared` with no durable
        // disposition is still exactly as indeterminate as it was. What settles it is the caller's
        // decision naming *that invocation*, wherever its claim now lives.
        for history in &histories {
            if history.unresolved_preparations().is_empty() {
                continue;
            }
            let named = authority
                .quiescence
                .iter()
                .any(|decision| decision.claim.invocation.nonce == history.nonce);
            if !named {
                return Err(Refusal::new(
                    RefusalCode::MutationBlocked,
                    format!(
                        "invocation {} retains a durable Prepared with no disposition, so its \
                         historical attribution stays unknown; the caller's quiescence procedure \
                         must name its claim before any later mutation",
                        history.nonce
                    ),
                ));
            }
        }
        Ok(Admission {
            registry,
            authority,
            store,
            retained,
            selected,
        })
    }

    fn perform(&self, report: &mut Report) -> Admitted<()> {
        let Admission {
            registry,
            authority,
            store,
            retained,
            selected,
        } = self.admitted(report)?;
        let reservation = journal::reserve(self.host, &store)?;
        report.invocation = Some(reservation.id().clone());
        // An invocation proceeding under an admitted quiescence decision does not publish a claim
        // of its own: the predecessor's is still the retained one, and this executor never
        // reclaims another invocation's claim. Which also means it has nothing to release at the
        // end — removing the archived claim is the caller's administrative step, not this run's.
        let claim = claim(reservation.id(), &authority, &registry.reference);
        let published = retained.is_none();
        if published {
            journal::publish_claim(self.host, &store, &claim)?;
        }
        authority::recheck(self.host, &self.roots.registry, &registry.reference)?;

        let mut record = journal::Journal::open(&reservation);
        let context = opened(
            &authority,
            &registry.reference,
            self.request.retry_of.clone(),
            &selected,
            self.request.allow_removals,
        )?;
        record.append(self.host, JournalFact::Opened(Box::new(context)))?;

        let alias = alias_of(&authority).to_owned();
        let api = self.platform.api(&authority, &alias)?;
        observe::authenticate(api.as_ref(), &authority.target, &authority.principal)?;
        let helm = self.platform.helm(&authority, &self.roots.helm_prefix)?;

        for (index, operation) in selected.iter().enumerate() {
            let outcome = self.operation(
                &mut record,
                &authority,
                &registry.reference,
                api.as_ref(),
                helm.as_ref(),
                index,
                operation,
            );
            match outcome {
                Ok(()) => report.settled.push(operation.service.clone()),
                Err(refusal) => {
                    report.unresolved = Some(operation.service.clone());
                    let at = journal::index(index).ok();
                    let _ = record.append(self.host, stopped(at, &refusal));
                    return Err(refusal);
                }
            }
        }

        let accounted = journal::index(selected.len())?;
        record.append(self.host, JournalFact::Completed(accounted))?;
        if published {
            journal::release_claim(self.host, &store, reservation.id())?;
        }
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn operation(
        &self,
        record: &mut journal::Journal,
        authority: &Authority,
        registry: &RegistryRef,
        api: &dyn observe::Api,
        helm: &dyn Helm,
        index: usize,
        operation: &Operation,
    ) -> Admitted<()> {
        let permit = authority.permit(&operation.service).ok_or_else(|| {
            Refusal::new(
                RefusalCode::AuthorityMismatch,
                format!(
                    "the selected authority admits no release permit for {}",
                    operation.service
                ),
            )
        })?;
        let at = journal::index(index)?;
        observe::authenticate_namespace(api, permit)?;

        // Artifact and local preparation finishes before the mutation-authorizing observation
        // begins. Publishing `Prepared` is a separate, later step: preparation is not a decision.
        let prepared = if operation.retirement {
            None
        } else {
            Some(self.prepare(helm, authority, permit, &operation.service)?)
        };

        let started = self.host.now_ms();
        let identity = helm.release(permit, alias_of(authority))?;
        let snapshot = observe::snapshot(api, permit, identity)?;
        let finished = self.host.now_ms();
        let before = observation(
            at,
            ObservationPhase::Before,
            started,
            finished,
            snapshot.clone(),
        )?;
        let sequence = record.append(self.host, JournalFact::Observed(before.clone()))?;

        match observe::decide(
            permit,
            &snapshot,
            &authority.authority_id,
            operation.retirement,
        )? {
            observe::Predicate::DesiredMatches | observe::Predicate::AlreadyAbsent => {
                // A fresh match or an authenticated absence is recorded as an observation and
                // skips the mutation. It does not require `Prepared`, and it does not fabricate an
                // earlier application fact to explain what it found.
                return Ok(());
            }
            observe::Predicate::ApplyFromBaseline
            | observe::Predicate::FirstCreation
            | observe::Predicate::RemoveBaseline => {}
        }

        record.append(self.host, journal::prepared(at, sequence))?;

        // The final checks, all inside the same budget the observation started.
        observe::admit_freshness(&before, self.host.now_ms())?;
        authority::recheck(self.host, &self.roots.registry, registry)?;
        observe::authenticate(api, &authority.target, &authority.principal)?;
        process::admit(self.host, &authority.host.helm, &self.roots.helm_prefix)?;

        let outcome = if operation.retirement {
            helm.remove(permit, alias_of(authority), &self.request.timeout)?
        } else {
            let chart = prepared
                .as_ref()
                .ok_or_else(|| Refusal::new(RefusalCode::PreparationFailed, "no prepared chart"))?;
            helm.apply(
                chart,
                permit,
                alias_of(authority),
                &model::ownership_marker(&authority.authority_id, &permit.incarnation),
                &self.request.timeout,
            )?
        };
        let disposition = outcome.disposition();
        record.append(self.host, journal::outcome(at, disposition))?;
        if !admits_later_mutation(disposition) {
            return Err(Refusal::new(
                match disposition {
                    ProcessDisposition::NotLaunched => RefusalCode::LaunchFailed,
                    _ => RefusalCode::EffectIndeterminate,
                },
                format!(
                    "{}: the call was {} and the target's state is not established by this \
                     invocation",
                    operation.service,
                    if disposition == ProcessDisposition::NotLaunched {
                        "definitely not launched"
                    } else {
                        "started and its effect cannot be established"
                    }
                ),
            ));
        }

        let empty = PreparedChart {
            chart_path: PathBuf::new(),
            values_path: PathBuf::new(),
            values: String::new(),
            payload_digest: Digest::of_bytes(b""),
            rendered: Vec::new(),
        };
        let admitted = prepared.as_ref().unwrap_or(&empty);
        self.confirm(
            record, authority, api, helm, at, permit, operation, admitted,
        )
    }

    /// The required fresh, durable readback after an acknowledged mutation.
    ///
    /// An acknowledgement is historical process evidence about a child, and nothing more. What
    /// establishes the current state is this observation, and it has to complete and become
    /// durable before any later operation may mutate.
    #[allow(clippy::too_many_arguments)]
    fn confirm(
        &self,
        record: &mut journal::Journal,
        authority: &Authority,
        api: &dyn observe::Api,
        helm: &dyn Helm,
        at: Index,
        permit: &ReleasePermit,
        operation: &Operation,
        prepared: &PreparedChart,
    ) -> Admitted<()> {
        let started = self.host.now_ms();
        let identity = helm.release(permit, alias_of(authority))?;
        let (snapshot, live) = observe::observe(api, permit, identity)?;
        let finished = self.host.now_ms();
        let after = observation(
            at,
            ObservationPhase::After,
            started,
            finished,
            snapshot.clone(),
        )?;
        record.append(self.host, JournalFact::Observed(after))?;

        if operation.retirement {
            let baseline = permit.baseline.as_ref().ok_or_else(|| {
                Refusal::new(RefusalCode::InvalidInput, "a retirement has a baseline")
            })?;
            observe::absence_holds(&snapshot, baseline)?;
        } else {
            let desired = permit.desired.as_ref().ok_or_else(|| {
                Refusal::new(
                    RefusalCode::InvalidInput,
                    "an apply has a desired projection",
                )
            })?;
            if let Some(identity) = &snapshot.helm {
                observe::admit_ownership(identity, &authority.authority_id, &permit.incarnation)?;
                observe::admit_manifest(identity, desired)?;
            } else {
                return Err(Refusal::new(
                    RefusalCode::ObservationUnavailable,
                    "no release storage was observed after an acknowledged apply",
                ));
            }
            if !observe::projection_holds(&snapshot, desired) {
                return Err(Refusal::new(
                    RefusalCode::ObservedDrift,
                    "the desired projection does not hold after an acknowledged apply",
                ));
            }
            if !observe::addresses_absent(&snapshot, &permit.baseline_only()) {
                return Err(Refusal::new(
                    RefusalCode::DirectObjectsRemain,
                    "a baseline-only object is still present after an acknowledged apply",
                ));
            }
            // Every authored rendered field, recursively, as well as the approved projection
            // digest. The digest alone cannot catch a caller fingerprint that approved an object
            // whose image, selector, replica count or secret reference is not what the chart
            // authored — which is exactly what C07 step 8 says it must not silently override.
            observe::authored_holds(&prepared.rendered, &live)?;
        }
        Ok(())
    }

    fn prepare(
        &self,
        helm: &dyn Helm,
        authority: &Authority,
        permit: &ReleasePermit,
        service: &str,
    ) -> Admitted<PreparedChart> {
        let release = self
            .documents
            .desired
            .releases
            .get(
                &ess_deployment::Identifier::new(service)
                    .map_err(|error| Refusal::new(RefusalCode::InvalidInput, format!("{error}")))?,
            )
            .ok_or_else(|| {
                Refusal::new(
                    RefusalCode::InvalidInput,
                    format!("the desired document has no release for {service}"),
                )
            })?;
        let desired = permit.desired.as_ref().ok_or_else(|| {
            Refusal::new(
                RefusalCode::BaselineMismatch,
                format!("the authority admits no desired projection for {service}"),
            )
        })?;
        let runtime = chart::read_runtime(self.host, &self.roots.registry, &desired.chart)?;
        let payload = self.platform.payload(release)?;
        chart::admit_payload(&payload, &runtime, &desired.chart)?;

        let values = values_document(release)?;
        let directory = self.platform.private(service)?;
        let prepared = chart::prepare(self.host, &directory, &payload, &values, service)?;
        let rendered = helm.render(&prepared, permit, alias_of(authority))?;
        let objects = chart::admit_rendered(&rendered, &permit.namespace.name)?;
        chart::admit_inventory(&objects, desired)?;
        Ok(PreparedChart {
            rendered: objects,
            ..prepared
        })
    }
}

/// The first admitted context alias, which is the one every operation of this invocation uses.
fn alias_of(authority: &Authority) -> &str {
    authority
        .contexts
        .first()
        .expect("an admitted authority names at least one context")
        .as_str()
}

/// The private values document for one release, in the exact existing shape.
///
/// Secret *references* stay references. No credential and no Secret value is written here, and
/// nothing in the document is a value the caller did not already put in its own intent.
pub fn values_document(release: &ess_deployment::DeploymentRelease) -> Admitted<String> {
    serde_yaml::to_string(&serde_json::json!({
        "serviceAccount": {"name": &release.service_account},
        "images": release.images.iter().map(|(name, artifact)| {
            (name.as_str(), serde_json::json!({
                "repository": &artifact.reference,
                "digest": &artifact.digest,
            }))
        }).collect::<std::collections::BTreeMap<_, _>>(),
        "config": &release.config,
        "secrets": &release.secrets,
        "endpoints": &release.endpoints,
    }))
    .map_err(|error| {
        Refusal::new(
            RefusalCode::PreparationFailed,
            format!("the private values could not be serialized: {error}"),
        )
    })
}

/// Requires the admitted store to belong to this authority's host, epoch and physical target.
fn check_store(store: &journal::Store, authority: &Authority) -> Admitted<()> {
    let header: &StoreHeader = store.header();
    if header.store_epoch != authority.host.store_epoch {
        return Err(Refusal::new(
            RefusalCode::StoreInvalid,
            "the store's epoch is not the one the authority provisions",
        ));
    }
    if header.host_id != authority.host.host_id {
        return Err(Refusal::new(
            RefusalCode::StoreInvalid,
            "the store belongs to another host",
        ));
    }
    if header.target != authority.target {
        return Err(Refusal::new(
            RefusalCode::TargetMismatch,
            "the store's lock namespace is bound to another physical target",
        ));
    }
    Ok(())
}

/// The journal format this implementation writes and reads.
pub const JOURNAL_FORMAT: JournalFormat = JournalFormat::V1;

// --- The production platform ---------------------------------------------------------------------

use std::cell::RefCell;
use std::process::Command;

use ess_kubernetes::recovery::{RecoveryClient, RecoveryError, Request};
use observe::{ApiRead, Identity};

/// The production arrangement: the real API over TLS, the real admitted artifact, the real cache.
///
/// Constructed unconditionally by the shipped `ess` binary. There is no flag, environment variable
/// or fixture profile that selects a different one; a test target that needs another arrangement
/// links the library and supplies its own [`Platform`].
pub struct ProductionPlatform {
    cache: PathBuf,
    scratch: RefCell<Vec<crate::TemporaryDirectory>>,
}

impl std::fmt::Debug for ProductionPlatform {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ProductionPlatform")
            .field("cache", &self.cache)
            .finish_non_exhaustive()
    }
}

impl ProductionPlatform {
    /// The platform over one digest-pinned chart cache root.
    pub fn new(cache: impl Into<PathBuf>) -> Self {
        Self {
            cache: cache.into(),
            scratch: RefCell::new(Vec::new()),
        }
    }
}

fn transport(error: &RecoveryError) -> Refusal {
    // The adapter's own diagnostics are already coarse, and this keeps them that way: what crosses
    // into a refusal is the kind of failure, never the server's words, a header or a URL.
    Refusal::new(
        RefusalCode::ObservationUnavailable,
        format!("the bounded authenticated read did not complete ({error})"),
    )
}

/// Reads the bearer token out of a protected kubeconfig, and returns it to nowhere else.
///
/// Deliberately not part of `authority::read_kubeconfig`, which returns *facts* about a kubeconfig
/// and is called from the registry scan. This function's result goes straight into an
/// `Authorization` header inside `ess-kubernetes` and is never logged, journalled or serialized.
fn bearer(kubeconfig: &Path) -> Admitted<String> {
    let text = std::fs::read_to_string(kubeconfig).map_err(|error| {
        Refusal::new(
            RefusalCode::PrincipalMismatch,
            format!("the protected kubeconfig is unreadable: {error}"),
        )
    })?;
    let document: serde_yaml::Value = serde_yaml::from_str(&text).map_err(|_| {
        Refusal::new(
            RefusalCode::PrincipalMismatch,
            "the protected kubeconfig is not a YAML document",
        )
    })?;
    let user = document
        .get("users")
        .and_then(serde_yaml::Value::as_sequence)
        .and_then(|users| users.first())
        .and_then(|user| user.get("user"))
        .ok_or_else(|| {
            Refusal::new(
                RefusalCode::PrincipalMismatch,
                "the protected kubeconfig defines no executing user",
            )
        })?;
    if let Some(token) = user.get("token").and_then(serde_yaml::Value::as_str) {
        return Ok(token.to_owned());
    }
    let path = user
        .get("token-file")
        .and_then(serde_yaml::Value::as_str)
        .ok_or_else(|| {
            Refusal::new(
                RefusalCode::PrincipalMismatch,
                "the protected kubeconfig supplies no admitted credential",
            )
        })?;
    std::fs::read_to_string(path)
        .map(|token| token.trim_end_matches('\n').to_owned())
        .map_err(|error| {
            Refusal::new(
                RefusalCode::PrincipalMismatch,
                format!("the admitted credential is unreadable: {error}"),
            )
        })
}

/// The certificate authority bytes the pinned target trusts, from the protected kubeconfig.
fn trust_anchor(kubeconfig: &Path) -> Admitted<Vec<u8>> {
    let text = std::fs::read_to_string(kubeconfig).map_err(|error| {
        Refusal::new(
            RefusalCode::TargetMismatch,
            format!("the protected kubeconfig is unreadable: {error}"),
        )
    })?;
    let document: serde_yaml::Value = serde_yaml::from_str(&text).map_err(|_| {
        Refusal::new(
            RefusalCode::TargetMismatch,
            "the protected kubeconfig is not a YAML document",
        )
    })?;
    let encoded = document
        .get("clusters")
        .and_then(serde_yaml::Value::as_sequence)
        .and_then(|clusters| clusters.first())
        .and_then(|cluster| cluster.get("cluster"))
        .and_then(|cluster| cluster.get("certificate-authority-data"))
        .and_then(serde_yaml::Value::as_str)
        .ok_or_else(|| {
            Refusal::new(
                RefusalCode::TargetMismatch,
                "the protected kubeconfig embeds no certificate authority",
            )
        })?;
    decode_base64(encoded).ok_or_else(|| {
        Refusal::new(
            RefusalCode::TargetMismatch,
            "the embedded certificate authority is not base64",
        )
    })
}

/// Decodes standard base64 without pulling in a dependency for sixty-four characters.
fn decode_base64(text: &str) -> Option<Vec<u8>> {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut bits = 0u32;
    let mut held = 0u32;
    let mut out = Vec::new();
    for byte in text.bytes().filter(|byte| !byte.is_ascii_whitespace()) {
        if byte == b'=' {
            break;
        }
        let value = ALPHABET.iter().position(|candidate| *candidate == byte)?;
        bits = (bits << 6) | u32::try_from(value).ok()?;
        held += 6;
        if held >= 8 {
            held -= 8;
            out.push(u8::try_from((bits >> held) & 0xff).ok()?);
        }
    }
    Some(out)
}

struct LiveApi {
    client: RecoveryClient,
}

impl observe::Api for LiveApi {
    fn identity_namespace(&self) -> Admitted<Identity> {
        let response = self
            .client
            .request(&Request::Namespace {
                name: model::IDENTITY_NAMESPACE.to_owned(),
            })
            .map_err(|error| transport(&error))?;
        identity_of(&response.body)
    }

    fn self_subject(&self) -> Admitted<Identity> {
        let response = self
            .client
            .request(&Request::SelfSubject)
            .map_err(|error| transport(&error))?;
        let user = response
            .body
            .get("status")
            .and_then(|status| status.get("userInfo"))
            .ok_or_else(|| {
                Refusal::new(
                    RefusalCode::PrincipalMismatch,
                    "the self-subject review carried no user information",
                )
            })?;
        let username = user
            .get("username")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default();
        // `system:serviceaccount:<namespace>:<name>` is the only shape this profile admits.
        let mut parts = username.split(':');
        let (Some("system"), Some("serviceaccount"), Some(namespace), Some(name)) =
            (parts.next(), parts.next(), parts.next(), parts.next())
        else {
            return Err(Refusal::new(
                RefusalCode::PrincipalMismatch,
                "the authenticated identity is not a ServiceAccount",
            ));
        };
        Ok(Identity {
            name: name.to_owned(),
            namespace: Some(namespace.to_owned()),
            uid: user
                .get("uid")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_default()
                .to_owned(),
        })
    }

    fn namespace(&self, name: &str) -> Admitted<Identity> {
        let response = self
            .client
            .request(&Request::Namespace {
                name: name.to_owned(),
            })
            .map_err(|error| transport(&error))?;
        identity_of(&response.body)
    }

    fn object(&self, namespace: &str, address: &model::ObjectAddress) -> Admitted<ApiRead> {
        let request = match address.kind {
            model::ObjectKind::Deployment => Request::Workload {
                namespace: namespace.to_owned(),
                plural: "deployments".to_owned(),
                name: address.name.to_string(),
            },
            model::ObjectKind::StatefulSet => Request::Workload {
                namespace: namespace.to_owned(),
                plural: "statefulsets".to_owned(),
                name: address.name.to_string(),
            },
            model::ObjectKind::Service => Request::Service {
                namespace: namespace.to_owned(),
                name: address.name.to_string(),
            },
        };
        let response = self
            .client
            .request(&request)
            .map_err(|error| transport(&error))?;
        if response.status == 404 {
            return Ok(ApiRead::Absent);
        }
        let metadata = response.body.get("metadata").ok_or_else(|| {
            Refusal::new(
                RefusalCode::ObservationUnavailable,
                "an authenticated read returned an object with no metadata",
            )
        })?;
        Ok(ApiRead::Present {
            uid: metadata
                .get("uid")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_default()
                .to_owned(),
            resource_version: metadata
                .get("resourceVersion")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_default()
                .to_owned(),
            object: response.body,
        })
    }
}

fn identity_of(body: &serde_json::Value) -> Admitted<Identity> {
    let metadata = body.get("metadata").ok_or_else(|| {
        Refusal::new(
            RefusalCode::ObservationUnavailable,
            "an authenticated identity read returned no metadata",
        )
    })?;
    Ok(Identity {
        name: metadata
            .get("name")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .to_owned(),
        namespace: None,
        uid: metadata
            .get("uid")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .to_owned(),
    })
}

struct LiveHelm {
    admitted: process::AdmittedHelm,
    kubeconfig: String,
    private: PathBuf,
}

impl LiveHelm {
    fn command(&self, arguments: &[String]) -> Admitted<Command> {
        process::admit_arguments(arguments)?;
        let mut command = Command::new(self.admitted.path());
        command.args(arguments);
        command.env_clear();
        for (key, value) in process::environment(&self.private, &Text::new(&self.kubeconfig)?) {
            command.env(key, value);
        }
        Ok(command)
    }
}

impl Helm for LiveHelm {
    fn release(&self, permit: &ReleasePermit, context: &str) -> Admitted<Option<HelmIdentity>> {
        let status = self.command(&[
            "status".to_owned(),
            permit.release_name.to_string(),
            "--output".to_owned(),
            "json".to_owned(),
            "--namespace".to_owned(),
            permit.namespace.name.to_string(),
            "--kubeconfig".to_owned(),
            self.kubeconfig.clone(),
            "--kube-context".to_owned(),
            context.to_owned(),
        ])?;
        let outcome = process::run(
            status,
            process::OUTPUT_LIMIT,
            std::time::Duration::from_secs(60),
        )?;
        if outcome.status != Some(0) {
            // Helm's own error text is not an absence predicate. What this establishes is that the
            // release metadata could not be read, and the authenticated API reads decide absence.
            return Ok(None);
        }
        let decoded: serde_json::Value = serde_json::from_slice(&outcome.stdout).map_err(|_| {
            Refusal::new(
                RefusalCode::ObservationUnavailable,
                "the release status was not the admitted output shape",
            )
        })?;
        let revision = decoded
            .get("version")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or_default();
        let manifest = self.command(&[
            "get".to_owned(),
            "manifest".to_owned(),
            permit.release_name.to_string(),
            "--revision".to_owned(),
            revision.to_string(),
            "--namespace".to_owned(),
            permit.namespace.name.to_string(),
            "--kubeconfig".to_owned(),
            self.kubeconfig.clone(),
            "--kube-context".to_owned(),
            context.to_owned(),
        ])?;
        let manifest = process::run(
            manifest,
            process::OUTPUT_LIMIT,
            std::time::Duration::from_secs(60),
        )?;
        let rendered = String::from_utf8(manifest.stdout).map_err(|_| {
            Refusal::new(
                RefusalCode::ObservationUnavailable,
                "the stored manifest was not UTF-8",
            )
        })?;
        let inventory = chart::admit_rendered(&rendered, &permit.namespace.name)?;
        let hooks = self.command(&[
            "get".to_owned(),
            "hooks".to_owned(),
            permit.release_name.to_string(),
            "--revision".to_owned(),
            revision.to_string(),
            "--namespace".to_owned(),
            permit.namespace.name.to_string(),
            "--kubeconfig".to_owned(),
            self.kubeconfig.clone(),
            "--kube-context".to_owned(),
            context.to_owned(),
        ])?;
        let hooks = process::run(
            hooks,
            process::OUTPUT_LIMIT,
            std::time::Duration::from_secs(60),
        )?;
        let hook_count = u64::from(!hooks.stdout.iter().all(u8::is_ascii_whitespace));
        Ok(Some(HelmIdentity {
            description: Text::new(
                decoded
                    .get("info")
                    .and_then(|info| info.get("description"))
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("(none)"),
            )?,
            revision: Index::new(revision)?,
            storage_uid: Text::new(
                decoded
                    .get("info")
                    .and_then(|info| info.get("last_deployed"))
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("(none)"),
            )?,
            manifest: inventory.into_iter().map(|object| object.address).collect(),
            hook_count: Index::new(hook_count)?,
        }))
    }

    fn render(
        &self,
        chart: &PreparedChart,
        permit: &ReleasePermit,
        _context: &str,
    ) -> Admitted<String> {
        let command = self.command(&[
            "template".to_owned(),
            permit.release_name.to_string(),
            chart.chart_path.display().to_string(),
            "--values".to_owned(),
            chart.values_path.display().to_string(),
            "--namespace".to_owned(),
            permit.namespace.name.to_string(),
            "--no-hooks".to_owned(),
            "--skip-crds".to_owned(),
        ])?;
        let outcome = process::run(
            command,
            process::OUTPUT_LIMIT,
            std::time::Duration::from_secs(60),
        )?;
        if outcome.status != Some(0) {
            return Err(Refusal::new(
                RefusalCode::PreparationFailed,
                "the admitted chart did not render",
            ));
        }
        String::from_utf8(outcome.stdout).map_err(|_| {
            Refusal::new(
                RefusalCode::PreparationFailed,
                "the rendered stream was not UTF-8",
            )
        })
    }

    fn apply(
        &self,
        chart: &PreparedChart,
        permit: &ReleasePermit,
        context: &str,
        marker: &str,
        timeout: &str,
    ) -> Admitted<Outcome> {
        let arguments = process::apply_arguments(process::Apply {
            release: permit.release_name.as_str(),
            chart: &chart.chart_path,
            values: &chart.values_path,
            namespace: permit.namespace.name.as_str(),
            kubeconfig: &self.kubeconfig,
            context,
            marker,
            timeout,
        });
        let command = self.command(&arguments)?;
        process::run(
            command,
            process::OUTPUT_LIMIT,
            std::time::Duration::from_secs(900),
        )
    }

    fn remove(&self, permit: &ReleasePermit, context: &str, timeout: &str) -> Admitted<Outcome> {
        let arguments = process::remove_arguments(
            permit.release_name.as_str(),
            permit.namespace.name.as_str(),
            &self.kubeconfig,
            context,
            timeout,
        );
        let command = self.command(&arguments)?;
        process::run(
            command,
            process::OUTPUT_LIMIT,
            std::time::Duration::from_secs(900),
        )
    }
}

impl Platform for ProductionPlatform {
    fn api(&self, authority: &Authority, _context: &str) -> Admitted<Box<dyn observe::Api>> {
        let kubeconfig = Path::new(authority.host.kubeconfig.as_str());
        let client = RecoveryClient::connect(
            authority.target.api_server.as_str(),
            &trust_anchor(kubeconfig)?,
            &bearer(kubeconfig)?,
        )
        .map_err(|error| transport(&error))?;
        Ok(Box::new(LiveApi { client }))
    }

    fn helm(&self, authority: &Authority, prefix: &str) -> Admitted<Box<dyn Helm>> {
        let admitted = process::admit(
            &RealHost::new("/etc/ess/recovery/host-id"),
            &authority.host.helm,
            prefix,
        )?;
        let private = self.private("helm")?;
        Ok(Box::new(LiveHelm {
            admitted,
            kubeconfig: authority.host.kubeconfig.to_string(),
            private,
        }))
    }

    fn payload(&self, release: &ess_deployment::DeploymentRelease) -> Admitted<Vec<u8>> {
        if release.chart.kind != ess_deployment::ArtifactKind::HelmChart {
            return Err(Refusal::new(
                RefusalCode::InvalidInput,
                format!("{} does not select a Helm chart artifact", release.service),
            ));
        }
        let reference = format!(
            "{}@{}",
            release.chart.reference.trim_start_matches("oci://"),
            release.chart.digest
        );
        crate::oci_cache::payload(&reference, &self.cache, crate::oci_cache::Profile::Helm).map_err(
            |error| {
                Refusal::new(
                    RefusalCode::PreparationFailed,
                    format!("the pinned chart payload was not proved: {error}"),
                )
            },
        )
    }

    fn private(&self, label: &str) -> Admitted<PathBuf> {
        let directory = crate::TemporaryDirectory::create(&format!("ess-recovery-{label}"))
            .map_err(|error| {
                Refusal::new(
                    RefusalCode::PreparationFailed,
                    format!("a private transient directory could not be created: {error}"),
                )
            })?;
        let path = directory.path().to_path_buf();
        self.scratch.borrow_mut().push(directory);
        Ok(path)
    }
}
