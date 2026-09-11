//! Authored setup must survive fresh CLI version selection and explicit evidence admission.
use ess_conformance::{coverage::AdmittedInput, AdmittedSuite, CountReport};
use std::{fs, process::Command};

#[test]
fn fresh_entity_setup_selects_executable_formats_and_requires_report_two() {
    let dir = tempfile::tempdir().unwrap();
    let model = dir.path().join("model.yaml");
    let scenarios = dir.path().join("scenarios");
    fs::create_dir(&scenarios).unwrap();
    fs::write(&model, include_str!("fixtures/entity-setup-model.yaml")).unwrap();
    fs::write(
        scenarios.join("history.yaml"),
        include_str!("fixtures/entity-setup-scenario.yaml"),
    )
    .unwrap();
    for verb in ["author", "synthesize"] {
        for coverage in [false, true] {
            let suite_path = dir.path().join(format!("{verb}-{coverage}.json"));
            let mut command = Command::new(env!("CARGO_BIN_EXE_ess"));
            command
                .args(["conform", verb, "--path"])
                .arg(&model)
                .arg("--scenarios")
                .arg(&scenarios)
                .arg("--out")
                .arg(&suite_path);
            if coverage {
                command.args(["--suite-format", "5"]);
            }
            let output = command.output().unwrap();
            assert!(output.status.success(), "{output:?}");
            let original = fs::read_to_string(&suite_path).unwrap();
            let admitted = AdmittedSuite::from_json(&original).unwrap();
            assert_eq!(
                admitted.suite().provenance.suite_version.major(),
                if coverage { 7 } else { 6 }
            );
            assert!(original.contains("establish_entity"));
            let report = dir.path().join("report.json");
            fs::write(&report, "untouched\n").unwrap();
            let refused = Command::new(env!("CARGO_BIN_EXE_ess"))
                .args(["conform", "run", "--suite"])
                .arg(&suite_path)
                .args([
                    "--target",
                    "billing",
                    "--report-format",
                    "1",
                    "--report-out",
                ])
                .arg(&report)
                .output()
                .unwrap();
            assert!(!refused.status.success(), "{refused:?}");
            assert!(
                String::from_utf8_lossy(&refused.stderr).contains("report-format 2"),
                "{refused:?}"
            );
            assert_eq!(fs::read_to_string(&report).unwrap(), "untouched\n");
            let run = Command::new(env!("CARGO_BIN_EXE_ess"))
                .args(["conform", "run", "--suite"])
                .arg(&suite_path)
                .args([
                    "--target",
                    "billing",
                    "--report-format",
                    "2",
                    "--report-out",
                ])
                .arg(&report)
                .output()
                .unwrap();
            assert!(report.exists(), "{run:?}");
            // Billing has no CallRecord setup capability. A report must not call that adoption.
            let report_text = fs::read_to_string(&report).unwrap();
            CountReport::from_json(&report_text, &admitted).unwrap();
            let value: serde_json::Value = serde_json::from_str(&report_text).unwrap();
            assert_ne!(value["conformance_status"], "passed");
            if coverage {
                let selected = AdmittedInput::from_suite(admitted)
                    .unwrap()
                    .select(&[])
                    .unwrap();
                assert_eq!(
                    selected.selected().suite().provenance.suite_version.major(),
                    7
                );
                assert_eq!(selected.parents()[0].original_json(), original);
            }
        }
    }
}
