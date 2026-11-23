//! Explicit count-report dispatch preserves the legacy default and never claims coverage.
use ess_cli::TemporaryDirectory;
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

fn suite_files(label: &str) -> TemporaryDirectory {
    let directory = TemporaryDirectory::create(&format!("ess-count-cli-{label}")).unwrap();
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
        // The default is report/1. A fresh suite is ess-conformance/34 (#312,
        // docs/design/scenario-initial-state-and-cross-caller-witnesses.md), which report/1 is
        // refused for before the target runs, so the default and explicit report/1 refuse alike.
        assert!(!default.status.success() && !explicit.status.success());
        assert_eq!(default.stdout.len(), 0);
        assert_eq!(default.stdout, explicit.stdout);
        assert_eq!(default.stderr, explicit.stderr);
        assert!(
            String::from_utf8_lossy(&default.stderr)
                .contains("require explicit --report-format 2 before execution"),
            "{default:?}"
        );
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
        assert_eq!(report.counts().passed, 32);
        assert_eq!(
            report.conformance_status(),
            ess_conformance::CountStatus::Inconclusive
        );
        let detailed: serde_json::Value = if format == "json" {
            serde_json::from_slice(&result.stdout).unwrap()
        } else {
            ess_yaml::from_slice(&result.stdout).unwrap()
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
        assert_eq!(output.stdout.len(), 0);
        assert_eq!(
            std::fs::read_to_string(&destination).unwrap(),
            "untouched\n"
        );
    }
    let suite = std::fs::read_to_string(directory.join("suite.json")).unwrap();
    for bad in [
        suite.replace("ess-conformance/34", "ess-conformance/35"),
        suite.replace("\"provenance\": {", "\"provenance\": {\"future\": 1,"),
        suite.replace(
            "\"scenarios\": {",
            "\"scenarios\": {}, \"scen\\u0061rios\": {",
        ),
    ] {
        assert_ne!(bad, suite, "each refusal case changes the suite it runs");
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
            assert_eq!(output.stdout.len(), 0);
            assert_eq!(
                std::fs::read_to_string(&destination).unwrap(),
                "untouched\n"
            );
        }
    }
}

fn entries(directory: &Path) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(directory)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

#[test]
fn a_report_replaces_an_earlier_report_with_its_canonical_bytes_and_leaves_nothing_beside_it() {
    let directory = suite_files("replace");
    let report = directory.join("report.json");
    std::fs::write(&report, "earlier\n").unwrap();
    let output = run_suite(
        &directory,
        &[
            "--report-format",
            "2",
            "--report-out",
            report.to_str().unwrap(),
        ],
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let text = std::fs::read_to_string(&report).unwrap();
    assert_ne!(text, "earlier\n");
    let admitted = ess_conformance::AdmittedSuite::from_json(
        &std::fs::read_to_string(directory.join("suite.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(
        ess_conformance::CountReport::from_json(&text, &admitted)
            .unwrap()
            .to_canonical_json()
            .unwrap(),
        text
    );
    assert_eq!(entries(&directory), ["report.json", "suite.json"]);
}

#[test]
fn a_report_onto_a_directory_fails_and_leaves_nothing_beside_it() {
    let directory = suite_files("directory");
    let report = directory.join("report.json");
    std::fs::create_dir(&report).unwrap();
    std::fs::write(report.join("keep.txt"), "keep\n").unwrap();
    let output = run_suite(
        &directory,
        &[
            "--report-format",
            "2",
            "--report-out",
            report.to_str().unwrap(),
        ],
    );
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(output.stdout, Vec::<u8>::new());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains(&format!("writing {}", report.display())),
        "{stderr}"
    );
    assert_eq!(
        std::fs::read_to_string(report.join("keep.txt")).unwrap(),
        "keep\n"
    );
    assert_eq!(entries(&directory), ["report.json", "suite.json"]);
}

/// `--report-out /dev/stdout` wrote the report to standard output on 0.97.4 (`fs::write` opens
/// the path). Standard output is a pipe here, as under `| jq`.
#[cfg(target_os = "linux")]
#[test]
fn a_report_out_of_dev_stdout_writes_the_report_to_standard_output() {
    let directory = suite_files("dev-stdout");
    let output = run_suite(
        &directory,
        &["--report-format", "2", "--report-out", "/dev/stdout"],
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("\"conformance_status\""));
}

/// `--report-out /dev/null` discarded the report on 0.97.4. The helper creates its temporary
/// file in the destination's directory, so a writer that may not create files in `/dev` fails,
/// and one that may (root in a container) renames a regular file over the device.
#[cfg(target_os = "linux")]
#[test]
fn a_report_out_of_dev_null_succeeds() {
    let directory = suite_files("dev-null");
    let output = run_suite(
        &directory,
        &["--report-format", "2", "--report-out", "/dev/null"],
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

/// A destination that is not a regular file and not a directory is not replaced by a regular
/// file: on 0.97.4 the write to a socket failed and the socket stayed.
#[cfg(unix)]
#[test]
fn a_report_out_of_a_non_regular_file_does_not_replace_it() {
    use std::os::unix::fs::FileTypeExt;
    let directory = suite_files("socket");
    let socket = directory.join("report.sock");
    let _listener = std::os::unix::net::UnixListener::bind(&socket).unwrap();
    let _ = run_suite(
        &directory,
        &[
            "--report-format",
            "2",
            "--report-out",
            socket.to_str().unwrap(),
        ],
    );
    let kind = std::fs::symlink_metadata(&socket).unwrap().file_type();
    assert!(
        kind.is_socket(),
        "the socket at the destination was replaced: is_file={}",
        kind.is_file()
    );
}
