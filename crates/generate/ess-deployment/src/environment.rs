use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

use semver::Version;

use crate::diagnostic::{Diagnostic, DiagnosticCode, Diagnostics, Stage};
use crate::identity::{canonical_json, Digest, Identifier};
use crate::release::{Artifact, ArtifactKind};
use crate::runtime::{ConfigKind, RuntimeIr};
use crate::stack::StackLock;

/// Private environment-binding format.
pub const ENVIRONMENT_FORMAT: &str = "ess-environment/1";
/// Compiler-owned independent release-set format.
pub const DEPLOYMENT_FORMAT: &str = "ess-deployment/1";

/// Reference to one key in an environment-owned secret object.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SecretBinding {
    /// Kubernetes Secret name.
    pub name: String,
    /// Key inside the Secret.
    pub key: String,
}

/// Environment bindings for one first-party stack service.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReleaseBinding {
    /// Stack service identity.
    pub service: Identifier,
    /// Independent Helm release name.
    pub release_name: String,
    /// Kubernetes `ServiceAccount` name.
    pub service_account: String,
    /// Non-secret configuration values by declared slot.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub config: BTreeMap<Identifier, String>,
    /// Secret object references by declared slot.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub secrets: BTreeMap<Identifier, SecretBinding>,
    /// Bound endpoint URLs by declared slot.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub endpoints: BTreeMap<Identifier, String>,
}

/// Environment bindings for one typed external system.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExternalBinding {
    /// External system identity from the generic stack.
    pub system: Identifier,
    /// Concrete endpoints by required endpoint name.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub endpoints: BTreeMap<Identifier, String>,
    /// Available authority or workload-token audiences.
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub authorities: BTreeSet<String>,
    /// Non-secret environment coordinates.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub config: BTreeMap<Identifier, String>,
}

/// Private target-environment binding.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EnvironmentSpec {
    format: String,
    /// Stable environment identity.
    pub environment: Identifier,
    /// Exact generic stack lock being deployed.
    pub stack_digest: Digest,
    /// Kubeconfig context selected by the executor.
    pub cluster: String,
    /// Default Kubernetes namespace.
    pub namespace: String,
    /// First-party service bindings.
    pub releases: Vec<ReleaseBinding>,
    /// External-system bindings.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub external_systems: Vec<ExternalBinding>,
}

impl EnvironmentSpec {
    /// Reads a strict JSON environment binding.
    pub fn from_json(text: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(text)
    }

    /// Reads a strict YAML environment binding.
    pub fn from_yaml(text: &str) -> Result<Self, serde_yaml::Error> {
        serde_yaml::from_str(text)
    }
}

/// One exact independent Helm release in deployment IR.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeploymentRelease {
    /// Stack service identity.
    pub service: Identifier,
    /// Helm release name.
    pub release_name: String,
    /// Kubernetes namespace.
    pub namespace: String,
    /// Exact independently versioned chart artifact.
    pub chart: Artifact,
    /// Exact image artifacts by build-output identity.
    pub images: BTreeMap<Identifier, Artifact>,
    /// Kubernetes `ServiceAccount` name.
    pub service_account: String,
    /// Non-secret configuration.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub config: BTreeMap<Identifier, String>,
    /// Secret references, never values.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub secrets: BTreeMap<Identifier, SecretBinding>,
    /// Bound endpoint URLs.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub endpoints: BTreeMap<Identifier, String>,
    /// Required workload-token audiences.
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub audiences: BTreeSet<String>,
    /// Explicit rollout prerequisites by stack service identity.
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub depends_on: BTreeSet<Identifier>,
}

/// Compiler-owned exact deployment and rollout DAG.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeploymentIr {
    format: String,
    /// Stable target environment.
    pub environment: Identifier,
    /// Exact stack lock digest.
    pub stack_digest: Digest,
    /// Executor-selected kubeconfig context.
    pub cluster: String,
    /// Independent releases by stack service identity.
    pub releases: BTreeMap<Identifier, DeploymentRelease>,
    /// Stable topological rollout order. Executors may parallelize adjacent independent entries.
    pub rollout_order: Vec<Identifier>,
}

impl DeploymentIr {
    /// Reads strict deployment IR JSON.
    pub fn from_json(text: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(text)
    }

    /// Canonical JSON with a trailing newline.
    pub fn to_canonical_json(&self) -> String {
        canonical_json(self)
    }

    /// Digest of the exact desired deployment.
    pub fn digest(&self) -> Digest {
        Digest::of_bytes(self.to_canonical_json().as_bytes())
    }
}

/// Bind an exact generic stack lock to one private target environment.
#[allow(clippy::too_many_lines)]
pub fn compile_deployment(
    environment: &EnvironmentSpec,
    stack: &StackLock,
) -> Result<DeploymentIr, Diagnostics> {
    let mut diagnostics = Vec::new();
    if environment.format != ENVIRONMENT_FORMAT {
        diagnostics.push(Diagnostic::new(
            Stage::Deployment,
            DiagnosticCode::UnsupportedFormat,
            Some(environment.environment.clone()),
            format!("environment format must be {ENVIRONMENT_FORMAT:?}"),
        ));
    }
    if environment.stack_digest != stack.digest() {
        diagnostics.push(Diagnostic::new(
            Stage::Deployment,
            DiagnosticCode::DigestMismatch,
            Some(environment.environment.clone()),
            "environment stack_digest does not match the supplied exact stack lock",
        ));
    }
    if environment.cluster.is_empty() || environment.namespace.is_empty() {
        diagnostics.push(Diagnostic::new(
            Stage::Deployment,
            DiagnosticCode::MissingBinding,
            Some(environment.environment.clone()),
            "cluster and namespace bindings must not be empty",
        ));
    }

    let release_bindings = unique_bindings(
        &environment.releases,
        |binding| &binding.service,
        "release binding",
        &mut diagnostics,
    );
    let external_bindings = unique_bindings(
        &environment.external_systems,
        |binding| &binding.system,
        "external-system binding",
        &mut diagnostics,
    );

    for (name, external) in &stack.external_systems {
        let Some(binding) = external_bindings.get(name) else {
            diagnostics.push(Diagnostic::new(
                Stage::Deployment,
                DiagnosticCode::MissingBinding,
                Some(name.clone()),
                format!("external system {name} has no environment binding"),
            ));
            continue;
        };
        for endpoint in &external.endpoints {
            if !binding.endpoints.contains_key(endpoint) {
                diagnostics.push(Diagnostic::new(
                    Stage::Deployment,
                    DiagnosticCode::MissingBinding,
                    Some(name.clone()),
                    format!("external system {name} endpoint {endpoint} is unbound"),
                ));
            }
        }
        for authority in &external.authorities {
            if !binding.authorities.contains(authority) {
                diagnostics.push(Diagnostic::new(
                    Stage::Deployment,
                    DiagnosticCode::AuthorityUnbound,
                    Some(name.clone()),
                    format!("external system {name} authority {authority:?} is unbound"),
                ));
            }
        }
        for config in &external.config {
            if !binding.config.contains_key(config) {
                diagnostics.push(Diagnostic::new(
                    Stage::Deployment,
                    DiagnosticCode::MissingBinding,
                    Some(name.clone()),
                    format!("external system {name} configuration {config} is unbound"),
                ));
            }
        }
    }

    let mut releases = BTreeMap::new();
    for (service, locked) in &stack.systems {
        let Some(binding) = release_bindings.get(service) else {
            diagnostics.push(Diagnostic::new(
                Stage::Deployment,
                DiagnosticCode::MissingBinding,
                Some(service.clone()),
                format!("stack service {service} has no environment release binding"),
            ));
            continue;
        };
        if binding.release_name.is_empty() {
            diagnostics.push(Diagnostic::new(
                Stage::Deployment,
                DiagnosticCode::MissingBinding,
                Some(service.clone()),
                "Helm release name must not be empty",
            ));
        }
        for (slot, kind) in &locked.runtime.config {
            if *kind == ConfigKind::Required && !binding.config.contains_key(slot) {
                diagnostics.push(Diagnostic::new(
                    Stage::Deployment,
                    DiagnosticCode::MissingBinding,
                    Some(service.clone()),
                    format!("required configuration slot {slot} is unbound"),
                ));
            }
        }
        for slot in &locked.runtime.secrets {
            if !binding.secrets.contains_key(slot) {
                diagnostics.push(Diagnostic::new(
                    Stage::Deployment,
                    DiagnosticCode::MissingBinding,
                    Some(service.clone()),
                    format!("required secret slot {slot} is unbound"),
                ));
            }
        }
        let mut endpoints = binding.endpoints.clone();
        for (slot, target) in &locked.runtime.endpoints {
            if endpoints.contains_key(slot) {
                continue;
            }
            let derived = locked
                .runtime
                .endpoint_names
                .get(slot)
                .and_then(|endpoint| {
                    derive_endpoint(
                        stack,
                        &release_bindings,
                        &external_bindings,
                        target,
                        endpoint,
                    )
                });
            if let Some(derived) = derived {
                endpoints.insert(slot.clone(), derived);
            } else {
                diagnostics.push(Diagnostic::new(
                    Stage::Deployment,
                    DiagnosticCode::MissingBinding,
                    Some(service.clone()),
                    format!("endpoint slot {slot} targeting {target} is unbound"),
                ));
            }
        }
        if !locked.runtime.audiences.is_empty() && binding.service_account.is_empty() {
            diagnostics.push(Diagnostic::new(
                Stage::Deployment,
                DiagnosticCode::AuthorityUnbound,
                Some(service.clone()),
                "workload-token audiences require a service-account binding",
            ));
        }

        let images = locked
            .runtime_artifacts
            .iter()
            .filter(|(_, artifact)| artifact.kind == ArtifactKind::OciImage)
            .map(|(name, artifact)| (name.clone(), artifact.clone()))
            .collect();
        releases.insert(
            service.clone(),
            DeploymentRelease {
                service: service.clone(),
                release_name: binding.release_name.clone(),
                namespace: environment.namespace.clone(),
                chart: locked.chart.clone(),
                images,
                service_account: binding.service_account.clone(),
                config: binding.config.clone(),
                secrets: binding.secrets.clone(),
                endpoints,
                audiences: locked.runtime.audiences.clone(),
                depends_on: locked.depends_on.clone(),
            },
        );
    }
    for service in release_bindings.keys() {
        if !stack.systems.contains_key(service) {
            diagnostics.push(Diagnostic::new(
                Stage::Deployment,
                DiagnosticCode::UnknownReference,
                Some(service.clone()),
                format!("environment binds service {service} absent from the stack lock"),
            ));
        }
    }

    let rollout_order = rollout_order(&releases).unwrap_or_default();
    if rollout_order.len() != releases.len() {
        diagnostics.push(Diagnostic::new(
            Stage::Deployment,
            DiagnosticCode::DependencyCycle,
            Some(environment.environment.clone()),
            "deployment rollout graph contains a cycle",
        ));
    }

    if diagnostics.is_empty() {
        Ok(DeploymentIr {
            format: DEPLOYMENT_FORMAT.to_owned(),
            environment: environment.environment.clone(),
            stack_digest: environment.stack_digest.clone(),
            cluster: environment.cluster.clone(),
            releases,
            rollout_order,
        })
    } else {
        Err(Diagnostics::from(diagnostics))
    }
}

fn derive_endpoint(
    stack: &StackLock,
    release_bindings: &BTreeMap<Identifier, &ReleaseBinding>,
    external_bindings: &BTreeMap<Identifier, &ExternalBinding>,
    target: &Identifier,
    endpoint: &Identifier,
) -> Option<String> {
    if let Some(external) = external_bindings.get(target) {
        return external.endpoints.get(endpoint).cloned();
    }

    let candidates = stack
        .systems
        .iter()
        .filter(|(service, system)| {
            (*service == target || system.system == *target)
                && system.runtime.provided_endpoints.contains_key(endpoint)
        })
        .collect::<Vec<_>>();
    let [(service, system)] = candidates.as_slice() else {
        return None;
    };
    let release = release_bindings.get(*service)?;
    let provided = system.runtime.provided_endpoints.get(endpoint)?;
    Some(format!(
        "{}://{}-{}:{}",
        provided.scheme.as_str(),
        release.release_name,
        endpoint,
        provided.port
    ))
}

fn unique_bindings<'a, T>(
    values: &'a [T],
    key: impl Fn(&T) -> &Identifier,
    kind: &str,
    diagnostics: &mut Vec<Diagnostic>,
) -> BTreeMap<Identifier, &'a T> {
    let mut result = BTreeMap::new();
    for value in values {
        let id = key(value);
        if result.insert(id.clone(), value).is_some() {
            diagnostics.push(Diagnostic::new(
                Stage::Deployment,
                DiagnosticCode::DuplicateIdentifier,
                Some(id.clone()),
                format!("{kind} {id} is declared more than once"),
            ));
        }
    }
    result
}

fn rollout_order(releases: &BTreeMap<Identifier, DeploymentRelease>) -> Option<Vec<Identifier>> {
    let mut remaining: BTreeMap<_, _> = releases
        .iter()
        .map(|(name, release)| (name.clone(), release.depends_on.clone()))
        .collect();
    let mut ready: BTreeSet<_> = remaining
        .iter()
        .filter(|(_, dependencies)| dependencies.is_empty())
        .map(|(name, _)| name.clone())
        .collect();
    let mut order = Vec::new();
    while let Some(name) = ready.pop_first() {
        order.push(name.clone());
        remaining.remove(&name);
        for (candidate, dependencies) in &mut remaining {
            dependencies.remove(&name);
            if dependencies.is_empty() {
                ready.insert(candidate.clone());
            }
        }
    }
    (order.len() == releases.len()).then_some(order)
}

/// Deterministic generic Helm chart files projected from runtime IR.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HelmProjection {
    files: BTreeMap<String, String>,
}

impl HelmProjection {
    /// Projected files by relative path.
    pub fn files(&self) -> &BTreeMap<String, String> {
        &self.files
    }
}

/// Project one component-owned, configuration-neutral Helm chart.
pub fn project_helm(
    realization: &RuntimeIr,
    chart: &Identifier,
    version: &Version,
) -> HelmProjection {
    let chart_yaml = format!(
        "apiVersion: v2\nname: {chart}\ndescription: Generated from {}\ntype: application\nversion: {version}\n",
        realization.runtime()
    );
    let mut values = String::from("serviceAccount:\n  name: \"\"\nimages:\n");
    let mut image_names = BTreeSet::new();
    for process in realization.processes().values() {
        image_names.insert(process.image.clone());
    }
    for image in &image_names {
        writeln!(
            &mut values,
            "  {image}:\n    repository: \"\"\n    digest: \"\""
        )
        .unwrap();
    }
    values.push_str("config: {}\nsecrets: {}\nendpoints: {}\nworkloads:\n");
    for workload in realization.workloads().values() {
        writeln!(
            &mut values,
            "  {}:\n    replicas: {}",
            workload.name, workload.replicas
        )
        .unwrap();
        if let Some(storage) = &workload.storage {
            writeln!(
                &mut values,
                "    storage:\n      class: \"\"\n      size: {storage:?}"
            )
            .unwrap();
        } else if !workload.volumes.is_empty() {
            values.push_str("    volumes:\n");
            for volume in &workload.volumes {
                writeln!(
                    &mut values,
                    "      {}:\n        class: \"\"\n        size: {:?}",
                    volume.name, volume.size
                )
                .unwrap();
            }
        }
    }

    let schema = serde_json::json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "type": "object",
        "additionalProperties": false,
        "required": ["serviceAccount", "images", "config", "secrets", "endpoints", "workloads"],
        "properties": {
            "serviceAccount": {"type": "object", "additionalProperties": false, "required": ["name"], "properties": {"name": {"type": "string", "minLength": 1}}},
            "images": {"type": "object"},
            "config": {"type": "object", "additionalProperties": {"type": "string"}},
            "secrets": {"type": "object"},
            "endpoints": {"type": "object", "additionalProperties": {"type": "string", "format": "uri"}},
            "workloads": {"type": "object"}
        }
    });
    let mut schema_json = serde_json::to_string_pretty(&schema).expect("schema serializes");
    schema_json.push('\n');

    let workloads = render_workloads(realization);
    let services = render_services(realization);
    let files = BTreeMap::from([
        ("Chart.yaml".to_owned(), chart_yaml),
        ("values.yaml".to_owned(), values),
        ("values.schema.json".to_owned(), schema_json),
        ("templates/workloads.yaml".to_owned(), workloads),
        ("templates/services.yaml".to_owned(), services),
    ]);
    HelmProjection { files }
}

#[allow(clippy::too_many_lines)]
fn render_workloads(realization: &RuntimeIr) -> String {
    let mut output = String::from("{{- $root := . -}}\n");
    for (index, workload) in realization.workloads().values().enumerate() {
        if index > 0 {
            output.push_str("---\n");
        }
        let stateful = workload.storage.is_some() || !workload.volumes.is_empty();
        writeln!(
            &mut output,
            "apiVersion: apps/v1\nkind: {}",
            if stateful {
                "StatefulSet"
            } else {
                "Deployment"
            }
        )
        .unwrap();
        writeln!(
            &mut output,
            "metadata:\n  name: {{{{ .Release.Name }}}}-{}",
            workload.name
        )
        .unwrap();
        writeln!(
            &mut output,
            "spec:\n  replicas: {{{{ index .Values.workloads \"{}\" \"replicas\" }}}}",
            workload.name
        )
        .unwrap();
        if stateful {
            writeln!(
                &mut output,
                "  serviceName: {{{{ .Release.Name }}}}-{}-headless",
                workload.name
            )
            .unwrap();
        }
        output.push_str("  selector:\n    matchLabels:\n      app.kubernetes.io/instance: {{ .Release.Name }}\n");
        writeln!(
            &mut output,
            "      app.kubernetes.io/component: {}",
            workload.name
        )
        .unwrap();
        output.push_str("  template:\n    metadata:\n      labels:\n        app.kubernetes.io/instance: {{ .Release.Name }}\n");
        writeln!(
            &mut output,
            "        app.kubernetes.io/component: {}",
            workload.name
        )
        .unwrap();
        output.push_str("    spec:\n      serviceAccountName: {{ .Values.serviceAccount.name }}\n      containers:\n");
        for container_name in &workload.containers {
            let container = realization
                .containers()
                .get(container_name)
                .expect("realization references are total");
            let process = realization
                .processes()
                .get(&container.process)
                .expect("realization references are total");
            writeln!(&mut output, "        - name: {}", container.name).unwrap();
            writeln!(&mut output, "          image: \"{{{{ index .Values.images \"{}\" \"repository\" }}}}@{{{{ index .Values.images \"{}\" \"digest\" }}}}\"", process.image, process.image).unwrap();
            output.push_str("          imagePullPolicy: IfNotPresent\n");
            if !process.entrypoint.is_empty() {
                writeln!(
                    &mut output,
                    "          command: {}",
                    serde_json::to_string(&process.entrypoint).unwrap()
                )
                .unwrap();
            }
            if !process.arguments.is_empty() {
                writeln!(
                    &mut output,
                    "          args: {}",
                    serde_json::to_string(&process.arguments).unwrap()
                )
                .unwrap();
            }
            if let Some(port) = container.http_port {
                writeln!(&mut output, "          ports:\n            - name: http\n              containerPort: {port}").unwrap();
            }
            if let Some(path) = &container.readiness_path {
                writeln!(&mut output, "          readinessProbe:\n            httpGet:\n              path: {path}\n              port: http").unwrap();
            }
            if let Some(path) = &container.liveness_path {
                writeln!(&mut output, "          livenessProbe:\n            httpGet:\n              path: {path}\n              port: http").unwrap();
            }
            if !container.volume_mounts.is_empty() {
                output.push_str("          volumeMounts:\n");
                for mount in &container.volume_mounts {
                    writeln!(
                        &mut output,
                        "            - name: {}\n              mountPath: {:?}",
                        mount.volume, mount.mount_path
                    )
                    .unwrap();
                }
            }
            let has_env = !container.config.is_empty()
                || !container.secrets.is_empty()
                || !container.endpoints.is_empty();
            if has_env {
                output.push_str("          env:\n");
            }
            for slot in &container.config {
                writeln!(&mut output, "            - name: {}", slot.environment).unwrap();
                match slot.kind {
                    ConfigKind::Literal => writeln!(
                        &mut output,
                        "              value: {:?}",
                        slot.value.as_deref().unwrap_or_default()
                    )
                    .unwrap(),
                    ConfigKind::Required | ConfigKind::Optional => writeln!(
                        &mut output,
                        "              value: {{{{ index .Values.config \"{}\" | quote }}}}",
                        slot.name
                    )
                    .unwrap(),
                }
            }
            for slot in &container.endpoints {
                writeln!(&mut output, "            - name: {}\n              value: {{{{ index .Values.endpoints \"{}\" | quote }}}}", slot.environment, slot.name).unwrap();
            }
            for slot in &container.secrets {
                writeln!(&mut output, "            - name: {}\n              valueFrom:\n                secretKeyRef:\n                  name: {{{{ index .Values.secrets \"{}\" \"name\" }}}}\n                  key: {{{{ index .Values.secrets \"{}\" \"key\" }}}}", slot.environment, slot.name, slot.name).unwrap();
            }
        }
        if stateful {
            output.push_str("  volumeClaimTemplates:\n");
            if workload.storage.is_some() {
                render_volume_claim(&mut output, workload, "data");
            } else {
                for volume in &workload.volumes {
                    render_volume_claim(&mut output, workload, volume.name.as_str());
                }
            }
        }
    }
    output
}

fn render_volume_claim(output: &mut String, workload: &crate::runtime::Workload, volume: &str) {
    writeln!(
        output,
        "    - metadata:\n        name: {volume}\n      spec:\n        accessModes: [\"ReadWriteOnce\"]"
    )
    .unwrap();
    if workload.storage.is_some() {
        writeln!(
            output,
            "        {{{{- with (index $root.Values.workloads {:?} \"storage\" \"class\") }}}}\n        storageClassName: {{{{ . | quote }}}}\n        {{{{- end }}}}\n        resources:\n          requests:\n            storage: {{{{ index $root.Values.workloads {:?} \"storage\" \"size\" | quote }}}}",
            workload.name.as_str(),
            workload.name.as_str()
        )
        .unwrap();
    } else {
        writeln!(
            output,
            "        {{{{- with (index $root.Values.workloads {:?} \"volumes\" {:?} \"class\") }}}}\n        storageClassName: {{{{ . | quote }}}}\n        {{{{- end }}}}\n        resources:\n          requests:\n            storage: {{{{ index $root.Values.workloads {:?} \"volumes\" {:?} \"size\" | quote }}}}",
            workload.name.as_str(),
            volume,
            workload.name.as_str(),
            volume
        )
        .unwrap();
    }
}

fn render_services(realization: &RuntimeIr) -> String {
    let mut output = String::new();
    let mut documents = 0usize;
    for endpoint in realization.provided_endpoints().values() {
        if documents > 0 {
            output.push_str("---\n");
        }
        documents += 1;
        let container = realization
            .containers()
            .get(&endpoint.container)
            .expect("compiled provided endpoint container is total");
        writeln!(
            &mut output,
            "apiVersion: v1\nkind: Service\nmetadata:\n  name: {{{{ .Release.Name }}}}-{}\nspec:\n  selector:\n    app.kubernetes.io/instance: {{{{ .Release.Name }}}}\n    app.kubernetes.io/component: {}\n  ports:\n    - name: http\n      port: {}\n      targetPort: http",
            endpoint.name,
            endpoint.workload,
            container.http_port.expect("compiled endpoint has a port")
        )
        .unwrap();
    }
    for workload in realization
        .workloads()
        .values()
        .filter(|workload| workload.storage.is_some() || !workload.volumes.is_empty())
    {
        if documents > 0 {
            output.push_str("---\n");
        }
        documents += 1;
        writeln!(
            &mut output,
            "apiVersion: v1\nkind: Service\nmetadata:\n  name: {{{{ .Release.Name }}}}-{}-headless\nspec:\n  clusterIP: None\n  selector:\n    app.kubernetes.io/instance: {{{{ .Release.Name }}}}\n    app.kubernetes.io/component: {}",
            workload.name, workload.name
        )
        .unwrap();
    }
    output
}
