//! Adversary, w1fix pass 1: `ess conform author` assembles a suite from authored scenarios alone,
//! with no synthesis, so its call to `select_fresh_format_for` is the only thing that could write
//! suite/26 for an authored `satisfies` reading `missing()` over an `Optional` struct
//! (beyond10x/ess#176). `formats.md` documents the construct as "a view `satisfies` predicate",
//! and an authored `assert: [{view, satisfies}]` is one.
//!
//! Red: the authoring surface refuses the predicate before any format is selected
//! (`ESS-AUTHOR-026`, "publishes nothing at `metrics`"), because `authored.rs` admits only scalar
//! reads where synthesis admits presence of an `Optional` aggregate (`input.rs`
//! `predicate_projectable`). So the new call in `author_suite` is unreachable for this construct,
//! and a synthesized suite can carry a check its own author cannot write.
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

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

fn workspace(name: &str) -> PathBuf {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("adversary-w1fix-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("model")).unwrap();
    fs::create_dir_all(root.join("scenarios")).unwrap();
    // Without the invariant, so nothing synthesized carries the construct and only the authored
    // scenario can select the format.
    let model = QUEUE.replace(
        "    invariants:\n      - any: [state == Paused, {not: \"defined(metrics)\"}]\n",
        "",
    );
    assert_ne!(model, QUEUE, "the invariant is removed");
    fs::write(root.join("model/queue.yaml"), model).unwrap();
    fs::write(root.join("scenarios/fresh.yaml"), SCENARIO).unwrap();
    root
}

fn author(root: &Path) -> (bool, String, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_ess"))
        .args([
            "conform",
            "author",
            "--path",
            root.join("model/queue.yaml").to_str().unwrap(),
            "--scenarios",
            root.join("scenarios/fresh.yaml").to_str().unwrap(),
        ])
        .output()
        .unwrap();
    (
        output.status.success(),
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

#[test]
fn adversary_w1fix_an_authored_missing_over_an_optional_struct_is_written_in_suite_26() {
    let root = workspace("ordinary");
    let (success, stdout, stderr) = author(&root);
    let _ = fs::remove_dir_all(&root);
    assert!(
        !stderr.contains("ESS-AUTHOR-026") && !stdout.contains("ESS-AUTHOR-026"),
        "the authored presence predicate is refused:\n{stdout}\n{stderr}"
    );
    assert!(success, "{stdout}\n{stderr}");
    assert!(stdout.contains("ess-conformance/34"), "{stdout}");
}

/// `ess verify conform synthesize` re-selects the format after merging authored scenarios into the
/// synthesized suite. No test drove that call: a mutant writing `select_fresh_format()` there
/// lowers the #176 invariant's suite below /26 again and every existing suite stays green.
#[test]
fn adversary_w1fix_the_synthesize_verb_writes_the_invariant_suite_in_26() {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("adversary-w1fix-synthesize-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    let model = root.join("queue.yaml");
    fs::write(&model, QUEUE).unwrap();
    let out = root.join("suite.json");
    let output = Command::new(env!("CARGO_BIN_EXE_ess"))
        .args([
            "verify",
            "conform",
            "synthesize",
            "--path",
            model.to_str().unwrap(),
            "--target",
            "ir",
            "--out",
            out.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    let written = fs::read_to_string(&out).unwrap_or_default();
    let _ = fs::remove_dir_all(&root);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        written.contains("\"suite_version\": \"ess-conformance/34\"")
            || written.contains("\"suite_version\":\"ess-conformance/34\""),
        "{written}"
    );
}
