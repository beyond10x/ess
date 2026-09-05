use std::collections::{BTreeMap, BTreeSet};
use std::path::{Component, Path};

use crate::build::BuildIr;
use crate::diagnostic::{Diagnostic, DiagnosticCode, Diagnostics, Stage};
use crate::identity::{canonical_json, Digest, Identifier};
use crate::release::{verify_release, ArtifactKind, ReleaseManifest};
use crate::runtime::RuntimeIr;

/// Repository-owned component descriptor format.
pub const COMPONENT_FORMAT: &str = "ess-component/1";
/// Canonical compiled component descriptor format.
pub const COMPONENT_IR_FORMAT: &str = "ess-component-ir/1";
/// Immutable OCI component-bundle payload format.
pub const RELEASE_BUNDLE_FORMAT: &str = "ess-release-bundle/1";

/// Independently versioned release units emitted for one component.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ComponentReleaseUnits {
    /// Release unit containing executable runtime artifacts.
    pub runtime: Identifier,
    /// Release unit containing the configuration-neutral Helm chart.
    pub chart: Identifier,
}

/// Paths to the existing ESS documents which define one deployable component.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ComponentInputs {
    /// ESS semantic source directory or root file.
    pub specification: String,
    /// `ess-realization/1` document.
    pub realization: String,
    /// `ess-build/1` document.
    pub build: String,
    /// `ess-runtime/1` or later compatible document.
    pub runtime: String,
}

/// Human-authored repository manifest for one independently releasable component.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ComponentSpec {
    format: String,
    /// Stable component identity within build and release automation.
    pub component: Identifier,
    /// ESS system implemented by this component.
    pub system: Identifier,
    /// Compatible ESS semantic version, such as `v1`.
    pub semantic_version: String,
    /// Repository-relative ESS inputs.
    pub inputs: ComponentInputs,
    /// Independent runtime and chart release units.
    pub release_units: ComponentReleaseUnits,
}

impl ComponentSpec {
    /// Reads a strict JSON component descriptor.
    pub fn from_json(text: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(text)
    }

    /// Reads a strict YAML component descriptor.
    pub fn from_yaml(text: &str) -> Result<Self, serde_yaml::Error> {
        serde_yaml::from_str(text)
    }
}

/// Validated, canonical component descriptor.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct ComponentIr {
    format: String,
    component: Identifier,
    system: Identifier,
    semantic_version: String,
    inputs: ComponentInputs,
    release_units: ComponentReleaseUnits,
}

crate::validation::checked_deserialize!(ComponentIr {
    format: String,
    component: Identifier,
    system: Identifier,
    semantic_version: String,
    inputs: ComponentInputs,
    release_units: ComponentReleaseUnits,
});

impl ComponentIr {
    /// Recheck descriptor invariants without opening the referenced inputs.
    pub fn validate(&self) -> Result<(), Diagnostics> {
        let mut diagnostics = Vec::new();
        crate::validation::check(
            &mut diagnostics,
            self.format == COMPONENT_IR_FORMAT,
            Stage::Runtime,
            DiagnosticCode::UnsupportedFormat,
            &self.component,
            format!("component IR format must be {COMPONENT_IR_FORMAT:?}"),
        );
        let specification = ComponentSpec {
            format: COMPONENT_FORMAT.to_owned(),
            component: self.component.clone(),
            system: self.system.clone(),
            semantic_version: self.semantic_version.clone(),
            inputs: self.inputs.clone(),
            release_units: self.release_units.clone(),
        };
        if let Err(errors) = compile_component(&specification) {
            diagnostics.extend_from_slice(errors.as_slice());
        }
        crate::validation::finish(diagnostics)
    }

    /// Reads strict compiler-owned component IR.
    pub fn from_json(text: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(text)
    }

    /// Stable component identity.
    pub fn component(&self) -> &Identifier {
        &self.component
    }

    /// ESS system supplied by the component.
    pub fn system(&self) -> &Identifier {
        &self.system
    }

    /// Compatible ESS semantic version.
    pub fn semantic_version(&self) -> &str {
        &self.semantic_version
    }

    /// Repository-relative inputs.
    pub fn inputs(&self) -> &ComponentInputs {
        &self.inputs
    }

    /// Independently versioned release units.
    pub fn release_units(&self) -> &ComponentReleaseUnits {
        &self.release_units
    }

    /// Canonical JSON with a trailing newline.
    pub fn to_canonical_json(&self) -> String {
        canonical_json(self)
    }

    /// Digest of the exact descriptor.
    pub fn digest(&self) -> Digest {
        Digest::of_bytes(self.to_canonical_json().as_bytes())
    }
}

/// Validate a repository-owned component descriptor without touching its referenced files.
pub fn compile_component(specification: &ComponentSpec) -> Result<ComponentIr, Diagnostics> {
    let mut diagnostics = Vec::new();
    if specification.format != COMPONENT_FORMAT {
        diagnostics.push(Diagnostic::new(
            Stage::Runtime,
            DiagnosticCode::UnsupportedFormat,
            Some(specification.component.clone()),
            format!(
                "component format {:?} is unsupported; expected {COMPONENT_FORMAT:?}",
                specification.format
            ),
        ));
    }
    if !specification
        .semantic_version
        .strip_prefix('v')
        .is_some_and(|major| !major.is_empty() && major.bytes().all(|byte| byte.is_ascii_digit()))
    {
        diagnostics.push(Diagnostic::new(
            Stage::Runtime,
            DiagnosticCode::InvalidValue,
            Some(specification.component.clone()),
            "component semantic_version must be a major written v<digits>",
        ));
    }
    for (name, path) in [
        ("specification", &specification.inputs.specification),
        ("realization", &specification.inputs.realization),
        ("build", &specification.inputs.build),
        ("runtime", &specification.inputs.runtime),
    ] {
        if !safe_relative_path(path) {
            diagnostics.push(Diagnostic::new(
                Stage::Runtime,
                DiagnosticCode::InvalidValue,
                Some(specification.component.clone()),
                format!("component {name} path must be repository-relative and may not traverse parents"),
            ));
        }
    }
    if specification.release_units.runtime == specification.release_units.chart {
        diagnostics.push(Diagnostic::new(
            Stage::Release,
            DiagnosticCode::DuplicateIdentifier,
            Some(specification.component.clone()),
            "runtime and chart must be independent release units",
        ));
    }
    if diagnostics.is_empty() {
        Ok(ComponentIr {
            format: COMPONENT_IR_FORMAT.to_owned(),
            component: specification.component.clone(),
            system: specification.system.clone(),
            semantic_version: specification.semantic_version.clone(),
            inputs: specification.inputs.clone(),
            release_units: specification.release_units.clone(),
        })
    } else {
        Err(Diagnostics::from(diagnostics))
    }
}

fn safe_relative_path(value: &str) -> bool {
    !value.is_empty()
        && !Path::new(value).is_absolute()
        && Path::new(value)
            .components()
            .all(|component| matches!(component, Component::Normal(_) | Component::CurDir))
}

/// Canonical release metadata transported as one digest-addressed OCI artifact.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct ReleaseBundle {
    format: String,
    /// Exact repository-owned component descriptor.
    pub component: ComponentIr,
    /// Exact compiled build graph.
    pub build: BuildIr,
    /// Exact compiled runtime realization.
    pub runtime: RuntimeIr,
    /// Runtime and chart release manifests by release-unit identity.
    #[serde(deserialize_with = "crate::validation::unique_map")]
    pub releases: BTreeMap<Identifier, ReleaseManifest>,
}

crate::validation::checked_deserialize!(ReleaseBundle {
    format: String,
    /// Exact repository-owned component descriptor.
    pub component: ComponentIr,
    /// Exact compiled build graph.
    pub build: BuildIr,
    /// Exact compiled runtime realization.
    pub runtime: RuntimeIr,
    /// Runtime and chart release manifests by release-unit identity.
    #[serde(deserialize_with = "crate::validation::unique_map")]
    pub releases: BTreeMap<Identifier, ReleaseManifest>,
});

impl ReleaseBundle {
    /// Validate the complete included graphs and their relationships after public-field mutation.
    /// Digests do not prove omitted semantic inputs or release authenticity.
    pub fn validate(&self) -> Result<(), Diagnostics> {
        let mut diagnostics = Vec::new();
        crate::validation::check(
            &mut diagnostics,
            self.format == RELEASE_BUNDLE_FORMAT,
            Stage::Release,
            DiagnosticCode::UnsupportedFormat,
            self.component.component(),
            format!("release bundle format must be {RELEASE_BUNDLE_FORMAT:?}"),
        );
        for (name, release) in &self.releases {
            crate::validation::check(
                &mut diagnostics,
                name == &release.release_unit,
                Stage::Release,
                DiagnosticCode::InvalidValue,
                name,
                "release map key must match its release unit",
            );
        }
        if let Err(errors) = bundle_release(
            self.component.clone(),
            self.build.clone(),
            self.runtime.clone(),
            self.releases.values().cloned().collect(),
        ) {
            diagnostics.extend_from_slice(errors.as_slice());
        }
        crate::validation::finish(diagnostics)
    }

    /// Reads a strict JSON release bundle.
    pub fn from_json(text: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(text)
    }

    /// Reads a strict YAML release bundle.
    pub fn from_yaml(text: &str) -> Result<Self, serde_yaml::Error> {
        serde_yaml::from_str(text)
    }

    /// Canonical JSON with a trailing newline.
    pub fn to_canonical_json(&self) -> String {
        canonical_json(self)
    }

    /// Digest of the complete verified bundle payload.
    pub fn digest(&self) -> Digest {
        Digest::of_bytes(self.to_canonical_json().as_bytes())
    }
}

/// Construct and verify one release bundle from existing ESS release artifacts.
pub fn bundle_release(
    component: ComponentIr,
    build: BuildIr,
    runtime: RuntimeIr,
    releases: Vec<ReleaseManifest>,
) -> Result<ReleaseBundle, Diagnostics> {
    let mut diagnostics = Vec::new();
    if let Err(errors) = runtime.validate_against_build(&build) {
        diagnostics.extend_from_slice(errors.as_slice());
    }

    let mut indexed = BTreeMap::new();
    for release in releases {
        let release_unit = release.release_unit.clone();
        if indexed.insert(release_unit.clone(), release).is_some() {
            diagnostics.push(Diagnostic::new(
                Stage::Release,
                DiagnosticCode::DuplicateIdentifier,
                Some(release_unit),
                "release bundle contains the same release unit more than once",
            ));
        }
    }

    let expected = BTreeSet::from([
        component.release_units.runtime.clone(),
        component.release_units.chart.clone(),
    ]);
    let actual = indexed.keys().cloned().collect::<BTreeSet<_>>();
    for missing in expected.difference(&actual) {
        diagnostics.push(Diagnostic::new(
            Stage::Release,
            DiagnosticCode::MissingOutput,
            Some(missing.clone()),
            "release bundle omits a declared component release unit",
        ));
    }
    for extra in actual.difference(&expected) {
        diagnostics.push(Diagnostic::new(
            Stage::Release,
            DiagnosticCode::UnknownReference,
            Some(extra.clone()),
            "release bundle contains a release unit absent from the component descriptor",
        ));
    }
    for release in indexed.values() {
        if release.system != component.system {
            diagnostics.push(Diagnostic::new(
                Stage::Release,
                DiagnosticCode::DigestMismatch,
                Some(release.release_unit.clone()),
                "release system differs from the component system",
            ));
        }
        if let Err(refusal) = verify_release(release, &build, &runtime) {
            diagnostics.extend(refusal.as_slice().iter().cloned());
        }
    }
    verify_release_kind(
        indexed.get(&component.release_units.runtime),
        ArtifactKind::OciImage,
        &component.release_units.runtime,
        &mut diagnostics,
    );
    verify_release_kind(
        indexed.get(&component.release_units.chart),
        ArtifactKind::HelmChart,
        &component.release_units.chart,
        &mut diagnostics,
    );

    if diagnostics.is_empty() {
        Ok(ReleaseBundle {
            format: RELEASE_BUNDLE_FORMAT.to_owned(),
            component,
            build,
            runtime,
            releases: indexed,
        })
    } else {
        Err(Diagnostics::from(diagnostics))
    }
}

/// Verify a bundle read from an untrusted OCI layer.
pub fn verify_release_bundle(bundle: ReleaseBundle) -> Result<ReleaseBundle, Diagnostics> {
    bundle.validate()?;
    Ok(bundle)
}

fn verify_release_kind(
    release: Option<&ReleaseManifest>,
    expected: ArtifactKind,
    release_unit: &Identifier,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let Some(release) = release else {
        return;
    };
    if !release
        .artifacts
        .values()
        .any(|artifact| artifact.kind == expected)
    {
        diagnostics.push(Diagnostic::new(
            Stage::Release,
            DiagnosticCode::MissingOutput,
            Some(release_unit.clone()),
            format!("release unit does not contain its required {expected:?} artifact"),
        ));
    }
}
