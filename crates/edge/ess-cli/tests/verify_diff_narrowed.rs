//! A refusal added, a required input added and a refusal guard widened are breaking for callers,
//! and `ess verify diff --fail-on breaking` fails on each by name (`ess-diff/17`,
//! <https://github.com/beyond10x/ess/issues/514>).
//!
//! The reported revisions, run through the binary: before the fix each was `unknown` for callers
//! and the breaking gate passed, exit 0. The fixtures are `ess-diff`'s own copies.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use serde_json::Value;

const REPORTED: [(&str, &str, &str); 3] = [
    (
        "added-refusal.yaml",
        "command/demo.tok.Present/outcome-added/admin-scope",
        "refusal-added",
    ),
    (
        "required-input.yaml",
        "command/demo.tok.Present/input-added/audience",
        "required-input",
    ),
    (
        "widened-guard.yaml",
        "command/demo.tok.Present/outcome-condition-changed/empty-scope",
        "refusal-widened",
    ),
];

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../verify/ess-diff/tests/fixtures/narrowed")
        .join(name)
}

fn diff(to: &str, extra: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ess"))
        .args(["verify", "diff", "--from"])
        .arg(fixture("v1.yaml"))
        .arg("--to")
        .arg(fixture(to))
        .args(extra)
        .output()
        .unwrap()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

#[test]
fn each_reported_revision_fails_the_breaking_gate_by_name() {
    for (to, id, _) in REPORTED {
        let output = diff(to, &["--fail-on", "breaking"]);
        let said = stderr(&output);
        assert_eq!(output.status.code(), Some(4), "{to}: {said}");
        assert!(
            said.contains(&format!("fails --fail-on breaking: {id}")),
            "{to}: {said}"
        );
        let printed = String::from_utf8_lossy(&output.stdout);
        assert!(printed.contains("breaking for callers"), "{to}: {printed}");
    }
}

#[test]
fn the_json_delta_is_ess_diff_17_and_names_the_narrowing() {
    for (to, id, narrows) in REPORTED {
        let output = diff(to, &["--compatibility", "--format", "json"]);
        assert_eq!(output.status.code(), Some(0), "{to}: {}", stderr(&output));
        let delta: Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(delta["format"], "ess-diff/17", "{delta:#}");
        let change = delta["changes"]
            .as_array()
            .unwrap()
            .iter()
            .find(|change| change["id"] == id)
            .unwrap_or_else(|| panic!("`{id}` in {delta:#}"));
        assert_eq!(change["compatibility"]["callers"], "breaking", "{change:#}");
        assert_eq!(change["compatibility"]["narrows"], narrows, "{change:#}");
    }
}
