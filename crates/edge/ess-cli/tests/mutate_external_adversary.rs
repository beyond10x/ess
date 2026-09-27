//! Adversary cases for `ess verify conform mutate --emit/--collect` (beyond10x/ess#153): exit
//! statuses and stdout the unit's own `mutate_external.rs` does not pin.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use serde_json::{json, Value};

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
    String::from_utf8_lossy(bytes).into_owned()
}

fn scratch(name: &str) -> PathBuf {
    let directory = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("mutate-external-adversary")
        .join(format!("{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&directory);
    std::fs::create_dir_all(&directory).expect("a scratch directory");
    directory
}

fn read_json(path: &Path) -> Value {
    serde_json::from_slice(&std::fs::read(path).expect("readable")).expect("JSON")
}

fn emit_args(emitted: &Path, extra: &[&str]) -> Output {
    let mut args = vec![
        "--path",
        "examples/billing",
        "--class",
        "error-swap",
        "--emit",
        emitted.to_str().unwrap(),
    ];
    args.extend_from_slice(extra);
    mutate(&args)
}

fn emit(name: &str) -> PathBuf {
    let emitted = scratch(name).join("emitted");
    let output = emit_args(&emitted, &[]);
    assert_eq!(output.status.code(), Some(0), "{}", text(&output.stderr));
    emitted
}

fn fabricate(dir: &Path, failed: &[String]) {
    let suite = read_json(&dir.join("suite.json"));
    let provenance = &suite["provenance"];
    let report = json!({
        "format": "ess-conformance-report/1",
        "specification": format!(
            "{}/{}",
            provenance["system"].as_str().unwrap(),
            provenance["specification_version"].as_str().unwrap()
        ),
        "spec_digest": provenance["spec_digest"],
        "implementation": "project-runner 1.0.0",
        "status": if failed.is_empty() { "passed" } else { "failed" },
        "scenarios_total": suite["scenarios"].as_object().unwrap().len(),
        "scenarios_failed": failed.len(),
        "suite_version": provenance["suite_version"],
        "failed_scenarios": failed.iter().map(|id| format!("failed {id}")).collect::<Vec<_>>(),
        "completed_at": 1_700_000_000_000_u64,
    });
    std::fs::write(
        dir.join("report.json"),
        serde_json::to_string_pretty(&report).unwrap() + "\n",
    )
    .unwrap();
}

fn first_scenario(dir: &Path) -> String {
    read_json(&dir.join("suite.json"))["scenarios"]
        .as_object()
        .unwrap()
        .keys()
        .next()
        .unwrap()
        .clone()
}

fn dirs(emitted: &Path) -> Vec<String> {
    read_json(&emitted.join("manifest.json"))["mutants"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|mutant| mutant["dir"].as_str().map(str::to_owned))
        .collect()
}

#[test]
fn adversary_collect_exits_zero_when_every_mutant_is_killed() {
    let emitted = emit("all-killed");
    fabricate(&emitted.join("baseline"), &[]);
    for dir in dirs(&emitted) {
        let dir = emitted.join(dir);
        let killer = first_scenario(&dir);
        fabricate(&dir, &[killer]);
    }
    let output = mutate(&["--collect", emitted.to_str().unwrap()]);
    assert_eq!(output.status.code(), Some(0), "{}", text(&output.stdout));
}

#[test]
fn adversary_collect_exits_three_when_no_mutant_report_was_written() {
    let emitted = emit("none-reported");
    fabricate(&emitted.join("baseline"), &[]);
    let output = mutate(&["--collect", emitted.to_str().unwrap(), "--format", "json"]);
    assert_eq!(output.status.code(), Some(3), "{}", text(&output.stderr));
    let report: Value = serde_json::from_slice(&output.stdout).expect("a report on stdout");
    assert_eq!(report["counts"]["inconclusive"], report["counts"]["mutants"]);
}

#[test]
fn adversary_collect_of_a_directory_with_no_manifest_exits_one() {
    let directory = scratch("no-manifest");
    let output = mutate(&["--collect", directory.to_str().unwrap()]);
    assert_eq!(output.status.code(), Some(1), "{}", text(&output.stderr));
    assert!(text(&output.stderr).contains("manifest.json"));
}

#[test]
fn adversary_collect_of_a_manifest_that_leaves_the_emission_exits_one_and_reads_nothing() {
    let emitted = emit("traversal");
    fabricate(&emitted.join("baseline"), &[]);
    // A sibling outside the emission holding a green baseline copy the manifest points at.
    let outside = emitted.parent().unwrap().join("outside");
    std::fs::create_dir_all(&outside).unwrap();
    for file in ["suite.json", "report.json"] {
        std::fs::copy(emitted.join("baseline").join(file), outside.join(file)).unwrap();
    }
    let path = emitted.join("manifest.json");
    let manifest = std::fs::read_to_string(&path)
        .unwrap()
        .replace("\"dir\": \"baseline\"", "\"dir\": \"../outside\"");
    std::fs::write(&path, manifest).unwrap();
    let output = mutate(&["--collect", emitted.to_str().unwrap()]);
    assert_eq!(output.status.code(), Some(1), "{}", text(&output.stdout));
    assert!(output.stdout.is_empty(), "{}", text(&output.stdout));
}

#[test]
fn adversary_emit_json_prints_exactly_the_manifest_it_wrote() {
    let emitted = scratch("emit-json").join("emitted");
    let output = emit_args(&emitted, &["--format", "json"]);
    assert_eq!(output.status.code(), Some(0), "{}", text(&output.stderr));
    assert_eq!(
        text(&output.stdout),
        std::fs::read_to_string(emitted.join("manifest.json")).unwrap()
    );
}

#[test]
fn adversary_emit_onto_a_file_exits_one_and_leaves_it() {
    let file = scratch("emit-file").join("a-file");
    std::fs::write(&file, "keep\n").unwrap();
    let output = emit_args(&file, &[]);
    assert_eq!(output.status.code(), Some(1), "{}", text(&output.stderr));
    assert_eq!(std::fs::read_to_string(&file).unwrap(), "keep\n");
}

#[test]
fn adversary_emit_with_no_site_exits_three_and_writes_nothing() {
    let emitted = scratch("emit-003").join("emitted");
    let output = mutate(&[
        "--path",
        "examples/oracle-fixture",
        "--class",
        "order-flip",
        "--emit",
        emitted.to_str().unwrap(),
    ]);
    assert_eq!(output.status.code(), Some(3), "{}", text(&output.stderr));
    assert!(text(&output.stderr).contains("ESS-MUTATE-003"));
    assert!(!emitted.exists(), "a refused emission writes no directory");
}

#[test]
fn adversary_emit_rejects_report_out_as_a_usage_error() {
    let emitted = scratch("emit-report-out").join("emitted");
    let out = emitted.with_extension("json");
    let output = emit_args(&emitted, &["--report-out", out.to_str().unwrap()]);
    assert_eq!(output.status.code(), Some(2), "{}", text(&output.stderr));
}
