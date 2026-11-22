//! `ess specify validate` on a two-file specification names the source that declares the system
//! in every format help
//! (`story:every-format-version-hint-names-the-source-that-declares-the-system`).

use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

use serde_json::Value;

const SYSTEM_AT_10: &str = "format: ess/10\nsystem: demo\nversion: v1\ndomains: [demo.jobs]\n";
const SYSTEM_AT_16: &str = "format: ess/16\nsystem: demo\nversion: v1\ndomains: [demo.jobs]\n";

const JOBS_DOMAIN: &str = "domain: demo.jobs
entities:
  - name: demo.jobs.Job
    identity: {name: job_id, type: Uuid}
    fields: []
    lifecycle:
      initial: Scheduled
      states: [Scheduled]
      terminal: [Scheduled]
events:
  - name: demo.jobs.JobScheduled
    fields: [{name: job_id, type: Uuid}]
errors:
  - {name: demo.jobs.InPast, fields: []}
commands:
  - name: demo.jobs.Schedule
    input:
      - {name: starts_at, type: Timestamp}
      - {name: label, type: String}
    outcomes:
      - name: start-in-past
        when: starts_at < now
        error: demo.jobs.InPast
      - name: label-too-long
        when: label.count > 40
        error: demo.jobs.InPast
      - name: scheduled
        creates: demo.jobs.Job
        instance: job_id
        emits: [demo.jobs.JobScheduled]
        payload: {demo.jobs.JobScheduled: {job_id: {generated: true}}}
";

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

fn two_files(system: &str, domain: &str) -> PathBuf {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let directory = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!(
        "validate-format-hint-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(directory.join("domains")).expect("a scratch directory");
    std::fs::write(directory.join("system.yaml"), system).expect("system.yaml is written");
    std::fs::write(directory.join("domains/jobs.yaml"), domain).expect("jobs.yaml is written");
    directory
}

fn validate(directory: &Path, json: bool) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_ess"));
    command.current_dir(root()).args(["specify", "validate"]);
    if json {
        command.args(["--format", "json"]);
    }
    command
        .arg("--path")
        .arg(directory)
        .output()
        .expect("the `ess` binary runs")
}

#[test]
fn the_text_report_names_the_source_in_every_format_help() {
    let directory = two_files(SYSTEM_AT_10, JOBS_DOMAIN);
    let output = validate(&directory, false);
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    for version in ["ess/16", "ess/11"] {
        let help =
            format!("  help: write `format: {version}` on the source that declares the system");
        assert!(stderr.contains(&help), "{help}\n{stderr}");
    }
    assert!(stderr.contains("--> domains/jobs.yaml:"), "{stderr}");
    assert!(!stderr.contains("ess/16; write"), "{stderr}");
}

#[test]
fn the_json_report_carries_the_hint() {
    let directory = two_files(SYSTEM_AT_10, JOBS_DOMAIN);
    let output = validate(&directory, true);
    let report: Value = serde_json::from_slice(&output.stdout).expect("standard output is JSON");
    let diagnostics = report["diagnostics"].as_array().expect("diagnostics");
    let length = diagnostics
        .iter()
        .find(|d| {
            d["message"]
                .as_str()
                .is_some_and(|m| m.starts_with("`label.count`"))
        })
        .unwrap_or_else(|| panic!("no length refusal: {report:#}"));
    assert_eq!(length["code"], "ESS-COMMAND-009");
    assert_eq!(
        length["hint"],
        "write `format: ess/11` on the source that declares the system"
    );
    assert_eq!(length["span"]["source"], "domains/jobs.yaml");
}

#[test]
fn a_header_key_on_the_domain_file_is_sent_to_the_system_file() {
    let domain = format!("format: ess/16\n{JOBS_DOMAIN}");
    let directory = two_files(SYSTEM_AT_16, &domain);
    let output = validate(&directory, false);
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("[missing_declaration] domains/jobs.yaml.system:"),
        "{stderr}"
    );
    assert!(
        stderr.contains("move these keys to `system.yaml`"),
        "{stderr}"
    );
}

#[test]
fn the_same_files_at_ess_16_are_valid() {
    let directory = two_files(SYSTEM_AT_16, JOBS_DOMAIN);
    let output = validate(&directory, false);
    assert_eq!(output.status.code(), Some(0), "{output:?}");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("demo v1 — 2 file(s), valid"), "{stdout}");
}
