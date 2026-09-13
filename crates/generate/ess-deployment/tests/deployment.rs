//! Closed-loop tests for the deployment-model formats and projections.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_deployment::{
    bundle_release, compile_build, compile_component, compile_deployment, compile_runtime,
    project_build_mermaid, project_buildkit, project_helm, resolve_stack, verify_release,
    verify_release_bundle, BuildIr, BuildSpec, ComponentSpec, DeploymentIr, DiagnosticCode,
    EnvironmentSpec, ReleaseBundle, ReleaseCatalog, ReleaseManifest, RuntimeIr, RuntimeSpec,
    StackLock, StackSpec,
};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

fn fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../examples/oracle-fixture")
        .canonicalize()
        .expect("fixture exists")
}

fn semantic() -> ess_compiler::EssIr {
    let root = fixture();
    let mut paths = vec![root.join("system.yaml"), root.join("components.yaml")];
    paths.extend([
        root.join("domains/order.yaml"),
        root.join("domains/dispatch.yaml"),
    ]);
    let mut sources = SourceMap::new();
    let parsed: Vec<_> = paths
        .into_iter()
        .map(|path| {
            let label = path
                .strip_prefix(&root)
                .expect("fixture member")
                .display()
                .to_string();
            let text = std::fs::read_to_string(&path).expect("read fixture");
            let raw = RawSpecFile::parse(&text).expect("parse fixture");
            sources.insert(label.clone(), text);
            (Source::new(label), raw)
        })
        .collect();
    let specification = Specification::assemble(parsed).expect("assemble fixture");
    compile(&specification, &sources).expect("compile fixture")
}

fn build_spec() -> BuildSpec {
    BuildSpec::from_yaml(&build_spec_yaml()).expect("build fixture parses")
}

fn build_spec_yaml() -> String {
    format!(
        r#"
format: ess-build/1
build: oracle-runtime
platforms:
  - os: linux
    architecture: amd64
secrets: [registry-token]
nodes:
  - id: base
    kind: oci_base
    reference: docker.io/library/alpine
    digest: sha256:{zeros}
  - id: source
    kind: source
    path: .
    destination: /src
  - id: compile
    kind: run
    base: base
    argv: [cp, /src/oracle, /usr/local/bin/oracle]
    mounts:
      - kind: input
        from: source
        target: /src
  - id: runtime-image
    kind: image
    rootfs: compile
    config:
      entrypoint: [/usr/local/bin/oracle]
      user: "10001"
  - id: chart-file
    kind: artifact
    from: compile
    path: /src/chart.tgz
outputs:
  - name: app
    release_unit: oracle-runtime
    node: runtime-image
    kind: oci_image
    repository: registry.example/oracle
  - name: chart
    release_unit: oracle-chart
    node: chart-file
    kind: helm_chart
"#,
        zeros = "0".repeat(64)
    )
}

fn build() -> BuildIr {
    compile_build(&build_spec()).expect("build compiles")
}

#[test]
fn canonical_build_ir_restores_an_omitted_empty_secret_set() {
    let source = build_spec_yaml().replace("secrets: [registry-token]\n", "");
    let specification = BuildSpec::from_yaml(&source).expect("build without secrets parses");
    let compiled = compile_build(&specification).expect("build without secrets compiles");
    let canonical = compiled.to_canonical_json();
    assert!(canonical.contains("\"platforms\""));
    assert!(!canonical.contains("\"secrets\""));
    assert_eq!(
        BuildIr::from_json(&canonical).expect("canonical build IR reads back"),
        compiled
    );
}

fn component() -> ess_deployment::ComponentIr {
    let specification = ComponentSpec::from_yaml(
        r"
format: ess-component/1
component: oracle
system: oracle
semantic_version: v1
inputs:
  specification: spec/oracle
  realization: ess/realization.yaml
  build: ess/build.yaml
  runtime: ess/runtime.yaml
release_units:
  runtime: oracle-runtime
  chart: oracle-chart
",
    )
    .expect("component fixture parses");
    compile_component(&specification).expect("component compiles")
}

fn physical_realization(semantic: &ess_compiler::EssIr) -> ess_realization::RealizationIr {
    let source = format!(
        r"type: ess-realization/1
id: oracle-implementation
specification:
  system: oracle
  version: v1
  source_digest: sha256:{semantic_digest}
synthesis:
  target: rust-linux-amd64/1
  generator: ess/0.8.0
components: [order-service, dispatch-service]
actors: []
implementations:
  - id: oracle-binary
    components: [order-service, dispatch-service]
    artifact:
      kind: source
      locator: https://example.invalid/oracle.git
      identity: git:{commit}
entrypoints:
  - id: http-api
    title: HTTP API
    summary: Invoke the Oracle API.
    primary: true
    interaction: invoke
    attachment: network
    availability: internal
    support: preview
    implementation: oracle-binary
    actors: []
    surfaces:
      - kind: command
        name: oracle.order.PlaceOrder
    invocation:
      kind: url
      url: http://127.0.0.1:8080
",
        semantic_digest = semantic.source_digest(),
        commit = "a".repeat(40),
    );
    let specification =
        ess_realization::RealizationSpec::from_yaml(&source).expect("realization fixture parses");
    ess_realization::compile(&specification, semantic).expect("physical realization compiles")
}

fn runtime_spec(
    semantic: &ess_compiler::EssIr,
    physical: &ess_realization::RealizationIr,
    build: &BuildIr,
) -> RuntimeSpec {
    RuntimeSpec::from_yaml(&format!(
        r"
format: ess-runtime/1
runtime: oracle-runtime
semantic_digest: sha256:{semantic_digest}
realization_digest: {realization_digest}
build_digest: {build_digest}
processes:
  - name: server
    image: app
containers:
  - name: server
    process: server
    http_port: 8080
    readiness_path: /ready
    liveness_path: /live
    config:
      - name: log-level
        environment: LOG_LEVEL
        kind: optional
    secrets:
      - name: database-password
        environment: DATABASE_PASSWORD
        key: password
    endpoints:
      - name: carrier-api
        environment: CARRIER_URL
        system: carrier
        endpoint: api
    volume_mounts:
      - volume: data
        mount_path: /var/lib/oracle
    audiences: [urn:example:oracle]
workloads:
  - name: oracle
    components: [order-service, dispatch-service]
    containers: [server]
    replicas: 1
    volumes:
      - name: data
        size: 1Gi
provided_endpoints:
  - name: api
    workload: oracle
    container: server
    scheme: http
",
        semantic_digest = semantic.source_digest(),
        realization_digest = physical.realization_digest(),
        build_digest = build.digest(),
    ))
    .expect("runtime fixture parses")
}

fn runtime(semantic: &ess_compiler::EssIr, build: &BuildIr) -> RuntimeIr {
    let physical = physical_realization(semantic);
    compile_runtime(
        &runtime_spec(semantic, &physical, build),
        semantic,
        &physical,
        build,
    )
    .expect("runtime compiles")
}

fn release_manifest(build: &BuildIr, realization: &RuntimeIr, chart: bool) -> ReleaseManifest {
    let digest = format!("sha256:{}", "1".repeat(64));
    let evidence = ["provenance", "sbom", "signature", "conformance"]
        .into_iter()
        .map(|kind| {
            format!(
                r#""{kind}": {{"reference":"registry.example/evidence/{kind}","digest":"{digest}"}}"#
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let (release_unit, version, artifacts) = if chart {
        (
            "oracle-chart",
            "4.5.6",
            format!(
                r#""chart": {{"build_output":"chart","kind":"helm_chart","reference":"oci://registry.example/charts/oracle","digest":"{digest}"}}"#
            ),
        )
    } else {
        (
            "oracle-runtime",
            "1.2.3",
            format!(
                r#""app": {{"build_output":"app","kind":"oci_image","reference":"registry.example/oracle","digest":"{digest}","platforms":{{"linux/amd64":"{digest}"}}}}"#
            ),
        )
    };
    ReleaseManifest::from_json(&format!(
        r#"{{
  "format": "ess-release/1",
  "release_unit": "{release_unit}",
  "system": "oracle",
  "version": "{version}",
  "source_commit": "{commit}",
  "semantic_digest": "{semantic}",
  "build_digest": "{build_digest}",
  "runtime_digest": "{runtime_digest}",
  "artifacts": {{
    {artifacts}
  }},
  "evidence": {{{evidence}}}
}}"#,
        commit = "a".repeat(40),
        semantic = realization.semantic_digest(),
        build_digest = build.digest(),
        runtime_digest = realization.digest(),
    ))
    .expect("release fixture parses")
}

#[test]
fn build_graph_is_canonical_and_projects_executable_buildkit_inputs() {
    let first = build();
    let second = build();
    assert_eq!(first.to_canonical_json(), second.to_canonical_json());
    assert_eq!(first.order().len(), first.nodes().len());
    let files = project_buildkit(&first);
    let dockerfile = &files.files()["Dockerfile.ess"];
    assert!(dockerfile.contains("FROM docker.io/library/alpine@sha256:"));
    assert!(dockerfile.contains("COPY [\".\",\"/src\"]"));
    assert!(!dockerfile.contains("] ["));
    assert!(!dockerfile.contains("] /"));
    assert!(files.files()["docker-bake.hcl"].contains("target \"app\""));
    assert!(!files
        .files()
        .values()
        .any(|value| value.contains("secret value")));
    let graph = project_build_mermaid(&first);
    assert!(graph.starts_with("flowchart LR\n"));
    assert!(graph.contains("source<br/><small>source</small>"));
    assert!(graph.contains("app<br/><small>OCI image · oracle-runtime</small>"));
    assert!(graph.contains("chart<br/><small>Helm chart · oracle-chart</small>"));
    assert_eq!(graph, project_build_mermaid(&second));
}

#[test]
fn undeclared_secrets_and_cycles_are_stage_strict_refusals() {
    let undeclared_yaml = build_spec_yaml()
        .replacen("secrets: [registry-token]", "secrets: []", 1)
        .replacen(
            "      - kind: input\n        from: source\n        target: /src",
            "      - kind: secret\n        secret: registry-token\n        target: /run/secrets/token",
            1,
        );
    let undeclared = BuildSpec::from_yaml(&undeclared_yaml).expect("mutated build parses");
    let refusal = compile_build(&undeclared).expect_err("undeclared secret is refused");
    assert!(refusal.contains(DiagnosticCode::UndeclaredSecret));

    let cycle_text = build_spec_yaml().replacen("    base: base", "    base: runtime-image", 1);
    let cycle = BuildSpec::from_yaml(&cycle_text).expect("cyclic build parses");
    assert!(compile_build(&cycle)
        .expect_err("cycle is refused")
        .contains(DiagnosticCode::DependencyCycle));
}

#[test]
fn realization_runtime_release_stack_and_environment_form_one_exact_chain() {
    let semantic = semantic();
    let build = build();
    let realization = runtime(&semantic, &build);
    let release = release_manifest(&build, &realization, false);
    let chart_release = release_manifest(&build, &realization, true);
    verify_release(&release, &build, &realization).expect("runtime release verifies");
    verify_release(&chart_release, &build, &realization).expect("chart release verifies");

    let stack = StackSpec::from_yaml(&format!(
        r"
format: ess-stack/1
stack: oracle-stack
composition_digest: sha256:{composition}
systems:
  - service: oracle
    system: oracle
    semantic_version: v1
    runtime_release: ^1.0
    chart_release: ^4.0
    surfaces: [oracle.order.PlaceOrder]
external_systems:
  - system: carrier
    contract: carrier-http/v1
    endpoints: [api]
",
        composition = "2".repeat(64)
    ))
    .expect("stack parses");
    let catalog = ReleaseCatalog::from_yaml(&format!(
        r"
format: ess-release-catalog/1
releases:
  - semantic_version: v1
    surfaces: [oracle.order.PlaceOrder]
    release:
{release}
    runtime:
{realization}
  - semantic_version: v1
    surfaces: [oracle.order.PlaceOrder]
    release:
{chart_release}
    runtime:
{realization}
",
        release = indent_yaml(&serde_yaml::to_string(&release).unwrap(), 6),
        chart_release = indent_yaml(&serde_yaml::to_string(&chart_release).unwrap(), 6),
        realization = indent_yaml(&serde_yaml::to_string(&realization).unwrap(), 6),
    ))
    .expect("catalog parses");
    let lock = resolve_stack(&stack, &catalog).expect("stack resolves");
    let lock_again: StackLock =
        serde_json::from_str(&lock.to_canonical_json()).expect("lock reads back");
    assert_eq!(lock, lock_again);

    let environment = EnvironmentSpec::from_yaml(&format!(
        r"
format: ess-environment/1
environment: test
stack_digest: {stack_digest}
cluster: test-cluster
namespace: oracle
releases:
  - service: oracle
    release_name: oracle
    service_account: oracle
    secrets:
      database-password:
        name: oracle-database
        key: password
external_systems:
  - system: carrier
    endpoints:
      api: https://carrier.example.test
",
        stack_digest = lock.digest()
    ))
    .expect("environment parses");
    let deployment = compile_deployment(&environment, &lock).expect("deployment compiles");
    let read_back: DeploymentIr =
        serde_json::from_str(&deployment.to_canonical_json()).expect("deployment reads back");
    assert_eq!(deployment, read_back);
    assert_eq!(deployment.rollout_order.len(), 1);
    assert_eq!(
        deployment.releases[&"oracle".parse().unwrap()].endpoints[&"carrier-api".parse().unwrap()],
        "https://carrier.example.test"
    );

    let chart = project_helm(
        &realization,
        &"oracle".parse().unwrap(),
        &"1.0.0".parse().unwrap(),
    );
    assert!(chart.files()["templates/workloads.yaml"].contains("kind: StatefulSet"));
    assert!(chart.files()["templates/workloads.yaml"].contains("secretKeyRef"));
    assert!(chart.files()["templates/workloads.yaml"].contains("volumeClaimTemplates"));
    assert!(chart.files()["templates/workloads.yaml"].contains("mountPath: \"/var/lib/oracle\""));
    assert!(chart.files()["templates/services.yaml"].contains("kind: Service"));
    assert!(chart.files()["templates/services.yaml"].contains("-api"));
    assert!(chart.files()["values.yaml"].contains("serviceAccount:\n  name: default\n"));
}

#[test]
fn helm_defaults_materialize_typed_secret_slots_without_secret_bytes() {
    let semantic = semantic();
    let runtime = runtime(&semantic, &build());
    let chart = project_helm(
        &runtime,
        &"oracle".parse().unwrap(),
        &"1.0.0".parse().unwrap(),
    );
    assert!(chart.files()["values.yaml"]
        .contains("secrets:\n  database-password:\n    name: \"\"\n    key: \"password\"\n"));
    let schema: serde_json::Value =
        serde_json::from_str(&chart.files()["values.schema.json"]).unwrap();
    assert_eq!(
        schema["properties"]["secrets"]["required"],
        serde_json::json!(["database-password"])
    );
    assert_eq!(
        schema["properties"]["secrets"]["properties"]["database-password"]["required"],
        serde_json::json!(["name", "key"])
    );
}

#[test]
fn component_release_bundle_is_canonical_and_revalidates_after_transport() {
    let semantic = semantic();
    let build = build();
    let runtime = runtime(&semantic, &build);
    let bundle = bundle_release(
        component(),
        build.clone(),
        runtime.clone(),
        vec![
            release_manifest(&build, &runtime, false),
            release_manifest(&build, &runtime, true),
        ],
    )
    .expect("component release bundle verifies");
    let transported = ReleaseBundle::from_json(&bundle.to_canonical_json()).expect("bundle reads");
    let verified = verify_release_bundle(transported).expect("transported bundle re-verifies");
    assert_eq!(verified.digest(), bundle.digest());
}

fn indent_yaml(yaml: &str, spaces: usize) -> String {
    let prefix = " ".repeat(spaces);
    yaml.lines()
        .map(|line| format!("{prefix}{line}"))
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn input_order_does_not_change_locked_bytes() {
    let build = build();
    let mut artifacts: Vec<_> = build.outputs().clone().into_iter().collect();
    artifacts.reverse();
    let reordered: BTreeMap<_, _> = artifacts.into_iter().collect();
    assert_eq!(build.outputs(), &reordered);
    let names: BTreeSet<_> = build.outputs().keys().cloned().collect();
    assert_eq!(names.len(), 2);
}

fn persisted_bundle() -> ReleaseBundle {
    let build = build();
    let runtime = runtime(&semantic(), &build);
    bundle_release(
        component(),
        build.clone(),
        runtime.clone(),
        vec![
            release_manifest(&build, &runtime, false),
            release_manifest(&build, &runtime, true),
        ],
    )
    .unwrap()
}

fn persisted_lock_and_deployment() -> (StackLock, DeploymentIr) {
    let bundle = persisted_bundle();
    let catalog: ReleaseCatalog = serde_json::from_value(serde_json::json!({
        "format": "ess-release-catalog/1",
        "releases": bundle.releases.values().map(|release| serde_json::json!({
            "semantic_version": "v1", "release": release, "runtime": bundle.runtime,
        })).collect::<Vec<_>>()
    }))
    .unwrap();
    let stack: StackSpec = serde_json::from_value(serde_json::json!({
        "format": "ess-stack/1", "stack": "example", "composition_digest": bundle.digest(),
        "systems": (["frontend", "worker"].map(|service| serde_json::json!({
            "service": service, "system": "oracle", "semantic_version": "v1",
            "runtime_release": "^1", "chart_release": "^4",
        }))),
        "external_systems": [{"system": "carrier", "contract": "carrier-http/v1"}],
    }))
    .unwrap();
    let lock = resolve_stack(&stack, &catalog).unwrap();
    let environment: EnvironmentSpec = serde_json::from_value(serde_json::json!({
        "format": "ess-environment/1", "environment": "test", "stack_digest": lock.digest(),
        "cluster": "test-cluster", "namespace": "test",
        "releases": (["frontend", "worker"].map(|service| serde_json::json!({
            "service": service, "release_name": service, "service_account": service,
            "secrets": {"database-password": {"name": "database", "key": "password"}},
            "endpoints": {"carrier-api": "https://example.invalid"},
        }))),
        "external_systems": [{"system": "carrier"}],
    }))
    .unwrap();
    let deployment = compile_deployment(&environment, &lock).unwrap();
    (lock, deployment)
}

fn assert_persisted_refused<T: serde::de::DeserializeOwned>(value: &serde_json::Value) {
    let json = serde_json::to_string(value).unwrap();
    assert!(
        serde_json::from_str::<T>(&json).is_err(),
        "JSON admitted {json}"
    );
    assert!(
        serde_json::from_value::<T>(value.clone()).is_err(),
        "JSON value admitted {json}"
    );
    let yaml = serde_yaml::to_string(value).unwrap();
    assert!(
        serde_yaml::from_str::<T>(&yaml).is_err(),
        "YAML admitted {json}"
    );
    assert!(
        serde_json::from_str::<Vec<T>>(&format!("[{json}]")).is_err(),
        "nested JSON admitted {json}"
    );
    assert!(
        serde_yaml::from_str::<Vec<T>>(&format!("- {}", indent_yaml(&yaml, 2).trim_start()))
            .is_err(),
        "nested YAML admitted {json}"
    );
}

fn assert_mutations_refused<T: serde::Serialize + serde::de::DeserializeOwned>(
    valid: &T,
    mutations: &[(&str, serde_json::Value)],
) {
    let mut admitted = Vec::new();
    for (pointer, replacement) in mutations {
        let mut invalid = serde_json::to_value(valid).unwrap();
        *invalid
            .pointer_mut(pointer)
            .unwrap_or_else(|| panic!("missing fixture field {pointer}")) = replacement.clone();
        if serde_json::from_value::<T>(invalid.clone()).is_ok() {
            admitted.push(format!("{pointer}: {replacement}"));
        } else {
            assert_persisted_refused::<T>(&invalid);
        }
    }
    assert!(admitted.is_empty(), "admitted mutations: {admitted:?}");
}

#[test]
fn persisted_build_readers_refuse_invalid_graphs_and_compiler_constraints() {
    use serde_json::json;
    assert_mutations_refused(
        &build(),
        &[
            ("/format", json!("future/99")),
            ("/platforms", json!([])),
            ("/platforms/0/os", json!("")),
            ("/platforms/0/architecture", json!("")),
            ("/order", json!([])),
            ("/order", json!(["base", "base"])),
            (
                "/order",
                json!(["source", "base", "compile", "chart-file", "runtime-image"]),
            ),
            ("/nodes/compile/base", json!("missing")),
            ("/nodes/compile/base", json!("runtime-image")),
            ("/nodes/source/path", json!("../escape")),
            ("/nodes/source/destination", json!("relative")),
            ("/nodes/base/reference", json!("alpine:latest")),
            ("/nodes/compile/argv", json!([])),
            ("/nodes/compile/argv", json!([""])),
            (
                "/nodes/compile/mounts",
                json!([{"kind":"secret", "secret":"missing", "target":"/secret"}]),
            ),
            ("/nodes/compile/mounts/0/target", json!("relative")),
            ("/nodes/chart-file/path", json!("relative")),
            ("/outputs/app/name", json!("renamed")),
            ("/outputs/app/node", json!("missing")),
            ("/outputs/app/node", json!("chart-file")),
            ("/outputs/app/repository", json!(null)),
            ("/outputs", json!({})),
        ],
    );
}

#[test]
fn persisted_runtime_readers_refuse_local_relationship_and_slot_defects() {
    use serde_json::json;
    let runtime = runtime(&semantic(), &build());
    assert_mutations_refused(
        &runtime,
        &[
            ("/format", json!("future/99")),
            ("/processes/server/name", json!("other")),
            ("/containers/server/name", json!("other")),
            ("/containers/server/process", json!("missing")),
            ("/workloads/oracle/name", json!("other")),
            ("/workloads/oracle/replicas", json!(0)),
            ("/workloads/oracle/containers", json!([])),
            ("/workloads/oracle/containers", json!(["missing"])),
            ("/containers/server/http_port", json!(null)),
            ("/containers/server/config/0/kind", json!("literal")),
            (
                "/containers/server/secrets/0/environment",
                json!("LOG_LEVEL"),
            ),
            ("/containers/server/secrets/0/name", json!("log-level")),
            (
                "/containers/server/volume_mounts/0/mount_path",
                json!("/var/../escape"),
            ),
            (
                "/containers/server/volume_mounts/0/volume",
                json!("missing"),
            ),
            ("/workloads/oracle/volumes/0/size", json!("")),
            ("/provided_endpoints/api/name", json!("other")),
            ("/provided_endpoints/api/workload", json!("missing")),
            ("/provided_endpoints/api/container", json!("missing")),
        ],
    );
    let mut repeated = serde_json::to_value(&runtime).unwrap();
    repeated["workloads"]["another"] = repeated["workloads"]["oracle"].clone();
    repeated["workloads"]["another"]["name"] = json!("another");
    assert_persisted_refused::<RuntimeIr>(&repeated);
}

#[test]
fn persisted_component_and_release_readers_refuse_invalid_local_documents() {
    use serde_json::json;
    assert_mutations_refused(
        &component(),
        &[
            ("/format", json!("future/99")),
            ("/semantic_version", json!("1.0")),
            ("/inputs/runtime", json!("../escape")),
            ("/inputs/build", json!("")),
            ("/release_units/chart", json!("oracle-runtime")),
        ],
    );
    let bundle = persisted_bundle();
    assert_mutations_refused(
        bundle.releases.values().next().unwrap(),
        &[
            ("/format", json!("future/99")),
            ("/source_commit", json!("main")),
            ("/artifacts/chart/build_output", json!("other")),
            ("/artifacts", json!({})),
            ("/evidence", json!({})),
        ],
    );
    let mut image_release =
        serde_json::to_value(&bundle.releases[&"oracle-runtime".parse().unwrap()]).unwrap();
    image_release["artifacts"]["app"]["platforms"] = json!({});
    assert_persisted_refused::<ReleaseManifest>(&image_release);
}

#[test]
fn persisted_lock_readers_preserve_local_service_identity_and_reject_invariants() {
    use serde_json::json;
    let (lock, _) = persisted_lock_and_deployment();
    assert_ne!(
        lock.systems[&"frontend".parse().unwrap()].system.as_str(),
        "frontend"
    );
    assert_mutations_refused(
        &lock,
        &[
            ("/format", json!("future/99")),
            ("/external_systems/carrier/system", json!("other")),
            ("/systems/frontend/depends_on", json!(["missing"])),
            ("/systems/frontend/depends_on", json!(["frontend"])),
            ("/systems/frontend/chart/kind", json!("binary")),
            (
                "/systems/frontend/runtime_artifacts/app/build_output",
                json!("other"),
            ),
            (
                "/systems/frontend/runtime_artifacts/app/platforms",
                json!({}),
            ),
            ("/systems/frontend/runtime_artifacts", json!({})),
            (
                "/systems/frontend/runtime/config/log-level",
                json!("literal"),
            ),
            (
                "/systems/frontend/runtime/endpoint_names",
                json!({"absent":"api"}),
            ),
        ],
    );
    let mut cycle = serde_json::to_value(&lock).unwrap();
    cycle["systems"]["frontend"]["depends_on"] = json!(["worker"]);
    cycle["systems"]["worker"]["depends_on"] = json!(["frontend"]);
    assert_persisted_refused::<StackLock>(&cycle);
}

#[test]
fn persisted_deployment_readers_reject_invalid_release_sets_and_canonical_order() {
    use serde_json::json;
    let (_, deployment) = persisted_lock_and_deployment();
    assert_mutations_refused(
        &deployment,
        &[
            ("/format", json!("future/99")),
            ("/cluster", json!("")),
            ("/rollout_order", json!([])),
            ("/rollout_order", json!(["frontend"])),
            ("/rollout_order", json!(["frontend", "frontend"])),
            ("/rollout_order", json!(["frontend", "missing"])),
            ("/rollout_order", json!(["worker", "frontend"])),
            ("/releases/frontend/service", json!("other")),
            ("/releases/frontend/release_name", json!("")),
            ("/releases/frontend/namespace", json!("")),
            ("/releases/frontend/service_account", json!("")),
            ("/releases/frontend/chart/kind", json!("binary")),
            ("/releases/frontend/images/app/build_output", json!("other")),
            ("/releases/frontend/images/app/kind", json!("binary")),
            ("/releases/frontend/images/app/platforms", json!({})),
        ],
    );
    let mut invalid = serde_json::to_value(&deployment).unwrap();
    invalid["releases"]["frontend"]["depends_on"] = json!(["missing"]);
    assert_persisted_refused::<DeploymentIr>(&invalid);
    invalid["releases"]["frontend"]["depends_on"] = json!(["worker"]);
    invalid["releases"]["worker"]["depends_on"] = json!(["frontend"]);
    assert_persisted_refused::<DeploymentIr>(&invalid);
}

#[test]
fn persisted_bundle_checks_original_keys_and_consistently_rehashed_nested_graphs() {
    use serde_json::json;
    let bundle = persisted_bundle();
    assert_mutations_refused(&bundle, &[("/format", json!("future/99"))]);
    let mut renamed = bundle.clone();
    let release = renamed
        .releases
        .remove(&"oracle-chart".parse().unwrap())
        .unwrap();
    renamed.releases.insert("renamed".parse().unwrap(), release);
    assert!(verify_release_bundle(renamed.clone()).is_err());
    assert_persisted_refused::<ReleaseBundle>(&serde_json::to_value(&renamed).unwrap());

    for pointer in [
        "/runtime/containers/server/process",
        "/runtime/processes/server/image",
    ] {
        let mut invalid = serde_json::to_value(&bundle).unwrap();
        *invalid.pointer_mut(pointer).unwrap() = json!("missing");
        // Preserve field order when hashing by replacing inside compiler-produced bytes.
        let runtime_json = if pointer.contains("containers") {
            bundle
                .runtime
                .to_canonical_json()
                .replace("\"process\": \"server\"", "\"process\": \"missing\"")
        } else {
            bundle
                .runtime
                .to_canonical_json()
                .replace("\"image\": \"app\"", "\"image\": \"missing\"")
        };
        let digest = ess_deployment::Digest::of_bytes(runtime_json.as_bytes());
        for release in invalid["releases"].as_object_mut().unwrap().values_mut() {
            release["runtime_digest"] = json!(digest);
        }
        assert_persisted_refused::<ReleaseBundle>(&invalid);
    }
    let mut mismatch = serde_json::to_value(&bundle).unwrap();
    mismatch["runtime"]["build_digest"] = json!(ess_deployment::Digest::of_bytes(b"other build"));
    assert_persisted_refused::<ReleaseBundle>(&mismatch);
}

#[test]
fn persisted_documents_preserve_compiler_bytes_across_all_public_reader_routes() {
    fn roundtrip<T: serde::Serialize + serde::de::DeserializeOwned>(value: &T) {
        let json = serde_json::to_string_pretty(value).unwrap();
        let from_json: T = serde_json::from_str(&json).unwrap();
        assert_eq!(serde_json::to_string_pretty(&from_json).unwrap(), json);
        let from_yaml: T = serde_yaml::from_str(&serde_yaml::to_string(value).unwrap()).unwrap();
        assert_eq!(serde_json::to_string_pretty(&from_yaml).unwrap(), json);
    }
    let bundle = persisted_bundle();
    roundtrip(&bundle.build);
    roundtrip(&bundle.runtime);
    roundtrip(&bundle.component);
    for release in bundle.releases.values() {
        roundtrip(release);
    }
    roundtrip(&bundle);
    let (lock, deployment) = persisted_lock_and_deployment();
    roundtrip(&lock);
    roundtrip(&deployment);
}

#[test]
fn persisted_readers_reject_duplicate_map_keys_before_collection() {
    fn duplicate<T: serde::Serialize + serde::de::DeserializeOwned>(
        value: &T,
        key: &str,
        entry: &serde_json::Value,
    ) {
        let json = serde_json::to_string(value).unwrap();
        let needle = format!("\"{key}\":");
        let duplicate = format!("\"{key}\":{entry},\"{key}\":");
        let invalid = json.replacen(&needle, &duplicate, 1);
        assert_ne!(invalid, json);
        assert!(
            serde_json::from_str::<T>(&invalid).is_err(),
            "duplicate key admitted {invalid}"
        );
        assert!(serde_json::from_str::<Vec<T>>(&format!("[{invalid}]")).is_err());
    }
    let bundle = persisted_bundle();
    duplicate(
        &bundle.build,
        "app",
        &serde_json::to_value(&bundle.build).unwrap()["outputs"]["app"],
    );
    duplicate(
        &bundle.runtime,
        "server",
        &serde_json::to_value(&bundle.runtime).unwrap()["processes"]["server"],
    );
    let release = &bundle.releases[&"oracle-runtime".parse().unwrap()];
    duplicate(
        release,
        "app",
        &serde_json::to_value(release).unwrap()["artifacts"]["app"],
    );
    duplicate(
        release,
        "linux/amd64",
        &serde_json::to_value(release).unwrap()["artifacts"]["app"]["platforms"]["linux/amd64"],
    );
    duplicate(
        &bundle,
        "oracle-chart",
        &serde_json::to_value(&bundle).unwrap()["releases"]["oracle-chart"],
    );
    let (lock, deployment) = persisted_lock_and_deployment();
    duplicate(
        &lock,
        "frontend",
        &serde_json::to_value(&lock).unwrap()["systems"]["frontend"],
    );
    duplicate(
        &deployment,
        "frontend",
        &serde_json::to_value(&deployment).unwrap()["releases"]["frontend"],
    );
    duplicate(
        &deployment,
        "database-password",
        &serde_json::to_value(&deployment).unwrap()["releases"]["frontend"]["secrets"]
            ["database-password"],
    );
}

#[test]
fn persisted_convenience_readers_and_catalogs_use_the_checked_boundary() {
    fn future<T: serde::Serialize>(value: &T) -> String {
        let mut value = serde_json::to_value(value).unwrap();
        value["format"] = serde_json::json!("future/99");
        serde_json::to_string(&value).unwrap()
    }
    let bundle = persisted_bundle();
    assert!(BuildIr::from_json(&future(&bundle.build)).is_err());
    assert!(RuntimeIr::from_json(&future(&bundle.runtime)).is_err());
    assert!(ess_deployment::ComponentIr::from_json(&future(&bundle.component)).is_err());
    let release = bundle.releases.values().next().unwrap();
    assert!(ReleaseManifest::from_json(&future(release)).is_err());
    assert!(ReleaseManifest::from_yaml(&future(release)).is_err());
    assert!(ReleaseBundle::from_json(&future(&bundle)).is_err());
    assert!(ReleaseBundle::from_yaml(&future(&bundle)).is_err());
    let (lock, deployment) = persisted_lock_and_deployment();
    assert!(StackLock::from_json(&future(&lock)).is_err());
    assert!(DeploymentIr::from_json(&future(&deployment)).is_err());
    let catalog = serde_json::json!({
        "format": "ess-release-catalog/1",
        "releases": [{"semantic_version":"v1", "release":release, "runtime":bundle.runtime}],
    });
    let valid: ReleaseCatalog = serde_json::from_value(catalog.clone()).unwrap();
    assert!(ReleaseCatalog::from_json(&future(&valid)).is_err());
    assert!(ReleaseCatalog::from_yaml(&future(&valid)).is_err());
    assert_mutations_refused(
        &valid,
        &[
            ("/format", serde_json::json!("future/99")),
            (
                "/releases/0/release/runtime_digest",
                serde_json::json!(bundle.build.digest()),
            ),
            (
                "/releases/0/runtime/containers/server/process",
                serde_json::json!("absent"),
            ),
        ],
    );
}

#[test]
fn mutable_public_documents_are_rechecked_at_consuming_entrypoints() {
    let bundle = persisted_bundle();
    let mut release = bundle.releases.values().next().unwrap().clone();
    release.evidence.clear();
    assert!(verify_release(&release, &bundle.build, &bundle.runtime).is_err());
    let (mut lock, mut deployment) = persisted_lock_and_deployment();
    lock.systems
        .get_mut(&"frontend".parse().unwrap())
        .unwrap()
        .chart
        .kind = ess_deployment::ArtifactKind::Binary;
    let environment: EnvironmentSpec = serde_json::from_value(serde_json::json!({
        "format": "ess-environment/1", "environment":"test", "stack_digest": lock.digest(),
        "cluster":"test", "namespace":"test", "releases":[],
    }))
    .unwrap();
    assert!(compile_deployment(&environment, &lock)
        .unwrap_err()
        .contains(DiagnosticCode::InvalidValue));
    deployment.rollout_order.clear();
    assert!(deployment.validate().is_err());
    let mut catalog: ReleaseCatalog = serde_json::from_value(serde_json::json!({
        "format":"ess-release-catalog/1", "releases": [{
            "semantic_version":"v1", "runtime":bundle.runtime,
            "release":bundle.releases.values().next().unwrap(),
        }],
    }))
    .unwrap();
    catalog.releases[0].release.evidence.clear();
    let stack: StackSpec = serde_json::from_value(serde_json::json!({
        "format":"ess-stack/1", "stack":"empty", "composition_digest":bundle.digest(), "systems":[],
    }))
    .unwrap();
    assert!(resolve_stack(&stack, &catalog)
        .unwrap_err()
        .contains(DiagnosticCode::MissingEvidence));
}

#[test]
fn persisted_duplicate_keys_are_rejected_at_every_populated_nested_map() {
    fn walk(value: &serde_json::Value, path: &str, paths: &mut Vec<String>) {
        match value {
            serde_json::Value::Object(map) => {
                if !map.is_empty() {
                    paths.push(path.to_owned());
                }
                for (key, value) in map {
                    walk(
                        value,
                        &format!("{path}/{}", key.replace('~', "~0").replace('/', "~1")),
                        paths,
                    );
                }
            }
            serde_json::Value::Array(values) => {
                for (index, value) in values.iter().enumerate() {
                    walk(value, &format!("{path}/{index}"), paths);
                }
            }
            _ => {}
        }
    }
    fn check<T: serde::Serialize + serde::de::DeserializeOwned>(document: &T) {
        let document = serde_json::to_value(document).unwrap();
        let mut paths = Vec::new();
        walk(&document, "", &mut paths);
        assert!(!paths.is_empty());
        for pointer in paths {
            let parent = document.pointer(&pointer).unwrap();
            let (key, value) = parent.as_object().unwrap().iter().next().unwrap();
            let raw = serde_json::to_string(parent).unwrap().replacen(
                '{',
                &format!("{{{}:{value},", serde_json::to_string(key).unwrap()),
                1,
            );
            let mut invalid = document.clone();
            *invalid.pointer_mut(&pointer).unwrap() =
                serde_json::json!("__duplicate_map_fixture__");
            let json = serde_json::to_string(&invalid)
                .unwrap()
                .replace("\"__duplicate_map_fixture__\"", &raw);
            assert!(
                serde_json::from_str::<T>(&json).is_err(),
                "JSON duplicate admitted at {pointer}"
            );
            assert!(
                serde_yaml::from_str::<T>(&json).is_err(),
                "YAML duplicate admitted at {pointer}"
            );
        }
    }
    let mut source = build_spec();
    for node in &mut source.nodes {
        if let ess_deployment::BuildNode::Run { environment, .. } = &mut node.node {
            environment.insert("LANG".to_owned(), "C".to_owned());
        }
        if let ess_deployment::BuildNode::Image { config, .. } = &mut node.node {
            config.environment.insert("LANG".to_owned(), "C".to_owned());
        }
    }
    check(&compile_build(&source).unwrap());
    check(&persisted_bundle());
    let (lock, mut deployment) = persisted_lock_and_deployment();
    deployment
        .releases
        .get_mut(&"frontend".parse().unwrap())
        .unwrap()
        .config
        .insert("log-level".parse().unwrap(), "debug".to_owned());
    check(&lock);
    check(&deployment);
}

#[test]
fn adversary_runtime_readers_match_compiler_slot_and_volume_refusals() {
    let semantic = semantic();
    let build = build();
    let physical = physical_realization(&semantic);
    let valid = runtime_spec(&semantic, &physical, &build);
    let compiled = compile_runtime(&valid, &semantic, &physical, &build).unwrap();
    assert_eq!(
        RuntimeIr::from_json(&compiled.to_canonical_json())
            .unwrap()
            .to_canonical_json(),
        compiled.to_canonical_json()
    );
    let mut variants = Vec::new();
    let mut invalid = valid.clone();
    invalid.containers[0].config[0].value = Some("forbidden optional literal".to_owned());
    variants.push(invalid);
    let mut invalid = valid.clone();
    invalid.containers[0].endpoints[0].environment = "DATABASE_PASSWORD".to_owned();
    variants.push(invalid);
    let mut invalid = valid.clone();
    invalid.containers[0]
        .volume_mounts
        .push(valid.containers[0].volume_mounts[0].clone());
    variants.push(invalid);
    let mut invalid = valid.clone();
    invalid.workloads[0].storage = Some("1Gi".to_owned());
    variants.push(invalid);
    let mut invalid = valid.clone();
    invalid.workloads[0]
        .volumes
        .push(valid.workloads[0].volumes[0].clone());
    variants.push(invalid);

    for invalid in variants {
        assert!(compile_runtime(&invalid, &semantic, &physical, &build).is_err());
        let mut persisted = serde_json::to_value(&compiled).unwrap();
        persisted["containers"]["server"] = serde_json::to_value(&invalid.containers[0]).unwrap();
        persisted["workloads"]["oracle"] = serde_json::to_value(&invalid.workloads[0]).unwrap();
        assert_persisted_refused::<RuntimeIr>(&persisted);
    }
}

#[test]
fn adversary_rehashed_bundle_cannot_launder_a_build_cycle() {
    let bundle = persisted_bundle();
    let canonical = bundle.to_canonical_json();
    assert_eq!(
        ReleaseBundle::from_json(&canonical)
            .unwrap()
            .to_canonical_json(),
        canonical
    );
    let invalid_build = bundle
        .build
        .to_canonical_json()
        .replace("\"base\": \"base\"", "\"base\": \"runtime-image\"");
    assert_ne!(invalid_build, bundle.build.to_canonical_json());
    let build_digest = ess_deployment::Digest::of_bytes(invalid_build.as_bytes());
    let invalid_runtime = bundle
        .runtime
        .to_canonical_json()
        .replace(bundle.build.digest().as_str(), build_digest.as_str());
    // The standalone runtime is locally valid; the included build makes the graph provable.
    let runtime = RuntimeIr::from_json(&invalid_runtime).unwrap();
    let mut invalid = serde_json::to_value(&bundle).unwrap();
    invalid["build"] = serde_json::from_str(&invalid_build).unwrap();
    invalid["runtime"] = serde_json::to_value(&runtime).unwrap();
    for release in invalid["releases"].as_object_mut().unwrap().values_mut() {
        release["build_digest"] = serde_json::json!(build_digest);
        release["runtime_digest"] = serde_json::json!(runtime.digest());
    }
    assert_eq!(runtime.build_digest(), &build_digest);
    assert_persisted_refused::<ReleaseBundle>(&invalid);
}

#[test]
fn adversary_unselected_catalog_candidate_mutation_is_revalidated() {
    let bundle = persisted_bundle();
    let catalog: ReleaseCatalog = serde_json::from_value(serde_json::json!({
        "format":"ess-release-catalog/1",
        "releases": bundle.releases.values().map(|release| serde_json::json!({
            "semantic_version":"v1", "release":release, "runtime":bundle.runtime,
        })).collect::<Vec<_>>(),
    }))
    .unwrap();
    let stack: StackSpec = serde_json::from_value(serde_json::json!({
        "format":"ess-stack/1", "stack":"empty", "systems":[],
        "composition_digest":bundle.digest(),
    }))
    .unwrap();
    assert!(resolve_stack(&stack, &catalog).is_ok());
    for field in ["build_digest", "semantic_digest", "runtime_digest"] {
        let mut invalid = catalog.clone();
        let digest = ess_deployment::Digest::of_bytes(b"other input");
        match field {
            "build_digest" => invalid.releases[1].release.build_digest = digest,
            "semantic_digest" => invalid.releases[1].release.semantic_digest = digest,
            "runtime_digest" => invalid.releases[1].release.runtime_digest = digest,
            _ => unreachable!(),
        }
        assert!(resolve_stack(&stack, &invalid)
            .unwrap_err()
            .contains(DiagnosticCode::DigestMismatch));
        assert_persisted_refused::<ReleaseCatalog>(&serde_json::to_value(&invalid).unwrap());
    }
}

// ---------------------------------------------------------------------------
// A component declares its settings, and the runtime slots are derived from them.
//
// `ess-runtime/1` types configuration and says nothing about what any of it *is*. A component that
// declares `settings:` says it once, in the specification, and the slots follow from that — so a
// runtime document that also hand-authors them is two sources for one fact, and is refused.
// ---------------------------------------------------------------------------

/// The fixture's semantic model, with `settings:` added to `order-service`.
///
/// The declaration is injected into the text rather than committed to `examples/oracle-fixture`,
/// because every other test in this file — and `ess-compiler`'s oracle tests — pin that directory's
/// digest, and a component that declares no settings is exactly the case they are pinning.
fn semantic_with_settings() -> ess_compiler::EssIr {
    let root = fixture();
    let mut sources = SourceMap::new();
    let parsed: Vec<_> = [
        "system.yaml",
        "components.yaml",
        "domains/order.yaml",
        "domains/dispatch.yaml",
    ]
    .into_iter()
    .map(|label| {
        let mut text = std::fs::read_to_string(root.join(label)).expect("read fixture");
        if label == "components.yaml" {
            text = text.replace(
                "  - component: order-service\n",
                "  - component: order-service\n    settings:\n      \
                 - name: state-root\n        type: oracle.order.Email\n        required: true\n      \
                 - name: carrier-token\n        type: oracle.order.Email\n        required: true\n        secret: true\n",
            );
        }
        let raw = RawSpecFile::parse(&text).expect("parse fixture");
        sources.insert(label.to_owned(), text);
        (Source::new(label), raw)
    })
    .collect();
    let specification = Specification::assemble(parsed).expect("assemble fixture");
    compile(&specification, &sources).expect("compile fixture")
}

/// The fixture runtime document, with whatever `config:` and `secrets:` blocks are passed in.
fn runtime_spec_with_slots(
    semantic: &ess_compiler::EssIr,
    physical: &ess_realization::RealizationIr,
    build: &BuildIr,
    slots: &str,
) -> RuntimeSpec {
    RuntimeSpec::from_yaml(&format!(
        r"
format: ess-runtime/1
runtime: oracle-runtime
semantic_digest: sha256:{semantic_digest}
realization_digest: {realization_digest}
build_digest: {build_digest}
processes:
  - name: server
    image: app
containers:
  - name: server
    process: server
    http_port: 8080
{slots}    endpoints:
      - name: carrier-api
        environment: CARRIER_URL
        system: carrier
        endpoint: api
workloads:
  - name: oracle
    components: [order-service, dispatch-service]
    containers: [server]
    replicas: 1
provided_endpoints:
  - name: api
    workload: oracle
    container: server
    scheme: http
",
        semantic_digest = semantic.source_digest(),
        realization_digest = physical.realization_digest(),
        build_digest = build.digest(),
    ))
    .expect("runtime fixture parses")
}

#[test]
fn declared_settings_derive_the_runtime_slots() {
    let semantic = semantic_with_settings();
    let build = build();
    let physical = physical_realization(&semantic);
    let compiled = compile_runtime(
        &runtime_spec_with_slots(&semantic, &physical, &build, ""),
        &semantic,
        &physical,
        &build,
    )
    .expect("a runtime that hand-authors no slot compiles");

    let container = &compiled.containers()[&"server".parse().unwrap()];
    assert_eq!(
        container
            .config
            .iter()
            .map(|slot| (slot.name.to_string(), slot.environment.clone(), slot.kind))
            .collect::<Vec<_>>(),
        vec![(
            "state-root".to_owned(),
            "STATE_ROOT".to_owned(),
            ess_deployment::ConfigKind::Required
        )],
        "a non-secret setting becomes one config slot, keyed by an upper-snake-cased name"
    );
    assert_eq!(
        container
            .secrets
            .iter()
            .map(|slot| (
                slot.name.to_string(),
                slot.environment.clone(),
                slot.key.clone()
            ))
            .collect::<Vec<_>>(),
        vec![(
            "carrier-token".to_owned(),
            "CARRIER_TOKEN".to_owned(),
            "carrier-token".to_owned()
        )],
        "a secret setting becomes a secret slot whose key is the setting's own name"
    );
}

#[test]
fn deriving_the_runtime_slots_twice_produces_identical_bytes() {
    let semantic = semantic_with_settings();
    let build = build();
    let physical = physical_realization(&semantic);
    let specification = runtime_spec_with_slots(&semantic, &physical, &build, "");
    let first = compile_runtime(&specification, &semantic, &physical, &build)
        .expect("compiles")
        .to_canonical_json();
    let second = compile_runtime(&specification, &semantic, &physical, &build)
        .expect("compiles")
        .to_canonical_json();
    assert_eq!(first.as_bytes(), second.as_bytes());
}

#[test]
fn a_hand_authored_slot_beside_a_declared_setting_is_refused_naming_both() {
    let semantic = semantic_with_settings();
    let build = build();
    let physical = physical_realization(&semantic);
    let specification = runtime_spec_with_slots(
        &semantic,
        &physical,
        &build,
        "    config:\n      - name: log-level\n        environment: LOG_LEVEL\n        kind: optional\n",
    );
    let refusal = compile_runtime(&specification, &semantic, &physical, &build)
        .expect_err("two sources for one fact is refused");
    let rendered = refusal
        .as_slice()
        .iter()
        .map(|diagnostic| {
            format!(
                "{} {}",
                diagnostic
                    .subject()
                    .map_or_else(String::new, ToString::to_string),
                diagnostic.detail()
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        rendered.contains("log-level"),
        "the refusal names the hand-authored slot: {rendered}"
    );
    assert!(
        rendered.contains("order-service"),
        "and the component whose settings replace it: {rendered}"
    );
}

#[test]
fn a_hand_authored_secret_beside_a_declared_setting_is_refused_naming_both() {
    let semantic = semantic_with_settings();
    let build = build();
    let physical = physical_realization(&semantic);
    let specification = runtime_spec_with_slots(
        &semantic,
        &physical,
        &build,
        "    secrets:\n      - name: database-password\n        environment: DATABASE_PASSWORD\n        key: password\n",
    );
    let refusal = compile_runtime(&specification, &semantic, &physical, &build)
        .expect_err("a hand-authored secret is the same two sources");
    let rendered = refusal
        .as_slice()
        .iter()
        .map(|diagnostic| {
            format!(
                "{} {}",
                diagnostic
                    .subject()
                    .map_or_else(String::new, ToString::to_string),
                diagnostic.detail()
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        rendered.contains("database-password") && rendered.contains("order-service"),
        "the refusal names both: {rendered}"
    );
}

/// The property that makes this additive rather than a migration.
#[test]
fn a_component_declaring_no_settings_keeps_its_hand_authored_slots() {
    let semantic = semantic();
    let build = build();
    let physical = physical_realization(&semantic);
    let compiled = compile_runtime(
        &runtime_spec(&semantic, &physical, &build),
        &semantic,
        &physical,
        &build,
    )
    .expect("the hand-authored fixture still compiles");
    let container = &compiled.containers()[&"server".parse().unwrap()];
    assert_eq!(
        container
            .config
            .iter()
            .map(|slot| slot.environment.clone())
            .collect::<Vec<_>>(),
        vec!["LOG_LEVEL".to_owned()],
        "nothing derived, nothing removed"
    );
    assert_eq!(
        container
            .secrets
            .iter()
            .map(|slot| slot.environment.clone())
            .collect::<Vec<_>>(),
        vec!["DATABASE_PASSWORD".to_owned()]
    );
}

// ---------------------------------------------------------------------------
// Two properties of the derivation that its first shape did not have, each found by the adversary
// pass recorded at `review-result:adversary-wave25-unit3-pass-1`.
// ---------------------------------------------------------------------------

/// The fixture's semantic model with a `settings:` block spliced under each named component.
///
/// The general form of [`semantic_with_settings`], which splices one block under `order-service`.
/// Injected into the text rather than committed to `examples/oracle-fixture` for the same reason:
/// every other case in this file pins that directory's digest.
fn semantic_with_settings_on(settings: &[(&str, &str)]) -> ess_compiler::EssIr {
    let root = fixture();
    let mut sources = SourceMap::new();
    let parsed: Vec<_> = [
        "system.yaml",
        "components.yaml",
        "domains/order.yaml",
        "domains/dispatch.yaml",
    ]
    .into_iter()
    .map(|label| {
        let mut text = std::fs::read_to_string(root.join(label)).expect("read fixture");
        if label == "components.yaml" {
            for (component, block) in settings {
                let anchor = format!("  - component: {component}\n");
                assert!(
                    text.contains(&anchor),
                    "the fixture declares the component {component}"
                );
                text = text.replace(&anchor, &format!("{anchor}    settings:\n{block}"));
            }
        }
        let raw = RawSpecFile::parse(&text).expect("parse fixture");
        sources.insert(label.to_owned(), text);
        (Source::new(label), raw)
    })
    .collect();
    let specification = Specification::assemble(parsed).expect("assemble fixture");
    compile(&specification, &sources).expect("compile fixture")
}

/// The fixture runtime document with whatever `workloads:` block is passed in, and no slot of its
/// own: no `config:`, no `secrets:`, no `endpoints:`.
fn runtime_spec_with_workloads(
    semantic: &ess_compiler::EssIr,
    physical: &ess_realization::RealizationIr,
    build: &BuildIr,
    workloads: &str,
) -> RuntimeSpec {
    RuntimeSpec::from_yaml(&format!(
        r"
format: ess-runtime/1
runtime: oracle-runtime
semantic_digest: sha256:{semantic_digest}
realization_digest: {realization_digest}
build_digest: {build_digest}
processes:
  - name: server
    image: app
containers:
  - name: server
    process: server
    http_port: 8080
{workloads}provided_endpoints:
  - name: api
    workload: oracle
    container: server
    scheme: http
",
        semantic_digest = semantic.source_digest(),
        realization_digest = physical.realization_digest(),
        build_digest = build.digest(),
    ))
    .expect("runtime fixture parses")
}

const ONE_WORKLOAD: &str = "\
workloads:
  - name: oracle
    components: [order-service, dispatch-service]
    containers: [server]
    replicas: 1
";

/// Two workloads, each realizing one component, both selecting the one container role.
///
/// Nothing in `ess-runtime/1` refuses this: `compile_runtime` refuses a *component* realized by
/// more than one workload, and says nothing about a container role selected by more than one.
const TWO_WORKLOADS_ONE_CONTAINER: &str = "\
workloads:
  - name: dispatch
    components: [dispatch-service]
    containers: [server]
    replicas: 1
  - name: oracle
    components: [order-service]
    containers: [server]
    replicas: 1
";

fn rendered(diagnostics: &ess_deployment::Diagnostics) -> String {
    diagnostics
        .as_slice()
        .iter()
        .map(|diagnostic| {
            format!(
                "{:?} {} {}",
                diagnostic.code(),
                diagnostic
                    .subject()
                    .map_or_else(String::new, ToString::to_string),
                diagnostic.detail()
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// A type that admits no absence must not derive a slot the deployment gate treats as unbindable.
///
/// `validate_settings` refuses `required: false` over `oracle.order.Email` because — its own words
/// — "`Optional<…>` is the model's only way of saying that a value may be absent". Deleting the
/// `required:` line must therefore not buy the same slot the refusal exists to prevent: silence is
/// answered by the type, not by `false`, and `ConfigKind::Optional` is the kind `environment.rs`
/// does *not* require an environment binding for.
#[test]
fn a_setting_whose_type_admits_no_absence_does_not_derive_an_optional_slot() {
    let semantic = semantic_with_settings_on(&[(
        "order-service",
        "      - name: state-root\n        type: oracle.order.Email\n",
    )]);
    let build = build();
    let physical = physical_realization(&semantic);
    let compiled = compile_runtime(
        &runtime_spec_with_workloads(&semantic, &physical, &build, ONE_WORKLOAD),
        &semantic,
        &physical,
        &build,
    )
    .expect("a runtime that hand-authors no slot compiles");

    let container = &compiled.containers()[&"server".parse().unwrap()];
    let derived: Vec<_> = container
        .config
        .iter()
        .map(|slot| (slot.name.to_string(), slot.kind))
        .collect();
    assert_eq!(
        derived,
        vec![(
            "state-root".to_owned(),
            ess_deployment::ConfigKind::Required
        )],
        "a setting typed `oracle.order.Email` says the value is present — `Optional<…>` is the \
         model's only way to say otherwise — so the derived slot must be `required`, and an \
         environment that leaves it unbound must be refused by `environment.rs`"
    );
}

/// A runtime document that hand-authors nothing must not be told that it hand-authored a slot.
///
/// The refusal reads `container.config`/`container.secrets` to decide whether the container has a
/// second author. When the derivation walked workloads and appended in place, a container role
/// selected by two workloads read its own first derivation back as hand-authored work and the
/// document was refused with a message naming a slot that appears nowhere in it.
#[test]
fn a_container_role_shared_by_two_workloads_is_not_accused_of_hand_authoring() {
    let semantic = semantic_with_settings_on(&[
        (
            "order-service",
            "      - name: state-root\n        type: oracle.order.Email\n        required: true\n",
        ),
        (
            "dispatch-service",
            "      - name: carrier-token\n        type: oracle.order.Email\n        required: true\n",
        ),
    ]);
    let build = build();
    let physical = physical_realization(&semantic);
    let specification =
        runtime_spec_with_workloads(&semantic, &physical, &build, TWO_WORKLOADS_ONE_CONTAINER);
    let source = serde_json::to_string(&specification).unwrap_or_default();
    assert!(
        !source.contains("\"config\":[{"),
        "the runtime document under test hand-authors no config slot"
    );

    match compile_runtime(&specification, &semantic, &physical, &build) {
        Ok(compiled) => {
            let container = &compiled.containers()[&"server".parse().unwrap()];
            assert_eq!(
                container
                    .config
                    .iter()
                    .map(|slot| slot.name.to_string())
                    .collect::<Vec<_>>(),
                vec!["carrier-token".to_owned(), "state-root".to_owned()],
                "the shared role derives the union of both workloads' settings, once each, in \
                 component-name order"
            );
        }
        Err(diagnostics) => {
            let text = rendered(&diagnostics);
            assert!(
                !text.contains("hand-authors the slot"),
                "the runtime document hand-authors no slot, yet it is refused for hand-authoring \
                 one; `derive_component_settings` read back its own derivation from the previous \
                 workload:\n{text}"
            );
            assert!(
                !text.contains(&format!("{:?}", DiagnosticCode::DuplicateIdentifier)),
                "nor is any slot a duplicate:\n{text}"
            );
        }
    }
}
