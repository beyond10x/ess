//! Adversary pass 1 on `ess verify conform mutate --scenarios <dir>`
//! (<https://github.com/beyond10x/ess/issues/515>), end to end. The library's cases are in
//! `crates/verify/ess-conformance/tests/adversary_j3_pass1.rs`.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const MODEL: &str = "crates/verify/ess-conformance/tests/fixtures/mutation-authored.yaml";
const SCENARIOS: &str = "crates/verify/ess-conformance/tests/fixtures/mutation-authored";
const AUTHORED_ID: &str = "work.tasks/authored/retries-reach-the-limit";

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("the workspace root exists")
}

fn ess(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ess"))
        .current_dir(root())
        .args(args)
        .output()
        .expect("the `ess` binary runs")
}

fn mutate(model: &str, args: &[&str]) -> Output {
    let mut all = vec!["verify", "conform", "mutate", "--path", model];
    all.extend_from_slice(args);
    ess(&all)
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

fn scratch(name: &str) -> PathBuf {
    let directory = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("adversary-j3")
        .join(name);
    let _ = std::fs::remove_dir_all(&directory);
    std::fs::create_dir_all(&directory).expect("a scratch directory");
    directory
}

/// The status `run`'s JSON report gives the scenario `id`.
fn status_of(report: &serde_json::Value, id: &str) -> serde_json::Value {
    report["scenarios"]
        .as_array()
        .expect("scenarios")
        .iter()
        .find(|scenario| {
            scenario
                .as_object()
                .is_some_and(|it| it.values().any(|v| v == id))
        })
        .unwrap_or_else(|| panic!("no {id} in {report:#}"))["status"]
        .clone()
}

const CLOCKED: &str = r"format: ess/23
system: demo
version: v1
domain: demo.tok
types:
  - {name: demo.tok.TokenId, kind: newtype, of: String}
errors:
  - name: demo.tok.Expired
  - name: demo.tok.Revoked
commands:
  - name: demo.tok.Ping
    input:
      - {name: token_id, type: demo.tok.TokenId}
    outcomes:
      - name: ponged
        accepts: nothing
  - name: demo.tok.Present
    input:
      - {name: token_id, type: demo.tok.TokenId}
      - {name: exp, type: Timestamp}
    outcomes:
      - name: expired
        when: exp < now - 10m
        error: demo.tok.Expired
      - name: accepted
        accepts: nothing
";

const LONG_EXPIRED: &str = r"type: ess-scenario/1
domain: demo.tok
scenario: long-expired
summary: A token that expired long ago is refused as expired.
timeline:
  - at: 2026-01-05T09:00:00Z
    command: demo.tok.Present
    input: {token_id: t-1, exp: 2000-01-01T00:00:00Z}
    outcome: expired
    error: {name: demo.tok.Expired}
";

/// "`DIR` is read and refused as `ess verify conform run --scenarios` reads it" — and `run
/// --target interpreted --scenarios` passes this authored scenario, deciding the `now` guard at
/// the step's instant (<https://github.com/beyond10x/ess/issues/510>). `mutate --scenarios
/// --target interpreted` runs it on interpreters built with no command clock, so it is
/// `unsupported` on the baseline and cannot kill the `error-swap` mutant it pins.
#[test]
fn an_authored_scenario_run_passes_kills_the_mutant_it_pins() {
    let directory = scratch("clocked");
    let model = directory.join("system.yaml");
    std::fs::write(&model, CLOCKED).unwrap();
    let scenarios = directory.join("scenarios");
    std::fs::create_dir_all(&scenarios).unwrap();
    std::fs::write(scenarios.join("long-expired.yaml"), LONG_EXPIRED).unwrap();
    let (model, scenarios) = (model.to_str().unwrap(), scenarios.to_str().unwrap());

    let run = ess(&[
        "verify",
        "conform",
        "run",
        "--target",
        "interpreted",
        "--report-format",
        "2",
        "--format",
        "json",
        "--path",
        model,
        "--scenarios",
        scenarios,
    ]);
    let report: serde_json::Value = serde_json::from_slice(&run.stdout)
        .unwrap_or_else(|error| panic!("{error}: {}{}", text(&run.stdout), text(&run.stderr)));
    assert_eq!(
        status_of(&report, "demo.tok/authored/long-expired"),
        "passed",
        "precondition, `run` passes it: {report:#}"
    );

    let output = mutate(
        model,
        &[
            "--class",
            "error-swap",
            "--target",
            "interpreted",
            "--scenarios",
            scenarios,
            "--format",
            "json",
        ],
    );
    let report: serde_json::Value =
        serde_json::from_slice(&output.stdout).unwrap_or_else(|error| {
            panic!("{error}: {}{}", text(&output.stdout), text(&output.stderr))
        });
    let mutant = report["mutants"]
        .as_array()
        .expect("mutants")
        .iter()
        .find(|it| it["id"] == "error-swap/demo.tok.Present/expired")
        .unwrap_or_else(|| panic!("{report:#}"));
    assert_eq!(mutant["verdict"], "killed", "{report:#}");
    assert_eq!(
        mutant["killers"],
        serde_json::json!(["demo.tok/authored/long-expired"]),
        "{mutant:#}"
    );
}

/// `--scenarios` with no target at all is a usage error naming the missing `--target`, not
/// audited.
#[test]
fn scenarios_without_a_target_are_refused() {
    let output = mutate(MODEL, &["--class", "error-swap", "--scenarios", SCENARIOS]);
    let stderr = text(&output.stderr);
    assert_eq!(output.status.code(), Some(2), "{stderr}");
    assert!(stderr.contains("--target <TARGET>"), "{stderr}");
    assert!(output.stdout.is_empty(), "{}", text(&output.stdout));
}

/// `--scenarios` beside `--collect` and beside `--known-failing` are usage errors.
#[test]
fn scenarios_beside_collect_or_known_failing_are_usage_errors() {
    let directory = scratch("conflicts");
    let declaration = directory.join("known.yaml");
    std::fs::write(&declaration, "anything").unwrap();
    for extra in [
        vec!["--collect", directory.to_str().unwrap()],
        vec![
            "--target",
            "interpreted",
            "--known-failing",
            declaration.to_str().unwrap(),
        ],
    ] {
        let mut args = vec!["--scenarios", SCENARIOS];
        args.extend(extra.iter().copied());
        let output = mutate(MODEL, &args);
        assert_eq!(
            output.status.code(),
            Some(2),
            "{extra:?}: {}",
            text(&output.stderr)
        );
        assert!(
            text(&output.stderr).contains("cannot be used with"),
            "{}",
            text(&output.stderr)
        );
    }
}

/// A missing directory is refused before anything is audited.
#[test]
fn a_missing_scenarios_directory_is_refused() {
    let missing = scratch("missing").join("absent");
    let output = mutate(
        MODEL,
        &[
            "--class",
            "error-swap",
            "--target",
            "interpreted",
            "--scenarios",
            missing.to_str().unwrap(),
        ],
    );
    assert!(!output.status.success(), "{}", text(&output.stderr));
    assert!(
        output.stdout.is_empty(),
        "nothing is audited: {}",
        text(&output.stdout)
    );
}

/// An empty directory is treated as `run --scenarios` treats it: refused by both, or read by both
/// as no authored scenario, in which case the audit is the audit without `--scenarios`.
#[test]
fn an_empty_scenarios_directory_is_what_run_makes_of_it() {
    let empty = scratch("empty");
    let empty = empty.to_str().unwrap();
    let run = ess(&[
        "verify",
        "conform",
        "run",
        "--target",
        "interpreted",
        "--path",
        MODEL,
        "--scenarios",
        empty,
    ]);
    let with = mutate(
        MODEL,
        &[
            "--class",
            "error-swap",
            "--target",
            "interpreted",
            "--scenarios",
            empty,
            "--format",
            "json",
        ],
    );
    let refused_by_run = run.status.code() == Some(1) && !text(&run.stderr).is_empty();
    if refused_by_run {
        assert_eq!(with.status.code(), Some(1), "{}", text(&with.stderr));
    } else {
        let without = mutate(
            MODEL,
            &[
                "--class",
                "error-swap",
                "--target",
                "interpreted",
                "--format",
                "json",
            ],
        );
        assert_eq!(
            with.status.code(),
            without.status.code(),
            "{}",
            text(&with.stderr)
        );
        assert_eq!(text(&with.stdout), text(&without.stdout));
    }
}

/// `run --scenarios` after the `compiled_authored` refactor: the authored scenario still passes
/// against the interpreter, beside every synthesized scenario.
#[test]
fn run_with_scenarios_still_runs_the_authored_scenario() {
    let run = ess(&[
        "verify",
        "conform",
        "run",
        "--target",
        "interpreted",
        "--report-format",
        "2",
        "--format",
        "json",
        "--path",
        MODEL,
        "--scenarios",
        SCENARIOS,
    ]);
    let report: serde_json::Value = serde_json::from_slice(&run.stdout)
        .unwrap_or_else(|error| panic!("{error}: {}{}", text(&run.stdout), text(&run.stderr)));
    assert_eq!(status_of(&report, AUTHORED_ID), "passed", "{report:#}");
    assert_eq!(run.status.code(), Some(0), "{}", text(&run.stderr));
}

/// Two audits with `--scenarios` print the same bytes, in both formats.
#[test]
fn two_audits_with_scenarios_print_the_same_bytes() {
    for format in ["json", "text"] {
        let args = [
            "--class",
            "error-swap",
            "--class",
            "guard-boundary",
            "--target",
            "interpreted",
            "--scenarios",
            SCENARIOS,
            "--format",
            format,
        ];
        let (first, second) = (mutate(MODEL, &args), mutate(MODEL, &args));
        assert_eq!(first.status.code(), second.status.code(), "{format}");
        assert_eq!(text(&first.stdout), text(&second.stdout), "{format}");
    }
}
