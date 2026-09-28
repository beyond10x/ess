//! `ess verify conform check-history`: a recorded history's verdict is the exit status.
//!
//! `story:linearizability-checker-over-the-interpreter`: exit 0 linearizable, 1 violation, 3 unknown.
//! A history the reader or the checker refuses is neither, and exits 2. The verdicts themselves are
//! decided in `crates/verify/ess-conformance/tests/linearizability.rs`; this holds the command line to
//! them.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_conformance::faulty::{self, Fault};
use ess_conformance::history::History;
use ess_conformance::record::{self, Atomic};
use ess_conformance::reference::Billing;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

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

const REGISTER: &str = "crates/verify/ess-conformance/tests/fixtures/register/register.yaml";

fn register(history: &str) -> String {
    format!("crates/verify/ess-conformance/tests/fixtures/register/{history}")
}

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

/// The seed whose interleaving overlaps the two payments of the `LostUpdate` workload.
const OVERLAPPING: u64 = 0;

fn written(directory: &Path, name: &str, history: &History) -> String {
    let path = directory.join(name);
    std::fs::write(
        &path,
        serde_json::to_vec(history).expect("a history serializes"),
    )
    .expect("the history is written");
    path.display().to_string()
}

fn code(output: &Output) -> Option<i32> {
    output.status.code()
}

#[test]
fn a_linearizable_register_history_exits_0() {
    let output = ess(&[
        "verify",
        "conform",
        "check-history",
        "--path",
        REGISTER,
        "--history",
        &register("linearizable.json"),
    ]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(
        code(&output),
        Some(0),
        "{stdout}{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(stdout.starts_with("verdict: Linearizable\n"), "{stdout}");
}

#[test]
fn a_register_history_no_order_explains_exits_1_and_reports_its_shrunk_history_as_json() {
    let output = ess(&[
        "verify",
        "conform",
        "check-history",
        "--path",
        REGISTER,
        "--history",
        &register("not-linearizable.json"),
        "--format",
        "json",
    ]);
    assert_eq!(
        code(&output),
        Some(1),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("the report is JSON");
    assert_eq!(report["verdict"], "Violation");
    assert!(report["linearization"].is_array());
    assert_eq!(report["shrunk"]["format"], "ess-history/1");
}

#[test]
fn lost_update_exits_1_the_reference_exits_0_and_a_spent_budget_exits_3() {
    let model = billing_model();
    let directory = tempfile::tempdir().expect("a scratch directory");
    let faulted = record::record(
        &model,
        &faulty::billing(Fault::LostUpdate),
        &faulty::lost_update_workload(),
        OVERLAPPING,
    )
    .expect("recorded");
    let reference = Billing::new();
    let unfaulted = record::record(
        &model,
        &Atomic(&reference),
        &faulty::lost_update_workload(),
        OVERLAPPING,
    )
    .expect("recorded");
    let faulted = written(directory.path(), "lost-update.json", &faulted);
    let unfaulted = written(directory.path(), "reference.json", &unfaulted);

    let check = |history: &str, extra: &[&str]| {
        let mut args = vec![
            "verify",
            "conform",
            "check-history",
            "--path",
            "examples/billing",
            "--history",
            history,
        ];
        args.extend_from_slice(extra);
        ess(&args)
    };
    let violation = check(&faulted, &[]);
    assert_eq!(
        code(&violation),
        Some(1),
        "{}{}",
        String::from_utf8_lossy(&violation.stdout),
        String::from_utf8_lossy(&violation.stderr)
    );
    let text = String::from_utf8_lossy(&violation.stdout);
    assert!(text.contains("verdict: Violation"), "{text}");
    assert!(text.contains("shrunk history:"), "{text}");
    assert!(text.contains("longest partial linearization"), "{text}");

    assert_eq!(code(&check(&unfaulted, &[])), Some(0));
    let unknown = check(&unfaulted, &["--budget", "1"]);
    assert_eq!(code(&unknown), Some(3));
    assert!(String::from_utf8_lossy(&unknown.stdout).contains("verdict: Unknown"));

    // One history and one budget, one report, byte for byte.
    let again = check(&faulted, &["--format", "json"]);
    let once_more = check(&faulted, &["--format", "json"]);
    assert_eq!(again.stdout, once_more.stdout);
}

#[test]
fn a_history_recorded_against_another_specification_is_refused_with_exit_2() {
    let output = ess(&[
        "verify",
        "conform",
        "check-history",
        "--path",
        "examples/billing",
        "--history",
        &register("linearizable.json"),
    ]);
    assert_eq!(code(&output), Some(2));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("history.spec-digest-mismatch"), "{stderr}");
}
