//! Reuses the independently compiled delivery-model fixture; OCI envelopes are assembled separately.
use ess_compiler::{resolve::compile, source::SourceMap};
use ess_deployment::{
    bundle_release, compile_build, compile_component, compile_runtime, BuildIr, BuildSpec,
    ComponentSpec, ReleaseBundle, ReleaseManifest, RuntimeIr, RuntimeSpec,
};
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source,
};
use std::path::{Path, PathBuf};
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

pub fn persisted_bundle() -> ReleaseBundle {
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
