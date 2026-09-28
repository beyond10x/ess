//! `ess conform author --format json` with a refused scenario: standard output stays the suite
//! document, and the refusal is named on standard error, so an exit of 1 never arrives without
//! saying which scenario did not compile or why.
use std::{fs, path::PathBuf, process::Command};

const QUEUE: &str = include_str!(
    "../../../verify/ess-conformance/tests/fixtures/defined-over-optional-aggregates.yaml"
);

/// Asserts on a view the model does not declare, so authoring refuses the scenario.
const REFUSED: &str = "type: ess-scenario/1
domain: demo.queue
scenario: reads-a-view-nobody-declared
summary: Asserts on a view the model does not declare.
timeline:
  - at: 2026-01-05T09:00:00Z
    command: demo.queue.OpenQueue
    actor: demo.queue.Operator
    input: {}
    outcome: opened
assert:
  - view: demo.queue.NoSuchView
    contains: {}
";

#[test]
fn a_refused_scenario_is_named_on_stderr_and_stdout_stays_the_document() {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("author-json-refusals-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    let model = root.join("queue.yaml");
    let scenario = root.join("refused.yaml");
    fs::write(&model, QUEUE).unwrap();
    fs::write(&scenario, REFUSED).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_ess"))
        .args([
            "conform",
            "author",
            "--path",
            model.to_str().unwrap(),
            "--scenarios",
            scenario.to_str().unwrap(),
            "--format",
            "json",
        ])
        .output()
        .unwrap();
    let _ = fs::remove_dir_all(&root);
    let (stdout, stderr) = (
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    );
    assert_eq!(output.status.code(), Some(1), "{stdout}\n{stderr}");
    assert!(
        stderr.contains("ESS-AUTHOR-") && stderr.contains("NoSuchView"),
        "the refusal is named on stderr:\n{stderr}"
    );
    let document: serde_json::Value = serde_json::from_str(&stdout)
        .unwrap_or_else(|error| panic!("stdout is the JSON document ({error}):\n{stdout}"));
    assert!(document.get("provenance").is_some(), "{stdout}");
}
