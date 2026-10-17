//! Adversary pass 1 on beyond10x/ess#536: the explorers draw a command whose outcome `preserves:`
//! a supplied subject.
//!
//! `explore-preserves.yaml` reaches a preserving branch only through the held state. This file
//! drives `adversary-l2-preserves.yaml`, where a preserving branch sits under a `when_subject`
//! predicate reading the row and the input (`Retitle`), an input guard (`Touch`), and a held state
//! (`Publish`), with targets in both languages that change the row on
//! each no-op answer. It also reads the plan concurrent exploration draws from (Go, through an
//! in-package test), the one place the `concurrent` flag of `exploreEffectRefusal` decides
//! anything: under a held-state or row condition concurrent exploration has already left the
//! command out for the condition.
//!
//! A missing `tsc`, `node` or `go` panics rather than skipping.

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
use serde_json::Value;

const FIXTURE: &str = include_str!("fixtures/adversary-l2-preserves.yaml");

fn ir() -> EssIr {
    let mut sources = SourceMap::new();
    sources.insert("adversary-l2-preserves.yaml", FIXTURE);
    let raw = RawSpecFile::parse(FIXTURE).expect("the fixture parses");
    let specification =
        Specification::assemble(vec![(Source::new("adversary-l2-preserves.yaml"), raw)])
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

fn scratch(label: &str) -> support_scratch::Scratch {
    let root = support_scratch::Scratch::adopt(
        std::env::temp_dir().join(format!("ess-adv-l2-{}-{label}", std::process::id())),
    );
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

fn strings(value: &Value) -> Vec<&str> {
    value
        .as_array()
        .map(|items| items.iter().filter_map(Value::as_str).collect())
        .unwrap_or_default()
}

#[derive(Debug)]
struct Lane {
    results: BTreeMap<String, Value>,
    asserts: BTreeMap<String, String>,
    plans: BTreeMap<String, Value>,
    log: String,
}

fn read_lane(out: &Path, cases: &Value, log: String, plans: bool) -> Lane {
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
    let mut read = BTreeMap::new();
    if plans {
        for mode in ["sequential", "concurrent"] {
            let text = std::fs::read_to_string(out.join(format!("plan-{mode}.json")))
                .unwrap_or_else(|_| panic!("no `{mode}` plan:\n{log}"));
            read.insert(mode.to_owned(), serde_json::from_str(&text).expect("JSON"));
        }
    }
    Lane {
        results,
        asserts,
        plans: read,
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
        fixture("adversary-l2-preserves-target.mjs"),
        ts.join("adversary-l2-preserves-target.mjs"),
    )
    .expect("the target copies");
    let driver = read_fixture("explore-preconditions-driver.mjs").replace(
        "./explore-preconditions-target.mjs",
        "./adversary-l2-preserves-target.mjs",
    );
    write(&ts.join("adv-l2-driver.mjs"), &driver);
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
        .args(["--test", "adv-l2-driver.mjs"])
        .env("ESS_EXPLORE_CASES", cases.to_string())
        .env("ESS_EXPLORE_OUT", &out)
        .current_dir(&ts)
        .output()
        .expect("`node` is on PATH");
    let log = printed(&run);
    assert!(run.status.success(), "{log}");
    read_lane(&out, cases, log, false)
}

fn go(root: &Path, ir: &EssIr, cases: &Value) -> Lane {
    let module = root.join("go");
    for artifact in ess_conformance::go::emit_with_model(&suite(ir), ir).unwrap() {
        write(&module.join(&artifact.path), &artifact.contents);
    }
    write(
        &module.join("go.mod"),
        "module example.invalid/advkeep\n\ngo 1.24\n",
    );
    std::fs::copy(
        fixture("adversary_l2_preserves_target.go"),
        module.join("essconform/adversary_l2_preserves_target_test.go"),
    )
    .expect("the target copies");
    let driver = read_fixture("explore_preconditions_driver_test.go")
        .replace("newExplorePreTarget(one.Mode)", "newAdvL2Target(one.Mode)");
    write(&module.join("essconform/adv_l2_driver_test.go"), &driver);
    let out = root.join("out-go");
    std::fs::create_dir_all(&out).unwrap();
    let run = Command::new("go")
        .args([
            "test",
            "./essconform",
            "-run",
            "TestExplorePreconditions|TestAdversaryL2Plan",
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
    read_lane(&out, cases, log, true)
}

const MODES: &[(&str, &str)] = &[
    ("same-title-writes", "advkeep.docs.Retitle"),
    ("dry-run-writes", "advkeep.docs.Touch"),
    ("already-archives", "advkeep.docs.Publish"),
];

fn lanes() -> &'static (Lane, Lane) {
    static LANES: OnceLock<(Lane, Lane)> = OnceLock::new();
    LANES.get_or_init(|| {
        let ir = ir();
        let mut cases = vec![serde_json::json!({"name": "correct"})];
        for (mode, _) in MODES {
            cases.push(serde_json::json!({"name": mode, "mode": mode}));
        }
        let cases = Value::Array(cases);
        let root = scratch("lanes");
        let lanes = (typescript(&root, &ir, &cases), go(&root, &ir, &cases));
        std::fs::remove_dir_all(&root).ok();
        lanes
    })
}

fn each_lane() -> [(&'static str, &'static Lane); 2] {
    [("typescript", &lanes().0), ("go", &lanes().1)]
}

/// Every declared outcome of `adversary-l2-preserves.yaml`.
const EVERY_OUTCOME: &[&str] = &[
    "advkeep.docs.Archive/archived",
    "advkeep.docs.Create/created",
    "advkeep.docs.Publish/already-published",
    "advkeep.docs.Publish/not-draft",
    "advkeep.docs.Publish/published",
    "advkeep.docs.Retitle/retitled",
    "advkeep.docs.Retitle/same-title",
    "advkeep.docs.Touch/dry-run",
    "advkeep.docs.Touch/touched",
];

/// A correct target passes with nothing excluded and every branch reached, the preserving ones
/// under a row predicate, an input guard and a held state included.
#[test]
fn a_correct_target_passes_every_preserving_branch_in_both_languages() {
    for (language, lane) in each_lane() {
        let found = &lane.results["correct"];
        assert_eq!(
            lane.asserts["correct"], "ok",
            "{language}: {found}\n{}",
            lane.log
        );
        assert_eq!(
            found["excluded"],
            serde_json::json!([]),
            "{language}: {found}"
        );
        let mut reached = strings(&found["reached"]);
        reached.sort_unstable();
        assert_eq!(reached, EVERY_OUTCOME, "{language}: {found}");
        assert!(
            strings(&found["unreached"]).is_empty(),
            "{language}: {found}"
        );
    }
}

/// A target that answers the preserving branch and changes the row fails, ending on that command,
/// whichever condition selected the branch.
#[test]
fn a_target_that_changes_the_row_on_any_preserving_branch_fails() {
    for (language, lane) in each_lane() {
        for (mode, command) in MODES {
            let found = &lane.results[*mode];
            assert!(
                lane.asserts[*mode].starts_with("failed: explore: seed "),
                "{language}: `{mode}` passed: {}\n{found}",
                lane.asserts[*mode]
            );
            let trace = strings(&found["failure"]["trace"]);
            assert!(
                trace.last().is_some_and(|step| step.starts_with(command)),
                "{language}: `{mode}` ends on no `{command}` step: {}",
                found["failure"]
            );
        }
    }
}

#[test]
fn both_languages_report_the_same_result_for_every_preserving_case() {
    let (ts, go) = lanes();
    for name in ts.results.keys() {
        assert_eq!(ts.results[name], go.results[name], "`{name}` differs");
        assert_eq!(ts.asserts[name], go.asserts[name], "`{name}`");
    }
}

/// Concurrent exploration leaves `Touch` out because its `dry-run` branch preserves the subject:
/// the only condition concurrent exploration admits beside a preserve is one it reads from the
/// input, so this is the case that decides the flag. Sequential exploration draws every command.
#[test]
fn concurrent_exploration_leaves_out_a_preserve_under_an_input_guard_by_its_effect() {
    let plans = &lanes().1.plans;
    let sequential = &plans["sequential"];
    assert_eq!(
        sequential["excluded"],
        serde_json::json!([]),
        "{sequential}"
    );
    assert_eq!(
        strings(&sequential["drawn"]),
        [
            "advkeep.docs.Archive",
            "advkeep.docs.Create",
            "advkeep.docs.Publish",
            "advkeep.docs.Retitle",
            "advkeep.docs.Touch",
        ],
        "{sequential}"
    );
    let concurrent = &plans["concurrent"];
    let excluded: BTreeMap<&str, &str> = concurrent["excluded"]
        .as_array()
        .expect("a list")
        .iter()
        .map(|it| {
            (
                it["subject"].as_str().unwrap(),
                it["reason"].as_str().unwrap(),
            )
        })
        .collect();
    assert_eq!(
        excluded.get("advkeep.docs.Touch").copied(),
        Some("outcome `dry-run` preserves its subject from `supplied`"),
        "{concurrent}"
    );
    assert!(
        excluded.contains_key("advkeep.docs.Retitle")
            && excluded.contains_key("advkeep.docs.Publish"),
        "{concurrent}"
    );
    assert_eq!(
        strings(&concurrent["drawn"]),
        ["advkeep.docs.Archive", "advkeep.docs.Create"],
        "{concurrent}"
    );
}
