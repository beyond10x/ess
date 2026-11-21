//! The process-target guide's adapter runs, and holds the specification it serves to its suite.
//!
//! `website/docs/guides/verify/target-process.md`, the page "An implementation in any language",
//! shows one complete `ess --target process` adapter in Python 3, in the fenced block whose fence
//! carries `title="adapter/adapter.py"`. It serves the `tasks` specification that the Start-here page
//! `website/docs/start/first-specification.md` writes in its block carrying
//! `file=tasks/spec/system.yaml`. This test takes the bytes of both blocks from the pages, so the
//! documented adapter is the tested one, lays them out as the guide does, synthesizes the suite and
//! runs the guide's command against the `ess` this package builds.
//!
//! The Python is documentation under test, not a checker: the checker is this file, and no `.py`
//! file is committed. Where `python3` is not on `PATH` each case prints its skip and the reason,
//! and passes, as `tutorial_page.rs` does for `go`.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use serde_json::Value;

const ESS: &str = env!("CARGO_BIN_EXE_ess");
const GUIDE: &str = "website/docs/guides/verify/target-process.md";
const ADAPTER: &str = r#"title="adapter/adapter.py""#;
const SPECIFICATION_PAGE: &str = "website/docs/start/first-specification.md";
const SPECIFICATION: &str = "file=tasks/spec/system.yaml";

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(3)
        .expect("the CLI is three levels below the workspace root")
        .to_path_buf()
}

/// The body of the one fenced block on `page` whose fence carries the attribute `attribute`, with
/// each line ending in `\n`.
fn fenced(page: &str, attribute: &str) -> String {
    let text = fs::read_to_string(root().join(page))
        .unwrap_or_else(|error| panic!("read {page}: {error}"));
    let mut found: Vec<String> = Vec::new();
    let mut open: Option<(bool, String)> = None;
    for line in text.lines() {
        let trimmed = line.trim_start();
        match open.take() {
            None => {
                if let Some(info) = trimmed.strip_prefix("```") {
                    let wanted = info
                        .split_whitespace()
                        .skip(1)
                        .any(|word| word == attribute);
                    open = Some((wanted, String::new()));
                }
            }
            Some((wanted, mut body)) => {
                if trimmed.starts_with("```") {
                    if wanted {
                        found.push(body);
                    }
                } else {
                    body.push_str(line);
                    body.push('\n');
                    open = Some((wanted, body));
                }
            }
        }
    }
    assert!(open.is_none(), "{page} ends inside a code fence");
    assert_eq!(
        found.len(),
        1,
        "{page} must hold exactly one fenced block carrying `{attribute}`"
    );
    found.remove(0)
}

/// Why the guide's adapter cannot run here, or `None` when it can.
fn python_skipped() -> Option<&'static str> {
    let runs = Command::new("python3")
        .arg("--version")
        .output()
        .is_ok_and(|output| output.status.success());
    (!runs).then_some("python3 is not on PATH")
}

fn ess(directory: &Path, args: &[&str]) -> Output {
    Command::new(ESS)
        .current_dir(directory)
        .args(args)
        .output()
        .expect("the `ess` binary runs")
}

fn shown(output: &Output) -> String {
    format!(
        "exit {:?}\nstdout:\n{}\nstderr:\n{}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

/// One run of the guide's commands: the suite's scenario ids, `run`'s output and the report/2 it
/// wrote.
struct Run {
    scenarios: BTreeSet<String>,
    output: Output,
    report: Value,
}

/// Lays out the guide's `tasks` directory under `case` with `adapter` as `adapter/adapter.py`,
/// synthesizes the suite and runs the process target as the guide does.
fn run_guide(case: &str, adapter: &str) -> Run {
    let directory = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("process_target_guide")
        .join(case);
    if directory.exists() {
        fs::remove_dir_all(&directory).expect("clear the case directory");
    }
    fs::create_dir_all(directory.join("spec")).expect("make spec/");
    fs::create_dir_all(directory.join("adapter")).expect("make adapter/");
    fs::write(
        directory.join("spec/system.yaml"),
        fenced(SPECIFICATION_PAGE, SPECIFICATION),
    )
    .expect("write spec/system.yaml");
    fs::write(directory.join("adapter/adapter.py"), adapter).expect("write adapter/adapter.py");

    let synthesized = ess(
        &directory,
        &[
            "verify",
            "conform",
            "synthesize",
            "--path",
            "spec",
            "--suite-format",
            "5",
            "--out",
            "suite.json",
        ],
    );
    assert!(
        synthesized.status.success(),
        "synthesis fails: {}",
        shown(&synthesized)
    );
    let suite: Value = serde_json::from_slice(
        &fs::read(directory.join("suite.json")).expect("synthesis writes suite.json"),
    )
    .expect("the suite is JSON");
    let scenarios: BTreeSet<String> = suite["scenarios"]
        .as_object()
        .expect("the suite holds its scenarios by id")
        .keys()
        .cloned()
        .collect();
    assert!(!scenarios.is_empty(), "the suite holds no scenario");

    let output = ess(
        &directory,
        &[
            "verify",
            "conform",
            "run",
            "--suite",
            "suite.json",
            "--target",
            "process",
            "--target-dir",
            "adapter",
            "--target-timeout",
            "30",
            "--report-format",
            "2",
            "--strict",
            "--report-out",
            "report.json",
            "--",
            "python3",
            "adapter.py",
        ],
    );
    let bytes = fs::read(directory.join("report.json"))
        .unwrap_or_else(|error| panic!("no report written ({error}): {}", shown(&output)));
    let report = serde_json::from_slice(&bytes).expect("the report is JSON");
    Run {
        scenarios,
        output,
        report,
    }
}

fn ids(report: &Value, category: &str) -> BTreeSet<String> {
    report["outcomes"][category]
        .as_array()
        .unwrap_or_else(|| panic!("report/2 lists `outcomes.{category}`"))
        .iter()
        .map(|id| id.as_str().expect("a scenario id is a string").to_owned())
        .collect()
}

fn count(report: &Value, category: &str) -> u64 {
    report["counts"][category]
        .as_u64()
        .unwrap_or_else(|| panic!("report/2 counts `{category}`"))
}

#[test]
fn the_guides_python_adapter_passes_every_scenario_of_its_specification() {
    if let Some(reason) = python_skipped() {
        eprintln!("skipped: the guide's Python adapter does not run here: {reason}");
        return;
    }
    let run = run_guide("documented", &fenced(GUIDE, ADAPTER));

    assert_eq!(
        run.output.status.code(),
        Some(0),
        "the documented adapter does not pass under --strict: {}",
        shown(&run.output)
    );
    let total = u64::try_from(run.scenarios.len()).expect("a scenario count fits u64");
    assert_eq!(count(&run.report, "total"), total, "{}", run.report);
    assert_eq!(count(&run.report, "passed"), total, "{}", run.report);
    for category in ["failed", "error", "unsupported", "skipped"] {
        assert_eq!(
            count(&run.report, category),
            0,
            "{category}: {}",
            run.report
        );
    }
    assert_eq!(ids(&run.report, "passed"), run.scenarios);
}

#[test]
fn the_guides_adapter_with_one_wrong_outcome_fails_that_scenario() {
    if let Some(reason) = python_skipped() {
        eprintln!("skipped: the guide's Python adapter does not run here: {reason}");
        return;
    }
    let documented = fenced(GUIDE, ADAPTER);
    let refusal = r#"took("rejected""#;
    assert_eq!(
        documented.matches(refusal).count(),
        1,
        "the guide's adapter answers the `rejected` outcome in exactly one place"
    );
    let run = run_guide(
        "wrong-outcome",
        &documented.replace(refusal, r#"took("added""#),
    );

    assert_eq!(
        run.output.status.code(),
        Some(1),
        "an adapter answering `added` for a refused AddTask must fail the run: {}",
        shown(&run.output)
    );
    let wrong = "tasks.list.AddTask/outcome/rejected";
    assert_eq!(
        ids(&run.report, "failed"),
        BTreeSet::from([wrong.to_owned()]),
        "{}",
        run.report
    );
    let mut others = run.scenarios.clone();
    others.remove(wrong);
    assert_eq!(ids(&run.report, "passed"), others, "{}", run.report);
}
