//! The three formats the zero-open-issues epic fills are open and empty: source `ess/37`, the suite
//! pair `ess-conformance/64` (ordinary) and `/65` (coverage), and `ess-diff/24`. Opening them adds
//! no construct, so a document at the new number reads exactly as at the one before it.

use ess_cli::TemporaryDirectory;
use ess_diff::{DeltaFormat, EssDelta, RawEssDelta};
use serde_json::{json, Value};
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

fn ess(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ess"))
        .args(args)
        .output()
        .unwrap()
}

fn good(args: &[&str]) -> Output {
    let output = ess(args);
    assert!(
        output.status.success(),
        "{args:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    output
}

/// A ledger model valid at `ess/36`, written into a directory of its own declaring `format`.
fn ledger_at(directory: &Path, label: &str, format: &str) -> PathBuf {
    const LEDGER: &str = include_str!(
        "../../../verify/ess-conformance/tests/fixtures/aggregate-measures-over-expressions.yaml"
    );
    let target = directory.join(label);
    fs::create_dir_all(&target).unwrap();
    let text = LEDGER.replacen("format: ess/36\n", &format!("format: {format}\n"), 1);
    fs::write(target.join("system.yaml"), text).unwrap();
    target
}

#[test]
fn ess_37_is_listed_unreleased_and_reads_as_ess_36() {
    let listed = good(&["specify", "formats", "--format", "json"]);
    let rows: Value = serde_json::from_slice(&listed.stdout).unwrap();
    let row = rows
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["format"] == "ess/37")
        .unwrap_or_else(|| panic!("ess/37 is not listed: {rows}"));
    assert_eq!(row["release"], Value::Null, "{row}");
    assert_eq!(row["newest"], json!(true), "{row}");

    let directory = TemporaryDirectory::create("ess-epic-formats-source").unwrap();
    let older = ledger_at(&directory, "older", "ess/36");
    let newer = ledger_at(&directory, "newer", "ess/37");
    good(&["specify", "validate", "--path", newer.to_str().unwrap()]);
    let compile = |path: &Path| {
        good(&[
            "specify",
            "compile",
            "--path",
            path.to_str().unwrap(),
            "--format",
            "json",
        ])
        .stdout
    };
    let older = String::from_utf8(compile(&older)).unwrap();
    let newer = String::from_utf8(compile(&newer)).unwrap();
    assert_eq!(
        older.replace("ess/36", "ess/37"),
        newer,
        "IR bytes differ apart from the format field"
    );
}

/// A suite for billing, synthesized with `extra`, and a copy with its `suite_version` rewritten.
fn rewritten(directory: &Path, label: &str, extra: &[&str], major: u32) -> (PathBuf, PathBuf) {
    let original = directory.join(format!("{label}.json"));
    let model = root().join("examples/billing");
    let mut args = vec!["conform", "synthesize", "--path", model.to_str().unwrap()];
    args.extend_from_slice(extra);
    args.extend(["--out", original.to_str().unwrap()]);
    good(&args);
    let mut suite: Value = serde_json::from_str(&fs::read_to_string(&original).unwrap()).unwrap();
    suite["provenance"]["suite_version"] = json!(format!("ess-conformance/{major}"));
    let moved = directory.join(format!("{label}-{major}.json"));
    fs::write(&moved, serde_json::to_string(&suite).unwrap()).unwrap();
    (original, moved)
}

/// The verdict a run gives: its report, without the suite it names.
fn verdict(suite: &Path) -> Value {
    let output = good(&[
        "conform",
        "run",
        "--suite",
        suite.to_str().unwrap(),
        "--target",
        "billing",
        "--report-format",
        "2",
        "--format",
        "json",
    ]);
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    json!({
        "counts": report["summary"]["counts"],
        "execution_status": report["summary"]["execution_status"],
        "conformance_status": report["summary"]["conformance_status"],
        "outcomes": report["summary"]["outcomes"],
    })
}

#[test]
fn the_suite_pair_64_and_65_is_read_where_62_and_63_are() {
    let directory = TemporaryDirectory::create("ess-epic-formats-suite").unwrap();
    let (ordinary, ordinary_64) = rewritten(&directory, "ordinary", &[], 64);
    let (coverage, coverage_65) = rewritten(&directory, "coverage", &["--suite-format", "5"], 65);
    assert_eq!(verdict(&ordinary), verdict(&ordinary_64));
    assert_eq!(verdict(&coverage), verdict(&coverage_65));

    // No synthesis writes them yet: the major follows the features the specification uses.
    for path in [&ordinary, &coverage] {
        let text = fs::read_to_string(path).unwrap();
        assert!(
            !text.contains("\"ess-conformance/64\"") && !text.contains("\"ess-conformance/65\""),
            "{path:?}"
        );
    }
}

#[test]
fn ess_diff_24_is_read_and_written_where_23_is_and_no_change_kind_is_written_at_it() {
    let format = DeltaFormat::parse("ess-diff/24").unwrap();
    assert!(format.is_supported());
    assert!(ess_diff::SUPPORTED_DELTA_FORMATS.contains(&24));

    let directory = TemporaryDirectory::create("ess-epic-formats-diff").unwrap();
    let model = ledger_at(&directory, "model", "ess/36");
    let delta = good(&[
        "verify",
        "diff",
        "--from",
        model.to_str().unwrap(),
        "--to",
        model.to_str().unwrap(),
        "--format",
        "json",
    ]);
    let text = String::from_utf8(delta.stdout).unwrap();
    assert!(!text.contains("ess-diff/24"), "{text}");

    let mut document: Value = serde_json::from_str(&text).unwrap();
    document["format"] = json!("ess-diff/24");
    let raw: RawEssDelta = serde_json::from_value(document).unwrap();
    let read = EssDelta::try_from(raw).unwrap();
    let written = read.to_canonical_json_for(format).unwrap();
    assert!(written.contains("\"ess-diff/24\""), "{written}");
}
