//! `ess verify conform mutate --scenarios <dir>` (<https://github.com/beyond10x/ess/issues/515>):
//! the authored scenarios, read and refused as `run --scenarios` reads them, run against the
//! interpreter of the specification for the baseline and against the interpreter of each mutant,
//! beside the synthesized suites. A mutant only an authored scenario pins is killed by it and the
//! report names it as the killer. The library's verdicts are decided in
//! `crates/verify/ess-conformance/tests/mutation_authored.rs`.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const MODEL: &str = "crates/verify/ess-conformance/tests/fixtures/mutation-authored.yaml";
const SCENARIOS: &str = "crates/verify/ess-conformance/tests/fixtures/mutation-authored";
const AUTHORED_ID: &str = "work.tasks/authored/retries-reach-the-limit";

const ERROR_SWAP: &str = "error-swap/work.tasks.RetryTask/at-limit";
const STRICTER: &str = "guard-boundary/work.tasks.RetryTask/at-limit/when_subject/0";

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("the workspace root exists")
}

/// `mutate` on the fixture, over the classes that have a site on `at-limit` and no unavailable
/// site, so the exit status says what the verdicts are.
fn mutate(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ess"))
        .current_dir(root())
        .args(["verify", "conform", "mutate", "--path", MODEL])
        .args([
            "--class",
            "error-swap",
            "--class",
            "guard-boundary",
            "--class",
            "guard-negate",
        ])
        .args(args)
        .output()
        .expect("the `ess` binary runs")
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8(bytes.to_vec()).expect("UTF-8")
}

fn scratch(name: &str) -> PathBuf {
    let directory = root()
        .join("target/mutate-authored")
        .join(format!("{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&directory);
    std::fs::create_dir_all(&directory).expect("a scratch directory");
    directory
}

fn verdict<'a>(report: &'a serde_json::Value, id: &str) -> &'a serde_json::Value {
    report["mutants"]
        .as_array()
        .expect("mutants")
        .iter()
        .find(|mutant| mutant["id"] == id)
        .unwrap_or_else(|| panic!("{id}: {report:#}"))
}

#[test]
fn without_scenarios_the_mutants_only_an_authored_scenario_pins_are_unwitnessed() {
    let output = mutate(&["--target", "interpreted", "--format", "json"]);
    assert_eq!(output.status.code(), Some(3), "{}", text(&output.stderr));
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    for id in [ERROR_SWAP, STRICTER] {
        assert_eq!(verdict(&report, id)["verdict"], "unwitnessed", "{report:#}");
    }
}

#[test]
fn the_authored_scenario_kills_them_and_is_named_as_the_killer() {
    let output = mutate(&[
        "--target",
        "interpreted",
        "--scenarios",
        SCENARIOS,
        "--format",
        "json",
    ]);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}{}",
        text(&output.stdout),
        text(&output.stderr)
    );
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["format"], "ess-mutation-report/3", "{report:#}");
    for id in [ERROR_SWAP, STRICTER] {
        let mutant = verdict(&report, id);
        assert_eq!(mutant["verdict"], "killed", "{mutant:#}");
        assert_eq!(
            mutant["killers"],
            serde_json::json!([AUTHORED_ID]),
            "{mutant:#}"
        );
    }
    assert_eq!(report["counts"]["unwitnessed"], 0, "{report:#}");
    assert_eq!(report["baseline"]["scenarios"], 3, "{report:#}");

    let output = mutate(&["--target", "interpreted", "--scenarios", SCENARIOS]);
    let stdout = text(&output.stdout);
    assert_eq!(output.status.code(), Some(0), "{stdout}");
    assert!(
        stdout.lines().any(|line| line
            == format!(
                "killed {ERROR_SWAP}: `error: work.tasks.Exhausted` becomes `error: \
                 work.tasks.Blocked` — by {AUTHORED_ID} (1 in total)"
            )),
        "{stdout}"
    );
}

/// An authored scenario the specification's own interpreter fails refuses the audit as a failed
/// synthesized baseline scenario does: `ESS-MUTATE-001`, exit 3, naming it.
#[test]
fn an_authored_scenario_the_baseline_fails_refuses_the_audit() {
    let directory = scratch("baseline-fails");
    let authored =
        std::fs::read_to_string(root().join(SCENARIOS).join("retries-reach-the-limit.yaml"))
            .unwrap();
    let wrong = authored.replacen(
        "09:00:18Z, command: work.tasks.RetryTask, input: {task_id: {$instance: task}}, outcome: \
         retried}",
        "09:00:18Z, command: work.tasks.RetryTask, input: {task_id: {$instance: task}}, outcome: \
         at-limit}",
        1,
    );
    assert_ne!(wrong, authored, "the edit applies");
    std::fs::write(directory.join("retries-reach-the-limit.yaml"), wrong).unwrap();
    let output = mutate(&[
        "--target",
        "interpreted",
        "--scenarios",
        directory.to_str().unwrap(),
    ]);
    let stderr = text(&output.stderr);
    assert_eq!(output.status.code(), Some(3), "{stderr}");
    assert!(
        stderr.contains("ESS-MUTATE-001") && stderr.contains(AUTHORED_ID),
        "{stderr}"
    );
}

/// A document `run --scenarios` refuses is refused here the same way, before anything runs.
#[test]
fn an_authored_document_run_refuses_is_refused_with_its_code() {
    let directory = scratch("unreadable");
    std::fs::write(
        directory.join("broken.yaml"),
        "type: ess-scenario/1\ndomain: work.tasks\nscenario: broken\nsummary: s\ntimelime: []\n",
    )
    .unwrap();
    let output = mutate(&[
        "--target",
        "interpreted",
        "--scenarios",
        directory.to_str().unwrap(),
    ]);
    let stderr = text(&output.stderr);
    assert_eq!(output.status.code(), Some(1), "{stderr}");
    assert!(stderr.contains("ESS-AUTHOR-"), "{stderr}");
    assert!(
        text(&output.stdout).is_empty(),
        "nothing is audited: {}",
        text(&output.stdout)
    );
}

/// Only the interpreter implements a mutant: a built-in target that implements one fixed
/// specification, and an emission whose suites an adopter's runner of the unchanged
/// specification runs, could never be failed by an authored scenario.
#[test]
fn scenarios_are_refused_where_nothing_implements_the_mutant() {
    let output = mutate(&["--target", "billing", "--scenarios", SCENARIOS]);
    let stderr = text(&output.stderr);
    assert_eq!(output.status.code(), Some(1), "{stderr}");
    assert!(
        stderr.contains("--scenarios") && stderr.contains("--target interpreted"),
        "{stderr}"
    );

    let directory = scratch("emit");
    let output = mutate(&[
        "--emit",
        directory.join("out").to_str().unwrap(),
        "--scenarios",
        SCENARIOS,
    ]);
    assert_eq!(output.status.code(), Some(2), "{}", text(&output.stderr));
    assert!(
        text(&output.stderr).contains("cannot be used with"),
        "{}",
        text(&output.stderr)
    );
    assert!(!directory.join("out").exists(), "nothing is emitted");
}
