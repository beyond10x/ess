//! Persisted plans must be checked before analysis and before either external executor.
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::OnceLock;

struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "ess-delivery-test-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ));
        std::fs::create_dir(&path).unwrap();
        Self(path)
    }
    fn write(&self, name: &str, value: &serde_json::Value) -> PathBuf {
        let path = self.0.join(name);
        let text = if name.ends_with("yaml") {
            serde_yaml::to_string(value).unwrap()
        } else {
            serde_json::to_string(value).unwrap()
        };
        std::fs::write(&path, text).unwrap();
        path
    }
    fn command(&self) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_ess"));
        command
            .env("PATH", executors())
            .env("ESS_TEST_DELIVERY_LOG", self.0.join("calls"));
        command
    }
    fn reconcile(&self, desired: &Path, current: Option<&Path>, dry_run: bool) -> Output {
        let mut command = self.command();
        command
            .args(["deployment", "reconcile", "--path"])
            .arg(desired)
            .arg("--cache")
            .arg(self.0.join("cache"));
        if let Some(current) = current {
            command
                .arg("--current")
                .arg(current)
                .arg("--allow-removals");
        }
        if dry_run {
            command.arg("--dry-run");
        }
        command.output().unwrap()
    }
    fn assert_no_calls(&self) {
        assert!(
            !self.0.join("calls").exists(),
            "executors ran: {}",
            std::fs::read_to_string(self.0.join("calls")).unwrap_or_default()
        );
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.0).unwrap();
    }
}

fn executors() -> &'static Path {
    static EXECUTORS: OnceLock<PathBuf> = OnceLock::new();
    EXECUTORS
        .get_or_init(|| {
            let root = Fixture::new();
            let output = Command::new("rustc")
                .arg("--edition=2021")
                .arg(Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/support/fake_delivery.rs"))
                .arg("-o")
                .arg(root.0.join("oras"))
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
            std::fs::copy(root.0.join("oras"), root.0.join("helm")).unwrap();
            let path = root.0.clone();
            // Shared process fixtures remain under TMPDIR for the duration of this test binary.
            std::mem::forget(root);
            path
        })
        .as_path()
}

fn plan() -> serde_json::Value {
    let digest = format!("sha256:{}", "1".repeat(64));
    serde_json::json!({
        "format": "ess-deployment/1", "environment": "test", "stack_digest": digest,
        "cluster": "test-cluster", "rollout_order": ["first", "last"],
        "releases": {
            "first": {"service":"first", "release_name":"first", "namespace":"test", "service_account":"default",
                "chart":{"build_output":"chart", "kind":"helm_chart", "reference":"oci://example.invalid/chart", "digest":digest}, "images":{"app":{"build_output":"app", "kind":"oci_image", "reference":"example.invalid/app", "digest":digest, "platforms":{"linux/amd64":digest}}}},
            "last": {"service":"last", "release_name":"last", "namespace":"test", "service_account":"default",
                "chart":{"build_output":"chart", "kind":"helm_chart", "reference":"oci://example.invalid/chart", "digest":digest}, "images":{"app":{"build_output":"app", "kind":"oci_image", "reference":"example.invalid/app", "digest":digest, "platforms":{"linux/amd64":digest}}}}
        }
    })
}

#[test]
fn entire_desired_plan_is_refused_before_oras_or_helm() {
    for extension in ["json", "yaml"] {
        for (pointer, value) in [
            ("/releases/last/chart/kind", serde_json::json!("binary")),
            ("/format", serde_json::json!("future/99")),
            ("/rollout_order", serde_json::json!(["first"])),
            ("/rollout_order", serde_json::json!(["first", "missing"])),
            ("/releases/last/service", serde_json::json!("other")),
        ] {
            let fixture = Fixture::new();
            let mut invalid = plan();
            *invalid.pointer_mut(pointer).unwrap() = value;
            let desired = fixture.write(&format!("desired.{extension}"), &invalid);
            let output = fixture.reconcile(&desired, None, false);
            assert!(
                !output.status.success(),
                "admitted {pointer}: {}",
                String::from_utf8_lossy(&output.stdout)
            );
            fixture.assert_no_calls();
        }
    }
}

#[test]
fn invalid_current_removal_is_refused_before_analysis_and_execution() {
    for dry_run in [true, false] {
        let fixture = Fixture::new();
        let mut desired = plan();
        desired["releases"] = serde_json::json!({});
        desired["rollout_order"] = serde_json::json!([]);
        let desired = fixture.write("desired.json", &desired);
        let mut current = plan();
        current["releases"]["last"]["chart"]["kind"] = serde_json::json!("binary");
        let current = fixture.write("current.yaml", &current);
        let output = fixture.reconcile(&desired, Some(&current), dry_run);
        assert!(
            !output.status.success(),
            "invalid current state was admitted"
        );
        fixture.assert_no_calls();
        let output = fixture
            .command()
            .args(["deployment", "diff", "--from"])
            .arg(&current)
            .arg("--to")
            .arg(&desired)
            .output()
            .unwrap();
        assert!(!output.status.success(), "invalid diff input was admitted");
        fixture.assert_no_calls();
    }
}

#[test]
fn valid_plan_reaches_both_local_fake_executors_in_rollout_order() {
    let fixture = Fixture::new();
    let desired = fixture.write("desired.json", &plan());
    let output = fixture.reconcile(&desired, None, false);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        std::fs::read_to_string(fixture.0.join("calls")).unwrap(),
        "oras\nhelm\nhelm\n"
    );
}
