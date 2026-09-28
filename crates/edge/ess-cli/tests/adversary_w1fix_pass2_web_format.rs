//! Adversary, w1fix pass 2: `ess verify conform web` assembles its suite from authored scenarios
//! and selects the format with `select_fresh_format_for` (`main.rs` `conform_web`). No test drives
//! that call with the #176 construct: a mutant writing `select_fresh_format()` there emits an
//! authored `missing(metrics)` over an `Optional` struct in a suite below /26, which a 0.37.0 reader
//! would accept and read as absent.
use std::{fs, path::PathBuf, process::Command};

const QUEUE: &str = include_str!(
    "../../../verify/ess-conformance/tests/fixtures/defined-over-optional-aggregates.yaml"
);

const SCENARIO: &str = "type: ess-scenario/1
domain: demo.queue
scenario: a-fresh-queue-holds-no-metrics
summary: A queue just opened holds no metrics.
timeline:
  - at: 2026-01-05T09:00:00Z
    command: demo.queue.OpenQueue
    actor: demo.queue.Operator
    input: {}
    outcome: opened
assert:
  - view: demo.queue.QueueById
    satisfies: missing(metrics)
";

#[test]
fn adversary_w1fix_pass2_the_web_verb_writes_an_authored_presence_predicate_in_suite_26() {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("adversary-w1fix-pass2-web-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("scenarios")).unwrap();
    let model = QUEUE.replace(
        "    invariants:\n      - any: [state == Paused, {not: \"defined(metrics)\"}]\n",
        "",
    );
    assert_ne!(model, QUEUE, "the invariant is removed");
    fs::write(root.join("queue.yaml"), model).unwrap();
    fs::write(root.join("scenarios/fresh.yaml"), SCENARIO).unwrap();
    let out = root.join("web");
    let output = Command::new(env!("CARGO_BIN_EXE_ess"))
        .args(["verify", "conform", "web", "--path"])
        .arg(root.join("queue.yaml"))
        .arg("--scenarios")
        .arg(root.join("scenarios/fresh.yaml"))
        .arg("--out")
        .arg(&out)
        .output()
        .unwrap();
    let suite = fs::read_to_string(out.join("suite.json")).unwrap_or_default();
    let listing: Vec<_> = fs::read_dir(&out)
        .map(|dir| {
            dir.filter_map(|entry| entry.ok().map(|e| e.file_name()))
                .collect()
        })
        .unwrap_or_default();
    let _ = fs::remove_dir_all(&root);
    let (stdout, stderr) = (
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
    assert!(output.status.success(), "{stdout}\n{stderr}");
    assert!(
        suite.contains("missing(metrics)") || suite.contains("defined(metrics)"),
        "the authored predicate is in the emitted suite ({listing:?}): {suite}"
    );
    assert!(
        suite.contains("\"ess-conformance/26\""),
        "the emitted suite is suite/26: {suite}"
    );
}
