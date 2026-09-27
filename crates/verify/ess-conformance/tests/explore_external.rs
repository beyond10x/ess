//! The model-based explorer takes `external:` branches (beyond10x/ess#156), in both emitted
//! languages.
//!
//! The fixture is `tests/fixtures/explore-external.yaml`: one command with an input-eligible
//! external decline (`when:` beside `external:`), one with an unguarded external failure that moves
//! its subject. The targets are `explore-external-target.mjs` and `explore_external_target.go`,
//! switched by a mode: a double that arranges, one that cannot, and one that accepts the
//! arrangement and ignores it.
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

const FIXTURE: &str = include_str!("fixtures/explore-external.yaml");

fn ir() -> EssIr {
    let mut sources = SourceMap::new();
    sources.insert("explore-external.yaml", FIXTURE);
    let raw = RawSpecFile::parse(FIXTURE).expect("the fixture parses");
    let specification = Specification::assemble(vec![(Source::new("explore-external.yaml"), raw)])
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
        "ess-explore-external-{}-{label}",
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
    for name in ["explore-external-target.mjs", "explore-external-driver.mjs"] {
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
        .args(["--test", "explore-external-driver.mjs"])
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
        "module example.invalid/exploreext\n\ngo 1.24\n",
    );
    std::fs::copy(
        fixture("explore_external_target.go"),
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
    for case in cases.as_array().unwrap() {
        let line = format!(
            "--- PASS: TestExploreExternal/{}",
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
        {"name": "arranges"},
        {"name": "cannot-arrange", "mode": "cannot-arrange"},
        {"name": "cannot-arrange-allowed", "mode": "cannot-arrange", "allowExcluded": true},
        {"name": "ignores-arrangement", "mode": "ignores-arrangement"},
        {"name": "ignores-arrangement-replayed", "mode": "ignores-arrangement", "options": {"seed": 1}},
    ])
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

const DECLARED: &[&str] = &[
    "exploreext.pay.Authorize/authorized",
    "exploreext.pay.Authorize/declined",
    "exploreext.pay.Authorize/invalid",
    "exploreext.pay.Capture/captured",
    "exploreext.pay.Capture/failed",
    "exploreext.pay.Capture/wrong-state",
];

const ORDINARY: &[&str] = &[
    "exploreext.pay.Authorize/authorized",
    "exploreext.pay.Authorize/invalid",
    "exploreext.pay.Capture/captured",
    "exploreext.pay.Capture/wrong-state",
];

#[test]
fn a_double_that_arranges_external_branches_reaches_each_and_reports_it_reached() {
    for lane in [&lanes().0, &lanes().1] {
        let found = result(lane, "arranges");
        assert_eq!(lane.asserts["arranges"], "ok", "{found}");
        assert!(found.get("failure").is_none(), "{found}");
        assert_eq!(found["excluded"], json!([]), "{found}");
        assert!(strings(&found["excludedOutcomes"]).is_empty(), "{found}");
        assert_eq!(strings(&found["reached"]), DECLARED, "{found}");
        assert!(strings(&found["unreached"]).is_empty(), "{found}");
        assert_eq!(found["executed"], 12000, "{found}");
        assert_eq!(
            found["external"],
            json!([
                {
                    "outcome": "exploreext.pay.Authorize/declined",
                    "cause": "the card network declines",
                    "reach": "reached"
                },
                {
                    "outcome": "exploreext.pay.Capture/failed",
                    "cause": "the acquirer fails the capture",
                    "reach": "reached"
                }
            ]),
            "{found}"
        );
    }
}

#[test]
fn a_double_that_cannot_arrange_reports_the_branches_unarrangeable_and_no_disagreement() {
    for lane in [&lanes().0, &lanes().1] {
        let found = result(lane, "cannot-arrange");
        assert!(found.get("failure").is_none(), "{found}");
        assert_eq!(strings(&found["reached"]), ORDINARY, "{found}");
        assert!(strings(&found["unreached"]).is_empty(), "{found}");
        assert_eq!(found["excluded"], json!([]), "{found}");
        let reason = "the target cannot arrange it: this double arranges no external outcome";
        assert_eq!(
            found["external"],
            json!([
                {
                    "outcome": "exploreext.pay.Authorize/declined",
                    "cause": "the card network declines",
                    "reach": "unarrangeable",
                    "reason": reason
                },
                {
                    "outcome": "exploreext.pay.Capture/failed",
                    "cause": "the acquirer fails the capture",
                    "reach": "unarrangeable",
                    "reason": reason
                }
            ]),
            "{found}"
        );
        assert_eq!(
            lane.asserts["cannot-arrange"],
            "failed: explore: 2 external outcome(s) the target could not arrange were never tried:"
        );
        assert_eq!(lane.asserts["cannot-arrange-allowed"], "ok");
    }
}

#[test]
fn a_double_that_ignores_the_arrangement_is_a_disagreement_on_the_external_branch() {
    for lane in [&lanes().0, &lanes().1] {
        let found = result(lane, "ignores-arrangement");
        let failure = &found["failure"];
        let message = failure["message"].as_str().unwrap_or_default();
        assert!(
            message.starts_with(
                "outcome: the target answered `authorized`, the specification says `declined`"
            ) || message.starts_with(
                "outcome: the target answered `captured`, the specification says `failed`"
            ),
            "{found}"
        );
        let trace = strings(&failure["trace"]);
        let last = trace.last().expect("a shrunk trace");
        assert!(
            last.ends_with(" [external: declined]") || last.ends_with(" [external: failed]"),
            "{found}"
        );
        assert_eq!(failure["shrinkComplete"], true, "{found}");
        assert_eq!(failure["seed"], 1, "{found}");
        let replayed = result(lane, "ignores-arrangement-replayed");
        assert_eq!(replayed["sequences"], 1, "{replayed}");
        assert_eq!(
            replayed["failure"], *failure,
            "replaying seed 1 fails the same way"
        );
        assert!(
            lane.asserts["ignores-arrangement"].starts_with("failed: explore: seed "),
            "{}",
            lane.asserts["ignores-arrangement"]
        );
    }
}

#[test]
fn both_languages_report_the_same_result_for_every_external_case() {
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
