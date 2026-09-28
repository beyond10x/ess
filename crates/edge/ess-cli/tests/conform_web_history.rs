//! `ess verify conform web --history`: a checked history drawn as client lanes
//! (`story:concurrent-history-lanes`).
//!
//! What the page says is decided in `crates/verify/ess-conformance/tests/lanes.rs`; this holds the
//! command line to it: the page for the shrunk `LostUpdate` history names both payments, the same
//! history renders to the same bytes, and `--out` writes the bytes it prints.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_conformance::faulty::{self, Fault};
use ess_conformance::history::History;
use ess_conformance::linearize::{self, DEFAULT_BUDGET};
use ess_conformance::record;
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

fn web(history: &str, extra: &[&str]) -> Output {
    let mut args = vec![
        "verify",
        "conform",
        "web",
        "--path",
        "examples/billing",
        "--history",
        history,
    ];
    args.extend_from_slice(extra);
    ess(&args)
}

#[test]
fn the_shrunk_lost_update_page_names_both_payments_and_renders_to_the_same_bytes() {
    let model = billing_model();
    let recorded = record::record(
        &model,
        &faulty::billing(Fault::LostUpdate),
        &faulty::lost_update_workload(),
        OVERLAPPING,
    )
    .expect("recorded");
    let shrunk = linearize::shrink(&model, &recorded, DEFAULT_BUDGET).expect("shrunk");
    let directory = tempfile::tempdir().expect("a scratch directory");
    let history = written(directory.path(), "shrunk.json", &shrunk);

    let first = web(&history, &[]);
    assert_eq!(
        first.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&first.stderr)
    );
    let page = String::from_utf8(first.stdout.clone()).expect("the page is UTF-8");
    assert!(page.starts_with("<!DOCTYPE html>"), "{page}");
    let pays: Vec<&str> = shrunk
        .operations
        .iter()
        .filter(|operation| operation.command.as_str() == "billing.invoice.PayInvoice")
        .map(|operation| operation.operation_id.as_str())
        .collect();
    assert_eq!(pays.len(), 2);
    for id in pays {
        assert!(
            page.contains(&format!(
                "<li class=\"conflict\" data-operation=\"{id}\" data-required=\"Issued\" \
                 data-supplied=\"Paid\""
            )),
            "payment {id} is named with the state it needed and the state it got:\n{page}"
        );
    }

    let second = web(&history, &[]);
    assert_eq!(
        first.stdout, second.stdout,
        "the same history, the same bytes"
    );

    let out = directory.path().join("page");
    let written = web(&history, &["--out", &out.display().to_string()]);
    assert_eq!(
        written.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&written.stderr)
    );
    assert_eq!(
        std::fs::read(out.join("index.html")).expect("the page is written"),
        first.stdout,
        "`--out` writes the page it prints"
    );
}

#[test]
fn history_and_scenarios_are_not_taken_together() {
    let output = ess(&[
        "verify",
        "conform",
        "web",
        "--path",
        "examples/billing",
        "--history",
        "absent.json",
        "--scenarios",
        "examples/billing",
    ]);
    assert_eq!(output.status.code(), Some(2), "a usage error");
}
