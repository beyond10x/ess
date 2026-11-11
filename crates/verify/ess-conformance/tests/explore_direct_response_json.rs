//! Adversary cases for story:a-required-response-member-answered-as-null-fails-exploration.
//!
//! `FIXTURE` below adds a REQUIRED `Json` response member,
//! `body`. JSON null is one of that type's own values (docs/design/wire-presence-json-prefix.md), and
//! both suite runtimes admit a present null there, so the explorers must not count it as absent.
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

const FIXTURE: &str = r"# An adversary fixture: a REQUIRED `Json` response member, whose own values include null.
# (story:go-explorer-compares-direct-response-members).
#
# `Mint` returns a token (`minted`, `returns: true`): its response declares `token` and `label`,
# both REQUIRED, and `note`, an `Optional` member. The model determines none of their values, so
# the explorer checks each REQUIRED member for presence only. `too-long` refuses a label over eight
# characters and returns nothing. The targets beside it are `explore-direct-response-target.mjs`
# and `explore_direct_response_target.go`, each switched by the mode the driver passes.

format: ess/29
system: explorert
version: v1
domain: explorert.keys
summary: Tokens minted and returned to the caller.

actors:
  - name: explorert.keys.Client
    may: [explorert.keys.Mint]

errors:
  - {name: explorert.keys.TooLong, summary: The label is over eight characters.}

commands:
  - name: explorert.keys.Mint
    input:
      - {name: label, type: String}
    response:
      - {name: token, type: String}
      - {name: label, type: String}
      - {name: note, type: Optional<String>}
      - {name: body, type: Json}
    outcomes:
      - name: too-long
        when: label.count > 8
        error: explorert.keys.TooLong
      - name: minted
        returns: true
";

fn ir() -> EssIr {
    let mut sources = SourceMap::new();
    sources.insert("explore-direct-response-json.yaml", FIXTURE);
    let raw = RawSpecFile::parse(FIXTURE).expect("the fixture parses");
    let specification = Specification::assemble(vec![(
        Source::new("explore-direct-response-json.yaml"),
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
            "explore-direct-response-json-target.mjs",
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
        "module example.invalid/exploredirectresponsejson\n\ngo 1.24\n",
    );
    std::fs::copy(
        fixture("explore_direct_response_json_target.go"),
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
        {"name": "null-body", "mode": "null-body"},
        {"name": "drops-body", "mode": "drops-body"},
        {"name": "undefined-token", "mode": "undefined-token"},
    ])
}

/// Both lanes, run once for every test below.
fn lanes() -> &'static (Lane, Lane) {
    static LANES: OnceLock<(Lane, Lane)> = OnceLock::new();
    LANES.get_or_init(|| {
        let ir = ir();
        let cases = cases();
        let root = support_scratch::Scratch::new(format!(
            "ess-explore-direct-response-json-{}",
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

/// A REQUIRED `Json` member answered as JSON null is present: null is a `Json` value, and both
/// suite runtimes admit it (Go `directJSON`, TypeScript `jsonValue`). Exploration must pass.
#[test]
fn a_required_json_member_answered_as_null_passes_exploration() {
    let mut wrong = Vec::new();
    for (language, lane) in [("typescript", &lanes().0), ("go", &lanes().1)] {
        let found = result(lane, "complete");
        assert_eq!(
            lane.asserts["complete"], "ok",
            "{language} `complete`: {found}"
        );
        if lane.asserts["null-body"] != "ok" {
            wrong.push(format!(
                "{language} `null-body`: {}",
                lane.asserts["null-body"]
            ));
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

/// The `Json` member is still checked for presence: left out, it fails naming it.
#[test]
fn a_required_json_member_left_out_fails_naming_it() {
    for (language, lane) in [("typescript", &lanes().0), ("go", &lanes().1)] {
        let found = result(lane, "drops-body");
        let message = found["failure"]["message"].as_str().unwrap_or_default();
        assert!(
            message.starts_with(
                "response: the target's response to `minted` has no `body`, a member the \
                 specification requires"
            ),
            "{language}: {found}"
        );
    }
}

/// A TypeScript target answering a REQUIRED member as `undefined` (an own key with no value, the
/// result of `{token: record.token}` on a missing field) fails like a Go target leaving it out.
/// Pins the `=== undefined` clause of the TypeScript check, which no case of
/// `explore_direct_response.rs` reaches.
#[test]
fn a_required_member_answered_as_undefined_fails_naming_it() {
    for (language, lane) in [("typescript", &lanes().0), ("go", &lanes().1)] {
        let found = result(lane, "undefined-token");
        let message = found["failure"]["message"].as_str().unwrap_or_default();
        assert!(
            message.starts_with(
                "response: the target's response to `minted` has no `token`, a member the \
                 specification requires"
            ),
            "{language}: {found}"
        );
    }
}
