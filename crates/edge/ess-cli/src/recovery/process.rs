//! The admitted Helm artifact, its restricted invocation, and process uncertainty.
//!
//! Two things are separated here that are easy to confuse. The *authority* declares that a
//! reviewed artifact with an exact digest, version and protocol satisfies the profile's semantics,
//! including that a successful restricted call finishes its own mutation work before returning.
//! The *probes* below only detect incompatibility. A binary that prints suitable help and hashes
//! correctly but lacks those semantics is not admitted by anything this module can observe, and
//! hashing a file does not make it stable between the check and the execution — that rests on the
//! trusted account's immutable-installation contract.
//!
//! Uncertainty is conservative and asymmetric. `NotLaunched` is published only where the absence
//! of a launch is *established*; every started call with a nonzero exit, a timeout or a lost
//! acknowledgement is indeterminate, retains the claim and stops the invocation. Killing and
//! reaping the owned direct child does not establish that its descendants are gone or that every
//! API request it already issued has drained.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::time::{Duration, Instant};

use super::model::{
    Admitted, Digest, HelmBinary, HelmProtocol, ProcessDisposition, Refusal, RefusalCode, Text,
};
use super::{Host, Trust};

/// The bound on each probe output stream.
pub const PROBE_LIMIT: usize = 64 * 1024;
/// The bound on each probe's wall time.
pub const PROBE_TIMEOUT: Duration = Duration::from_secs(5);
/// The bound on each mutation's captured output stream.
pub const OUTPUT_LIMIT: usize = 256 * 1024;

fn unsupported(detail: impl Into<String>) -> Refusal {
    Refusal::new(RefusalCode::UnsupportedProfile, detail)
}

fn launch_failed(detail: impl Into<String>) -> Refusal {
    Refusal::new(RefusalCode::LaunchFailed, detail)
}

/// An admitted Helm artifact: hashed, probed and bound to its declared contract.
#[derive(Debug, Clone)]
pub struct AdmittedHelm {
    path: PathBuf,
    digest: Digest,
    version: String,
}

impl AdmittedHelm {
    /// The absolute admitted artifact. Every permitted operation uses this exact path.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// The digest of the whole executable as it was read.
    pub fn digest(&self) -> &Digest {
        &self.digest
    }

    /// The exact admitted version string.
    pub fn version(&self) -> &str {
        &self.version
    }
}

/// Hashes and admits the declared artifact, then probes it for compatibility.
///
/// The installation rules are checked before the file is read: a wrapper, a symbolic link, a
/// nonregular file, an executor-, group- or other-writable file, a setuid or setgid bit and a
/// writable parent component each refuse. The filename digest, the configured digest and the
/// complete hash of the bytes must all agree.
pub fn admit(host: &dyn Host, binary: &HelmBinary, prefix: &str) -> Admitted<AdmittedHelm> {
    binary.admitted_under(prefix)?;
    if binary.protocol != HelmProtocol::Helm3Recovery1 {
        return Err(unsupported("unsupported Helm protocol"));
    }
    let path = PathBuf::from(binary.path.as_str());
    let facts = super::journal::under(
        RefusalCode::UnsupportedProfile,
        super::journal::admit_path(host, &path, Trust::Administrative),
    )?;
    if !facts.regular {
        return Err(unsupported(format!(
            "{} is not a regular file",
            path.display()
        )));
    }
    if facts.mode & 0o022 != 0 {
        return Err(unsupported(format!(
            "{} is writable by group or other",
            path.display()
        )));
    }
    if facts.mode & 0o6000 != 0 {
        return Err(unsupported(format!(
            "{} is setuid or setgid",
            path.display()
        )));
    }
    if facts.uid == host.executor_uid() {
        return Err(unsupported(format!(
            "{} is owned by the executor and is not an immutable installation",
            path.display()
        )));
    }
    let bytes = std::fs::read(&path)
        .map_err(|error| unsupported(format!("{} is unreadable: {error}", path.display())))?;
    let digest = Digest::of_bytes(&bytes);
    if digest != binary.digest {
        return Err(unsupported(format!(
            "{} does not hash to its configured digest",
            path.display()
        )));
    }
    let admitted = AdmittedHelm {
        path,
        digest,
        version: binary.version.to_string(),
    };
    probe(&admitted)?;
    Ok(admitted)
}

/// The exact flags each permitted operation's help output must offer.
const REQUIRED_FLAGS: &[(&str, &[&str])] = &[
    (
        "upgrade",
        &[
            "--install",
            "--atomic",
            "--wait",
            "--timeout",
            "--description",
            "--no-hooks",
            "--skip-crds",
            "--values",
            "--namespace",
            "--kubeconfig",
            "--kube-context",
        ],
    ),
    (
        "uninstall",
        &[
            "--no-hooks",
            "--wait",
            "--timeout",
            "--namespace",
            "--kubeconfig",
            "--kube-context",
        ],
    ),
    (
        "template",
        &["--values", "--namespace", "--no-hooks", "--skip-crds"],
    ),
    ("status", &["--output", "--namespace"]),
    ("get", &["--revision", "--namespace"]),
];

/// Runs the bounded read-only compatibility probes.
fn probe(helm: &AdmittedHelm) -> Admitted<()> {
    let version = run_bounded(
        helm,
        &["version", "--template", "{{.Version}}"],
        PROBE_LIMIT,
        PROBE_TIMEOUT,
    )?;
    if !version.launched || version.status != Some(0) {
        return Err(unsupported("the version probe did not return successfully"));
    }
    let printed = String::from_utf8(version.stdout)
        .map_err(|_| unsupported("the version probe printed non-UTF-8"))?;
    let trimmed = printed.strip_suffix('\n').unwrap_or(&printed);
    if trimmed != helm.version || printed.trim_end_matches('\n').len() + 1 < printed.len() {
        return Err(unsupported(format!(
            "the artifact reports version {trimmed:?} and the authority declares {:?}",
            helm.version
        )));
    }
    for (operation, flags) in REQUIRED_FLAGS {
        let offered = run_bounded(helm, &[operation, "--help"], PROBE_LIMIT, PROBE_TIMEOUT)?;
        if !offered.launched || offered.status != Some(0) {
            return Err(unsupported(format!(
                "the {operation} help probe did not return successfully"
            )));
        }
        let text = String::from_utf8(offered.stdout)
            .map_err(|_| unsupported(format!("the {operation} help probe printed non-UTF-8")))?;
        let names: Vec<&str> = text
            .split(|c: char| !(c.is_ascii_alphanumeric() || c == '-'))
            .filter(|token| token.starts_with("--"))
            .collect();
        for flag in *flags {
            if !names.contains(flag) {
                return Err(unsupported(format!(
                    "the artifact's {operation} help offers no {flag}"
                )));
            }
        }
    }
    Ok(())
}

/// The explicitly constructed child environment.
///
/// Nothing is inherited. Every Helm, kubeconfig, proxy and executable-injection setting is
/// cleared, the storage driver is pinned to Secret-backed release storage, and the configuration,
/// cache, data and plugin locations are private to this invocation.
pub fn environment(private: &Path, kubeconfig: &Text) -> BTreeMap<String, String> {
    BTreeMap::from([
        ("HELM_DRIVER".to_owned(), "secret".to_owned()),
        (
            "HELM_CONFIG_HOME".to_owned(),
            private.join("config").display().to_string(),
        ),
        (
            "HELM_CACHE_HOME".to_owned(),
            private.join("cache").display().to_string(),
        ),
        (
            "HELM_DATA_HOME".to_owned(),
            private.join("data").display().to_string(),
        ),
        (
            "HELM_PLUGINS".to_owned(),
            private.join("plugins").display().to_string(),
        ),
        ("KUBECONFIG".to_owned(), kubeconfig.to_string()),
        ("PATH".to_owned(), "/nonexistent".to_owned()),
    ])
}

/// What one bounded child invocation established.
#[derive(Debug, Clone)]
pub struct Outcome {
    /// Whether the child was launched at all.
    pub launched: bool,
    /// The exit code, when the child was reaped with one.
    pub status: Option<i32>,
    /// Whether the bound elapsed before the child settled.
    pub timed_out: bool,
    /// Bounded captured stdout.
    pub stdout: Vec<u8>,
    /// Bounded captured stderr.
    pub stderr: Vec<u8>,
}

impl Outcome {
    /// The conservative disposition this outcome establishes.
    ///
    /// Only a definite spawn refusal is `NotLaunched`. A nonzero exit, a timeout and a lost
    /// acknowledgement are all `Indeterminate` — the child may have run, and a message from it is
    /// not independent evidence about the target.
    pub fn disposition(&self) -> ProcessDisposition {
        if !self.launched {
            ProcessDisposition::NotLaunched
        } else if self.status == Some(0) && !self.timed_out {
            ProcessDisposition::Acknowledged
        } else {
            ProcessDisposition::Indeterminate
        }
    }
}

fn drain(mut stream: impl std::io::Read + Send + 'static, limit: usize) -> mpsc::Receiver<Vec<u8>> {
    let (sender, receiver) = mpsc::channel();
    std::thread::spawn(move || {
        let mut retained = Vec::new();
        let mut chunk = [0u8; 8192];
        while let Ok(read) = stream.read(&mut chunk) {
            if read == 0 {
                break;
            }
            let keep = read.min(limit.saturating_sub(retained.len()));
            retained.extend_from_slice(&chunk[..keep]);
        }
        let _ = sender.send(retained);
    });
    receiver
}

fn run_bounded(
    helm: &AdmittedHelm,
    arguments: &[&str],
    limit: usize,
    bound: Duration,
) -> Admitted<Outcome> {
    let mut command = Command::new(helm.path());
    command.args(arguments);
    command.env_clear();
    command.env("HELM_DRIVER", "secret");
    run(command, limit, bound)
}

/// Launches one bounded child and classifies what it establishes.
///
/// The timeout path kills and reaps the owned direct child. That is all it does, and all it is
/// reported as doing: it does not claim to have terminated a descendant or drained a request the
/// child already issued, which is why the disposition it produces is indeterminate.
pub fn run(mut command: Command, limit: usize, bound: Duration) -> Admitted<Outcome> {
    let deadline = Instant::now() + bound;
    let mut child = match command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(child) => child,
        Err(error) => {
            return Ok(Outcome {
                launched: false,
                status: None,
                timed_out: false,
                stdout: Vec::new(),
                stderr: format!("{error}").into_bytes(),
            })
        }
    };
    let stdout = drain(
        child
            .stdout
            .take()
            .ok_or_else(|| launch_failed("the child's stdout pipe is missing"))?,
        limit,
    );
    let stderr = drain(
        child
            .stderr
            .take()
            .ok_or_else(|| launch_failed("the child's stderr pipe is missing"))?,
        limit,
    );
    loop {
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            return Ok(Outcome {
                launched: true,
                status: None,
                timed_out: true,
                stdout: stdout
                    .recv_timeout(Duration::from_secs(1))
                    .unwrap_or_default(),
                stderr: stderr
                    .recv_timeout(Duration::from_secs(1))
                    .unwrap_or_default(),
            });
        }
        match child.try_wait() {
            Ok(Some(status)) => {
                return Ok(Outcome {
                    launched: true,
                    status: status.code(),
                    timed_out: false,
                    stdout: stdout
                        .recv_timeout(Duration::from_secs(5))
                        .unwrap_or_default(),
                    stderr: stderr
                        .recv_timeout(Duration::from_secs(5))
                        .unwrap_or_default(),
                })
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(5)),
            Err(error) => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(Refusal::new(
                    RefusalCode::EffectIndeterminate,
                    format!("the owned child could not be waited for: {error}"),
                ));
            }
        }
    }
}

/// Everything one apply's argument vector is built from.
///
/// A struct rather than eight positional parameters: `namespace`, `kubeconfig`, `context` and
/// `marker` are all strings, and a caller that transposed two of them would build a command that
/// runs against the wrong place and still compiles.
#[derive(Debug, Clone, Copy)]
pub struct Apply<'a> {
    /// The Helm release name.
    pub release: &'a str,
    /// The verified private chart snapshot.
    pub chart: &'a Path,
    /// The private values document.
    pub values: &'a Path,
    /// The pinned namespace.
    pub namespace: &'a str,
    /// The protected kubeconfig.
    pub kubeconfig: &'a str,
    /// The admitted context alias.
    pub context: &'a str,
    /// The exact ownership marker.
    pub marker: &'a str,
    /// The Helm wait timeout.
    pub timeout: &'a str,
}

/// The exact restricted argument vector for one apply.
///
/// `--create-namespace` is absent because the admitted namespace must already exist with its
/// pinned UID; `--no-hooks` and `--skip-crds` are present because the profile admits neither; and
/// the description carries the exact ownership marker on every managed apply.
pub fn apply_arguments(apply: Apply<'_>) -> Vec<String> {
    vec![
        "upgrade".to_owned(),
        "--install".to_owned(),
        apply.release.to_owned(),
        apply.chart.display().to_string(),
        "--namespace".to_owned(),
        apply.namespace.to_owned(),
        "--kubeconfig".to_owned(),
        apply.kubeconfig.to_owned(),
        "--kube-context".to_owned(),
        apply.context.to_owned(),
        "--values".to_owned(),
        apply.values.display().to_string(),
        "--description".to_owned(),
        apply.marker.to_owned(),
        "--no-hooks".to_owned(),
        "--skip-crds".to_owned(),
        "--atomic".to_owned(),
        "--wait".to_owned(),
        "--timeout".to_owned(),
        apply.timeout.to_owned(),
    ]
}

/// The exact restricted argument vector for one removal.
///
/// `--keep-history` is absent because this contract's absence predicate includes Helm release
/// storage, and no `--ignore-not-found` appears anywhere: absence is established by an
/// authenticated read, never by a tolerated error.
pub fn remove_arguments(
    release: &str,
    namespace: &str,
    kubeconfig: &str,
    context: &str,
    timeout: &str,
) -> Vec<String> {
    vec![
        "uninstall".to_owned(),
        release.to_owned(),
        "--namespace".to_owned(),
        namespace.to_owned(),
        "--kubeconfig".to_owned(),
        kubeconfig.to_owned(),
        "--kube-context".to_owned(),
        context.to_owned(),
        "--no-hooks".to_owned(),
        "--wait".to_owned(),
        "--timeout".to_owned(),
        timeout.to_owned(),
    ]
}

/// The flags this profile never passes, in any operation.
///
/// Each is here because it would convert an unproven claim into an apparently settled one:
/// `--create-namespace` creates the namespace whose UID is the pinned identity,
/// `--keep-history` leaves the release storage the absence predicate covers, and
/// `--ignore-not-found` turns a failed read into an absence.
pub const REFUSED_FLAGS: &[&str] = &[
    "--create-namespace",
    "--keep-history",
    "--ignore-not-found",
    "--post-renderer",
    "--dependency-update",
];

/// Refuses an argument vector that carries any flag outside the closed contract.
pub fn admit_arguments(arguments: &[String]) -> Admitted<()> {
    for argument in arguments {
        if REFUSED_FLAGS.contains(&argument.as_str()) {
            return Err(unsupported(format!(
                "{argument} is never passed by this profile"
            )));
        }
    }
    Ok(())
}
