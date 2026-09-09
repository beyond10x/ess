//! The test-only executable entry into the shared production recovery code.
//!
//! This binary is the offline qualification's "real independent Rust driver process". It links the
//! same library the shipped `ess` binary links and runs the same store admission, reservation,
//! claim publication and journal code; what it supplies instead of the production host is the
//! injected [`fake_recovery::FixtureHost`], which reports the administrative metadata an
//! unprivileged fixture cannot really have and which fails or interrupts at exactly one named
//! durability barrier.
//!
//! It is a separate process on purpose, and the separation is the evidence. A claim retained after
//! its holder died, a store read by a second executor that was not there when the first one wrote
//! it, a restart that sees only what was actually published — none of those is observable inside
//! one test process, and a fixture that simulated them would be testing its own simulation.
//!
//! What it is not: a bypass. Nothing here is reachable from the shipped `ess` binary, no flag or
//! environment variable in `ess` selects any of it, and injected administrative ownership is not
//! proof that root-owned files were provisioned.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use ess_cli::recovery::journal::{open_store, publish_claim, read_claim, reserve, Journal};
use ess_cli::recovery::model::{
    read_canonical, Index, JournalFact, LockClaim, LockFormat, RegistryRef, Uuid,
};
use ess_cli::recovery::{Barrier, Host};

#[path = "fake_recovery.rs"]
mod fake_recovery;

/// What one driver run was asked to do.
#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Job {
    /// The fixture scratch root.
    root: PathBuf,
    /// What this run does: `claim`, `observe` or `release`.
    mode: String,
    /// The nonces the injected host will issue, in order.
    nonces: Vec<String>,
    /// A durability barrier to fail, as its declared name.
    fail: Option<String>,
    /// A durability barrier to be interrupted at, as its declared name.
    interrupt: Option<String>,
    /// Whether an `Opened` fact is published after the claim.
    open_journal: bool,
}

fn barrier(name: &str) -> Option<Barrier> {
    ess_cli::recovery::BARRIERS
        .iter()
        .copied()
        .find(|candidate| candidate.name() == name)
}

fn main() -> ExitCode {
    let Some(path) = std::env::args().nth(1) else {
        eprintln!("usage: ess-recovery-driver <job.json>");
        return ExitCode::FAILURE;
    };
    let text = match std::fs::read_to_string(&path) {
        Ok(text) => text,
        Err(error) => {
            eprintln!("the job description is unreadable: {error}");
            return ExitCode::FAILURE;
        }
    };
    let job: Job = match serde_json::from_str(&text) {
        Ok(job) => job,
        Err(error) => {
            eprintln!("the job description is not admissible: {error}");
            return ExitCode::FAILURE;
        }
    };
    match run(&job) {
        Ok(message) => {
            println!("{message}");
            ExitCode::SUCCESS
        }
        Err(message) => {
            eprintln!("{message}");
            ExitCode::FAILURE
        }
    }
}

fn run(job: &Job) -> Result<String, String> {
    let nonces: Vec<Uuid> = job
        .nonces
        .iter()
        .map(|nonce| Uuid::new(nonce.clone()).map_err(|refusal| refusal.detail))
        .collect::<Result<_, String>>()?;
    let mut host = fake_recovery::FixtureHost::new(&job.root, nonces);
    if let Some(name) = &job.fail {
        host = host.failing(
            barrier(name).ok_or_else(|| format!("no barrier is named {name}"))?,
            None,
        );
    }
    if let Some(name) = &job.interrupt {
        host = host.interrupting(
            barrier(name).ok_or_else(|| format!("no barrier is named {name}"))?,
            None,
        );
    }
    let state_root = job.root.join(fake_recovery::STATE);
    let store = open_store(&host, &state_root).map_err(|refusal| refusal.to_string())?;

    match job.mode.as_str() {
        // Read only. This is what a second executor may always do while somebody else's claim is
        // retained: report what it can see, and mutate nothing.
        "observe" => {
            let retained = read_claim(&host, &store).map_err(|refusal| refusal.to_string())?;
            Ok(match retained {
                Some(retained) => format!(
                    "observation-only: invocation {} retains the target claim",
                    retained.claim.invocation.nonce
                ),
                None => "observation-only: no claim is retained".to_owned(),
            })
        }
        // Reserve, publish a claim, optionally open a journal, and then return holding the claim.
        // The process exits with the claim published: nothing removes it, because nothing here has
        // completed safely.
        "claim" => {
            if let Some(retained) = read_claim(&host, &store).map_err(|r| r.to_string())? {
                return Err(format!(
                    "the target claim is retained by invocation {}; this executor never reclaims \
                     another invocation's claim",
                    retained.claim.invocation.nonce
                ));
            }
            let reservation = reserve(&host, &store).map_err(|refusal| refusal.to_string())?;
            let claim = claim_for(&host, &job.root, &reservation)?;
            publish_claim(&host, &store, &claim).map_err(|refusal| refusal.to_string())?;
            if job.open_journal {
                let mut record = Journal::open(&reservation);
                let context = context_for(&job.root)?;
                record
                    .append(&host, JournalFact::Opened(Box::new(context)))
                    .map_err(|refusal| refusal.to_string())?;
            }
            Ok(format!(
                "claimed: invocation {} holds the target claim",
                reservation.id().nonce
            ))
        }
        other => Err(format!("no driver mode is named {other}")),
    }
}

fn claim_for(
    host: &dyn Host,
    root: &Path,
    reservation: &ess_cli::recovery::journal::Reservation,
) -> Result<LockClaim, String> {
    let _ = host;
    let context = context_for(root)?;
    Ok(LockClaim {
        format: LockFormat::V1,
        invocation: reservation.id().clone(),
        authority_id: context.authority.authority_id.clone(),
        authority_revision: context.authority.revision,
        target: context.authority.target.clone(),
        registry: context.registry,
    })
}

/// Reads the invocation context the fixture published for this run.
///
/// A file rather than a command-line blob: the context embeds a whole authority revision, and the
/// point of the driver is that it reads the same canonical bytes the engine would.
fn context_for(root: &Path) -> Result<ess_cli::recovery::model::InvocationContext, String> {
    let path = root.join("driver-context.json");
    let text = std::fs::read_to_string(&path)
        .map_err(|error| format!("{} is unreadable: {error}", path.display()))?;
    read_canonical(&text).map_err(|refusal| refusal.to_string())
}

/// Keeps the imports this module needs visible to the compiler in every configuration.
#[allow(dead_code)]
fn unused_marker(index: Index, reference: RegistryRef) -> (Index, RegistryRef) {
    (index, reference)
}
