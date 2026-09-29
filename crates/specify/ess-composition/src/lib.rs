//! Validated composition of selected components from independently compiled ESS models.
//!
//! A service-local [`EssIr`] owns compiler handles that cannot cross into another compilation.
//! This crate keeps that boundary intact: a composition names a service with [`ServiceKey`] and a
//! construct with [`EssSemanticRef`], then resolves that pair against the IR registered for exactly
//! that key. The result is a compiler-minted [`EssCompositionIr`] containing stable names only.
//!
//! Every import selects one exact ESS component. Commands come only from that component's
//! `accepts` surface; queries come only from views in domains it owns. Published or emitted events,
//! command errors, and named types reached through command inputs, event/error fields and query
//! row shapes/fields are retained as references. View parameters are not traversed. These names
//! describe the selected surface; they do not carry complete payload definitions or codecs.
//!
//! Under [`CONFORMANCE_COMPOSITION_FORMAT`] a reference may also name any type declared in a domain
//! the selected component owns, and `conformances` asserts that a consumer's local type has the
//! shape of an imported component's type. That assertion is checked field by field against both
//! compiled models; the only tolerated difference is a consumer treating a required value as
//! optional. Owner-declared types are referenceable, not added to the client surface.
//!
//! Under [`READER_COMPOSITION_FORMAT`] a conformance may carry `reader: true`: the local type only
//! reads the imported one off the wire, so it may also read a newtype as its primitive, an enum as
//! `String`, enum variants by wire name, a struct or `String`-keyed map as `Map<String, Json>`, and
//! a subset of the imported fields. `reader: true` also asserts that the consumer's reader ignores
//! keys it does not declare; ESS-generated closed types (`additionalProperties: false`,
//! `deny_unknown_fields`) do not, so a consumer reading through them must not use `reader` for a
//! field subset.
//!
//! The persisted input formats are [`SUPPORTED_COMPOSITION_FORMATS`]. Generated clients consume
//! the derived [`EssClientPlan`] rather than reinterpreting multiple service models independently.
//! The Rust client constrains operation descriptors to that selection, but forwards request and
//! response bytes unchanged through an application-provided transport. It does no payload
//! admission, response decoding, authority verification or live endpoint/model identity handshake.

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

mod conformance;

/// The composition document format [`CompositionSpec::new`] writes: services and references only.
pub const COMPOSITION_FORMAT: &str = "ess-composition/1";

/// The composition format that admits `conformances` and references to owner-declared types.
pub const CONFORMANCE_COMPOSITION_FORMAT: &str = "ess-composition/2";

/// The composition format that also admits `reader: true` on a `conformances` entry.
pub const READER_COMPOSITION_FORMAT: &str = "ess-composition/3";

/// Every `ess-composition/N` major this build reads; any other marker is refused.
pub const SUPPORTED_COMPOSITION_FORMATS: &[u32] = &[1, 2, 3];

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

/// One end of a [`TypeConformance`]: a declared type in the service selected by its key.
#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
#[serde(deny_unknown_fields)]
pub struct TypeBinding {
    service: ServiceKey,
    #[serde(rename = "type")]
    declared: DeclaredTypeRef,
}

impl TypeBinding {
    /// Names a declared type in one imported service.
    pub fn new(service: ServiceKey, declared: DeclaredTypeRef) -> Self {
        Self { service, declared }
    }

    /// The service whose compiled model declares the type.
    pub fn service(&self) -> &ServiceKey {
        &self.service
    }

    /// The declared type's stable name.
    pub fn declared(&self) -> &DeclaredTypeRef {
        &self.declared
    }
}

/// An `ess-composition/2` or `/3` assertion that a consumer's local type has an imported type's
/// shape.
///
/// Both ends must be referenceable in their selected components. The shapes are compared field by
/// field; a local value may be optional where the imported one is required, and nothing else may
/// differ. A difference is refused as [`CompositionCode::TypeConformanceDrift`].
///
/// A reader assertion (`reader: true`, `ess-composition/3` only) also admits the widening a
/// consumer that only reads the imported type may do; see [`TypeConformance::for_reader`].
#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
#[serde(deny_unknown_fields)]
pub struct TypeConformance {
    local: TypeBinding,
    conforms_to: TypeBinding,
    /// `reader:` as authored (`ess-composition/3`): `None` when the key is absent, `Some(None)`
    /// for an authored `null`. Absent from every `/2` entry, which keeps its bytes; refused under
    /// `/1` and `/2` whatever its value.
    #[serde(
        default,
        deserialize_with = "present",
        skip_serializing_if = "Option::is_none"
    )]
    reader: Option<ReaderKey>,
}

/// An authored `reader:` value, `null` included; kept apart from absence by `Option<ReaderKey>`.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
#[serde(transparent)]
struct ReaderKey(Option<bool>);

/// Records that a key was written, `null` included, which a plain `Option` field cannot tell from
/// an absent one.
fn present<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: serde::Deserialize<'de>,
{
    T::deserialize(deserializer).map(Some)
}

impl TypeConformance {
    /// Asserts that `local` conforms to `conforms_to`.
    pub fn new(local: TypeBinding, conforms_to: TypeBinding) -> Self {
        Self {
            local,
            conforms_to,
            reader: None,
        }
    }

    /// Asserts that `local` reads every value `conforms_to` allows (`ess-composition/3`).
    ///
    /// Beyond [`TypeConformance::new`]'s rule, the local type may read a newtype (through any
    /// chain) as its representation, an enum as `String` (`Optional<String>` where the imported
    /// enum is optional), an enum by wire name with a superset of the imported wire names, a
    /// struct or `String`-keyed map as `Map<String, Json>`, and a subset of a struct's fields. A
    /// local field the imported struct lacks by name is compared with the imported field that
    /// shares its wire name, or else must be optional and not `null_when_absent`.
    ///
    /// The assertion includes a precondition the comparison cannot see: the consumer's reader
    /// ignores keys it does not declare. ESS-generated closed types (`additionalProperties: false`,
    /// `deny_unknown_fields`) do not, so a consumer reading through them must not use `reader` for
    /// a field subset.
    pub fn for_reader(local: TypeBinding, conforms_to: TypeBinding) -> Self {
        Self {
            local,
            conforms_to,
            reader: Some(ReaderKey(Some(true))),
        }
    }

    /// Whether this is a reader assertion (`reader: true`).
    pub fn reader(&self) -> bool {
        self.reader == Some(ReaderKey(Some(true)))
    }

    /// Whether the `reader` key was written, whatever its value.
    fn reader_key(&self) -> bool {
        self.reader.is_some()
    }

    /// The consumer's own type.
    pub fn local(&self) -> &TypeBinding {
        &self.local
    }

    /// The imported type the local one must match.
    pub fn conforms_to(&self) -> &TypeBinding {
        &self.conforms_to
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
    /// Present only from `ess-composition/2`; a `/1` document carrying the key, even empty, is
    /// refused by [`compile`], as is a `/2` entry carrying `reader`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    conformances: Option<Vec<TypeConformance>>,
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
            conformances: None,
        }
    }

    /// Creates an `ess-composition/2` input, which may reference owner-declared types and assert
    /// type conformances. Semantic validation happens in [`compile`].
    pub fn with_conformances(
        composition: ServiceKey,
        services: Vec<ServiceImportSpec>,
        references: Vec<CompositionRef>,
        conformances: Vec<TypeConformance>,
    ) -> Self {
        Self {
            format: CONFORMANCE_COMPOSITION_FORMAT.to_owned(),
            composition,
            services,
            references,
            conformances: (!conformances.is_empty()).then_some(conformances),
        }
    }

    /// Creates an `ess-composition/3` input, whose conformances may be reader assertions
    /// ([`TypeConformance::for_reader`]). Semantic validation happens in [`compile`].
    pub fn with_reader_conformances(
        composition: ServiceKey,
        services: Vec<ServiceImportSpec>,
        references: Vec<CompositionRef>,
        conformances: Vec<TypeConformance>,
    ) -> Self {
        Self {
            format: READER_COMPOSITION_FORMAT.to_owned(),
            ..Self::with_conformances(composition, services, references, conformances)
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

    /// Type conformance assertions, before resolution. Empty when the key is absent.
    pub fn conformances(&self) -> &[TypeConformance] {
        self.conformances.as_deref().unwrap_or_default()
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
    /// A local type asserted to conform to an imported type differs from it in a field's name,
    /// presence or type (from `ess-composition/2`).
    TypeConformanceDrift,
}

impl CompositionCode {
    /// Every category, in declaration order, with what it means and how to repair it.
    ///
    /// `tests/diagnostic_catalogue.rs` fails when a variant is missing here, and
    /// `website/docs/reference/diagnostics.md` is rendered from it by `cargo xtask diagnostics`.
    pub const CATALOGUE: &'static [ess_compiler::diagnostic::CatalogueEntry<Self>] = &[
        ess_compiler::diagnostic::CatalogueEntry {
            key: Self::UnsupportedFormat,
            meaning: "The document's format marker is not one this build reads.",
            repair: "Write a composition format this build reads.",
        },
        ess_compiler::diagnostic::CatalogueEntry {
            key: Self::DuplicateServiceKey,
            meaning: "Two imports declare the same service key.",
            repair: "Give each import its own key.",
        },
        ess_compiler::diagnostic::CatalogueEntry {
            key: Self::DuplicateServiceIdentity,
            meaning: "Two imports bind the same ESS system, version and component.",
            repair: "Import each system, version and component once.",
        },
        ess_compiler::diagnostic::CatalogueEntry {
            key: Self::MissingServiceInput,
            meaning: "No compiled specification was supplied for a declared import.",
            repair: "Supply the compiled specification for the import, or remove the import.",
        },
        ess_compiler::diagnostic::CatalogueEntry {
            key: Self::UndeclaredServiceInput,
            meaning: "A compiled specification was supplied that the document does not import.",
            repair: "Import it in the document, or stop supplying it.",
        },
        ess_compiler::diagnostic::CatalogueEntry {
            key: Self::DuplicateServiceInput,
            meaning: "The same key was supplied more than once.",
            repair: "Supply each import once.",
        },
        ess_compiler::diagnostic::CatalogueEntry {
            key: Self::SystemMismatch,
            meaning: "The compiled specification names a different system than the import.",
            repair: "Supply the specification of the system the import names, or correct the \
                     import.",
        },
        ess_compiler::diagnostic::CatalogueEntry {
            key: Self::VersionMismatch,
            meaning: "The compiled specification's version differs from the import's.",
            repair: "Supply the version the import names, or update the import.",
        },
        ess_compiler::diagnostic::CatalogueEntry {
            key: Self::DigestMismatch,
            meaning: "The compiled specification's digest differs from the import's.",
            repair: "Supply the specification the digest names, or update the digest.",
        },
        ess_compiler::diagnostic::CatalogueEntry {
            key: Self::UnknownReferenceService,
            meaning: "A reference names a service the document does not import.",
            repair: "Import the service, or correct the key in the reference.",
        },
        ess_compiler::diagnostic::CatalogueEntry {
            key: Self::UnresolvedSemanticReference,
            meaning: "A name does not resolve in the service its key selects.",
            repair: "Correct the name to one the imported specification declares.",
        },
        ess_compiler::diagnostic::CatalogueEntry {
            key: Self::UnknownComponent,
            meaning: "The imported specification does not declare the selected component.",
            repair: "Select a component the imported specification declares.",
        },
        ess_compiler::diagnostic::CatalogueEntry {
            key: Self::ReferenceOutsideComponent,
            meaning: "A name exists in the imported specification but is outside the selected \
                      component.",
            repair: "Refer only to what the selected component exposes, or select the component \
                     that owns it.",
        },
        ess_compiler::diagnostic::CatalogueEntry {
            key: Self::TypeConformanceDrift,
            meaning: "A local type said to conform to an imported type differs from it in a \
                      field's name, presence or type.",
            repair: "Change the local type to match the imported one field for field.",
        },
    ];

    /// This category's entry in [`Self::CATALOGUE`].
    pub fn catalogue_entry(self) -> &'static ess_compiler::diagnostic::CatalogueEntry<Self> {
        Self::CATALOGUE
            .iter()
            .find(|entry| entry.key == self)
            .expect("every category is catalogued")
    }
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
            Self::TypeConformanceDrift => "type_conformance_drift",
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
///
/// The system, version and compiled-model digest identify the imported ESS model. The remaining
/// values select a component and named references, not complete payload or codec definitions.
/// The digest does not identify raw YAML, composition/client-plan bytes or a running service.
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
    /// Every type declared in a domain the component owns. Referenceable under
    /// `ess-composition/2` and `/3`; never serialised, so neither IR nor client-plan bytes move.
    #[serde(skip)]
    declared_types: BTreeSet<DeclaredTypeRef>,
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

    /// Named error references reachable from selected commands.
    pub fn errors(&self) -> &BTreeSet<ErrorRef> {
        &self.errors
    }

    /// Named types recursively reached from command inputs, event/error fields and query row
    /// shapes/fields. View parameters are not traversed, and type definitions are not embedded.
    pub fn types(&self) -> &BTreeSet<DeclaredTypeRef> {
        &self.types
    }

    /// Every type declared in a domain the selected component owns, whether or not the client
    /// surface reaches it. `ess-composition/2` and `/3` references and conformances may name these.
    pub fn declared_types(&self) -> &BTreeSet<DeclaredTypeRef> {
        &self.declared_types
    }

    fn exports(&self, reference: &EssSemanticRef, owner_declared: bool) -> bool {
        match reference {
            EssSemanticRef::Command { name } => self.commands.contains(name),
            EssSemanticRef::Outcome { name } => self.commands.contains(&name.command),
            EssSemanticRef::Event { name } => self.events.contains(name),
            EssSemanticRef::Error { name } => self.errors.contains(name),
            EssSemanticRef::View { name } => self.queries.contains(name),
            EssSemanticRef::Type { name } => {
                self.types.contains(name) || (owner_declared && self.declared_types.contains(name))
            }
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
    #[serde(skip_serializing_if = "BTreeSet::is_empty")]
    conformances: BTreeSet<TypeConformance>,
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

    /// Every checked type conformance assertion (`ess-composition/2` and `/3`).
    pub fn conformances(&self) -> &BTreeSet<TypeConformance> {
        &self.conformances
    }

    /// Canonical JSON with a trailing newline.
    pub fn to_canonical_json(&self) -> String {
        canonical_json(self)
    }

    /// Derives the transport-neutral command/query surface for client generators.
    ///
    /// This preserves the selected names and compiled-model identity; it does not establish
    /// payload compatibility between a generated caller and a running service.
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
///
/// Carries the imported system, version, compiled-model digest, selected component and ordered
/// named references. It has no complete payload definitions or encoder/decoder definitions.
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

    /// Named types reached by the traversal described in [`ResolvedService::types`].
    pub fn types(&self) -> &BTreeSet<DeclaredTypeRef> {
        &self.types
    }
}

/// A language-neutral plan for composition client generators.
///
/// The plan pins model identity and selected operation names. Generated byte-buffer transport
/// does not provide end-to-end typed payload compatibility.
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
    /// component. Private `Operation` fields and its private constructor constrain normal
    /// downstream Rust callers to emitted descriptors, such as `service_todo::COMMAND_CREATE_LIST`.
    ///
    /// `Client::execute` passes its `&[u8]` payload unchanged to `Transport<Authority>` and returns
    /// the transport's `Vec<u8>` unchanged. It does no payload admission or response decoding:
    /// even a payload incompatible with the selected command's ESS declaration is forwarded.
    /// A missing endpoint yields `ClientError::MissingEndpoint` before authority lookup or
    /// transport execution; a transport failure is wrapped in `ClientError::Transport`.
    ///
    /// The application supplies endpoint, authority and transport providers. Authority is passed
    /// separately from the payload; verifying it and binding endpoints to the intended running
    /// service are application responsibilities. Provider injection performs neither verification
    /// nor a live model-digest handshake. No authentication operands are generated, but the opaque
    /// payload is not inspected or sanitized and can contain application-chosen coordinates.
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
    let format = validate_format(specification, &mut diagnostics);
    let registry = collect_registry(services, &mut diagnostics);
    let (declared_keys, resolved) = resolve_services(specification, &registry, &mut diagnostics);
    let admission = Admission {
        registry: &registry,
        declared_keys: &declared_keys,
        resolved: &resolved,
        owner_declared: format != COMPOSITION_FORMAT,
    };
    let references = resolve_references(specification, &admission, &mut diagnostics);
    let conformances =
        conformance::resolve_conformances(specification, &admission, &mut diagnostics);

    if diagnostics.is_empty() {
        Ok(EssCompositionIr {
            format: format.to_owned(),
            composition: specification.composition.clone(),
            services: resolved,
            references,
            conformances,
        })
    } else {
        Err(CompositionDiagnostics(diagnostics))
    }
}

/// The admitted format marker; an unsupported one reads as `ess-composition/1` after refusal.
fn validate_format(
    specification: &CompositionSpec,
    diagnostics: &mut Vec<CompositionDiagnostic>,
) -> &'static str {
    let format = match specification.format.as_str() {
        COMPOSITION_FORMAT => COMPOSITION_FORMAT,
        CONFORMANCE_COMPOSITION_FORMAT => CONFORMANCE_COMPOSITION_FORMAT,
        READER_COMPOSITION_FORMAT => READER_COMPOSITION_FORMAT,
        other => {
            diagnostics.push(CompositionDiagnostic::new(
                CompositionCode::UnsupportedFormat,
                None,
                format!(
                    "format {other:?} is unsupported; expected {COMPOSITION_FORMAT}, \
                     {CONFORMANCE_COMPOSITION_FORMAT} or {READER_COMPOSITION_FORMAT}"
                ),
            ));
            return COMPOSITION_FORMAT;
        }
    };
    if format == COMPOSITION_FORMAT && specification.conformances.is_some() {
        diagnostics.push(CompositionDiagnostic::new(
            CompositionCode::UnsupportedFormat,
            None,
            format!(
                "`conformances` is an {CONFORMANCE_COMPOSITION_FORMAT} construct; \
                 {COMPOSITION_FORMAT} does not admit it"
            ),
        ));
    }
    if format != READER_COMPOSITION_FORMAT
        && specification
            .conformances()
            .iter()
            .any(TypeConformance::reader_key)
    {
        diagnostics.push(CompositionDiagnostic::new(
            CompositionCode::UnsupportedFormat,
            None,
            format!(
                "`reader` on a conformance is an {READER_COMPOSITION_FORMAT} construct; \
                 {format} does not admit it, whatever its value"
            ),
        ));
    }
    format
}

/// What every reference and conformance end is admitted against.
struct Admission<'a, 'ir> {
    registry: &'a BTreeMap<ServiceKey, &'ir EssIr>,
    declared_keys: &'a BTreeSet<ServiceKey>,
    resolved: &'a BTreeMap<ServiceKey, ResolvedService>,
    /// Whether a type declared in an owned domain is referenceable (from `ess-composition/2`).
    owner_declared: bool,
}

impl<'ir> Admission<'_, 'ir> {
    /// Admits one service-qualified semantic name, returning its compiled model when the selected
    /// component exports it and recording why when it does not.
    fn admit(
        &self,
        service: &ServiceKey,
        semantic: &EssSemanticRef,
        diagnostics: &mut Vec<CompositionDiagnostic>,
    ) -> Option<&'ir EssIr> {
        let Some(ir) = self.registry.get(service).copied() else {
            diagnostics.push(CompositionDiagnostic::new(
                CompositionCode::UnknownReferenceService,
                Some(service.clone()),
                format!(
                    "reference `{semantic}` selects service `{service}`, which has no compiled input"
                ),
            ));
            return None;
        };
        if !self.declared_keys.contains(service) {
            diagnostics.push(CompositionDiagnostic::new(
                CompositionCode::UnknownReferenceService,
                Some(service.clone()),
                format!(
                    "reference `{semantic}` selects service `{service}`, which is not imported"
                ),
            ));
            return None;
        }
        if !ir.resolves(semantic) {
            diagnostics.push(CompositionDiagnostic::new(
                CompositionCode::UnresolvedSemanticReference,
                Some(service.clone()),
                format!("service `{service}` does not resolve `{semantic}`"),
            ));
            return None;
        }
        let resolved = self.resolved.get(service)?;
        if resolved.exports(semantic, self.owner_declared) {
            return Some(ir);
        }
        let hint = if !self.owner_declared && resolved.exports(semantic, true) {
            format!(
                "; a type its owned domains declare is referenceable under \
                 {CONFORMANCE_COMPOSITION_FORMAT}"
            )
        } else {
            String::new()
        };
        diagnostics.push(CompositionDiagnostic::new(
            CompositionCode::ReferenceOutsideComponent,
            Some(service.clone()),
            format!(
                "service `{service}` resolves `{semantic}`, but selected component `{}` does not \
                 export it{hint}",
                resolved.component
            ),
        ));
        None
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

    let declared_types = component
        .owns
        .iter()
        .flat_map(|domain| ir.domain(domain).types.iter().map(DeclaredTypeRef::from))
        .collect();

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
        declared_types,
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
    admission: &Admission<'_, '_>,
    diagnostics: &mut Vec<CompositionDiagnostic>,
) -> BTreeSet<CompositionRef> {
    let mut references = BTreeSet::new();
    for reference in &specification.references {
        if admission
            .admit(&reference.service, &reference.semantic, diagnostics)
            .is_some()
        {
            references.insert(reference.clone());
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
