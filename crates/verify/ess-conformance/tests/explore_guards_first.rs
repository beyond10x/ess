//! The generated explorer decides a step in Entity Runtime's order (beyond10x/ess#235), in both
//! emitted languages.
//!
//! Entity Runtime sorts a command's branches into input-guarded refusals, the other guarded
//! branches, the default and the wrong-state branch, takes the first whose guard holds, and answers
//! the wrong-state branch only where the branch it took moves from a state no move of the command
//! starts from (`ess-entity-runtime` `lower_command`, entity-core `select_outcome` and
//! `admit_state`). The explorer decided the wrong-state answer before any guard, so a target in
//! that order disagreed wherever a refused input met a subject in a wrong state.
//!
//! The fixture is `tests/fixtures/explore-guards-first.yaml`; the targets are
//! `explore-guards-first-target.mjs` and `explore_guards_first_target.go`, switched by mode, run by
//! the preconditions drivers with the target swapped in.
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

const FIXTURE: &str = include_str!("fixtures/explore-guards-first.yaml");

fn ir() -> EssIr {
    let mut sources = SourceMap::new();
    sources.insert("explore-guards-first.yaml", FIXTURE);
    let raw = RawSpecFile::parse(FIXTURE).expect("the fixture parses");
    let specification =
        Specification::assemble(vec![(Source::new("explore-guards-first.yaml"), raw)])
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

fn read_fixture(name: &str) -> String {
    std::fs::read_to_string(fixture(name)).expect("the fixture reads")
}

fn scratch(label: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "ess-explore-guards-first-{}-{label}",
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
    results: BTreeMap<String, Value>,
    asserts: BTreeMap<String, String>,
    log: String,
}

fn read_lane(out: &Path, cases: &Value, log: String) -> Lane {
    let mut results = BTreeMap::new();
    let mut asserts = BTreeMap::new();
    for case in cases.as_array().expect("a case list") {
        let name = case["name"].as_str().expect("a case name").to_owned();
        let assert = std::fs::read_to_string(out.join(format!("{name}.assert")))
            .unwrap_or_else(|_| panic!("`{name}` wrote no assertion:\n{log}"));
        let text = std::fs::read_to_string(out.join(format!("{name}.json")))
            .unwrap_or_else(|_| panic!("`{name}` wrote no result ({assert}):\n{log}"));
        results.insert(name.clone(), serde_json::from_str(&text).expect("JSON"));
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
    std::fs::copy(
        fixture("explore-guards-first-target.mjs"),
        ts.join("explore-guards-first-target.mjs"),
    )
    .expect("the target copies");
    let driver = read_fixture("explore-preconditions-driver.mjs").replace(
        "./explore-preconditions-target.mjs",
        "./explore-guards-first-target.mjs",
    );
    write(&ts.join("guards-first-driver.mjs"), &driver);
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
        .args(["--test", "guards-first-driver.mjs"])
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
        "module example.invalid/exploreorder\n\ngo 1.24\n",
    );
    std::fs::copy(
        fixture("explore_guards_first_target.go"),
        module.join("essconform/explore_guards_first_target_test.go"),
    )
    .expect("the target copies");
    let driver = read_fixture("explore_preconditions_driver_test.go").replace(
        "newExplorePreTarget(one.Mode)",
        "newExploreOrderTarget(one.Mode)",
    );
    write(
        &module.join("essconform/explore_guards_first_driver_test.go"),
        &driver,
    );
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

/// Every mode, run once per lane: `(typescript, go)`.
fn lanes() -> &'static (Lane, Lane) {
    static LANES: OnceLock<(Lane, Lane)> = OnceLock::new();
    LANES.get_or_init(|| {
        let ir = ir();
        let cases = json!([
            {"name": "correct"},
            {"name": "state-first", "mode": "state-first"},
            {"name": "last-refusal", "mode": "last-refusal"},
            {"name": "accept-first", "mode": "accept-first"},
        ]);
        let root = scratch("lanes");
        let lanes = (typescript(&root, &ir, &cases), go(&root, &ir, &cases));
        std::fs::remove_dir_all(&root).ok();
        lanes
    })
}

fn each_lane() -> [(&'static str, &'static Lane); 2] {
    [("typescript", &lanes().0), ("go", &lanes().1)]
}

fn strings(value: &Value) -> Vec<&str> {
    value
        .as_array()
        .unwrap_or_else(|| panic!("a list, not {value}"))
        .iter()
        .map(|item| item.as_str().expect("a string"))
        .collect()
}

const EVERY_OUTCOME: [&str; 10] = [
    "exploreorder.desk.Hold/capped",
    "exploreorder.desk.Hold/held",
    "exploreorder.desk.Hold/negative",
    "exploreorder.desk.Hold/tiny",
    "exploreorder.desk.Hold/unrated",
    "exploreorder.desk.Hold/wrong-state",
    "exploreorder.desk.Open/opened",
    "exploreorder.desk.Release/noted",
    "exploreorder.desk.Release/released",
    "exploreorder.desk.Release/wrong-state",
];

#[test]
fn a_target_in_entity_runtime_order_passes_and_reaches_every_branch() {
    for (language, lane) in each_lane() {
        let found = &lane.results["correct"];
        assert_eq!(
            lane.asserts["correct"], "ok",
            "{language}: {found}\n{}",
            lane.log
        );
        assert_eq!(strings(&found["reached"]), EVERY_OUTCOME, "{language}");
        assert!(
            found["excluded"].as_array().is_some_and(Vec::is_empty),
            "{language}: {found}"
        );
        // The only draws left undecided are an input no refusal claims naming a ticket nobody
        // holds; never a refusal beside a state, two refusals, or a refusal beside `held`.
        for draw in strings(&found["ambiguous"]) {
            assert!(
                draw.ends_with(": no record for the supplied instance"),
                "{language}: `{draw}` was redrawn\n{found}"
            );
        }
    }
    assert_eq!(
        lanes().0.results["correct"],
        lanes().1.results["correct"],
        "typescript and go explore the same seeds differently"
    );
}

/// Each target that departs from Entity Runtime's order is caught, with the branch the model
/// expected named in the failure.
#[test]
fn a_target_out_of_entity_runtime_order_is_caught() {
    for (mode, expected) in [
        (
            "state-first",
            ["exploreorder.desk.Hold", "exploreorder.desk.Release"],
        ),
        ("last-refusal", ["exploreorder.desk.Hold", "tiny"]),
        ("accept-first", ["exploreorder.desk.Hold", "capped"]),
    ] {
        for (language, lane) in each_lane() {
            let found = &lane.results[mode];
            assert!(
                lane.asserts[mode].starts_with("failed: "),
                "{language}: `{mode}` passed: {}\n{found}",
                lane.asserts[mode]
            );
            let failure = &found["failure"];
            let text = failure.to_string();
            let named = expected.iter().filter(|it| text.contains(**it)).count();
            match mode {
                "state-first" => assert!(named >= 1, "{language}: {mode}: {failure}"),
                _ => assert_eq!(named, 2, "{language}: {mode}: {failure}"),
            }
        }
    }
}
