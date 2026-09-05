use std::collections::{BTreeMap, BTreeSet};

use ess_compiler::EssIr;
use ess_realization::RealizationIr as PhysicalRealizationIr;

use crate::build::{BuildIr, BuildOutputKind};
use crate::diagnostic::{Diagnostic, DiagnosticCode, Diagnostics, Stage};
use crate::identity::{canonical_json, Digest, Identifier};

/// Human-authored deployable-runtime format.
pub const RUNTIME_FORMAT: &str = "ess-runtime/1";
/// Compiler-owned deployable-runtime format.
pub const RUNTIME_IR_FORMAT: &str = "ess-runtime-ir/1";

/// Classification of a non-secret runtime setting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConfigKind {
    /// Required environment-specific configuration.
    Required,
    /// Optional environment-specific configuration.
    Optional,
    /// A public literal fixed by the realization.
    Literal,
}

/// One non-secret runtime configuration input.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConfigSlot {
    /// Stable configuration identity.
    pub name: Identifier,
    /// Environment variable consumed by the process.
    pub environment: String,
    /// Required, optional, or realization-owned literal.
    pub kind: ConfigKind,
    /// Public literal value; accepted only for `literal` slots.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
}

/// One secret runtime input. The model cannot represent its bytes.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SecretSlot {
    /// Stable secret requirement identity.
    pub name: Identifier,
    /// Environment variable consumed by the process.
    pub environment: String,
    /// Key expected inside the environment-owned secret object.
    pub key: String,
}

/// One external or composed service endpoint required at runtime.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EndpointSlot {
    /// Stable endpoint requirement identity.
    pub name: Identifier,
    /// Environment variable receiving the bound URL.
    pub environment: String,
    /// Stack service or typed external-system identity.
    pub system: Identifier,
    /// Named endpoint exposed by the target system. When present, composition may bind it
    /// directly to another component-owned Service instead of requiring an environment URL.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub endpoint: Option<Identifier>,
}

/// Network protocol exposed by a component-owned endpoint.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EndpointScheme {
    /// Plain HTTP inside the deployment boundary.
    Http,
    /// TLS-protected HTTP.
    Https,
}

impl EndpointScheme {
    /// URI scheme spelling used by generated endpoint bindings.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Http => "http",
            Self::Https => "https",
        }
    }
}

/// A stable network endpoint provided by one workload container.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProvidedEndpoint {
    /// Endpoint identity consumed by other systems.
    pub name: Identifier,
    /// Workload whose pods back the generated Kubernetes Service.
    pub workload: Identifier,
    /// Container role whose HTTP port is exposed.
    pub container: Identifier,
    /// Protocol used to construct in-cluster URLs.
    pub scheme: EndpointScheme,
}

/// One persistent volume mounted by a container role.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VolumeMount {
    /// Workload volume identity.
    pub volume: Identifier,
    /// Absolute path inside the container.
    pub mount_path: String,
}

/// One executable process realized by an OCI image output.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Process {
    /// Stable process identity.
    pub name: Identifier,
    /// OCI image output in the compiled build.
    pub image: Identifier,
    /// Optional runtime entrypoint override.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub entrypoint: Vec<String>,
    /// Optional fixed runtime arguments.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub arguments: Vec<String>,
}

/// A process invocation inside a workload pod.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContainerRole {
    /// Stable container-role identity.
    pub name: Identifier,
    /// Process invoked by the container.
    pub process: Identifier,
    /// Named HTTP port, when the container serves one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub http_port: Option<u16>,
    /// HTTP readiness path, when applicable.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub readiness_path: Option<String>,
    /// HTTP liveness path, when applicable.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub liveness_path: Option<String>,
    /// Non-secret configuration interface.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub config: Vec<ConfigSlot>,
    /// Secret configuration interface.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub secrets: Vec<SecretSlot>,
    /// Required service endpoints.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub endpoints: Vec<EndpointSlot>,
    /// Persistent workload volumes mounted by this container.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub volume_mounts: Vec<VolumeMount>,
    /// Required workload-token audiences.
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub audiences: BTreeSet<String>,
}

/// One named persistent volume owned by a stateful workload.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PersistentVolume {
    /// Stable volume identity used by container mounts.
    pub name: Identifier,
    /// Kubernetes storage quantity, such as `1Gi`.
    pub size: String,
}

/// Deployment-neutral runtime controller intent.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Workload {
    /// Stable workload identity.
    pub name: Identifier,
    /// Semantic ESS components satisfied by this workload.
    pub components: BTreeSet<Identifier>,
    /// Container roles in the pod.
    pub containers: BTreeSet<Identifier>,
    /// Desired instance count; must satisfy every semantic floor and ceiling.
    pub replicas: u32,
    /// Persistent volume size, when the workload is stateful.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub storage: Option<String>,
    /// Named persistent volumes. This supersedes the legacy single `storage` marker.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub volumes: Vec<PersistentVolume>,
}

/// Human-authored mapping from semantic ESS to built runtime artifacts.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeSpec {
    format: String,
    /// Stable runtime identity.
    pub runtime: Identifier,
    /// ESS semantic source digest, including `sha256:` prefix.
    pub semantic_digest: Digest,
    /// Exact physical realization digest.
    pub realization_digest: Digest,
    /// Exact compiled build digest.
    pub build_digest: Digest,
    /// Executable processes.
    pub processes: Vec<Process>,
    /// Container roles.
    pub containers: Vec<ContainerRole>,
    /// Workloads.
    pub workloads: Vec<Workload>,
    /// Stable network endpoints supplied by this runtime.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub provided_endpoints: Vec<ProvidedEndpoint>,
}

impl RuntimeSpec {
    /// Reads a strict JSON runtime.
    pub fn from_json(text: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(text)
    }

    /// Reads a strict YAML runtime.
    pub fn from_yaml(text: &str) -> Result<Self, serde_yaml::Error> {
        serde_yaml::from_str(text)
    }
}

/// Validated deployable-runtime IR.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeIr {
    format: String,
    runtime: Identifier,
    semantic_digest: Digest,
    realization_digest: Digest,
    build_digest: Digest,
    #[serde(deserialize_with = "crate::validation::unique_map")]
    processes: BTreeMap<Identifier, Process>,
    #[serde(deserialize_with = "crate::validation::unique_map")]
    containers: BTreeMap<Identifier, ContainerRole>,
    #[serde(deserialize_with = "crate::validation::unique_map")]
    workloads: BTreeMap<Identifier, Workload>,
    #[serde(deserialize_with = "crate::validation::unique_map")]
    provided_endpoints: BTreeMap<Identifier, ProvidedEndpoint>,
}

crate::validation::checked_deserialize!(RuntimeIr {
    format: String,
    runtime: Identifier,
    semantic_digest: Digest,
    realization_digest: Digest,
    build_digest: Digest,
    #[serde(deserialize_with = "crate::validation::unique_map")]
    processes: BTreeMap<Identifier, Process>,
    #[serde(deserialize_with = "crate::validation::unique_map")]
    containers: BTreeMap<Identifier, ContainerRole>,
    #[serde(deserialize_with = "crate::validation::unique_map")]
    workloads: BTreeMap<Identifier, Workload>,
    #[serde(deserialize_with = "crate::validation::unique_map")]
    provided_endpoints: BTreeMap<Identifier, ProvidedEndpoint>,
});

impl RuntimeIr {
    /// Validate relationships and compiler rules recoverable from these bytes alone.
    /// Semantic coverage, replica bounds and statefulness need the original compiler inputs.
    pub fn validate(&self) -> Result<(), Diagnostics> {
        use crate::validation::check;
        let mut diagnostics = Vec::new();
        check(
            &mut diagnostics,
            self.format == RUNTIME_IR_FORMAT,
            Stage::Runtime,
            DiagnosticCode::UnsupportedFormat,
            &self.runtime,
            format!("runtime IR format must be {RUNTIME_IR_FORMAT:?}"),
        );
        for (name, process) in &self.processes {
            check(
                &mut diagnostics,
                name == &process.name,
                Stage::Runtime,
                DiagnosticCode::InvalidValue,
                name,
                "process map key must match its name",
            );
        }
        for (name, container) in &self.containers {
            check(
                &mut diagnostics,
                name == &container.name,
                Stage::Runtime,
                DiagnosticCode::InvalidValue,
                name,
                "container map key must match its name",
            );
            check(
                &mut diagnostics,
                self.processes.contains_key(&container.process),
                Stage::Runtime,
                DiagnosticCode::UnknownReference,
                name,
                "container selects an unknown process",
            );
            validate_container(container, &mut diagnostics);
        }
        let mut components = BTreeSet::new();
        for (name, workload) in &self.workloads {
            check(
                &mut diagnostics,
                name == &workload.name,
                Stage::Runtime,
                DiagnosticCode::InvalidValue,
                name,
                "workload map key must match its name",
            );
            check(
                &mut diagnostics,
                workload.replicas > 0,
                Stage::Runtime,
                DiagnosticCode::InvalidValue,
                name,
                "workload replicas must be positive",
            );
            check(
                &mut diagnostics,
                !workload.containers.is_empty(),
                Stage::Runtime,
                DiagnosticCode::InvalidValue,
                name,
                "workload must select at least one container",
            );
            for container in &workload.containers {
                check(
                    &mut diagnostics,
                    self.containers.contains_key(container),
                    Stage::Runtime,
                    DiagnosticCode::UnknownReference,
                    name,
                    "workload selects an unknown container",
                );
            }
            for component in &workload.components {
                check(
                    &mut diagnostics,
                    components.insert(component),
                    Stage::Runtime,
                    DiagnosticCode::DuplicateComponent,
                    name,
                    format!("component {component} is realized by more than one workload"),
                );
            }
            validate_workload_volumes(workload, &self.containers, &mut diagnostics);
        }
        self.validate_endpoints(&mut diagnostics);
        crate::validation::finish(diagnostics)
    }

    fn validate_endpoints(&self, diagnostics: &mut Vec<Diagnostic>) {
        use crate::validation::check;
        for (name, endpoint) in &self.provided_endpoints {
            check(
                diagnostics,
                name == &endpoint.name,
                Stage::Runtime,
                DiagnosticCode::InvalidValue,
                name,
                "provided endpoint map key must match its name",
            );
            check(
                diagnostics,
                self.workloads
                    .get(&endpoint.workload)
                    .is_some_and(|workload| workload.containers.contains(&endpoint.container)),
                Stage::Runtime,
                DiagnosticCode::UnknownReference,
                name,
                "provided endpoint must select a container in its workload",
            );
            check(
                diagnostics,
                self.containers
                    .get(&endpoint.container)
                    .is_some_and(|container| container.http_port.is_some()),
                Stage::Runtime,
                DiagnosticCode::MissingBinding,
                name,
                "provided endpoint requires an HTTP port",
            );
        }
    }

    /// Validate the additional relationships for which a bundle supplies the build bytes.
    pub fn validate_against_build(&self, build: &BuildIr) -> Result<(), Diagnostics> {
        let mut diagnostics = self
            .validate()
            .err()
            .map_or_else(Vec::new, |errors| errors.as_slice().to_vec());
        crate::validation::check(
            &mut diagnostics,
            self.build_digest == build.digest(),
            Stage::Runtime,
            DiagnosticCode::DigestMismatch,
            &self.runtime,
            "runtime build_digest does not match the supplied build IR",
        );
        for process in self.processes.values() {
            crate::validation::check(
                &mut diagnostics,
                build
                    .outputs()
                    .get(&process.image)
                    .is_some_and(|output| output.kind == BuildOutputKind::OciImage),
                Stage::Runtime,
                DiagnosticCode::MissingOutput,
                &process.name,
                "process image must select an OCI image build output",
            );
        }
        crate::validation::finish(diagnostics)
    }

    /// Reads strict compiler-owned runtime JSON.
    pub fn from_json(text: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(text)
    }

    /// Stable runtime identity.
    pub fn runtime(&self) -> &Identifier {
        &self.runtime
    }

    /// Exact semantic source digest.
    pub fn semantic_digest(&self) -> &Digest {
        &self.semantic_digest
    }

    /// Exact physical realization digest.
    pub fn realization_digest(&self) -> &Digest {
        &self.realization_digest
    }

    /// Exact compiled build digest.
    pub fn build_digest(&self) -> &Digest {
        &self.build_digest
    }

    /// Executable processes by stable identity.
    pub fn processes(&self) -> &BTreeMap<Identifier, Process> {
        &self.processes
    }

    /// Container roles by stable identity.
    pub fn containers(&self) -> &BTreeMap<Identifier, ContainerRole> {
        &self.containers
    }

    /// Workloads by stable identity.
    pub fn workloads(&self) -> &BTreeMap<Identifier, Workload> {
        &self.workloads
    }

    /// Network endpoints provided by the runtime.
    pub fn provided_endpoints(&self) -> &BTreeMap<Identifier, ProvidedEndpoint> {
        &self.provided_endpoints
    }

    /// Canonical JSON with a trailing newline.
    pub fn to_canonical_json(&self) -> String {
        canonical_json(self)
    }

    /// Digest of this exact runtime.
    pub fn digest(&self) -> Digest {
        Digest::of_bytes(self.to_canonical_json().as_bytes())
    }
}

/// Validate the complete semantic-to-runtime mapping.
#[allow(clippy::too_many_lines)]
pub fn compile_runtime(
    specification: &RuntimeSpec,
    semantic: &EssIr,
    realization: &PhysicalRealizationIr,
    build: &BuildIr,
) -> Result<RuntimeIr, Diagnostics> {
    let mut diagnostics = Vec::new();
    if specification.format != RUNTIME_FORMAT {
        diagnostics.push(Diagnostic::new(
            Stage::Runtime,
            DiagnosticCode::UnsupportedFormat,
            Some(specification.runtime.clone()),
            format!(
                "runtime format {:?} is unsupported; expected {RUNTIME_FORMAT:?}",
                specification.format
            ),
        ));
    }
    let expected_semantic = Digest::new(format!("sha256:{}", semantic.source_digest()))
        .expect("ESS compiler source digests are lowercase SHA-256");
    if specification.semantic_digest != expected_semantic {
        diagnostics.push(Diagnostic::new(
            Stage::Runtime,
            DiagnosticCode::DigestMismatch,
            Some(specification.runtime.clone()),
            format!(
                "semantic digest {} does not match compiled ESS {}",
                specification.semantic_digest, expected_semantic
            ),
        ));
    }
    let expected_realization = Digest::new(realization.realization_digest().as_str())
        .expect("realization compiler digests are lowercase SHA-256");
    if specification.realization_digest != expected_realization {
        diagnostics.push(Diagnostic::new(
            Stage::Runtime,
            DiagnosticCode::DigestMismatch,
            Some(specification.runtime.clone()),
            format!(
                "realization digest {} does not match compiled realization {}",
                specification.realization_digest, expected_realization
            ),
        ));
    }
    if specification.build_digest != build.digest() {
        diagnostics.push(Diagnostic::new(
            Stage::Runtime,
            DiagnosticCode::DigestMismatch,
            Some(specification.runtime.clone()),
            "build digest does not match the supplied build IR",
        ));
    }

    let processes = unique_map(
        &specification.processes,
        |process| &process.name,
        "process",
        &mut diagnostics,
    );
    for process in processes.values() {
        match build.outputs().get(&process.image) {
            Some(output) if output.kind == BuildOutputKind::OciImage => {}
            Some(_) => diagnostics.push(Diagnostic::new(
                Stage::Runtime,
                DiagnosticCode::InvalidValue,
                Some(process.name.clone()),
                format!("process image {} is not an OCI image output", process.image),
            )),
            None => diagnostics.push(Diagnostic::new(
                Stage::Runtime,
                DiagnosticCode::MissingOutput,
                Some(process.name.clone()),
                format!("process selects unknown build output {}", process.image),
            )),
        }
    }

    let containers = unique_map(
        &specification.containers,
        |container| &container.name,
        "container role",
        &mut diagnostics,
    );
    for container in containers.values() {
        if !processes.contains_key(&container.process) {
            diagnostics.push(Diagnostic::new(
                Stage::Runtime,
                DiagnosticCode::UnknownReference,
                Some(container.name.clone()),
                format!("container selects unknown process {}", container.process),
            ));
        }
        validate_container(container, &mut diagnostics);
    }

    let workloads = unique_map(
        &specification.workloads,
        |workload| &workload.name,
        "workload",
        &mut diagnostics,
    );
    let semantic_components: BTreeMap<Identifier, _> = semantic
        .components()
        .keys()
        .map(|component| {
            (
                Identifier::new(component.as_str())
                    .expect("ESS component names are valid realization identifiers"),
                component,
            )
        })
        .collect();
    let mut realized_components = BTreeSet::new();
    for workload in workloads.values() {
        if workload.replicas == 0 {
            diagnostics.push(Diagnostic::new(
                Stage::Runtime,
                DiagnosticCode::InvalidValue,
                Some(workload.name.clone()),
                "a realized workload must run at least one replica",
            ));
        }
        if workload.containers.is_empty() {
            diagnostics.push(Diagnostic::new(
                Stage::Runtime,
                DiagnosticCode::MissingComponent,
                Some(workload.name.clone()),
                "a workload must contain at least one container role",
            ));
        }
        for container in &workload.containers {
            if !containers.contains_key(container) {
                diagnostics.push(Diagnostic::new(
                    Stage::Runtime,
                    DiagnosticCode::UnknownReference,
                    Some(workload.name.clone()),
                    format!("workload selects unknown container role {container}"),
                ));
            }
        }
        for component in &workload.components {
            if !semantic_components.contains_key(component) {
                diagnostics.push(Diagnostic::new(
                    Stage::Runtime,
                    DiagnosticCode::MissingComponent,
                    Some(workload.name.clone()),
                    format!("workload selects unknown semantic component {component}"),
                ));
            }
            if !realized_components.insert(component.clone()) {
                diagnostics.push(Diagnostic::new(
                    Stage::Runtime,
                    DiagnosticCode::DuplicateComponent,
                    Some(component.clone()),
                    format!("semantic component {component} is realized by more than one workload"),
                ));
            }
            if let Some(requirement) = semantic.workloads().iter().find_map(|(name, workload)| {
                (name.as_str() == component.as_str()).then_some(workload)
            }) {
                if workload.replicas < requirement.replicas.min
                    || requirement
                        .replicas
                        .max
                        .is_some_and(|max| workload.replicas > max)
                {
                    diagnostics.push(Diagnostic::new(
                        Stage::Runtime,
                        DiagnosticCode::InvalidValue,
                        Some(workload.name.clone()),
                        format!(
                            "{} replicas do not satisfy semantic range {}..{} for component {}",
                            workload.replicas,
                            requirement.replicas.min,
                            requirement
                                .replicas
                                .max
                                .map_or_else(|| "unbounded".to_owned(), |max| max.to_string()),
                            component
                        ),
                    ));
                }
                if !requirement.stateless
                    && workload.storage.is_none()
                    && workload.volumes.is_empty()
                {
                    diagnostics.push(Diagnostic::new(
                        Stage::Runtime,
                        DiagnosticCode::MissingBinding,
                        Some(workload.name.clone()),
                        format!("stateful component {component} requires a storage declaration"),
                    ));
                }
            }
        }
        validate_workload_volumes(workload, &containers, &mut diagnostics);
    }
    for component in semantic_components.keys() {
        if !realized_components.contains(component) {
            diagnostics.push(Diagnostic::new(
                Stage::Runtime,
                DiagnosticCode::MissingComponent,
                Some(component.clone()),
                format!("semantic component {component} has no realized workload"),
            ));
        }
    }

    let provided_endpoints = unique_map(
        &specification.provided_endpoints,
        |endpoint| &endpoint.name,
        "provided endpoint",
        &mut diagnostics,
    );
    for endpoint in provided_endpoints.values() {
        let Some(workload) = workloads.get(&endpoint.workload) else {
            diagnostics.push(Diagnostic::new(
                Stage::Runtime,
                DiagnosticCode::UnknownReference,
                Some(endpoint.name.clone()),
                format!(
                    "provided endpoint selects unknown workload {}",
                    endpoint.workload
                ),
            ));
            continue;
        };
        if !workload.containers.contains(&endpoint.container) {
            diagnostics.push(Diagnostic::new(
                Stage::Runtime,
                DiagnosticCode::UnknownReference,
                Some(endpoint.name.clone()),
                format!(
                    "provided endpoint container {} is not part of workload {}",
                    endpoint.container, endpoint.workload
                ),
            ));
            continue;
        }
        if containers
            .get(&endpoint.container)
            .and_then(|container| container.http_port)
            .is_none()
        {
            diagnostics.push(Diagnostic::new(
                Stage::Runtime,
                DiagnosticCode::MissingBinding,
                Some(endpoint.name.clone()),
                "provided endpoint container must declare an HTTP port",
            ));
        }
    }

    if diagnostics.is_empty() {
        Ok(RuntimeIr {
            format: RUNTIME_IR_FORMAT.to_owned(),
            runtime: specification.runtime.clone(),
            semantic_digest: specification.semantic_digest.clone(),
            realization_digest: specification.realization_digest.clone(),
            build_digest: specification.build_digest.clone(),
            processes,
            containers,
            workloads,
            provided_endpoints,
        })
    } else {
        Err(Diagnostics::from(diagnostics))
    }
}

fn unique_map<T: Clone>(
    values: &[T],
    key: impl Fn(&T) -> &Identifier,
    kind: &str,
    diagnostics: &mut Vec<Diagnostic>,
) -> BTreeMap<Identifier, T> {
    let mut result = BTreeMap::new();
    for value in values {
        let id = key(value);
        if result.insert(id.clone(), value.clone()).is_some() {
            diagnostics.push(Diagnostic::new(
                Stage::Runtime,
                DiagnosticCode::DuplicateIdentifier,
                Some(id.clone()),
                format!("{kind} {id} is declared more than once"),
            ));
        }
    }
    result
}

fn validate_container(container: &ContainerRole, diagnostics: &mut Vec<Diagnostic>) {
    let mut names = BTreeSet::new();
    let mut variables = BTreeSet::new();
    for slot in &container.config {
        if !names.insert(slot.name.clone()) || !variables.insert(slot.environment.clone()) {
            diagnostics.push(Diagnostic::new(
                Stage::Runtime,
                DiagnosticCode::DuplicateIdentifier,
                Some(container.name.clone()),
                format!(
                    "container {} has a duplicate configuration slot or environment variable",
                    container.name
                ),
            ));
        }
        let literal_is_valid = matches!(slot.kind, ConfigKind::Literal) == slot.value.is_some();
        if !literal_is_valid {
            diagnostics.push(Diagnostic::new(
                Stage::Runtime,
                DiagnosticCode::InvalidValue,
                Some(slot.name.clone()),
                "only literal configuration slots may carry a value, and every literal must carry one",
            ));
        }
    }
    for slot in &container.secrets {
        if !names.insert(slot.name.clone()) || !variables.insert(slot.environment.clone()) {
            diagnostics.push(Diagnostic::new(
                Stage::Runtime,
                DiagnosticCode::DuplicateIdentifier,
                Some(container.name.clone()),
                "secret and non-secret slots must have unique names and environment variables",
            ));
        }
    }
    for slot in &container.endpoints {
        if !names.insert(slot.name.clone()) || !variables.insert(slot.environment.clone()) {
            diagnostics.push(Diagnostic::new(
                Stage::Runtime,
                DiagnosticCode::DuplicateIdentifier,
                Some(container.name.clone()),
                "endpoint and configuration slots must have unique names and environment variables",
            ));
        }
    }
    let mut mounted_volumes = BTreeSet::new();
    let mut mount_paths = BTreeSet::new();
    for mount in &container.volume_mounts {
        if !mounted_volumes.insert(mount.volume.clone())
            || !mount_paths.insert(mount.mount_path.clone())
        {
            diagnostics.push(Diagnostic::new(
                Stage::Runtime,
                DiagnosticCode::DuplicateIdentifier,
                Some(container.name.clone()),
                "volume identities and mount paths must be unique within a container",
            ));
        }
        if !mount.mount_path.starts_with('/') || mount.mount_path.contains("/../") {
            diagnostics.push(Diagnostic::new(
                Stage::Runtime,
                DiagnosticCode::InvalidValue,
                Some(container.name.clone()),
                format!(
                    "volume mount path {:?} must be absolute and may not traverse parents",
                    mount.mount_path
                ),
            ));
        }
    }
    if (container.readiness_path.is_some() || container.liveness_path.is_some())
        && container.http_port.is_none()
    {
        diagnostics.push(Diagnostic::new(
            Stage::Runtime,
            DiagnosticCode::InvalidValue,
            Some(container.name.clone()),
            "HTTP probes require an HTTP port",
        ));
    }
}

fn validate_workload_volumes(
    workload: &Workload,
    containers: &BTreeMap<Identifier, ContainerRole>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let mut volumes = BTreeSet::new();
    if workload.storage.is_some() && !workload.volumes.is_empty() {
        diagnostics.push(Diagnostic::new(
            Stage::Runtime,
            DiagnosticCode::InvalidValue,
            Some(workload.name.clone()),
            "legacy storage and named volumes may not be declared together",
        ));
    }
    for volume in &workload.volumes {
        if !volumes.insert(volume.name.clone()) {
            diagnostics.push(Diagnostic::new(
                Stage::Runtime,
                DiagnosticCode::DuplicateIdentifier,
                Some(workload.name.clone()),
                format!("volume {} is declared more than once", volume.name),
            ));
        }
        if volume.size.is_empty() {
            diagnostics.push(Diagnostic::new(
                Stage::Runtime,
                DiagnosticCode::InvalidValue,
                Some(volume.name.clone()),
                "persistent volume size must not be empty",
            ));
        }
    }
    for container_name in &workload.containers {
        let Some(container) = containers.get(container_name) else {
            continue;
        };
        for mount in &container.volume_mounts {
            if !volumes.contains(&mount.volume) {
                diagnostics.push(Diagnostic::new(
                    Stage::Runtime,
                    DiagnosticCode::UnknownReference,
                    Some(container.name.clone()),
                    format!(
                        "container mounts volume {} absent from workload {}",
                        mount.volume, workload.name
                    ),
                ));
            }
        }
    }
}
