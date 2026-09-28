//! Second adversarial pass against `ess verify conform import-history`
//! (`story:recorded-history-validation`): the output-clash refusal and the gaps file.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

const ADAPTER: &str = "crates/verify/ess-conformance/tests/fixtures/recorded/adapter.yaml";

const RECORDED: &str = concat!(
    r#"{"correlation":"5f0c8a2e-3b7d-4c11-9e2a-7d4b1c0e9f31","#,
    r#""request":{"client":"w","command":"billing.invoice.Issue","subject":"s","at_ms":1},"#,
    r#""response":{"status":"ok","outcome":"Ok","at_ms":2}}"#,
    "\n"
);

fn import_with(spec: &Path, log: &Path, adapter: &Path, output: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ess"))
        .current_dir(root())
        .args([
            "verify",
            "conform",
            "import-history",
            "--path",
            &spec.display().to_string(),
            "--log",
            &log.display().to_string(),
            "--adapter",
            &adapter.display().to_string(),
            "--output",
            &output.display().to_string(),
        ])
        .output()
        .expect("the `ess` binary runs")
}

fn import(log: &Path, output: &Path) -> Output {
    import_with(
        Path::new("examples/billing"),
        log,
        Path::new(ADAPTER),
        output,
    )
}

/// `--output` a hard link to `--log`. The clash check compares canonical paths, which a hard link
/// does not share, so the command reads the log, truncates the one inode both names point at, and
/// writes the history into it: the recorded log is gone and the exit is 0.
#[test]
fn an_output_hard_linked_to_the_log_is_refused_and_the_log_survives() {
    let directory = tempfile::tempdir().expect("a scratch directory");
    let log = directory.path().join("calls.jsonl");
    std::fs::write(&log, RECORDED).expect("written");
    let output = directory.path().join("history.json");
    std::fs::hard_link(&log, &output).expect("a hard link");
    let run = import(&log, &output);
    let after = std::fs::read_to_string(&log).expect("the log is still readable");
    assert_eq!(
        after,
        RECORDED,
        "the recorded log was overwritten through its hard link (exit {:?}): {}",
        run.status.code(),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(run.status.code(), Some(2));
}

/// The gaps file `<output>.gaps.json` a hard link to `--adapter`: the adapter is overwritten with
/// the gaps array and the exit is 0.
#[test]
fn a_gaps_file_hard_linked_to_the_adapter_is_refused_and_the_adapter_survives() {
    let directory = tempfile::tempdir().expect("a scratch directory");
    let log = directory.path().join("calls.jsonl");
    std::fs::write(&log, RECORDED).expect("written");
    let declared = std::fs::read_to_string(root().join(ADAPTER)).expect("the adapter fixture");
    let adapter = directory.path().join("adapter.yaml");
    std::fs::write(&adapter, &declared).expect("written");
    let output = directory.path().join("history.json");
    std::fs::hard_link(&adapter, directory.path().join("history.json.gaps.json"))
        .expect("a hard link");
    let run = import_with(Path::new("examples/billing"), &log, &adapter, &output);
    let after = std::fs::read_to_string(&adapter).expect("the adapter is still readable");
    assert_eq!(
        after,
        declared,
        "the adapter was overwritten through its hard link (exit {:?}): {}",
        run.status.code(),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(run.status.code(), Some(2));
}

fn copy_tree(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).expect("a directory");
    for entry in std::fs::read_dir(from).expect("readable") {
        let path = entry.expect("an entry").path();
        let target = to.join(path.file_name().expect("a name"));
        if path.is_dir() {
            copy_tree(&path, &target);
        } else {
            std::fs::copy(&path, &target).expect("copied");
        }
    }
}

/// `--output` naming the specification's own `system.yaml`. The clash check covers `--log` and
/// `--adapter` only; the third input the command reads is overwritten with the history, exit 0,
/// and the specification the history was recorded against no longer exists.
#[test]
fn an_output_naming_the_specification_is_refused_and_the_specification_survives() {
    let directory = tempfile::tempdir().expect("a scratch directory");
    let spec = directory.path().join("billing");
    copy_tree(&root().join("examples/billing"), &spec);
    let system = spec.join("system.yaml");
    let before = std::fs::read_to_string(&system).expect("the system file");
    let log = directory.path().join("calls.jsonl");
    std::fs::write(&log, RECORDED).expect("written");
    let run = import_with(&spec, &log, Path::new(ADAPTER), &system);
    let after = std::fs::read_to_string(&system).expect("still readable");
    assert_eq!(
        after,
        before,
        "the specification was overwritten (exit {:?}): {}",
        run.status.code(),
        String::from_utf8_lossy(&run.stderr)
    );
    assert_eq!(run.status.code(), Some(2));
}

/// The gaps file cannot be written (its path is a directory). The command exits 2 after it has
/// already replaced `--output` with the new history, so an exit 2 leaves a history on disk with no
/// gaps record beside it, the one record the documentation says records which values were carried.
#[test]
fn an_unwritable_gaps_file_leaves_no_history_written() {
    let directory = tempfile::tempdir().expect("a scratch directory");
    let log = directory.path().join("calls.jsonl");
    std::fs::write(&log, RECORDED).expect("written");
    let output = directory.path().join("history.json");
    std::fs::create_dir(directory.path().join("history.json.gaps.json")).expect("a directory");
    let run = import(&log, &output);
    assert_eq!(
        run.status.code(),
        Some(2),
        "{}",
        String::from_utf8_lossy(&run.stderr)
    );
    assert!(
        !output.exists(),
        "exit 2, and a history without its gaps record was written anyway"
    );
}
