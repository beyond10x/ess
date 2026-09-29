//! Adversary pass 1 for beyond10x/ess#216 at the command line: the declared-coverage path
//! (`--suite-format 5`) runs a caller-reading specification under `--target interpreted`, and a
//! suite from one caller-reading model is refused against a model that differs only in a caller
//! source or only in an actor attribute.
#![allow(clippy::missing_panics_doc)]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const TEMPLATE: &str = "format: ess/16
system: demo
version: v1
summary: A note records the caller that wrote it.
domain: demo.notes
types:
  - {name: demo.notes.NoteId, kind: newtype, of: Uuid}
  - {name: demo.notes.AuthorId, kind: newtype, of: Uuid}
entities:
  - name: demo.notes.Note
    identity: {name: note_id, type: demo.notes.NoteId}
    fields:
      - {name: author_id, type: demo.notes.AuthorId}
      - {name: text, type: String}
    lifecycle: {initial: Open, states: [Open], terminal: [Open], transitions: []}
actors:
  - name: demo.notes.Writer
    attributes:
      - {name: author_id, type: demo.notes.AuthorId}
      - {name: delegate_id, type: demo.notes.AuthorId}
{EXTRA}    may: [demo.notes.WriteNote]
commands:
  - name: demo.notes.WriteNote
    input:
      - {name: text, type: String}
    outcomes:
      - name: written
        creates: demo.notes.Note
        instance: note_id
        sets: {author_id: {caller: author_id}, text: input.text}
        emits: [demo.notes.NoteWritten]
        payload:
          demo.notes.NoteWritten: {note_id: {generated: true}, written_by: {caller: {ATTRIBUTE}}, text: input.text}
events:
  - name: demo.notes.NoteWritten
    fields:
      - {name: note_id, type: demo.notes.NoteId}
      - {name: written_by, type: demo.notes.AuthorId}
      - {name: text, type: String}
views:
  - name: demo.notes.NoteDetails
    source: demo.notes.Note
    consistency: read_your_writes
    fields:
      - {name: note_id, type: demo.notes.NoteId}
      - {name: author_id, type: demo.notes.AuthorId}
      - {name: text, type: String}
      - {name: state, type: demo.notes.Note.State}
";

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

fn model(attribute: &str, extra: &str) -> tempfile::TempDir {
    let directory = tempfile::tempdir().expect("a scratch directory");
    let text = TEMPLATE
        .replace("{ATTRIBUTE}", attribute)
        .replace("{EXTRA}", extra);
    std::fs::write(directory.path().join("system.yaml"), text).expect("the model is written");
    directory
}

fn ess(args: &[&std::ffi::OsStr]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ess"))
        .current_dir(root())
        .args(args)
        .output()
        .expect("the `ess` binary runs")
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

/// `--suite-format 5 --report-format 2`: the declared-coverage suite synthesized in the run is the
/// model interpreted.
#[test]
fn the_declared_coverage_run_interprets_a_caller_specification() {
    let spec = model("author_id", "");
    let output = ess(&[
        "verify".as_ref(),
        "conform".as_ref(),
        "run".as_ref(),
        "--target".as_ref(),
        "interpreted".as_ref(),
        "--suite-format".as_ref(),
        "5".as_ref(),
        "--report-format".as_ref(),
        "2".as_ref(),
        "--format".as_ref(),
        "json".as_ref(),
        "--path".as_ref(),
        spec.path().as_os_str(),
    ]);
    let stderr = text(&output.stderr);
    assert!(
        !stderr.contains("is not the suite's spec_digest"),
        "exit {:?}: {stderr}",
        output.status.code()
    );
    let report: serde_json::Value = serde_json::from_slice(&output.stdout)
        .unwrap_or_else(|error| panic!("a report is rendered ({error}): {stderr}"));
    assert!(
        report["scenarios"]
            .as_array()
            .is_some_and(|scenarios| !scenarios.is_empty()),
        "{report:#}"
    );
}

/// A committed declared-coverage suite of the caller model runs against that model.
#[test]
fn a_written_declared_coverage_suite_runs_against_its_caller_model() {
    let spec = model("author_id", "");
    let suite = spec.path().join("suite5.json");
    let written = ess(&[
        "verify".as_ref(),
        "conform".as_ref(),
        "synthesize".as_ref(),
        "--target".as_ref(),
        "ir".as_ref(),
        "--suite-format".as_ref(),
        "5".as_ref(),
        "--path".as_ref(),
        spec.path().as_os_str(),
        "--out".as_ref(),
        suite.as_os_str(),
    ]);
    assert!(written.status.success(), "{}", text(&written.stderr));
    let output = ess(&[
        "verify".as_ref(),
        "conform".as_ref(),
        "run".as_ref(),
        "--target".as_ref(),
        "interpreted".as_ref(),
        "--report-format".as_ref(),
        "2".as_ref(),
        "--format".as_ref(),
        "json".as_ref(),
        "--path".as_ref(),
        spec.path().as_os_str(),
        "--suite".as_ref(),
        suite.as_os_str(),
    ]);
    let stderr = text(&output.stderr);
    assert!(
        !stderr.contains("is not the suite's spec_digest"),
        "exit {:?}: {stderr}",
        output.status.code()
    );
}

/// A suite from the base model is refused against a model that differs only in which caller
/// attribute the event reads, and against one that differs only in an unread actor attribute.
#[test]
fn a_suite_is_refused_against_a_model_differing_only_in_caller_data() {
    let base = model("author_id", "");
    let suite = base.path().join("suite.json");
    let written = ess(&[
        "verify".as_ref(),
        "conform".as_ref(),
        "synthesize".as_ref(),
        "--target".as_ref(),
        "ir".as_ref(),
        "--path".as_ref(),
        base.path().as_os_str(),
        "--out".as_ref(),
        suite.as_os_str(),
    ]);
    assert!(written.status.success(), "{}", text(&written.stderr));
    for (what, other) in [
        ("a different caller source", model("delegate_id", "")),
        (
            "one more actor attribute",
            model(
                "author_id",
                "      - {name: team_id, type: demo.notes.AuthorId}\n",
            ),
        ),
    ] {
        let output = ess(&[
            "verify".as_ref(),
            "conform".as_ref(),
            "run".as_ref(),
            "--target".as_ref(),
            "interpreted".as_ref(),
            "--format".as_ref(),
            "json".as_ref(),
            "--path".as_ref(),
            other.path().as_os_str(),
            "--suite".as_ref(),
            suite.as_os_str(),
        ]);
        let stderr = text(&output.stderr);
        assert!(
            stderr.contains("is not the suite's spec_digest"),
            "{what}: exit {:?}\nstdout: {}\nstderr: {stderr}",
            output.status.code(),
            text(&output.stdout)
        );
        assert_eq!(output.status.code(), Some(1), "{what}: {stderr}");
    }
}
