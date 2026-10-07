//! Complete browser product: original-byte Rust admission and explicit independent installation.
//! The browser renders declarations; only the Rust Runner creates execution evidence.
pub mod abi;
pub mod bundle;
pub mod host;
pub mod presentation;

use crate::{Clock, ConformanceTarget, CountReport, CountRun, Ids, Runner, RunnerConfig};
use std::{collections::BTreeSet, fmt, marker::PhantomData};

/// Product ABI identity, independent of suite/report versions.
pub const ABI: &str = "ess-conformance-browser-abi/1";
/// Exact host/runtime ABI version.
pub const ABI_VERSION: u32 = 0x0001_0000;
/// Whole request/response envelope bound (unmeasured profile; validated by product tests).
pub const MAX_FRAME: usize = 64 * 1024 * 1024;
/// Safe closed product failures; no source, target or panic text is retained.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum Error {
    /// Invalid framing or length.
    InvalidFrame = 1,
    /// Unsupported host/runtime identity.
    IncompatibleAbi,
    /// Invalid bundle metadata or file binding.
    InvalidBundle,
    /// Original suite, lineage or source admission refused.
    AdmissionRefused,
    /// Stale or unknown capability.
    InvalidHandle,
    /// Ordinary suites do not acquire coverage selection.
    SelectionNotAvailable,
    /// Installation was not available.
    InstallationRequired,
    /// Product resource bound exceeded.
    ResourceLimit,
    /// Installation or execution could not complete.
    ExecutionError,
    /// An internal result could not be encoded.
    InternalFailure,
}
impl Error {
    /// Stable disclosure-safe category.
    pub const fn code(self) -> &'static str {
        match self {
            Self::InvalidFrame => "invalid_frame",
            Self::IncompatibleAbi => "incompatible_abi",
            Self::InvalidBundle => "invalid_bundle",
            Self::AdmissionRefused => "admission_refused",
            Self::InvalidHandle => "invalid_handle",
            Self::SelectionNotAvailable => "selection_not_available",
            Self::InstallationRequired => "installation_required",
            Self::ResourceLimit => "resource_limit",
            Self::ExecutionError => "execution_error",
            Self::InternalFailure => "internal_failure",
        }
    }
}
impl fmt::Display for Error {
    fn fmt(&self, out: &mut fmt::Formatter<'_>) -> fmt::Result {
        out.write_str(self.code())
    }
}
impl std::error::Error for Error {}
/// Closed product result.
pub type Result<T> = std::result::Result<T, Error>;

/// Installation input deliberately excludes expected outputs and suite DTOs.
pub struct RunContext {
    /// Shared fixture/setup/scenario namespace.
    pub namespace: String,
}
/// Explicit consumer implementation and its aligned clock.
pub trait Installation {
    /// Concrete, Sized implementation.
    type Target: ConformanceTarget;
    /// Clock sharing the implementation's time authority.
    type Clock: Clock;
    /// Called only after full original-byte admission, from an explicit Run.
    fn create(context: &RunContext) -> Result<Installed<Self::Target, Self::Clock>>;
}
/// A target and clock installed together.
pub struct Installed<T, C> {
    /// Independent implementation.
    pub target: T,
    /// Matching wall and advancing execution-budget clock.
    pub clock: C,
    /// Existing Runner observation budget.
    pub config: RunnerConfig,
}
/// Actual completed execution; reports retain exact canonical bytes.
pub struct Completed {
    /// Exact admitted selected digest.
    pub digest: String,
    /// Run nonce, separate from declaration navigation.
    pub nonce: [u8; 16],
    /// UI generation captured before execution.
    pub generation: u32,
    /// Canonical `CountReport` produced from `ExecutedRun`.
    pub report: String,
    /// Canonical `CountRun` produced from that same `ExecutedRun`.
    pub run: String,
    /// Lossless sanitized report display, never raw target state.
    pub display: String,
}
/// Worker-local admission and execution state. Construction invokes no installation hooks.
pub struct Product<I> {
    loaded: Option<(u32, bundle::Loaded)>,
    next_handle: u32,
    nonces: BTreeSet<[u8; 16]>,
    installation: PhantomData<I>,
}
impl<I: Installation> Default for Product<I> {
    fn default() -> Self {
        Self::new()
    }
}
impl<I: Installation> Product<I> {
    /// Create an uninstalled, unloaded worker state.
    pub fn new() -> Self {
        Self {
            loaded: None,
            next_handle: 1,
            nonces: BTreeSet::new(),
            installation: PhantomData,
        }
    }
    /// Admit a complete immutable bundle; any failure invalidates previous authority.
    pub fn load(&mut self, manifest: &str, blobs: Vec<bundle::Blob>) -> Result<u32> {
        self.loaded = None;
        let loaded = bundle::Loaded::admit(manifest, blobs)?;
        self.replace(loaded)
    }
    fn replace(&mut self, loaded: bundle::Loaded) -> Result<u32> {
        let handle = self.next_handle;
        self.next_handle = handle.checked_add(1).ok_or(Error::ResourceLimit)?;
        self.loaded = Some((handle, loaded));
        Ok(handle)
    }
    /// Read a valid capability without invoking any target hooks.
    pub fn loaded(&self, handle: u32) -> Result<&bundle::Loaded> {
        self.loaded
            .as_ref()
            .filter(|(id, _)| *id == handle)
            .map(|(_, loaded)| loaded)
            .ok_or(Error::InvalidHandle)
    }
    /// Narrow only coverage authority, preserving all original parent bytes.
    pub fn select(&mut self, handle: u32, ids: &[crate::ScenarioId]) -> Result<u32> {
        let selected = self.loaded(handle)?.select(ids)?;
        self.replace(selected)
    }
    /// Release this worker's immutable input capability.
    pub fn release(&mut self, handle: u32) -> Result<()> {
        self.loaded(handle)?;
        self.loaded = None;
        Ok(())
    }
    /// Execute the real Runner, then derive both reports from its immutable result.
    pub fn run(&mut self, handle: u32, nonce: [u8; 16], generation: u32) -> Result<Completed> {
        self.loaded(handle)?;
        if nonce == [0; 16] || self.nonces.contains(&nonce) {
            return Err(Error::InvalidFrame);
        }
        if self.nonces.len() >= 65_536 {
            return Err(Error::ResourceLimit);
        }
        // Consume before installation, including a factory/execution failure. Never evict.
        self.nonces.insert(nonce);
        let loaded = self.loaded(handle)?;
        let suite = loaded.selected();
        let nonce_text = bundle::hex(&nonce);
        let namespace = format!("browser-{nonce_text}-{}", suite.digest());
        let Installed {
            target,
            clock,
            config,
        } = I::create(&RunContext {
            namespace: namespace.clone(),
        })?;
        let runner = Runner::new(config, clock, Ids::seeded(&namespace));
        let executed = runner.run_admitted(suite, &target);
        let report = CountReport::from_run(&executed, suite)
            .map_err(|_| Error::InternalFailure)?
            .to_canonical_json()
            .map_err(|_| Error::InternalFailure)?;
        let run = CountRun::from_run(&executed, suite)
            .map_err(|_| Error::InternalFailure)?
            .to_canonical_json()
            .map_err(|_| Error::InternalFailure)?;
        let display = presentation::report(&run)?;
        if report
            .len()
            .checked_add(run.len())
            .and_then(|n| n.checked_add(display.len()))
            .is_none_or(|n| n > MAX_FRAME - 1024)
        {
            return Err(Error::ResourceLimit);
        }
        Ok(Completed {
            digest: suite.digest().into(),
            nonce,
            generation,
            report,
            run,
            display,
        })
    }
}
