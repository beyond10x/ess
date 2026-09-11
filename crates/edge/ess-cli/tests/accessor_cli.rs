//! CLI boundaries for newly generated accessor vocabulary and exact report lineage.
use ess_conformance::{coverage::AdmittedInput, AdmittedSuite, CountReport};
use serde_json::Value;
use std::{
    fs,
    path::Path,
    process::{Command, Output},
};

fn command(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ess"))
        .args(args)
        .output()
        .unwrap()
}

fn generated(dir: &Path, coverage: bool) -> (std::path::PathBuf, AdmittedSuite) {
    let model = dir.join("system.yaml");
    fs::write(&model, include_str!("fixtures/bounded-accessor.yaml")).unwrap();
    let suite = dir.join("suite.json");
    let mut args = vec![
        "conform",
        "synthesize",
        "--path",
        model.to_str().unwrap(),
        "--out",
        suite.to_str().unwrap(),
        "--format",
        "json",
    ];
    if coverage {
        args.extend(["--suite-format", "5"]);
    }
    let output = command(&args);
    // Coverage records the fixture's separate, unforceable binding failure policy as a refusal.
    // Ordinary generation keeps its historical diagnostic exit0; coverage returns exit1.
    assert_eq!(
        output.status.code(),
        Some(i32::from(coverage)),
        "{output:?}"
    );
    assert_eq!(output.stdout, fs::read(&suite).unwrap());
    let text = fs::read_to_string(&suite).unwrap();
    let value: Value = serde_json::from_str(&text).unwrap();
    if coverage {
        assert_eq!(value["coverage"]["refused"].as_array().unwrap().len(), 1);
        assert_eq!(value["coverage"]["refused"][0]["code"], "ESS-SYNTH-010");
    }
    assert_eq!(
        value["provenance"]["suite_version"],
        if coverage {
            "ess-conformance/7"
        } else {
            "ess-conformance/6"
        }
    );
    assert!(
        text.contains("observed_accessor"),
        "the fixture must exercise actual accessor vocabulary: {text}"
    );
    (suite, AdmittedSuite::from_json(&text).unwrap())
}

#[test]
fn accessor_suites_require_report_two_even_without_an_output_destination() {
    for coverage in [false, true] {
        let dir = tempfile::tempdir().unwrap();
        let (suite, _) = generated(dir.path(), coverage);
        let report = dir.path().join("report.json");
        fs::write(&report, "untouched\n").unwrap();
        for destination in [false, true] {
            for extra in [
                vec![],
                vec!["--allow-incomplete"],
                vec!["--report-format", "1"],
            ] {
                let mut args = vec![
                    "conform",
                    "run",
                    "--suite",
                    suite.to_str().unwrap(),
                    "--target",
                    "billing",
                ];
                if destination {
                    args.extend(["--report-out", report.to_str().unwrap()]);
                }
                args.extend(extra);
                let output = command(&args);
                assert!(!output.status.success(), "{output:?}");
                assert!(output.stdout.is_empty(), "{output:?}");
                assert!(
                    String::from_utf8_lossy(&output.stderr).contains("report-format 2"),
                    "{output:?}"
                );
                assert_eq!(fs::read_to_string(&report).unwrap(), "untouched\n");
            }
        }
    }
}

#[test]
fn accessor_report_two_binds_the_exact_generated_suite_bytes() {
    for coverage in [false, true] {
        let dir = tempfile::tempdir().unwrap();
        let (suite, admitted) = generated(dir.path(), coverage);
        let report = dir.path().join("report.json");
        let output = command(&[
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
            "--report-out",
            report.to_str().unwrap(),
        ]);
        // Billing is intentionally unrelated: its run is evidence of report plumbing, not accessor adoption.
        let detailed: Value =
            serde_json::from_slice(&output.stdout).unwrap_or_else(|e| panic!("{e}: {output:?}"));
        assert_ne!(detailed["summary"]["conformance_status"], "passed");
        let original_report = fs::read_to_string(&report).unwrap();
        CountReport::from_json(&original_report, &admitted).unwrap();
        let original: Value = serde_json::from_str(&fs::read_to_string(&suite).unwrap()).unwrap();
        let compact = AdmittedSuite::from_json(&serde_json::to_string(&original).unwrap()).unwrap();
        assert_ne!(admitted.digest(), compact.digest());
        assert!(
            CountReport::from_json(&original_report, &compact).is_err(),
            "equal parsed content does not authorize a stale exact-byte report"
        );
    }
}

#[test]
fn accessor_coverage_selection_keeps_version_seven_and_exact_parent_bytes() {
    let dir = tempfile::tempdir().unwrap();
    let (suite, _) = generated(dir.path(), true);
    let ids = dir.path().join("ids.json");
    fs::write(&ids, "[]\n").unwrap();
    let carrier = dir.path().join("selected.json");
    let output = command(&[
        "conform",
        "select",
        "--suite",
        suite.to_str().unwrap(),
        "--ids",
        ids.to_str().unwrap(),
        "--out",
        carrier.to_str().unwrap(),
    ]);
    assert!(output.status.success(), "{output:?}");
    let input = AdmittedInput::from_json(&fs::read_to_string(&carrier).unwrap()).unwrap();
    assert_eq!(
        input.parents()[0].original_json(),
        fs::read_to_string(&suite).unwrap()
    );
    assert_eq!(input.selected().suite().provenance.suite_version.major(), 7);
    assert!(input.selected().suite().scenarios.is_empty());
    assert!(input.selected().coverage().unwrap().counts.outside > 0);
    let orphan = dir.path().join("orphan.json");
    fs::write(&orphan, input.selected().original_json()).unwrap();
    let refused = command(&[
        "conform",
        "run",
        "--suite",
        orphan.to_str().unwrap(),
        "--target",
        "billing",
        "--report-format",
        "2",
        "--format",
        "json",
    ]);
    assert!(!refused.status.success(), "{refused:?}");
    assert!(refused.stdout.is_empty(), "{refused:?}");
    let run = command(&[
        "conform",
        "run",
        "--suite-input",
        carrier.to_str().unwrap(),
        "--target",
        "billing",
        "--report-format",
        "2",
        "--strict",
        "--format",
        "json",
    ]);
    assert_eq!(run.status.code(), Some(3), "{run:?}");
    let detailed: Value = serde_json::from_slice(&run.stdout).unwrap();
    assert_eq!(detailed["summary"]["counts"]["total"], 0);
    assert_ne!(detailed["summary"]["conformance_status"], "passed");
}
