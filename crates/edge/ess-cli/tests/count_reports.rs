//! Explicit count-report dispatch preserves the legacy default and never claims coverage.
use std::{path::Path, process::Command};

#[test]
fn count_report_opt_in_has_a_distinct_detailed_surface_and_unknown_coverage() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let mut command = Command::new(env!("CARGO_BIN_EXE_ess"));
    command
        .args(["conform", "run", "--path"])
        .arg(root.join("examples/billing"))
        .args([
            "--target",
            "billing",
            "--report-format",
            "2",
            "--format",
            "json",
        ]);
    let diagnostic = command.output().unwrap();
    assert!(
        diagnostic.status.success(),
        "{}",
        String::from_utf8_lossy(&diagnostic.stderr)
    );
    let detailed: serde_json::Value = serde_json::from_slice(&diagnostic.stdout).unwrap();
    assert_eq!(detailed["format"], "ess-conformance-run/2");
    assert_eq!(detailed["summary"]["execution_status"], "passed");
    assert_eq!(
        detailed["summary"]["coverage"],
        serde_json::json!({"knowledge":"unknown"})
    );
    assert_eq!(detailed["summary"]["conformance_status"], "inconclusive");
    let strict = command.arg("--strict").output().unwrap();
    assert_eq!(strict.status.code(), Some(3));
}

fn suite_files(label: &str) -> std::path::PathBuf {
    let directory =
        std::env::temp_dir().join(format!("ess-count-cli-{label}-{}", std::process::id()));
    std::fs::create_dir_all(&directory).unwrap();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let output = Command::new(env!("CARGO_BIN_EXE_ess"))
        .args(["conform", "synthesize", "--path"])
        .arg(root.join("examples/billing"))
        .arg("--out")
        .arg(directory.join("suite.json"))
        .output()
        .unwrap();
    assert!(output.status.success());
    directory
}
fn run_suite(directory: &Path, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_ess"))
        .args(["conform", "run", "--suite"])
        .arg(directory.join("suite.json"))
        .args(["--target", "billing"])
        .args(args)
        .output()
        .unwrap()
}
#[test]
fn count_cli_preserves_default_bytes_and_standalone_detailed_pairing() {
    let directory = suite_files("surfaces");
    for format in ["json", "yaml"] {
        let default = run_suite(&directory, &["--format", format]);
        let explicit = run_suite(
            &directory,
            &[
                "--format",
                format,
                "--report-format",
                "1",
                "--allow-incomplete",
            ],
        );
        assert!(default.status.success() && explicit.status.success());
        assert_eq!(default.stdout, explicit.stdout);
        let destination = directory.join(format!("report-{format}.json"));
        let result = run_suite(
            &directory,
            &[
                "--format",
                format,
                "--report-format",
                "2",
                "--report-out",
                destination.to_str().unwrap(),
            ],
        );
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        let text = std::fs::read_to_string(destination).unwrap();
        let suite = ess_conformance::AdmittedSuite::from_json(
            &std::fs::read_to_string(directory.join("suite.json")).unwrap(),
        )
        .unwrap();
        let report = ess_conformance::CountReport::from_json(&text, &suite).unwrap();
        assert_eq!(report.counts().passed, 29);
        assert_eq!(
            report.conformance_status(),
            ess_conformance::CountStatus::Inconclusive
        );
        let detailed: serde_json::Value = if format == "json" {
            serde_json::from_slice(&result.stdout).unwrap()
        } else {
            serde_yaml::from_slice(&result.stdout).unwrap()
        };
        assert_eq!(detailed.as_object().unwrap().len(), 4);
        assert_eq!(detailed["format"], "ess-conformance-run/2");
        assert_eq!(
            detailed["summary"],
            serde_json::from_str::<serde_json::Value>(&text).unwrap()
        );
    }
}
#[test]
fn count_cli_configuration_and_original_suite_refusals_preserve_destinations() {
    let directory = suite_files("refusals");
    let destination = directory.join("report.json");
    std::fs::write(&destination, "untouched\n").unwrap();
    for args in [
        vec!["--strict"],
        vec!["--report-format", "3"],
        vec!["--report-format", "2", "--strict", "--allow-incomplete"],
    ] {
        let mut args = args;
        args.extend(["--report-out", destination.to_str().unwrap()]);
        let output = run_suite(&directory, &args);
        assert!(!output.status.success());
        assert!(output.stdout.is_empty());
        assert_eq!(
            std::fs::read_to_string(&destination).unwrap(),
            "untouched\n"
        );
    }
    let suite = std::fs::read_to_string(directory.join("suite.json")).unwrap();
    for bad in [
        suite.replace("ess-conformance/4", "ess-conformance/5"),
        suite.replace("\"provenance\": {", "\"provenance\": {\"future\": 1,"),
        suite.replace(
            "\"scenarios\": {",
            "\"scenarios\": {}, \"scen\\u0061rios\": {",
        ),
    ] {
        std::fs::write(directory.join("suite.json"), bad).unwrap();
        for args in [
            vec![],
            vec!["--report-format", "2"],
            vec!["--report-format", "1", "--allow-incomplete"],
        ] {
            let mut args = args;
            args.extend(["--report-out", destination.to_str().unwrap()]);
            let output = run_suite(&directory, &args);
            assert!(!output.status.success());
            assert!(output.stdout.is_empty());
            assert_eq!(
                std::fs::read_to_string(&destination).unwrap(),
                "untouched\n"
            );
        }
    }
}
