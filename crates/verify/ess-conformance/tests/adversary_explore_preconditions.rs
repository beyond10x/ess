//! Adversary cases for structured precondition literals in the generated explorers
//! (beyond10x/ess#205).
//!
//! The fixture is `tests/fixtures/adversary-explore-preconditions.yaml`: the session command takes
//! a `Timestamp`, a guarded refusal, and a list of structs whose leaves are an absent optional, an
//! integer past 2^53, a map with keys outside the Basic Multilingual Plane, an empty map and text
//! authored out of field order. Exploration draws neither the timestamp nor the list, so the
//! command is excluded from sequences and only the precondition sends it. The targets are the ones
//! `explore_preconditions.rs` uses.
//!
//! A missing `tsc`, `node` or `go` panics rather than skipping.

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

const FIXTURE: &str = include_str!("fixtures/adversary-explore-preconditions.yaml");

fn ir() -> EssIr {
    let mut sources = SourceMap::new();
    sources.insert("adversary-explore-preconditions.yaml", FIXTURE);
    let raw = RawSpecFile::parse(FIXTURE).expect("the fixture parses");
    let specification = Specification::assemble(vec![(
        Source::new("adversary-explore-preconditions.yaml"),
        raw,
    )])
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
    let root = std::env::temp_dir().join(format!(
        "ess-adversary-explore-preconditions-{}-{label}",
        std::process::id()
    ));
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

#[derive(Debug)]
struct Lane {
    results: BTreeMap<String, Option<Value>>,
    asserts: BTreeMap<String, String>,
    log: String,
}

fn read_lane(out: &Path, cases: &Value, log: String) -> Lane {
    let mut results = BTreeMap::new();
    let mut asserts = BTreeMap::new();
    for case in cases.as_array().expect("a case list") {
        let name = case["name"].as_str().expect("a case name").to_owned();
        let json = std::fs::read_to_string(out.join(format!("{name}.json")))
            .ok()
            .map(|text| serde_json::from_str(&text).expect("the result is JSON"));
        let assert = std::fs::read_to_string(out.join(format!("{name}.assert")))
            .unwrap_or_else(|_| panic!("`{name}` wrote no assertion:\n{log}"));
        results.insert(name.clone(), json);
        asserts.insert(name, assert.trim_end().to_owned());
    }
    Lane {
        results,
        asserts,
        log,
    }
}

fn typescript(root: &Path, ir: &EssIr, cases: &Value) -> Lane {
    let package = root.join("typescript");
    for artifact in ess_conformance::ts::emit_with_model(&suite(ir), ir).unwrap() {
        write(&package.join(&artifact.path), &artifact.contents);
    }
    let ts = package.join("essconform");
    for name in [
        "explore-preconditions-target.mjs",
        "explore-preconditions-driver.mjs",
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
        .args(["--test", "explore-preconditions-driver.mjs"])
        .env("ESS_EXPLORE_CASES", cases.to_string())
        .env("ESS_EXPLORE_OUT", &out)
        .current_dir(&ts)
        .output()
        .expect("`node` is on PATH");
    let log = printed(&run);
    assert!(run.status.success(), "{log}");
    read_lane(&out, cases, log)
}

fn go(root: &Path, ir: &EssIr, cases: &Value) -> Lane {
    let module = root.join("go");
    for artifact in ess_conformance::go::emit_with_model(&suite(ir), ir).unwrap() {
        write(&module.join(&artifact.path), &artifact.contents);
    }
    write(
        &module.join("go.mod"),
        "module example.invalid/explorepre\n\ngo 1.24\n",
    );
    std::fs::copy(
        fixture("explore_preconditions_target.go"),
        module.join("essconform/explore_preconditions_target_test.go"),
    )
    .unwrap();
    std::fs::copy(
        fixture("explore_preconditions_driver_test.go"),
        module.join("essconform/explore_preconditions_driver_test.go"),
    )
    .unwrap();
    let out = root.join("out-go");
    std::fs::create_dir_all(&out).unwrap();
    let run = Command::new("go")
        .args([
            "test",
            "./essconform",
            "-run",
            "TestExplorePreconditions",
            "-count=1",
            "-v",
        ])
        .env("ESS_EXPLORE_CASES", cases.to_string())
        .env("ESS_EXPLORE_OUT", &out)
        .env("GOWORK", "off")
        .current_dir(&module)
        .output()
        .expect("`go` is on PATH");
    let log = printed(&run);
    assert!(run.status.success(), "{log}");
    read_lane(&out, cases, log)
}

fn cases() -> Value {
    json!([
        {"name": "session", "options": {"seeds": 10, "steps": 10}, "allowExcluded": true},
        {"name": "drops-accounts", "mode": "drops-accounts", "options": {"seeds": 2, "steps": 5}, "allowExcluded": true},
    ])
}

fn lanes() -> &'static (Lane, Lane) {
    static LANES: OnceLock<(Lane, Lane)> = OnceLock::new();
    LANES.get_or_init(|| {
        let ir = ir();
        let cases = cases();
        let root = scratch("lanes");
        let lanes = (typescript(&root, &ir, &cases), go(&root, &ir, &cases));
        std::fs::remove_dir_all(&root).ok();
        lanes
    })
}

/// The Rust suite sends the literal as written: the integer past 2^53 exactly.
#[test]
fn the_rust_suite_sends_the_literal_exactly() {
    let suite = suite(&ir());
    let (_, scenario) = suite.scenarios.iter().next().expect("a scenario");
    let ess_conformance::scenario::ScenarioStep::ExecuteCommand { input, .. } = &scenario.steps[0]
    else {
        panic!(
            "the scenario opens with the precondition: {:?}",
            scenario.steps
        );
    };
    let sent = serde_json::to_string(&input["accounts"]).unwrap();
    assert!(sent.contains("9007199254740993"), "{sent}");
}

/// The explorer decides the guard over the timestamp the precondition sends and takes `opened`.
#[test]
fn the_explorer_takes_the_precondition_branch_the_specification_selects() {
    for lane in [&lanes().0, &lanes().1] {
        assert_eq!(lane.asserts["session"], "ok", "{}", lane.log);
    }
}

/// No sequence draws the precondition's command: it is never a reached outcome of exploration.
#[test]
fn no_sequence_sends_the_sendable_command() {
    for lane in [&lanes().0, &lanes().1] {
        let found = lane.results["session"]
            .as_ref()
            .unwrap_or_else(|| panic!("a result: {}", lane.log));
        let reached = found["reached"].as_array().expect("reached");
        assert!(
            !reached.iter().any(|it| it
                .as_str()
                .is_some_and(|it| it.starts_with("explorepre.desk.OpenSession/"))),
            "{found}"
        );
    }
}

/// A target that drops the list is reported with the same bytes in both languages, and the value
/// the specification says is the one the Rust suite sends.
#[test]
fn the_dropped_list_is_reported_alike_and_exactly() {
    let (typescript, go) = lanes();
    assert_eq!(
        typescript.asserts["drops-accounts"],
        go.asserts["drops-accounts"]
    );
    assert!(
        go.asserts["drops-accounts"].contains("9007199254740993"),
        "{}",
        go.asserts["drops-accounts"]
    );
}

#[test]
fn both_languages_report_the_same_result() {
    let (typescript, go) = lanes();
    assert_eq!(typescript.asserts, go.asserts);
    assert_eq!(typescript.results, go.results);
}
