//! Adopter-level outcomes for the native semantic-to-observation boundary.
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT: AtomicU64 = AtomicU64::new(0);
fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}
fn ess() -> Command {
    Command::new(env!("CARGO_BIN_EXE_ess"))
}
fn write(path: &Path, value: &Value) {
    std::fs::write(path, serde_json::to_vec_pretty(value).unwrap()).unwrap();
}
struct Fixture {
    dir: PathBuf,
    bindings: Value,
    observation: Value,
}
impl Fixture {
    fn new() -> Self {
        let dir = std::env::temp_dir().join(format!(
            "ess-observed-bindings-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&dir).unwrap();
        let mut realization: Value = serde_yaml::from_str(
            &std::fs::read_to_string(root().join("examples/realizations/billing-local.yaml"))
                .unwrap(),
        )
        .unwrap();
        let digest = format!("sha256:{}", "a".repeat(64));
        let image = format!("registry.example/billing@{digest}");
        realization["implementations"][0]["artifact"] =
            json!({"kind":"container", "locator": image, "identity":digest});
        write(&dir.join("realization.json"), &realization);
        let compiled = ess()
            .args(["specify", "realization", "compile", "--path"])
            .arg(dir.join("realization.json"))
            .arg("--spec")
            .arg(root().join("examples/billing"))
            .args(["--format", "json"])
            .output()
            .unwrap();
        assert!(compiled.status.success(), "{compiled:?}");
        let ir: Value = serde_json::from_slice(&compiled.stdout).unwrap();
        let bindings = json!({
            "format":"ess-observed-bindings/1", "id":"billing-qa", "realization_digest":ir["realization_digest"],
            "scope":{"context":"synthetic-context", "namespace":"app"},
            "bindings":[{"id":"api", "implementation":"billing-binary", "workload":{"kind":"deployment", "name":"web"}, "container":"web", "image":image}]
        });
        let mut observation: Value = serde_json::from_slice(
            &std::fs::read(
                root().join("crates/infra/infra-compiler/tests/fixtures/namespace-topology.json"),
            )
            .unwrap(),
        )
        .unwrap();
        observation["kinds"]["deployments"]["items"][0]["spec"]["template"]["spec"]["containers"]
            [0]["image"] = json!(image);
        // A producer that records init containers writes the key even when it found none; the
        // fixture states that, so OBS-BIND-008 has evidence to be satisfied from.
        observation["kinds"]["deployments"]["items"][0]["spec"]["template"]["spec"]
            ["initContainers"] = json!([]);
        Self {
            dir,
            bindings,
            observation,
        }
    }
    fn command(&self) -> Command {
        write(&self.dir.join("bindings.json"), &self.bindings);
        write(&self.dir.join("observation.json"), &self.observation);
        let mut cmd = ess();
        cmd.args(["verify", "bindings", "--spec"])
            .arg(root().join("examples/billing"))
            .arg("--realization")
            .arg(self.dir.join("realization.json"))
            .arg("--bindings")
            .arg(self.dir.join("bindings.json"))
            .args(["--format", "json"]);
        cmd
    }
    fn run(&self) -> (Output, Value) {
        let output = self
            .command()
            .arg("--infra")
            .arg(self.dir.join("observation.json"))
            .output()
            .unwrap();
        let report = serde_json::from_slice(&output.stdout)
            .expect("exactly one JSON report even for refusal");
        (output, report)
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.dir).unwrap();
    }
}
fn check(report: &Value, code: &str) -> Value {
    report["bindings"]["api"]["checks"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["code"] == code)
        .unwrap()["status"]
        .clone()
}

#[test]
fn exact_artifact_reference_succeeds_and_repeated_bytes_are_identical() {
    let fixture = Fixture::new();
    let (first, report) = fixture.run();
    assert_eq!(first.status.code(), Some(0), "{first:?}");
    assert_eq!(report["status"], "satisfied");
    assert_eq!(report["acquisition"], "supplied_observation");
    assert_eq!(
        report["bindings"]["api"]["components"],
        json!(["invoice-service"])
    );
    assert_eq!(report["bindings"]["api"]["observed_uid"], "workload-1");
    assert_eq!(first.stdout, fixture.run().0.stdout);
    let output = fixture
        .command()
        .arg("--infra")
        .arg(fixture.dir.join("observation.json"))
        .arg("--markdown-out")
        .arg(fixture.dir.join("report.md"))
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(std::fs::read_to_string(fixture.dir.join("report.md"))
        .unwrap()
        .contains("invoice-service"));
}

#[test]
fn a_tag_is_unknown_even_when_template_and_declared_image_agree() {
    let mut fixture = Fixture::new();
    fixture.bindings["bindings"][0]["image"] = json!("registry.example/billing:1");
    fixture.observation["kinds"]["deployments"]["items"][0]["spec"]["template"]["spec"]
        ["containers"][0]["image"] = json!("registry.example/billing:1");
    let (output, report) = fixture.run();
    assert_eq!(output.status.code(), Some(2));
    assert_eq!(check(&report, "OBS-BIND-004"), "satisfied");
    assert_eq!(check(&report, "OBS-BIND-005"), "unknown");
}

#[test]
fn workload_container_and_image_changes_are_separate_violations() {
    let mut fixture = Fixture::new();
    fixture.observation["kinds"]["deployments"]["items"][0]["spec"]["template"]["spec"]
        ["containers"][0]["image"] = json!(format!(
        "registry.example/billing@sha256:{}",
        "b".repeat(64)
    ));
    let (output, report) = fixture.run();
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(check(&report, "OBS-BIND-004"), "violated");
    assert_eq!(check(&report, "OBS-BIND-005"), "violated");
    fixture.bindings["bindings"][0]["container"] = json!("missing");
    let (output, report) = fixture.run();
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(check(&report, "OBS-BIND-003"), "violated");
    fixture.observation["kinds"]["deployments"]["items"] = json!([]);
    let (output, report) = fixture.run();
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(check(&report, "OBS-BIND-002"), "violated");
}

#[test]
fn scope_mismatch_and_legacy_coverage_never_prove_target_absence() {
    let mut fixture = Fixture::new();
    fixture.bindings["scope"]["context"] = json!("another-cluster");
    let (output, report) = fixture.run();
    assert_eq!(output.status.code(), Some(2));
    assert_eq!(check(&report, "OBS-BIND-002"), "unknown");
    fixture.bindings["scope"]["context"] = json!("synthetic-context");
    fixture.bindings["scope"]["namespace"] = json!("elsewhere");
    assert_eq!(fixture.run().0.status.code(), Some(2));
    fixture.bindings["scope"]["namespace"] = json!("app");
    fixture.observation["format"] = json!("infra-observation/1");
    fixture
        .observation
        .as_object_mut()
        .unwrap()
        .remove("coverage");
    let (output, report) = fixture.run();
    assert_eq!(output.status.code(), Some(2));
    assert_eq!(check(&report, "OBS-BIND-002"), "unknown");
}

#[test]
fn malformed_and_empty_contracts_and_invalid_references_fail_before_acquisition() {
    let fixture = Fixture::new();
    for mutate in [
        |v: &mut Value| {
            v["bindings"] = json!([]);
        },
        |v: &mut Value| {
            let duplicate = v["bindings"][0].clone();
            v["bindings"].as_array_mut().unwrap().push(duplicate);
        },
        |v: &mut Value| {
            v["bindings"][0]["implementation"] = json!("unknown");
        },
        |v: &mut Value| {
            v["realization_digest"] = json!(format!("sha256:{}", "f".repeat(64)));
        },
        |v: &mut Value| {
            v["scope"]["namespace"] = json!("../other");
        },
        |v: &mut Value| {
            v["bindings"][0]["unexpected"] = json!(true);
        },
    ] {
        let mut changed = fixture.bindings.clone();
        mutate(&mut changed);
        let mut command = fixture.command();
        write(&fixture.dir.join("bindings.json"), &changed);
        let output = command
            .args(["--infra", "/deliberately-missing-observation"])
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(1));
        let report: Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(report["checks"][0]["code"], "OBS-BIND-000");
        assert!(report["observation"].is_null());
    }
}

#[test]
fn two_explicit_roles_can_use_one_implementation_and_order_is_canonical() {
    let mut fixture = Fixture::new();
    let mut other = fixture.observation["kinds"]["deployments"]["items"][0].clone();
    other["metadata"]["name"] = json!("worker");
    other["metadata"]["uid"] = json!("worker-1");
    fixture.observation["kinds"]["deployments"]["items"]
        .as_array_mut()
        .unwrap()
        .push(other);
    let mut binding = fixture.bindings["bindings"][0].clone();
    binding["id"] = json!("worker");
    binding["workload"]["name"] = json!("worker");
    fixture.bindings["bindings"]
        .as_array_mut()
        .unwrap()
        .push(binding);
    let (first, report) = fixture.run();
    assert_eq!(first.status.code(), Some(0), "{first:?}");
    assert_eq!(report["bindings"].as_object().unwrap().len(), 2);
    fixture.bindings["bindings"]
        .as_array_mut()
        .unwrap()
        .reverse();
    assert_eq!(first.stdout, fixture.run().0.stdout);
}

#[test]
fn unavailable_observation_is_unknown_and_existing_output_is_preserved() {
    let fixture = Fixture::new();
    let output = fixture
        .command()
        .args(["--infra", "/deliberately-missing-observation"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["status"], "unknown");
    assert_eq!(report["checks"][0]["code"], "OBS-BIND-006");
    let existing = fixture.dir.join("existing.json");
    std::fs::write(&existing, b"keep me").unwrap();
    let output = fixture
        .command()
        .args(["--live", "--observation-out"])
        .arg(&existing)
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert_eq!(std::fs::read(&existing).unwrap(), b"keep me");
}

#[test]
#[cfg(unix)]
fn failed_live_collection_is_json_unknown_and_does_not_use_an_old_snapshot() {
    let fixture = Fixture::new();
    let bin = fixture.dir.join("bin");
    std::fs::create_dir(&bin).unwrap();
    let output_path = fixture.dir.join("new-observation.json");
    let output = fixture
        .command()
        .args(["--live", "--observation-out"])
        .arg(&output_path)
        .env("PATH", &bin)
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["checks"][0]["code"], "OBS-BIND-006");
    assert!(
        report["checks"][0]["detail"]
            .as_str()
            .unwrap()
            .contains("kubectl"),
        "{report}"
    );
    assert!(report["observation"].is_null());
    assert!(!output_path.exists());
}

#[test]
fn live_output_in_a_checkout_is_refused_before_collection() {
    let fixture = Fixture::new();
    std::fs::write(fixture.dir.join(".git"), "gitdir: /unavailable/linked-tree").unwrap();
    let output_path = fixture.dir.join("new-observation.json");
    let output = fixture
        .command()
        .args(["--live", "--observation-out"])
        .arg(&output_path)
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(report["checks"][0]["detail"]
        .as_str()
        .unwrap()
        .contains("outside Git"));
    assert!(!output_path.exists());
}

#[test]
fn discovery_manifest_model_preserves_offline_binding_results_and_refuses_before_collection() {
    // Preserve this new fixture and its exact process records for the acquisition boundary.
    let fixture = std::mem::ManuallyDrop::new(Fixture::new());
    let model = fixture.dir.join("selected-model");
    std::fs::create_dir_all(model.join("model/domains")).unwrap();
    let files = [
        "system.yaml",
        "components.yaml",
        "domains/invoice.yaml",
        "domains/email.yaml",
        "topology.yaml",
    ];
    for file in files {
        std::fs::copy(
            root().join("examples/billing").join(file),
            model.join("model").join(file),
        )
        .unwrap();
    }
    write(
        &model.join("ess-inputs.yaml"),
        &json!({"format":"ess-inputs/1","specification":files.map(|p|format!("model/{p}")),"scenarios":["missing-inactive"]}),
    );
    let (baseline, _) = fixture.run();
    let mut command = ess();
    command
        .args(["verify", "bindings", "--spec"])
        .arg(&model)
        .arg("--realization")
        .arg(fixture.dir.join("realization.json"))
        .arg("--bindings")
        .arg(fixture.dir.join("bindings.json"))
        .arg("--infra")
        .arg(fixture.dir.join("observation.json"))
        .args(["--format", "json"]);
    let run = |label: &str, command: &mut Command| {
        std::fs::write(
            fixture.dir.join(format!("{label}.command")),
            format!("{command:?}\n"),
        )
        .unwrap();
        let start = std::time::Instant::now();
        command
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());
        let child = command.spawn().unwrap();
        let pid = child.id();
        let out = child.wait_with_output().unwrap();
        let elapsed = start.elapsed().as_secs_f64();
        std::fs::write(fixture.dir.join(format!("{label}.stdout")), &out.stdout).unwrap();
        std::fs::write(fixture.dir.join(format!("{label}.stderr")), &out.stderr).unwrap();
        write(
            &fixture.dir.join(format!("{label}.status.json")),
            &json!({"pid":pid,"exit":out.status.code(),"seconds":elapsed}),
        );
        out
    };
    let actual = run("discovery-offline", &mut command);
    assert!(actual.status.success(), "{actual:?}");
    assert_eq!(actual.stdout, baseline.stdout);
    write(
        &model.join("ess-inputs.yaml"),
        &json!({"format":"ess-inputs/1","specification":["absent"],"scenarios":[]}),
    );
    let sentinel = fixture.dir.join("refused-report.json");
    std::fs::write(&sentinel, "owned").unwrap();
    let mut live = ess();
    live.args(["verify", "bindings", "--spec"])
        .arg(&model)
        .arg("--realization")
        .arg(fixture.dir.join("realization.json"))
        .arg("--bindings")
        .arg(fixture.dir.join("bindings.json"))
        .args(["--live", "--format", "json", "--markdown-out"])
        .arg(&sentinel)
        .arg("--observation-out")
        .arg(fixture.dir.join("no-observation.json"))
        .env("PATH", fixture.dir.join("no-executables"));
    let refused = run("discovery-before-live", &mut live);
    assert!(!refused.status.success());
    let text = String::from_utf8_lossy(&refused.stdout);
    assert!(
        text.contains("ess-inputs.yaml") && text.contains("absent"),
        "{refused:?}"
    );
    assert!(!text.contains("starting kubectl"));
    assert_eq!(std::fs::read_to_string(sentinel).unwrap(), "owned");
}

#[test]
fn an_unbound_container_in_a_bound_workload_is_a_named_violation_until_it_is_bound() {
    let mut fixture = Fixture::new();
    let (output, report) = fixture.run();
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert_eq!(check(&report, "OBS-BIND-008"), "satisfied");
    let mut sidecar = fixture.observation["kinds"]["deployments"]["items"][0]["spec"]["template"]
        ["spec"]["containers"][0]
        .clone();
    sidecar["name"] = json!("sidecar");
    fixture.observation["kinds"]["deployments"]["items"][0]["spec"]["template"]["spec"]
        ["containers"]
        .as_array_mut()
        .unwrap()
        .push(sidecar);
    let (output, report) = fixture.run();
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    assert_eq!(report["status"], "violated");
    assert_eq!(check(&report, "OBS-BIND-008"), "violated");
    let detail = report["bindings"]["api"]["checks"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["code"] == "OBS-BIND-008")
        .unwrap()["detail"]
        .as_str()
        .unwrap()
        .to_owned();
    assert!(detail.contains("sidecar"), "{detail}");
    // The bound container's own checks are unchanged: the finding is the extra one, not a
    // reinterpretation of the declared one.
    assert_eq!(check(&report, "OBS-BIND-003"), "satisfied");
    assert_eq!(check(&report, "OBS-BIND-004"), "satisfied");
    let mut bound = fixture.bindings["bindings"][0].clone();
    bound["id"] = json!("sidecar");
    bound["container"] = json!("sidecar");
    fixture.bindings["bindings"]
        .as_array_mut()
        .unwrap()
        .push(bound);
    let (output, report) = fixture.run();
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert_eq!(check(&report, "OBS-BIND-008"), "satisfied");
}

/// A native sidecar (Kubernetes 1.29+) is an `initContainers` entry with `restartPolicy: Always`:
/// it runs for the pod's whole life, exactly like the unbound sidecar OBS-BIND-008 exists to catch.
/// The observation drops `initContainers`, so the check cannot see it — and it must then not claim
/// that "every container in the workload template is named by a binding".
#[test]
fn an_unbound_native_sidecar_is_not_reported_as_every_container_bound() {
    let mut fixture = Fixture::new();
    let image = fixture.bindings["bindings"][0]["image"].clone();
    fixture.observation["kinds"]["deployments"]["items"][0]["spec"]["template"]["spec"]
        ["initContainers"] = json!([{"name": "proxy", "image": image, "restartPolicy": "Always"}]);
    let (output, report) = fixture.run();
    let detail = report["bindings"]["api"]["checks"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["code"] == "OBS-BIND-008")
        .unwrap()["detail"]
        .clone();
    assert_ne!(
        check(&report, "OBS-BIND-008"),
        "satisfied",
        "an unbound native sidecar `proxy` passed OBS-BIND-008 (exit {:?}): {detail}",
        output.status.code()
    );
}

#[test]
fn a_native_sidecar_is_checked_like_a_container_and_a_plain_init_container_is_named_as_unchecked() {
    let mut fixture = Fixture::new();
    let image = fixture.bindings["bindings"][0]["image"].clone();
    fixture.observation["kinds"]["deployments"]["items"][0]["spec"]["template"]["spec"]
        ["initContainers"] = json!([
            {"name": "migrate", "image": image},
            {"name": "proxy", "image": image, "restartPolicy": "Always"}
    ]);
    let detail = |report: &Value| {
        report["bindings"]["api"]["checks"]
            .as_array()
            .unwrap()
            .iter()
            .find(|c| c["code"] == "OBS-BIND-008")
            .unwrap()["detail"]
            .as_str()
            .unwrap()
            .to_owned()
    };
    let (output, report) = fixture.run();
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    assert_eq!(check(&report, "OBS-BIND-008"), "violated");
    let text = detail(&report);
    assert!(
        text.contains("proxy") && !text.contains("migrate"),
        "{text}"
    );
    let mut bound = fixture.bindings["bindings"][0].clone();
    bound["id"] = json!("proxy");
    bound["container"] = json!("proxy");
    fixture.bindings["bindings"]
        .as_array_mut()
        .unwrap()
        .push(bound);
    let (output, report) = fixture.run();
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert_eq!(check(&report, "OBS-BIND-008"), "satisfied");
    // The satisfied detail says what it checked and what it did not.
    let text = detail(&report);
    assert!(!text.contains("every container"), "{text}");
    assert!(
        text.contains("native sidecar") && text.contains("init"),
        "{text}"
    );
}

#[test]
fn the_report_names_its_third_format_version() {
    // `/3` adds acknowledged foreign containers, so a satisfied `OBS-BIND-008` no longer means
    // every container is bound; a `/2` reader must not read it as `/2`.
    let fixture = Fixture::new();
    let (_, report) = fixture.run();
    assert_eq!(report["format"], "ess-observed-bindings-report/3");
    assert_eq!(report["bindings"]["api"]["acknowledged"], json!([]));
}

/// Unknown differs from false: a template that does not record `initContainers`, from a producer
/// older than the first release that collects them, cannot show that no native sidecar runs.
#[test]
fn an_observation_that_never_recorded_init_containers_leaves_obs_bind_008_unknown() {
    let mut fixture = Fixture::new();
    let template =
        &mut fixture.observation["kinds"]["deployments"]["items"][0]["spec"]["template"]["spec"];
    template.as_object_mut().unwrap().remove("initContainers");
    fixture.observation["scout_version"] = json!("0.31.0");
    let detail = |report: &Value| {
        report["bindings"]["api"]["checks"]
            .as_array()
            .unwrap()
            .iter()
            .find(|c| c["code"] == "OBS-BIND-008")
            .unwrap()["detail"]
            .as_str()
            .unwrap()
            .to_owned()
    };
    let (output, report) = fixture.run();
    assert_eq!(output.status.code(), Some(2), "{output:?}");
    assert_eq!(check(&report, "OBS-BIND-008"), "unknown");
    let text = detail(&report);
    assert!(text.contains("0.31.0") && text.contains("0.32.0"), "{text}");
    // A producer version that collects init containers makes an absent key mean none.
    fixture.observation["scout_version"] = json!("0.32.0");
    let (output, report) = fixture.run();
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert_eq!(check(&report, "OBS-BIND-008"), "satisfied");
    // A version this build cannot order is not evidence either.
    fixture.observation["scout_version"] = json!("synthetic");
    assert_eq!(check(&fixture.run().1, "OBS-BIND-008"), "unknown");
    // An unbound plain container is a violation whatever the producer recorded about init
    // containers: the evidence of it is in the containers list itself.
    let mut extra = fixture.observation["kinds"]["deployments"]["items"][0]["spec"]["template"]
        ["spec"]["containers"][0]
        .clone();
    extra["name"] = json!("sidecar");
    fixture.observation["kinds"]["deployments"]["items"][0]["spec"]["template"]["spec"]
        ["containers"]
        .as_array_mut()
        .unwrap()
        .push(extra);
    let (output, report) = fixture.run();
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    assert_eq!(check(&report, "OBS-BIND-008"), "violated");
}

fn detail(report: &Value, code: &str) -> String {
    report["bindings"]["api"]["checks"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["code"] == code)
        .unwrap()["detail"]
        .as_str()
        .unwrap()
        .to_owned()
}

/// Runs the web deployment's template with a third-party mesh proxy as a native sidecar and a
/// third-party agent as a plain container; neither is code this realization builds, and neither
/// runs an image a binding or implementation declares.
fn with_foreign_sidecars(fixture: &mut Fixture) {
    let template =
        &mut fixture.observation["kinds"]["deployments"]["items"][0]["spec"]["template"]["spec"];
    template["initContainers"] =
        json!([{"name": "mesh-proxy", "image": "mesh.example/proxy:9", "restartPolicy": "Always"}]);
    template["containers"]
        .as_array_mut()
        .unwrap()
        .push(json!({"name": "log-agent", "image": "vendor.example/log-agent:3"}));
}

fn acknowledge(fixture: &mut Fixture, containers: &Value) {
    fixture.bindings["format"] = json!("ess-observed-bindings/2");
    fixture.bindings["foreign_containers"] =
        json!([{"workload": {"kind": "deployment", "name": "web"}, "containers": containers}]);
}

#[test]
fn an_acknowledged_foreign_sidecar_satisfies_obs_bind_008_and_is_reported_acknowledged_not_bound() {
    let mut fixture = Fixture::new();
    with_foreign_sidecars(&mut fixture);
    acknowledge(
        &mut fixture,
        &json!([
            {"name": "mesh-proxy", "reason": "service-mesh proxy the platform injects"},
            {"name": "log-agent", "reason": "vendor log shipper"}
        ]),
    );
    let (output, report) = fixture.run();
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert_eq!(check(&report, "OBS-BIND-008"), "satisfied", "{report:#}");
    // Acknowledged, not bound: no binding result is minted for either, and each is listed with
    // its reason on the binding whose workload runs it.
    assert_eq!(report["bindings"].as_object().unwrap().len(), 1);
    assert_eq!(
        report["bindings"]["api"]["acknowledged"],
        json!([
            {"container": "log-agent", "reason": "vendor log shipper"},
            {"container": "mesh-proxy", "reason": "service-mesh proxy the platform injects"}
        ])
    );
    let text = detail(&report, "OBS-BIND-008");
    assert!(
        text.contains("acknowledged") && text.contains("mesh-proxy"),
        "{text}"
    );
}

#[test]
fn an_unacknowledged_sidecar_still_violates_obs_bind_008_beside_an_acknowledged_one() {
    let mut fixture = Fixture::new();
    with_foreign_sidecars(&mut fixture);
    acknowledge(
        &mut fixture,
        &json!([{"name": "mesh-proxy", "reason": "service-mesh proxy the platform injects"}]),
    );
    let (output, report) = fixture.run();
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    assert_eq!(check(&report, "OBS-BIND-008"), "violated");
    let text = detail(&report, "OBS-BIND-008");
    // The finding names what is unaccounted for, up to the next clause, and only that.
    let unbound = text.split_once("no binding names").map_or_else(
        || panic!("{text}"),
        |(_, rest)| rest.split(';').next().unwrap_or_default(),
    );
    assert!(
        unbound.contains("log-agent") && !unbound.contains("mesh-proxy"),
        "{text}"
    );
    assert_eq!(
        report["bindings"]["api"]["acknowledged"][0]["container"],
        "mesh-proxy"
    );
}

/// The infrastructure model records containers and native sidecars but never a plain init
/// container's name, so an acknowledgement naming neither may name a plain init container: it
/// cannot be shown stale, and it cannot satisfy either. It is `unknown`, named, and not listed as
/// acknowledged.
#[test]
fn a_stale_acknowledgement_is_named_and_never_satisfies_obs_bind_008() {
    let mut fixture = Fixture::new();
    acknowledge(
        &mut fixture,
        &json!([{"name": "retired-proxy", "reason": "proxy removed last quarter"}]),
    );
    let (output, report) = fixture.run();
    assert_eq!(output.status.code(), Some(2), "{output:?}");
    assert_eq!(check(&report, "OBS-BIND-008"), "unknown");
    let text = detail(&report, "OBS-BIND-008");
    assert!(
        text.contains("retired-proxy") && text.contains("plain init"),
        "{text}"
    );
    assert_eq!(report["bindings"]["api"]["acknowledged"], json!([]));
    // Over an observation that never recorded init containers, the name may also be an
    // unrecorded native sidecar: still not evidence of staleness either way.
    let template =
        &mut fixture.observation["kinds"]["deployments"]["items"][0]["spec"]["template"]["spec"];
    template.as_object_mut().unwrap().remove("initContainers");
    fixture.observation["scout_version"] = json!("0.31.0");
    let (output, report) = fixture.run();
    assert_eq!(output.status.code(), Some(2), "{output:?}");
    assert_eq!(check(&report, "OBS-BIND-008"), "unknown");
    assert!(
        detail(&report, "OBS-BIND-008").contains("retired-proxy"),
        "{report:#}"
    );
}

/// One edit that turns a valid acknowledgement document into one the contract must refuse.
type Mutation = fn(&mut Value);

#[test]
fn malformed_acknowledgements_are_refused_before_acquisition() {
    let mut fixture = Fixture::new();
    acknowledge(
        &mut fixture,
        &json!([{"name": "mesh-proxy", "reason": "service-mesh proxy the platform injects"}]),
    );
    let cases: [(&str, Mutation); 8] = [
        ("duplicates a binding", |v| {
            v["foreign_containers"][0]["containers"][0]["name"] = json!("web");
        }),
        ("empty reason", |v| {
            v["foreign_containers"][0]["containers"][0]["reason"] = json!("  ");
        }),
        ("invalid container name", |v| {
            v["foreign_containers"][0]["containers"][0]["name"] = json!("Mesh_Proxy");
        }),
        ("duplicate container", |v| {
            let again = v["foreign_containers"][0]["containers"][0].clone();
            v["foreign_containers"][0]["containers"]
                .as_array_mut()
                .unwrap()
                .push(again);
        }),
        ("duplicate workload", |v| {
            let again = v["foreign_containers"][0].clone();
            v["foreign_containers"].as_array_mut().unwrap().push(again);
        }),
        ("empty containers", |v| {
            v["foreign_containers"][0]["containers"] = json!([]);
        }),
        ("workload no binding binds", |v| {
            v["foreign_containers"][0]["workload"]["name"] = json!("worker");
        }),
        ("first format carrying acknowledgements", |v| {
            v["format"] = json!("ess-observed-bindings/1");
        }),
    ];
    for (label, mutate) in cases {
        let mut changed = fixture.bindings.clone();
        mutate(&mut changed);
        let mut command = fixture.command();
        write(&fixture.dir.join("bindings.json"), &changed);
        let output = command
            .args(["--infra", "/deliberately-missing-observation"])
            .output()
            .unwrap();
        let report: Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(output.status.code(), Some(1), "{label}: {report:#}");
        assert_eq!(
            report["checks"][0]["code"], "OBS-BIND-000",
            "{label}: {report:#}"
        );
        assert!(report["observation"].is_null(), "{label}");
    }
}

/// `ess-observed-bindings/1` is read unchanged: it acknowledges nothing, so an unbound sidecar
/// still violates, and its binding digest is the one ESS 0.32.1 computed for the same bytes.
#[test]
fn the_first_input_format_still_reads_unchanged_and_acknowledges_nothing() {
    let mut fixture = Fixture::new();
    let (output, report) = fixture.run();
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    assert_eq!(
        report["binding_digest"],
        "sha256:f35e7ee642fe6a4f424ffbdfd52cd1d3465af3b9cf158e10ef57ccaaf347bafd"
    );
    with_foreign_sidecars(&mut fixture);
    let (output, report) = fixture.run();
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    assert_eq!(check(&report, "OBS-BIND-008"), "violated");
    let text = detail(&report, "OBS-BIND-008");
    assert!(
        text.contains("mesh-proxy") && text.contains("log-agent"),
        "{text}"
    );
}

/// Adversary (wave 0021, ess-sidecar): an acknowledgement is for a container "this realization
/// does not build" (design, "Acknowledged foreign containers"). A container running the exact
/// image a binding declares is this realization's own code; acknowledging it as foreign must not
/// let `OBS-BIND-008` pass it unchecked.
#[test]
fn adversary_an_acknowledgement_cannot_hide_a_container_running_a_bound_image() {
    let mut fixture = Fixture::new();
    let image = fixture.bindings["bindings"][0]["image"].clone();
    fixture.observation["kinds"]["deployments"]["items"][0]["spec"]["template"]["spec"]
        ["containers"]
        .as_array_mut()
        .unwrap()
        .push(json!({"name": "worker", "image": image}));
    acknowledge(
        &mut fixture,
        &json!([{"name": "worker", "reason": "vendor agent"}]),
    );
    let (output, report) = fixture.run();
    assert_ne!(
        check(&report, "OBS-BIND-008"),
        "satisfied",
        "container `worker` runs the bound image {image} yet was accepted as foreign (exit {:?}): {}",
        output.status.code(),
        detail(&report, "OBS-BIND-008")
    );
}

/// Adversary (wave 0021, ess-sidecar): a plain init container is in the workload template, so an
/// acknowledgement naming it is not stale. The mesh pattern without native sidecars injects exactly
/// this pair. Coordinator's decision on what this asserts: the infrastructure model drops plain init
/// containers, so their names are never evidence either way, and the acknowledgement is `unknown`
/// (not a stale violation, and not satisfied) with a detail that says plain init containers are
/// not recorded.
#[test]
fn adversary_acknowledging_a_plain_init_container_is_not_a_stale_violation() {
    let mut fixture = Fixture::new();
    // A producer that records init containers, so the template's init list is evidence.
    fixture.observation["scout_version"] = json!("0.32.0");
    let template =
        &mut fixture.observation["kinds"]["deployments"]["items"][0]["spec"]["template"]["spec"];
    template["initContainers"] = json!([{"name": "mesh-init", "image": "mesh.example/init:9"}]);
    template["containers"]
        .as_array_mut()
        .unwrap()
        .push(json!({"name": "mesh-proxy", "image": "mesh.example/proxy:9"}));
    acknowledge(
        &mut fixture,
        &json!([
            {"name": "mesh-init", "reason": "mesh traffic-redirect init container"},
            {"name": "mesh-proxy", "reason": "service-mesh proxy the platform injects"}
        ]),
    );
    let (output, report) = fixture.run();
    let text = detail(&report, "OBS-BIND-008");
    assert_eq!(
        check(&report, "OBS-BIND-008"),
        "unknown",
        "a plain init container present in the template was reported stale (exit {:?}): {text}",
        output.status.code(),
    );
    assert_eq!(output.status.code(), Some(2), "{text}");
    assert!(
        text.contains("mesh-init") && text.contains("plain init containers"),
        "{text}"
    );
    // The plain container of the pair is present and is acknowledged.
    assert_eq!(
        report["bindings"]["api"]["acknowledged"],
        json!([{"container": "mesh-proxy", "reason": "service-mesh proxy the platform injects"}])
    );
}

/// Adversary (wave 0021, ess-sidecar): "A `/1` document carrying `foreign_containers` is refused."
/// Before `/2` existed the closed `/1` reader refused the key outright; an empty list is still the
/// key, and accepting it widens `/1`.
#[test]
fn adversary_a_first_format_document_carrying_an_empty_foreign_containers_is_refused() {
    let mut fixture = Fixture::new();
    fixture.bindings["foreign_containers"] = json!([]);
    let (output, report) = fixture.run();
    assert_eq!(output.status.code(), Some(1), "{report:#}");
    assert_eq!(report["checks"][0]["code"], "OBS-BIND-000", "{report:#}");
}

/// An acknowledgement is for code this realization does not build. A container running a
/// binding's declared image, or an implementation's artifact locator, is this realization's own
/// code under another name, so acknowledging it violates `OBS-BIND-008`, naming it.
#[test]
fn an_acknowledged_container_running_a_declared_image_or_artifact_locator_violates() {
    let mut fixture = Fixture::new();
    let locator = fixture.bindings["bindings"][0]["image"].clone();
    // The binding declares a tag, so the locator is only reachable through the realization.
    fixture.bindings["bindings"][0]["image"] = json!("registry.example/billing:1");
    let template =
        &mut fixture.observation["kinds"]["deployments"]["items"][0]["spec"]["template"]["spec"];
    template["containers"][0]["image"] = json!("registry.example/billing:1");
    template["containers"]
        .as_array_mut()
        .unwrap()
        .push(json!({"name": "worker", "image": locator}));
    acknowledge(
        &mut fixture,
        &json!([{"name": "worker", "reason": "vendor agent"}]),
    );
    let (output, report) = fixture.run();
    assert_eq!(output.status.code(), Some(1), "{report:#}");
    assert_eq!(check(&report, "OBS-BIND-008"), "violated");
    let text = detail(&report, "OBS-BIND-008");
    assert!(text.contains("worker"), "{text}");
    assert_eq!(report["bindings"]["api"]["acknowledged"], json!([]));
    // The same container under a binding's own declared image.
    fixture.observation["kinds"]["deployments"]["items"][0]["spec"]["template"]["spec"]
        ["containers"][1]["image"] = json!("registry.example/billing:1");
    let (output, report) = fixture.run();
    assert_eq!(output.status.code(), Some(1), "{report:#}");
    assert_eq!(check(&report, "OBS-BIND-008"), "violated");
    assert!(detail(&report, "OBS-BIND-008").contains("worker"));
}

/// A digest names the artifact; the repository in front of it is only where it was fetched from.
/// An acknowledged container whose `@sha256:` digest equals a binding's image digest or an
/// implementation's artifact locator digest is that artifact under another name, and violates.
/// Tags are not compared: the same tag under another repository is not evidence of the same bytes.
#[test]
fn an_acknowledged_container_running_a_declared_digest_under_another_name_violates() {
    let digest = format!("sha256:{}", "a".repeat(64));
    let run = |binding_image: &str, foreign_image: &str| {
        let mut fixture = Fixture::new();
        fixture.bindings["bindings"][0]["image"] = json!(binding_image);
        let template = &mut fixture.observation["kinds"]["deployments"]["items"][0]["spec"]
            ["template"]["spec"];
        template["containers"][0]["image"] = json!(binding_image);
        template["containers"]
            .as_array_mut()
            .unwrap()
            .push(json!({"name": "worker", "image": foreign_image}));
        acknowledge(
            &mut fixture,
            &json!([{"name": "worker", "reason": "vendor agent"}]),
        );
        fixture.run().1
    };
    // The binding's own digest-pinned image, mirrored under another repository and tag.
    let report = run(
        &format!("registry.example/billing@{digest}"),
        &format!("mirror.example/vendor/agent:7@{digest}"),
    );
    assert_eq!(check(&report, "OBS-BIND-008"), "violated", "{report:#}");
    assert!(detail(&report, "OBS-BIND-008").contains("worker"));
    assert_eq!(report["bindings"]["api"]["acknowledged"], json!([]));
    // The binding declares a tag, so only the implementation's artifact locator carries the digest.
    let report = run(
        "registry.example/billing:1",
        &format!("mirror.example/agent@{digest}"),
    );
    assert_eq!(check(&report, "OBS-BIND-008"), "violated", "{report:#}");
    // A different digest, and the same tag under another repository, are foreign.
    let report = run(
        "registry.example/billing:1",
        &format!("mirror.example/agent@sha256:{}", "b".repeat(64)),
    );
    assert_ne!(check(&report, "OBS-BIND-008"), "violated", "{report:#}");
    let report = run("registry.example/billing:1", "mirror.example/billing:1");
    assert_ne!(check(&report, "OBS-BIND-008"), "violated", "{report:#}");
    assert_eq!(
        report["bindings"]["api"]["acknowledged"][0]["container"],
        "worker"
    );
}

/// Adversary 2 (wave 0021, ess-sidecar): "a digest names the artifact" (design, "Acknowledged
/// foreign containers"). A container implementation's `identity` is that digest — `OBS-BIND-005`
/// compares a pinned template digest with it — and the realization accepts a tag-only locator
/// beside it. An acknowledged container running `…@<identity>` under another repository is then
/// the implementation's exact bytes, yet neither the locator nor the binding image pins it.
#[test]
fn adversary2_an_acknowledged_container_running_the_artifact_identity_digest_violates() {
    let mut fixture = Fixture::new();
    let digest = format!("sha256:{}", "a".repeat(64));
    let path = fixture.dir.join("realization.json");
    let mut realization: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    realization["implementations"][0]["artifact"] = json!({
        "kind": "container", "locator": "registry.example/billing:1", "identity": digest
    });
    write(&path, &realization);
    let compiled = ess()
        .args(["specify", "realization", "compile", "--path"])
        .arg(&path)
        .arg("--spec")
        .arg(root().join("examples/billing"))
        .args(["--format", "json"])
        .output()
        .unwrap();
    assert!(compiled.status.success(), "{compiled:?}");
    let ir: Value = serde_json::from_slice(&compiled.stdout).unwrap();
    fixture.bindings["realization_digest"] = ir["realization_digest"].clone();
    fixture.bindings["bindings"][0]["image"] = json!("registry.example/billing:1");
    let template =
        &mut fixture.observation["kinds"]["deployments"]["items"][0]["spec"]["template"]["spec"];
    template["containers"][0]["image"] = json!("registry.example/billing:1");
    template["containers"]
        .as_array_mut()
        .unwrap()
        .push(json!({"name": "worker", "image": format!("mirror.example/agent@{digest}")}));
    acknowledge(
        &mut fixture,
        &json!([{"name": "worker", "reason": "vendor agent"}]),
    );
    let (output, report) = fixture.run();
    assert_eq!(
        check(&report, "OBS-BIND-008"),
        "violated",
        "`worker` runs the implementation's artifact identity {digest} yet was accepted as \
         foreign (exit {:?}): {}; acknowledged {}",
        output.status.code(),
        detail(&report, "OBS-BIND-008"),
        report["bindings"]["api"]["acknowledged"]
    );
}

/// Adversary 2 (wave 0021, ess-sidecar): "a `/1` document carrying `foreign_containers` is
/// refused, even as `[]`" (CHANGELOG). `foreign_containers: null` — in YAML, the bare key
/// `foreign_containers:` — is the key carried, and the closed `/1` reader before `/2` refused it.
#[test]
fn adversary2_a_first_format_document_carrying_a_null_foreign_containers_is_refused() {
    let mut fixture = Fixture::new();
    fixture.bindings["foreign_containers"] = Value::Null;
    let (output, report) = fixture.run();
    assert_eq!(
        report["checks"][0]["code"],
        "OBS-BIND-000",
        "a /1 document carrying `foreign_containers: null` was read (exit {:?}): {report:#}",
        output.status.code()
    );
    assert_eq!(output.status.code(), Some(1), "{report:#}");
}
