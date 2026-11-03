//! The generated explorer compares a command's direct response members
//! (`story:go-explorer-compares-direct-response-members`), in both emitted languages.
//!
//! The fixture is `tests/fixtures/explore-direct-response.yaml`: `Mint` answers `minted`
//! (`returns: true`) with a response declaring the REQUIRED `token` and `label` and the `Optional`
//! `note`. The model determines no response value, so each REQUIRED member is checked for presence
//! only: a target that leaves `token` out fails exploration naming it; one that returns values of
//! its own, or leaves the `Optional` member out, passes. The targets are
//! `explore-direct-response-target.mjs` and `explore_direct_response_target.go`, run by the
//! external-branch drivers.
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

const FIXTURE: &str = include_str!("fixtures/explore-direct-response.yaml");

fn ir() -> EssIr {
    let mut sources = SourceMap::new();
    sources.insert("explore-direct-response.yaml", FIXTURE);
    let raw = RawSpecFile::parse(FIXTURE).expect("the fixture parses");
    let specification =
        Specification::assemble(vec![(Source::new("explore-direct-response.yaml"), raw)])
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
    for (from, to) in [
        (
            "explore-direct-response-target.mjs",
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

fn go(root: &Path, ir: &EssIr, cases: &Value) -> Lane {
    let module = root.join("go");
    for artifact in ess_conformance::go::emit_with_model(&suite(ir), ir).unwrap() {
        write(&module.join(&artifact.path), &artifact.contents);
    }
    write(
        &module.join("go.mod"),
        "module example.invalid/exploredirectresponse\n\ngo 1.24\n",
    );
    std::fs::copy(
        fixture("explore_direct_response_target.go"),
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

fn cases() -> Value {
    json!([
        {"name": "complete"},
        {"name": "drops-token", "mode": "drops-token"},
        {"name": "other-label", "mode": "other-label"},
        {"name": "with-note", "mode": "with-note"},
    ])
}

/// Both lanes, run once for every test below.
fn lanes() -> &'static (Lane, Lane) {
    static LANES: OnceLock<(Lane, Lane)> = OnceLock::new();
    LANES.get_or_init(|| {
        let ir = ir();
        let cases = cases();
        let root = support_scratch::Scratch::new(format!(
            "ess-explore-direct-response-{}",
            std::process::id()
        ));
        (typescript(&root, &ir, &cases), go(&root, &ir, &cases))
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

/// A target that leaves out the REQUIRED `token` fails exploration, naming the member.
#[test]
fn a_target_dropping_a_required_response_member_fails_naming_it() {
    for (language, lane) in [("typescript", &lanes().0), ("go", &lanes().1)] {
        let found = result(lane, "drops-token");
        let message = found["failure"]["message"].as_str().unwrap_or_default();
        assert!(
            message.starts_with(
                "response: the target's response to `minted` has no `token`, a member the \
                 specification requires"
            ),
            "{language}: {found}"
        );
        assert!(
            lane.asserts["drops-token"].starts_with("failed: explore: seed "),
            "{language}: {}",
            lane.asserts["drops-token"]
        );
    }
}

/// Every member is present: the target passes, whatever values it returns, since the model
/// determines none of them; an `Optional` member may be left out or returned.
#[test]
fn members_the_model_does_not_determine_are_checked_for_presence_only() {
    for (language, lane) in [("typescript", &lanes().0), ("go", &lanes().1)] {
        for name in ["complete", "other-label", "with-note"] {
            let found = result(lane, name);
            assert_eq!(lane.asserts[name], "ok", "{language} `{name}`: {found}");
            let reached: Vec<&str> = found["reached"]
                .as_array()
                .unwrap_or_else(|| panic!("a reached list, not {found}"))
                .iter()
                .map(|item| item.as_str().expect("a string"))
                .collect();
            assert!(
                reached.contains(&"explorert.keys.Mint/minted"),
                "{language} `{name}`: {found}"
            );
        }
    }
}

#[test]
fn both_languages_report_the_same_result_for_every_case() {
    let (typescript, go) = lanes();
    for (name, result) in &typescript.results {
        assert_eq!(
            result, &go.results[name],
            "`{name}` differs between TypeScript and Go\nTypeScript: {:?}\nGo: {:?}",
            result, go.results[name]
        );
        assert_eq!(typescript.asserts[name], go.asserts[name], "`{name}`");
    }
}
