//! The generated explorer runs a system's `preconditions:` before every sequence (ess/15,
//! beyond10x/ess#152), in both emitted languages.
//!
//! The fixture is `tests/fixtures/explore-preconditions.yaml`: a command that runs only inside an
//! open session, and the session-opening command a precondition sends. Without the preconditions
//! the model starts empty while the target holds a user row, and the explorer disagreed at step 1;
//! with them the model starts from the row they leave. A precondition the target refuses is a
//! setup failure the exploration reports as refused, never a disagreement.
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

const FIXTURE: &str = include_str!("fixtures/explore-preconditions.yaml");

/// The same system, where opening the session takes a list of structs the precondition sends as a
/// literal (beyond10x/ess#205). The two targets serve both fixtures.
const LIST_FIXTURE: &str = include_str!("fixtures/explore-preconditions-list.yaml");

fn ir() -> EssIr {
    ir_of(FIXTURE)
}

fn ir_of(fixture: &str) -> EssIr {
    let mut sources = SourceMap::new();
    sources.insert("explore-preconditions.yaml", fixture);
    let raw = RawSpecFile::parse(fixture).expect("the fixture parses");
    let specification =
        Specification::assemble(vec![(Source::new("explore-preconditions.yaml"), raw)])
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
        "ess-explore-preconditions-{}-{label}",
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

/// What one lane reported: each case's result and assertion, and the runner log.
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
        .expect("`tsc` is on PATH: the explorer lane does not skip");
    assert!(compiled.status.success(), "{}", printed(&compiled));
    let out = root.join("out-typescript");
    std::fs::create_dir_all(&out).unwrap();
    let run = Command::new("node")
        .args(["--test", "explore-preconditions-driver.mjs"])
        .env("ESS_EXPLORE_CASES", cases.to_string())
        .env("ESS_EXPLORE_OUT", &out)
        .current_dir(&ts)
        .output()
        .expect("`node` is on PATH: the explorer lane does not skip");
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
        .expect("`go` is on PATH: the explorer lane does not skip");
    let log = printed(&run);
    assert!(run.status.success(), "{log}");
    for case in cases.as_array().unwrap() {
        let line = format!(
            "--- PASS: TestExplorePreconditions/{}",
            case["name"].as_str().unwrap()
        );
        assert!(
            log.lines().any(|it| it.trim_start().starts_with(&line)),
            "`{line}` is not in the go log:\n{log}"
        );
    }
    read_lane(&out, cases, log)
}

fn cases() -> Value {
    json!([
        {"name": "session", "options": {"seeds": 20, "steps": 20}},
        {"name": "refuses-session", "mode": "refuses-session", "options": {"seeds": 2, "steps": 5}},
    ])
}

/// The list fixture's cases. No sequence can draw a list, so `OpenSession` is excluded from
/// exploration and only the precondition sends it: its outcome is accepted as excluded.
fn list_cases() -> Value {
    json!([
        {"name": "session", "options": {"seeds": 20, "steps": 20}, "allowExcluded": true},
        {"name": "drops-accounts", "mode": "drops-accounts", "options": {"seeds": 2, "steps": 5}, "allowExcluded": true},
    ])
}

/// Both lanes over the list fixture, run once for every test below.
fn list_lanes() -> &'static (Lane, Lane) {
    static LANES: OnceLock<(Lane, Lane)> = OnceLock::new();
    LANES.get_or_init(|| {
        let ir = ir_of(LIST_FIXTURE);
        let cases = list_cases();
        let root = scratch("list-lanes");
        let lanes = (typescript(&root, &ir, &cases), go(&root, &ir, &cases));
        std::fs::remove_dir_all(&root).ok();
        lanes
    })
}

/// Both lanes, run once for every test below.
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

#[test]
fn the_model_starts_from_the_state_the_preconditions_leave() {
    for lane in [&lanes().0, &lanes().1] {
        assert_eq!(lane.asserts["session"], "ok", "{}", lane.log);
        let found = lane.results["session"]
            .as_ref()
            .unwrap_or_else(|| panic!("a result: {}", lane.log));
        assert!(found.get("failure").is_none(), "{found}");
        assert!(
            found["reached"]
                .as_array()
                .unwrap()
                .iter()
                .any(|reached| reached == "explorepre.desk.ClearWrapUp/cleared"),
            "the command that needs the session is reached: {found}"
        );
    }
}

#[test]
fn a_refused_precondition_is_a_setup_failure_and_not_a_disagreement() {
    for lane in [&lanes().0, &lanes().1] {
        let assert = &lane.asserts["refuses-session"];
        assert!(
            assert
                .starts_with("refused: precondition `explorepre.desk.OpenSession` failed as setup"),
            "{assert}\n{}",
            lane.log
        );
        assert!(lane.results["refuses-session"].is_none());
    }
}

/// beyond10x/ess#205: the session command takes a list, and the precondition sends it as a literal
/// rather than through a fixture exploration cannot resolve.
#[test]
fn issue_205_a_precondition_sends_a_list_literal() {
    let ir = ir_of(LIST_FIXTURE);
    let precondition = &ir.preconditions()[0];
    assert!(precondition.fixtures.is_empty(), "{precondition:?}");
    assert!(
        matches!(&precondition.input["accounts"], ess_primitives::node::Node::Seq(items) if items.len() == 1),
        "{precondition:?}"
    );
}

/// Synthesis opens every scenario with the precondition, sending the list as written and resolving
/// no fixture for it.
#[test]
fn issue_205_every_scenario_opens_by_sending_the_list_literal() {
    let suite = suite(&ir_of(LIST_FIXTURE));
    assert!(!suite.scenarios.is_empty());
    for (id, scenario) in &suite.scenarios {
        let ess_conformance::scenario::ScenarioStep::ExecuteCommand { command, input, .. } =
            &scenario.steps[0]
        else {
            panic!(
                "the scenario opens with the precondition: {:?}",
                scenario.steps
            );
        };
        assert_eq!(command.to_string(), "explorepre.desk.OpenSession");
        assert_eq!(
            serde_json::to_value(&input["accounts"]).unwrap(),
            json!({"kind": "literal", "value": [{"account_id": "a1", "name": "First"}]}),
            "{id}"
        );
    }
}

/// beyond10x/ess#205: the explorer sends a precondition whose command it cannot draw an input for,
/// and the model starts from the row it leaves, list included.
#[test]
fn issue_205_the_explorer_runs_a_precondition_whose_command_takes_a_list() {
    for lane in [&list_lanes().0, &list_lanes().1] {
        assert_eq!(lane.asserts["session"], "ok", "{}", lane.log);
        let found = lane.results["session"]
            .as_ref()
            .unwrap_or_else(|| panic!("a result: {}", lane.log));
        assert!(found.get("failure").is_none(), "{found}");
        assert!(
            found["reached"]
                .as_array()
                .unwrap()
                .iter()
                .any(|reached| reached == "explorepre.desk.ClearWrapUp/cleared"),
            "the command that needs the session is reached: {found}"
        );
        assert!(
            found["excluded"]
                .as_array()
                .unwrap()
                .iter()
                .any(|excluded| {
                    excluded["subject"] == "explorepre.desk.OpenSession"
                        && excluded["reason"]
                            .as_str()
                            .is_some_and(|reason| reason.contains("accounts"))
                }),
            "no sequence draws the list, so the command stays excluded from them: {found}"
        );
    }
}

/// The list the precondition sends is the list the model holds: a target that stores a different
/// one is a setup failure naming the field.
#[test]
fn a_target_that_drops_the_precondition_list_is_caught() {
    for lane in [&list_lanes().0, &list_lanes().1] {
        let assert = &lane.asserts["drops-accounts"];
        assert!(
            assert
                .starts_with("refused: precondition `explorepre.desk.OpenSession` failed as setup")
                && assert.contains("view-field: `explorepre.desk.Users`.accounts"),
            "{assert}\n{}",
            lane.log
        );
    }
}

#[test]
fn both_languages_report_the_same_result_over_the_list_fixture() {
    let (typescript, go) = list_lanes();
    assert_eq!(typescript.asserts, go.asserts);
    assert_eq!(typescript.results, go.results);
}

#[test]
fn both_languages_report_the_same_result() {
    let (typescript, go) = lanes();
    assert_eq!(typescript.asserts, go.asserts);
    assert_eq!(typescript.results, go.results);
}
