//! A concrete implementation and its human or machine entrypoints, bound to one exact ESS.
//!
//! [`RealizationSpec`] is adopter-authored `ess-realization/1`. [`compile`] resolves that document
//! against one [`EssIr`] and emits deterministic `ess-realization-ir/1`. This is deliberately a
//! sibling of `EssIr`: a semantic system does not change when it is exposed through a local TUI,
//! a JSON CLI, or a hosted workbench.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::fmt::Write as _;

use ess_compiler::refs::{ActorRef, ComponentRef, EssSemanticRef};
use ess_compiler::EssIr;
use ess_domain::name::{QualifiedName, Version};
use sha2::{Digest as _, Sha256};

/// The adopter-authored realization format.
pub const REALIZATION_FORMAT: &str = "ess-realization/1";

/// The compiled realization format.
pub const REALIZATION_IR_FORMAT: &str = "ess-realization-ir/1";

/// A path-safe, stable identifier.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize)]
#[serde(transparent)]
pub struct RealizationId(String);

impl RealizationId {
    /// Parses a lowercase identifier with single `.`, `_`, or `-` separators.
    pub fn new(value: impl AsRef<str>) -> Result<Self, IdentifierError> {
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
            Err(IdentifierError(value.to_owned()))
        }
    }

    /// The validated identifier.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for RealizationId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl<'de> serde::Deserialize<'de> for RealizationId {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        Self::new(value).map_err(serde::de::Error::custom)
    }
}

/// A malformed realization, implementation, or entrypoint identifier.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdentifierError(String);

impl fmt::Display for IdentifierError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "invalid identifier {:?}: expected lowercase segments separated by `.`, `_`, or `-`",
            self.0
        )
    }
}

impl std::error::Error for IdentifierError {}

/// An exact lowercase digest identifying immutable bytes or a Git tree.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize)]
#[serde(transparent)]
pub struct ArtifactIdentity(String);

impl ArtifactIdentity {
    /// Parses `sha256:<64 lowercase hex>` or `git:<40 lowercase hex>`.
    pub fn new(value: impl AsRef<str>) -> Result<Self, ArtifactIdentityError> {
        let value = value.as_ref();
        let valid = value
            .strip_prefix("sha256:")
            .is_some_and(|digest| lowercase_hex(digest, 64))
            || value
                .strip_prefix("git:")
                .is_some_and(|digest| lowercase_hex(digest, 40));
        if valid {
            Ok(Self(value.to_owned()))
        } else {
            Err(ArtifactIdentityError(value.to_owned()))
        }
    }

    /// The exact identity including its algorithm prefix.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for ArtifactIdentity {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl<'de> serde::Deserialize<'de> for ArtifactIdentity {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        Self::new(value).map_err(serde::de::Error::custom)
    }
}

fn lowercase_hex(value: &str, length: usize) -> bool {
    value.len() == length
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

/// A malformed immutable artifact identity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactIdentityError(String);

impl fmt::Display for ArtifactIdentityError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "invalid artifact identity {:?}: expected `sha256:<64 lowercase hex>` or `git:<40 lowercase hex>`",
            self.0
        )
    }
}

impl std::error::Error for ArtifactIdentityError {}

/// The exact ESS contract a realization implements.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpecificationIdentity {
    system: QualifiedName,
    version: Version,
    source_digest: ArtifactIdentity,
}

/// The synthesis decision that led to an implementation.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SynthesisIdentity {
    target: String,
    generator: String,
}

/// What kind of immutable artifact contains an implementation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactKind {
    /// A source tree at an exact commit.
    Source,
    /// A language ecosystem package.
    Package,
    /// A native executable archive or executable.
    Binary,
    /// An OCI container image.
    Container,
    /// A hosted service implementation identified by its immutable source or image.
    HostedService,
}

/// One immutable implementation artifact.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Artifact {
    kind: ArtifactKind,
    locator: String,
    identity: ArtifactIdentity,
}

/// One implementation choice in the source document.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ImplementationSpec {
    id: RealizationId,
    components: Vec<ComponentRef>,
    artifact: Artifact,
}

/// How a caller interacts with one entrypoint.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Interaction {
    /// Inspect state without requesting a semantic command.
    Observe,
    /// Invoke explicit semantic operations.
    Invoke,
    /// Run a model-backed loop that proposes semantic operations.
    AgentLoop,
}

impl Interaction {
    fn label(self) -> &'static str {
        match self {
            Self::Observe => "Observe",
            Self::Invoke => "Invoke",
            Self::AgentLoop => "Agent loop",
        }
    }
}

/// Where the interface attaches to the realized component.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Attachment {
    /// Interface and implementation share a process.
    InProcess,
    /// The interface binds only to the local host.
    Loopback,
    /// The interface crosses a network boundary.
    Network,
}

impl Attachment {
    fn label(self) -> &'static str {
        match self {
            Self::InProcess => "In process",
            Self::Loopback => "Loopback",
            Self::Network => "Network",
        }
    }
}

/// Public availability of an entrypoint; network reach alone never implies this.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Availability {
    /// Anyone may obtain or use this entrypoint under its published terms.
    Public,
    /// Access is granted explicitly rather than self-served.
    ApprovalRequired,
    /// The entrypoint is an organization-internal integration.
    Internal,
}

impl Availability {
    fn label(self) -> &'static str {
        match self {
            Self::Public => "Public",
            Self::ApprovalRequired => "Approval required",
            Self::Internal => "Internal",
        }
    }
}

/// Current support posture of an entrypoint.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Support {
    /// Maintained for adopter use.
    Supported,
    /// Available for evaluation while its contract may still move.
    Preview,
    /// An exploratory surface with no stability promise.
    Experimental,
    /// Defined, but not currently offered.
    Paused,
}

impl Support {
    fn label(self) -> &'static str {
        match self {
            Self::Supported => "Supported",
            Self::Preview => "Preview",
            Self::Experimental => "Experimental",
            Self::Paused => "Paused",
        }
    }
}

/// A typed runtime prerequisite. Values and credential material have no field in this shape.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeRequirement {
    kind: RuntimeRequirementKind,
    name: String,
    summary: String,
}

/// The closed runtime-requirement vocabulary.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeRequirementKind {
    /// Operating-system family or capability.
    OperatingSystem,
    /// CPU architecture.
    Architecture,
    /// Environment-variable name; never its value.
    EnvironmentVariable,
    /// Filesystem path or filesystem capability.
    Filesystem,
    /// Network reachability requirement.
    Network,
    /// Linux control-group delegation.
    Cgroup,
    /// A named credential source, never credential material.
    Credential,
}

/// Exactly one way to enter the implementation.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Invocation {
    /// Execute an argument vector without shell interpolation.
    Argv {
        /// Executable followed by its arguments.
        argv: Vec<String>,
    },
    /// Open an HTTP(S) URL, possibly containing declared `${NAME}` placeholders.
    Url {
        /// URL template without query, fragment, or embedded credentials.
        url: String,
    },
}

impl Invocation {
    fn markdown(&self) -> String {
        match self {
            Self::Argv { argv } => argv.join(" "),
            Self::Url { url } => url.clone(),
        }
    }
}

/// One entrypoint before semantic resolution.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EntryPointSpec {
    id: RealizationId,
    title: String,
    summary: String,
    primary: bool,
    interaction: Interaction,
    attachment: Attachment,
    availability: Availability,
    support: Support,
    implementation: RealizationId,
    #[serde(default)]
    actors: Vec<ActorRef>,
    surfaces: Vec<EssSemanticRef>,
    invocation: Invocation,
    #[serde(default)]
    requires: Vec<RuntimeRequirement>,
}

/// Optional evidence that this implementation was checked against the ESS suite.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConformanceEvidence {
    status: ConformanceStatus,
    suite_digest: ArtifactIdentity,
    report_digest: ArtifactIdentity,
}

/// The verdict recorded by conformance evidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConformanceStatus {
    /// Every scenario passed.
    Passed,
    /// At least one scenario failed.
    Failed,
    /// The report did not reach a verdict.
    Error,
}

/// An adopter-authored physical realization.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RealizationSpec {
    #[serde(rename = "type")]
    format: String,
    id: RealizationId,
    specification: SpecificationIdentity,
    synthesis: SynthesisIdentity,
    components: Vec<ComponentRef>,
    actors: Vec<ActorRef>,
    implementations: Vec<ImplementationSpec>,
    entrypoints: Vec<EntryPointSpec>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    conformance: Option<ConformanceEvidence>,
}

impl RealizationSpec {
    /// Reads strict JSON. Semantic validation happens in [`compile`].
    pub fn from_json(text: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(text)
    }

    /// Reads strict YAML. Semantic validation happens in [`compile`].
    pub fn from_yaml(text: &str) -> Result<Self, serde_yaml::Error> {
        serde_yaml::from_str(text)
    }
}

/// A compiled implementation choice.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct Implementation {
    components: BTreeSet<ComponentRef>,
    artifact: Artifact,
}

/// A compiled and resolved entrypoint.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct EntryPoint {
    title: String,
    summary: String,
    primary: bool,
    interaction: Interaction,
    attachment: Attachment,
    availability: Availability,
    support: Support,
    implementation: RealizationId,
    actors: BTreeSet<ActorRef>,
    surfaces: BTreeSet<EssSemanticRef>,
    invocation: Invocation,
    requires: BTreeSet<RuntimeRequirement>,
}

/// Deterministic, resolved `ess-realization-ir/1`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct RealizationIr {
    #[serde(rename = "type")]
    format: &'static str,
    id: RealizationId,
    realization_digest: ArtifactIdentity,
    specification: SpecificationIdentity,
    synthesis: SynthesisIdentity,
    components: BTreeSet<ComponentRef>,
    actors: BTreeSet<ActorRef>,
    implementations: BTreeMap<RealizationId, Implementation>,
    entrypoints: BTreeMap<RealizationId, EntryPoint>,
    #[serde(skip_serializing_if = "Option::is_none")]
    conformance: Option<ConformanceEvidence>,
}

impl RealizationIr {
    /// The stable realization identifier.
    pub fn id(&self) -> &RealizationId {
        &self.id
    }

    /// The exact digest of the implementation selection represented by this IR.
    pub fn realization_digest(&self) -> &ArtifactIdentity {
        &self.realization_digest
    }

    /// The exact ESS identity this realization implements.
    pub fn specification(&self) -> &SpecificationIdentity {
        &self.specification
    }

    /// The compiled entrypoints in identifier order.
    pub fn entrypoints(&self) -> &BTreeMap<RealizationId, EntryPoint> {
        &self.entrypoints
    }

    /// Canonical pretty JSON with a trailing newline.
    pub fn to_canonical_json(&self) -> String {
        canonical_json(self)
    }

    /// A deterministic public-facing run-mode reference.
    pub fn to_markdown(&self) -> String {
        let mut output = format!(
            "# Running modes\n\nGenerated from `{}` for {} {} (`{}`). Do not edit this file; change the realization declarations and regenerate it.\n\n",
            self.id,
            self.specification.system,
            self.specification.version,
            self.specification.source_digest
        );
        output.push_str("| Mode | Interaction | Attachment | Availability | Support | Start |\n");
        output.push_str("|---|---|---|---|---|---|\n");
        for entrypoint in self.entrypoints.values() {
            let primary = if entrypoint.primary {
                " (recommended)"
            } else {
                ""
            };
            let _ = writeln!(
                output,
                "| {}{} | {} | {} | {} | {} | `{}` |",
                table(&entrypoint.title),
                primary,
                entrypoint.interaction.label(),
                entrypoint.attachment.label(),
                entrypoint.availability.label(),
                entrypoint.support.label(),
                table(&entrypoint.invocation.markdown())
            );
        }
        for (id, entrypoint) in &self.entrypoints {
            let _ = write!(
                output,
                "\n## {}\n\n{}\n\n- Entrypoint: `{}`\n- Implementation: `{}`\n- Invocation: `{}`\n",
                entrypoint.title,
                entrypoint.summary,
                id,
                entrypoint.implementation,
                entrypoint.invocation.markdown()
            );
            if !entrypoint.requires.is_empty() {
                output.push_str("- Runtime requirements:\n");
                for requirement in &entrypoint.requires {
                    let _ = writeln!(
                        output,
                        "  - `{:?}` `{}` — {}",
                        requirement.kind, requirement.name, requirement.summary
                    );
                }
            }
        }
        output
    }
}

fn table(value: &str) -> String {
    value.replace('|', "\\|").replace('`', "\\`")
}

fn canonical_json(value: &impl serde::Serialize) -> String {
    let mut json = serde_json::to_string_pretty(value)
        .unwrap_or_else(|error| panic!("realization IR serializes: {error}"));
    json.push('\n');
    json
}

/// Stable refusal categories for realization compilation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RealizationCode {
    /// The source format marker is unsupported.
    UnsupportedFormat,
    /// The source names a different ESS system, version, or digest.
    SpecificationMismatch,
    /// A list contains the same stable identity more than once.
    DuplicateIdentity,
    /// A component, actor, semantic surface, or implementation does not resolve.
    UnresolvedReference,
    /// A component or actor lies outside the realization's declared subset.
    ReferenceOutsideRealization,
    /// A component has no implementation or more than one implementation.
    ImplementationCoverage,
    /// No entrypoint, no surface, or no implementation was declared.
    EmptyDeclaration,
    /// Zero or multiple entrypoints are marked primary.
    PrimaryEntrypoint,
    /// An invocation, artifact locator, synthesis identity, title, or summary is malformed.
    InvalidValue,
    /// Invocation text may contain credential material rather than indirection.
    SecretValue,
}

/// One location-aware realization refusal.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct RealizationDiagnostic {
    code: RealizationCode,
    path: String,
    message: String,
}

impl RealizationDiagnostic {
    /// The stable refusal category.
    pub const fn code(&self) -> RealizationCode {
        self.code
    }

    /// The source path associated with the refusal.
    pub fn path(&self) -> &str {
        &self.path
    }
}

/// Every refusal found while compiling one realization.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct RealizationDiagnostics(Vec<RealizationDiagnostic>);

impl RealizationDiagnostics {
    /// The ordered diagnostics.
    pub fn as_slice(&self) -> &[RealizationDiagnostic] {
        &self.0
    }

    /// Whether one category occurs.
    pub fn contains(&self, code: RealizationCode) -> bool {
        self.0.iter().any(|diagnostic| diagnostic.code == code)
    }
}

impl fmt::Display for RealizationDiagnostics {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        for diagnostic in &self.0 {
            writeln!(
                formatter,
                "  - {:?} at {}: {}",
                diagnostic.code, diagnostic.path, diagnostic.message
            )?;
        }
        Ok(())
    }
}

impl std::error::Error for RealizationDiagnostics {}

fn refuse(
    diagnostics: &mut Vec<RealizationDiagnostic>,
    code: RealizationCode,
    path: impl Into<String>,
    message: impl Into<String>,
) {
    diagnostics.push(RealizationDiagnostic {
        code,
        path: path.into(),
        message: message.into(),
    });
}

/// Resolves and canonicalizes one physical realization against an exact ESS.
pub fn compile(
    specification: &RealizationSpec,
    ess: &EssIr,
) -> Result<RealizationIr, RealizationDiagnostics> {
    let mut diagnostics = Vec::new();
    validate_header(specification, ess, &mut diagnostics);
    let components = resolve_components(specification, ess, &mut diagnostics);
    let actors = resolve_actors(specification, ess, &mut diagnostics);
    let implementations = resolve_implementations(specification, &components, &mut diagnostics);
    let entrypoints = resolve_entrypoints(
        specification,
        ess,
        &actors,
        &implementations,
        &mut diagnostics,
    );

    if !diagnostics.is_empty() {
        return Err(RealizationDiagnostics(diagnostics));
    }

    let digest = realization_digest(
        &specification.specification,
        &specification.synthesis,
        &implementations,
    );
    Ok(RealizationIr {
        format: REALIZATION_IR_FORMAT,
        id: specification.id.clone(),
        realization_digest: digest,
        specification: specification.specification.clone(),
        synthesis: specification.synthesis.clone(),
        components,
        actors,
        implementations,
        entrypoints,
        conformance: specification.conformance.clone(),
    })
}

fn validate_header(
    specification: &RealizationSpec,
    ess: &EssIr,
    diagnostics: &mut Vec<RealizationDiagnostic>,
) {
    if specification.format != REALIZATION_FORMAT {
        refuse(
            diagnostics,
            RealizationCode::UnsupportedFormat,
            "type",
            format!(
                "expected `{REALIZATION_FORMAT}`, found {:?}",
                specification.format
            ),
        );
    }
    let expected_digest = format!("sha256:{}", ess.source_digest());
    if specification.specification.system != *ess.system()
        || specification.specification.version != *ess.version()
        || specification.specification.source_digest.as_str() != expected_digest
    {
        refuse(
            diagnostics,
            RealizationCode::SpecificationMismatch,
            "specification",
            format!(
                "expected {} {} `{expected_digest}`",
                ess.system(),
                ess.version()
            ),
        );
    }
    validate_nonempty(
        diagnostics,
        "synthesis.target",
        &specification.synthesis.target,
    );
    validate_nonempty(
        diagnostics,
        "synthesis.generator",
        &specification.synthesis.generator,
    );
}

fn resolve_components(
    specification: &RealizationSpec,
    ess: &EssIr,
    diagnostics: &mut Vec<RealizationDiagnostic>,
) -> BTreeSet<ComponentRef> {
    let components: BTreeSet<_> = specification.components.iter().cloned().collect();
    duplicate_count(
        diagnostics,
        "components",
        specification.components.len(),
        components.len(),
    );
    if components.is_empty() {
        refuse(
            diagnostics,
            RealizationCode::EmptyDeclaration,
            "components",
            "at least one component is required",
        );
    }
    for component in &components {
        if !ess.components().contains_key(component.name()) {
            refuse(
                diagnostics,
                RealizationCode::UnresolvedReference,
                "components",
                format!("component `{component}` is not declared by the ESS"),
            );
        }
    }
    components
}

fn resolve_actors(
    specification: &RealizationSpec,
    ess: &EssIr,
    diagnostics: &mut Vec<RealizationDiagnostic>,
) -> BTreeSet<ActorRef> {
    let actors: BTreeSet<_> = specification.actors.iter().cloned().collect();
    duplicate_count(
        diagnostics,
        "actors",
        specification.actors.len(),
        actors.len(),
    );
    for actor in &actors {
        if !ess.actors().contains_key(actor.name()) {
            refuse(
                diagnostics,
                RealizationCode::UnresolvedReference,
                "actors",
                format!("actor `{actor}` is not declared by the ESS"),
            );
        }
    }
    actors
}

fn resolve_implementations(
    specification: &RealizationSpec,
    components: &BTreeSet<ComponentRef>,
    diagnostics: &mut Vec<RealizationDiagnostic>,
) -> BTreeMap<RealizationId, Implementation> {
    let mut implementations = BTreeMap::new();
    let mut component_coverage: BTreeMap<ComponentRef, usize> = BTreeMap::new();
    for (index, implementation) in specification.implementations.iter().enumerate() {
        let path = format!("implementations[{index}]");
        if implementations.contains_key(&implementation.id) {
            refuse(
                diagnostics,
                RealizationCode::DuplicateIdentity,
                format!("{path}.id"),
                format!(
                    "implementation `{}` is declared more than once",
                    implementation.id
                ),
            );
            continue;
        }
        validate_nonempty(
            diagnostics,
            &format!("{path}.artifact.locator"),
            &implementation.artifact.locator,
        );
        let selected: BTreeSet<_> = implementation.components.iter().cloned().collect();
        duplicate_count(
            diagnostics,
            &format!("{path}.components"),
            implementation.components.len(),
            selected.len(),
        );
        if selected.is_empty() {
            refuse(
                diagnostics,
                RealizationCode::EmptyDeclaration,
                format!("{path}.components"),
                "an implementation must implement at least one component",
            );
        }
        for component in &selected {
            if !components.contains(component) {
                refuse(
                    diagnostics,
                    RealizationCode::ReferenceOutsideRealization,
                    format!("{path}.components"),
                    format!("component `{component}` is outside this realization"),
                );
            }
            *component_coverage.entry(component.clone()).or_default() += 1;
        }
        implementations.insert(
            implementation.id.clone(),
            Implementation {
                components: selected,
                artifact: implementation.artifact.clone(),
            },
        );
    }
    if implementations.is_empty() {
        refuse(
            diagnostics,
            RealizationCode::EmptyDeclaration,
            "implementations",
            "at least one implementation is required",
        );
    }
    for component in components {
        match component_coverage
            .get(component)
            .copied()
            .unwrap_or_default()
        {
            1 => {}
            count => refuse(
                diagnostics,
                RealizationCode::ImplementationCoverage,
                "implementations",
                format!(
                    "component `{component}` has {count} implementations; expected exactly one"
                ),
            ),
        }
    }
    implementations
}

fn resolve_entrypoints(
    specification: &RealizationSpec,
    ess: &EssIr,
    actors: &BTreeSet<ActorRef>,
    implementations: &BTreeMap<RealizationId, Implementation>,
    diagnostics: &mut Vec<RealizationDiagnostic>,
) -> BTreeMap<RealizationId, EntryPoint> {
    let mut entrypoints = BTreeMap::new();
    let mut primary_count = 0_usize;
    for (index, entrypoint) in specification.entrypoints.iter().enumerate() {
        let path = format!("entrypoints[{index}]");
        if entrypoints.contains_key(&entrypoint.id) {
            refuse(
                diagnostics,
                RealizationCode::DuplicateIdentity,
                format!("{path}.id"),
                format!("entrypoint `{}` is declared more than once", entrypoint.id),
            );
            continue;
        }
        primary_count += usize::from(entrypoint.primary);
        entrypoints.insert(
            entrypoint.id.clone(),
            resolve_entrypoint(entrypoint, &path, ess, actors, implementations, diagnostics),
        );
    }
    if entrypoints.is_empty() {
        refuse(
            diagnostics,
            RealizationCode::EmptyDeclaration,
            "entrypoints",
            "at least one entrypoint is required",
        );
    }
    if primary_count != 1 {
        refuse(
            diagnostics,
            RealizationCode::PrimaryEntrypoint,
            "entrypoints",
            format!("exactly one entrypoint must be primary; found {primary_count}"),
        );
    }
    entrypoints
}

fn resolve_entrypoint(
    entrypoint: &EntryPointSpec,
    path: &str,
    ess: &EssIr,
    actors: &BTreeSet<ActorRef>,
    implementations: &BTreeMap<RealizationId, Implementation>,
    diagnostics: &mut Vec<RealizationDiagnostic>,
) -> EntryPoint {
    validate_nonempty(diagnostics, &format!("{path}.title"), &entrypoint.title);
    validate_nonempty(diagnostics, &format!("{path}.summary"), &entrypoint.summary);
    if !implementations.contains_key(&entrypoint.implementation) {
        refuse(
            diagnostics,
            RealizationCode::UnresolvedReference,
            format!("{path}.implementation"),
            format!(
                "implementation `{}` is not declared",
                entrypoint.implementation
            ),
        );
    }
    let selected_actors: BTreeSet<_> = entrypoint.actors.iter().cloned().collect();
    duplicate_count(
        diagnostics,
        &format!("{path}.actors"),
        entrypoint.actors.len(),
        selected_actors.len(),
    );
    for actor in &selected_actors {
        if !actors.contains(actor) {
            refuse(
                diagnostics,
                RealizationCode::ReferenceOutsideRealization,
                format!("{path}.actors"),
                format!("actor `{actor}` is outside this realization"),
            );
        }
    }
    let surfaces: BTreeSet<_> = entrypoint.surfaces.iter().cloned().collect();
    duplicate_count(
        diagnostics,
        &format!("{path}.surfaces"),
        entrypoint.surfaces.len(),
        surfaces.len(),
    );
    if surfaces.is_empty() {
        refuse(
            diagnostics,
            RealizationCode::EmptyDeclaration,
            format!("{path}.surfaces"),
            "an entrypoint must name at least one semantic surface",
        );
    }
    for surface in &surfaces {
        if !ess.resolves(surface) {
            refuse(
                diagnostics,
                RealizationCode::UnresolvedReference,
                format!("{path}.surfaces"),
                format!("{surface} is not declared by the ESS"),
            );
        }
    }
    validate_invocation(
        diagnostics,
        path,
        &entrypoint.invocation,
        &entrypoint.requires,
    );
    EntryPoint {
        title: entrypoint.title.clone(),
        summary: entrypoint.summary.clone(),
        primary: entrypoint.primary,
        interaction: entrypoint.interaction,
        attachment: entrypoint.attachment,
        availability: entrypoint.availability,
        support: entrypoint.support,
        implementation: entrypoint.implementation.clone(),
        actors: selected_actors,
        surfaces,
        invocation: entrypoint.invocation.clone(),
        requires: resolve_requirements(path, &entrypoint.requires, diagnostics),
    }
}

fn resolve_requirements(
    path: &str,
    source: &[RuntimeRequirement],
    diagnostics: &mut Vec<RealizationDiagnostic>,
) -> BTreeSet<RuntimeRequirement> {
    let requirements: BTreeSet<_> = source.iter().cloned().collect();
    duplicate_count(
        diagnostics,
        &format!("{path}.requires"),
        source.len(),
        requirements.len(),
    );
    for (index, requirement) in source.iter().enumerate() {
        validate_nonempty(
            diagnostics,
            &format!("{path}.requires[{index}].name"),
            &requirement.name,
        );
        validate_nonempty(
            diagnostics,
            &format!("{path}.requires[{index}].summary"),
            &requirement.summary,
        );
        if matches!(
            requirement.kind,
            RuntimeRequirementKind::EnvironmentVariable
        ) && !environment_name(&requirement.name)
        {
            refuse(
                diagnostics,
                RealizationCode::InvalidValue,
                format!("{path}.requires[{index}].name"),
                "environment-variable names use `A_Z`, `0_9`, and `_`, starting with a letter or `_`",
            );
        }
    }
    requirements
}

fn realization_digest(
    specification: &SpecificationIdentity,
    synthesis: &SynthesisIdentity,
    implementations: &BTreeMap<RealizationId, Implementation>,
) -> ArtifactIdentity {
    let bytes = serde_json::to_vec(&(specification, synthesis, implementations))
        .unwrap_or_else(|error| panic!("realization identity serializes: {error}"));
    let digest = Sha256::digest(bytes);
    let mut value = String::with_capacity(71);
    value.push_str("sha256:");
    for byte in digest {
        let _ = write!(value, "{byte:02x}");
    }
    ArtifactIdentity::new(value).expect("SHA-256 output is an exact lowercase digest")
}

fn duplicate_count(
    diagnostics: &mut Vec<RealizationDiagnostic>,
    path: &str,
    source: usize,
    unique: usize,
) {
    if source != unique {
        refuse(
            diagnostics,
            RealizationCode::DuplicateIdentity,
            path,
            format!("found {} duplicate declaration(s)", source - unique),
        );
    }
}

fn validate_nonempty(diagnostics: &mut Vec<RealizationDiagnostic>, path: &str, value: &str) {
    if value.trim().is_empty() || value.chars().any(char::is_control) {
        refuse(
            diagnostics,
            RealizationCode::InvalidValue,
            path,
            "must be non-empty text without control characters",
        );
    }
}

fn validate_invocation(
    diagnostics: &mut Vec<RealizationDiagnostic>,
    path: &str,
    invocation: &Invocation,
    requirements: &[RuntimeRequirement],
) {
    let environment: BTreeSet<_> = requirements
        .iter()
        .filter(|requirement| {
            matches!(
                requirement.kind,
                RuntimeRequirementKind::EnvironmentVariable
            )
        })
        .map(|requirement| requirement.name.as_str())
        .collect();
    let values = match invocation {
        Invocation::Argv { argv } => {
            if argv.is_empty() || argv.iter().any(String::is_empty) {
                refuse(
                    diagnostics,
                    RealizationCode::InvalidValue,
                    format!("{path}.invocation.argv"),
                    "argv must contain a non-empty executable and no empty argument",
                );
            }
            let sensitive = ["--api-key", "--token", "--password", "--secret"];
            for argument in argv {
                let lowered = argument.to_ascii_lowercase();
                if sensitive
                    .iter()
                    .any(|name| lowered == *name || lowered.starts_with(&format!("{name}=")))
                {
                    refuse(
                        diagnostics,
                        RealizationCode::SecretValue,
                        format!("{path}.invocation.argv"),
                        format!("`{argument}` may carry a secret; name an environment or credential source instead"),
                    );
                }
            }
            argv.iter().map(String::as_str).collect::<Vec<_>>()
        }
        Invocation::Url { url } => {
            if !(url.starts_with("http://") || url.starts_with("https://"))
                || url.contains('?')
                || url.contains('#')
                || url.split_once("://").is_some_and(|(_, authority)| {
                    authority
                        .split('/')
                        .next()
                        .is_some_and(|host| host.contains('@'))
                })
            {
                refuse(
                    diagnostics,
                    RealizationCode::InvalidValue,
                    format!("{path}.invocation.url"),
                    "URL must be HTTP(S) and contain no query, fragment, or embedded credentials",
                );
            }
            vec![url.as_str()]
        }
    };
    for value in values {
        match placeholders(value) {
            Ok(names) => {
                for name in names {
                    if !environment.contains(name.as_str()) {
                        refuse(
                            diagnostics,
                            RealizationCode::InvalidValue,
                            format!("{path}.invocation"),
                            format!("placeholder `${{{name}}}` has no environment_variable runtime requirement"),
                        );
                    }
                }
            }
            Err(message) => refuse(
                diagnostics,
                RealizationCode::InvalidValue,
                format!("{path}.invocation"),
                message,
            ),
        }
    }
}

fn placeholders(value: &str) -> Result<BTreeSet<String>, String> {
    let mut names = BTreeSet::new();
    let mut remainder = value;
    while let Some(start) = remainder.find("${") {
        remainder = &remainder[start + 2..];
        let Some(end) = remainder.find('}') else {
            return Err("invocation contains an unterminated `${NAME}` placeholder".to_owned());
        };
        let name = &remainder[..end];
        if !environment_name(name) {
            return Err(format!("invalid invocation placeholder `${{{name}}}`"));
        }
        names.insert(name.to_owned());
        remainder = &remainder[end + 1..];
    }
    Ok(names)
}

fn environment_name(value: &str) -> bool {
    let mut characters = value.chars();
    characters
        .next()
        .is_some_and(|character| character.is_ascii_uppercase() || character == '_')
        && characters.all(|character| {
            character.is_ascii_uppercase() || character.is_ascii_digit() || character == '_'
        })
}
