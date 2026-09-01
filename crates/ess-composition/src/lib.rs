//! Validated composition of selected components from independently compiled ESS models.
//!
//! A service-local [`EssIr`] owns compiler handles that cannot cross into another compilation.
//! This crate keeps that boundary intact: a composition names a service with [`ServiceKey`] and a
//! construct with [`EssSemanticRef`], then resolves that pair against the IR registered for exactly
//! that key. The result is a compiler-minted [`EssCompositionIr`] containing stable names only.
//!
//! Every import selects one exact ESS component. Commands come only from that component's
//! `accepts` surface; queries come only from views in domains it owns. Published or emitted events,
//! command errors, and recursively referenced named types form the closed dependency surface.
//! Nothing elsewhere in the same model leaks into composition IR or generated clients.
//!
//! The persisted input format is [`COMPOSITION_FORMAT`]. Generated clients consume the derived
//! [`EssClientPlan`] rather than reinterpreting multiple service models independently.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::fmt::Write as _;
use std::str::FromStr;

use ess_compiler::ir::{ResolvedBody, ResolvedField, ResolvedTypeRef, TypeHandle};
use ess_compiler::refs::{
    CommandRef, ComponentRef, DeclaredTypeRef, ErrorRef, EssSemanticRef, EventRef, ViewRef,
};
use ess_compiler::EssIr;
use ess_domain::name::{QualifiedName, Version};

/// The only composition document format this reader understands.
pub const COMPOSITION_FORMAT: &str = "ess-composition/1";

/// The language-neutral client-plan format emitted from composition IR.
pub const CLIENT_PLAN_FORMAT: &str = "ess-client-plan/1";

/// A stable, composition-local service identity.
///
/// Keys are lowercase path-safe segments separated by `.`, `_`, or `-`. They are not endpoint
/// paths and carry no tenant or realm coordinate.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize)]
#[serde(transparent)]
pub struct ServiceKey(String);

impl ServiceKey {
    /// Parses and validates a service key.
    pub fn new(value: impl AsRef<str>) -> Result<Self, KeyError> {
        let value = value.as_ref();
        let mut previous_separator = false;
        let valid = !value.is_empty()
            && value
                .chars()
                .next()
                .is_some_and(|character| character.is_ascii_lowercase())
            && value.chars().all(|character| {
                character.is_ascii_lowercase()
                    || character.is_ascii_digit()
                    || matches!(character, '.' | '_' | '-')
            })
            && value.chars().all(|character| {
                let separator = matches!(character, '.' | '_' | '-');
                let accepted = !(separator && previous_separator);
                previous_separator = separator;
                accepted
            })
            && !previous_separator;

        if valid {
            Ok(Self(value.to_owned()))
        } else {
            Err(KeyError(value.to_owned()))
        }
    }

    /// The validated key.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ServiceKey {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl FromStr for ServiceKey {
    type Err = KeyError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::new(value)
    }
}

impl<'de> serde::Deserialize<'de> for ServiceKey {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        Self::new(value).map_err(serde::de::Error::custom)
    }
}

/// A malformed [`ServiceKey`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyError(String);

impl fmt::Display for KeyError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "invalid service key {:?}: expected lowercase segments separated by `.`, `_`, or `-`",
            self.0
        )
    }
}

impl std::error::Error for KeyError {}

/// A full lowercase SHA-256 digest.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize)]
#[serde(transparent)]
pub struct SourceDigest(String);

impl SourceDigest {
    /// Parses an exact lowercase SHA-256 digest.
    pub fn new(value: impl AsRef<str>) -> Result<Self, DigestError> {
        let value = value.as_ref();
        if value.len() == 64
            && value
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        {
            Ok(Self(value.to_owned()))
        } else {
            Err(DigestError(value.to_owned()))
        }
    }

    /// Derives the digest from compiler-owned semantic bytes.
    pub fn of(ir: &EssIr) -> Self {
        Self(ir.source_digest())
    }

    /// The 64 lowercase hexadecimal characters.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for SourceDigest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl FromStr for SourceDigest {
    type Err = DigestError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::new(value)
    }
}

impl<'de> serde::Deserialize<'de> for SourceDigest {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        Self::new(value).map_err(serde::de::Error::custom)
    }
}

/// A malformed [`SourceDigest`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DigestError(String);

impl fmt::Display for DigestError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "invalid source digest {:?}: expected 64 lowercase hexadecimal characters",
            self.0
        )
    }
}

impl std::error::Error for DigestError {}

/// One imported component surface as the composition document binds it.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ServiceImportSpec {
    key: ServiceKey,
    system: QualifiedName,
    version: Version,
    source_digest: SourceDigest,
    component: ComponentRef,
}

impl ServiceImportSpec {
    /// Declares an exact model-and-component binding read from a release or composition lock.
    pub const fn new(
        key: ServiceKey,
        system: QualifiedName,
        version: Version,
        source_digest: SourceDigest,
        component: ComponentRef,
    ) -> Self {
        Self {
            key,
            system,
            version,
            source_digest,
            component,
        }
    }

    /// Binds one selected component from an exact compiled model.
    pub fn of(key: ServiceKey, component: ComponentRef, ir: &EssIr) -> Self {
        Self {
            key,
            system: ir.system().clone(),
            version: *ir.version(),
            source_digest: SourceDigest::of(ir),
            component,
        }
    }

    /// The composition-local identity.
    pub fn key(&self) -> &ServiceKey {
        &self.key
    }

    /// The ESS system identity expected at this key.
    pub fn system(&self) -> &QualifiedName {
        &self.system
    }

    /// The ESS specification version expected at this key.
    pub const fn version(&self) -> Version {
        self.version
    }

    /// The exact semantic digest expected at this key.
    pub fn source_digest(&self) -> &SourceDigest {
        &self.source_digest
    }

    /// The exact component whose declared outer surface is imported.
    pub fn component(&self) -> &ComponentRef {
        &self.component
    }
}

/// An exported semantic name qualified by the selected service component that owns it.
#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
#[serde(deny_unknown_fields)]
pub struct CompositionRef {
    service: ServiceKey,
    semantic: EssSemanticRef,
}

impl CompositionRef {
    /// Qualifies a service-local semantic name.
    pub fn new(service: ServiceKey, semantic: EssSemanticRef) -> Self {
        Self { service, semantic }
    }

    /// The service whose compiler namespace owns the reference.
    pub fn service(&self) -> &ServiceKey {
        &self.service
    }

    /// The stable ESS semantic name.
    pub fn semantic(&self) -> &EssSemanticRef {
        &self.semantic
    }
}

/// The human-authored persisted composition input.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CompositionSpec {
    format: String,
    composition: ServiceKey,
    services: Vec<ServiceImportSpec>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    references: Vec<CompositionRef>,
}

impl CompositionSpec {
    /// Creates a v1 composition input. Semantic validation happens in [`compile`].
    pub fn new(
        composition: ServiceKey,
        services: Vec<ServiceImportSpec>,
        references: Vec<CompositionRef>,
    ) -> Self {
        Self {
            format: COMPOSITION_FORMAT.to_owned(),
            composition,
            services,
            references,
        }
    }

    /// Reads a JSON composition and rejects every unknown field.
    pub fn from_json(text: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(text)
    }

    /// Reads a YAML composition and rejects every unknown field.
    pub fn from_yaml(text: &str) -> Result<Self, serde_yaml::Error> {
        serde_yaml::from_str(text)
    }

    /// The format marker found in the input.
    pub fn format(&self) -> &str {
        &self.format
    }

    /// The stable composition identity.
    pub fn composition(&self) -> &ServiceKey {
        &self.composition
    }

    /// Imported service bindings, before duplicate checks.
    pub fn services(&self) -> &[ServiceImportSpec] {
        &self.services
    }

    /// Cross-service semantic names, before resolution.
    pub fn references(&self) -> &[CompositionRef] {
        &self.references
    }

    /// Canonical JSON with a trailing newline.
    pub fn to_canonical_json(&self) -> String {
        canonical_json(self)
    }
}

/// One compiler-owned ESS service supplied to [`compile`].
#[derive(Debug, Clone, Copy)]
pub struct CompiledService<'a> {
    key: &'a ServiceKey,
    ir: &'a EssIr,
}

impl<'a> CompiledService<'a> {
    /// Associates a compiled service with its composition key.
    pub const fn new(key: &'a ServiceKey, ir: &'a EssIr) -> Self {
        Self { key, ir }
    }
}

/// A stable diagnostic category produced while compiling a composition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CompositionCode {
    /// The document's format marker is unsupported.
    UnsupportedFormat,
    /// More than one import declares the same service key.
    DuplicateServiceKey,
    /// More than one import binds the same ESS system, version, and component.
    DuplicateServiceIdentity,
    /// No compiled input was supplied for a declared import.
    MissingServiceInput,
    /// A compiled input was supplied but the document does not import it.
    UndeclaredServiceInput,
    /// A registry supplied the same key more than once.
    DuplicateServiceInput,
    /// The compiled service's system differs from the document.
    SystemMismatch,
    /// The compiled service's version differs from the document.
    VersionMismatch,
    /// The compiled service's semantic digest differs from the document.
    DigestMismatch,
    /// A semantic reference names a service the document does not import.
    UnknownReferenceService,
    /// A semantic name does not resolve in the service selected by its key.
    UnresolvedSemanticReference,
    /// The exact imported ESS model does not declare the selected component.
    UnknownComponent,
    /// A semantic name exists in the model but is outside the selected component surface.
    ReferenceOutsideComponent,
}

impl fmt::Display for CompositionCode {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let code = match self {
            Self::UnsupportedFormat => "unsupported_format",
            Self::DuplicateServiceKey => "duplicate_service_key",
            Self::DuplicateServiceIdentity => "duplicate_service_identity",
            Self::MissingServiceInput => "missing_service_input",
            Self::UndeclaredServiceInput => "undeclared_service_input",
            Self::DuplicateServiceInput => "duplicate_service_input",
            Self::SystemMismatch => "system_mismatch",
            Self::VersionMismatch => "version_mismatch",
            Self::DigestMismatch => "digest_mismatch",
            Self::UnknownReferenceService => "unknown_reference_service",
            Self::UnresolvedSemanticReference => "unresolved_semantic_reference",
            Self::UnknownComponent => "unknown_component",
            Self::ReferenceOutsideComponent => "reference_outside_component",
        };
        formatter.write_str(code)
    }
}

/// One repair-oriented composition diagnostic.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct CompositionDiagnostic {
    code: CompositionCode,
    #[serde(skip_serializing_if = "Option::is_none")]
    service: Option<ServiceKey>,
    detail: String,
}

impl CompositionDiagnostic {
    fn new(code: CompositionCode, service: Option<ServiceKey>, detail: impl Into<String>) -> Self {
        Self {
            code,
            service,
            detail: detail.into(),
        }
    }

    /// The stable machine-readable category.
    pub const fn code(&self) -> CompositionCode {
        self.code
    }

    /// The affected service, when the failure belongs to one.
    pub fn service(&self) -> Option<&ServiceKey> {
        self.service.as_ref()
    }

    /// The repair-oriented explanation.
    pub fn detail(&self) -> &str {
        &self.detail
    }
}

/// Every diagnostic from one composition attempt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompositionDiagnostics(Vec<CompositionDiagnostic>);

impl CompositionDiagnostics {
    /// The complete deterministic diagnostic list.
    pub fn as_slice(&self) -> &[CompositionDiagnostic] {
        &self.0
    }

    /// Whether one category occurred.
    pub fn contains(&self, code: CompositionCode) -> bool {
        self.0.iter().any(|diagnostic| diagnostic.code == code)
    }
}

impl fmt::Display for CompositionDiagnostics {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (index, diagnostic) in self.0.iter().enumerate() {
            if index > 0 {
                formatter.write_str("\n")?;
            }
            write!(formatter, "[{}] {}", diagnostic.code, diagnostic.detail)?;
        }
        Ok(())
    }
}

impl std::error::Error for CompositionDiagnostics {}

/// One selected component surface captured in compiler-minted composition IR.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct ResolvedService {
    system: QualifiedName,
    version: Version,
    source_digest: SourceDigest,
    component: ComponentRef,
    commands: BTreeSet<CommandRef>,
    queries: BTreeSet<ViewRef>,
    events: BTreeSet<EventRef>,
    errors: BTreeSet<ErrorRef>,
    types: BTreeSet<DeclaredTypeRef>,
}

impl ResolvedService {
    /// The ESS system identity.
    pub fn system(&self) -> &QualifiedName {
        &self.system
    }

    /// The ESS specification version.
    pub const fn version(&self) -> Version {
        self.version
    }

    /// The exact semantic source digest.
    pub fn source_digest(&self) -> &SourceDigest {
        &self.source_digest
    }

    /// The selected component, never the whole imported model.
    pub fn component(&self) -> &ComponentRef {
        &self.component
    }

    /// Command operations exported by the service.
    pub fn commands(&self) -> &BTreeSet<CommandRef> {
        &self.commands
    }

    /// Query operations exported from ESS views.
    pub fn queries(&self) -> &BTreeSet<ViewRef> {
        &self.queries
    }

    /// Events published by or emitted from selected commands.
    pub fn events(&self) -> &BTreeSet<EventRef> {
        &self.events
    }

    /// Error payload contracts reachable from selected commands.
    pub fn errors(&self) -> &BTreeSet<ErrorRef> {
        &self.errors
    }

    /// Complete recursive named-type closure of commands, queries, events, and errors.
    pub fn types(&self) -> &BTreeSet<DeclaredTypeRef> {
        &self.types
    }

    fn exports(&self, reference: &EssSemanticRef) -> bool {
        match reference {
            EssSemanticRef::Command { name } => self.commands.contains(name),
            EssSemanticRef::Outcome { name } => self.commands.contains(&name.command),
            EssSemanticRef::Event { name } => self.events.contains(name),
            EssSemanticRef::Error { name } => self.errors.contains(name),
            EssSemanticRef::View { name } => self.queries.contains(name),
            EssSemanticRef::Type { name } => self.types.contains(name),
            EssSemanticRef::Component { name } => name == &self.component,
            EssSemanticRef::Domain { .. }
            | EssSemanticRef::Entity { .. }
            | EssSemanticRef::Actor { .. }
            | EssSemanticRef::Transition { .. }
            | EssSemanticRef::Binding { .. } => false,
        }
    }
}

/// The validated, compiler-minted composition of exact ESS services.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct EssCompositionIr {
    format: String,
    composition: ServiceKey,
    services: BTreeMap<ServiceKey, ResolvedService>,
    #[serde(skip_serializing_if = "BTreeSet::is_empty")]
    references: BTreeSet<CompositionRef>,
}

impl EssCompositionIr {
    /// The persisted format marker.
    pub fn format(&self) -> &str {
        &self.format
    }

    /// The stable composition identity.
    pub fn composition(&self) -> &ServiceKey {
        &self.composition
    }

    /// Every imported service, ordered by stable key.
    pub fn services(&self) -> &BTreeMap<ServiceKey, ResolvedService> {
        &self.services
    }

    /// Every resolved cross-service semantic name.
    pub fn references(&self) -> &BTreeSet<CompositionRef> {
        &self.references
    }

    /// Canonical JSON with a trailing newline.
    pub fn to_canonical_json(&self) -> String {
        canonical_json(self)
    }

    /// Derives the transport-neutral command/query surface for client generators.
    pub fn client_plan(&self) -> EssClientPlan {
        let services = self
            .services
            .iter()
            .map(|(key, service)| {
                (
                    key.clone(),
                    ClientServicePlan {
                        system: service.system.clone(),
                        version: service.version,
                        source_digest: service.source_digest.clone(),
                        component: service.component.clone(),
                        commands: service.commands.clone(),
                        queries: service.queries.clone(),
                        events: service.events.clone(),
                        errors: service.errors.clone(),
                        types: service.types.clone(),
                    },
                )
            })
            .collect();

        EssClientPlan {
            format: CLIENT_PLAN_FORMAT.to_owned(),
            composition: self.composition.clone(),
            endpoint_provider: ClientProviderBinding::Injected,
            authority_provider: ClientProviderBinding::Injected,
            services,
        }
    }
}

/// How generated clients obtain an environment-specific value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ClientProviderBinding {
    /// The application supplies the provider at construction time.
    Injected,
}

/// One service namespace in a generated client plan.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct ClientServicePlan {
    system: QualifiedName,
    version: Version,
    source_digest: SourceDigest,
    component: ComponentRef,
    commands: BTreeSet<CommandRef>,
    queries: BTreeSet<ViewRef>,
    events: BTreeSet<EventRef>,
    errors: BTreeSet<ErrorRef>,
    types: BTreeSet<DeclaredTypeRef>,
}

impl ClientServicePlan {
    /// The service's ESS system identity.
    pub fn system(&self) -> &QualifiedName {
        &self.system
    }

    /// The service's ESS specification version.
    pub const fn version(&self) -> Version {
        self.version
    }

    /// The exact source digest the client surface derives from.
    pub fn source_digest(&self) -> &SourceDigest {
        &self.source_digest
    }

    /// The selected component whose surface these operations describe.
    pub fn component(&self) -> &ComponentRef {
        &self.component
    }

    /// Commands generated under this service namespace.
    pub fn commands(&self) -> &BTreeSet<CommandRef> {
        &self.commands
    }

    /// Queries generated under this service namespace.
    pub fn queries(&self) -> &BTreeSet<ViewRef> {
        &self.queries
    }

    /// Events reachable from the selected component surface.
    pub fn events(&self) -> &BTreeSet<EventRef> {
        &self.events
    }

    /// Errors reachable from the selected component commands.
    pub fn errors(&self) -> &BTreeSet<ErrorRef> {
        &self.errors
    }

    /// Recursive type closure required by the selected surface.
    pub fn types(&self) -> &BTreeSet<DeclaredTypeRef> {
        &self.types
    }
}

/// A language-neutral plan for composition client generators.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct EssClientPlan {
    format: String,
    composition: ServiceKey,
    endpoint_provider: ClientProviderBinding,
    authority_provider: ClientProviderBinding,
    services: BTreeMap<ServiceKey, ClientServicePlan>,
}

impl EssClientPlan {
    /// The client-plan format marker.
    pub fn format(&self) -> &str {
        &self.format
    }

    /// The composition whose namespaces this plan exposes.
    pub fn composition(&self) -> &ServiceKey {
        &self.composition
    }

    /// How the generated client obtains service endpoints.
    pub const fn endpoint_provider(&self) -> ClientProviderBinding {
        self.endpoint_provider
    }

    /// How the generated client obtains authentication authority.
    pub const fn authority_provider(&self) -> ClientProviderBinding {
        self.authority_provider
    }

    /// The services exposed as client namespaces.
    pub fn services(&self) -> &BTreeMap<ServiceKey, ClientServicePlan> {
        &self.services
    }

    /// Canonical JSON with a trailing newline.
    pub fn to_canonical_json(&self) -> String {
        canonical_json(self)
    }

    /// Emits a dependency-free Rust client workspace around injected endpoint, authority, and
    /// transport providers.
    ///
    /// The generated operations are only the commands and queries exported by each selected
    /// component. Authentication context remains inside the injected authority provider; tenant,
    /// realm, user, and executor are never generated as operation operands.
    pub fn rust_artifacts(&self) -> BTreeMap<String, ClientArtifact> {
        let mut artifacts = BTreeMap::new();
        insert_client_artifact(
            &mut artifacts,
            "ess-client-plan.json",
            self.to_canonical_json(),
        );
        insert_client_artifact(
            &mut artifacts,
            "Cargo.toml",
            rust_client_manifest(self.composition()),
        );
        insert_client_artifact(&mut artifacts, "src/lib.rs", rust_client_library(self));
        artifacts
    }
}

/// One deterministic file emitted from an [`EssClientPlan`].
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct ClientArtifact {
    path: String,
    contents: String,
}

impl ClientArtifact {
    /// Slash-separated path relative to the generated client root.
    pub fn path(&self) -> &str {
        &self.path
    }

    /// Complete UTF-8 file contents.
    pub fn contents(&self) -> &str {
        &self.contents
    }
}

fn insert_client_artifact(
    artifacts: &mut BTreeMap<String, ClientArtifact>,
    path: &str,
    contents: String,
) {
    let previous = artifacts.insert(
        path.to_owned(),
        ClientArtifact {
            path: path.to_owned(),
            contents,
        },
    );
    assert!(
        previous.is_none(),
        "client artifact paths are statically unique"
    );
}

fn rust_client_manifest(composition: &ServiceKey) -> String {
    let package = composition
        .as_str()
        .chars()
        .map(|character| match character {
            '.' | '_' => '-',
            other => other,
        })
        .collect::<String>();
    format!(
        "# Generated from ess-client-plan/1; do not edit.\n\
         [package]\n\
         name = \"{package}-ess-client\"\n\
         version = \"0.0.0\"\n\
         edition = \"2021\"\n\
         publish = false\n\
         \n\
         [lib]\n\
         path = \"src/lib.rs\"\n"
    )
}

// One contiguous scaffold makes the generated Rust contract reviewable in its emitted order.
#[allow(clippy::too_many_lines)]
fn rust_client_library(plan: &EssClientPlan) -> String {
    let mut output = String::new();
    let _ = writeln!(
        output,
        "//! Generated composition client for `{}`.\n//!\n//! Endpoints, verified authority, and transport are injected. Operation inputs never contain\n//! authentication coordinates.\n\n#![forbid(unsafe_code)]\n",
        plan.composition
    );
    let _ = writeln!(
        output,
        "/// Stable composition identity.\npub const COMPOSITION: &str = {:?};\n",
        plan.composition.as_str()
    );
    output.push_str(
        r#"/// A selected ESS component and its exact semantic dependency closure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Service {
    key: &'static str,
    system: &'static str,
    version: &'static str,
    source_digest: &'static str,
    component: &'static str,
    types: &'static [&'static str],
    events: &'static [&'static str],
    errors: &'static [&'static str],
}

impl Service {
    /// Composition-local service key.
    pub const fn key(self) -> &'static str { self.key }
    /// Exact ESS system identity.
    pub const fn system(self) -> &'static str { self.system }
    /// Exact ESS specification version.
    pub const fn version(self) -> &'static str { self.version }
    /// Exact compiler-owned semantic source digest.
    pub const fn source_digest(self) -> &'static str { self.source_digest }
    /// Selected ESS component.
    pub const fn component(self) -> &'static str { self.component }
    /// Recursive named-type closure required by the client surface.
    pub const fn types(self) -> &'static [&'static str] { self.types }
    /// Event contracts required by the client surface.
    pub const fn events(self) -> &'static [&'static str] { self.events }
    /// Error contracts required by the client surface.
    pub const fn errors(self) -> &'static [&'static str] { self.errors }
}

/// The two callable ESS surface kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperationKind {
    /// An intent represented by an ESS command.
    Command,
    /// A query represented by an ESS view.
    Query,
}

/// One unforgeable generated operation descriptor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Operation {
    service_key: &'static str,
    semantic: &'static str,
    kind: OperationKind,
}

impl Operation {
    const fn new(service_key: &'static str, semantic: &'static str, kind: OperationKind) -> Self {
        Self { service_key, semantic, kind }
    }
    /// Composition-local service key.
    pub const fn service_key(self) -> &'static str { self.service_key }
    /// Fully qualified ESS command or view name.
    pub const fn semantic(self) -> &'static str { self.semantic }
    /// Whether this is a command or query.
    pub const fn kind(self) -> OperationKind { self.kind }
}

/// Supplies an environment endpoint for one exact selected service.
pub trait EndpointProvider {
    /// Returns the endpoint or `None` when this environment has no binding.
    fn endpoint(&self, service: &Service) -> Option<&str>;
}

/// Supplies verified authentication authority at execution time.
pub trait AuthorityProvider {
    /// Application-owned verified authority type.
    type Authority;
    /// Current verified authority, including any optional realm internally.
    fn authority(&self) -> &Self::Authority;
}

/// Executes encoded operation payloads over an application-selected protocol.
pub trait Transport<Authority> {
    /// Transport or remote-service failure.
    type Error;
    /// Executes one generated operation.
    fn execute(
        &self,
        endpoint: &str,
        authority: &Authority,
        operation: Operation,
        payload: &[u8],
    ) -> Result<Vec<u8>, Self::Error>;
}

/// Failure before or during generated client execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClientError<E> {
    /// The environment did not bind the selected service.
    MissingEndpoint(&'static str),
    /// The injected transport failed.
    Transport(E),
}

/// Composition client with all environmental decisions injected.
pub struct Client<Endpoints, Authority, Wire> {
    endpoints: Endpoints,
    authority: Authority,
    wire: Wire,
}

impl<Endpoints, Authority, Wire> Client<Endpoints, Authority, Wire> {
    /// Binds providers without performing I/O.
    pub const fn new(endpoints: Endpoints, authority: Authority, wire: Wire) -> Self {
        Self { endpoints, authority, wire }
    }
}

impl<Endpoints, Authority, Wire> Client<Endpoints, Authority, Wire>
where
    Endpoints: EndpointProvider,
    Authority: AuthorityProvider,
    Wire: Transport<Authority::Authority>,
{
    /// Executes one generated command or query with an encoded domain payload.
    pub fn execute(
        &self,
        operation: Operation,
        payload: &[u8],
    ) -> Result<Vec<u8>, ClientError<Wire::Error>> {
        let service = service(operation.service_key)
            .expect("generated operations always name a generated service");
        let endpoint = self.endpoints.endpoint(service)
            .ok_or(ClientError::MissingEndpoint(operation.service_key))?;
        self.wire
            .execute(endpoint, self.authority.authority(), operation, payload)
            .map_err(ClientError::Transport)
    }
}

"#,
    );

    for (key, service) in &plan.services {
        render_rust_service_module(&mut output, key, service);
    }

    output.push_str("/// Looks up generated service metadata by composition-local key.\n");
    output.push_str("pub fn service(key: &str) -> Option<&'static Service> {\n    match key {\n");
    for key in plan.services.keys() {
        let _ = writeln!(
            output,
            "        {:?} => Some(&{}::SERVICE),",
            key.as_str(),
            rust_service_module(key)
        );
    }
    output.push_str("        _ => None,\n    }\n}\n");
    output
}

fn render_rust_service_module(output: &mut String, key: &ServiceKey, service: &ClientServicePlan) {
    let module = rust_service_module(key);
    let _ = writeln!(
        output,
        "/// Client surface for `{}` / component `{}`.\npub mod {module} {{\n    use super::{{Operation, OperationKind, Service}};",
        key, service.component
    );
    render_rust_str_slice(
        output,
        "TYPES",
        service.types.iter().map(ToString::to_string),
    );
    render_rust_str_slice(
        output,
        "EVENTS",
        service.events.iter().map(ToString::to_string),
    );
    render_rust_str_slice(
        output,
        "ERRORS",
        service.errors.iter().map(ToString::to_string),
    );
    let _ = writeln!(
        output,
        "    /// Exact selected service metadata.\n    pub const SERVICE: Service = Service {{\n        key: {:?},\n        system: {:?},\n        version: {:?},\n        source_digest: {:?},\n        component: {:?},\n        types: TYPES,\n        events: EVENTS,\n        errors: ERRORS,\n    }};",
        key.as_str(),
        service.system.to_string(),
        service.version.to_string(),
        service.source_digest.as_str(),
        service.component.to_string(),
    );

    let mut command_names = BTreeSet::new();
    for command in &service.commands {
        let identifier = operation_constant("COMMAND", command.name().local(), &mut command_names);
        let _ = writeln!(
            output,
            "    /// ESS command `{command}`.\n    pub const {identifier}: Operation = Operation::new({:?}, {:?}, OperationKind::Command);",
            key.as_str(),
            command.to_string()
        );
    }
    let mut query_names = BTreeSet::new();
    for query in &service.queries {
        let identifier = operation_constant("QUERY", query.name().local(), &mut query_names);
        let _ = writeln!(
            output,
            "    /// ESS view query `{query}`.\n    pub const {identifier}: Operation = Operation::new({:?}, {:?}, OperationKind::Query);",
            key.as_str(),
            query.to_string()
        );
    }
    output.push_str("}\n\n");
}

fn render_rust_str_slice(
    output: &mut String,
    name: &str,
    values: impl IntoIterator<Item = String>,
) {
    let _ = writeln!(
        output,
        "    /// Exact selected surface {name}.\n    pub const {name}: &[&str] = &["
    );
    for value in values {
        let _ = writeln!(output, "        {value:?},");
    }
    output.push_str("    ];\n");
}

fn operation_constant(prefix: &str, local: &str, used: &mut BTreeSet<String>) -> String {
    let mut base = format!("{prefix}_");
    let mut previous_lower_or_digit = false;
    for character in local.chars() {
        if character.is_ascii_uppercase() {
            if previous_lower_or_digit {
                base.push('_');
            }
            base.push(character);
            previous_lower_or_digit = false;
        } else if character.is_ascii_alphanumeric() {
            base.push(character.to_ascii_uppercase());
            previous_lower_or_digit = true;
        } else if !base.ends_with('_') {
            base.push('_');
            previous_lower_or_digit = false;
        }
    }
    let mut candidate = base.clone();
    let mut suffix = 2_u32;
    while !used.insert(candidate.clone()) {
        candidate = format!("{base}_{suffix}");
        suffix += 1;
    }
    candidate
}

fn rust_service_module(key: &ServiceKey) -> String {
    let mut module = String::from("service_");
    for character in key.as_str().chars() {
        match character {
            '.' => module.push_str("_dot_"),
            '-' => module.push_str("_dash_"),
            '_' => module.push_str("_underscore_"),
            other => module.push(other),
        }
    }
    module
}

/// Compiles one composition against exact model IRs and selected components supplied by the adopter.
///
/// Diagnostics accumulate and are ordered by document/registry order. No partially validated IR is
/// returned: [`EssCompositionIr`] is available only when every identity, digest and semantic name
/// agrees.
pub fn compile<'a>(
    specification: &CompositionSpec,
    services: impl IntoIterator<Item = CompiledService<'a>>,
) -> Result<EssCompositionIr, CompositionDiagnostics> {
    let mut diagnostics = Vec::new();
    validate_format(specification, &mut diagnostics);
    let registry = collect_registry(services, &mut diagnostics);
    let (declared_keys, resolved) = resolve_services(specification, &registry, &mut diagnostics);
    let references = resolve_references(
        specification,
        &registry,
        &declared_keys,
        &resolved,
        &mut diagnostics,
    );

    if diagnostics.is_empty() {
        Ok(EssCompositionIr {
            format: COMPOSITION_FORMAT.to_owned(),
            composition: specification.composition.clone(),
            services: resolved,
            references,
        })
    } else {
        Err(CompositionDiagnostics(diagnostics))
    }
}

fn validate_format(specification: &CompositionSpec, diagnostics: &mut Vec<CompositionDiagnostic>) {
    if specification.format != COMPOSITION_FORMAT {
        diagnostics.push(CompositionDiagnostic::new(
            CompositionCode::UnsupportedFormat,
            None,
            format!(
                "format {:?} is unsupported; expected {COMPOSITION_FORMAT}",
                specification.format
            ),
        ));
    }
}

fn collect_registry<'a>(
    services: impl IntoIterator<Item = CompiledService<'a>>,
    diagnostics: &mut Vec<CompositionDiagnostic>,
) -> BTreeMap<ServiceKey, &'a EssIr> {
    let mut registry = BTreeMap::new();
    for service in services {
        if registry.insert(service.key.clone(), service.ir).is_some() {
            diagnostics.push(CompositionDiagnostic::new(
                CompositionCode::DuplicateServiceInput,
                Some(service.key.clone()),
                format!(
                    "compiled service input `{}` was supplied more than once",
                    service.key
                ),
            ));
        }
    }
    registry
}

fn resolve_services(
    specification: &CompositionSpec,
    registry: &BTreeMap<ServiceKey, &EssIr>,
    diagnostics: &mut Vec<CompositionDiagnostic>,
) -> (BTreeSet<ServiceKey>, BTreeMap<ServiceKey, ResolvedService>) {
    let mut declared_keys = BTreeSet::new();
    let mut declared_identities = BTreeSet::new();
    let mut resolved = BTreeMap::new();
    for imported in &specification.services {
        validate_import_identity(
            imported,
            &mut declared_keys,
            &mut declared_identities,
            diagnostics,
        );
        let Some(ir) = registry.get(&imported.key).copied() else {
            diagnostics.push(CompositionDiagnostic::new(
                CompositionCode::MissingServiceInput,
                Some(imported.key.clone()),
                format!(
                    "no compiled ESS service was supplied for `{}`",
                    imported.key
                ),
            ));
            continue;
        };
        validate_import_contract(imported, ir, diagnostics);
        if let Some(service) = resolved_service(imported, ir, diagnostics) {
            resolved.entry(imported.key.clone()).or_insert(service);
        }
    }
    for key in registry.keys() {
        if !declared_keys.contains(key) {
            diagnostics.push(CompositionDiagnostic::new(
                CompositionCode::UndeclaredServiceInput,
                Some(key.clone()),
                format!("compiled service `{key}` is not imported by the composition"),
            ));
        }
    }
    (declared_keys, resolved)
}

fn validate_import_identity(
    imported: &ServiceImportSpec,
    declared_keys: &mut BTreeSet<ServiceKey>,
    declared_identities: &mut BTreeSet<(QualifiedName, Version, ComponentRef)>,
    diagnostics: &mut Vec<CompositionDiagnostic>,
) {
    if !declared_keys.insert(imported.key.clone()) {
        diagnostics.push(CompositionDiagnostic::new(
            CompositionCode::DuplicateServiceKey,
            Some(imported.key.clone()),
            format!("service key `{}` is imported more than once", imported.key),
        ));
    }
    if !declared_identities.insert((
        imported.system.clone(),
        imported.version,
        imported.component.clone(),
    )) {
        diagnostics.push(CompositionDiagnostic::new(
            CompositionCode::DuplicateServiceIdentity,
            Some(imported.key.clone()),
            format!(
                "ESS service {} {} component {} is already bound to another key",
                imported.system, imported.version, imported.component
            ),
        ));
    }
}

fn validate_import_contract(
    imported: &ServiceImportSpec,
    ir: &EssIr,
    diagnostics: &mut Vec<CompositionDiagnostic>,
) {
    if ir.system() != &imported.system {
        diagnostics.push(CompositionDiagnostic::new(
            CompositionCode::SystemMismatch,
            Some(imported.key.clone()),
            format!(
                "service `{}` declares system {}, but its IR is {}",
                imported.key,
                imported.system,
                ir.system()
            ),
        ));
    }
    if ir.version() != &imported.version {
        diagnostics.push(CompositionDiagnostic::new(
            CompositionCode::VersionMismatch,
            Some(imported.key.clone()),
            format!(
                "service `{}` declares version {}, but its IR is {}",
                imported.key,
                imported.version,
                ir.version()
            ),
        ));
    }
    let actual_digest = SourceDigest::of(ir);
    if actual_digest != imported.source_digest {
        diagnostics.push(CompositionDiagnostic::new(
            CompositionCode::DigestMismatch,
            Some(imported.key.clone()),
            format!(
                "service `{}` declares digest {}, but its IR computes {}",
                imported.key, imported.source_digest, actual_digest
            ),
        ));
    }
}

fn resolved_service(
    imported: &ServiceImportSpec,
    ir: &EssIr,
    diagnostics: &mut Vec<CompositionDiagnostic>,
) -> Option<ResolvedService> {
    let Some(component) = ir.components().get(imported.component.name()) else {
        diagnostics.push(CompositionDiagnostic::new(
            CompositionCode::UnknownComponent,
            Some(imported.key.clone()),
            format!(
                "service `{}` selects component `{}`, which {} {} does not declare",
                imported.key,
                imported.component,
                ir.system(),
                ir.version()
            ),
        ));
        return None;
    };

    let commands: BTreeSet<_> = component.accepts.iter().map(CommandRef::from).collect();
    let mut queries = BTreeSet::new();
    for domain in &component.owns {
        queries.extend(ir.domain(domain).views.iter().map(ViewRef::from));
    }

    let mut events: BTreeSet<_> = component.publishes.iter().map(EventRef::from).collect();
    let mut errors = BTreeSet::new();
    for command in &component.accepts {
        let command = ir.command(command);
        events.extend(command.emits().map(EventRef::from));
        errors.extend(command.errors().map(ErrorRef::from));
    }

    let mut types = BTreeSet::new();
    for command in &commands {
        collect_field_types(ir, &ir.commands()[command.name()].input, &mut types);
    }
    for event in &events {
        collect_field_types(ir, &ir.events()[event.name()].fields, &mut types);
    }
    for error in &errors {
        collect_field_types(ir, &ir.errors()[error.name()].fields, &mut types);
    }
    for query in &queries {
        let view = &ir.views()[query.name()];
        if let Some(shape) = &view.shape {
            collect_declared_type(ir, shape, &mut types);
        }
        collect_field_types(ir, &view.fields, &mut types);
    }

    Some(ResolvedService {
        system: ir.system().clone(),
        version: *ir.version(),
        source_digest: SourceDigest::of(ir),
        component: imported.component.clone(),
        commands,
        queries,
        events,
        errors,
        types,
    })
}

fn collect_field_types(
    ir: &EssIr,
    fields: &[ResolvedField],
    types: &mut BTreeSet<DeclaredTypeRef>,
) {
    for field in fields {
        collect_type_ref(ir, &field.type_ref, types);
    }
}

fn collect_type_ref(ir: &EssIr, type_ref: &ResolvedTypeRef, types: &mut BTreeSet<DeclaredTypeRef>) {
    for handle in type_ref.named_leaves() {
        collect_declared_type(ir, handle, types);
    }
}

fn collect_declared_type(ir: &EssIr, handle: &TypeHandle, types: &mut BTreeSet<DeclaredTypeRef>) {
    if !types.insert(DeclaredTypeRef::from(handle)) {
        return;
    }
    match &ir.named_type(handle).body {
        ResolvedBody::Newtype { of, .. } => collect_type_ref(ir, of, types),
        ResolvedBody::Struct { fields, .. } => collect_field_types(ir, fields, types),
        ResolvedBody::Union { variants, .. } => {
            for variant in variants.values() {
                collect_type_ref(ir, variant, types);
            }
        }
        ResolvedBody::Enum { .. } => {}
    }
}

fn resolve_references(
    specification: &CompositionSpec,
    registry: &BTreeMap<ServiceKey, &EssIr>,
    declared_keys: &BTreeSet<ServiceKey>,
    resolved: &BTreeMap<ServiceKey, ResolvedService>,
    diagnostics: &mut Vec<CompositionDiagnostic>,
) -> BTreeSet<CompositionRef> {
    let mut references = BTreeSet::new();
    for reference in &specification.references {
        let Some(ir) = registry.get(&reference.service).copied() else {
            diagnostics.push(CompositionDiagnostic::new(
                CompositionCode::UnknownReferenceService,
                Some(reference.service.clone()),
                format!(
                    "reference `{}` selects service `{}`, which has no compiled input",
                    reference.semantic, reference.service
                ),
            ));
            continue;
        };
        if !declared_keys.contains(&reference.service) {
            diagnostics.push(CompositionDiagnostic::new(
                CompositionCode::UnknownReferenceService,
                Some(reference.service.clone()),
                format!(
                    "reference `{}` selects service `{}`, which is not imported",
                    reference.semantic, reference.service
                ),
            ));
        } else if !ir.resolves(&reference.semantic) {
            diagnostics.push(CompositionDiagnostic::new(
                CompositionCode::UnresolvedSemanticReference,
                Some(reference.service.clone()),
                format!(
                    "service `{}` does not resolve `{}`",
                    reference.service, reference.semantic
                ),
            ));
        } else if let Some(service) = resolved.get(&reference.service) {
            if service.exports(&reference.semantic) {
                references.insert(reference.clone());
            } else {
                diagnostics.push(CompositionDiagnostic::new(
                    CompositionCode::ReferenceOutsideComponent,
                    Some(reference.service.clone()),
                    format!(
                        "service `{}` resolves `{}`, but selected component `{}` does not export it",
                        reference.service, reference.semantic, service.component
                    ),
                ));
            }
        }
    }
    references
}

fn canonical_json(value: &impl serde::Serialize) -> String {
    let mut json = serde_json::to_string_pretty(value)
        .unwrap_or_else(|error| panic!("validated composition serialises: {error}"));
    json.push('\n');
    json
}
