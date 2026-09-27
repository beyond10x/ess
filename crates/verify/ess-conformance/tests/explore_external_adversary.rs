//! Adversary cases for the explorer's external branches (beyond10x/ess#156), both languages.
//!
//! 1. `explore-external-adversary.yaml`: `Close` has an ordinary branch from `Open` and an external
//!    branch from `Held`. From `Held` only the external branch applies; the explorer must take it.
//! 2. A mutant of the "an arrangement may stay in force" rule, applied to the emitted explorer: the
//!    ordinary branch is always offered. The existing `arranges` double must then disagree.
//!
//! A missing `tsc`, `node` or `go` panics rather than skipping.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_conformance::scenario::ConformanceSuite;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use serde_json::{json, Value};

fn ir_of(name: &str, text: &str) -> EssIr {
    let mut sources = SourceMap::new();
    sources.insert(name, text);
    let raw = RawSpecFile::parse(text).expect("the fixture parses");
    let specification = Specification::assemble(vec![(Source::new(name), raw)])
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
        "ess-explore-external-adversary-{}-{label}",
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

struct Lane {
    results: BTreeMap<String, Value>,
    log: String,
}

fn read_lane(out: &Path, cases: &Value, log: String) -> Lane {
    let mut results = BTreeMap::new();
    for case in cases.as_array().unwrap() {
        let name = case["name"].as_str().unwrap().to_owned();
        let text = std::fs::read_to_string(out.join(format!("{name}.json")))
            .unwrap_or_else(|_| panic!("`{name}` wrote no result:\n{log}"));
        results.insert(name, serde_json::from_str(&text).unwrap());
    }
    Lane { results, log }
}

/// The emitted explorer, with `mutate` applied to its source.
struct Setup<'a> {
    ir: EssIr,
    ts_target: &'a str,
    go_target: &'a str,
    go_constructor: &'a str,
    mutate: &'a dyn Fn(&str, String) -> String,
}

fn typescript(root: &Path, setup: &Setup, cases: &Value) -> Lane {
    let package = root.join("typescript");
    for artifact in ess_conformance::ts::emit_with_model(&suite(&setup.ir), &setup.ir).unwrap() {
        let contents = (setup.mutate)(&artifact.path, artifact.contents.clone());
        write(&package.join(&artifact.path), &contents);
    }
    let ts = package.join("essconform");
    std::fs::copy(fixture(setup.ts_target), ts.join(setup.ts_target)).unwrap();
    let driver = read_fixture("explore-external-driver.mjs").replace(
        "./explore-external-target.mjs",
        &format!("./{}", setup.ts_target),
    );
    write(&ts.join("adversary-driver.mjs"), &driver);
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
        .args(["--test", "adversary-driver.mjs"])
        .env("ESS_EXPLORE_CASES", cases.to_string())
        .env("ESS_EXPLORE_OUT", &out)
        .current_dir(&ts)
        .output()
        .expect("`node` is on PATH");
    let log = printed(&run);
    assert!(run.status.success(), "{log}");
    read_lane(&out, cases, log)
}

fn go(root: &Path, setup: &Setup, cases: &Value) -> Lane {
    let module = root.join("go");
    for artifact in ess_conformance::go::emit_with_model(&suite(&setup.ir), &setup.ir).unwrap() {
        let contents = (setup.mutate)(&artifact.path, artifact.contents.clone());
        write(&module.join(&artifact.path), &contents);
    }
    write(
        &module.join("go.mod"),
        "module example.invalid/exploreadv\n\ngo 1.24\n",
    );
    std::fs::copy(
        fixture(setup.go_target),
        module.join("essconform/adversary_target_test.go"),
    )
    .unwrap();
    let driver = read_fixture("explore_external_driver_test.go")
        .replace("newExploreExtTarget(one.Mode)", setup.go_constructor);
    write(&module.join("essconform/adversary_driver_test.go"), &driver);
    let out = root.join("out-go");
    std::fs::create_dir_all(&out).unwrap();
    let run = Command::new("go")
        .args([
            "test",
            "./essconform",
            "-run",
            "TestExploreExternal",
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

fn both(label: &str, setup: &Setup, cases: &Value) -> (Lane, Lane) {
    let root = scratch(label);
    let lanes = (typescript(&root, setup, cases), go(&root, setup, cases));
    std::fs::remove_dir_all(&root).ok();
    lanes
}

fn strings(value: &Value) -> Vec<&str> {
    value
        .as_array()
        .unwrap_or_else(|| panic!("a list, not {value}"))
        .iter()
        .map(|item| item.as_str().expect("a string"))
        .collect()
}

#[test]
fn an_external_branch_from_a_state_the_ordinary_branch_does_not_act_from_is_reached() {
    let setup = Setup {
        ir: ir_of(
            "explore-external-adversary.yaml",
            &read_fixture("explore-external-adversary.yaml"),
        ),
        ts_target: "explore-external-adversary-target.mjs",
        go_target: "explore_external_adversary_target.go",
        go_constructor: "newExploreAdvTarget()",
        mutate: &|_, contents| contents,
    };
    let cases = json!([{"name": "held-override"}]);
    let (typescript, go) = both("held", &setup, &cases);
    let reach = |lane: &Lane| lane.results["held-override"]["external"][0]["reach"].clone();
    assert_eq!(
        (reach(&typescript), reach(&go)),
        (json!("reached"), json!("reached")),
        "(typescript, go) reach of Close/overridden"
    );
    for (language, lane) in [("typescript", &typescript), ("go", &go)] {
        let found = &lane.results["held-override"];
        assert!(found.get("failure").is_none(), "{language}: {found}");
        assert_eq!(
            found["external"],
            json!([{
                "outcome": "exploreadv.desk.Close/overridden",
                "cause": "a supervisor overrides the hold",
                "reach": "reached"
            }]),
            "{language}: the external branch from `Held` was never taken\n{found}\n{}",
            lane.log
        );
        assert!(
            !strings(&found["unreached"]).contains(&"exploreadv.desk.Close/overridden"),
            "{language}: {found}"
        );
    }
}

/// Offers the ordinary branch even after an arrangement of the same command.
fn always_offer_ordinary(path: &str, contents: String) -> String {
    let (from, to) = if path.ends_with("explore.go") {
        (
            "decision.outcome != nil && (!s.forced[command.name] || len(decision.externals) == 0)",
            "decision.outcome != nil",
        )
    } else if path.ends_with("explore.ts") {
        (
            "decision.outcome !== undefined &&\n    (!s.forced.has(command.name) || decision.externals.length === 0)",
            "decision.outcome !== undefined",
        )
    } else {
        return contents;
    };
    assert!(contents.contains(from), "the mutation site moved in {path}");
    contents.replace(from, to)
}

#[test]
fn the_arrangement_rule_is_needed_the_mutant_that_always_offers_the_ordinary_branch_disagrees() {
    let setup = Setup {
        ir: ir_of(
            "explore-external.yaml",
            &read_fixture("explore-external.yaml"),
        ),
        ts_target: "explore-external-target.mjs",
        go_target: "explore_external_target.go",
        go_constructor: "newExploreExtTarget(one.Mode)",
        mutate: &always_offer_ordinary,
    };
    let cases = json!([{"name": "arranges-mutant"}]);
    let (typescript, go) = both("mutant", &setup, &cases);
    for (language, lane) in [("typescript", &typescript), ("go", &go)] {
        let found = &lane.results["arranges-mutant"];
        assert!(
            found.get("failure").is_some(),
            "{language}: the mutant survived the arranges double\n{found}"
        );
    }
}
