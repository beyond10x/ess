//! Public generated-Go Run admission, completion and exact original identities.
use ess_conformance::{AdmittedSuite, ConformanceSuite, CountReport};
use serde_json::{json, Value};
use std::{
    path::{Path, PathBuf},
    process::{Command, Output},
};

const ID: &str = "review.count/authored/one";

fn document() -> Value {
    json!({"provenance":{"suite_version":"ess-conformance/4", "system":"review",
        "specification_version":"v1", "spec_digest":"a".repeat(64), "contract_digest":"a".repeat(64)},
        "scenarios":{ID:{"purpose":"Review completion and original admission", "steps":[], "source":[]}}})
}

fn module(label: &str) -> PathBuf {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .unwrap();
    let directory = root
        .join("target/review-boundaries-8/adversary-pass-1")
        .join(format!("go-{label}-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    let suite = ConformanceSuite::from_json(&document().to_string()).unwrap();
    for artifact in ess_conformance::go::emit(&suite) {
        let path = directory.join(artifact.path);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, artifact.contents).unwrap();
    }
    std::fs::write(directory.join("go.mod"), "module countreview\n\ngo 1.24\n").unwrap();
    std::fs::write(
        directory.join("essconform/review_test.go"),
        include_str!("fixtures/count-writer-pass1/target_test.go"),
    )
    .unwrap();
    directory
}

fn invoke(directory: &Path, label: &str, mode: &str, strict: bool, destination: bool) -> Output {
    let mut command = Command::new("go");
    command
        .args(["test", "-count=1", "-v", "./...", "-run", "^TestReview$"])
        .current_dir(directory)
        .env("ESS_REPORT_FORMAT", "2")
        .env_remove("ESS_CONFORMANCE_STRICT")
        .env_remove("ESS_CONFORMANCE_ALLOW_INCOMPLETE")
        .env_remove("ESS_REPORT_OUT")
        .env("REVIEW_MODE", mode)
        .env("REVIEW_MARKER", directory.join(format!("{label}.marker")));
    if strict {
        command.env("ESS_CONFORMANCE_STRICT", "1");
    }
    if destination {
        command.env(
            "ESS_REPORT_OUT",
            directory.join(format!("{label}.report.json")),
        );
    }
    let output = command
        .output()
        .expect("the required Go toolchain executes");
    let record = format!(
        "cwd: {}\ncommand: {command:?}\nexit: {:?}\nstdout:\n{}\nstderr:\n{}",
        directory.display(),
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    std::fs::write(directory.join(format!("{label}.log")), &record).unwrap();
    println!("{record}");
    output
}

#[test]
fn generated_go_rejects_closed_predicate_metadata_before_any_target() {
    let directory = module("predicate");
    let path = directory.join("essconform/suite.json");
    let valid = json!({"forall":{"in":"rows","as":"row","that":true}});
    let mut document = document();
    document["scenarios"][ID]["steps"] = json!([{"step":"expect_view","view":"review.count.Rows",
        "expectation":{"expect":"satisfies","predicate":valid.clone()}}]);
    std::fs::write(&path, document.to_string()).unwrap();
    assert!(AdmittedSuite::from_json(&document.to_string()).is_ok());
    assert!(invoke(&directory, "valid", "begin-skip", false, true)
        .status
        .success());

    let mut too_deep = json!(true);
    for _ in 0..33 {
        too_deep = json!({"not":too_deep});
    }
    let mut violations = Vec::new();
    for (label, predicate) in [
        (
            "unknown-quantifier-field",
            json!({"forall":{"in":"rows","as":"row","that":true,"future":true}}),
        ),
        (
            "invalid-quantifier-binding",
            json!({"forall":{"in":"rows","as":"row.part","that":true}}),
        ),
        ("excess-predicate-depth", too_deep),
    ] {
        document["scenarios"][ID]["steps"][0]["expectation"]["predicate"] = predicate;
        let original = document.to_string();
        assert!(
            AdmittedSuite::from_json(&original).is_err(),
            "Rust original admission control: {label}"
        );
        std::fs::write(&path, original).unwrap();
        let output = invoke(&directory, label, "begin-skip", false, true);
        let marker = directory.join(format!("{label}.marker"));
        let report = directory.join(format!("{label}.report.json"));
        if output.status.success() || marker.exists() || report.exists() {
            violations.push(format!(
                "{label}: exit={:?}, target={}, report={}",
                output.status.code(),
                marker.exists(),
                report.exists()
            ));
        }
    }
    assert!(
        violations.is_empty(),
        "new admitted execution must validate the frozen predicate envelope before targets:\n{}",
        violations.join("\n")
    );
}

#[test]
fn generated_go_abnormal_teardown_cannot_publish_a_completed_skip() {
    let directory = module("teardown");
    let mut document = document();
    document["scenarios"][ID]["steps"] =
        json!([{"step":"execute_command", "command":"review.count.Do"}]);
    let original = document.to_string();
    let suite = AdmittedSuite::from_json(&original).unwrap();
    std::fs::write(directory.join("essconform/suite.json"), original).unwrap();
    for (mode, success, failed, skipped) in [("skip", true, 0, 1), ("skip-end-error", false, 1, 0)]
    {
        let output = invoke(&directory, mode, mode, false, true);
        assert_eq!(output.status.success(), success);
        let report = CountReport::from_json(
            &std::fs::read_to_string(directory.join(format!("{mode}.report.json"))).unwrap(),
            &suite,
        )
        .unwrap();
        assert_eq!(report.counts().failed, failed);
        assert_eq!(report.counts().skipped, skipped);
    }
    let mut completed = Vec::new();
    for destination in [false, true] {
        let label = format!("abnormal-{destination}");
        let output = invoke(&directory, &label, "skip-end-goexit", false, destination);
        let report = directory.join(format!("{label}.report.json"));
        if output.status.success() || report.exists() {
            completed.push(format!(
                "destination={destination}: exit={:?}, report={}",
                output.status.code(),
                report.exists()
            ));
        }
    }
    assert!(
        completed.is_empty(),
        "an EndScenario that never returns cannot finish a report/2 invocation:\n{}",
        completed.join("\n")
    );
}

#[test]
fn generated_go_skip_counts_preserve_opaque_ids_and_strictness_without_a_destination() {
    let directory = module("opaque-ids");
    let ids = [
        "review.count.Type/invariant/at/review.count.Rows/line\none",
        "review.count.Type/invariant/at/review.count.Rows/line_one",
        "review.count.Type/invariant/at/review.count.Rows/quote\" e\u{301}",
    ];
    let mut document = document();
    document["scenarios"] = json!({});
    for id in ids {
        document["scenarios"][id] =
            json!({"purpose":"Preserve the exact opaque field position", "steps":[], "source":[]});
    }
    let original = serde_json::to_string_pretty(&document)
        .unwrap()
        .replace('\n', "\r\n")
        + "\r\n";
    let suite = AdmittedSuite::from_json(&original).unwrap();
    std::fs::write(directory.join("essconform/suite.json"), original).unwrap();
    let output = invoke(&directory, "diagnostic", "begin-skip", false, true);
    assert!(output.status.success());
    let report = std::fs::read_to_string(directory.join("diagnostic.report.json")).unwrap();
    let admitted = CountReport::from_json(&report, &suite).unwrap();
    assert_eq!(admitted.counts().total, 3);
    assert_eq!(admitted.counts().failed, 0);
    assert_eq!(admitted.counts().skipped, 3);
    let report: Value = serde_json::from_str(&report).unwrap();
    let mut ids = ids.to_vec();
    ids.sort_unstable();
    assert_eq!(report["outcomes"]["skipped"], json!(ids));
    assert_eq!(report["suite"]["digest"], suite.digest());
    assert_eq!(report["execution_status"], "inconclusive");
    assert_eq!(report["conformance_status"], "inconclusive");
    let strict = invoke(&directory, "strict", "begin-skip", true, false);
    assert!(!strict.status.success());
    assert!(String::from_utf8_lossy(&strict.stdout).contains("strict conformance"));
    assert!(!directory.join("strict.report.json").exists());
}
