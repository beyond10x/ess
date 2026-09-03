use std::collections::{BTreeMap, BTreeSet};

use semver::{Version, VersionReq};

use crate::diagnostic::{Diagnostic, DiagnosticCode, Diagnostics, Stage};
use crate::identity::{canonical_json, Digest, Identifier};
use crate::release::{verify_release_document, Artifact, ReleaseManifest};
use crate::runtime::{ConfigKind, RuntimeIr};

/// Generic deployable stack format.
pub const STACK_FORMAT: &str = "ess-stack/1";
/// Exact resolved stack-lock format.
pub const STACK_LOCK_FORMAT: &str = "ess-stack-lock/1";
/// Offline release catalogue format.
pub const RELEASE_CATALOG_FORMAT: &str = "ess-release-catalog/1";

/// One first-party system required by a stack.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SystemRequirement {
    /// Composition-local service identity.
    pub service: Identifier,
    /// Required ESS system identity.
    pub system: Identifier,
    /// Required ESS semantic major, such as `v1`.
    pub semantic_version: String,
    /// Compatible runtime implementation releases.
    pub runtime_release: VersionReq,
    /// Compatible independently versioned chart releases.
    pub chart_release: VersionReq,
    /// Required semantic surfaces, carried for compatibility verification.
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub surfaces: BTreeSet<String>,
    /// Explicit rollout prerequisites by service identity.
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub depends_on: BTreeSet<Identifier>,
}

/// A typed environment-owned or externally managed system.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExternalSystem {
    /// Stack-local identity.
    pub system: Identifier,
    /// Contract family expected from the provider.
    pub contract: String,
    /// Whether this stack is expected to deploy the provider.
    #[serde(default)]
    pub managed: bool,
    /// Required named endpoints.
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub endpoints: BTreeSet<Identifier>,
    /// Required authority or token audiences.
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub authorities: BTreeSet<String>,
    /// Required non-secret configuration coordinates.
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub config: BTreeSet<Identifier>,
}

/// Human-authored generic product stack.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StackSpec {
    format: String,
    /// Stable stack identity.
    pub stack: Identifier,
    /// Exact semantic composition digest.
    pub composition_digest: Digest,
    /// First-party system requirements.
    pub systems: Vec<SystemRequirement>,
    /// Typed external-system requirements.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub external_systems: Vec<ExternalSystem>,
}

impl StackSpec {
    /// Reads a strict JSON stack specification.
    pub fn from_json(text: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(text)
    }

    /// Reads a strict YAML stack specification.
    pub fn from_yaml(text: &str) -> Result<Self, serde_yaml::Error> {
        serde_yaml::from_str(text)
    }
}

/// One release available to the offline stack resolver.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReleaseCandidate {
    /// ESS semantic major supplied by this release.
    pub semantic_version: String,
    /// Semantic surfaces supplied by this release.
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub surfaces: BTreeSet<String>,
    /// Exact immutable release manifest.
    pub release: ReleaseManifest,
    /// Exact runtime used to derive deployment requirements.
    pub runtime: RuntimeIr,
}

/// Explicit offline input to stack resolution.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReleaseCatalog {
    format: String,
    /// Available releases in arbitrary input order.
    pub releases: Vec<ReleaseCandidate>,
}

impl ReleaseCatalog {
    /// Reads a strict JSON release catalogue.
    pub fn from_json(text: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(text)
    }

    /// Reads a strict YAML release catalogue.
    pub fn from_yaml(text: &str) -> Result<Self, serde_yaml::Error> {
        serde_yaml::from_str(text)
    }
}

/// Environment inputs required by a locked system realization.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeRequirements {
    /// Required non-secret configuration slots.
    pub config: BTreeMap<Identifier, ConfigKind>,
    /// Required secret slot names.
    pub secrets: BTreeSet<Identifier>,
    /// Required endpoint slots and target systems.
    pub endpoints: BTreeMap<Identifier, Identifier>,
    /// Required workload token audiences.
    pub audiences: BTreeSet<String>,
}

/// Exact first-party release selected for one stack service.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LockedSystem {
    /// ESS system identity.
    pub system: Identifier,
    /// ESS semantic major.
    pub semantic_version: String,
    /// Independent release-unit identity.
    pub release_unit: Identifier,
    /// Exact release version.
    pub version: Version,
    /// Exact release-manifest digest.
    pub release_digest: Digest,
    /// Exact semantic digest.
    pub semantic_digest: Digest,
    /// Exact build digest.
    pub build_digest: Digest,
    /// Exact runtime digest.
    pub runtime_digest: Digest,
    /// Immutable runtime artifacts.
    pub runtime_artifacts: BTreeMap<Identifier, Artifact>,
    /// Independent chart release-unit identity.
    pub chart_release_unit: Identifier,
    /// Exact independent chart version.
    pub chart_version: Version,
    /// Exact independent chart release-manifest digest.
    pub chart_release_digest: Digest,
    /// Exact chart artifact.
    pub chart: Artifact,
    /// Required environment bindings derived from runtime IR.
    pub runtime: RuntimeRequirements,
    /// Explicit rollout prerequisites.
    pub depends_on: BTreeSet<Identifier>,
}

/// Exact, reviewable resolution of a generic stack.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StackLock {
    format: String,
    /// Stable stack identity.
    pub stack: Identifier,
    /// Digest of the authored stack specification.
    pub stack_digest: Digest,
    /// Exact semantic composition digest.
    pub composition_digest: Digest,
    /// Exact releases by composition-local service identity.
    pub systems: BTreeMap<Identifier, LockedSystem>,
    /// Typed external requirements copied from the stack.
    pub external_systems: BTreeMap<Identifier, ExternalSystem>,
}

impl StackLock {
    /// Reads a strict exact stack lock.
    pub fn from_json(text: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(text)
    }

    /// Canonical JSON with a trailing newline.
    pub fn to_canonical_json(&self) -> String {
        canonical_json(self)
    }

    /// Digest of this exact stack lock.
    pub fn digest(&self) -> Digest {
        Digest::of_bytes(self.to_canonical_json().as_bytes())
    }
}

/// Resolve a stack deterministically from an explicit offline release catalogue.
#[allow(clippy::too_many_lines)]
pub fn resolve_stack(
    specification: &StackSpec,
    catalog: &ReleaseCatalog,
) -> Result<StackLock, Diagnostics> {
    let mut diagnostics = Vec::new();
    if specification.format != STACK_FORMAT {
        diagnostics.push(Diagnostic::new(
            Stage::Composition,
            DiagnosticCode::UnsupportedFormat,
            Some(specification.stack.clone()),
            format!("stack format must be {STACK_FORMAT:?}"),
        ));
    }
    if catalog.format != RELEASE_CATALOG_FORMAT {
        diagnostics.push(Diagnostic::new(
            Stage::Composition,
            DiagnosticCode::UnsupportedFormat,
            Some(specification.stack.clone()),
            format!("release catalogue format must be {RELEASE_CATALOG_FORMAT:?}"),
        ));
    }

    let mut requirements = BTreeMap::new();
    for requirement in &specification.systems {
        if requirements
            .insert(requirement.service.clone(), requirement)
            .is_some()
        {
            diagnostics.push(Diagnostic::new(
                Stage::Composition,
                DiagnosticCode::DuplicateIdentifier,
                Some(requirement.service.clone()),
                format!(
                    "stack service {} is declared more than once",
                    requirement.service
                ),
            ));
        }
        for dependency in &requirement.depends_on {
            if dependency == &requirement.service {
                diagnostics.push(Diagnostic::new(
                    Stage::Composition,
                    DiagnosticCode::DependencyCycle,
                    Some(requirement.service.clone()),
                    "a stack service cannot depend on itself",
                ));
            }
        }
    }
    for requirement in requirements.values() {
        for dependency in &requirement.depends_on {
            if !requirements.contains_key(dependency) {
                diagnostics.push(Diagnostic::new(
                    Stage::Composition,
                    DiagnosticCode::UnknownReference,
                    Some(requirement.service.clone()),
                    format!("rollout dependency {dependency} is not a declared stack service"),
                ));
            }
        }
    }
    if has_requirement_cycle(&requirements) {
        diagnostics.push(Diagnostic::new(
            Stage::Composition,
            DiagnosticCode::DependencyCycle,
            Some(specification.stack.clone()),
            "the explicit stack rollout graph contains a cycle",
        ));
    }

    let mut external_systems = BTreeMap::new();
    for external in &specification.external_systems {
        if external_systems
            .insert(external.system.clone(), external.clone())
            .is_some()
        {
            diagnostics.push(Diagnostic::new(
                Stage::Composition,
                DiagnosticCode::DuplicateIdentifier,
                Some(external.system.clone()),
                format!(
                    "external system {} is declared more than once",
                    external.system
                ),
            ));
        }
    }

    let mut systems = BTreeMap::new();
    for requirement in requirements.values() {
        let candidate = catalog
            .releases
            .iter()
            .filter(|candidate| {
                candidate.release.system == requirement.system
                    && candidate.semantic_version == requirement.semantic_version
                    && requirement
                        .runtime_release
                        .matches(&candidate.release.version)
                    && requirement.surfaces.is_subset(&candidate.surfaces)
                    && candidate
                        .release
                        .artifacts
                        .values()
                        .any(|artifact| artifact.kind == crate::release::ArtifactKind::OciImage)
            })
            .max_by(|left, right| {
                left.release
                    .version
                    .cmp(&right.release.version)
                    .then_with(|| left.release.digest().cmp(&right.release.digest()))
            });
        let Some(candidate) = candidate else {
            diagnostics.push(Diagnostic::new(
                Stage::Composition,
                DiagnosticCode::UnsatisfiedConstraint,
                Some(requirement.service.clone()),
                format!(
                    "no release of {} satisfies semantic {}, release {}, and required surfaces",
                    requirement.system, requirement.semantic_version, requirement.runtime_release
                ),
            ));
            continue;
        };
        let chart_candidate =
            catalog
                .releases
                .iter()
                .filter(|chart| {
                    chart.release.system == requirement.system
                        && chart.semantic_version == requirement.semantic_version
                        && requirement.chart_release.matches(&chart.release.version)
                        && requirement.surfaces.is_subset(&chart.surfaces)
                        && chart.release.artifacts.values().any(|artifact| {
                            artifact.kind == crate::release::ArtifactKind::HelmChart
                        })
                        && chart.release.semantic_digest == candidate.release.semantic_digest
                        && chart.release.runtime_digest == candidate.release.runtime_digest
                })
                .max_by(|left, right| {
                    left.release
                        .version
                        .cmp(&right.release.version)
                        .then_with(|| left.release.digest().cmp(&right.release.digest()))
                });
        let Some(chart_candidate) = chart_candidate else {
            diagnostics.push(Diagnostic::new(
                Stage::Composition,
                DiagnosticCode::UnsatisfiedConstraint,
                Some(requirement.service.clone()),
                format!(
                    "no chart release of {} satisfies {}, semantic digest {}, and realization digest {}",
                    requirement.system,
                    requirement.chart_release,
                    candidate.release.semantic_digest,
                    candidate.release.runtime_digest
                ),
            ));
            continue;
        };
        diagnostics.extend(verify_release_document(&candidate.release));
        diagnostics.extend(verify_release_document(&chart_candidate.release));
        if candidate.release.runtime_digest != candidate.runtime.digest()
            || candidate.release.semantic_digest != *candidate.runtime.semantic_digest()
        {
            diagnostics.push(Diagnostic::new(
                Stage::Composition,
                DiagnosticCode::DigestMismatch,
                Some(requirement.service.clone()),
                "catalogue runtime does not match the selected release manifest",
            ));
            continue;
        }
        if chart_candidate.release.runtime_digest != chart_candidate.runtime.digest()
            || chart_candidate.release.runtime_digest != candidate.release.runtime_digest
        {
            diagnostics.push(Diagnostic::new(
                Stage::Composition,
                DiagnosticCode::DigestMismatch,
                Some(requirement.service.clone()),
                "selected chart release was not projected from the runtime realization",
            ));
            continue;
        }
        let charts: Vec<_> = chart_candidate
            .release
            .artifacts
            .values()
            .filter(|artifact| artifact.kind == crate::release::ArtifactKind::HelmChart)
            .cloned()
            .collect();
        if charts.len() != 1 {
            diagnostics.push(Diagnostic::new(
                Stage::Composition,
                DiagnosticCode::MissingOutput,
                Some(requirement.service.clone()),
                format!(
                    "a chart release must contain exactly one Helm chart; found {}",
                    charts.len()
                ),
            ));
            continue;
        }
        systems.insert(
            requirement.service.clone(),
            LockedSystem {
                system: candidate.release.system.clone(),
                semantic_version: candidate.semantic_version.clone(),
                release_unit: candidate.release.release_unit.clone(),
                version: candidate.release.version.clone(),
                release_digest: candidate.release.digest(),
                semantic_digest: candidate.release.semantic_digest.clone(),
                build_digest: candidate.release.build_digest.clone(),
                runtime_digest: candidate.release.runtime_digest.clone(),
                runtime_artifacts: candidate.release.artifacts.clone(),
                chart_release_unit: chart_candidate.release.release_unit.clone(),
                chart_version: chart_candidate.release.version.clone(),
                chart_release_digest: chart_candidate.release.digest(),
                chart: charts[0].clone(),
                runtime: runtime_requirements(&candidate.runtime),
                depends_on: requirement.depends_on.clone(),
            },
        );
    }

    if diagnostics.is_empty() {
        let stack_digest = Digest::of_bytes(canonical_json(specification).as_bytes());
        Ok(StackLock {
            format: STACK_LOCK_FORMAT.to_owned(),
            stack: specification.stack.clone(),
            stack_digest,
            composition_digest: specification.composition_digest.clone(),
            systems,
            external_systems,
        })
    } else {
        Err(Diagnostics::from(diagnostics))
    }
}

fn runtime_requirements(realization: &RuntimeIr) -> RuntimeRequirements {
    let mut config = BTreeMap::new();
    let mut secrets = BTreeSet::new();
    let mut endpoints = BTreeMap::new();
    let mut audiences = BTreeSet::new();
    for container in realization.containers().values() {
        for slot in &container.config {
            if slot.kind != ConfigKind::Literal {
                config.insert(slot.name.clone(), slot.kind);
            }
        }
        secrets.extend(container.secrets.iter().map(|slot| slot.name.clone()));
        endpoints.extend(
            container
                .endpoints
                .iter()
                .map(|slot| (slot.name.clone(), slot.system.clone())),
        );
        audiences.extend(container.audiences.iter().cloned());
    }
    RuntimeRequirements {
        config,
        secrets,
        endpoints,
        audiences,
    }
}

fn has_requirement_cycle(requirements: &BTreeMap<Identifier, &SystemRequirement>) -> bool {
    fn visit(
        node: &Identifier,
        requirements: &BTreeMap<Identifier, &SystemRequirement>,
        visiting: &mut BTreeSet<Identifier>,
        visited: &mut BTreeSet<Identifier>,
    ) -> bool {
        if visited.contains(node) {
            return false;
        }
        if !visiting.insert(node.clone()) {
            return true;
        }
        if let Some(requirement) = requirements.get(node) {
            for dependency in &requirement.depends_on {
                if visit(dependency, requirements, visiting, visited) {
                    return true;
                }
            }
        }
        visiting.remove(node);
        visited.insert(node.clone());
        false
    }

    let mut visiting = BTreeSet::new();
    let mut visited = BTreeSet::new();
    requirements
        .keys()
        .any(|node| visit(node, requirements, &mut visiting, &mut visited))
}
