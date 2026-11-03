//! A component's package explores only that component's commands
//! (`story:component-explorer-walks-only-its-component`), in both emitted languages.
//!
//! The fixture is `tests/fixtures/explore-component.yaml`: `front` accepts `OpenTicket` and
//! `CloseTicket`, `back` accepts `WriteNote`. The targets, `explore-component-target.mjs` and
//! `explore_component_target.go`, implement `front` alone. The package is emitted as
//! `ess generate synthesize --component front` emits it: the suite `synthesize_for` scopes to the
//! component, beside the whole compiled model as `ir.json`, so the model's digest still matches the
//! suite's. Its explorer walks `front`'s commands alone and passes with `AllowExcluded` unset; the
//! whole system's package, explored against the same target, still reports `WriteNote` excluded,
//! which `AllowExcluded` alone accepts.
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
use serde_json::{json, Value};

const FIXTURE: &str = include_str!("fixtures/explore-component.yaml");
const WRITE_NOTE: &str = "explorecmp.desk.WriteNote";

fn ir() -> EssIr {
    let mut sources = SourceMap::new();
    sources.insert("explore-component.yaml", FIXTURE);
    let raw = RawSpecFile::parse(FIXTURE).expect("the fixture parses");
    let specification = Specification::assemble(vec![(Source::new("explore-component.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("the fixture validates:\n{errors}"));
    compile(&specification, &sources).unwrap_or_else(|diagnostics| panic!("{diagnostics}"))
}

/// The suite `synthesize --component front` writes, or the whole system's where `component` is
/// `None`.
fn suite(ir: &EssIr, component: Option<&str>) -> ConformanceSuite {
    let mut suite = match component {
        None => ess_conformance::synthesize(ir).suite,
        Some(name) => {
            ess_conformance::synthesize::synthesize_for(ir, name)
                .unwrap_or_else(|unknown| panic!("{unknown}"))
                .suite
        }
    };
    suite.select_fresh_format_for(ir);
    suite
}

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
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

fn typescript(root: &Path, ir: &EssIr, suite: &ConformanceSuite, cases: &Value) -> Lane {
    let package = root.join("typescript");
    for artifact in ess_conformance::ts::emit_with_model(suite, ir).unwrap() {
        write(&package.join(&artifact.path), &artifact.contents);
    }
    let ts = package.join("essconform");
    for (from, to) in [
        (
            "explore-component-target.mjs",
            "explore-external-target.mjs",
        ),
        ("explore-external-driver.mjs", "explore-external-driver.mjs"),
    ] {
        std::fs::copy(fixture(from), ts.join(to)).expect("the fixture copies");
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
        .args(["--test", "explore-external-driver.mjs"])
        .env("ESS_EXPLORE_CASES", cases.to_string())
        .env("ESS_EXPLORE_OUT", &out)
        .current_dir(&ts)
        .output()
        .expect("`node` is on PATH: the explorer lane does not skip");
    let log = printed(&run);
    assert!(run.status.success(), "{log}");
    read_lane(&out, cases, log)
}

fn go(root: &Path, ir: &EssIr, suite: &ConformanceSuite, cases: &Value) -> Lane {
    let module = root.join("go");
    for artifact in ess_conformance::go::emit_with_model(suite, ir).unwrap() {
        write(&module.join(&artifact.path), &artifact.contents);
    }
    write(
        &module.join("go.mod"),
        "module example.invalid/explorecomponent\n\ngo 1.24\n",
    );
    std::fs::copy(
        fixture("explore_component_target.go"),
        module.join("essconform/explore_external_target_test.go"),
    )
    .unwrap();
    std::fs::copy(
        fixture("explore_external_driver_test.go"),
        module.join("essconform/explore_external_driver_test.go"),
    )
    .unwrap();
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
        .expect("`go` is on PATH: the explorer lane does not skip");
    let log = printed(&run);
    assert!(run.status.success(), "{log}");
    read_lane(&out, cases, log)
}

/// The component's package and the whole system's, each in both languages: `(typescript, go)`.
struct Lanes {
    component: (Lane, Lane),
    whole: (Lane, Lane),
}

fn lanes() -> &'static Lanes {
    static LANES: OnceLock<Lanes> = OnceLock::new();
    LANES.get_or_init(|| {
        let ir = ir();
        let root =
            support_scratch::Scratch::new(format!("ess-explore-component-{}", std::process::id()));
        let run = |label: &str, component: Option<&str>, cases: &Value| {
            let suite = suite(&ir, component);
            let at = root.join(label);
            (
                typescript(&at, &ir, &suite, cases),
                go(&at, &ir, &suite, cases),
            )
        };
        Lanes {
            component: run("component", Some("front"), &json!([{"name": "front"}])),
            whole: run(
                "whole",
                None,
                &json!([
                    {"name": "whole"},
                    {"name": "whole-allowed", "allowExcluded": true},
                ]),
            ),
        }
    })
}

fn result<'a>(lane: &'a Lane, name: &str) -> &'a Value {
    lane.results[name].as_ref().unwrap_or_else(|| {
        panic!(
            "`{name}` returned no result: {}\n{}",
            lane.asserts[name], lane.log
        )
    })
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
fn the_component_suite_records_its_component_and_the_whole_model_digest() {
    let ir = ir();
    let scoped = suite(&ir, Some("front"));
    let whole = suite(&ir, None);
    assert_eq!(scoped.provenance.component.as_deref(), Some("front"));
    assert_eq!(scoped.provenance.spec_digest, whole.provenance.spec_digest);
}

/// The component's explorer holds only `front`'s commands and passes with `AllowExcluded` unset.
#[test]
fn a_component_package_explores_only_its_components_commands() {
    let (typescript, go) = &lanes().component;
    for (language, lane) in [("typescript", typescript), ("go", go)] {
        let found = result(lane, "front");
        assert_eq!(lane.asserts["front"], "ok", "{language}: {found}");
        assert_eq!(found["excluded"], json!([]), "{language}: {found}");
        assert_eq!(
            strings(&found["reached"]),
            [
                "explorecmp.desk.CloseTicket/closed",
                "explorecmp.desk.CloseTicket/not-open",
                "explorecmp.desk.OpenTicket/opened",
            ],
            "{language}: {found}"
        );
        assert!(
            !found.to_string().contains(WRITE_NOTE),
            "{language}: `back`'s command is not explored: {found}"
        );
    }
}

/// The whole system's package explored against the same target still reports `back`'s command,
/// which only `AllowExcluded` accepts: the option keeps meaning that a real exclusion was accepted.
#[test]
fn the_whole_system_package_still_reports_the_other_components_command() {
    let (typescript, go) = &lanes().whole;
    for (language, lane) in [("typescript", typescript), ("go", go)] {
        let found = result(lane, "whole");
        assert!(
            lane.asserts["whole"].starts_with("failed: "),
            "{language}: {}\n{found}",
            lane.asserts["whole"]
        );
        let excluded: Vec<&str> = found["excluded"]
            .as_array()
            .unwrap_or_else(|| panic!("an exclusion list, not {found}"))
            .iter()
            .map(|exclusion| exclusion["subject"].as_str().expect("a subject"))
            .collect();
        assert!(excluded.contains(&WRITE_NOTE), "{language}: {found}");
        assert_eq!(
            lane.asserts["whole-allowed"], "ok",
            "{language}: {}",
            lane.log
        );
    }
}

#[test]
fn both_languages_report_the_same_result_for_every_case() {
    for (typescript, go) in [&lanes().component, &lanes().whole] {
        for (name, result) in &typescript.results {
            assert_eq!(
                result, &go.results[name],
                "`{name}` differs between TypeScript and Go\nTypeScript: {:?}\nGo: {:?}",
                result, go.results[name]
            );
            assert_eq!(typescript.asserts[name], go.asserts[name], "`{name}`");
        }
    }
}
