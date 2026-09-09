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
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};

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

/// Lays out a complete protected arrangement whose authority admits exactly `desired`.
///
/// This is the "synthetic admitted authority" the offline qualification allows a test-only Rust
/// adapter to inject, and it is what lets the cache-boundary vectors keep their assertions after
/// `reconcile` became authority-gated. It is built *from the desired document's own bytes*, so the
/// authority's pinned digest is the digest of the plan the vector actually wrote — which is why the
/// admission it goes through is the production one, not a stub.
///
/// It admits nothing the production reader would not: the registry is scanned whole, each
/// authority must equal its retained immutable revision, the kubeconfig must resolve to the pinned
/// endpoint and trust, and the host and executor must match. What is *injected* is the
/// administrative ownership of the files, and only that.
pub fn provision_cache_lane(
    root: &Path,
    desired_bytes: &str,
    services: &[(String, String, String)],
) -> std::io::Result<Uuid> {
    use ess_cli::recovery::model::{
        Authority, AuthorityFormat, AuthorityRegistry, ChartSource, Digest, HelmBinary,
        HelmProtocol, HelmVersion, HostPolicy, Index, NamespacePin, ObjectAddress,
        ObjectFingerprint, ObjectKind, PrincipalPin, Profile, RegistryFormat, ReleasePermit,
        ReleaseProjection, TargetPin, Text,
    };
    scaffold(root)?;
    let text = |value: &str| Text::new(value).expect("a fixture Text admits");
    let (helm_path, helm_digest) = install_helm(root, b"cache-lane fixture artifact")?;
    let kubeconfig = write_kubeconfig(
        root,
        "kubeconfig.yaml",
        "https://api.fixture.invalid:6443",
        FIXTURE_CA,
        &["fixture"],
    )?;
    let mut releases: Vec<ReleasePermit> = services
        .iter()
        .map(|(service, namespace, release_name)| ReleasePermit {
            service: text(service),
            namespace: NamespacePin {
                name: text(namespace),
                uid: text(&format!("ns-{namespace}")),
            },
            release_name: text(release_name),
            incarnation: uuid(0x22),
            may_create: true,
            baseline: None,
            desired: Some(ReleaseProjection {
                chart: ChartSource {
                    runtime_digest: Digest::of_bytes(service.as_bytes()),
                    chart_name: text("fixture"),
                    chart_version: text("1.0.0"),
                },
                objects: vec![ObjectFingerprint {
                    object: ObjectAddress {
                        kind: ObjectKind::Deployment,
                        name: text(release_name),
                    },
                    content_digest: Digest::of_bytes(release_name.as_bytes()),
                }],
            }),
            repair_from: None,
        })
        .collect();
    releases.sort_by(|left, right| left.service.cmp(&right.service));
    let authority = Authority {
        format: AuthorityFormat::V1,
        profile: Profile::SingleHostGeneratedHelm1,
        authority_id: uuid(0x71),
        revision: Index::new(1).expect("a fixture revision admits"),
        target: TargetPin {
            api_server: text("https://api.fixture.invalid:6443"),
            ca_digest: Digest::of_bytes(FIXTURE_CA.as_bytes()),
            identity_namespace: NamespacePin {
                name: text("kube-system"),
                uid: text("cluster-fixture"),
            },
        },
        principal: PrincipalPin {
            namespace: text("ess-system"),
            name: text("ess-recovery"),
            uid: text("sa-fixture"),
        },
        host: HostPolicy {
            host_id: text("fixture-host"),
            executor_uid: rustix::process::getuid().as_raw(),
            store_epoch: uuid(0x11),
            state_root: text(&root.join(STATE).display().to_string()),
            kubeconfig: text(&kubeconfig.display().to_string()),
            helm: HelmBinary {
                path: text(&helm_path.display().to_string()),
                digest: Digest::new(&helm_digest).expect("the installed digest admits"),
                version: HelmVersion::new("v3.16.2").expect("a canonical version admits"),
                protocol: HelmProtocol::Helm3Recovery1,
            },
        },
        contexts: vec![text("fixture")],
        environment: text("fixture"),
        desired_digest: Digest::of_bytes(desired_bytes.as_bytes()),
        baseline_digest: None,
        releases,
        quiescence: Vec::new(),
    };
    let id = authority.authority_id.clone();
    publish_registry(
        root,
        &AuthorityRegistry {
            format: RegistryFormat::V1,
            generation: Index::new(1).expect("a fixture generation admits"),
            authorities: vec![authority],
        },
    )?;
    Ok(id)
}

/// The fixture certificate authority's embedded bytes, shared by every arrangement here.
pub const FIXTURE_CA: &str = "LS0tLUZJWFRVUkUtQ0EtLS0t";

// --- The synthetic cluster, and the platform the engine runs against ---------------------------
//
// The cluster's state is a file, not a field. That is the whole point: it outlives the process
// that mutated it, a restart reads only what was actually written, and the executor's journal
// never reconstructs it. A synthetic authenticated response is a synthetic authenticated response
// — it establishes nothing about a real cluster's identity or the caller's authority — but what it
// *is* is independently controlled, which is what a restart case needs.

use ess_cli::recovery::model::{
    Authority, Digest, HelmIdentity, Index, ObjectAddress, ObjectKind, ReleasePermit,
    ReleaseProjection, Text,
};
use ess_cli::recovery::observe::{ApiRead, Identity};
use ess_cli::recovery::process::Outcome;
use ess_cli::recovery::{chart::PreparedChart, Helm, Platform};

/// One live object as the synthetic cluster reports it, at a named generation.
///
/// A real object shape, not a token: the engine takes `observe::projection_digest` of whatever the
/// API returns, so the caller-approved fingerprint has to be the projection digest of *this*, and
/// the two differ whenever the generation does.
pub fn live_object(
    namespace: &str,
    kind: ObjectKind,
    name: &str,
    generation: &str,
) -> serde_json::Value {
    serde_json::json!({
        "apiVersion": kind.api_version(),
        "kind": kind.to_string(),
        "metadata": {
            "name": name,
            "namespace": namespace,
            "labels": {"app.kubernetes.io/instance": name},
            "annotations": {},
            "ownerReferences": [],
            "finalizers": [],
            "uid": "server-assigned",
            "resourceVersion": "server-assigned",
            "generation": 7,
            "creationTimestamp": "server-assigned",
            "managedFields": ["server-assigned"]
        },
        "spec": {"replicas": 1, "generationTag": generation},
        "status": {"observedGeneration": 7}
    })
}

/// The approved live-projection digest for one address at one generation.
///
/// One function so a fingerprint the caller approves and a fingerprint the cluster reports are the
/// same value by construction. A fixture that computed them separately would be asserting that two
/// copies of a constant agree.
pub fn live_digest(namespace: &str, kind: ObjectKind, name: &str, generation: &str) -> Digest {
    ess_cli::recovery::observe::projection_digest(&live_object(namespace, kind, name, generation))
}

/// The independently controlled synthetic cluster, stored as one canonical file.
#[derive(Debug, Clone)]
pub struct Cluster {
    path: PathBuf,
}

impl Cluster {
    /// Opens the cluster whose state lives at `<root>/target-state/state.json`.
    pub fn at(root: &Path) -> Self {
        Self {
            path: root.join(TARGET).join("state.json"),
        }
    }

    fn read(&self) -> serde_json::Value {
        std::fs::read_to_string(&self.path)
            .ok()
            .and_then(|text| serde_json::from_str(&text).ok())
            .unwrap_or_else(|| serde_json::json!({"releases": {}, "objects": {}}))
    }

    fn write(&self, value: &serde_json::Value) {
        if let Some(parent) = self.path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        std::fs::write(
            &self.path,
            serde_json::to_string(value).expect("the cluster state serializes"),
        )
        .expect("the synthetic cluster is writable");
    }

    /// Places one release exactly as `projection` describes it, marker and all.
    ///
    /// The projection carries the caller-approved fingerprints, so the cluster reports exactly what
    /// the authority approved. A fixture that computed the two separately would be asserting that
    /// two copies of a constant agree; this way the only thing under test is whether the engine
    /// compares them.
    pub fn place(
        &self,
        permit: &ReleasePermit,
        marker: &str,
        projection: &ReleaseProjection,
        generation: &str,
    ) {
        let mut state = self.read();
        let key = format!("{}/{}", permit.namespace.name, permit.release_name);
        // An upgrade replaces the stored manifest, and an address the new manifest does not name
        // stops existing. Modelling that is what makes "after apply, every baseline-only address
        // is absent" a fact about the target rather than a fact about this fixture.
        let previous: Vec<String> = state["releases"][&key]["manifest"]
            .as_array()
            .map(|items| {
                items
                    .iter()
                    .filter_map(|item| item.as_str().map(str::to_owned))
                    .collect()
            })
            .unwrap_or_default();
        let retained: Vec<String> = projection
            .objects
            .iter()
            .map(|object| format!("{}/{}", object.object.kind, object.object.name))
            .collect();
        for address in previous {
            if !retained.contains(&address) {
                if let Some(map) = state["objects"].as_object_mut() {
                    map.remove(&format!("{}/{address}", permit.namespace.name));
                }
            }
        }
        state["releases"][&key] = serde_json::json!({
            "description": marker,
            "manifest": projection
                .objects
                .iter()
                .map(|object| format!("{}/{}", object.object.kind, object.object.name))
                .collect::<Vec<_>>(),
        });
        for object in &projection.objects {
            Self::place_object(
                &mut state,
                permit.namespace.name.as_str(),
                &object.object,
                generation,
            );
        }
        self.write(&state);
    }

    fn place_object(
        state: &mut serde_json::Value,
        namespace: &str,
        address: &ObjectAddress,
        generation: &str,
    ) {
        let key = format!("{namespace}/{}/{}", address.kind, address.name);
        state["objects"][&key] = serde_json::json!({
            "uid": format!("uid-{}-{}", address.kind, address.name),
            "resourceVersion": "1",
            "object": live_object(namespace, address.kind, address.name.as_str(), generation),
        });
    }

    /// Places one object with no release, as a foreign occupant.
    pub fn occupy(&self, namespace: &str, address: &ObjectAddress, generation: &str) {
        let mut state = self.read();
        Self::place_object(&mut state, namespace, address, generation);
        state["objects"][&format!("{namespace}/{}/{}", address.kind, address.name)]["uid"] =
            serde_json::json!("uid-foreign");
        self.write(&state);
    }

    /// Removes one release and every object its manifest named.
    pub fn retire(&self, namespace: &str, release: &str) {
        let mut state = self.read();
        let key = format!("{namespace}/{release}");
        let manifest: Vec<String> = state["releases"][&key]["manifest"]
            .as_array()
            .map(|items| {
                items
                    .iter()
                    .filter_map(|item| item.as_str().map(str::to_owned))
                    .collect()
            })
            .unwrap_or_default();
        if let Some(map) = state["releases"].as_object_mut() {
            map.remove(&key);
        }
        let pinned: Vec<String> = state["pinned"]
            .as_array()
            .map(|items| {
                items
                    .iter()
                    .filter_map(|item| item.as_str().map(str::to_owned))
                    .collect()
            })
            .unwrap_or_default();
        for address in manifest {
            let key = format!("{namespace}/{address}");
            if pinned.contains(&key) {
                continue;
            }
            if let Some(map) = state["objects"].as_object_mut() {
                map.remove(&key);
            }
        }
        self.write(&state);
    }

    /// Leaves one object behind after a removal, as a finalizer or a retained direct object would.
    ///
    /// Pinned, not raced. A fixture that removed the object concurrently with the run would be
    /// asserting a scheduling accident; this marks the address as one the uninstall does not
    /// clear, which is exactly what a finalizer is.
    pub fn pin(&self, namespace: &str, address: &ObjectAddress) {
        let mut state = self.read();
        let key = format!("{namespace}/{}/{}", address.kind, address.name);
        let mut pinned: Vec<String> = state["pinned"]
            .as_array()
            .map(|items| {
                items
                    .iter()
                    .filter_map(|item| item.as_str().map(str::to_owned))
                    .collect()
            })
            .unwrap_or_default();
        if !pinned.contains(&key) {
            pinned.push(key);
        }
        state["pinned"] = serde_json::json!(pinned);
        self.write(&state);
    }

    /// Rewrites one object's content, as manual drift between invocations would.
    pub fn drift(&self, namespace: &str, address: &ObjectAddress, generation: &str) {
        let mut state = self.read();
        Self::place_object(&mut state, namespace, address, generation);
        self.write(&state);
    }

    /// Rewrites one release's ownership marker, as a foreign incarnation would.
    pub fn rebrand(&self, namespace: &str, release: &str, marker: &str) {
        let mut state = self.read();
        let key = format!("{namespace}/{release}");
        state["releases"][&key]["description"] = serde_json::json!(marker);
        self.write(&state);
    }

    /// Whether a release's storage is present.
    pub fn has_release(&self, namespace: &str, release: &str) -> bool {
        !self.read()["releases"][&format!("{namespace}/{release}")].is_null()
    }

    /// Whether one direct object address is occupied.
    pub fn has_object(&self, namespace: &str, address: &ObjectAddress) -> bool {
        !self.read()["objects"][&format!("{namespace}/{}/{}", address.kind, address.name)].is_null()
    }
}

/// What the fixture's Helm does at one operation index.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HelmFault {
    /// The launch is definitely refused. Nothing ran.
    NotLaunched,
    /// The call started and failed with no effect on the target.
    StartedNoEffect,
    /// The call started, changed the target, and then failed.
    EffectThenFailure,
    /// The call started, changed the target, and its acknowledgement never arrived.
    LostAcknowledgement,
    /// The call started, changed the target, and timed out.
    Timeout,
}

/// The engine's platform over the synthetic cluster.
///
/// Real production code runs above it: the same admission, the same journal, the same predicates,
/// the same ordering. What is injected below it is the cluster's answers and the process outcomes,
/// through the Rust seams the offline qualification names. A compatible answer here is not proof
/// of stock Helm's semantics and a synthetic UID is not a real cluster's identity.
pub struct FixturePlatform {
    shared: std::sync::Arc<Shared>,
    payload: Vec<u8>,
}

/// The parts the engine's Helm seam and the test both hold.
///
/// Shared by reference count rather than by lifetime: the `Helm` the platform hands back outlives
/// the borrow that produced it, and a fixture that borrowed would need an escape hatch to say so.
#[derive(Debug)]
struct Shared {
    cluster: Cluster,
    faults: BTreeMap<usize, HelmFault>,
    /// The operation index whose chart payload cannot be acquired, if any.
    payload_fault: Option<usize>,
    /// Whether authenticated reads are unavailable.
    ///
    /// Unavailable, not absent. A failed read establishes nothing about the target, which is the
    /// whole of R04: an executor that treated it as "nothing is there" would infer empty state.
    unavailable: bool,
    calls: std::sync::Mutex<Vec<String>>,
    index: AtomicUsize,
    acquisitions: AtomicUsize,
}

impl std::fmt::Debug for FixturePlatform {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("FixturePlatform")
            .field("shared", &self.shared)
            .finish_non_exhaustive()
    }
}

impl FixturePlatform {
    /// A platform over one cluster, serving `payload` as every release's proved chart bytes.
    pub fn new(cluster: Cluster, payload: Vec<u8>) -> Self {
        Self {
            shared: std::sync::Arc::new(Shared {
                cluster,
                faults: BTreeMap::new(),
                payload_fault: None,
                unavailable: false,
                calls: std::sync::Mutex::new(Vec::new()),
                index: AtomicUsize::new(0),
                acquisitions: AtomicUsize::new(0),
            }),
            payload,
        }
    }

    /// The same platform with one operation index failing in a named way.
    #[must_use]
    pub fn failing_at(self, index: usize, fault: HelmFault) -> Self {
        let mut faults = self.shared.faults.clone();
        faults.insert(index, fault);
        self.rebuilt(faults, self.shared.payload_fault, self.shared.unavailable)
    }

    /// The same platform whose chart payload cannot be acquired at one operation index.
    #[must_use]
    pub fn acquisition_failing_at(self, index: usize) -> Self {
        self.rebuilt(
            self.shared.faults.clone(),
            Some(index),
            self.shared.unavailable,
        )
    }

    /// The same platform whose authenticated reads are unavailable.
    #[must_use]
    pub fn unavailable(self) -> Self {
        self.rebuilt(self.shared.faults.clone(), self.shared.payload_fault, true)
    }

    fn rebuilt(
        &self,
        faults: BTreeMap<usize, HelmFault>,
        payload_fault: Option<usize>,
        unavailable: bool,
    ) -> Self {
        Self {
            shared: std::sync::Arc::new(Shared {
                cluster: self.shared.cluster.clone(),
                faults,
                payload_fault,
                unavailable,
                calls: std::sync::Mutex::new(Vec::new()),
                index: AtomicUsize::new(0),
                acquisitions: AtomicUsize::new(0),
            }),
            payload: self.payload.clone(),
        }
    }

    /// Every mutating call this platform was asked for, in order.
    pub fn calls(&self) -> Vec<String> {
        self.shared
            .calls
            .lock()
            .expect("the call log is not poisoned")
            .clone()
    }

    /// The cluster this platform acts on.
    pub fn cluster(&self) -> &Cluster {
        &self.shared.cluster
    }
}

impl Shared {
    fn next_index(&self) -> usize {
        self.index.fetch_add(1, Ordering::Relaxed)
    }
}

struct FixtureApi {
    cluster: Cluster,
    unavailable: bool,
}

impl ess_cli::recovery::observe::Api for FixtureApi {
    fn identity_namespace(&self) -> Admitted<Identity> {
        Ok(Identity {
            name: "kube-system".to_owned(),
            namespace: None,
            uid: "cluster-fixture".to_owned(),
        })
    }

    fn self_subject(&self) -> Admitted<Identity> {
        Ok(Identity {
            name: "ess-recovery".to_owned(),
            namespace: Some("ess-system".to_owned()),
            uid: "sa-fixture".to_owned(),
        })
    }

    fn namespace(&self, name: &str) -> Admitted<Identity> {
        Ok(Identity {
            name: name.to_owned(),
            namespace: None,
            uid: format!("ns-{name}"),
        })
    }

    fn object(&self, namespace: &str, address: &ObjectAddress) -> Admitted<ApiRead> {
        if self.unavailable {
            return Err(ess_cli::recovery::model::Refusal::new(
                ess_cli::recovery::model::RefusalCode::ObservationUnavailable,
                "the authenticated read did not complete",
            ));
        }
        let state = self.cluster.read();
        let key = format!("{namespace}/{}/{}", address.kind, address.name);
        let found = &state["objects"][&key];
        if found.is_null() {
            return Ok(ApiRead::Absent);
        }
        Ok(ApiRead::Present {
            uid: found["uid"].as_str().unwrap_or_default().to_owned(),
            resource_version: found["resourceVersion"]
                .as_str()
                .unwrap_or_default()
                .to_owned(),
            // The whole object, exactly as it was placed. The engine takes its own projection
            // digest of this; nothing here hands it a fingerprint to compare against itself.
            object: found["object"].clone(),
        })
    }
}

struct FixtureHelm {
    shared: std::sync::Arc<Shared>,
}

impl Helm for FixtureHelm {
    fn release(&self, permit: &ReleasePermit, _context: &str) -> Admitted<Option<HelmIdentity>> {
        let state = self.shared.cluster.read();
        let key = format!("{}/{}", permit.namespace.name, permit.release_name);
        let found = &state["releases"][&key];
        if found.is_null() {
            return Ok(None);
        }
        let manifest: Vec<ObjectAddress> = found["manifest"]
            .as_array()
            .map(|items| {
                items
                    .iter()
                    .filter_map(|item| item.as_str())
                    .filter_map(parse_address)
                    .collect()
            })
            .unwrap_or_default();
        Ok(Some(HelmIdentity {
            description: Text::new(found["description"].as_str().unwrap_or("(none)"))?,
            revision: Index::new(1)?,
            storage_uid: Text::new(format!("secret-{key}"))?,
            manifest,
            hook_count: Index::new(0)?,
        }))
    }

    fn render(
        &self,
        _chart: &PreparedChart,
        permit: &ReleasePermit,
        _context: &str,
    ) -> Admitted<String> {
        use std::fmt::Write as _;
        let objects = permit
            .desired
            .as_ref()
            .or(permit.baseline.as_ref())
            .map_or(&[][..], |projection| projection.objects.as_slice());
        let mut text = String::new();
        for (index, object) in objects.iter().enumerate() {
            if index > 0 {
                text.push_str("---\n");
            }
            let kind = object.object.kind;
            let _ = write!(
                text,
                "apiVersion: {}\nkind: {kind}\nmetadata:\n  name: {}\n  namespace: {}\n",
                kind.api_version(),
                object.object.name,
                permit.namespace.name
            );
        }
        Ok(text)
    }

    fn apply(
        &self,
        _chart: &PreparedChart,
        permit: &ReleasePermit,
        _context: &str,
        marker: &str,
        _timeout: &str,
    ) -> Admitted<Outcome> {
        let index = self.shared.next_index();
        self.shared
            .calls
            .lock()
            .expect("the call log is not poisoned")
            .push(format!("apply {}", permit.service));
        let Some(projection) = permit.desired.as_ref() else {
            return Err(ess_cli::recovery::model::invalid(
                "an apply was attempted for a permit with no desired projection",
            ));
        };
        let effect = |shared: &Shared| shared.cluster.place(permit, marker, projection, "desired");
        Ok(settle(&self.shared, index, effect))
    }

    fn remove(&self, permit: &ReleasePermit, _context: &str, _timeout: &str) -> Admitted<Outcome> {
        let index = self.shared.next_index();
        self.shared
            .calls
            .lock()
            .expect("the call log is not poisoned")
            .push(format!("remove {}", permit.service));
        let effect = |shared: &Shared| {
            shared
                .cluster
                .retire(permit.namespace.name.as_str(), permit.release_name.as_str());
        };
        Ok(settle(&self.shared, index, effect))
    }
}

/// Applies the fault at `index`, if any, around the operation's effect on the cluster.
fn settle(shared: &Shared, index: usize, effect: impl Fn(&Shared)) -> Outcome {
    let acknowledged = Outcome {
        launched: true,
        status: Some(0),
        timed_out: false,
        stdout: Vec::new(),
        stderr: Vec::new(),
    };
    match shared.faults.get(&index) {
        None => {
            effect(shared);
            acknowledged
        }
        Some(HelmFault::NotLaunched) => Outcome {
            launched: false,
            status: None,
            timed_out: false,
            stdout: Vec::new(),
            stderr: b"spawn refused".to_vec(),
        },
        Some(HelmFault::StartedNoEffect) => Outcome {
            launched: true,
            status: Some(1),
            timed_out: false,
            stdout: Vec::new(),
            stderr: b"started and failed before any effect".to_vec(),
        },
        Some(HelmFault::EffectThenFailure) => {
            effect(shared);
            Outcome {
                launched: true,
                status: Some(1),
                timed_out: false,
                stdout: Vec::new(),
                stderr: b"started, changed the target, then failed".to_vec(),
            }
        }
        Some(HelmFault::LostAcknowledgement) => {
            effect(shared);
            Outcome {
                launched: true,
                status: None,
                timed_out: false,
                stdout: Vec::new(),
                stderr: b"acknowledgement lost".to_vec(),
            }
        }
        Some(HelmFault::Timeout) => {
            effect(shared);
            Outcome {
                launched: true,
                status: None,
                timed_out: true,
                stdout: Vec::new(),
                stderr: b"timed out".to_vec(),
            }
        }
    }
}

fn parse_address(value: &str) -> Option<ObjectAddress> {
    let (kind, name) = value.split_once('/')?;
    Some(ObjectAddress {
        kind: ObjectKind::parse(kind)?,
        name: Text::new(name).ok()?,
    })
}

impl Platform for FixturePlatform {
    fn api(
        &self,
        _authority: &Authority,
        _context: &str,
    ) -> Admitted<Box<dyn ess_cli::recovery::observe::Api>> {
        Ok(Box::new(FixtureApi {
            cluster: self.shared.cluster.clone(),
            unavailable: self.shared.unavailable,
        }))
    }

    fn helm(&self, _authority: &Authority, _prefix: &str) -> Admitted<Box<dyn Helm>> {
        Ok(Box::new(FixtureHelm {
            shared: std::sync::Arc::clone(&self.shared),
        }))
    }

    fn payload(&self, release: &ess_deployment::DeploymentRelease) -> Admitted<Vec<u8>> {
        let index = self.shared.acquisitions.fetch_add(1, Ordering::Relaxed);
        if self.shared.payload_fault == Some(index) {
            return Err(ess_cli::recovery::model::Refusal::new(
                ess_cli::recovery::model::RefusalCode::PreparationFailed,
                format!(
                    "the pinned chart payload for {} was not proved: the acquisition failed",
                    release.service
                ),
            ));
        }
        Ok(self.payload.clone())
    }

    fn private(&self, label: &str) -> Admitted<PathBuf> {
        let directory = std::env::temp_dir().join(format!(
            "ess-fixture-private-{}-{label}-{}",
            std::process::id(),
            self.shared.index.load(Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&directory).map_err(|error| {
            ess_cli::recovery::model::invalid(format!("a private directory: {error}"))
        })?;
        Ok(directory)
    }
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
