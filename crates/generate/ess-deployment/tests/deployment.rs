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
