//! `ess verify conform mutate --emit DIR` / `--collect DIR`: the audit against a project's runner.
//!
//! `docs/design/mutation-audit-and-model-runner.md`, "Auditing an external target". `--emit` writes
//! the baseline suite and every mutant's suite and runs nothing; the project runs its own runner
//! over each and writes an `ess-conformance-report/2` beside it; `--collect` scores those reports
//! into `ess-mutation-report/3`. Here the project's runner is fabricated: each report is written by
//! hand from the emitted suite, some red, some green, one missing.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use serde_json::{json, Value};

#[path = "support/project_report.rs"]
mod project_report;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("the workspace root exists")
}

fn mutate(args: &[&str]) -> Output {
    let mut all = vec!["verify", "conform", "mutate"];
    all.extend_from_slice(args);
    Command::new(env!("CARGO_BIN_EXE_ess"))
        .current_dir(root())
        .args(&all)
        .output()
        .expect("the `ess` binary runs")
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8(bytes.to_vec()).expect("UTF-8")
}

/// A fresh directory for one test; the emission goes to a child that does not exist yet.
fn scratch(name: &str) -> PathBuf {
    let directory = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("mutate-external")
        .join(format!("{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&directory);
    std::fs::create_dir_all(&directory).expect("a scratch directory");
    directory
}

fn read_json(path: &Path) -> Value {
    let bytes = std::fs::read(path).unwrap_or_else(|error| panic!("{}: {error}", path.display()));
    serde_json::from_slice(&bytes).expect("JSON")
}

/// Emits billing's `error-swap` (five mutants that compile) and `transition-to` (three that do
/// not) into `<scratch>/emitted`.
fn emit(name: &str) -> PathBuf {
    let emitted = scratch(name).join("emitted");
    let output = mutate(&[
        "--path",
        "examples/billing",
        "--class",
        "error-swap",
        "--class",
        "transition-to",
        "--emit",
        emitted.to_str().unwrap(),
    ]);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}{}",
        text(&output.stdout),
        text(&output.stderr)
    );
    emitted
}

/// The `ess-conformance-report/2` a project runner writes for the suite in `dir`, failing exactly
/// the scenarios in `failed`.
fn fabricate(dir: &Path, failed: &[String]) {
    let failed: Vec<String> = failed.iter().map(|id| format!("failed {id}")).collect();
    project_report::fabricate(dir, &failed);
}

fn first_scenario(dir: &Path) -> String {
    let suite = read_json(&dir.join("suite.json"));
    suite["scenarios"]
        .as_object()
        .expect("scenarios")
        .keys()
        .next()
        .expect("a scenario")
        .clone()
}

fn dirs(manifest: &Value) -> Vec<String> {
    manifest["mutants"]
        .as_array()
        .expect("mutants")
        .iter()
        .filter_map(|mutant| mutant["dir"].as_str().map(str::to_owned))
        .collect()
}

#[test]
fn emit_writes_every_suite_and_runs_nothing() {
    let emitted = emit("layout");
    let manifest = read_json(&emitted.join("manifest.json"));
    assert_eq!(manifest["format"], "ess-mutation-manifest/3");
    assert_eq!(manifest["mutants"].as_array().unwrap().len(), 8);
    assert!(emitted.join("baseline/suite.json").is_file());
    let ran = dirs(&manifest);
    assert_eq!(
        ran.len(),
        5,
        "the three transition-to mutants are stillborn"
    );
    for dir in &ran {
        assert!(emitted.join(dir).join("suite.json").is_file(), "{dir}");
        let identity = read_json(&emitted.join(dir).join("mutant.json"));
        assert_eq!(identity["class"], "error-swap");
        assert!(identity["site"].is_string() && identity["change"].is_string());
    }
    let reports = walk(&emitted)
        .into_iter()
        .filter(|path| path.ends_with("report.json"))
        .count();
    assert_eq!(reports, 0, "emit runs no target and writes no report");
}

#[test]
fn collect_scores_red_green_and_missing_reports() {
    let emitted = emit("collect");
    let manifest = read_json(&emitted.join("manifest.json"));
    fabricate(&emitted.join("baseline"), &[]);
    let ran = dirs(&manifest);
    let killer = first_scenario(&emitted.join(&ran[0]));
    fabricate(&emitted.join(&ran[0]), std::slice::from_ref(&killer));
    fabricate(
        &emitted.join(&ran[1]),
        &[first_scenario(&emitted.join(&ran[1]))],
    );
    fabricate(&emitted.join(&ran[2]), &[]);
    fabricate(&emitted.join(&ran[3]), &[]);
    // ran[4] has no report.

    let out = emitted.parent().unwrap().join("mutation-report.json");
    let output = mutate(&[
        "--collect",
        emitted.to_str().unwrap(),
        "--format",
        "json",
        "--report-out",
        out.to_str().unwrap(),
    ]);
    assert_eq!(
        output.status.code(),
        Some(1),
        "a survivor exits 1: {}{}",
        text(&output.stdout),
        text(&output.stderr)
    );
    assert_eq!(output.stdout, std::fs::read(&out).expect("--report-out"));
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["format"], "ess-mutation-report/3");
    assert_eq!(report["implementation"], "project-runner 1.0.0");
    assert_eq!(
        report["counts"],
        json!({"equivalent": 0, "inconclusive": 1, "killed": 2, "mutants": 8, "stillborn": 3, "survived": 2, "unwitnessed": 0})
    );
    let entry = |id: &str| {
        report["mutants"]
            .as_array()
            .unwrap()
            .iter()
            .find(|entry| entry["id"] == id)
            .unwrap_or_else(|| panic!("{id} is listed"))
            .clone()
    };
    assert_eq!(entry(&ran[0])["verdict"], "killed");
    assert_eq!(entry(&ran[0])["killers"], json!([killer]));
    assert_eq!(entry(&ran[2])["verdict"], "survived");
    let missing = entry(&ran[4]);
    assert_eq!(missing["verdict"], "inconclusive");
    assert!(
        missing["unscored"]
            .as_str()
            .is_some_and(|why| why.contains("report.json")),
        "{missing}"
    );

    let text_output = mutate(&["--collect", emitted.to_str().unwrap()]);
    let stdout = text(&text_output.stdout);
    assert!(
        stdout.starts_with(
            "mutation audit of billing v3 against project-runner 1.0.0: 8 mutant(s), 2 killed, 2 \
             survived, 1 inconclusive, 3 stillborn"
        ),
        "{stdout}"
    );
}

#[test]
fn a_baseline_report_that_did_not_pass_exits_three_with_mutate_001() {
    let emitted = emit("baseline");
    let baseline = emitted.join("baseline");
    let failing = first_scenario(&baseline);
    fabricate(&baseline, std::slice::from_ref(&failing));
    for dir in dirs(&read_json(&emitted.join("manifest.json"))) {
        fabricate(&emitted.join(dir), &[]);
    }
    let out = emitted.parent().unwrap().join("mutation-report.json");
    let output = mutate(&[
        "--collect",
        emitted.to_str().unwrap(),
        "--report-out",
        out.to_str().unwrap(),
    ]);
    let stderr = text(&output.stderr);
    assert_eq!(output.status.code(), Some(3), "{stderr}");
    assert!(stderr.starts_with("refusal[ESS-MUTATE-001]"), "{stderr}");
    assert!(stderr.contains(&format!("\n  {failing}")), "{stderr}");
    assert!(!out.exists(), "a refused collection writes no report");
}

#[test]
fn emit_refuses_a_directory_that_already_holds_files() {
    let emitted = emit("twice");
    let output = mutate(&[
        "--path",
        "examples/billing",
        "--class",
        "error-swap",
        "--emit",
        emitted.to_str().unwrap(),
    ]);
    assert_eq!(output.status.code(), Some(1), "{}", text(&output.stderr));
    assert!(text(&output.stderr).contains("not empty"));
}

#[test]
fn exactly_one_of_target_emit_and_collect_is_required() {
    let directory = scratch("usage");
    let dir = directory.join("x");
    let dir = dir.to_str().unwrap();
    for args in [
        vec!["--path", "examples/billing"],
        vec!["--target", "billing", "--emit", dir],
        vec!["--target", "billing", "--collect", dir],
        vec!["--emit", dir, "--collect", dir],
        vec!["--collect", dir, "--class", "error-swap"],
    ] {
        let output = mutate(&args);
        assert_eq!(
            output.status.code(),
            Some(2),
            "{args:?}: {}",
            text(&output.stderr)
        );
    }
}

#[test]
fn the_help_names_emit_and_collect() {
    let output = mutate(&["--help"]);
    let help = text(&output.stdout);
    assert!(output.status.success(), "{help}");
    assert!(help.contains("--emit <EMIT>"), "{help}");
    assert!(help.contains("--collect <COLLECT>"), "{help}");
}

// ---- --component (beyond10x/ess#236) --------------------------------------------------------------

/// Emits billing's `error-swap` mutants for `component` into `<scratch>/emitted`.
fn emit_component(name: &str, component: &str) -> PathBuf {
    let emitted = scratch(name).join("emitted");
    let output = mutate(&[
        "--path",
        "examples/billing",
        "--class",
        "error-swap",
        "--component",
        component,
        "--emit",
        emitted.to_str().unwrap(),
    ]);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}{}",
        text(&output.stdout),
        text(&output.stderr)
    );
    emitted
}

#[test]
fn a_component_emission_collects_into_a_report_naming_the_component() {
    let emitted = emit_component("component", "invoice-service");
    let manifest = read_json(&emitted.join("manifest.json"));
    assert_eq!(manifest["format"], "ess-mutation-manifest/4");
    assert_eq!(manifest["component"], "invoice-service");
    let baseline = read_json(&emitted.join("baseline/suite.json"));
    assert_eq!(baseline["provenance"]["component"], "invoice-service");
    // The project's runner is the built-in billing reference, run over each emitted suite.
    let ran = dirs(&manifest);
    assert!(!ran.is_empty(), "{manifest}");
    for dir in std::iter::once("baseline").chain(ran.iter().map(String::as_str)) {
        let suite = emitted.join(dir).join("suite.json");
        let report = emitted.join(dir).join("report.json");
        let output = Command::new(env!("CARGO_BIN_EXE_ess"))
            .current_dir(root())
            .args([
                "verify",
                "conform",
                "run",
                "--path",
                "examples/billing",
                "--suite",
                suite.to_str().unwrap(),
                "--target",
                "billing",
                "--report-format",
                "2",
                "--report-out",
                report.to_str().unwrap(),
            ])
            .output()
            .expect("the `ess` binary runs");
        assert!(report.is_file(), "{dir}: {}", text(&output.stderr));
    }

    let output = mutate(&[
        "--collect",
        emitted.to_str().unwrap(),
        "--component",
        "invoice-service",
        "--format",
        "json",
    ]);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}{}",
        text(&output.stdout),
        text(&output.stderr)
    );
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["format"], "ess-mutation-report/4");
    assert_eq!(report["component"], "invoice-service");

    let other = mutate(&[
        "--collect",
        emitted.to_str().unwrap(),
        "--component",
        "email-service",
    ]);
    let stderr = text(&other.stderr);
    assert_eq!(other.status.code(), Some(1), "{stderr}");
    assert!(
        stderr.contains("email-service") && stderr.contains("invoice-service"),
        "{stderr}"
    );
}

#[test]
fn the_built_in_targets_refuse_component_by_name() {
    let output = mutate(&[
        "--path",
        "examples/billing",
        "--target",
        "billing",
        "--component",
        "invoice-service",
    ]);
    let stderr = text(&output.stderr);
    assert_eq!(output.status.code(), Some(2), "{stderr}");
    assert!(
        stderr.contains("cannot be used with")
            && stderr.contains("'--component <COMPONENT>'")
            && stderr.contains("'--target <TARGET>'"),
        "{stderr}"
    );
}

#[test]
fn an_undeclared_component_is_refused_and_nothing_is_written() {
    let emitted = scratch("undeclared").join("emitted");
    let output = mutate(&[
        "--path",
        "examples/billing",
        "--class",
        "error-swap",
        "--component",
        "ledger",
        "--emit",
        emitted.to_str().unwrap(),
    ]);
    let stderr = text(&output.stderr);
    assert_eq!(output.status.code(), Some(1), "{stderr}");
    assert!(
        stderr.contains("ledger") && stderr.contains("invoice-service"),
        "{stderr}"
    );
    assert!(!emitted.join("manifest.json").exists());
}

#[test]
fn the_help_offers_the_new_classes_and_component() {
    let output = mutate(&["--help"]);
    let help = text(&output.stdout);
    assert!(help.contains("--component <COMPONENT>"), "{help}");
    assert!(help.contains("sets-drop"), "{help}");
    assert!(help.contains("precedence-swap"), "{help}");
}

fn walk(directory: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    let mut pending = vec![directory.to_path_buf()];
    while let Some(next) = pending.pop() {
        for entry in std::fs::read_dir(&next).expect("readable") {
            let path = entry.expect("an entry").path();
            if path.is_dir() {
                pending.push(path);
            } else {
                found.push(path);
            }
        }
    }
    found
}
