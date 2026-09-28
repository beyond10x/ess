//! Adversarial cases for `ess verify conform check-history`'s exit status and JSON report.
//!
//! The subcommand's help promises: exit 0 linearizable, 1 violation, 3 unknown, and exit 2 where
//! "the specification did not load, or the history or one of its operations was refused". A caller
//! (a CI step, the Go and TypeScript explorers of `story:concurrent-explorer-runner`) reads the
//! verdict from the status alone, so a status that means two things is a verdict nobody can read.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

fn ess(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ess"))
        .current_dir(root())
        .args(args)
        .output()
        .expect("the `ess` binary runs")
}

const REGISTER: &str = "crates/verify/ess-conformance/tests/fixtures/register/register.yaml";
const NOT_LINEARIZABLE: &str =
    "crates/verify/ess-conformance/tests/fixtures/register/not-linearizable.json";

fn explained(output: &Output) -> String {
    format!(
        "status {:?}\nstdout:\n{}\nstderr:\n{}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

#[test]
fn a_history_file_that_does_not_exist_is_refused_with_exit_2_not_reported_as_a_violation() {
    let output = ess(&[
        "verify",
        "conform",
        "check-history",
        "--path",
        REGISTER,
        "--history",
        "crates/verify/ess-conformance/tests/fixtures/register/no-such-history.json",
    ]);
    assert_eq!(output.status.code(), Some(2), "{}", explained(&output));
}

#[test]
fn a_specification_path_that_does_not_exist_is_refused_with_exit_2_not_reported_as_a_violation() {
    let output = ess(&[
        "verify",
        "conform",
        "check-history",
        "--path",
        "crates/verify/ess-conformance/tests/fixtures/register/no-such-model.yaml",
        "--history",
        NOT_LINEARIZABLE,
    ]);
    assert_eq!(output.status.code(), Some(2), "{}", explained(&output));
}

#[test]
fn the_shrunk_history_a_json_report_prints_is_itself_a_violation_on_the_command_line() {
    let first = ess(&[
        "verify",
        "conform",
        "check-history",
        "--path",
        REGISTER,
        "--history",
        NOT_LINEARIZABLE,
        "--format",
        "json",
    ]);
    assert_eq!(first.status.code(), Some(1), "{}", explained(&first));
    let report: serde_json::Value =
        serde_json::from_slice(&first.stdout).expect("the report is JSON");
    let keys: Vec<&str> = report
        .as_object()
        .expect("an object")
        .keys()
        .map(String::as_str)
        .collect();
    let mut expected = vec![
        "budget",
        "linearization",
        "not_judged",
        "operations",
        "partitions",
        "shrunk",
        "steps",
        "subject_key",
        "verdict",
    ];
    expected.sort_unstable();
    let mut sorted = keys.clone();
    sorted.sort_unstable();
    assert_eq!(sorted, expected, "the report's fields: {keys:?}");

    let directory = tempfile::tempdir().expect("a scratch directory");
    let shrunk = directory.path().join("shrunk.json");
    std::fs::write(
        &shrunk,
        serde_json::to_vec(&report["shrunk"]).expect("serializes"),
    )
    .expect("written");
    let again = ess(&[
        "verify",
        "conform",
        "check-history",
        "--path",
        REGISTER,
        "--history",
        shrunk.to_str().expect("a UTF-8 path"),
    ]);
    assert_eq!(again.status.code(), Some(1), "{}", explained(&again));
}
