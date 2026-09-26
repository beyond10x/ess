//! Adversary pass 2 on M8: documents a released ESS wrote are read by this build without a
//! version that says whether `initContainers` were ever recorded.
//!
//! `fixtures/observed-bindings-m8-pass2/` holds what ESS 0.31.0 (this unit's base, 3e0ed2c5)
//! wrote for one namespace whose deployment runs an unbound native sidecar `proxy`
//! (`kubectl-deployments.json`): an `infra-observation/2` from `ess-kubernetes scan --namespace`
//! and an `infra-ir/2` from `ess infra import kubernetes --namespace --out`. 0.31.0 dropped
//! `initContainers`, and both documents keep their format label in this build, so this build
//! cannot tell "no native sidecar" from "never looked".
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};
#[allow(dead_code)]
#[path = "support/executable.rs"]
mod executable;

static NEXT: AtomicU64 = AtomicU64::new(0);

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}
fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/observed-bindings-m8-pass2")
}
fn ess() -> Command {
    Command::new(env!("CARGO_BIN_EXE_ess"))
}
fn write(path: &Path, value: &Value) {
    std::fs::write(path, serde_json::to_vec_pretty(value).unwrap()).unwrap();
}

const KINDS: &[&str] = &[
    "namespaces",
    "nodes",
    "deployments",
    "statefulsets",
    "daemonsets",
    "replicasets",
    "jobs",
    "cronjobs",
    "pods",
    "services",
    "ingresses",
    "configmaps",
    "secrets",
    "serviceaccounts",
    "persistentvolumeclaims",
    "poddisruptionbudgets",
    "horizontalpodautoscalers",
];

struct Case {
    dir: PathBuf,
}

impl Case {
    fn new() -> Self {
        let dir = std::env::temp_dir().join(format!(
            "ess-m8-adv2-{}-{}",
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
        write(
            &dir.join("bindings.json"),
            &json!({
                "format":"ess-observed-bindings/1", "id":"billing-qa",
                "realization_digest":ir["realization_digest"],
                "scope":{"context":"synthetic-context", "namespace":"app"},
                "bindings":[{"id":"api", "implementation":"billing-binary",
                    "workload":{"kind":"deployment", "name":"web"}, "container":"web", "image":image}]
            }),
        );
        // The kubectl responses the 0.31.0 documents were produced from.
        let responses = dir.join("responses");
        std::fs::create_dir(&responses).unwrap();
        for kind in KINDS {
            std::fs::write(responses.join(format!("{kind}.json")), r#"{"items":[]}"#).unwrap();
        }
        std::fs::copy(
            fixtures().join("kubectl-deployments.json"),
            responses.join("deployments.json"),
        )
        .unwrap();
        std::fs::write(
            responses.join("namespace.json"),
            r#"{"metadata":{"name":"app","uid":"ns-1"}}"#,
        )
        .unwrap();
        std::fs::write(
            responses.join("node.json"),
            r#"{"metadata":{"name":"node-a","uid":"node-1"}}"#,
        )
        .unwrap();
        std::fs::write(
            responses.join("pods.json"),
            r#"{"items":[{"metadata":{"name":"web-1","namespace":"app","uid":"web-1"},"spec":{"nodeName":"node-a"}}]}"#,
        )
        .unwrap();
        Self { dir }
    }

    fn verify(&self) -> Command {
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

    fn report(output: &Output) -> Value {
        serde_json::from_slice(&output.stdout).expect("exactly one JSON report")
    }

    /// This build's own live read of the same kubectl responses, through the fake kubectl the
    /// `ess-kubernetes` suite uses.
    fn live(&self) -> Value {
        let bin = self.dir.join("bin");
        std::fs::create_dir(&bin).unwrap();
        let kubectl = bin.join("kubectl");
        let compiled = Command::new("rustc")
            .args(["--edition=2021", "-Dwarnings"])
            .arg(root().join("crates/infra/ess-kubernetes/tests/fixtures/fake_command.rs"))
            .arg("-o")
            .arg(&kubectl)
            .output()
            .unwrap();
        assert!(compiled.status.success(), "{compiled:?}");
        executable::install_copy(&kubectl, &bin.join("date")).unwrap();
        let output = self
            .verify()
            .args(["--live", "--observation-out"])
            .arg(self.dir.join("live-observation.json"))
            .env("PATH", &bin)
            .env("ESS_TEST_TOPOLOGY_DIRECTORY", self.dir.join("responses"))
            .output()
            .unwrap();
        Self::report(&output)
    }

    fn supplied(&self, fixture: &str) -> Value {
        let output = self
            .verify()
            .arg("--infra")
            .arg(fixtures().join(fixture))
            .output()
            .unwrap();
        Self::report(&output)
    }
}

impl Drop for Case {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

fn check(report: &Value, code: &str) -> (Value, Value) {
    let found = report["bindings"]["api"]["checks"]
        .as_array()
        .unwrap_or_else(|| panic!("no binding checks in {report:#}"))
        .iter()
        .find(|c| c["code"] == code)
        .unwrap_or_else(|| panic!("no {code} in {report:#}"))
        .clone();
    (found["status"].clone(), found["detail"].clone())
}

/// The same namespace, read live by this build, violates OBS-BIND-008 (control). The documents
/// ESS 0.31.0 wrote for that namespace must not be read by this build as having been checked for
/// native sidecars: "each container and native sidecar ... is named by a binding" is a claim the
/// 0.31.0 producer never made, so OBS-BIND-008 over them is at most `unknown`.
#[test]
fn a_released_observation_that_never_recorded_init_containers_does_not_satisfy_obs_bind_008() {
    let case = Case::new();
    let live = case.live();
    let (status, detail) = check(&live, "OBS-BIND-008");
    assert_eq!(
        status, "violated",
        "control: the live read sees `proxy`: {detail}"
    );
    for fixture in [
        "observation-2-from-0.31.0.json",
        "infra-ir-2-from-0.31.0.json",
    ] {
        let report = case.supplied(fixture);
        let (status, detail) = check(&report, "OBS-BIND-008");
        assert_ne!(
            status, "satisfied",
            "{fixture} (written by ESS 0.31.0, which dropped initContainers) satisfies \
             OBS-BIND-008 for a namespace whose live read violates it; report status {}: {detail}",
            report["status"]
        );
    }
}
