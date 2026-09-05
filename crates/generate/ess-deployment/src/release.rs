use std::collections::{BTreeMap, BTreeSet};

use semver::Version;

use crate::build::{BuildIr, BuildOutputKind};
use crate::diagnostic::{Diagnostic, DiagnosticCode, Diagnostics, Stage};
use crate::identity::{canonical_json, Digest, Identifier};
use crate::runtime::RuntimeIr;

/// Immutable release-manifest format.
pub const RELEASE_FORMAT: &str = "ess-release/1";

/// Published artifact kind.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactKind {
    /// OCI image or multi-platform image index.
    OciImage,
    /// Executable or library file.
    Binary,
    /// General archive.
    Archive,
    /// Packaged Helm chart.
    HelmChart,
}

impl From<BuildOutputKind> for ArtifactKind {
    fn from(value: BuildOutputKind) -> Self {
        match value {
            BuildOutputKind::OciImage => Self::OciImage,
            BuildOutputKind::Binary => Self::Binary,
            BuildOutputKind::Archive => Self::Archive,
            BuildOutputKind::HelmChart => Self::HelmChart,
        }
    }
}

/// One immutable release artifact.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Artifact {
    /// Corresponding named build output.
    pub build_output: Identifier,
    /// Artifact kind.
    pub kind: ArtifactKind,
    /// Registry or download coordinate. Identity remains the digest.
    pub reference: String,
    /// Exact artifact or OCI index digest.
    pub digest: Digest,
    /// Exact OCI child manifests by platform.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    #[serde(deserialize_with = "crate::validation::unique_map")]
    pub platforms: BTreeMap<String, Digest>,
}

/// Required release evidence kind.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, serde::Serialize, serde::Deserialize,
)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceKind {
    /// Build provenance or SLSA statement.
    Provenance,
    /// Software bill of materials.
    Sbom,
    /// Cryptographic artifact signature.
    Signature,
    /// Semantic/runtime conformance result.
    Conformance,
}

/// One immutable evidence attachment.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Evidence {
    /// Evidence artifact coordinate.
    pub reference: String,
    /// Exact evidence digest.
    pub digest: Digest,
}

/// Executor-produced immutable release record.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct ReleaseManifest {
    format: String,
    /// Independently versioned release unit.
    pub release_unit: Identifier,
    /// System whose implementation is released.
    pub system: Identifier,
    /// Release version, separate from the ESS semantic major.
    pub version: Version,
    /// Exact source revision.
    pub source_commit: String,
    /// Exact compiled ESS semantic digest.
    pub semantic_digest: Digest,
    /// Exact compiled build digest.
    pub build_digest: Digest,
    /// Exact compiled runtime digest.
    pub runtime_digest: Digest,
    /// Published artifacts by stable output name.
    #[serde(deserialize_with = "crate::validation::unique_map")]
    pub artifacts: BTreeMap<Identifier, Artifact>,
    /// Release evidence by required kind.
    #[serde(deserialize_with = "crate::validation::unique_map")]
    pub evidence: BTreeMap<EvidenceKind, Evidence>,
}

crate::validation::checked_deserialize!(ReleaseManifest {
    format: String,
    /// Independently versioned release unit.
    pub release_unit: Identifier,
    /// System whose implementation is released.
    pub system: Identifier,
    /// Release version, separate from the ESS semantic major.
    pub version: Version,
    /// Exact source revision.
    pub source_commit: String,
    /// Exact compiled ESS semantic digest.
    pub semantic_digest: Digest,
    /// Exact compiled build digest.
    pub build_digest: Digest,
    /// Exact compiled runtime digest.
    pub runtime_digest: Digest,
    /// Published artifacts by stable output name.
    #[serde(deserialize_with = "crate::validation::unique_map")]
    pub artifacts: BTreeMap<Identifier, Artifact>,
    /// Release evidence by required kind.
    #[serde(deserialize_with = "crate::validation::unique_map")]
    pub evidence: BTreeMap<EvidenceKind, Evidence>,
});

impl ReleaseManifest {
    /// Recheck manifest-local rules after mutation; attachments do not prove authenticity.
    pub fn validate(&self) -> Result<(), Diagnostics> {
        crate::validation::finish(verify_release_document(self))
    }

    /// Reads a strict release manifest.
    pub fn from_json(text: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(text)
    }

    /// Reads a strict YAML release manifest.
    pub fn from_yaml(text: &str) -> Result<Self, serde_yaml::Error> {
        serde_yaml::from_str(text)
    }

    /// Persisted format marker.
    pub fn format(&self) -> &str {
        &self.format
    }

    /// Canonical JSON with a trailing newline.
    pub fn to_canonical_json(&self) -> String {
        canonical_json(self)
    }

    /// Digest of this exact release manifest.
    pub fn digest(&self) -> Digest {
        Digest::of_bytes(self.to_canonical_json().as_bytes())
    }
}

/// Verify an executor-produced release against the exact build and runtime it claims.
pub fn verify_release(
    release: &ReleaseManifest,
    build: &BuildIr,
    runtime: &RuntimeIr,
) -> Result<(), Diagnostics> {
    let mut diagnostics = verify_release_document(release);
    if let Err(errors) = runtime.validate_against_build(build) {
        diagnostics.extend_from_slice(errors.as_slice());
    }

    if release.build_digest != build.digest() {
        diagnostics.push(Diagnostic::new(
            Stage::Release,
            DiagnosticCode::DigestMismatch,
            Some(release.release_unit.clone()),
            "release build digest does not match the supplied build IR",
        ));
    }
    if release.runtime_digest != runtime.digest() {
        diagnostics.push(Diagnostic::new(
            Stage::Release,
            DiagnosticCode::DigestMismatch,
            Some(release.release_unit.clone()),
            "release runtime digest does not match the supplied runtime IR",
        ));
    }
    if release.semantic_digest != *runtime.semantic_digest() {
        diagnostics.push(Diagnostic::new(
            Stage::Release,
            DiagnosticCode::DigestMismatch,
            Some(release.release_unit.clone()),
            "release semantic digest does not match the supplied runtime IR",
        ));
    }

    let owned_outputs: BTreeMap<_, _> = build
        .outputs()
        .iter()
        .filter(|(_, output)| output.release_unit == release.release_unit)
        .collect();
    for (name, output) in &owned_outputs {
        match release.artifacts.get(name) {
            Some(artifact)
                if artifact.build_output == **name
                    && artifact.kind == ArtifactKind::from(output.kind) => {}
            Some(_) => diagnostics.push(Diagnostic::new(
                Stage::Release,
                DiagnosticCode::DigestMismatch,
                Some((*name).clone()),
                "release artifact kind or build-output identity does not match build IR",
            )),
            None => diagnostics.push(Diagnostic::new(
                Stage::Release,
                DiagnosticCode::MissingOutput,
                Some((*name).clone()),
                format!("release omits build output {name}"),
            )),
        }
    }
    for name in release.artifacts.keys() {
        if !owned_outputs.contains_key(name) {
            diagnostics.push(Diagnostic::new(
                Stage::Release,
                DiagnosticCode::UnknownReference,
                Some(name.clone()),
                format!("release includes artifact {name} not declared by the build"),
            ));
        }
    }

    if diagnostics.is_empty() {
        Ok(())
    } else {
        Err(Diagnostics::from(diagnostics))
    }
}

pub(crate) fn verify_release_document(release: &ReleaseManifest) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    if release.format != RELEASE_FORMAT {
        diagnostics.push(Diagnostic::new(
            Stage::Release,
            DiagnosticCode::UnsupportedFormat,
            Some(release.release_unit.clone()),
            format!(
                "release format {:?} is unsupported; expected {RELEASE_FORMAT:?}",
                release.format
            ),
        ));
    }
    if !matches!(release.source_commit.len(), 40 | 64)
        || !release
            .source_commit
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        diagnostics.push(Diagnostic::new(
            Stage::Release,
            DiagnosticCode::InvalidValue,
            Some(release.release_unit.clone()),
            "source_commit must be a complete lowercase hexadecimal Git object id",
        ));
    }
    if release.artifacts.is_empty() {
        diagnostics.push(Diagnostic::new(
            Stage::Release,
            DiagnosticCode::MissingOutput,
            Some(release.release_unit.clone()),
            "a release must contain at least one immutable artifact",
        ));
    }
    validate_artifacts(&release.artifacts, Stage::Release, &mut diagnostics);

    let required = BTreeSet::from([
        EvidenceKind::Provenance,
        EvidenceKind::Sbom,
        EvidenceKind::Signature,
        EvidenceKind::Conformance,
    ]);
    for kind in required {
        if !release.evidence.contains_key(&kind) {
            diagnostics.push(Diagnostic::new(
                Stage::Release,
                DiagnosticCode::MissingEvidence,
                Some(release.release_unit.clone()),
                format!("release is missing required {kind:?} evidence"),
            ));
        }
    }
    diagnostics
}

/// Rules shared by manifest, lock and deployment artifact maps.
pub(crate) fn validate_artifacts(
    artifacts: &BTreeMap<Identifier, Artifact>,
    stage: Stage,
    diagnostics: &mut Vec<Diagnostic>,
) {
    for (name, artifact) in artifacts {
        if name != &artifact.build_output {
            diagnostics.push(Diagnostic::new(
                stage,
                DiagnosticCode::DigestMismatch,
                Some(name.clone()),
                "artifact map key must equal its build_output identity",
            ));
        }
        if artifact.kind == ArtifactKind::OciImage && artifact.platforms.is_empty() {
            diagnostics.push(Diagnostic::new(
                stage,
                DiagnosticCode::MissingOutput,
                Some(name.clone()),
                "an OCI image release must record exact per-platform child manifests",
            ));
        }
    }
}
