//! The injected host, the fixture arrangement, and the stateful child the matrix runs against.
//!
//! This file has two roles, and keeping them in one place is deliberate: the fixture that *builds*
//! a protected arrangement and the executable that *acts* on it have to agree byte for byte about
//! where everything is, and a second copy of those paths is a second thing to get wrong.
//!
//! As a **library** (included by `tests/execution_recovery.rs` and by
//! `tests/support/recovery_driver.rs`) it supplies [`FixtureHost`] — the `Host` implementation that
//! injects the administrative metadata an ordinary unprivileged directory cannot have, and fails
//! exactly one named durability barrier — together with the builders that lay out a protected
//! registry, store and tool tree under one scratch root.
//!
//! As a **binary** it is the stand-in for the admitted Helm artifact *and* the synthetic target.
//! The target's state is a directory of its own: it outlives the driver process that mutated it,
//! it is never reconstructed from the executor's journal, and that independence is what makes
//! "the target changed and the run did not record it" a real state rather than an assertion.
//!
//! What is injected is injected, and this comment is the label. Positive administrative ownership
//! under an unprivileged fixture exercises the decision algorithm; it is not proof that root-owned
//! files were deployed. A simulated loss of an unsynchronized publication proves the protocol's
//! ordering and restart response, not the storage hardware's power-loss behavior. Output that this
//! executable makes compatible with Helm's is not proof of stock Helm's semantics.

#![allow(dead_code)]
// A fixture builder chain reads better than a struct literal here, and `must_use` on each link
// would say nothing a reader does not already see. Everything else in the workspace's lint set
// stands.
#![allow(clippy::must_use_candidate, clippy::return_self_not_must_use)]

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use ess_cli::recovery::model::{invalid, Admitted, Refusal, RefusalCode, Uuid};
use ess_cli::recovery::{Barrier, FileFacts, Host};

/// Where a fixture's protected administrative tree lives under its scratch root.
pub const ADMIN: &str = "etc/ess/recovery";
/// Where a fixture's admitted tool installation lives under its scratch root.
pub const TOOLS: &str = "opt/ess/recovery-tools/helm";
/// Where a fixture's executor-owned state root lives under its scratch root.
pub const STATE: &str = "var/lib/ess/recovery";
/// Where the independently controlled synthetic target's state lives under its scratch root.
pub const TARGET: &str = "target-state";

/// Which barrier a driver fails, and at which label.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fault {
    /// The barrier to fail.
    pub barrier: Barrier,
    /// The label to fail it at, or `None` for the first crossing of that barrier.
    pub label: Option<String>,
    /// Whether to terminate the process rather than refuse, simulating interruption.
    pub interrupt: bool,
}

/// The injected host: real filesystem, administrative metadata, one faultable barrier.
#[derive(Debug)]
pub struct FixtureHost {
    root: PathBuf,
    executor_uid: u32,
    host_id: String,
    nonces: Vec<Uuid>,
    issued: AtomicU64,
    clock: AtomicU64,
    step_ms: u64,
    fault: Option<Fault>,
    overrides: Vec<(PathBuf, u32, u32)>,
    crossed: std::sync::Mutex<Vec<(Barrier, String)>>,
}

impl FixtureHost {
    /// A host over one scratch root, with deterministic nonces and no injected fault.
    pub fn new(root: impl Into<PathBuf>, nonces: Vec<Uuid>) -> Self {
        Self {
            root: root.into(),
            executor_uid: rustix::process::getuid().as_raw(),
            host_id: "fixture-host".to_owned(),
            nonces,
            issued: AtomicU64::new(0),
            clock: AtomicU64::new(0),
            step_ms: 1,
            fault: None,
            overrides: Vec::new(),
            crossed: std::sync::Mutex::new(Vec::new()),
        }
    }

    /// The same host reporting an exact owner and mode for one path.
    ///
    /// The injection above makes an unprivileged fixture look administratively provisioned, which
    /// is what the positive cases need. The negative cases need the opposite — an artifact that
    /// really is executor-owned, group-writable or setuid — and this is how they say so, one path
    /// at a time, instead of turning the whole injection off and refusing for a different reason.
    pub fn with_metadata(mut self, path: impl Into<PathBuf>, uid: u32, mode: u32) -> Self {
        self.overrides.push((path.into(), uid, mode));
        self
    }

    /// The same host with one named durability barrier failing.
    pub fn failing(mut self, barrier: Barrier, label: Option<&str>) -> Self {
        self.fault = Some(Fault {
            barrier,
            label: label.map(str::to_owned),
            interrupt: false,
        });
        self
    }

    /// The same host terminating the process at one named barrier, simulating interruption.
    pub fn interrupting(mut self, barrier: Barrier, label: Option<&str>) -> Self {
        self.fault = Some(Fault {
            barrier,
            label: label.map(str::to_owned),
            interrupt: true,
        });
        self
    }

    /// The same host with a monotonic clock advancing by `step_ms` per reading.
    pub fn stepping(mut self, step_ms: u64) -> Self {
        self.step_ms = step_ms;
        self
    }

    /// The same host reporting a different provisioned host identity.
    pub fn named(mut self, host_id: &str) -> Self {
        host_id.clone_into(&mut self.host_id);
        self
    }

    /// The barriers this host has crossed, in order.
    pub fn crossed(&self) -> Vec<(Barrier, String)> {
        self.crossed
            .lock()
            .expect("the barrier log is not poisoned")
            .clone()
    }

    /// The scratch root.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// The protected administrative root.
    pub fn admin(&self) -> PathBuf {
        self.root.join(ADMIN)
    }

    /// The admitted tool installation prefix, with its trailing separator.
    pub fn tools_prefix(&self) -> String {
        format!("{}/", self.root.join(TOOLS).display())
    }

    /// The executor-owned state root.
    pub fn state_root(&self) -> PathBuf {
        self.root.join(STATE)
    }

    /// The independently controlled synthetic target's state directory.
    pub fn target(&self) -> PathBuf {
        self.root.join(TARGET)
    }

    /// Whether this path is part of the trusted control account's tree.
    ///
    /// Both directions matter. A file *under* the administrative root is root-owned, and so is
    /// every *ancestor* of it up to the scratch root — `admit_path` walks the whole chain, and a
    /// component owned by the executor is precisely the writable parent the production rule
    /// refuses. Getting only the first direction right made every registry case refuse at
    /// `<root>/etc` before it reached the thing it was about.
    fn administrative(&self, path: &Path) -> bool {
        let admin = self.admin();
        let tools = self.root.join(TOOLS);
        path.starts_with(&admin)
            || path.starts_with(&tools)
            || admin.starts_with(path)
            || tools.starts_with(path)
    }
}

impl Host for FixtureHost {
    fn now_ms(&self) -> u64 {
        self.clock.fetch_add(self.step_ms, Ordering::Relaxed)
    }

    fn random_nonce(&self) -> Admitted<Uuid> {
        let issued = self.issued.fetch_add(1, Ordering::Relaxed);
        self.nonces
            .get(usize::try_from(issued).unwrap_or(usize::MAX))
            .cloned()
            .ok_or_else(|| invalid("the fixture host has issued every nonce it was given"))
    }

    fn executor_uid(&self) -> u32 {
        self.executor_uid
    }

    fn host_id(&self) -> Admitted<String> {
        Ok(self.host_id.clone())
    }

    fn facts(&self, path: &Path) -> Admitted<FileFacts> {
        // An ancestor of the scratch root stands in for the platform's own protected chain. A
        // fixture cannot own `/home`, and walking one is not what any of these cases decide.
        if self.root.starts_with(path) {
            return Ok(FileFacts {
                symlink: false,
                regular: false,
                directory: true,
                uid: 0,
                gid: 0,
                mode: 0o755,
                device: 0,
                inode: 0,
                size: 0,
            });
        }
        let mut facts = ess_cli::recovery::real_facts(path)?;
        // The injected half, and the only injected half: ownership and the shared-writable bits.
        // Regularity, symlink-ness, size and physical identity stay exactly what the disk says,
        // so a symlink or a nonregular file in a fixture still refuses for the real reason.
        facts.uid = if self.administrative(path) {
            0
        } else {
            self.executor_uid
        };
        facts.gid = 0;
        facts.mode &= 0o7755;
        facts.mode &= !0o022;
        if let Some((_, uid, mode)) = self
            .overrides
            .iter()
            .find(|(overridden, _, _)| overridden == path)
        {
            facts.uid = *uid;
            facts.mode = *mode;
        }
        Ok(facts)
    }

    fn barrier(&self, barrier: Barrier, label: &str) -> Admitted<()> {
        self.crossed
            .lock()
            .expect("the barrier log is not poisoned")
            .push((barrier, label.to_owned()));
        let Some(fault) = &self.fault else {
            return Ok(());
        };
        if fault.barrier != barrier {
            return Ok(());
        }
        if fault.label.as_deref().is_some_and(|wanted| wanted != label) {
            return Ok(());
        }
        if fault.interrupt {
            // A real termination of a real process. Nothing after this point runs, which is the
            // only honest way to produce "the record was not written and the caller cannot know".
            std::process::exit(97);
        }
        Err(Refusal::new(
            RefusalCode::EvidenceIncomplete,
            format!("the injected control failed the {} barrier", barrier.name()),
        ))
    }
}

/// Lays out one protected fixture arrangement under `root`.
///
/// Directories only. What goes in them is each family's business, because what a case is about is
/// usually exactly which file is missing, malformed or already there.
pub fn scaffold(root: &Path) -> std::io::Result<()> {
    for relative in [
        ADMIN,
        &format!("{ADMIN}/registry-history"),
        &format!("{ADMIN}/authorities"),
        &format!("{ADMIN}/runtime"),
        TOOLS,
        STATE,
        &format!("{STATE}/invocations"),
        TARGET,
    ] {
        std::fs::create_dir_all(root.join(relative))?;
    }
    Ok(())
}

/// A deterministic fixture UUID from one byte.
pub fn uuid(byte: u8) -> Uuid {
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

// --- The protected arrangement -----------------------------------------------------------------

/// Publishes a complete registry snapshot, its immutable generation archive and every authority's
/// retained immutable revision, all at once.
///
/// The three have to agree byte for byte, and a builder that wrote only one of them would make
/// every "the archive does not match" case pass for the wrong reason.
pub fn publish_registry(
    root: &Path,
    registry: &ess_cli::recovery::model::AuthorityRegistry,
) -> std::io::Result<()> {
    let admin = root.join(ADMIN);
    let bytes = ess_cli::recovery::model::write_canonical(registry);
    std::fs::write(admin.join("registry.json"), &bytes)?;
    std::fs::write(
        admin
            .join("registry-history")
            .join(format!("{}.json", registry.generation)),
        &bytes,
    )?;
    for authority in &registry.authorities {
        let history = admin
            .join("authorities")
            .join(authority.authority_id.as_str())
            .join("history");
        std::fs::create_dir_all(&history)?;
        std::fs::write(
            history.join(format!("{}.json", authority.revision)),
            ess_cli::recovery::model::write_canonical(authority),
        )?;
    }
    Ok(())
}

/// Writes a protected kubeconfig defining one cluster and the named contexts.
pub fn write_kubeconfig(
    root: &Path,
    name: &str,
    server: &str,
    certificate_authority: &str,
    contexts: &[&str],
) -> std::io::Result<PathBuf> {
    use std::fmt::Write as _;
    let mut text = format!(
        "apiVersion: v1\nkind: Config\nclusters:\n- name: pinned\n  cluster:\n    server: \
         {server}\n    certificate-authority-data: {certificate_authority}\ncontexts:\n"
    );
    for context in contexts {
        let _ = write!(
            text,
            "- name: {context}\n  context:\n    cluster: pinned\n    user: executor\n"
        );
    }
    text.push_str("users:\n- name: executor\n  user:\n    token-file: /dev/null\n");
    let path = root.join(ADMIN).join(name);
    std::fs::write(&path, text)?;
    Ok(path)
}

/// Installs an artifact under the admitted `<prefix><sha256hex>/helm` arrangement.
///
/// The bytes are whatever the caller supplies, which is the point: the digest in the authority has
/// to be the digest of what is actually there, and a fixture that installed something else would
/// be testing the fixture.
pub fn install_helm(root: &Path, bytes: &[u8]) -> std::io::Result<(PathBuf, String)> {
    let digest = ess_cli::recovery::model::Digest::of_bytes(bytes);
    let hex = digest.as_str().trim_start_matches("sha256:").to_owned();
    let directory = root.join(TOOLS).join(&hex);
    std::fs::create_dir_all(&directory)?;
    let path = directory.join("helm");
    std::fs::write(&path, bytes)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755))?;
    }
    Ok((path, digest.as_str().to_owned()))
}

/// Installs an exact copy of `source` as the admitted artifact.
///
/// The bytes are copied rather than linked, so the installed file is a real regular file with its
/// own digest under the admitted `<prefix><sha256hex>/helm` arrangement, and the probes meet a
/// real executable rather than a script.
pub fn install_executable(root: &Path, source: &Path) -> std::io::Result<(PathBuf, String)> {
    let bytes = std::fs::read(source)?;
    install_helm(root, &bytes)
}

/// Builds a gzip/TAR chart archive out of exactly the members it is given.
///
/// Given, not derived. Every generated-chart refusal in the matrix is a *shape*: a link, a
/// traversal, a duplicate name, an extra member, a missing one, a second root. A builder that
/// produced a well-formed archive from a file map could not express any of them, so this one takes
/// the members literally, including the entry type.
pub fn archive(members: &[(&str, &[u8], tar::EntryType)]) -> Vec<u8> {
    use std::io::Write as _;
    let mut builder = tar::Builder::new(Vec::new());
    for (path, contents, kind) in members {
        let mut header = tar::Header::new_gnu();
        header.set_mode(0o644);
        header.set_mtime(0);
        header.set_uid(0);
        header.set_gid(0);
        header.set_entry_type(*kind);
        if *kind == tar::EntryType::Symlink {
            header.set_size(0);
            header.set_cksum();
            builder
                .append_link(
                    &mut header,
                    path,
                    std::str::from_utf8(contents).expect("a link target is UTF-8"),
                )
                .expect("a link member appends");
            continue;
        }
        header.set_size(contents.len() as u64);
        // The name goes into the header block directly rather than through `set_path`. The
        // builder refuses to *write* a traversing path, and a traversing path is one of the shapes
        // the reader has to refuse — a fixture that could not produce one would leave that
        // refusal untested.
        let name = path.as_bytes();
        assert!(name.len() < 100, "a fixture member name fits one header");
        let gnu = header.as_gnu_mut().expect("a GNU header");
        gnu.name[..name.len()].copy_from_slice(name);
        header.set_cksum();
        builder
            .append(&header, *contents)
            .expect("a member appends");
    }
    let tarball = builder.into_inner().expect("the archive closes");
    let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::fast());
    encoder.write_all(&tarball).expect("the archive compresses");
    encoder.finish().expect("the archive closes")
}

/// The regular-file members of one chart root, in projection order.
pub fn chart_members<'a>(
    root: &str,
    files: &'a std::collections::BTreeMap<String, String>,
) -> Vec<(String, &'a [u8], tar::EntryType)> {
    files
        .iter()
        .map(|(path, contents)| {
            (
                format!("{root}/{path}"),
                contents.as_bytes(),
                tar::EntryType::Regular,
            )
        })
        .collect()
}

/// Borrows a member list into the shape [`archive`] takes.
pub fn as_members<'a>(
    members: &'a [(String, &'a [u8], tar::EntryType)],
) -> Vec<(&'a str, &'a [u8], tar::EntryType)> {
    members
        .iter()
        .map(|(path, contents, kind)| (path.as_str(), *contents, *kind))
        .collect()
}

// --- The synthetic target and the stand-in for the admitted Helm artifact ---------------------
//
// The target is a directory of JSON files, one per release, holding the release's stored manifest
// inventory, its description marker and the direct objects that exist. The driver never reads it;
// only the fake Helm and the fake API do, and only through their own process boundary.

/// One release's durable state in the synthetic target.
#[derive(Debug, Clone, Default)]
pub struct ReleaseState {
    /// The Helm description marker, absent when release storage is absent.
    pub description: Option<String>,
    /// The stored manifest inventory as `Kind/name` strings.
    pub manifest: Vec<String>,
    /// The direct objects that exist, as `Kind/name` to their projection digest.
    pub objects: BTreeMap<String, String>,
}

/// Reads one release's durable state out of the synthetic target.
pub fn read_release(target: &Path, namespace: &str, release: &str) -> ReleaseState {
    let path = target.join(format!("{namespace}--{release}.json"));
    let Ok(text) = std::fs::read_to_string(path) else {
        return ReleaseState::default();
    };
    let Ok(value) = serde_json::from_str::<serde_json::Value>(&text) else {
        return ReleaseState::default();
    };
    ReleaseState {
        description: value
            .get("description")
            .and_then(serde_json::Value::as_str)
            .map(str::to_owned),
        manifest: value
            .get("manifest")
            .and_then(serde_json::Value::as_array)
            .map(|items| {
                items
                    .iter()
                    .filter_map(|item| item.as_str().map(str::to_owned))
                    .collect()
            })
            .unwrap_or_default(),
        objects: value
            .get("objects")
            .and_then(serde_json::Value::as_object)
            .map(|map| {
                map.iter()
                    .filter_map(|(key, value)| {
                        value.as_str().map(|value| (key.clone(), value.to_owned()))
                    })
                    .collect()
            })
            .unwrap_or_default(),
    }
}

/// Writes one release's durable state into the synthetic target.
pub fn write_release(
    target: &Path,
    namespace: &str,
    release: &str,
    state: &ReleaseState,
) -> std::io::Result<()> {
    let path = target.join(format!("{namespace}--{release}.json"));
    if state.description.is_none() && state.objects.is_empty() {
        return match std::fs::remove_file(&path) {
            Err(error) if error.kind() != std::io::ErrorKind::NotFound => Err(error),
            _ => Ok(()),
        };
    }
    let document = serde_json::json!({
        "description": state.description,
        "manifest": state.manifest,
        "objects": state.objects,
    });
    std::fs::write(
        path,
        serde_json::to_string(&document).expect("state serializes"),
    )
}

/// The fixture profile, read from a file beside the installed artifact.
///
/// A file rather than an environment variable, deliberately. The production child environment is
/// *constructed*, not inherited — `process::environment` clears everything — so an environment
/// switch would be invisible to this executable when it runs as the admitted artifact, and a
/// production path that did let one through would be the defect. The profile lives beside the
/// binary, so it changes no byte of the binary the authority pinned.
#[derive(Debug, Default)]
pub struct Profile {
    /// What the version probe reports.
    pub version: Option<String>,
    /// A flag to omit from one operation's help output, as `operation:--flag`.
    pub omit: Option<String>,
    /// Which mutation fault to inject.
    pub fault: Option<String>,
    /// The stored manifest inventory an apply records, comma separated `Kind/name`.
    pub inventory: Option<String>,
    /// The independently controlled synthetic target's directory.
    pub target: Option<String>,
}

/// The profile file's name, beside the installed artifact.
pub const PROFILE: &str = "fixture-profile";

/// Writes a fixture profile beside an installed artifact.
pub fn write_profile(artifact: &Path, lines: &[(&str, &str)]) -> std::io::Result<()> {
    use std::fmt::Write as _;
    let mut text = String::new();
    for (key, value) in lines {
        let _ = writeln!(text, "{key}={value}");
    }
    std::fs::write(
        artifact
            .parent()
            .expect("an installed artifact has a directory")
            .join(PROFILE),
        text,
    )
}

fn profile() -> Profile {
    let Ok(exe) = std::env::current_exe() else {
        return Profile::default();
    };
    let Some(directory) = exe.parent() else {
        return Profile::default();
    };
    let Ok(text) = std::fs::read_to_string(directory.join(PROFILE)) else {
        return Profile::default();
    };
    let mut profile = Profile::default();
    for line in text.lines() {
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let value = Some(value.to_owned());
        match key {
            "version" => profile.version = value,
            "omit" => profile.omit = value,
            "fault" => profile.fault = value,
            "inventory" => profile.inventory = value,
            "target" => profile.target = value,
            _ => {}
        }
    }
    profile
}

fn main() -> std::process::ExitCode {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let profile = profile();
    let target = PathBuf::from(profile.target.clone().unwrap_or_default());
    match helm(&arguments, &target, &profile) {
        Ok(output) => {
            print!("{output}");
            std::process::ExitCode::SUCCESS
        }
        Err(message) => {
            eprint!("{message}");
            std::process::ExitCode::FAILURE
        }
    }
}

/// The bounded, closed subset of Helm's surface this fixture answers.
///
/// It answers exactly the probes and operations C06 admits and nothing else. An argument outside
/// that set is an error here, so a production path that started passing one would be caught by the
/// fixture rather than tolerated by it. Compatible output is not proof of stock Helm's semantics,
/// and nothing here claims it is.
fn helm(arguments: &[String], target: &Path, profile: &Profile) -> Result<String, String> {
    match arguments.first().map(String::as_str) {
        Some("version") => Ok(format!(
            "{}\n",
            profile
                .version
                .clone()
                .unwrap_or_else(|| "v3.16.2".to_owned())
        )),
        Some(operation) if arguments.get(1).map(String::as_str) == Some("--help") => {
            Ok(help(operation, profile.omit.as_deref()))
        }
        Some("upgrade") => mutate(arguments, target, profile, false),
        Some("uninstall") => mutate(arguments, target, profile, true),
        Some(other) => Err(format!("unsupported operation {other}\n")),
        None => Err("no operation\n".to_owned()),
    }
}

fn help(operation: &str, omit: Option<&str>) -> String {
    use std::fmt::Write as _;
    let flags: &[&str] = match operation {
        "upgrade" => &[
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
        "uninstall" => &[
            "--no-hooks",
            "--wait",
            "--timeout",
            "--namespace",
            "--kubeconfig",
            "--kube-context",
        ],
        "template" => &["--values", "--namespace", "--no-hooks", "--skip-crds"],
        "status" => &["--output", "--namespace"],
        "get" => &["--revision", "--namespace"],
        _ => &[],
    };
    let omitted = omit
        .and_then(|omit| omit.split_once(':'))
        .filter(|(named, _)| *named == operation)
        .map(|(_, flag)| flag);
    let mut text = format!("Usage: helm {operation} [flags]\n\nFlags:\n");
    for flag in flags {
        if omitted == Some(flag) {
            continue;
        }
        let _ = writeln!(text, "  {flag}");
    }
    text
}

fn value(arguments: &[String], flag: &str) -> Option<String> {
    arguments
        .iter()
        .position(|argument| argument == flag)
        .and_then(|position| arguments.get(position + 1))
        .cloned()
}

/// Applies or removes one release against the durable synthetic target.
///
/// The fault selects between the barriers a mutation can fail at, and each one is a different fact
/// about the target: `no-effect` fails before touching it, `effect-then-fail` changes it and then
/// fails, `lost-ack` changes it and never returns, `timeout` hangs. The target is a directory of
/// its own, so what it says survives this process either way.
fn mutate(
    arguments: &[String],
    target: &Path,
    profile: &Profile,
    remove: bool,
) -> Result<String, String> {
    let fault = profile.fault.clone().unwrap_or_default();
    let namespace = value(arguments, "--namespace").ok_or("no namespace\n")?;
    let release = arguments.get(1).cloned().ok_or("no release\n")?;
    if fault == "no-effect" {
        return Err("the injected control failed this call before any effect\n".to_owned());
    }
    if fault == "timeout" {
        std::thread::sleep(std::time::Duration::from_secs(3600));
    }
    let mut state = read_release(target, &namespace, &release);
    if remove {
        state = ReleaseState::default();
    } else {
        state.description = value(arguments, "--description");
        state.manifest = profile
            .inventory
            .clone()
            .unwrap_or_default()
            .split(',')
            .filter(|entry| !entry.is_empty())
            .map(str::to_owned)
            .collect();
        state.objects = state
            .manifest
            .iter()
            .map(|address| (address.clone(), format!("live:{address}")))
            .collect();
    }
    write_release(target, &namespace, &release, &state)
        .map_err(|error| format!("the synthetic target could not be written: {error}\n"))?;
    if fault == "effect-then-fail" {
        return Err("the injected control failed this call after its effect\n".to_owned());
    }
    if fault == "lost-ack" {
        // The effect happened and the acknowledgement never arrives. Terminating rather than
        // returning is the point: a caller that saw an exit code would have evidence it must not
        // have.
        std::process::exit(101);
    }
    Ok(format!("Release \"{release}\" has been upgraded.\n"))
}
