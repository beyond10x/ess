//! `story:interpreted-command-execution`, through the binary an operator runs.
//!
//! The acceptance, verbatim: every scenario in `examples/billing`'s committed suite that exercises
//! only command execution, transitions, `sets:` writes, emitted events and declared refusals reports
//! the same result under `--target interpreted` as under `--target billing`.
//!
//! Selected from the suite's own steps and pinned, so a filter that admits nothing cannot pass.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::Command;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

const SUITE: &str = "suites/generated/billing/suite.json";

/// The steps that are command execution or an assertion about what it did, as the suite spells them.
const COMMAND_EXECUTION: &[&str] = &[
    "execute_command",
    "expect_outcome",
    "expect_error",
    "expect_no_error",
    "expect_event",
    "expect_no_event",
    "expect_no_events",
    "capture_instance",
];

/// Every scenario of one run, by id: its status and its checks.
fn run(target: &str) -> BTreeMap<String, serde_json::Value> {
    let output = Command::new(env!("CARGO_BIN_EXE_ess"))
        .current_dir(root())
        .args(["verify", "conform", "run", "--target", target])
        .args(["--path", "examples/billing"])
        .args(["--suite", SUITE])
        .args(["--format", "json"])
        .output()
        .expect("the `ess` binary runs");
    let report: serde_json::Value =
        serde_json::from_slice(&output.stdout).unwrap_or_else(|error| {
            panic!(
                "`--target {target}` renders report/1 as JSON: {error}\n{}",
                String::from_utf8_lossy(&output.stderr)
            )
        });
    report["scenarios"]
        .as_array()
        .expect("scenarios")
        .iter()
        .map(|scenario| {
            (
                scenario["scenario"].as_str().expect("an id").to_owned(),
                serde_json::json!({
                    "status": scenario["status"],
                    "checks": scenario["checks"],
                }),
            )
        })
        .collect()
}

#[test]
fn every_command_execution_scenario_reports_the_same_result_under_interpreted_as_under_billing() {
    let suite: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(root().join(SUITE)).expect("the committed suite is readable"),
    )
    .expect("the committed suite is JSON");
    let in_scope: BTreeSet<String> = suite["scenarios"]
        .as_object()
        .expect("scenarios by id")
        .iter()
        .filter(|(_, scenario)| {
            scenario["steps"]
                .as_array()
                .expect("steps")
                .iter()
                .all(|step| COMMAND_EXECUTION.contains(&step["step"].as_str().expect("a step")))
        })
        .map(|(id, _)| id.clone())
        .collect();
    assert_eq!(
        in_scope.len(),
        14,
        "the committed suite holds fourteen command-execution scenarios: {in_scope:#?}"
    );

    let billing = run("billing");
    let interpreted = run("interpreted");
    for id in &in_scope {
        assert_eq!(
            billing[id]["status"], "passed",
            "`{id}` passes against the hand-written reference"
        );
        assert_eq!(
            interpreted[id], billing[id],
            "`{id}` reports the same result under `--target interpreted` as under `--target billing`"
        );
    }
}
