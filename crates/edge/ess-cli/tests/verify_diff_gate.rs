//! beyond10x/ess#290: `ess verify diff` can fail on a breaking change, and only when asked to.
//!
//! Run against the committed revision pair, whose delta holds three breaking changes (a currency
//! variant added to a type the system writes, one removed from a type callers send and the system
//! stores, a grant removed), two unknown ones (predicate rewrites) and one compatible one (a grant
//! added).

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use serde_json::Value;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

fn diff(extra: &[&str]) -> Output {
    let root = root();
    Command::new(env!("CARGO_BIN_EXE_ess"))
        .args(["verify", "diff", "--from"])
        .arg(root.join("examples/revision-pair/before"))
        .arg("--to")
        .arg(root.join("examples/revision-pair/after"))
        .args(extra)
        .output()
        .unwrap()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

const BREAKING: [&str; 3] = [
    "type/catalog.pricing.Currency/variant-added/CHF",
    "type/catalog.pricing.Currency/variant-removed/GBP",
    "actor/catalog.pricing.Auditor/grant-removed/catalog.pricing.RetirePriceList",
];

fn scratch(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("verify-diff-gate-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    dir.join(name)
}

/// An acknowledgements file for the pair, naming `ids`; `swap` writes the digests the wrong way.
fn acknowledgements(name: &str, ids: &[&str], swap: bool) -> PathBuf {
    let classified = diff(&["--compatibility", "--format", "json"]);
    let delta: Value = serde_json::from_slice(&classified.stdout).unwrap();
    let (before, after) = (
        delta["before"]["spec_digest"].clone(),
        delta["after"]["spec_digest"].clone(),
    );
    let (before, after) = if swap {
        (after, before)
    } else {
        (before, after)
    };
    let path = scratch(name);
    std::fs::write(
        &path,
        serde_json::json!({
            "format": "ess-diff-acknowledgements/1",
            "before": before,
            "after": after,
            "acknowledged": ids,
        })
        .to_string(),
    )
    .unwrap();
    path
}

#[test]
fn without_the_new_options_the_command_prints_and_exits_as_before() {
    let text = diff(&[]);
    assert_eq!(text.status.code(), Some(0), "{}", stderr(&text));
    let printed = String::from_utf8(text.stdout).unwrap();
    assert!(
        printed.contains("2 widening, 2 narrowing, 2 other"),
        "{printed}"
    );
    assert!(!printed.contains("compatib"), "{printed}");
    assert!(!printed.contains("breaking"), "{printed}");

    let json = diff(&["--format", "json"]);
    assert_eq!(json.status.code(), Some(0));
    let delta: Value = serde_json::from_slice(&json.stdout).unwrap();
    assert_eq!(delta["format"], "ess-diff/2");
    assert!(delta["changes"]
        .as_array()
        .unwrap()
        .iter()
        .all(|change| change.get("compatibility").is_none()));
}

#[test]
fn compatibility_classifies_every_change_in_ess_diff_14() {
    let json = diff(&["--compatibility", "--format", "json"]);
    assert_eq!(json.status.code(), Some(0), "{}", stderr(&json));
    let delta: Value = serde_json::from_slice(&json.stdout).unwrap();
    assert_eq!(delta["format"], "ess-diff/14");
    let mut breaking: Vec<String> = Vec::new();
    for change in delta["changes"].as_array().unwrap() {
        let verdict = change["compatibility"]["verdict"].as_str().unwrap();
        if verdict == "breaking" {
            breaking.push(change["id"].as_str().unwrap().to_owned());
        }
    }
    breaking.sort();
    let mut expected: Vec<String> = BREAKING.iter().map(|id| (*id).to_owned()).collect();
    expected.sort();
    assert_eq!(breaking, expected);

    let text = diff(&["--compatibility"]);
    let printed = String::from_utf8(text.stdout).unwrap();
    assert!(
        printed.contains("compatibility: 3 breaking, 2 unknown, 1 compatible"),
        "{printed}"
    );
    assert!(
        printed.contains("breaking for callers, history"),
        "{printed}"
    );
}

#[test]
fn fail_on_breaking_exits_4_and_names_each_unacknowledged_change() {
    let output = diff(&["--fail-on", "breaking"]);
    assert_eq!(output.status.code(), Some(4), "{}", stderr(&output));
    let said = stderr(&output);
    for id in BREAKING {
        assert!(
            said.contains(&format!("fails --fail-on breaking: {id}")),
            "{said}"
        );
    }
    assert!(!said.contains("invariants-changed"), "{said}");
}

#[test]
fn acknowledging_every_breaking_change_passes_and_unknowns_still_fail_the_stricter_gate() {
    let file = acknowledgements("all-breaking.json", &BREAKING, false);
    let passed = diff(&[
        "--fail-on",
        "breaking",
        "--acknowledgements",
        file.to_str().unwrap(),
    ]);
    assert_eq!(passed.status.code(), Some(0), "{}", stderr(&passed));
    assert!(stderr(&passed).contains("passed (3 acknowledged)"));

    let strict = diff(&[
        "--fail-on",
        "breaking-or-unknown",
        "--acknowledgements",
        file.to_str().unwrap(),
    ]);
    assert_eq!(strict.status.code(), Some(4), "{}", stderr(&strict));
    assert!(stderr(&strict).contains("2 unacknowledged change(s) fail"));
}

#[test]
fn a_dimension_narrows_what_the_gate_considers() {
    let output = diff(&["--fail-on", "breaking", "--dimension", "history"]);
    assert_eq!(output.status.code(), Some(4), "{}", stderr(&output));
    let said = stderr(&output);
    assert!(said.contains("1 unacknowledged change(s) fail"), "{said}");
    assert!(
        said.contains(
            "fails --fail-on breaking: type/catalog.pricing.Currency/variant-removed/GBP"
        ),
        "{said}"
    );
}

#[test]
fn acknowledgements_for_another_pair_are_refused_with_status_1() {
    let file = acknowledgements("swapped.json", &BREAKING, true);
    let output = diff(&[
        "--fail-on",
        "breaking",
        "--acknowledgements",
        file.to_str().unwrap(),
    ]);
    assert_eq!(output.status.code(), Some(1), "{}", stderr(&output));
    assert!(output.stdout.is_empty());
    assert!(
        stderr(&output).contains("conflicting_declaration"),
        "{}",
        stderr(&output)
    );
}

#[test]
fn acknowledgements_without_fail_on_is_a_usage_error() {
    let file = acknowledgements("unused.json", &BREAKING, false);
    let output = diff(&["--acknowledgements", file.to_str().unwrap()]);
    assert_eq!(output.status.code(), Some(2), "{}", stderr(&output));
}
