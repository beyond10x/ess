//! Adversary cases for exploration with restarts (beyond10x/ess#297), pass 1.
//!
//! Runs `tests/fixtures/restart.yaml` against `adversary-restart-target.mjs` and
//! `adversary_restart_target.go`, which add three modes to the unit's own targets: a counter that
//! is lost only on the second restart, a restart that answers without restarting, and a target that
//! mints one identity every time. The last also drives the base explorer (`d1026d1f0`, read with
//! `git show`) so that a run without `restartEvery` can be compared byte for byte with the explorer
//! before restarts existed.

mod support_scratch;

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
const BASE: &str = "d1026d1f0";

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

fn base_source(path: &str) -> String {
    let output = Command::new("git")
        .args(["-C", env!("CARGO_MANIFEST_DIR"), "show"])
        .arg(format!("{BASE}:crates/verify/ess-conformance/src/{path}"))
        .output()
        .expect("`git` is on PATH");
    assert!(output.status.success(), "{}", printed(&output));
    String::from_utf8(output.stdout).expect("UTF-8")
}

/// A mutated explorer source from `ESS_ADVERSARY_MUTANT_DIR`, for showing which mutant a case kills.
/// Unset in every ordinary run.
fn mutant(name: &str) -> Option<String> {
    let dir = std::env::var_os("ESS_ADVERSARY_MUTANT_DIR")?;
    Some(std::fs::read_to_string(Path::new(&dir).join(name)).expect("the mutant is readable"))
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

#[derive(Debug)]
struct Lane {
    bytes: BTreeMap<String, Option<String>>,
    asserts: BTreeMap<String, String>,
    processes: BTreeMap<String, u64>,
    log: String,
}

impl Lane {
    fn result(&self, name: &str) -> Value {
        let text = self.bytes[name].as_ref().unwrap_or_else(|| {
            panic!(
                "`{name}` returned no result: {}\n{}",
                self.asserts[name], self.log
            )
        });
        serde_json::from_str(text).expect("the result is JSON")
    }
}

fn read_lane(out: &Path, cases: &Value, log: String) -> Lane {
    let mut bytes = BTreeMap::new();
    let mut asserts = BTreeMap::new();
    let mut processes = BTreeMap::new();
    for case in cases.as_array().expect("a case list") {
        let name = case["name"].as_str().expect("a case name").to_owned();
        let json = std::fs::read_to_string(out.join(format!("{name}.json"))).ok();
        let assert = std::fs::read_to_string(out.join(format!("{name}.assert")))
            .unwrap_or_else(|_| panic!("`{name}` wrote no assertion:\n{log}"));
        let count = std::fs::read_to_string(out.join(format!("{name}.processes")))
            .unwrap_or_else(|_| panic!("`{name}` wrote no process count:\n{log}"));
        bytes.insert(name.clone(), json);
        asserts.insert(name.clone(), assert.trim_end().to_owned());
        processes.insert(name, count.trim_end().parse().expect("a count"));
    }
    Lane {
        bytes,
        asserts,
        processes,
        log,
    }
}

/// The TypeScript lane; with `base`, the emitted `explore.ts` is replaced by the base commit's.
fn typescript(root: &Path, ir: &EssIr, cases: &Value, base: bool) -> Lane {
    let package = root.join("typescript");
    for artifact in ess_conformance::ts::emit_with_model(&suite(ir), ir).unwrap() {
        let path = artifact.path.clone();
        let contents = if base && path.ends_with("src/explore.ts") {
            base_source("ts/explore.ts")
        } else if let Some(mutant) =
            mutant("explore.ts").filter(|_| path.ends_with("src/explore.ts"))
        {
            mutant
        } else {
            artifact.contents.clone()
        };
        write(&package.join(&path), &contents);
    }
    let ts = package.join("essconform");
    for name in [
        "adversary-restart-target.mjs",
        "adversary-restart-backend.mjs",
        "adversary-restart-driver.mjs",
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
        .expect("`tsc` is on PATH");
    assert!(compiled.status.success(), "{}", printed(&compiled));
    let out = root.join("out-typescript");
    std::fs::create_dir_all(&out).unwrap();
    let run = Command::new("node")
        .args(["--test", "adversary-restart-driver.mjs"])
        .env("ESS_EXPLORE_CASES", cases.to_string())
        .env("ESS_EXPLORE_OUT", &out)
        .current_dir(&ts)
        .output()
        .expect("`node` is on PATH");
    let log = printed(&run);
    assert!(run.status.success(), "{log}");
    read_lane(&out, cases, log)
}

/// The Go lane; with `base`, the emitted `explore.go` is replaced by the base commit's.
fn go(root: &Path, ir: &EssIr, cases: &Value, base: bool) -> Lane {
    let module = root.join("go");
    for artifact in ess_conformance::go::emit_with_model(&suite(ir), ir).unwrap() {
        let path = artifact.path.clone();
        let contents = if base && path.ends_with("explore.go") {
            base_source("go/explore.go")
        } else if let Some(mutant) = mutant("explore.go").filter(|_| path.ends_with("explore.go")) {
            mutant
        } else {
            artifact.contents.clone()
        };
        write(&module.join(&path), &contents);
    }
    write(
        &module.join("go.mod"),
        "module example.invalid/restart\n\ngo 1.24\n",
    );
    std::fs::copy(
        fixture("adversary_restart_target.go"),
        module.join("essconform/adversary_restart_target_test.go"),
    )
    .unwrap();
    std::fs::copy(
        fixture("adversary_restart_driver_test.go"),
        module.join("essconform/adversary_restart_driver_test.go"),
    )
    .unwrap();
    let out = root.join("out-go");
    std::fs::create_dir_all(&out).unwrap();
    let run = Command::new("go")
        .args([
            "test",
            "./essconform",
            "-run",
            "^TestAdversaryExploreRestart$",
            "-count=1",
            "-v",
        ])
        .env("ESS_EXPLORE_CASES", cases.to_string())
        .env("ESS_EXPLORE_OUT", &out)
        .env_remove("ESS_RESTART_BACKEND")
        .env("GOWORK", "off")
        .current_dir(&module)
        .output()
        .expect("`go` is on PATH");
    let log = printed(&run);
    assert!(run.status.success(), "{log}");
    read_lane(&out, cases, log)
}

/// Runs without `restartEvery`, compared with the base explorer.
fn plain_cases() -> Value {
    json!([
        {"name": "plain-durable", "mode": "durable", "options": {"seeds": 3, "steps": 4}},
        {"name": "plain-constant", "mode": "constant", "options": {"seeds": 3, "steps": 4}},
        {"name": "plain-constant-zero", "mode": "constant", "options": {"seeds": 3, "steps": 4, "restartEvery": 0}},
    ])
}

fn restart_cases() -> Value {
    json!([
        // A restart only after the last command of each sequence: nothing is created after one.
        {"name": "trailing-only", "mode": "counter-reset", "options": {"seeds": 3, "steps": 2, "restartEvery": 2}},
        // The counter survives the first restart and is lost on the second.
        {"name": "second-restart", "mode": "counter-reset-second", "options": {"seeds": 3, "steps": 6, "restartEvery": 2}},
        {"name": "second-restart-short", "mode": "counter-reset-second", "options": {"seeds": 3, "steps": 4, "restartEvery": 2}},
        // `restart` answers and nothing restarts: the counter-reset backend keeps its counter.
        {"name": "pretend", "mode": "pretend", "options": {"seeds": 3, "steps": 4, "restartEvery": 2}},
        // Every read must name the last command's consistency token, the read after a restart included.
        {"name": "token", "mode": "token", "options": {"seeds": 3, "steps": 4, "restartEvery": 2}},
    ])
}

fn all_cases() -> Value {
    let mut all = plain_cases().as_array().unwrap().clone();
    all.extend(restart_cases().as_array().unwrap().iter().cloned());
    Value::Array(all)
}

struct Lanes {
    typescript: Lane,
    go: Lane,
    base_typescript: Lane,
    base_go: Lane,
}

fn lanes() -> &'static Lanes {
    static LANES: OnceLock<Lanes> = OnceLock::new();
    LANES.get_or_init(|| {
        let ir = ir();
        let root = support_scratch::Scratch::adopt(
            std::env::temp_dir().join(format!("ess-adv-restart-{}", std::process::id())),
        );
        let _ = std::fs::remove_dir_all(&root);
        let lanes = Lanes {
            typescript: typescript(&root.join("head"), &ir, &all_cases(), false),
            go: go(&root.join("head"), &ir, &all_cases(), false),
            base_typescript: typescript(&root.join("base"), &ir, &plain_cases(), true),
            base_go: go(&root.join("base"), &ir, &plain_cases(), true),
        };
        std::fs::remove_dir_all(&root).ok();
        lanes
    })
}

fn both() -> [(&'static str, &'static Lane); 2] {
    [("typescript", &lanes().typescript), ("go", &lanes().go)]
}

fn trace(found: &Value) -> Vec<String> {
    found["failure"]["trace"]
        .as_array()
        .unwrap_or_else(|| panic!("a failure with a trace: {found}"))
        .iter()
        .map(|step| {
            let line = step.as_str().expect("a trace line");
            if line == "restart" {
                "restart".to_owned()
            } else {
                line.split(' ').next().unwrap().to_owned()
            }
        })
        .collect()
}

/// Acceptance: "a counter-reset faulty target must fail". With `restartEvery` equal to `steps`, every
/// restart follows the last command of its sequence, so no creation is ever checked after one; the
/// explorer reports restarts performed and the counter-reset target passes.
#[test]
fn restarts_after_only_the_last_command_must_not_pass_a_counter_reset_target() {
    for (language, lane) in both() {
        let found = lane.result("trailing-only");
        // Since every restart is followed by a command (#297 correction 1), seed 1 already fails
        // after its first restart and exploration stops there: one restart performed, and at
        // least one real process restart behind it.
        assert_eq!(
            found["restarts"],
            json!({"every": 2, "performed": 1}),
            "{language}: {found}"
        );
        assert!(lane.processes["trailing-only"] >= 2, "{language}");
        assert!(
            lane.asserts["trailing-only"].starts_with("failed: "),
            "{language}: a counter-reset target passed with {} restart(s), none of them followed by a command: `{}`",
            found["restarts"]["performed"],
            lane.asserts["trailing-only"]
        );
    }
}

/// The same hole from the brief's angle: a counter lost only on the second restart, explored with
/// the unit's own `{steps: 4, restartEvery: 2}`. The second restart is the trailing one.
#[test]
fn a_counter_lost_on_the_second_restart_must_not_pass_four_steps_restarted_every_two() {
    for (language, lane) in both() {
        let found = lane.result("second-restart-short");
        assert_eq!(
            found["restarts"],
            // Seed 1 fails after its second restart (#297 correction 1), so two were performed.
            json!({"every": 2, "performed": 2}),
            "{language}: {found}"
        );
        assert!(
            lane.asserts["second-restart-short"].starts_with("failed: "),
            "{language}: `{}`",
            lane.asserts["second-restart-short"]
        );
    }
}

/// With a command after the second restart, the lost counter is caught, and the shrinker keeps both
/// restarts and drops every command it can.
#[test]
fn a_counter_lost_on_the_second_restart_fails_and_shrinks_to_both_restarts() {
    for (language, lane) in both() {
        let found = lane.result("second-restart");
        let message = found["failure"]["message"].as_str().unwrap_or_default();
        assert!(
            message.starts_with(
                "identity: the target created \"00000000-0000-4000-8000-000000000001\" again, over an existing record (step 4: "
            ),
            "{language}: {found}"
        );
        assert_eq!(
            trace(&found),
            [
                "restart.ledger.RecordEntry",
                "restart",
                "restart",
                "restart.ledger.RecordEntry"
            ],
            "{language}: {found}"
        );
        assert_eq!(found["failure"]["originalLength"], 7, "{language}: {found}");
    }
    let (typescript, go) = (&lanes().typescript, &lanes().go);
    assert_eq!(
        typescript.result("second-restart"),
        go.result("second-restart")
    );
}

/// A restart that answers without restarting cannot be told from one that restarts: the result says
/// six restarts were performed while only one process per sequence ever ran. Documents the limit;
/// green.
#[test]
fn a_restart_that_does_not_restart_is_reported_as_performed() {
    for (language, lane) in both() {
        let found = lane.result("pretend");
        assert_eq!(lane.asserts["pretend"], "ok", "{language}: {found}");
        assert_eq!(
            found["restarts"],
            json!({"every": 2, "performed": 6}),
            "{language}: {found}"
        );
        assert_eq!(lane.processes["pretend"], 3, "{language}");
    }
}

/// Without `restartEvery`, or with it zero, the result bytes are the base explorer's, failures and
/// shrinking included.
#[test]
fn results_without_restarts_are_byte_identical_to_the_base_explorer() {
    let lanes = lanes();
    for (language, head, base) in [
        ("typescript", &lanes.typescript, &lanes.base_typescript),
        ("go", &lanes.go, &lanes.base_go),
    ] {
        for case in plain_cases().as_array().unwrap() {
            let name = case["name"].as_str().unwrap();
            assert!(head.bytes[name].is_some(), "{language}: {name}");
            assert_eq!(head.bytes[name], base.bytes[name], "{language}: {name}");
            assert_eq!(head.asserts[name], base.asserts[name], "{language}: {name}");
        }
        assert!(
            head.result("plain-constant")["failure"].is_object(),
            "{language}: the comparison covers a failure"
        );
    }
}

#[test]
fn both_languages_agree_on_every_adversary_case() {
    let (typescript, go) = (&lanes().typescript, &lanes().go);
    for case in all_cases().as_array().unwrap() {
        let name = case["name"].as_str().unwrap();
        assert_eq!(typescript.result(name), go.result(name), "`{name}`");
        assert_eq!(typescript.asserts[name], go.asserts[name], "`{name}`");
        assert_eq!(typescript.processes[name], go.processes[name], "`{name}`");
    }
}

/// After a restart every view is read again at the last command's consistency token, as the design
/// says. Nothing in `explore_restart.rs` returns a token, so reading at `""` after a restart passes
/// it; this target refuses such a read.
#[test]
fn a_restart_reads_every_view_at_the_last_commands_token() {
    for (language, lane) in both() {
        let found = lane.result("token");
        assert!(found.get("failure").is_none(), "{language}: {found}");
        assert_eq!(lane.asserts["token"], "ok", "{language}: {found}");
        assert_eq!(
            found["restarts"],
            json!({"every": 2, "performed": 6}),
            "{language}: {found}"
        );
    }
}
