//! Strict Rust representation of the 44 declared `recovery.execution` types.
//!
//! The declaration (`models/execution-recovery/domains/execution.yaml`) states shapes, closed
//! choices and simple invariants. Everything cross-record — grammar of a digest, uniqueness of an
//! address, ordering of a sequence, cardinality of an inventory — is a reader obligation and lives
//! here or in the module that owns the record. Nothing in this file reaches a cluster, a process
//! or an administrative decision.
//!
//! Canonical bytes are UTF-8 JSON with recursively sorted object keys, preserved array order, no
//! duplicate keys, no floating-point numbers and exactly one trailing newline. `serde_json`'s
//! object is a `BTreeMap`, so serialization sorts; the reading half is [`canonical_value`], which
//! refuses a duplicate key and a float rather than silently taking the last one.

use std::collections::BTreeMap;
use std::fmt;

use serde::de::{self, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize};

pub use ess_deployment::Digest;

/// Every way this contract refuses, as declared by `recovery.execution.RefusalCode`.
///
/// The variants are the closed list in the declaration. A refusal names one of them; no code is
/// invented at a call site, and no outcome is recast into a different one to obtain a zero exit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum RefusalCode {
    /// A document, flag or byte sequence did not admit.
    InvalidInput,
    /// A profile other than `SingleHostGeneratedHelm1` was named.
    UnsupportedProfile,
    /// The registry, an authority revision or its selection does not agree.
    AuthorityMismatch,
    /// The pinned physical target does not agree with what was read.
    TargetMismatch,
    /// The pinned principal does not agree with what was read.
    PrincipalMismatch,
    /// The supplied baseline does not agree with the admitted authority.
    BaselineMismatch,
    /// A foreign incarnation or a foreign occupant holds an address.
    OwnershipConflict,
    /// The required observation could not be acquired.
    ObservationUnavailable,
    /// The required observation was acquired but is outside the freshness budget.
    ObservationStale,
    /// The observation does not satisfy the operation's pre-state predicate.
    ObservedDrift,
    /// Mutation is blocked: a retained claim, an unresolved predecessor or a missing quiescence.
    MutationBlocked,
    /// The recovery store, its header or its provisioning did not admit.
    StoreInvalid,
    /// Durable evidence is missing, corrupt or incomplete.
    EvidenceIncomplete,
    /// Artifact, chart, values or command preparation failed.
    PreparationFailed,
    /// A launch was definitely refused.
    LaunchFailed,
    /// A started operation's effect cannot be established.
    EffectIndeterminate,
    /// A removal was selected without the reviewed removal authorization.
    RemovalNotPermitted,
    /// Direct objects or finalizers remain, so absence cannot be claimed.
    DirectObjectsRemain,
}

impl fmt::Display for RefusalCode {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::InvalidInput => "InvalidInput",
            Self::UnsupportedProfile => "UnsupportedProfile",
            Self::AuthorityMismatch => "AuthorityMismatch",
            Self::TargetMismatch => "TargetMismatch",
            Self::PrincipalMismatch => "PrincipalMismatch",
            Self::BaselineMismatch => "BaselineMismatch",
            Self::OwnershipConflict => "OwnershipConflict",
            Self::ObservationUnavailable => "ObservationUnavailable",
            Self::ObservationStale => "ObservationStale",
            Self::ObservedDrift => "ObservedDrift",
            Self::MutationBlocked => "MutationBlocked",
            Self::StoreInvalid => "StoreInvalid",
            Self::EvidenceIncomplete => "EvidenceIncomplete",
            Self::PreparationFailed => "PreparationFailed",
            Self::LaunchFailed => "LaunchFailed",
            Self::EffectIndeterminate => "EffectIndeterminate",
            Self::RemovalNotPermitted => "RemovalNotPermitted",
            Self::DirectObjectsRemain => "DirectObjectsRemain",
        })
    }
}

/// Every `RefusalCode` variant, for the checks that must enumerate the whole closed list.
///
/// A hand-maintained list beside a closed enum is the defect the enumeration exists to prevent, so
/// this constant is asserted exhaustive by a `match` in this module's own tests rather than by
/// anybody remembering to extend it.
pub const REFUSAL_CODES: &[RefusalCode] = &[
    RefusalCode::InvalidInput,
    RefusalCode::UnsupportedProfile,
    RefusalCode::AuthorityMismatch,
    RefusalCode::TargetMismatch,
    RefusalCode::PrincipalMismatch,
    RefusalCode::BaselineMismatch,
    RefusalCode::OwnershipConflict,
    RefusalCode::ObservationUnavailable,
    RefusalCode::ObservationStale,
    RefusalCode::ObservedDrift,
    RefusalCode::MutationBlocked,
    RefusalCode::StoreInvalid,
    RefusalCode::EvidenceIncomplete,
    RefusalCode::PreparationFailed,
    RefusalCode::LaunchFailed,
    RefusalCode::EffectIndeterminate,
    RefusalCode::RemovalNotPermitted,
    RefusalCode::DirectObjectsRemain,
];

/// A named refusal: the closed code, and what could not be established.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Refusal {
    /// The closed declared code.
    pub code: RefusalCode,
    /// What could not be established, in terms a caller can act on. Never a credential.
    pub detail: String,
}

impl Refusal {
    /// Names an unresolved claim under a closed code.
    pub fn new(code: RefusalCode, detail: impl Into<String>) -> Self {
        Self {
            code,
            detail: detail.into(),
        }
    }
}

impl fmt::Display for Refusal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.code, self.detail)
    }
}

impl std::error::Error for Refusal {}

/// The result of every admission in this contract.
pub type Admitted<T> = Result<T, Refusal>;

/// Refuses under [`RefusalCode::InvalidInput`].
pub fn invalid(detail: impl Into<String>) -> Refusal {
    Refusal::new(RefusalCode::InvalidInput, detail)
}

/// A nonempty string, as `recovery.execution.Text`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(transparent)]
pub struct Text(String);

impl Text {
    /// Admits a nonempty string with no control characters.
    ///
    /// The declaration states only `value != ""`. Control characters are a reader obligation: a
    /// newline or a NUL in a name reaches an argument vector, a path and a serialized record.
    pub fn new(value: impl Into<String>) -> Admitted<Self> {
        let value = value.into();
        if value.is_empty() {
            return Err(invalid("Text must not be empty"));
        }
        if value.chars().any(char::is_control) {
            return Err(invalid("Text must not contain control characters"));
        }
        Ok(Self(value))
    }

    /// The exact admitted spelling.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Text {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for Text {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        Self::new(value).map_err(de::Error::custom)
    }
}

/// A nonnegative integer, as `recovery.execution.Index`.
///
/// Represented as `u64` so that "nonnegative" is a property of the type rather than of a check a
/// later reader has to remember, and bounded below `2^53` so a canonical JSON number stays exact
/// for any reader that goes through a double.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(transparent)]
pub struct Index(u64);

/// The largest admitted [`Index`]: the exact-integer bound of an IEEE-754 double.
pub const INDEX_LIMIT: u64 = 1 << 53;

impl Index {
    /// Admits a nonnegative index below [`INDEX_LIMIT`].
    pub fn new(value: u64) -> Admitted<Self> {
        if value >= INDEX_LIMIT {
            return Err(invalid(format!(
                "Index {value} is out of the admitted range"
            )));
        }
        Ok(Self(value))
    }

    /// The admitted value.
    pub fn get(self) -> u64 {
        self.0
    }

    /// The next index, refusing at the bound rather than wrapping.
    pub fn next(self) -> Admitted<Self> {
        Self::new(self.0 + 1)
    }
}

impl fmt::Display for Index {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.0)
    }
}

impl<'de> Deserialize<'de> for Index {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = u64::deserialize(deserializer)?;
        Self::new(value).map_err(de::Error::custom)
    }
}

/// The lowercase 8-4-4-4-12 hexadecimal form ESS spells `Uuid`.
///
/// This is a declared builtin rather than one of the 44 named types, and it carries no dependency:
/// the grammar is the whole of it, and nothing here generates one.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(transparent)]
pub struct Uuid(String);

impl Uuid {
    /// Admits the exact lowercase hyphenated form.
    pub fn new(value: impl Into<String>) -> Admitted<Self> {
        let value = value.into();
        let groups: Vec<&str> = value.split('-').collect();
        let shaped = groups.len() == 5
            && [8usize, 4, 4, 4, 12]
                .iter()
                .zip(&groups)
                .all(|(width, group)| group.len() == *width)
            && groups.iter().all(|group| {
                group
                    .bytes()
                    .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
            });
        if shaped {
            Ok(Self(value))
        } else {
            Err(invalid(format!("invalid UUID {value:?}")))
        }
    }

    /// The exact admitted spelling.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Uuid {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for Uuid {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        Self::new(value).map_err(de::Error::custom)
    }
}

/// `(store_epoch, nonce)`, as `recovery.execution.InvocationId`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InvocationId {
    /// The caller-provisioned store epoch.
    pub store_epoch: Uuid,
    /// The OS-random nonce this invocation reserved.
    pub nonce: Uuid,
}

/// The single admitted authority document format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuthorityFormat {
    /// `ess-execution-authority/1`.
    #[serde(rename = "ess-execution-authority/1")]
    V1,
}

/// The single admitted store header format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StoreFormat {
    /// `ess-execution-store/1`.
    #[serde(rename = "ess-execution-store/1")]
    V1,
}

/// The single admitted lock claim format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LockFormat {
    /// `ess-execution-lock/1`.
    #[serde(rename = "ess-execution-lock/1")]
    V1,
}

/// The single admitted journal entry format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum JournalFormat {
    /// `ess-execution-evidence/1`.
    #[serde(rename = "ess-execution-evidence/1")]
    V1,
}

/// The single admitted registry format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RegistryFormat {
    /// `ess-execution-registry/1`.
    #[serde(rename = "ess-execution-registry/1")]
    V1,
}

/// The only supported execution profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Profile {
    /// One trusted host, exclusively managed release addresses, ESS-generated charts only.
    SingleHostGeneratedHelm1,
}

/// A namespace name bound to its authenticated UID.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NamespacePin {
    /// The namespace name.
    pub name: Text,
    /// Its authenticated UID.
    pub uid: Text,
}

/// The pinned physical cluster.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TargetPin {
    /// The canonical HTTPS API root.
    pub api_server: Text,
    /// The digest of the trusted CA bundle.
    pub ca_digest: Digest,
    /// The identity namespace, whose name this profile fixes to `kube-system`.
    pub identity_namespace: NamespacePin,
}

/// The namespace this profile fixes as the physical-cluster key.
pub const IDENTITY_NAMESPACE: &str = "kube-system";

impl TargetPin {
    /// Admits the canonical endpoint form and the fixed identity namespace name.
    ///
    /// The endpoint is compared as a canonical string elsewhere, so a form that two spellings can
    /// reach — a default port written out, an upper-case host, a trailing path, a query — is
    /// refused here rather than normalized silently into an alias.
    pub fn validate(&self) -> Admitted<()> {
        if self.identity_namespace.name.as_str() != IDENTITY_NAMESPACE {
            return Err(Refusal::new(
                RefusalCode::UnsupportedProfile,
                format!(
                    "the identity namespace is fixed to {IDENTITY_NAMESPACE:?}, not {:?}",
                    self.identity_namespace.name
                ),
            ));
        }
        canonical_endpoint(self.api_server.as_str())?;
        Ok(())
    }
}

/// Admits a canonical HTTPS API root and returns its `host:port` authority.
///
/// No user information, query, fragment or path; lowercase host; the default HTTPS port written
/// out is refused rather than normalized, because two admitted spellings of one endpoint is
/// precisely the alias the registry index exists to reject.
pub fn canonical_endpoint(value: &str) -> Admitted<String> {
    let rest = value
        .strip_prefix("https://")
        .ok_or_else(|| invalid(format!("API endpoint {value:?} is not an https:// root")))?;
    if rest.is_empty()
        || rest.contains('@')
        || rest.contains('?')
        || rest.contains('#')
        || rest.contains('/')
    {
        return Err(invalid(format!(
            "API endpoint {value:?} must be an https://host[:port] root with no user \
             information, path, query or fragment"
        )));
    }
    if rest.chars().any(|c| c.is_ascii_uppercase()) {
        return Err(invalid(format!(
            "API endpoint {value:?} must spell its host in lower case"
        )));
    }
    let (host, port) = match rest.rsplit_once(':') {
        Some((host, port)) if !host.is_empty() && !host.contains(':') => (host, Some(port)),
        Some(_) | None => (rest, None),
    };
    if host.is_empty() {
        return Err(invalid(format!("API endpoint {value:?} names no host")));
    }
    match port {
        Some("443") => {
            return Err(invalid(format!(
                "API endpoint {value:?} writes the default HTTPS port out; one endpoint has one \
                 canonical spelling"
            )))
        }
        Some(port) if port.is_empty() || !port.bytes().all(|b| b.is_ascii_digit()) => {
            return Err(invalid(format!(
                "API endpoint {value:?} has no decimal port"
            )))
        }
        _ => {}
    }
    Ok(rest.to_owned())
}

/// The pinned executing `ServiceAccount`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PrincipalPin {
    /// The `ServiceAccount` namespace.
    pub namespace: Text,
    /// The `ServiceAccount` name.
    pub name: Text,
    /// Its authenticated UID.
    pub uid: Text,
}

/// The host's executor, store and Helm policy.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HostPolicy {
    /// The independently provisioned host identity.
    pub host_id: Text,
    /// The non-root UID this executor must run as.
    pub executor_uid: u32,
    /// The caller-provisioned store epoch.
    pub store_epoch: Uuid,
    /// The absolute canonical state root.
    pub state_root: Text,
    /// The absolute protected kubeconfig path.
    pub kubeconfig: Text,
    /// The exact admitted Helm artifact.
    pub helm: HelmBinary,
}

impl HostPolicy {
    /// Admits `executor_uid > 0` and absolute non-relative paths.
    pub fn validate(&self) -> Admitted<()> {
        if self.executor_uid == 0 {
            return Err(Refusal::new(
                RefusalCode::UnsupportedProfile,
                "executor_uid must be a nonzero, non-root UID",
            ));
        }
        absolute_path(self.state_root.as_str(), "state_root")?;
        absolute_path(self.kubeconfig.as_str(), "kubeconfig")?;
        self.helm.validate()
    }
}

/// Admits an absolute path with no `.`, `..` or empty component.
pub fn absolute_path(value: &str, field: &str) -> Admitted<()> {
    if !value.starts_with('/') {
        return Err(invalid(format!("{field} {value:?} is not absolute")));
    }
    if value.len() > 1 && value.ends_with('/') {
        return Err(invalid(format!(
            "{field} {value:?} has a trailing separator"
        )));
    }
    for component in value.split('/').skip(1) {
        if component.is_empty() || component == "." || component == ".." {
            return Err(invalid(format!(
                "{field} {value:?} is not a canonical path"
            )));
        }
    }
    Ok(())
}

/// The digest-bound runtime, chart name and chart version a projection is rendered from.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChartSource {
    /// The canonical digest of the caller-pinned `ess-runtime-ir/1` document.
    pub runtime_digest: Digest,
    /// The chart name `project_helm` is called with.
    pub chart_name: Text,
    /// The chart version `project_helm` is called with.
    pub chart_version: Text,
}

/// The three object kinds this profile's charts can produce.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum ObjectKind {
    /// `apps/v1` Deployment.
    Deployment,
    /// `apps/v1` `StatefulSet`.
    StatefulSet,
    /// `v1` Service.
    Service,
}

impl ObjectKind {
    /// The `apiVersion` a rendered document of this kind must carry.
    pub fn api_version(self) -> &'static str {
        match self {
            Self::Deployment | Self::StatefulSet => "apps/v1",
            Self::Service => "v1",
        }
    }

    /// The declared kind matching a rendered `kind` string, or `None` for an unsupported one.
    pub fn parse(kind: &str) -> Option<Self> {
        match kind {
            "Deployment" => Some(Self::Deployment),
            "StatefulSet" => Some(Self::StatefulSet),
            "Service" => Some(Self::Service),
            _ => None,
        }
    }
}

impl fmt::Display for ObjectKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Deployment => "Deployment",
            Self::StatefulSet => "StatefulSet",
            Self::Service => "Service",
        })
    }
}

/// One direct object address within a release's namespace.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObjectAddress {
    /// The object kind.
    pub kind: ObjectKind,
    /// The object name.
    pub name: Text,
}

/// A caller-approved complete live projection digest for one address.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObjectFingerprint {
    /// The address this fingerprint approves.
    pub object: ObjectAddress,
    /// The digest of the complete approved live projection.
    pub content_digest: Digest,
}

/// One complete rendered inventory: its chart source and every approved object.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReleaseProjection {
    /// What the projection is rendered from.
    pub chart: ChartSource,
    /// The complete ordered object inventory.
    pub objects: Vec<ObjectFingerprint>,
}

impl ReleaseProjection {
    /// Admits a nonempty inventory with no duplicate address, in ascending address order.
    ///
    /// Ordering is not decoration. A persisted list whose order is free is a list two callers can
    /// write differently and a digest can disagree about, and this one is compared byte for byte
    /// against a rendered inventory.
    pub fn validate(&self) -> Admitted<()> {
        if self.objects.is_empty() {
            return Err(invalid("a release projection has no objects"));
        }
        for pair in self.objects.windows(2) {
            if pair[0].object >= pair[1].object {
                return Err(invalid(format!(
                    "release projection objects must be in ascending address order without \
                     duplicates; {:?}/{} is not before {:?}/{}",
                    pair[0].object.kind,
                    pair[0].object.name,
                    pair[1].object.kind,
                    pair[1].object.name
                )));
            }
        }
        Ok(())
    }

    /// The inventory's addresses.
    pub fn addresses(&self) -> Vec<ObjectAddress> {
        self.objects
            .iter()
            .map(|object| object.object.clone())
            .collect()
    }
}

/// Authenticated Helm release metadata for one address.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HelmIdentity {
    /// The exact `ess-recovery/1:<authority>:<incarnation>` ownership marker.
    pub description: Text,
    /// The observed release revision.
    pub revision: Index,
    /// The UID of the release storage Secret. Metadata only; no Secret value is read.
    pub storage_uid: Text,
    /// The stored manifest's object inventory.
    pub manifest: Vec<ObjectAddress>,
    /// How many hooks the stored release declares. Any hook refuses.
    pub hook_count: Index,
}

/// The exact ownership marker for an authority and incarnation.
pub fn ownership_marker(authority: &Uuid, incarnation: &Uuid) -> String {
    format!("ess-recovery/1:{authority}:{incarnation}")
}

/// An authenticated present object: its identity evidence and its projection digest.
///
/// `uid` and `resource_version` are recorded beside the digest rather than inside it. They are
/// server bookkeeping: they say which object was read, and they change without the projection
/// changing, so folding them into `content_digest` would make every readback differ.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PresentObject {
    /// The address read.
    pub object: ObjectAddress,
    /// The server-assigned UID.
    pub uid: Text,
    /// The server-assigned resource version.
    pub resource_version: Text,
    /// The digest of the complete live projection, excluding server bookkeeping metadata.
    pub content_digest: Digest,
}

/// One authenticated read result: absent, or present with its evidence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", deny_unknown_fields)]
pub enum ObjectRead {
    /// The API authoritatively answered that this address holds no object.
    Absent(ObjectAddress),
    /// The API returned an object at this address.
    Present(PresentObject),
}

impl ObjectRead {
    /// The address this read covers.
    pub fn address(&self) -> &ObjectAddress {
        match self {
            Self::Absent(address) => address,
            Self::Present(present) => &present.object,
        }
    }
}

/// One bounded authenticated statement about a release at one read interval.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReleaseSnapshot {
    /// The authenticated Helm release metadata, absent when release storage is absent.
    pub helm: Option<HelmIdentity>,
    /// Exactly one read for every member of the admitted address union.
    pub objects: Vec<ObjectRead>,
}

impl ReleaseSnapshot {
    /// Admits exactly one read per address in `union`, in ascending order.
    ///
    /// "An empty or partial list cannot prove absence" is the whole point of this check: a
    /// snapshot that omits an address is not a snapshot that found it absent.
    pub fn covers(&self, union: &[ObjectAddress]) -> Admitted<()> {
        let read: Vec<&ObjectAddress> = self.objects.iter().map(ObjectRead::address).collect();
        for pair in read.windows(2) {
            if pair[0] >= pair[1] {
                return Err(Refusal::new(
                    RefusalCode::ObservationUnavailable,
                    "snapshot reads must be in ascending address order without duplicates",
                ));
            }
        }
        let expected: Vec<&ObjectAddress> = union.iter().collect();
        if read != expected {
            return Err(Refusal::new(
                RefusalCode::ObservationUnavailable,
                format!(
                    "a snapshot must read every address of the admitted union exactly once; it \
                     covers {} of {}",
                    read.len(),
                    union.len()
                ),
            ));
        }
        Ok(())
    }

    /// The read at `address`, if the snapshot covers it.
    pub fn read(&self, address: &ObjectAddress) -> Option<&ObjectRead> {
        self.objects.iter().find(|read| read.address() == address)
    }
}

/// One service's exclusively managed release address and its admitted projections.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReleasePermit {
    /// The service this permit belongs to.
    pub service: Text,
    /// The release namespace and its authenticated UID.
    pub namespace: NamespacePin,
    /// The Helm release name.
    pub release_name: Text,
    /// The active incarnation of this address.
    pub incarnation: Uuid,
    /// Whether first creation is authorized.
    pub may_create: bool,
    /// The admitted baseline projection, absent for a first creation.
    pub baseline: Option<ReleaseProjection>,
    /// The admitted desired projection, absent for a baseline-only retirement.
    pub desired: Option<ReleaseProjection>,
    /// An exact independently reviewed repair pre-state.
    pub repair_from: Option<ReleaseSnapshot>,
}

impl ReleasePermit {
    /// Admits the cardinality the declaration's optional fields cannot state.
    ///
    /// "At least one of baseline or desired" is a reader obligation by C11's own words, and a
    /// permit with neither is a permit that names an address and authorizes nothing at it.
    pub fn validate(&self) -> Admitted<()> {
        if self.baseline.is_none() && self.desired.is_none() {
            return Err(invalid(format!(
                "release permit {} has neither a baseline nor a desired projection",
                self.service
            )));
        }
        if let Some(baseline) = &self.baseline {
            baseline.validate()?;
        }
        if let Some(desired) = &self.desired {
            desired.validate()?;
        }
        if self.baseline.is_some() && self.may_create {
            return Err(invalid(format!(
                "release permit {} admits a baseline, so it is not a first creation and \
                 may_create cannot regain first-deployment behavior",
                self.service
            )));
        }
        if let Some(repair) = &self.repair_from {
            repair.covers(&self.union())?;
        }
        Ok(())
    }

    /// The union of baseline and desired addresses, ascending and deduplicated.
    ///
    /// Every observation covers this union, so that a differing inventory never silently omits an
    /// address on either side.
    pub fn union(&self) -> Vec<ObjectAddress> {
        let mut union: Vec<ObjectAddress> = self
            .baseline
            .iter()
            .chain(self.desired.iter())
            .flat_map(ReleaseProjection::addresses)
            .collect();
        union.sort();
        union.dedup();
        union
    }

    /// Addresses in the desired inventory that the baseline does not contain.
    pub fn desired_only(&self) -> Vec<ObjectAddress> {
        let baseline = self.baseline.as_ref().map(ReleaseProjection::addresses);
        self.desired
            .as_ref()
            .map(ReleaseProjection::addresses)
            .unwrap_or_default()
            .into_iter()
            .filter(|address| {
                baseline
                    .as_ref()
                    .is_none_or(|baseline| !baseline.contains(address))
            })
            .collect()
    }

    /// Addresses in the baseline inventory that the desired inventory does not contain.
    pub fn baseline_only(&self) -> Vec<ObjectAddress> {
        let desired = self.desired.as_ref().map(ReleaseProjection::addresses);
        self.baseline
            .as_ref()
            .map(ReleaseProjection::addresses)
            .unwrap_or_default()
            .into_iter()
            .filter(|address| {
                desired
                    .as_ref()
                    .is_none_or(|desired| !desired.contains(address))
            })
            .collect()
    }
}

/// The immutable registry generation and whole-byte digest an invocation admitted.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RegistryRef {
    /// The registry generation.
    pub generation: Index,
    /// The digest of the complete registry bytes.
    pub digest: Digest,
}

/// The durable exclusion claim for one physical target.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LockClaim {
    /// The lock format.
    pub format: LockFormat,
    /// The invocation holding the claim.
    pub invocation: InvocationId,
    /// The selected authority.
    pub authority_id: Uuid,
    /// The selected authority's revision.
    pub authority_revision: Index,
    /// The pinned physical target.
    pub target: TargetPin,
    /// The registry bytes this claim was published under.
    pub registry: RegistryRef,
}

/// The only admitted quiescence statement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QuiescenceStatement {
    /// The named claim's executor, its descendants and its outstanding requests can write no more.
    NoFurtherWrites,
}

/// An administrative decision naming one exact retained claim.
///
/// It comes from the independently controlled authority source. A journal record cannot generate
/// it, and it grants nothing beyond the claim it names.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QuiescenceDecision {
    /// The complete retained claim.
    pub claim: LockClaim,
    /// The digest of that claim's exact retained bytes.
    pub claim_digest: Digest,
    /// The statement.
    pub statement: QuiescenceStatement,
}

/// One caller-provisioned authority revision.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Authority {
    /// The authority format.
    pub format: AuthorityFormat,
    /// The execution profile.
    pub profile: Profile,
    /// This authority's identity.
    pub authority_id: Uuid,
    /// This revision.
    pub revision: Index,
    /// The pinned physical cluster.
    pub target: TargetPin,
    /// The pinned principal.
    pub principal: PrincipalPin,
    /// The host, store and Helm policy.
    pub host: HostPolicy,
    /// The permitted kubeconfig context aliases.
    pub contexts: Vec<Text>,
    /// The environment this authority owns on its cluster.
    pub environment: Text,
    /// The exact canonical desired document digest.
    pub desired_digest: Digest,
    /// The exact canonical baseline document digest, absent for a first deployment.
    pub baseline_digest: Option<Digest>,
    /// The release permits, one per service.
    pub releases: Vec<ReleasePermit>,
    /// Administrative quiescence decisions naming exact retained claims.
    pub quiescence: Vec<QuiescenceDecision>,
}

impl Authority {
    /// Admits everything internal to one authority revision.
    ///
    /// The cross-authority indexes — endpoint aliases, store aliases, address collisions between
    /// two permits of two authorities — belong to `authority.rs`, which sees the whole registry.
    /// An authority that passes here is not thereby selected.
    pub fn validate(&self) -> Admitted<()> {
        if self.contexts.is_empty() {
            return Err(invalid(format!(
                "authority {} names no kubeconfig context",
                self.authority_id
            )));
        }
        for pair in self.contexts.windows(2) {
            if pair[0] >= pair[1] {
                return Err(invalid(format!(
                    "authority {} must list contexts in ascending order without duplicates",
                    self.authority_id
                )));
            }
        }
        self.target.validate()?;
        self.host.validate()?;
        if self.releases.is_empty() {
            return Err(invalid(format!(
                "authority {} admits no release permit",
                self.authority_id
            )));
        }
        for pair in self.releases.windows(2) {
            if pair[0].service >= pair[1].service {
                return Err(invalid(format!(
                    "authority {} must list one permit per service in ascending service order",
                    self.authority_id
                )));
            }
        }
        for permit in &self.releases {
            permit.validate()?;
        }
        let has_baseline = self.releases.iter().any(|permit| permit.baseline.is_some());
        if has_baseline && self.baseline_digest.is_none() {
            return Err(Refusal::new(
                RefusalCode::BaselineMismatch,
                format!(
                    "authority {} admits a baseline projection but pins no baseline digest",
                    self.authority_id
                ),
            ));
        }
        Ok(())
    }

    /// The permit for `service`, if this authority admits one.
    pub fn permit(&self, service: &str) -> Option<&ReleasePermit> {
        self.releases
            .iter()
            .find(|permit| permit.service.as_str() == service)
    }
}

/// The preprovisioned store's header.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StoreHeader {
    /// The store format.
    pub format: StoreFormat,
    /// The caller-provisioned epoch.
    pub store_epoch: Uuid,
    /// The host this store belongs to.
    pub host_id: Text,
    /// The physical target this store's lock namespace excludes on.
    pub target: TargetPin,
}

/// The complete active registry snapshot.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuthorityRegistry {
    /// The registry format.
    pub format: RegistryFormat,
    /// This snapshot's generation.
    pub generation: Index,
    /// Every active authority, sorted by authority UUID.
    pub authorities: Vec<Authority>,
}

/// What an invocation opened with.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InvocationContext {
    /// The selected authority revision, embedded whole.
    pub authority: Authority,
    /// The registry bytes it was selected from.
    pub registry: RegistryRef,
    /// An optional reference to a predecessor invocation. Never a selector over history.
    pub retry_of: Option<InvocationId>,
    /// The complete ordered selected service list: applies first, retirements in reverse order.
    pub selected: Vec<Text>,
    /// Whether the reviewed removal set was authorized.
    pub allow_removals: bool,
}

/// Which side of a mutation an observation was taken on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ObservationPhase {
    /// Before the mutation, and the only phase that can authorize one.
    Before,
    /// After an acknowledged mutation.
    After,
}

/// One complete observation of one operation at one invocation-local interval.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Observation {
    /// The operation index within this invocation's selected list.
    pub operation: Index,
    /// Which side of the mutation this is.
    pub phase: ObservationPhase,
    /// Invocation-local monotonic milliseconds at which acquisition started.
    pub started_ms: Index,
    /// Invocation-local monotonic milliseconds at which acquisition finished.
    pub finished_ms: Index,
    /// The complete snapshot.
    pub snapshot: ReleaseSnapshot,
}

impl Observation {
    /// Admits `started_ms <= finished_ms`.
    pub fn validate(&self) -> Admitted<()> {
        if self.started_ms > self.finished_ms {
            return Err(Refusal::new(
                RefusalCode::EvidenceIncomplete,
                "an observation cannot finish before it started",
            ));
        }
        Ok(())
    }
}

/// The freshness budget from `Observation::started_ms` to the final pre-launch check.
pub const FRESHNESS_BUDGET_MS: u64 = 30_000;

/// A durable decision to mutate, naming the exact `Observed` entry that authorizes it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Prepared {
    /// The operation index.
    pub operation: Index,
    /// The sequence of the `Observed` entry this decision rests on.
    pub observation_sequence: Index,
}

/// What can be established about a launched, refused or uncertain process.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProcessDisposition {
    /// Absence of launch is established: definite spawn refusal or definite prelaunch cancel.
    NotLaunched,
    /// The process returned success to this invocation. Historical process evidence only.
    Acknowledged,
    /// The process may have run and its effect cannot be established.
    Indeterminate,
}

/// One operation's durable process outcome.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProcessOutcome {
    /// The operation index.
    pub operation: Index,
    /// What was established.
    pub disposition: ProcessDisposition,
}

/// A settled stopping decision. Not evidence that every target effect is known.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Stopped {
    /// The operation the invocation stopped at, absent when it stopped before selecting one.
    pub operation: Option<Index>,
    /// The closed refusal code.
    pub reason: RefusalCode,
}

/// One durable fact.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", deny_unknown_fields)]
pub enum JournalFact {
    /// The invocation's context. Exactly one, at sequence zero.
    ///
    /// Boxed because it embeds the whole selected authority revision, which is an order of
    /// magnitude larger than any other fact; unboxed, every journal entry in memory would carry
    /// that much whether or not it is an `Opened`.
    Opened(Box<InvocationContext>),
    /// One complete observation.
    Observed(Observation),
    /// One durable decision to mutate.
    Prepared(Prepared),
    /// One durable process outcome.
    ProcessOutcome(ProcessOutcome),
    /// The terminal stopping fact.
    Stopped(Stopped),
    /// The terminal completion fact: the count of selected operations accounted for.
    Completed(Index),
}

impl JournalFact {
    /// Whether this fact closes a journal.
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Stopped(_) | Self::Completed(_))
    }

    /// The fact's discriminant name, for a diagnostic that must not print a payload.
    pub fn name(&self) -> &'static str {
        match self {
            Self::Opened(_) => "Opened",
            Self::Observed(_) => "Observed",
            Self::Prepared(_) => "Prepared",
            Self::ProcessOutcome(_) => "ProcessOutcome",
            Self::Stopped(_) => "Stopped",
            Self::Completed(_) => "Completed",
        }
    }
}

/// One immutable canonical journal file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct JournalEntry {
    /// The journal format.
    pub format: JournalFormat,
    /// The invocation this entry belongs to.
    pub invocation: InvocationId,
    /// The contiguous sequence number, zero for `Opened`.
    pub sequence: Index,
    /// The digest of the previous entry's exact bytes, absent only at sequence zero.
    pub previous_digest: Option<Digest>,
    /// The fact.
    pub fact: JournalFact,
}

/// A canonical stable Helm 3 version, as `recovery.execution.HelmVersion`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(transparent)]
pub struct HelmVersion(String);

impl HelmVersion {
    /// Admits exactly `v3.M.P` with decimal components and no prerelease or build suffix.
    pub fn new(value: impl Into<String>) -> Admitted<Self> {
        let value = value.into();
        let refuse = || {
            invalid(format!(
                "Helm version {value:?} is not a canonical stable v3.M.P version"
            ))
        };
        let rest = value.strip_prefix("v3.").ok_or_else(refuse)?;
        let (minor, patch) = rest.split_once('.').ok_or_else(refuse)?;
        for component in [minor, patch] {
            if component.is_empty()
                || !component.bytes().all(|byte| byte.is_ascii_digit())
                || (component.len() > 1 && component.starts_with('0'))
            {
                return Err(refuse());
            }
        }
        Ok(Self(value))
    }

    /// The exact admitted spelling.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for HelmVersion {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for HelmVersion {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        Self::new(value).map_err(de::Error::custom)
    }
}

/// The only admitted Helm protocol.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HelmProtocol {
    /// Helm 3, Secret-backed storage, the closed capability contract of C06.
    Helm3Recovery1,
}

/// The exact admitted Helm artifact.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HelmBinary {
    /// The absolute installation path.
    pub path: Text,
    /// The SHA-256 digest of the entire executable.
    pub digest: Digest,
    /// The exact version string.
    pub version: HelmVersion,
    /// The closed protocol.
    pub protocol: HelmProtocol,
}

/// The required installation prefix for an admitted Helm artifact.
pub const HELM_INSTALL_PREFIX: &str = "/opt/ess/recovery-tools/helm/";

impl HelmBinary {
    /// Admits the agreement of the installation directory's name and the configured digest.
    ///
    /// This is the declaration half. Whether the artifact sits under the *admitted prefix* is
    /// [`HelmBinary::admitted_under`], because the prefix is a deployment arrangement rather than
    /// a property of the record; and whether the file on disk hashes to `digest`, is regular, is
    /// unwritable and has an unwritable parent chain belongs to `process.rs`, which reads it.
    pub fn validate(&self) -> Admitted<()> {
        let path = self.path.as_str();
        absolute_path(path, "helm path")?;
        let hex = self.digest.as_str().trim_start_matches("sha256:");
        let tail = format!("/{hex}/helm");
        if !path.ends_with(&tail) {
            return Err(Refusal::new(
                RefusalCode::UnsupportedProfile,
                format!(
                    "an admitted Helm artifact is installed at <prefix>{tail}, and this one is at                      {path:?}"
                ),
            ));
        }
        Ok(())
    }

    /// Admits the artifact's exact installation path under one admitted prefix.
    ///
    /// The prefix is a construction parameter rather than a flag: the shipped binary passes
    /// [`HELM_INSTALL_PREFIX`], and a test arrangement that cannot write to `/opt` passes its own
    /// root. Nothing parses a prefix out of caller input.
    pub fn admitted_under(&self, prefix: &str) -> Admitted<()> {
        self.validate()?;
        let hex = self.digest.as_str().trim_start_matches("sha256:");
        let expected = format!("{prefix}{hex}/helm");
        if self.path.as_str() != expected {
            return Err(Refusal::new(
                RefusalCode::UnsupportedProfile,
                format!(
                    "an admitted Helm artifact is installed at {expected:?}, not {:?}",
                    self.path
                ),
            ));
        }
        Ok(())
    }
}

// --- Canonical bytes -------------------------------------------------------------------------

/// Reads canonical JSON, refusing a duplicate key and a floating-point number.
///
/// `serde_json` accepts both by default — the last duplicate wins and a float parses — and both
/// are exactly what a canonical byte contract cannot admit: two byte sequences that read as one
/// document, and a number whose readback is not its write.
pub fn canonical_value(text: &str) -> Admitted<serde_json::Value> {
    let mut deserializer = serde_json::Deserializer::from_str(text);
    let strict =
        Strict::deserialize(&mut deserializer).map_err(|error| invalid(format!("{error}")))?;
    deserializer.end().map_err(|error| {
        invalid(format!(
            "trailing bytes after a canonical document: {error}"
        ))
    })?;
    Ok(strict.0)
}

/// Reads exactly canonical bytes into `T`: strict JSON, then the canonical byte comparison.
///
/// The comparison is what makes the bytes canonical rather than merely valid. Two spellings that
/// both parse are two documents with two digests, and the digest is what the registry, the claim
/// and the journal chain all compare.
pub fn read_canonical<T>(text: &str) -> Admitted<T>
where
    T: Serialize + for<'de> Deserialize<'de>,
{
    let value = canonical_value(text)?;
    let parsed: T = serde_json::from_value(value).map_err(|error| invalid(format!("{error}")))?;
    let canonical = write_canonical(&parsed);
    if canonical != text {
        return Err(invalid(
            "the document is valid but is not the canonical byte spelling of itself",
        ));
    }
    Ok(parsed)
}

/// Writes canonical bytes: sorted keys, preserved array order, one trailing newline.
pub fn write_canonical<T: Serialize>(value: &T) -> String {
    let mut text = serde_json::to_string(value).expect("a typed recovery record always serializes");
    text.push('\n');
    text
}

/// The digest of a value's canonical bytes.
pub fn canonical_digest<T: Serialize>(value: &T) -> Digest {
    Digest::of_bytes(write_canonical(value).as_bytes())
}

/// A `serde_json::Value` that refuses a duplicate object key and a floating-point number.
struct Strict(serde_json::Value);

impl<'de> Deserialize<'de> for Strict {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_any(StrictVisitor).map(Strict)
    }
}

struct StrictVisitor;

impl<'de> Visitor<'de> for StrictVisitor {
    type Value = serde_json::Value;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a canonical JSON document")
    }

    fn visit_unit<E: de::Error>(self) -> Result<Self::Value, E> {
        Ok(serde_json::Value::Null)
    }

    fn visit_bool<E: de::Error>(self, value: bool) -> Result<Self::Value, E> {
        Ok(serde_json::Value::Bool(value))
    }

    fn visit_i64<E: de::Error>(self, value: i64) -> Result<Self::Value, E> {
        Ok(serde_json::Value::from(value))
    }

    fn visit_u64<E: de::Error>(self, value: u64) -> Result<Self::Value, E> {
        Ok(serde_json::Value::from(value))
    }

    fn visit_f64<E: de::Error>(self, _: f64) -> Result<Self::Value, E> {
        Err(de::Error::custom(
            "a canonical recovery document has no floating-point field",
        ))
    }

    fn visit_str<E: de::Error>(self, value: &str) -> Result<Self::Value, E> {
        Ok(serde_json::Value::String(value.to_owned()))
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut access: A) -> Result<Self::Value, A::Error> {
        let mut items = Vec::new();
        while let Some(Strict(item)) = access.next_element()? {
            items.push(item);
        }
        Ok(serde_json::Value::Array(items))
    }

    fn visit_map<A: MapAccess<'de>>(self, mut access: A) -> Result<Self::Value, A::Error> {
        let mut entries: BTreeMap<String, serde_json::Value> = BTreeMap::new();
        while let Some(key) = access.next_key::<String>()? {
            let Strict(value) = access.next_value()?;
            if entries.insert(key.clone(), value).is_some() {
                return Err(de::Error::custom(format!("duplicate object key {key:?}")));
            }
        }
        Ok(serde_json::Value::Object(entries.into_iter().collect()))
    }
}
