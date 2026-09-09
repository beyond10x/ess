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
    /// The label to fail or interrupt at, or `None` for the first crossing.
    #[serde(default)]
    label: Option<String>,
    /// How far the injected monotonic clock advances per reading.
    #[serde(default)]
    step_ms: Option<u64>,
    /// Whether an `Opened` fact is published after the claim.
    open_journal: bool,
    /// The desired `ess-deployment/1` document, for the cache lane.
    #[serde(default)]
    plan: Option<PathBuf>,
    /// The digest-pinned chart cache root, for the cache lane.
    #[serde(default)]
    cache: Option<PathBuf>,
    /// The authority to select, for the full-engine mode.
    #[serde(default)]
    authority_id: Option<String>,
    /// Which operation index fails, and how, for the full-engine mode.
    #[serde(default)]
    faults: Vec<(usize, String)>,
    /// Whether to provision and admit a synthetic authority before touching the cache.
    ///
    /// The **control**. With it false, the same vector must stop at the authority refusal without
    /// an ORAS call, a cache write or a Helm invocation — which is what proves the vectors with it
    /// true are reaching the cache boundary rather than passing on an earlier refusal.
    #[serde(default)]
    authority: bool,
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
    // The cache lane has no store of its own: it admits an authority and acquires a proof, and a
    // store it never reads must not be a precondition for either.
    if job.mode == "cache" {
        return cache_lane(job);
    }
    if job.mode == "engine" {
        return engine_lane(job);
    }
    let nonces: Vec<Uuid> = job
        .nonces
        .iter()
        .map(|nonce| Uuid::new(nonce.clone()).map_err(|refusal| refusal.detail))
        .collect::<Result<_, String>>()?;
    let mut host = fake_recovery::FixtureHost::new(&job.root, nonces);
    if let Some(name) = &job.fail {
        host = host.failing(
            barrier(name).ok_or_else(|| format!("no barrier is named {name}"))?,
            job.label.as_deref(),
        );
    }
    if let Some(name) = &job.interrupt {
        host = host.interrupting(
            barrier(name).ok_or_else(|| format!("no barrier is named {name}"))?,
            job.label.as_deref(),
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

/// Runs the complete production engine, in this process, against the arrangement at `root`.
///
/// This is the offline qualification's real independent driver: the same admission, journal,
/// preparation, observation, predicate and process code the shipped binary runs, in a process of
/// its own, against a synthetic target that outlives it. What the job description supplies is the
/// injected host and the process outcomes — through the Rust seams — and nothing else.
///
/// It exits nonzero on anything short of a complete accounting, so a caller can tell a settled run
/// from an unresolved one without parsing prose.
fn engine_lane(job: &Job) -> Result<String, String> {
    let root = job.root.clone();
    let authority = job
        .authority_id
        .as_ref()
        .ok_or_else(|| "the engine lane needs an authority".to_owned())?;
    let authority = Uuid::new(authority.clone()).map_err(|refusal| refusal.detail)?;

    let desired_bytes = std::fs::read_to_string(root.join("desired.json"))
        .map_err(|error| format!("reading the desired document: {error}"))?;
    let current_bytes = std::fs::read_to_string(root.join("current.json")).ok();
    let documents = ess_cli::recovery::Documents {
        desired: serde_json::from_str(&desired_bytes)
            .map_err(|error| format!("the desired document: {error}"))?,
        current: current_bytes
            .as_deref()
            .map(serde_json::from_str)
            .transpose()
            .map_err(|error| format!("the baseline document: {error}"))?,
        desired_bytes,
        current_bytes,
    };

    let nonces: Vec<Uuid> = job
        .nonces
        .iter()
        .map(|nonce| Uuid::new(nonce.clone()).map_err(|refusal| refusal.detail))
        .collect::<Result<_, String>>()?;
    let mut host = fake_recovery::FixtureHost::new(&root, nonces);
    if let Some(name) = &job.fail {
        host = host.failing(
            barrier(name).ok_or_else(|| format!("no barrier is named {name}"))?,
            job.label.as_deref(),
        );
    }
    if let Some(name) = &job.interrupt {
        host = host.interrupting(
            barrier(name).ok_or_else(|| format!("no barrier is named {name}"))?,
            job.label.as_deref(),
        );
    }
    if let Some(step) = job.step_ms {
        host = host.stepping(step);
    }

    let cluster = fake_recovery::Cluster::at(&root);
    let payload = std::fs::read(root.join("chart.tgz"))
        .map_err(|error| format!("the proved chart payload: {error}"))?;
    let mut platform = fake_recovery::FixturePlatform::new(cluster, payload);
    for (index, name) in &job.faults {
        let fault = fake_recovery::HelmFault::parse(name)
            .ok_or_else(|| format!("no Helm fault is named {name}"))?;
        platform = platform.failing_at(*index, fault);
    }

    let request = ess_cli::recovery::ReconcileRequest {
        path: root.join("desired.json"),
        current: None,
        cache: root.join("cache"),
        allow_removals: true,
        dry_run: false,
        timeout: "5m".to_owned(),
        authority: Some(authority),
        retry_of: None,
    };
    let roots = ess_cli::recovery::Roots {
        registry: root.join(fake_recovery::ADMIN),
        helm_prefix: format!("{}/", root.join("opt/ess/recovery-tools/helm").display()),
    };
    let report = ess_cli::recovery::execute(&host, &roots, &request, &documents, &platform);
    let calls = platform.calls().join(",");
    let rendered = format!("{}calls: {calls}\n", report.render());
    if report.complete() {
        Ok(rendered)
    } else {
        Err(rendered)
    }
}

/// Admits an authority, then acquires and consumes each affected release's proved chart payload.
///
/// It admits a synthetic authority through the *production* registry scan and then acquires the
/// pinned chart payload through the *production* OCI proof, so the original-byte transport
/// assertions keep their exact meaning. It deliberately stops there: C12 keeps opaque historical
/// chart fixtures as cache-layer vectors and forbids relabelling them as admissible generated
/// recovery charts, so this lane hands the proved payload to the fixture's own consumer instead of
/// rendering and comparing a projection.
fn cache_lane(job: &Job) -> Result<String, String> {
    let plan = job
        .plan
        .as_ref()
        .ok_or_else(|| "the cache lane needs a desired document".to_owned())?;
    let cache = job
        .cache
        .as_ref()
        .ok_or_else(|| "the cache lane needs a cache root".to_owned())?;
    let bytes = std::fs::read_to_string(plan)
        .map_err(|error| format!("reading {}: {error}", plan.display()))?;
    // Whole-input validation first, before the authority and before anything external.
    let desired: ess_deployment::DeploymentIr = if plan
        .extension()
        .is_some_and(|extension| extension == "json")
    {
        serde_json::from_str(&bytes)
            .map_err(|error| format!("parsing {} as JSON: {error}", plan.display()))?
    } else {
        serde_yaml::from_str(&bytes)
            .map_err(|error| format!("parsing {} as YAML: {error}", plan.display()))?
    };
    desired
        .validate()
        .map_err(|diagnostics| format!("validating desired deployment: {diagnostics:?}"))?;

    let services: Vec<(String, String, String)> = desired
        .rollout_order
        .iter()
        .map(|service| {
            let release = desired
                .releases
                .get(service)
                .expect("rollout order refers to a release");
            (
                service.to_string(),
                release.namespace.clone(),
                release.release_name.clone(),
            )
        })
        .collect();

    if !job.authority {
        return Err(
            "normal execution requires a caller-provisioned authority selected from the protected              registry; this run supplied none"
                .to_owned(),
        );
    }
    // Each driver provisions its own synthetic authority. Concurrent cache writers must share
    // the OCI cache, but must not truncate one another's registry during authority admission.
    // A driver handles one job per process, so the PID separates all simultaneously live writers.
    let arrangement = job.root.join(format!("recovery-{}", std::process::id()));
    let id = fake_recovery::provision_cache_lane(&arrangement, &bytes, &services)
        .map_err(|error| format!("the fixture arrangement could not be laid out: {error}"))?;
    let host = fake_recovery::FixtureHost::new(&arrangement, vec![uuid_of(0x40)]);
    let admin = arrangement.join(fake_recovery::ADMIN);
    let (_, authority) = ess_cli::recovery::authority::admit(&host, &admin, &id)
        .map_err(|refusal| refusal.to_string())?;
    ess_cli::recovery::admit_documents(&authority, &bytes, None)
        .map_err(|refusal| refusal.to_string())?;

    for (service, _, _) in &services {
        let identifier =
            ess_deployment::Identifier::new(service).map_err(|error| format!("{error}"))?;
        let release = desired
            .releases
            .get(&identifier)
            .ok_or_else(|| format!("no release for {service}"))?;
        let permit = authority
            .permit(service)
            .ok_or_else(|| format!("the authority admits no permit for {service}"))?;
        consume(release, permit, &authority, cache, &arrangement)?;
    }
    Ok(format!(
        "cache lane: {} release(s) acquired and consumed under authority {id}",
        services.len()
    ))
}

/// Acquires one release's proved payload and hands it to the fixture's consumer.
fn consume(
    release: &ess_deployment::DeploymentRelease,
    permit: &ess_cli::recovery::model::ReleasePermit,
    authority: &ess_cli::recovery::model::Authority,
    cache: &Path,
    arrangement: &Path,
) -> Result<(), String> {
    if release.chart.kind != ess_deployment::ArtifactKind::HelmChart {
        return Err(format!("{} does not select a Helm chart", release.service));
    }
    let reference = format!(
        "{}@{}",
        release.chart.reference.trim_start_matches("oci://"),
        release.chart.digest
    );
    // The production OCI proof, unchanged: the same original-byte manifest and blob verification,
    // the same cache layout, the same ORAS argument vectors.
    let payload = ess_cli::oci_cache::payload(&reference, cache, ess_cli::oci_cache::Profile::Helm)
        .map_err(|error| format!("{error:#}"))?;

    // The same transient private directory the production path uses, and for the same reason: the
    // verified snapshot exists for exactly the executor call and is gone afterwards, so nothing
    // downstream can read a chart that was never re-proved.
    let _ = arrangement;
    let private = ess_cli::TemporaryDirectory::create("ess-recovery-cache-lane")
        .map_err(|error| format!("{error}"))?;
    let private = private;
    let chart = private.path().join("chart.tgz");
    std::fs::write(&chart, &payload).map_err(|error| format!("{error}"))?;
    let values = private.path().join("values.yaml");
    let document = ess_cli::recovery::values_document(release).map_err(|r| r.to_string())?;
    std::fs::write(&values, document).map_err(|error| format!("{error}"))?;

    // The recovery lane's own argument vector, built by production code. `--create-namespace`,
    // `--keep-history` and `--ignore-not-found` are absent because this is where they would have
    // to appear, and `admit_arguments` refuses them here as well as everywhere else.
    let marker =
        ess_cli::recovery::model::ownership_marker(&authority.authority_id, &permit.incarnation);
    let arguments =
        ess_cli::recovery::process::apply_arguments(ess_cli::recovery::process::Apply {
            release: permit.release_name.as_str(),
            chart: &chart,
            values: &values,
            namespace: permit.namespace.name.as_str(),
            kubeconfig: authority.host.kubeconfig.as_str(),
            context: "fixture",
            marker: &marker,
            timeout: "5m",
        });
    ess_cli::recovery::process::admit_arguments(&arguments).map_err(|r| r.to_string())?;
    // The consumer is the fixture's own PATH executable, inheriting this process's environment so
    // its control files stay reachable. It is not the admitted artifact and is not treated as one:
    // executable admission is decided by its own family.
    let status = std::process::Command::new("helm")
        .args(&arguments)
        .status()
        .map_err(|error| format!("starting the chart consumer: {error}"))?;
    if !status.success() {
        return Err(format!("the chart consumer failed with {status}"));
    }
    Ok(())
}

fn uuid_of(byte: u8) -> Uuid {
    let hex = format!("{byte:02x}").repeat(16);
    Uuid::new(format!(
        "{}-{}-{}-{}-{}",
        &hex[0..8],
        &hex[8..12],
        &hex[12..16],
        &hex[16..20],
        &hex[20..32]
    ))
    .expect("a fixture UUID admits")
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
