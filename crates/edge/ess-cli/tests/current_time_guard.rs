//! `ess verify conform run` over a suite carrying `now_offset` values (beyond10x/ess#171,
//! `docs/design/current-time-guards.md`).
//!
//! The suite is synthesized from the #171 repro through the CLI's own path, admitted as
//! `ess-conformance/26`, and run. No built-in target implements the jobs model, so what is
//! observable here is the run itself: every `now_offset` resolves against the wall clock the CLI
//! supplies before the command reaches the target — which then answers that it does not know the
//! command — and nothing in the run is about the value.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::Command;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

/// The #171 repro as a model directory, one per test so parallel tests do not share it.
fn model(name: &str) -> PathBuf {
    let directory = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    // A leftover file from an earlier layout would be read as a second declaration.
    if directory.exists() {
        std::fs::remove_dir_all(&directory).expect("a stale model directory is removed");
    }
    std::fs::create_dir_all(directory.join("domains")).expect("a model directory");
    let repro = std::fs::read_to_string(
        root().join("crates/verify/ess-conformance/tests/fixtures/current-time-guard.yaml"),
    )
    .expect("the #171 repro");
    // The single-file fixture opens with the system header; a model directory splits it off, as
    // the issue's repro does.
    let domain = repro
        .strip_prefix("format: ess/16\nsystem: demo\nversion: v1\n")
        .expect("the fixture opens with its system header");
    std::fs::write(
        directory.join("system.yaml"),
        "format: ess/16\nsystem: demo\nversion: v1\nsummary: Minimal repro.\ndomains: [demo.jobs]\n",
    )
    .expect("system.yaml");
    std::fs::write(directory.join("domains/jobs.yaml"), domain).expect("the domain");
    directory
}

/// Runs the synthesized suite against `target` with report/2, which suites from /8 on require.
fn run(target: &str) -> (Option<i32>, serde_json::Value, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_ess"))
        .current_dir(root())
        .args(["verify", "conform", "run", "--target", target])
        .args(["--report-format", "2", "--format", "json", "--path"])
        .arg(model(&format!("current-time-guard-{target}")))
        .output()
        .expect("the `ess` binary runs");
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    let report = serde_json::from_slice(&output.stdout).unwrap_or_else(|error| {
        panic!(
            "{target} renders a JSON report: {error}\nstdout: {}\nstderr: {stderr}",
            String::from_utf8_lossy(&output.stdout)
        )
    });
    (output.status.code(), report, stderr)
}

fn scenarios(report: &serde_json::Value) -> &Vec<serde_json::Value> {
    let scenarios = report["scenarios"].as_array().expect("scenarios");
    assert_eq!(
        scenarios.len(),
        2,
        "one scenario per branch of #171: {report:#}"
    );
    scenarios
}

/// A target that executes steps: each `now_offset` is resolved before the command is handed over.
/// A value the runner could not resolve would be reported about the value; here every check is the
/// billing target refusing a command it does not implement, so the command was sent.
#[test]
fn issue_171_a_now_offset_suite_runs_through_the_cli_with_every_value_resolved() {
    let (code, report, stderr) = run("billing");
    assert_eq!(
        report["summary"]["suite"]["version"], "ess-conformance/26",
        "{report:#}"
    );
    for scenario in scenarios(&report) {
        for check in scenario["checks"].as_array().expect("checks") {
            let observed = check["diagnostic"]["observed"].to_string();
            assert!(
                observed.contains("failed: this implementation accepts only the commands"),
                "the command reached the target: {check:#}"
            );
            assert!(
                !check.to_string().contains("now_offset"),
                "nothing in the run is about the value: {check:#}"
            );
        }
    }
    assert_eq!(
        code,
        Some(3),
        "the target could not carry the command out: {stderr}"
    );
}

#[test]
fn issue_171_a_now_offset_suite_is_admitted_against_the_interpreter() {
    let (code, report, stderr) = run("interpreted");
    assert_eq!(code, Some(1), "{stderr}");
    let statuses: BTreeSet<&str> = scenarios(&report)
        .iter()
        .map(|scenario| scenario["status"].as_str().expect("a status"))
        .collect();
    assert_eq!(statuses, BTreeSet::from(["unsupported"]), "{report:#}");
}
