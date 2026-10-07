//! Exploration with real process restarts, executed in both emitted languages (beyond10x/ess#297).
//!
//! An implementation that mints identities from a counter kept only in its process passes every
//! scenario and every explored sequence that runs in one process lifetime. After a restart the
//! counter starts again and the next creation reuses an identity already stored. This file holds
//! the explorer's opt-in restarts (`restartEvery` / `RestartEvery`) to the three facts the runbook
//! asks for: an honest durable target passes across real restarts, a target whose counter resets on
//! restart fails with the colliding identity, and a target that cannot restart reports restarts
//! unsupported, which never passes. Without the option the explorer's results keep their bytes.
//!
//! The fixture is `tests/fixtures/restart.yaml`; the targets are `restart_target.go` and
//! `restart-target.mjs`, each of which runs its implementation as a child process over a state
//! directory and restarts that process when asked. A missing `tsc`, `node` or `go` panics rather
//! than skipping.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_conformance::scenario::ConformanceSuite;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use serde_json::{json, Value};

const FIXTURE: &str = include_str!("fixtures/restart.yaml");

fn ir() -> EssIr {
    let mut sources = SourceMap::new();
    sources.insert("restart.yaml", FIXTURE);
    let raw = RawSpecFile::parse(FIXTURE).expect("the fixture parses");
    let specification = Specification::assemble(vec![(Source::new("restart.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("the fixture validates:\n{errors}"));
    compile(&specification, &sources).unwrap_or_else(|diagnostics| panic!("{diagnostics}"))
}

fn suite(ir: &EssIr) -> ConformanceSuite {
    let mut suite = ess_conformance::synthesize(ir).suite;
    suite.select_fresh_format();
    suite
}

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

fn scratch(label: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("ess-restart-{}-{label}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("a scratch directory");
    root
}

fn write(path: &Path, contents: &str) {
    std::fs::create_dir_all(path.parent().expect("a parent")).expect("a directory");
    std::fs::write(path, contents).expect("writable");
}

fn printed(output: &std::process::Output) -> String {
    format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

/// What one lane reported: each case's result, assertion and count of backend processes.
#[derive(Debug)]
struct Lane {
    results: BTreeMap<String, Option<Value>>,
    asserts: BTreeMap<String, String>,
    processes: BTreeMap<String, u64>,
    log: String,
}

fn read_lane(out: &Path, cases: &Value, log: String) -> Lane {
    let mut results = BTreeMap::new();
    let mut asserts = BTreeMap::new();
    let mut processes = BTreeMap::new();
    for case in cases.as_array().expect("a case list") {
        let name = case["name"].as_str().expect("a case name").to_owned();
        let json = std::fs::read_to_string(out.join(format!("{name}.json")))
            .ok()
            .map(|text| serde_json::from_str(&text).expect("the result is JSON"));
        let assert = std::fs::read_to_string(out.join(format!("{name}.assert")))
            .unwrap_or_else(|_| panic!("`{name}` wrote no assertion:\n{log}"));
        let count = std::fs::read_to_string(out.join(format!("{name}.processes")))
            .unwrap_or_else(|_| panic!("`{name}` wrote no process count:\n{log}"));
        results.insert(name.clone(), json);
        asserts.insert(name.clone(), assert.trim_end().to_owned());
        processes.insert(name, count.trim_end().parse().expect("a count"));
    }
    Lane {
        results,
        asserts,
        processes,
        log,
    }
}

/// The TypeScript lane: the emitted package, compiled with `tsc --noCheck`, run with `node --test`.
fn typescript(root: &Path, ir: &EssIr, cases: &Value) -> Lane {
    let package = root.join("typescript");
    for artifact in ess_conformance::ts::emit_with_model(&suite(ir), ir).unwrap() {
        write(&package.join(&artifact.path), &artifact.contents);
    }
    let ts = package.join("essconform");
    for name in [
        "restart-target.mjs",
        "restart-backend.mjs",
        "restart-driver.mjs",
    ] {
        std::fs::copy(fixture(name), ts.join(name)).expect("the fixture copies");
    }
    write(
        &ts.join("runtime-test.tsconfig.json"),
        r#"{"extends":"./tsconfig.json","compilerOptions":{"types":[],"noCheck":true}}"#,
    );
    let compiled = Command::new("tsc")
        .args(["--project", "runtime-test.tsconfig.json"])
        .current_dir(&ts)
        .output()
        .expect("`tsc` is on PATH: the restart lane does not skip");
    assert!(compiled.status.success(), "{}", printed(&compiled));
    let out = root.join("out-typescript");
    std::fs::create_dir_all(&out).unwrap();
    let run = Command::new("node")
        .args(["--test", "restart-driver.mjs"])
        .env("ESS_EXPLORE_CASES", cases.to_string())
        .env("ESS_EXPLORE_OUT", &out)
        .current_dir(&ts)
        .output()
        .expect("`node` is on PATH: the restart lane does not skip");
    let log = printed(&run);
    assert!(run.status.success(), "{log}");
    for (index, case) in cases.as_array().unwrap().iter().enumerate() {
        let line = format!("ok {} - {}", index + 1, case["name"].as_str().unwrap());
        assert!(
            log.lines().any(|it| it == line),
            "`{line}` is not in the node log:\n{log}"
        );
    }
    read_lane(&out, cases, log)
}

/// The Go lane: the emitted package as a module of its own, run with `go test`.
fn go(root: &Path, ir: &EssIr, cases: &Value) -> Lane {
    let module = root.join("go");
    for artifact in ess_conformance::go::emit_with_model(&suite(ir), ir).unwrap() {
        write(&module.join(&artifact.path), &artifact.contents);
    }
    write(
        &module.join("go.mod"),
        "module example.invalid/restart\n\ngo 1.24\n",
    );
    std::fs::copy(
        fixture("restart_target.go"),
        module.join("essconform/restart_target_test.go"),
    )
    .unwrap();
    std::fs::copy(
        fixture("restart_driver_test.go"),
        module.join("essconform/restart_driver_test.go"),
    )
    .unwrap();
    let out = root.join("out-go");
    std::fs::create_dir_all(&out).unwrap();
    let run = Command::new("go")
        .args([
            "test",
            "./essconform",
            "-run",
            "^TestExploreRestart$",
            "-count=1",
            "-v",
        ])
        .env("ESS_EXPLORE_CASES", cases.to_string())
        .env("ESS_EXPLORE_OUT", &out)
        .env_remove("ESS_RESTART_BACKEND")
        .env("GOWORK", "off")
        .current_dir(&module)
        .output()
        .expect("`go` is on PATH: the restart lane does not skip");
    let log = printed(&run);
    assert!(run.status.success(), "{log}");
    for case in cases.as_array().unwrap() {
        let line = format!(
            "--- PASS: TestExploreRestart/{}",
            case["name"].as_str().unwrap()
        );
        assert!(
            log.lines().any(|it| it.trim_start().starts_with(&line)),
            "`{line}` is not in the go log:\n{log}"
        );
    }
    read_lane(&out, cases, log)
}

/// Three sequences of four commands, restarted after every second command.
fn restarting() -> Value {
    json!({"seeds": 3, "steps": 4, "restartEvery": 2})
}

fn cases() -> Value {
    json!([
        {"name": "durable", "mode": "durable", "options": restarting()},
        {"name": "durable-without-restarts", "mode": "durable", "options": {"seeds": 3, "steps": 4}},
        {"name": "counter-reset", "mode": "counter-reset", "options": restarting()},
        {"name": "counter-reset-replayed", "mode": "counter-reset", "options": {"seed": 1, "steps": 4, "restartEvery": 2}},
        {"name": "counter-reset-without-restarts", "mode": "counter-reset", "options": {"seeds": 3, "steps": 4}},
        {"name": "volatile", "mode": "volatile", "options": restarting()},
        {"name": "no-restart", "mode": "no-restart", "options": restarting()},
        {"name": "no-restart-allowed", "mode": "no-restart", "options": restarting(), "allowExcluded": true},
        {"name": "restart-refused", "mode": "restart-refused", "options": restarting()},
        {"name": "too-short", "mode": "durable", "options": {"seeds": 2, "steps": 1, "restartEvery": 2}},
        {"name": "negative", "mode": "durable", "options": {"seeds": 1, "steps": 2, "restartEvery": -1}},
        // A restart after the last command of every sequence, so nothing but the command F1 adds
        // after it is created after one.
        {"name": "trailing", "mode": "counter-reset", "options": {"seeds": 3, "steps": 2, "restartEvery": 2}},
        {"name": "trailing-durable", "mode": "durable", "options": {"seeds": 3, "steps": 2, "restartEvery": 2}},
        {"name": "every-command", "mode": "durable", "options": {"seeds": 2, "steps": 3, "restartEvery": 1}},
        {"name": "token", "mode": "token", "options": restarting()},
    ])
}

/// Cases only the TypeScript lane can express: Go `ConcurrentOptions` has no `RestartEvery`.
fn typescript_only_cases() -> Value {
    json!([
        {"name": "concurrent-refused", "mode": "durable", "options": {"seeds": 1, "restartEvery": 2}, "concurrent": true},
    ])
}

/// Both lanes, run once for every test below.
fn lanes() -> &'static (Lane, Lane) {
    static LANES: OnceLock<(Lane, Lane)> = OnceLock::new();
    LANES.get_or_init(|| {
        let ir = ir();
        let cases = cases();
        let mut typescript_cases = cases.as_array().unwrap().clone();
        typescript_cases.extend(typescript_only_cases().as_array().unwrap().iter().cloned());
        let root = scratch("lanes");
        let lanes = (
            typescript(&root, &ir, &Value::Array(typescript_cases)),
            go(&root, &ir, &cases),
        );
        std::fs::remove_dir_all(&root).ok();
        lanes
    })
}

fn both() -> [(&'static str, &'static Lane); 2] {
    [("typescript", &lanes().0), ("go", &lanes().1)]
}

fn result<'a>(lane: &'a Lane, name: &str) -> &'a Value {
    lane.results[name].as_ref().unwrap_or_else(|| {
        panic!(
            "`{name}` returned no result: {}\n{}",
            lane.asserts[name], lane.log
        )
    })
}

fn trace(found: &Value) -> Vec<&str> {
    found["failure"]["trace"]
        .as_array()
        .unwrap_or_else(|| panic!("a failure with a trace: {found}"))
        .iter()
        .map(|step| step.as_str().expect("a trace line"))
        .collect()
}

const RECORD: &str = r#"restart.ledger.RecordEntry {"amount":"#;
const FIRST: &str = "00000000-0000-4000-8000-000000000001";

#[test]
fn an_honest_durable_target_passes_across_real_process_restarts() {
    for (language, lane) in both() {
        let durable = result(lane, "durable");
        assert_eq!(lane.asserts["durable"], "ok", "{language}: {durable}");
        assert!(durable.get("failure").is_none(), "{language}: {durable}");
        // Four commands in each of three sequences, and one more after the restart that follows the
        // fourth: a restart is a check only once a command has followed it.
        assert_eq!(durable["executed"], 15, "{language}: {durable}");
        // Two restarts in each of three sequences, after the second and the fourth command.
        assert_eq!(
            durable["restarts"],
            json!({"every": 2, "performed": 6}),
            "{language}: {durable}"
        );
        // Each sequence started one backend process and each restart started another.
        assert_eq!(lane.processes["durable"], 9, "{language}");
    }
}

#[test]
fn a_counter_that_resets_on_restart_fails_with_the_colliding_identity() {
    for (language, lane) in both() {
        let found = result(lane, "counter-reset");
        let failure = &found["failure"];
        let message = failure["message"].as_str().unwrap_or_default();
        assert!(
            message.starts_with(&format!(
                "identity: the target created \"{FIRST}\" again, over an existing record (step 3: {RECORD}"
            )),
            "{language}: {found}"
        );
        let steps = trace(found);
        assert_eq!(steps.len(), 3, "{language}: {found}");
        assert!(steps[0].starts_with(RECORD), "{language}: {found}");
        assert_eq!(steps[1], "restart", "{language}: {found}");
        assert!(steps[2].starts_with(RECORD), "{language}: {found}");
        assert_eq!(failure["seed"], 1, "{language}");
        assert_eq!(failure["originalLength"], 4, "{language}: {found}");
        assert_eq!(failure["shrinkComplete"], true, "{language}");
        assert!(
            lane.asserts["counter-reset"].starts_with("failed: explore: seed 1: identity: "),
            "{language}: {}",
            lane.asserts["counter-reset"]
        );
        let replayed = result(lane, "counter-reset-replayed");
        assert_eq!(replayed["sequences"], 1, "{language}");
        assert_eq!(replayed["failure"], *failure, "{language}: {replayed}");
    }
}

#[test]
fn without_restarts_the_reset_counter_goes_undetected_and_results_keep_their_bytes() {
    for (language, lane) in both() {
        for name in ["counter-reset-without-restarts", "durable-without-restarts"] {
            let found = result(lane, name);
            assert_eq!(lane.asserts[name], "ok", "{language}: {name}: {found}");
            assert!(
                found.get("restarts").is_none(),
                "{language}: {name}: {found}"
            );
            assert_eq!(lane.processes[name], 3, "{language}: {name}");
        }
    }
}

#[test]
fn a_target_that_loses_its_state_on_restart_fails_at_the_restart() {
    for (language, lane) in both() {
        let found = result(lane, "volatile");
        assert_eq!(
            found["failure"]["message"],
            "view-rows: `restart.ledger.Entries` holds 0 row(s), the specification says 1 (step 2: restart)",
            "{language}: {found}"
        );
        let steps = trace(found);
        assert_eq!(steps.len(), 2, "{language}: {found}");
        assert!(steps[0].starts_with(RECORD), "{language}: {found}");
        assert_eq!(steps[1], "restart", "{language}: {found}");
    }
}

#[test]
fn a_target_that_cannot_restart_reports_restarts_unsupported_and_never_passes() {
    for (language, lane) in both() {
        for (name, reason) in [
            ("no-restart", "the target offers no restart"),
            ("no-restart-allowed", "the target offers no restart"),
            ("restart-refused", "this deployment cannot be restarted"),
        ] {
            let found = result(lane, name);
            assert!(
                found.get("failure").is_none(),
                "{language}: {name}: {found}"
            );
            // The exploration itself went on without restarts.
            assert_eq!(found["executed"], 12, "{language}: {name}: {found}");
            assert_eq!(
                found["restarts"],
                json!({"every": 2, "performed": 0, "unsupported": reason}),
                "{language}: {name}: {found}"
            );
            assert_eq!(
                lane.asserts[name],
                format!(
                    "failed: explore: restarts were requested every 2 step(s), and the target cannot restart: {reason}"
                ),
                "{language}: {name}"
            );
            assert_eq!(lane.processes[name], 3, "{language}: {name}");
        }
    }
}

#[test]
fn requested_restarts_that_no_sequence_reached_are_not_a_pass() {
    for (language, lane) in both() {
        let found = result(lane, "too-short");
        assert_eq!(
            found["restarts"],
            json!({"every": 2, "performed": 0}),
            "{language}: {found}"
        );
        assert_eq!(
            lane.asserts["too-short"],
            "failed: explore: restarts were requested every 2 step(s), and no sequence performed one",
            "{language}"
        );
    }
}

#[test]
fn a_negative_restart_interval_is_refused() {
    for (language, lane) in both() {
        assert_eq!(
            lane.asserts["negative"],
            "refused: the restart interval must be a whole number of steps, zero or more",
            "{language}"
        );
    }
}

#[test]
fn both_languages_report_the_same_result_for_every_case() {
    let (typescript, go) = lanes();
    for (name, result) in &go.results {
        assert_eq!(
            &typescript.results[name], result,
            "`{name}` differs between TypeScript and Go"
        );
        assert_eq!(typescript.asserts[name], go.asserts[name], "`{name}`");
        assert_eq!(typescript.processes[name], go.processes[name], "`{name}`");
    }
}

#[test]
fn a_restart_after_the_last_command_is_followed_by_one_more_command() {
    for (language, lane) in both() {
        // The counter lost on that restart is caught by the command after it.
        let found = result(lane, "trailing");
        let message = found["failure"]["message"].as_str().unwrap_or_default();
        assert!(
            message.starts_with(&format!(
                "identity: the target created \"{FIRST}\" again, over an existing record (step 3: {RECORD}"
            )),
            "{language}: {found}"
        );
        let steps = trace(found);
        assert_eq!(steps.len(), 3, "{language}: {found}");
        assert_eq!(steps[1], "restart", "{language}: {found}");
        assert_eq!(found["failure"]["seed"], 1, "{language}: {found}");
        assert_eq!(found["failure"]["originalLength"], 4, "{language}: {found}");
        // The restart was followed by a command, so it counts.
        assert_eq!(
            found["restarts"],
            json!({"every": 2, "performed": 1}),
            "{language}: {found}"
        );
        assert!(
            lane.asserts["trailing"].starts_with("failed: explore: seed 1: identity: "),
            "{language}: {}",
            lane.asserts["trailing"]
        );

        // A durable target passes, and each of its trailing restarts is performed.
        let durable = result(lane, "trailing-durable");
        assert_eq!(
            lane.asserts["trailing-durable"], "ok",
            "{language}: {durable}"
        );
        assert_eq!(durable["executed"], 9, "{language}: {durable}");
        assert_eq!(
            durable["restarts"],
            json!({"every": 2, "performed": 3}),
            "{language}: {durable}"
        );
        assert_eq!(lane.processes["trailing-durable"], 6, "{language}");

        // Restarting after every command adds exactly one command per sequence, and no restart
        // after it.
        let every = result(lane, "every-command");
        assert_eq!(lane.asserts["every-command"], "ok", "{language}: {every}");
        assert_eq!(every["executed"], 8, "{language}: {every}");
        assert_eq!(
            every["restarts"],
            json!({"every": 1, "performed": 6}),
            "{language}: {every}"
        );
        assert_eq!(lane.processes["every-command"], 8, "{language}");
    }
}

#[test]
fn the_read_after_a_restart_names_the_last_commands_token() {
    for (language, lane) in both() {
        let found = result(lane, "token");
        assert_eq!(lane.asserts["token"], "ok", "{language}: {found}");
        assert!(found.get("failure").is_none(), "{language}: {found}");
        assert_eq!(
            found["restarts"],
            json!({"every": 2, "performed": 6}),
            "{language}: {found}"
        );
    }
}

#[test]
fn concurrent_exploration_refuses_restart_every() {
    let lane = &lanes().0;
    assert_eq!(
        lane.asserts["concurrent-refused"],
        "refused: explore: `restartEvery` is set; concurrent exploration does not restart the \
         target, restarts are for `explore` only",
    );
    assert_eq!(lane.processes["concurrent-refused"], 0);
}
