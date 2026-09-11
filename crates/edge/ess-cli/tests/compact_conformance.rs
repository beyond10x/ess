//! Fresh compact artifacts preserve meaning while acquiring their own exact-byte identity.
use ess_conformance::{coverage::AdmittedInput, AdmittedSuite, ConformanceSuite, CountReport};
use serde_json::Value;
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}
fn directory(name: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!("ess-compact-{name}-{}", std::process::id()));
    fs::create_dir_all(&path).unwrap();
    path
}
fn command(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ess"))
        .args(args)
        .output()
        .unwrap()
}
fn generated(dir: &Path, version: &str, compact: bool) -> String {
    let model = root().join("examples/billing");
    let out = dir.join(if compact {
        "compact.json"
    } else {
        "pretty.json"
    });
    let mut args = vec![
        "conform",
        "synthesize",
        "--path",
        model.to_str().unwrap(),
        "--suite-format",
        version,
        "--target",
        "ir",
        "--out",
        out.to_str().unwrap(),
        "--format",
        "json",
    ];
    if compact {
        args.push("--compact");
    }
    let output = command(&args);
    assert!(output.status.success(), "{output:?}");
    assert_eq!(output.stdout, fs::read(&out).unwrap());
    String::from_utf8(output.stdout).unwrap()
}
fn assert_same_values(pretty: &str, compact: &str) {
    assert_eq!(
        serde_json::from_str::<Value>(pretty).unwrap(),
        serde_json::from_str::<Value>(compact).unwrap()
    );
    assert!(compact.len() < pretty.len());
    assert!(compact.ends_with('\n'));
    assert_eq!(compact.bytes().filter(|b| *b == b'\n').count(), 1);
    let pretty = AdmittedSuite::from_json(pretty).unwrap();
    let compact = AdmittedSuite::from_json(compact).unwrap();
    assert_eq!(pretty.suite(), compact.suite());
    assert_eq!(pretty.coverage(), compact.coverage());
    assert_ne!(pretty.digest(), compact.digest());
}

#[test]
fn ordinary_compact_is_opt_in_deterministic_and_pretty_default_is_exact() {
    let dir = directory("ordinary");
    let pretty = generated(&dir, "4", false);
    assert_eq!(
        pretty,
        ConformanceSuite::from_json(&pretty)
            .unwrap()
            .to_canonical_json()
            .unwrap()
    );
    let compact = generated(&dir, "4", true);
    assert_same_values(&pretty, &compact);
    assert_eq!(
        compact,
        format!(
            "{}\n",
            serde_json::to_string(&ConformanceSuite::from_json(&pretty).unwrap()).unwrap()
        )
    );
    assert_eq!(compact, generated(&dir, "4", true));
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn compact_coverage_preserves_inventory_and_exact_parent_lineage() {
    let dir = directory("coverage");
    let pretty = generated(&dir, "5", false);
    let compact = generated(&dir, "5", true);
    assert_same_values(&pretty, &compact);
    assert_eq!(
        compact,
        format!(
            "{}\n",
            serde_json::to_string(&serde_json::from_str::<Value>(&pretty).unwrap()).unwrap()
        )
    );
    assert_eq!(compact, generated(&dir, "5", true));
    let admitted = AdmittedInput::from_suite(AdmittedSuite::from_json(&compact).unwrap()).unwrap();
    let ids: Vec<_> = admitted
        .selected()
        .suite()
        .scenarios
        .keys()
        .take(1)
        .cloned()
        .collect();
    let child = admitted.select(&ids).unwrap();
    assert_eq!(child.document().parent_suites, vec![compact]);
    let carrier = child.document().to_canonical_json().unwrap();
    assert!(AdmittedInput::from_json(&carrier).is_ok());
    assert!(AdmittedInput::from_json(&carrier.replace(
        &serde_json::to_string(&child.document().parent_suites[0]).unwrap(),
        &serde_json::to_string(&pretty).unwrap()
    ))
    .is_err());
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn compact_reports_bind_new_bytes_and_retain_actual_execution_outcomes() {
    let dir = directory("reports");
    for version in ["4", "5"] {
        let mut results = Vec::new();
        let mut originals = Vec::new();
        let mut reports = Vec::new();
        for compact in [false, true] {
            let text = generated(&dir, version, compact);
            let suite = dir.join(if compact {
                "compact.json"
            } else {
                "pretty.json"
            });
            let report = dir.join(if compact {
                "compact-report.json"
            } else {
                "pretty-report.json"
            });
            let output = command(&[
                "conform",
                "run",
                "--suite",
                suite.to_str().unwrap(),
                "--target",
                "billing",
                "--report-format",
                "2",
                "--strict",
                "--format",
                "json",
                "--report-out",
                report.to_str().unwrap(),
            ]);
            assert_eq!(
                output.status.code(),
                Some(if version == "4" { 3 } else { 0 }),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
            results.push(serde_json::from_slice::<Value>(&output.stdout).unwrap());
            let report = fs::read_to_string(report).unwrap();
            CountReport::from_json(&report, &AdmittedSuite::from_json(&text).unwrap()).unwrap();
            originals.push(text);
            reports.push(report);
        }
        assert_eq!(
            results[0]["summary"]["counts"],
            results[1]["summary"]["counts"]
        );
        assert_eq!(
            results[0]["summary"]["conformance_status"],
            if version == "4" {
                "inconclusive"
            } else {
                "passed"
            }
        );
        assert_eq!(
            results[0]["summary"]["conformance_status"],
            results[1]["summary"]["conformance_status"]
        );
        assert_eq!(results[0]["summary"]["counts"]["passed"], 29);
        assert_eq!(results[1]["summary"]["counts"]["passed"], 29);
        for result in &mut results {
            for scenario in result["scenarios"].as_array_mut().unwrap() {
                scenario.as_object_mut().unwrap().remove("duration_ms");
            }
        }
        assert_eq!(results[0]["scenarios"], results[1]["scenarios"]);
        let error = CountReport::from_json(
            &reports[0],
            &AdmittedSuite::from_json(&originals[1]).unwrap(),
        )
        .unwrap_err();
        assert!(
            error.to_string().contains("exact suite reference mismatch"),
            "{error}"
        );
    }
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn compact_refuses_non_ir_before_source_resolution_or_output() {
    let dir = directory("refusal");
    let out = dir.join("output");
    for version in ["4", "5"] {
        let output = command(&[
            "conform",
            "synthesize",
            "--path",
            "absent-compact-source",
            "--target",
            "go",
            "--suite-format",
            version,
            "--compact",
            "--out",
            out.to_str().unwrap(),
        ]);
        assert!(!output.status.success());
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("--compact requires --target ir"),
            "{stderr}"
        );
        assert!(!out.exists());
    }
    fs::remove_dir_all(dir).unwrap();
}

#[test]
fn compact_library_writer_preserves_unicode_and_string_whitespace() {
    let dir = directory("strings");
    let pretty = generated(&dir, "4", false);
    let mut suite = ConformanceSuite::from_json(&pretty).unwrap();
    let mut changed = false;
    for scenario in suite.scenarios.values_mut() {
        for step in &mut scenario.steps {
            if let ess_conformance::ScenarioStep::ExecuteCommand { input, .. } = step {
                if let Some(template) = input.get_mut("template") {
                    *template = ess_conformance::ScenarioValue::Literal {
                        value: ess_primitives::node::Node::Text("a  b\t\n\r \" \\ Grüße 雪".into()),
                    };
                    changed = true;
                }
            }
        }
    }
    assert!(changed);
    let pretty = suite.to_canonical_json().unwrap();
    let compact = suite.to_compact_json().unwrap();
    assert_same_values(&pretty, &compact);
    assert_eq!(ConformanceSuite::from_json(&compact).unwrap(), suite);
    fs::remove_dir_all(dir).unwrap();
}
