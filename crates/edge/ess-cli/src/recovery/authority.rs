//! The complete active-registry scan, and the admission of exactly one authority out of it.
//!
//! Nothing in this module manufactures an authority. It reads what the trusted control account
//! published, checks the concrete bindings it can check, and refuses everything else. An authority
//! whose fields merely pass validation is not thereby approved, and a registry that is absent
//! authorizes nothing at all.
//!
//! The scan is deliberately whole-registry. Two unrelated active entries cannot silently name
//! different stores for one target or the same object address for two services, and an entry the
//! caller did not select is still read: an unreadable or invalid active entry prevents admission
//! rather than being skipped. That strictness is the only fence available here — re-reading a file
//! cannot exclude a malicious administrator, and the registry's completeness, the uniqueness of
//! the enrolled host and the absence of writers outside this mechanism remain trusted caller
//! assertions rather than discovered facts.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use super::journal::{read_protected, under, RECORD_LIMIT};
use super::model::{
    canonical_endpoint, read_canonical, Admitted, Authority, AuthorityRegistry, Digest,
    ObjectAddress, Refusal, RefusalCode, RegistryFormat, RegistryRef, ReleaseProjection, Text,
    Uuid,
};
use super::{Host, Trust};

/// The production root the trusted control account publishes under.
pub const PRODUCTION_ROOT: &str = "/etc/ess/recovery";

/// The complete active snapshot's file name.
pub const REGISTRY: &str = "registry.json";
/// The immutable generation archive's directory name.
pub const REGISTRY_HISTORY: &str = "registry-history";
/// The per-authority immutable revision archive's directory name.
pub const AUTHORITIES: &str = "authorities";

fn mismatch(detail: impl Into<String>) -> Refusal {
    Refusal::new(RefusalCode::AuthorityMismatch, detail)
}

/// The admitted registry snapshot and the reference every claim and context carries.
#[derive(Debug, Clone)]
pub struct AdmittedRegistry {
    /// The complete snapshot.
    pub registry: AuthorityRegistry,
    /// Its generation and whole-byte digest.
    pub reference: RegistryRef,
}

/// What one authority's kubeconfig and context resolve to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KubeconfigFacts {
    /// The canonical endpoint every admitted context resolves to.
    pub endpoint: String,
    /// The digest of the trusted CA bundle bytes.
    pub ca_digest: Digest,
    /// The context names the file defines.
    pub contexts: Vec<String>,
}

/// Reads a protected kubeconfig and refuses every unsupported credential configuration.
///
/// The refusals are the point. Insecure TLS, an executable credential plugin, a configured proxy
/// and impersonation each move the decision about who this process is out of the pinned authority
/// and into something the authority did not review, and no raw credential value from this file
/// reaches a diagnostic, an observation or the journal.
pub fn read_kubeconfig(host: &dyn Host, path: &Path) -> Admitted<KubeconfigFacts> {
    let text = under(
        RefusalCode::TargetMismatch,
        read_protected(host, path, Trust::Administrative),
    )?;
    if text.len() as u64 > RECORD_LIMIT {
        return Err(mismatch("the kubeconfig is past the admitted bound"));
    }
    let document: serde_yaml::Value = serde_yaml::from_str(&text)
        .map_err(|_| mismatch("the kubeconfig is not a YAML document"))?;

    for unsupported in [
        "exec",
        "auth-provider",
        "proxy-url",
        "as",
        "as-uid",
        "as-groups",
        "insecure-skip-tls-verify",
    ] {
        if contains_key(&document, unsupported) {
            return Err(Refusal::new(
                RefusalCode::UnsupportedProfile,
                format!("the kubeconfig configures {unsupported}, which this profile refuses"),
            ));
        }
    }

    let clusters = document
        .get("clusters")
        .and_then(serde_yaml::Value::as_sequence)
        .ok_or_else(|| mismatch("the kubeconfig defines no cluster"))?;
    if clusters.len() != 1 {
        return Err(mismatch(
            "this profile admits a kubeconfig defining exactly one cluster",
        ));
    }
    let cluster = clusters[0]
        .get("cluster")
        .ok_or_else(|| mismatch("a kubeconfig cluster entry has no cluster"))?;
    let server = cluster
        .get("server")
        .and_then(serde_yaml::Value::as_str)
        .ok_or_else(|| mismatch("a kubeconfig cluster names no server"))?;
    let endpoint = canonical_endpoint(server)?;
    let authority = cluster
        .get("certificate-authority-data")
        .and_then(serde_yaml::Value::as_str)
        .ok_or_else(|| mismatch("a kubeconfig cluster embeds no certificate authority"))?;

    let contexts = document
        .get("contexts")
        .and_then(serde_yaml::Value::as_sequence)
        .ok_or_else(|| mismatch("the kubeconfig defines no context"))?
        .iter()
        .map(|entry| {
            entry
                .get("name")
                .and_then(serde_yaml::Value::as_str)
                .map(str::to_owned)
                .ok_or_else(|| mismatch("a kubeconfig context has no name"))
        })
        .collect::<Admitted<Vec<String>>>()?;

    Ok(KubeconfigFacts {
        endpoint,
        ca_digest: Digest::of_bytes(authority.as_bytes()),
        contexts,
    })
}

fn contains_key(value: &serde_yaml::Value, key: &str) -> bool {
    match value {
        serde_yaml::Value::Mapping(mapping) => mapping
            .iter()
            .any(|(name, nested)| name.as_str() == Some(key) || contains_key(nested, key)),
        serde_yaml::Value::Sequence(items) => items.iter().any(|item| contains_key(item, key)),
        _ => false,
    }
}

/// Reads the complete active snapshot and matches it to its immutable generation archive.
pub fn read_registry(host: &dyn Host, root: &Path) -> Admitted<AdmittedRegistry> {
    let path = root.join(REGISTRY);
    let text = under(
        RefusalCode::AuthorityMismatch,
        read_protected(host, &path, Trust::Administrative),
    )?;
    let registry: AuthorityRegistry = read_canonical(&text).map_err(|error| {
        mismatch(format!(
            "{} is not a canonical ess-execution-registry/1 snapshot: {}",
            path.display(),
            error.detail
        ))
    })?;
    if registry.format != RegistryFormat::V1 {
        return Err(mismatch("unsupported registry format"));
    }
    let archived = root
        .join(REGISTRY_HISTORY)
        .join(format!("{}.json", registry.generation));
    let archive = read_protected(host, &archived, Trust::Administrative).map_err(|error| {
        mismatch(format!(
            "generation {} has no immutable archive: {}",
            registry.generation, error.detail
        ))
    })?;
    if archive != text {
        return Err(mismatch(format!(
            "the active snapshot does not equal its immutable archive at generation {}",
            registry.generation
        )));
    }
    Ok(AdmittedRegistry {
        reference: RegistryRef {
            generation: registry.generation,
            digest: Digest::of_bytes(text.as_bytes()),
        },
        registry,
    })
}

/// The indexes C05 requires over **every** active entry.
///
/// They are built before selection and over the whole registry, so an invalid entry the caller did
/// not ask for still prevents admission. Each index states one admitted relationship; a second
/// disagreeing binding for the same key is the collision it exists to reject.
#[derive(Debug, Default)]
struct Indexes {
    endpoint_by_cluster: BTreeMap<String, String>,
    ca_by_cluster: BTreeMap<String, Digest>,
    cluster_by_endpoint: BTreeMap<String, String>,
    namespace_uid: BTreeMap<(String, String), String>,
    namespace_name: BTreeMap<(String, String), String>,
    exclusion: BTreeMap<String, (String, u32, String, String)>,
    cluster_by_store: BTreeMap<String, String>,
    store_identity: BTreeMap<(u64, u64), String>,
    authority_by_environment: BTreeMap<(String, String), Uuid>,
    permit_by_service: BTreeSet<(Uuid, String, String)>,
    owner_by_release: BTreeMap<(String, String, String), (Uuid, String, Uuid)>,
    permit_by_object: BTreeMap<(String, String, String, String), (Uuid, String)>,
    helm: Option<(String, Digest, String)>,
}

fn insert<K: Ord + std::fmt::Debug, V: PartialEq + std::fmt::Debug>(
    map: &mut BTreeMap<K, V>,
    key: K,
    value: V,
    what: &str,
) -> Admitted<()> {
    if let Some(existing) = map.get(&key) {
        if existing != &value {
            return Err(mismatch(format!(
                "{what}: {key:?} is already bound to {existing:?} and cannot also bind {value:?}"
            )));
        }
        return Ok(());
    }
    map.insert(key, value);
    Ok(())
}

/// Admits the whole registry, then selects and authenticates the requested authority's bindings.
///
/// The order is load bearing. The full scan succeeds first; only then is the requested authority
/// selected. An implementation that selected first and validated the selection would admit a
/// registry whose other entries name a different store for the same target.
pub fn admit(
    host: &dyn Host,
    root: &Path,
    requested: &Uuid,
) -> Admitted<(AdmittedRegistry, Authority)> {
    let admitted = read_registry(host, root)?;
    let registry = &admitted.registry;
    if registry.authorities.is_empty() {
        return Err(mismatch("an empty registry authorizes no execution"));
    }
    for pair in registry.authorities.windows(2) {
        if pair[0].authority_id >= pair[1].authority_id {
            return Err(mismatch(
                "the registry lists active authorities sorted by authority UUID, without \
                 duplicates or a second active revision of one authority",
            ));
        }
    }

    let host_id = host.host_id()?;
    let executor_uid = host.executor_uid();
    let mut indexes = Indexes::default();

    for authority in &registry.authorities {
        scan_entry(host, root, authority, &host_id, executor_uid, &mut indexes)?;
    }

    let selected = registry
        .authorities
        .iter()
        .find(|authority| &authority.authority_id == requested)
        .ok_or_else(|| {
            mismatch(format!(
                "the active registry holds no authority {requested}; this implementation never \
                 accepts a self-authorizing file in its place"
            ))
        })?;
    Ok((admitted.clone(), selected.clone()))
}

fn permit_projections(
    permit: &super::model::ReleasePermit,
) -> impl Iterator<Item = &ReleaseProjection> {
    permit.baseline.iter().chain(permit.desired.iter())
}

fn permit_addresses<'a>(
    projections: impl Iterator<Item = &'a ReleaseProjection>,
) -> Vec<ObjectAddress> {
    let mut addresses: Vec<ObjectAddress> =
        projections.flat_map(ReleaseProjection::addresses).collect();
    addresses.sort();
    addresses.dedup();
    addresses
}

fn bind_namespace(indexes: &mut Indexes, cluster: &str, name: &Text, uid: &Text) -> Admitted<()> {
    insert(
        &mut indexes.namespace_uid,
        (cluster.to_owned(), name.to_string()),
        uid.to_string(),
        "within one cluster a namespace name has one UID",
    )?;
    insert(
        &mut indexes.namespace_name,
        (cluster.to_owned(), uid.to_string()),
        name.to_string(),
        "within one cluster a namespace UID has one name",
    )
}

/// Admits one active entry and folds it into every index.
///
/// Split out of [`admit`] because it is one entry's whole story — retained revision, host, Helm,
/// endpoint, namespaces, store, environment, kubeconfig, permits and object addresses — and the
/// loop that runs it over *every* active entry is the part worth reading at a glance.
fn scan_entry(
    host: &dyn Host,
    root: &Path,
    authority: &Authority,
    host_id: &str,
    executor_uid: u32,
    indexes: &mut Indexes,
) -> Admitted<()> {
    authority.validate()?;
    check_retained_revision(host, root, authority)?;

    if authority.host.host_id.as_str() != host_id {
        return Err(mismatch(format!(
            "authority {} names host {:?}, and this host is {host_id:?}",
            authority.authority_id, authority.host.host_id
        )));
    }
    if authority.host.executor_uid != executor_uid {
        return Err(mismatch(format!(
            "authority {} names executor uid {} and this executor is {executor_uid}",
            authority.authority_id, authority.host.executor_uid
        )));
    }
    let helm = (
        authority.host.helm.path.to_string(),
        authority.host.helm.digest.clone(),
        authority.host.helm.version.to_string(),
    );
    match &indexes.helm {
        Some(existing) if existing != &helm => {
            return Err(mismatch(
                "all active entries on this host must name the same admitted Helm artifact",
            ))
        }
        Some(_) => {}
        None => indexes.helm = Some(helm),
    }

    let cluster = authority.target.identity_namespace.uid.to_string();
    let endpoint = canonical_endpoint(authority.target.api_server.as_str())?;
    insert(
        &mut indexes.endpoint_by_cluster,
        cluster.clone(),
        endpoint.clone(),
        "one physical cluster has one canonical API endpoint",
    )?;
    insert(
        &mut indexes.ca_by_cluster,
        cluster.clone(),
        authority.target.ca_digest.clone(),
        "one physical cluster has one trusted CA digest",
    )?;
    insert(
        &mut indexes.cluster_by_endpoint,
        endpoint.clone(),
        cluster.clone(),
        "one canonical API endpoint has one identity-namespace UID",
    )?;
    bind_namespace(
        indexes,
        &cluster,
        &authority.target.identity_namespace.name,
        &authority.target.identity_namespace.uid,
    )?;

    let state_root = authority.host.state_root.to_string();
    insert(
        &mut indexes.exclusion,
        cluster.clone(),
        (
            authority.host.host_id.to_string(),
            authority.host.executor_uid,
            authority.host.store_epoch.to_string(),
            state_root.clone(),
        ),
        "one physical cluster has one host/executor/store/state-root exclusion namespace",
    )?;
    insert(
        &mut indexes.cluster_by_store,
        state_root.clone(),
        cluster.clone(),
        "one store root serves one physical cluster",
    )?;
    check_store_alias(host, indexes, &state_root, &cluster)?;

    insert(
        &mut indexes.authority_by_environment,
        (cluster.clone(), authority.environment.to_string()),
        authority.authority_id.clone(),
        "one environment on one cluster has one active authority",
    )?;

    check_context_aliases(host, authority, &endpoint)?;

    scan_permits(authority, &cluster, indexes)
}

/// Requires every admitted context alias to resolve through the protected kubeconfig.
///
/// A context name is an alias, and an alias that resolves somewhere else is the collision this
/// check exists for: one reused kubeconfig binding cannot point at two targets or two principals.
fn check_context_aliases(host: &dyn Host, authority: &Authority, endpoint: &str) -> Admitted<()> {
    let facts = read_kubeconfig(host, Path::new(authority.host.kubeconfig.as_str()))?;
    if facts.endpoint != endpoint {
        return Err(Refusal::new(
            RefusalCode::TargetMismatch,
            format!(
                "authority {}'s kubeconfig resolves to {:?} and its target pins {endpoint:?}",
                authority.authority_id, facts.endpoint
            ),
        ));
    }
    if facts.ca_digest != authority.target.ca_digest {
        return Err(Refusal::new(
            RefusalCode::TargetMismatch,
            format!(
                "authority {}'s kubeconfig embeds a different trusted CA than its target pins",
                authority.authority_id
            ),
        ));
    }
    for context in &authority.contexts {
        if !facts.contexts.iter().any(|name| name == context.as_str()) {
            return Err(Refusal::new(
                RefusalCode::TargetMismatch,
                format!(
                    "authority {} admits context {context:?}, which its kubeconfig does not define",
                    authority.authority_id
                ),
            ));
        }
    }
    Ok(())
}

/// Folds one entry's release permits into the address and ownership indexes.
///
/// Separate from the target half above because it is a different kind of claim: the target half is
/// about *where* this authority acts, and this half is about *what it exclusively owns there* —
/// one permit per service, one owner per physical release address, one permit per direct object
/// address across the whole registry.
fn scan_permits(authority: &Authority, cluster: &str, indexes: &mut Indexes) -> Admitted<()> {
    for permit in &authority.releases {
        if !indexes.permit_by_service.insert((
            authority.authority_id.clone(),
            authority.environment.to_string(),
            permit.service.to_string(),
        )) {
            return Err(mismatch(format!(
                "one service in one authority environment has one release permit, and {} \
             carries a second for {}",
                authority.authority_id, permit.service
            )));
        }
        bind_namespace(
            indexes,
            cluster,
            &permit.namespace.name,
            &permit.namespace.uid,
        )?;
        insert(
            &mut indexes.owner_by_release,
            (
                cluster.to_owned(),
                permit.namespace.uid.to_string(),
                permit.release_name.to_string(),
            ),
            (
                authority.authority_id.clone(),
                permit.service.to_string(),
                permit.incarnation.clone(),
            ),
            "one physical release address has one authority/service/incarnation",
        )?;
        for address in permit_addresses(permit_projections(permit)) {
            insert(
                &mut indexes.permit_by_object,
                (
                    cluster.to_owned(),
                    permit.namespace.uid.to_string(),
                    address.kind.to_string(),
                    address.name.to_string(),
                ),
                (authority.authority_id.clone(), permit.service.to_string()),
                "one direct object address belongs to one release permit",
            )?;
        }
    }
    Ok(())
}

/// Rejects equal, nested and physically identical store roots naming different clusters.
///
/// A lexical comparison alone is insufficient. Two different path strings can reach one directory
/// through a link or a bind, and a nested root shares the parent that publishes its names, so
/// physical identity is compared as well as the canonical string.
fn check_store_alias(
    host: &dyn Host,
    indexes: &mut Indexes,
    state_root: &str,
    cluster: &str,
) -> Admitted<()> {
    for (existing, existing_cluster) in indexes.cluster_by_store.clone() {
        if existing == state_root {
            continue;
        }
        let nested = PathBuf::from(&existing).starts_with(state_root)
            || PathBuf::from(state_root).starts_with(&existing);
        if nested && existing_cluster != cluster {
            return Err(mismatch(format!(
                "store roots {state_root:?} and {existing:?} are nested and name different \
                 physical clusters"
            )));
        }
    }
    if let Ok(facts) = host.facts(Path::new(state_root)) {
        insert(
            &mut indexes.store_identity,
            (facts.device, facts.inode),
            cluster.to_owned(),
            "one physical store directory serves one physical cluster",
        )?;
    }
    Ok(())
}

/// Requires every embedded authority to equal its retained immutable revision, byte for byte.
fn check_retained_revision(host: &dyn Host, root: &Path, authority: &Authority) -> Admitted<()> {
    let path = root
        .join(AUTHORITIES)
        .join(authority.authority_id.as_str())
        .join("history")
        .join(format!("{}.json", authority.revision));
    let retained = read_protected(host, &path, Trust::Administrative).map_err(|error| {
        mismatch(format!(
            "authority {} revision {} has no retained immutable revision: {}",
            authority.authority_id, authority.revision, error.detail
        ))
    })?;
    let parsed: Authority = read_canonical(&retained).map_err(|error| {
        mismatch(format!(
            "the retained revision of authority {} is not canonical: {}",
            authority.authority_id, error.detail
        ))
    })?;
    if &parsed != authority {
        return Err(mismatch(format!(
            "the registry's copy of authority {} revision {} differs from its retained immutable \
             revision",
            authority.authority_id, authority.revision
        )));
    }
    Ok(())
}

/// Re-reads the snapshot and refuses a changed generation or changed bytes mid-invocation.
///
/// C05 requires this immediately after claim acquisition and immediately before each mutation. A
/// changed registry stops the launch; it never becomes a new policy the running invocation adopts.
pub fn recheck(host: &dyn Host, root: &Path, admitted: &RegistryRef) -> Admitted<()> {
    let current = read_registry(host, root)?;
    if current.reference.generation != admitted.generation {
        return Err(mismatch(format!(
            "the registry moved from generation {} to {} during this invocation",
            admitted.generation, current.reference.generation
        )));
    }
    if current.reference.digest != admitted.digest {
        return Err(mismatch(format!(
            "the registry's bytes changed at generation {} during this invocation",
            admitted.generation
        )));
    }
    Ok(())
}

/// Admits the active registry against one piece of retained execution evidence.
///
/// C05 is one sentence with two halves, and both are decided here: "Reject a generation older than
/// retained execution evidence, **or different bytes for an already retained generation**."
///
/// The second half needs the whole reference, not the generation. A generation is immutable by
/// contract, so bytes that moved underneath one are a rewritten history — and nothing else in the
/// reader can see it: `read_registry` compares the active snapshot with its own generation
/// archive, and republishing at the same generation rewrites both. The retained `Opened` is the
/// only witness that the bytes were ever different, which is why the binding names it.
pub fn admit_generation(retained: &RegistryRef, current: &RegistryRef) -> Admitted<()> {
    if current.generation < retained.generation {
        return Err(mismatch(format!(
            "the active registry is generation {} and retained evidence names {}",
            current.generation, retained.generation
        )));
    }
    if current.generation == retained.generation && current.digest != retained.digest {
        return Err(mismatch(format!(
            "retained execution evidence ran under generation {} with different bytes than the \
             active registry now carries at that same generation; a generation is immutable and \
             this one has been rewritten",
            retained.generation
        )));
    }
    Ok(())
}
