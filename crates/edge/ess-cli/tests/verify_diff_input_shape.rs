//! A command input field turned from text into a record is breaking for callers, and
//! `ess verify diff --fail-on breaking` fails on it by name (`ess-diff/16`).
//!
//! The reported reproducer, run through the binary: before the fix the change was `unknown` for
//! callers and the breaking gate passed, exit 0. The fixtures are `ess-diff`'s own copies.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use serde_json::Value;

const RECEIPT: &str = "command/catalog.items.Accept/input-type-changed/receipt";

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../verify/ess-diff/tests/fixtures/upcast")
        .join(name)
}

fn diff(from: &str, to: &str, extra: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ess"))
        .args(["verify", "diff", "--from"])
        .arg(fixture(from))
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
fn a_text_input_turned_record_fails_the_breaking_gate_by_name() {
    let output = diff("before.yaml", "after.yaml", &["--fail-on", "breaking"]);
    assert_eq!(output.status.code(), Some(4), "{}", stderr(&output));
    let said = stderr(&output);
    assert!(
        said.contains(&format!("fails --fail-on breaking: {RECEIPT}")),
        "{said}"
    );
    let printed = String::from_utf8_lossy(&output.stdout);
    assert!(printed.contains("breaking for callers"), "{printed}");
}

#[test]
fn the_json_delta_is_ess_diff_16_and_names_the_shapes() {
    let output = diff(
        "before.yaml",
        "after.yaml",
        &["--compatibility", "--format", "json"],
    );
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    let delta: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(delta["format"], "ess-diff/16", "{delta:#}");
    let receipt = delta["changes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|change| change["id"] == RECEIPT)
        .unwrap_or_else(|| panic!("`{RECEIPT}` in {delta:#}"));
    assert_eq!(
        receipt["compatibility"]["callers"], "breaking",
        "{receipt:#}"
    );
    assert_eq!(
        receipt["compatibility"]["shapes"],
        serde_json::json!({"before": "scalar", "after": "record"}),
        "{receipt:#}"
    );
}

#[test]
fn the_additive_idiom_passes_the_breaking_gate() {
    let output = diff(
        "before.yaml",
        "after-additive.yaml",
        &["--fail-on", "breaking"],
    );
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
}
