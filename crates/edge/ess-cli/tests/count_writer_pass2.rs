//! Final bounded count-writer review through original suites and generated Go Run.
use ess_conformance::{AdmittedSuite, ConformanceSuite, CountReport};
use serde_json::{json, Value};
use std::{
    path::{Path, PathBuf},
    process::{Command, Output},
};

const ID: &str = "review.count/authored/second";

fn document(predicate: &Value) -> Value {
    json!({"provenance":{"suite_version":"ess-conformance/4", "system":"review",
        "specification_version":"v1", "spec_digest":"a".repeat(64), "contract_digest":"a".repeat(64)},
        "scenarios":{ID:{"purpose":"Final original admission and callback review", "steps":[
            {"step":"expect_view", "view":"review.count.Rows", "expectation":{"expect":"satisfies", "predicate":predicate}}
        ], "source":[]}}})
}

fn module(label: &str) -> PathBuf {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .unwrap();
    let directory = root
        .join("target/review-boundaries-8/adversary-pass-2")
        .join(format!("go-{label}-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    let suite = ConformanceSuite::from_json(&document(&json!(true)).to_string()).unwrap();
    for artifact in ess_conformance::go::emit(&suite) {
        let path = directory.join(artifact.path);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, artifact.contents).unwrap();
    }
    std::fs::write(
        directory.join("go.mod"),
        "module countreviewsecond\n\ngo 1.24\n",
    )
    .unwrap();
    std::fs::write(
        directory.join("essconform/review_test.go"),
        include_str!("fixtures/count-writer-pass2/target_test.go"),
    )
    .unwrap();
    directory
}

fn invoke(directory: &Path, label: &str, mode: &str, destination: bool) -> Output {
    let mut command = Command::new("go");
    command
        .args([
            "test",
            "-count=1",
            "-v",
            "./...",
            "-run",
            "^TestReviewSecond$",
        ])
        .current_dir(directory)
        .env("ESS_REPORT_FORMAT", "2")
        .env_remove("ESS_CONFORMANCE_STRICT")
        .env_remove("ESS_CONFORMANCE_ALLOW_INCOMPLETE")
        .env_remove("ESS_REPORT_OUT")
        .env("REVIEW_MODE", mode)
        .env("REVIEW_MARKER", directory.join(format!("{label}.marker")));
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
fn generated_go_admits_only_typed_predicate_paths_and_operator_envelopes() {
    let directory = module("leaf-admission");
    let valid = document(&json!({"ready": {"eq": true}})).to_string();
    let suite = AdmittedSuite::from_json(&valid).unwrap();
    std::fs::write(directory.join("essconform/suite.json"), valid).unwrap();
    assert!(invoke(&directory, "valid", "begin-skip", true)
        .status
        .success());
    let report = std::fs::read_to_string(directory.join("valid.report.json")).unwrap();
    assert_eq!(
        CountReport::from_json(&report, &suite)
            .unwrap()
            .counts()
            .skipped,
        1
    );

    let mut accepted = Vec::new();
    for (label, predicate) in [
        ("invalid-fact-path", json!({"ready..done": {"eq": true}})),
        (
            "unknown-constraint-operator",
            json!({"ready": {"eq": true, "future": true}}),
        ),
        ("invalid-expression-path", json!("ready..done == true")),
    ] {
        let original = document(&predicate).to_string();
        assert!(
            AdmittedSuite::from_json(&original).is_err(),
            "Rust direct-original refusal: {label}"
        );
        std::fs::write(directory.join("essconform/suite.json"), original).unwrap();
        let output = invoke(&directory, label, "begin-skip", true);
        let marker = directory.join(format!("{label}.marker")).exists();
        let report = directory.join(format!("{label}.report.json")).exists();
        if output.status.success() || marker || report {
            accepted.push(format!(
                "{label}: exit={:?}, target={marker}, report={report}",
                output.status.code()
            ));
        }
    }
    assert!(
        accepted.is_empty(),
        "invalid original predicate envelopes must refuse before target construction:\n{}",
        accepted.join("\n")
    );
}

#[test]
fn generated_go_abnormal_unsupported_error_formatting_cannot_complete() {
    let directory = module("error-formatting");
    let original = document(&json!(true)).to_string();
    let suite = AdmittedSuite::from_json(&original).unwrap();
    std::fs::write(directory.join("essconform/suite.json"), original).unwrap();
    assert!(invoke(&directory, "ordinary-skip", "begin-skip", true)
        .status
        .success());
    let text = std::fs::read_to_string(directory.join("ordinary-skip.report.json")).unwrap();
    assert_eq!(
        CountReport::from_json(&text, &suite)
            .unwrap()
            .counts()
            .skipped,
        1
    );
    let mut completed = Vec::new();
    for destination in [false, true] {
        let label = format!("format-goexit-{destination}");
        let output = invoke(&directory, &label, "begin-error-format-goexit", destination);
        assert!(directory.join(format!("{label}.marker")).exists());
        let report = directory.join(format!("{label}.report.json")).exists();
        if output.status.success() || report {
            completed.push(format!(
                "destination={destination}: exit={:?}, report={report}",
                output.status.code()
            ));
        }
    }
    assert!(
        completed.is_empty(),
        "an error formatter that exits before SkipNow cannot complete a report:\n{}",
        completed.join("\n")
    );
}
