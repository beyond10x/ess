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
