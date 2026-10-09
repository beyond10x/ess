//! `ess verify conform run --target interpreted` over a command with a current-time (`now`) guard
//! (<https://github.com/beyond10x/ess/issues/510>).
//!
//! The CLI built the interpreted target without a command clock, so every decision that reached a
//! `now` guard was Unknown and every scenario of such a command came back `unsupported`, authored
//! ones included. The target now decides at the instant of the step the runner is executing: the
//! instant that step's `now_offset` values resolve against. The CLI's interpreted run reads no
//! clock of the machine's, so two runs print the same bytes.
//!
//! Each case here fails again if the CLI builds the interpreted target without the clock.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

/// The reproducer from the issue, with neutral names, and the suite the CLI synthesizes from it,
/// in a directory of its own per test.
fn workspace(name: &str) -> (PathBuf, PathBuf) {
    let directory = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    if directory.exists() {
        std::fs::remove_dir_all(&directory).expect("a stale directory is removed");
    }
    std::fs::create_dir_all(&directory).expect("a directory");
    let model = directory.join("system.yaml");
    std::fs::copy(
        root().join("crates/verify/ess-conformance/tests/fixtures/now-guard-interpreted.yaml"),
        &model,
    )
    .expect("the reproducer");
    let suite = directory.join("suite.json");
    let output = Command::new(env!("CARGO_BIN_EXE_ess"))
        .args(["verify", "conform", "synthesize", "--path"])
        .arg(&model)
        .arg("--out")
        .arg(&suite)
        .output()
        .expect("the `ess` binary runs");
    assert!(
        output.status.success(),
        "synthesis succeeds: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    (model, suite)
}

fn run(model: &Path, suite: &Path, format: &str) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ess"))
        .args(["verify", "conform", "run", "--target", "interpreted"])
        .args(["--report-format", "2", "--format", format, "--path"])
        .arg(model)
        .arg("--suite")
        .arg(suite)
        .output()
        .expect("the `ess` binary runs")
}

#[test]
fn issue_510_the_reproducer_passes_every_scenario_against_the_interpreted_target() {
    let (model, suite) = workspace("issue-510-json");
    let output = run(&model, &suite, "json");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let report: serde_json::Value = serde_json::from_slice(&output.stdout)
        .unwrap_or_else(|error| panic!("a JSON report: {error}\n{stdout}"));
    let scenarios = report["scenarios"].as_array().expect("scenarios");
    assert_eq!(scenarios.len(), 2, "one scenario per branch: {report:#}");
    for scenario in scenarios {
        assert_eq!(
            scenario["status"], "passed",
            "the guard is decided at the step's instant: {scenario:#}"
        );
    }
    assert_eq!(report["summary"]["counts"]["passed"], 2, "{report:#}");
    assert_eq!(report["summary"]["counts"]["unsupported"], 0, "{report:#}");
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn issue_510_the_text_report_passes_too() {
    let (model, suite) = workspace("issue-510-text");
    let output = run(&model, &suite, "text");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("2 scenarios: 2 passed, 0 failed, 0 error, 0 unsupported"),
        "{stdout}"
    );
    assert_eq!(output.status.code(), Some(0), "{stdout}");
}

/// No clock of the machine's decides: the instant is the runner's, so a second run prints the
/// first run's bytes, in both formats.
#[test]
fn issue_510_two_runs_print_the_same_report_bytes() {
    let (model, suite) = workspace("issue-510-twice");
    for format in ["json", "text"] {
        let first = run(&model, &suite, format);
        let second = run(&model, &suite, format);
        assert_eq!(first.status.code(), Some(0), "{format}");
        assert_eq!(
            String::from_utf8_lossy(&first.stdout),
            String::from_utf8_lossy(&second.stdout),
            "{format}"
        );
    }
}
