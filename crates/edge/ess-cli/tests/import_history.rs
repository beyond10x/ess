//! `ess verify conform import-history`: a recorded log becomes an `ess-history/1` document through
//! a declared adapter, and `check-history` judges it (`story:recorded-history-validation`).
//!
//! The conversion itself is decided in `crates/verify/ess-conformance/tests/recorded_history.rs`;
//! this holds the command line to it: exit 0 and the document written, the coverage gaps on stderr,
//! exit 2 for a refused log with every missing field named on its line.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_conformance::faulty::{self, Fault};
use ess_conformance::history::{Completion, History, QualifiedName};
use ess_conformance::record::{self, Atomic};
use ess_conformance::reference::Billing;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use serde_json::json;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

fn ess(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ess"))
        .current_dir(root())
        .args(args)
        .output()
        .expect("the `ess` binary runs")
}

const ADAPTER: &str = "crates/verify/ess-conformance/tests/fixtures/recorded/adapter.yaml";

/// `examples/billing`, compiled from the files it lives in.
fn billing_model() -> EssIr {
    let base = root()
        .join("examples/billing")
        .canonicalize()
        .expect("the billing example exists");
    let mut found: Vec<PathBuf> = Vec::new();
    let mut pending = vec![base.clone()];
    while let Some(directory) = pending.pop() {
        for entry in std::fs::read_dir(&directory).expect("the example is readable") {
            let path = entry.expect("an entry").path();
            if path.is_dir() {
                pending.push(path);
            } else if path.extension().is_some_and(|it| it == "yaml") {
                found.push(path);
            }
        }
    }
    found.sort();
    let mut sources = SourceMap::new();
    let mut parsed = Vec::new();
    for path in found {
        let label = path
            .strip_prefix(&base)
            .expect("inside the example")
            .display()
            .to_string();
        let text = std::fs::read_to_string(&path).expect("readable");
        let raw = RawSpecFile::parse(&text).expect("well formed");
        sources.insert(label.clone(), text);
        parsed.push((Source::new(label), raw));
    }
    let specification = Specification::assemble(parsed).expect("billing validates");
    compile(&specification, &sources).expect("billing resolves")
}

/// `history` as a service's own log: nested request/response, epoch milliseconds, a correlation
/// id. With `returns` false, no line carries its return instant.
fn as_log(history: &History, returns: bool) -> String {
    let mut log = String::new();
    for operation in &history.operations {
        let mut response = match operation.completion {
            Completion::Returned => json!({
                "status": "ok",
                "outcome": operation.outcome.as_ref().map(QualifiedName::as_str),
                "at_ms": operation.returned_at.map(|at| 1_790_000_000_000 + at * 7),
            }),
            Completion::Indeterminate => json!({ "status": "timeout" }),
        };
        if !returns {
            response.as_object_mut().expect("an object").remove("at_ms");
        }
        let line = json!({
            "correlation": operation.operation_id.as_str(),
            "request": {
                "client": format!("worker-{}", operation.client),
                "command": operation.command.as_str(),
                "subject": operation.subject_key,
                "at_ms": 1_790_000_000_000 + operation.invoked_at * 7,
            },
            "response": response,
        });
        log.push_str(&line.to_string());
        log.push('\n');
    }
    log
}

fn recorded(model: &EssIr, fault: bool) -> History {
    let workload = faulty::lost_update_workload();
    if fault {
        record::record(model, &faulty::billing(Fault::LostUpdate), &workload, 0)
    } else {
        record::record(model, &Atomic(&Billing::new()), &workload, 0)
    }
    .expect("recorded")
}

fn import(log: &Path, output: &Path) -> Output {
    ess(&[
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
}

fn check(history: &Path) -> Output {
    ess(&[
        "verify",
        "conform",
        "check-history",
        "--path",
        "examples/billing",
        "--history",
        &history.display().to_string(),
    ])
}

#[test]
fn an_imported_lost_update_log_checks_as_a_violation_and_the_unfaulted_one_passes() {
    let model = billing_model();
    let directory = tempfile::tempdir().expect("a scratch directory");
    for (fault, verdict) in [(true, 1), (false, 0)] {
        let log = directory.path().join(format!("calls-{fault}.jsonl"));
        let history = directory.path().join(format!("history-{fault}.json"));
        std::fs::write(&log, as_log(&recorded(&model, fault), true)).expect("written");
        let imported = import(&log, &history);
        let stderr = String::from_utf8_lossy(&imported.stderr);
        assert_eq!(imported.status.code(), Some(0), "{stderr}");
        assert!(
            stderr.contains("coverage-gap: `seed` not carried"),
            "{stderr}"
        );
        let gaps: serde_json::Value = serde_json::from_slice(
            &std::fs::read(
                directory
                    .path()
                    .join(format!("history-{fault}.json.gaps.json")),
            )
            .expect("the gaps are written beside the history"),
        )
        .expect("the gaps are JSON");
        assert_eq!(
            gaps,
            json!([{
                "line": null,
                "field": "seed",
                "written": "a recorded log is drawn from no seed; written 0",
            }])
        );
        let checked = check(&history);
        assert_eq!(
            checked.status.code(),
            Some(verdict),
            "fault {fault}: {}{}",
            String::from_utf8_lossy(&checked.stdout),
            String::from_utf8_lossy(&checked.stderr)
        );
    }
}

#[test]
fn a_log_missing_return_instants_is_refused_with_exit_2_naming_each_operation() {
    let model = billing_model();
    let directory = tempfile::tempdir().expect("a scratch directory");
    let log = directory.path().join("calls.jsonl");
    let history = directory.path().join("history.json");
    std::fs::write(&log, as_log(&recorded(&model, true), false)).expect("written");
    let refused = import(&log, &history);
    let stderr = String::from_utf8_lossy(&refused.stderr);
    assert_eq!(refused.status.code(), Some(2), "{stderr}");
    assert!(stderr.contains("import.missing-fields"), "{stderr}");
    for line in 1..=4 {
        assert!(
            stderr.contains(&format!(
                "line {line} (operation 00000000-0000-4000-8000-{line:012x}): `returned_at` missing"
            )),
            "{stderr}"
        );
    }
    assert!(!history.exists(), "a refused log writes no history");
}
