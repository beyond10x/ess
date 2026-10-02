//! `ess verify conform report`: a runner's supplied results become report/2 over ESS's admission.
use serde_json::{json, Value};
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}
fn scratch(label: &str) -> PathBuf {
    let directory = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("conform-report-{label}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&directory);
    fs::create_dir_all(&directory).unwrap();
    directory
}
fn ess(args: &[&str], paths: &[(&str, &Path)]) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_ess"));
    command.args(args);
    for (flag, path) in paths {
        command.arg(flag).arg(path);
    }
    command.output().unwrap()
}
/// A declared-coverage suite for the billing example, and the report/2 ESS's own run writes for it.
fn suite_and_own_report(directory: &Path) -> (PathBuf, Value) {
    let suite = directory.join("suite.json");
    let synthesized = ess(
        &["verify", "conform", "synthesize", "--suite-format", "5"],
        &[
            ("--path", &root().join("examples/billing")),
            ("--out", &suite),
        ],
    );
    assert!(
        synthesized.status.success(),
        "{}",
        String::from_utf8_lossy(&synthesized.stderr)
    );
    let own = directory.join("own.json");
    let run = ess(
        &[
            "verify",
            "conform",
            "run",
            "--target",
            "billing",
            "--report-format",
            "2",
        ],
        &[("--suite", &suite), ("--report-out", &own)],
    );
    assert!(
        run.status.success(),
        "{}",
        String::from_utf8_lossy(&run.stderr)
    );
    (
        suite,
        serde_json::from_str(&fs::read_to_string(own).unwrap()).unwrap(),
    )
}
fn results_from(report: &Value) -> Value {
    let mut results = Vec::new();
    for status in ["passed", "failed", "error", "unsupported"] {
        for id in report["outcomes"][status].as_array().unwrap() {
            results.push(json!({"scenario_id": id, "status": status}));
        }
    }
    results.reverse();
    json!({"format":"ess-conformance-results/1","completed_at":1_700_000_000_000_u64,"results":results})
}
fn report(directory: &Path, suite: &Path, results: &Value, extra: &[&str]) -> (Output, PathBuf) {
    let supplied = directory.join("results.json");
    fs::write(&supplied, results.to_string()).unwrap();
    let out = directory.join("report.json");
    let _ = fs::remove_file(&out);
    let mut args = vec![
        "verify",
        "conform",
        "report",
        "--implementation",
        "downstream-impl 2.0",
    ];
    args.extend_from_slice(extra);
    let output = ess(
        &args,
        &[
            ("--suite", suite),
            ("--results", &supplied),
            ("--report-out", &out),
        ],
    );
    (output, out)
}

#[test]
fn supplied_results_match_the_report_ess_writes_for_its_own_run() {
    let directory = scratch("match");
    let (suite, own) = suite_and_own_report(&directory);
    let (output, out) = report(
        &directory,
        &suite,
        &results_from(&own),
        &["--runner", "acme-runner@1.4.0"],
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let text = fs::read_to_string(&out).unwrap();
    assert!(text.ends_with("}\n") && !text.ends_with("\n\n"));
    let written: Value = serde_json::from_str(&text).unwrap();
    assert_eq!(
        written["producer_profile"],
        "external-scenario-status/1;runner=acme-runner@1.4.0"
    );
    assert_eq!(written["implementation"], "downstream-impl 2.0");
    assert_eq!(written["completed_at"], 1_700_000_000_000_u64);
    for key in [
        "format",
        "specification",
        "spec_digest",
        "suite",
        "coverage",
        "policy",
        "counts",
        "outcomes",
        "execution_status",
        "conformance_status",
    ] {
        assert_eq!(written[key], own[key], "{key}");
    }
    assert_eq!(written["coverage"]["knowledge"], "complete_inventory");
}

#[test]
fn refusals_name_the_offending_result_and_write_nothing() {
    let directory = scratch("refused");
    let (suite, own) = suite_and_own_report(&directory);
    let good = results_from(&own);
    let first = good["results"][0]["scenario_id"].clone();

    let mut unknown = good.clone();
    unknown["results"]
        .as_array_mut()
        .unwrap()
        .push(json!({"scenario_id":"billing.ghost/authored/nothing","status":"passed"}));
    let mut missing = good.clone();
    missing["results"].as_array_mut().unwrap().remove(0);
    let mut duplicate = good.clone();
    duplicate["results"]
        .as_array_mut()
        .unwrap()
        .push(json!({"scenario_id":first,"status":"passed"}));
    let mut status = good.clone();
    status["results"][0]["status"] = json!("skipped");
    let mut digest = good.clone();
    digest["suite_digest"] = json!(format!("sha256:{}", "b".repeat(64)));

    for (label, results, reason) in [
        ("unknown", unknown, "UnknownScenario"),
        ("missing", missing, "MissingResult"),
        ("duplicate", duplicate, "DuplicateResult"),
        ("status", status, "UnknownStatus"),
        ("digest", digest, "SuiteDigestMismatch"),
    ] {
        let (output, out) = report(&directory, &suite, &results, &[]);
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert_eq!(output.status.code(), Some(2), "{label}: {stderr}");
        assert!(stderr.contains(reason), "{label}: {stderr}");
        assert!(!out.exists(), "{label}: a refused report was written");
    }
    let (output, out) = report(&directory, &suite, &good, &["--runner", "no-version"]);
    assert_eq!(output.status.code(), Some(2));
    assert!(!out.exists());
}

#[test]
fn without_a_runner_the_profile_still_marks_the_results_as_supplied() {
    let directory = scratch("plain");
    let (suite, own) = suite_and_own_report(&directory);
    let (output, out) = report(&directory, &suite, &results_from(&own), &[]);
    assert!(output.status.success());
    let written: Value = serde_json::from_str(&fs::read_to_string(out).unwrap()).unwrap();
    assert_eq!(written["producer_profile"], "external-scenario-status/1");
}

#[test]
fn scenario_namespace_requirement_is_visible_before_cli_execution() {
    let directory = scratch("initial-state");
    let (suite, compact) = suite_and_own_report(&directory);
    let document: Value = serde_json::from_str(&fs::read_to_string(&suite).unwrap()).unwrap();
    assert_eq!(document["provenance"]["scenario_initial_state"], "empty");
    assert_eq!(
        document["provenance"]["suite_version"],
        "ess-conformance/35"
    );
    assert!(
        compact.get("scenario_initial_state").is_none(),
        "report/2 remains closed"
    );
    let output = ess(
        &[
            "verify",
            "conform",
            "run",
            "--target",
            "billing",
            "--report-format",
            "2",
        ],
        &[("--suite", &suite)],
    );
    assert!(output.status.success(), "{output:?}");
    assert!(String::from_utf8_lossy(&output.stderr)
        .contains("Requires an empty logical modeled-instance/event/invocation namespace"));
}
