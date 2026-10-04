//! `ess verify conform mutate` writing `ess-mutation-report/2`: issues #203 and #210 at the CLI.
//!
//! A mutant whose suite gained synthesis refusals the baseline does not have, and that no scored
//! scenario killed, is `unwitnessed` and exits 3, never 1 as a survivor or 0. A baseline whose
//! runner skipped scenarios is scored on the rest; one that executed nothing exits 3 with
//! "nothing scored". The project's runner is fabricated: each report is written by hand from the
//! emitted suite.
//!
//! Issue #218: that #203 mutant leaves a guard no input satisfies, so it is `equivalent`, naming
//! the guard; a mutant on an outcome the baseline suite already refuses is `unwitnessed`, naming
//! the baseline refusal. Both exit 3 when nothing survived.

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

fn scratch(name: &str) -> PathBuf {
    let directory = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("mutate-report-v2")
        .join(format!("{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&directory);
    std::fs::create_dir_all(&directory).expect("a scratch directory");
    directory
}

fn read_json(path: &Path) -> Value {
    let bytes = std::fs::read(path).unwrap_or_else(|error| panic!("{}: {error}", path.display()));
    serde_json::from_slice(&bytes).expect("JSON")
}

/// Emits `--path spec` with `--class class` into `<scratch>/emitted`, and returns its stdout too.
fn emit(name: &str, spec: &Path, class: &str) -> (PathBuf, String) {
    let emitted = scratch(name).join("emitted");
    let output = mutate(&[
        "--path",
        spec.to_str().unwrap(),
        "--class",
        class,
        "--emit",
        emitted.to_str().unwrap(),
    ]);
    let stdout = text(&output.stdout);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{stdout}{}",
        text(&output.stderr)
    );
    (emitted, stdout)
}

/// The issue #203 specification, copied into a directory of its own.
fn shop(name: &str) -> PathBuf {
    let spec = scratch(&format!("{name}-spec"));
    std::fs::copy(
        root().join("crates/verify/ess-conformance/tests/fixtures/mutation-gained-refusal.yaml"),
        spec.join("system.yaml"),
    )
    .expect("the fixture copies");
    spec
}

/// The `ess-conformance-report/2` a project runner writes for the suite in `dir`, with one
/// `"<status> <id>"` entry per scenario that did not pass.
fn fabricate(dir: &Path, not_passed: &[String]) {
    project_report::fabricate(dir, not_passed);
}

fn scenario_ids(dir: &Path) -> Vec<String> {
    read_json(&dir.join("suite.json"))["scenarios"]
        .as_object()
        .expect("scenarios")
        .keys()
        .cloned()
        .collect()
}

fn suites(emitted: &Path) -> Vec<PathBuf> {
    let manifest = read_json(&emitted.join("manifest.json"));
    let mut dirs = vec![emitted.join("baseline")];
    dirs.extend(
        manifest["mutants"]
            .as_array()
            .expect("mutants")
            .iter()
            .filter_map(|mutant| mutant["dir"].as_str().map(|dir| emitted.join(dir))),
    );
    dirs
}

fn collect(emitted: &Path, out: &Path) -> Output {
    mutate(&[
        "--collect",
        emitted.to_str().unwrap(),
        "--report-out",
        out.to_str().unwrap(),
    ])
}

#[test]
fn a_dead_guard_mutant_is_reported_equivalent_with_its_added_refusal_and_exits_three() {
    let (emitted, stdout) = emit("unwitnessed", &shop("unwitnessed"), "guard-connective");
    assert!(
        stdout.contains("1 with synthesis refusals the baseline does not have")
            && stdout.contains("1 with a guard no input satisfies"),
        "{stdout}"
    );
    let manifest = read_json(&emitted.join("manifest.json"));
    assert_eq!(manifest["format"], "ess-mutation-manifest/3");
    for dir in suites(&emitted) {
        fabricate(&dir, &[]);
    }
    let out = emitted.parent().unwrap().join("mutation-report.json");
    let output = collect(&emitted, &out);
    let stdout = text(&output.stdout);
    assert_eq!(
        output.status.code(),
        Some(3),
        "{stdout}{}",
        text(&output.stderr)
    );
    assert!(
        stdout.contains("0 survived") && stdout.contains("1 equivalent"),
        "{stdout}"
    );
    let line = stdout
        .lines()
        .find(|line| line.starts_with("equivalent guard-connective/"))
        .unwrap_or_else(|| panic!("{stdout}"));
    assert!(
        line.contains("ESS-MUTATE-005")
            && line.contains("no input satisfies `(status == Paid and status == Shipped)`")
            && line.contains("ESS-SYNTH-003 `shop.order.ReportStatus/outcome/settled`"),
        "{line}"
    );
    let report = read_json(&out);
    assert_eq!(report["format"], "ess-mutation-report/3");
    assert_eq!(report["counts"]["equivalent"], 1);
    assert_eq!(report["mutants"][0]["verdict"], "equivalent");
    assert_eq!(
        report["mutants"][0]["unsatisfiable_guard"],
        "(status == Paid and status == Shipped)"
    );
    assert_eq!(
        report["mutants"][0]["added_refusals"],
        json!([{
            "code": "ESS-SYNTH-003",
            "scenario": "shop.order.ReportStatus/outcome/settled",
            "subject": "outcome shop.order.ReportStatus/settled"
        }])
    );
}

/// The issue #218 desk: its `contradictory` refusal is reached by no input, so the baseline
/// refuses that outcome's scenario.
fn desk(name: &str) -> PathBuf {
    let spec = scratch(&format!("{name}-spec"));
    std::fs::copy(
        root()
            .join("crates/verify/ess-conformance/tests/fixtures/mutation-unwitnessed-outcome.yaml"),
        spec.join("system.yaml"),
    )
    .expect("the fixture copies");
    spec
}

const ON_REFUSED: &str = "error-swap/desk.ticket.FileTicket/contradictory";
const ON_WITNESSED: &str = "error-swap/desk.ticket.FileTicket/refused";

#[test]
fn a_mutant_on_an_outcome_the_baseline_refuses_is_unwitnessed_with_the_target() {
    let spec = desk("target");
    let out = spec.join("mutation-report.json");
    let output = mutate(&[
        "--path",
        spec.to_str().unwrap(),
        "--target",
        "interpreted",
        "--class",
        "error-swap",
        "--report-out",
        out.to_str().unwrap(),
    ]);
    let stdout = text(&output.stdout);
    assert_eq!(
        output.status.code(),
        Some(3),
        "{stdout}{}",
        text(&output.stderr)
    );
    assert!(
        stdout.contains("1 killed, 0 survived") && stdout.contains("1 unwitnessed"),
        "{stdout}"
    );
    let line = stdout
        .lines()
        .find(|line| line.starts_with(&format!("unwitnessed {ON_REFUSED}:")))
        .unwrap_or_else(|| panic!("{stdout}"));
    assert!(
        line.contains("ESS-MUTATE-004")
            && line.contains(
                "the baseline refuses ESS-SYNTH-003 `desk.ticket.FileTicket/outcome/contradictory`"
            ),
        "{line}"
    );
    let report = read_json(&out);
    let entry = report["mutants"]
        .as_array()
        .unwrap()
        .iter()
        .find(|it| it["id"] == ON_REFUSED)
        .unwrap();
    assert_eq!(
        entry["baseline_refusals"],
        json!([{
            "code": "ESS-SYNTH-003",
            "scenario": "desk.ticket.FileTicket/outcome/contradictory",
            "subject": "outcome desk.ticket.FileTicket/contradictory"
        }])
    );
}

#[test]
fn a_collected_mutant_on_an_outcome_the_baseline_refuses_is_unwitnessed() {
    let (emitted, _) = emit("desk", &desk("collect"), "error-swap");
    for dir in suites(&emitted) {
        // The witnessed refusal's swap is killed by its own scenario, as a correct target would.
        let failed: Vec<String> = if dir.ends_with(ON_WITNESSED) {
            vec!["failed desk.ticket.FileTicket/outcome/refused".to_owned()]
        } else {
            Vec::new()
        };
        fabricate(&dir, &failed);
    }
    let out = emitted.parent().unwrap().join("mutation-report.json");
    let output = collect(&emitted, &out);
    let stdout = text(&output.stdout);
    assert_eq!(
        output.status.code(),
        Some(3),
        "{stdout}{}",
        text(&output.stderr)
    );
    assert!(
        stdout.contains("1 killed, 0 survived") && stdout.contains("1 unwitnessed"),
        "{stdout}"
    );
    let report = read_json(&out);
    assert_eq!(report["counts"]["unwitnessed"], 1);
    let entry = report["mutants"]
        .as_array()
        .unwrap()
        .iter()
        .find(|it| it["id"] == ON_REFUSED)
        .unwrap();
    assert_eq!(entry["verdict"], "unwitnessed");
    assert_eq!(
        entry["baseline_refusals"][0]["scenario"],
        "desk.ticket.FileTicket/outcome/contradictory"
    );
}

#[test]
fn a_baseline_with_skipped_scenarios_is_scored_on_the_rest() {
    let (emitted, _) = emit("skipped", &root().join("examples/billing"), "error-swap");
    let baseline = emitted.join("baseline");
    let skipped = scenario_ids(&baseline)[0].clone();
    fabricate(&baseline, &[format!("skipped {skipped}")]);
    for dir in suites(&emitted).into_iter().skip(1) {
        fabricate(&dir, &[]);
    }
    let out = emitted.parent().unwrap().join("mutation-report.json");
    let output = collect(&emitted, &out);
    let stdout = text(&output.stdout);
    let stderr = text(&output.stderr);
    assert!(!stderr.contains("ESS-MUTATE-001"), "{stderr}");
    let written = read_json(&out);
    for mutant in written["mutants"].as_array().unwrap() {
        // Every mutant report passed, so a mutant with an excluded scenario is inconclusive where
        // it changed that scenario and a survivor where it holds the baseline's own copy.
        if mutant["excluded"].is_array() {
            assert!(
                mutant["verdict"] == "inconclusive" || mutant["verdict"] == "survived",
                "{mutant}"
            );
        }
    }
    let expected = if written["counts"]["survived"] == 0 {
        3
    } else {
        1
    };
    assert_eq!(output.status.code(), Some(expected), "{stdout}{stderr}");
    assert!(stdout.contains("1 not scored"), "{stdout}");
    assert!(
        stdout.contains(&format!(
            "not scored {skipped}: the baseline's run reported it skipped"
        )),
        "{stdout}"
    );
    let report = read_json(&out);
    assert_eq!(
        report["baseline"]["not_scored"],
        json!([{"scenario": skipped, "status": "skipped"}])
    );
}

#[test]
fn a_baseline_that_executed_nothing_exits_three_with_nothing_scored() {
    let (emitted, _) = emit("nothing", &root().join("examples/billing"), "error-swap");
    let baseline = emitted.join("baseline");
    let all: Vec<String> = scenario_ids(&baseline)
        .into_iter()
        .map(|id| format!("skipped {id}"))
        .collect();
    fabricate(&baseline, &all);
    for dir in suites(&emitted).into_iter().skip(1) {
        fabricate(&dir, &[]);
    }
    let out = emitted.parent().unwrap().join("mutation-report.json");
    let output = collect(&emitted, &out);
    let stderr = text(&output.stderr);
    assert_eq!(output.status.code(), Some(3), "{stderr}");
    assert!(stderr.starts_with("nothing scored"), "{stderr}");
    assert!(
        !out.exists(),
        "a collection that scored nothing writes no report"
    );
}
