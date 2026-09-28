//! Adversarial cases against `ess verify conform import-history`
//! (`story:recorded-history-validation`).

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

const ADAPTER: &str = "crates/verify/ess-conformance/tests/fixtures/recorded/adapter.yaml";

fn import(log: &Path, output: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ess"))
        .current_dir(root())
        .args([
            "verify",
            "conform",
            "import-history",
            "--path",
            "examples/billing",
            "--log",
            &log.display().to_string(),
            "--adapter",
            ADAPTER,
            "--output",
            &output.display().to_string(),
        ])
        .output()
        .expect("the `ess` binary runs")
}

/// `--output` naming the `--log` it reads: the command reads the recorded log, then overwrites it
/// with the history and exits 0. The recorded production log, the only evidence the history was
/// derived from, is gone. The command must refuse (exit 2) and leave the log as it was.
#[test]
fn an_output_naming_the_log_is_refused_and_the_log_survives() {
    let directory = tempfile::tempdir().expect("a scratch directory");
    let log = directory.path().join("calls.jsonl");
    let recorded = concat!(
        r#"{"correlation":"5f0c8a2e-3b7d-4c11-9e2a-7d4b1c0e9f31","#,
        r#""request":{"client":"w","command":"billing.invoice.Issue","subject":"s","at_ms":1},"#,
        r#""response":{"status":"ok","outcome":"Ok","at_ms":2}}"#,
        "\n"
    );
    std::fs::write(&log, recorded).expect("written");
    let run = import(&log, &log);
    let after = std::fs::read_to_string(&log).expect("the log is still readable");
    assert_eq!(
        after,
        recorded,
        "the recorded log was overwritten (exit {:?}): {}",
        run.status.code(),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(run.status.code(), Some(2));
}
