//! Adversary pass 1 on beyond10x/ess#537: the sequential explorers exclude a refused step's held
//! state, not the command.
//!
//! The fixture is `tests/fixtures/explore-refused.yaml`; the target is
//! `adversary-l4-refused-target.mjs` / `adversary_l4_refused_target.go`, whose modes refuse a command
//! in some held states and drive it in the others. Each case runs in both languages, through the
//! preconditions drivers, and the two lanes are held to each other.
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

const FIXTURE: &str = include_str!("fixtures/explore-refused.yaml");

fn ir() -> EssIr {
    let mut sources = SourceMap::new();
    sources.insert("explore-refused.yaml", FIXTURE);
    let raw = RawSpecFile::parse(FIXTURE).expect("the fixture parses");
    let specification = Specification::assemble(vec![(Source::new("explore-refused.yaml"), raw)])
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
        std::env::temp_dir().join(format!("ess-adv-l4-{}-{label}", std::process::id())),
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
        .expect("a list")
        .iter()
        .map(|item| item.as_str().expect("a string"))
        .collect()
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

const TS_TARGET: &str = "adversary-l4-refused-target.mjs";
const GO_TARGET: &str = "adversary_l4_refused_target.go";

fn typescript(root: &Path, ir: &EssIr, cases: &Value) -> Lane {
    let package = root.join("typescript");
    for artifact in ess_conformance::ts::emit_with_model(&suite(ir), ir).unwrap() {
        write(&package.join(&artifact.path), &artifact.contents);
    }
    let ts = package.join("essconform");
    std::fs::copy(fixture(TS_TARGET), ts.join(TS_TARGET)).expect("the target copies");
    let driver = read_fixture("explore-preconditions-driver.mjs").replace(
        "./explore-preconditions-target.mjs",
        &format!("./{TS_TARGET}"),
    );
    write(&ts.join("adv-l4-driver.mjs"), &driver);
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
        .args(["--test", "adv-l4-driver.mjs"])
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
        "module example.invalid/advl4\n\ngo 1.24\n",
    );
    std::fs::copy(
        fixture(GO_TARGET),
        module.join("essconform/adversary_l4_refused_target_test.go"),
    )
    .expect("the target copies");
    let driver = read_fixture("explore_preconditions_driver_test.go").replace(
        "newExplorePreTarget(one.Mode)",
        "newAdversaryL4RefusedTarget(one.Mode)",
    );
    write(&module.join("essconform/adv_l4_driver_test.go"), &driver);
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
    read_lane(&out, cases, log)
}

fn cases() -> Value {
    serde_json::json!([
        {"name": "correct"},
        {"name": "hold-waiting", "mode": "hold-waiting"},
        {"name": "hold-waiting-allowed", "mode": "hold-waiting", "allowExcluded": true},
        {"name": "ringing-once", "mode": "ringing-once", "allowExcluded": true},
        {"name": "ringing-connected", "mode": "ringing-connected"},
        {"name": "hold-everywhere-answer-ringing", "mode": "hold-everywhere-answer-ringing"},
    ])
}

/// `(typescript, go)`, run once.
fn lanes() -> &'static (Lane, Lane) {
    static LANES: OnceLock<(Lane, Lane)> = OnceLock::new();
    LANES.get_or_init(|| {
        let ir = ir();
        let cases = cases();
        let root = scratch("refused");
        let lanes = (typescript(&root, &ir, &cases), go(&root, &ir, &cases));
        std::fs::remove_dir_all(&root).ok();
        lanes
    })
}

fn each_lane() -> [(&'static str, &'static Lane); 2] {
    [("typescript", &lanes().0), ("go", &lanes().1)]
}

const ANSWER: &str = "explorerefused.desk.Answer";
const HOLD: &str = "explorerefused.desk.Hold";

/// The target serves every view and drives every command: nothing is excluded or refused, and the
/// strict assertion passes. Holds the target itself to the specification before any mode is read.
#[test]
fn the_adversary_target_is_correct_when_it_refuses_nothing() {
    for (language, lane) in each_lane() {
        let found = &lane.results["correct"];
        assert_eq!(
            lane.asserts["correct"], "ok",
            "{language}: {found}\n{}",
            lane.log
        );
        assert!(found.get("refused").is_none(), "{language}: {found}");
        assert_eq!(
            found["excluded"],
            serde_json::json!([]),
            "{language}: {found}"
        );
    }
}

/// Before #537 any `ErrUnsupported` from a command excluded it, and the strict assertion
/// (`AllowExcluded` unset) failed: accepting a refusal was an explicit `allowExcluded`. A target
/// that refuses `Hold` while the call waits, and whose refused step's outcome (`not-ringing`) a
/// connected call reaches, is reported in `refused` and still fails the strict assertion: the
/// refusal is accepted only when the caller opts in.
#[test]
fn a_refusal_in_one_state_still_fails_the_strict_assertion() {
    for (language, lane) in each_lane() {
        let found = &lane.results["hold-waiting"];
        assert!(found.get("failure").is_none(), "{language}: {found}");
        let refused = found["refused"]
            .as_array()
            .unwrap_or_else(|| panic!("{language}: no refusal reported: {found}"));
        assert_eq!(refused.len(), 1, "{language}: {found}");
        assert_eq!(refused[0]["command"], HOLD, "{language}: {found}");
        assert_eq!(refused[0]["state"], "Waiting", "{language}: {found}");
        assert_eq!(
            refused[0]["reason"],
            "the target does not expose it: Hold has no callback while the call waits",
            "{language}: {found}"
        );
        assert_eq!(
            lane.asserts["hold-waiting-allowed"], "ok",
            "{language}: allowExcluded accepts the refusal"
        );
        assert!(
            lane.asserts["hold-waiting"].starts_with("failed: "),
            "{language}: the strict assertion accepted a target that refused `Hold` in `Waiting` \
             without allowExcluded (assert: `{}`): {found}",
            lane.asserts["hold-waiting"]
        );
    }
}

/// The refused (command, held state) is never sent again in the sequence that found it: the target,
/// which serves one sequence, answers a second `Answer` in `Ringing` with a plain error, which would
/// be a failure. Each seed keeps its own refusals (beyond10x/ess#576), so another seed may send it
/// again to its own target.
#[test]
fn a_refused_state_is_never_sent_again_in_its_sequence() {
    for (language, lane) in each_lane() {
        let found = &lane.results["ringing-once"];
        assert!(
            found.get("failure").is_none(),
            "{language}: the command was resent in its refused state: {found}"
        );
        let refused = found["refused"]
            .as_array()
            .unwrap_or_else(|| panic!("{language}: {found}"));
        assert_eq!(refused.len(), 1, "{language}: {found}");
        assert_eq!(refused[0]["state"], "Ringing", "{language}: {found}");
        assert_eq!(lane.asserts["ringing-once"], "ok", "{language}: {found}");
    }
}

/// Two refused states of one command driven in a third: both are listed, in byte order of the
/// state, and only the outcomes those states alone could take are excluded.
#[test]
fn two_refused_states_are_listed_in_state_order_and_only_their_outcomes_excluded() {
    for (language, lane) in each_lane() {
        let found = &lane.results["ringing-connected"];
        assert!(found.get("failure").is_none(), "{language}: {found}");
        let refused = found["refused"]
            .as_array()
            .unwrap_or_else(|| panic!("{language}: {found}"));
        let keys: Vec<(&str, &str)> = refused
            .iter()
            .map(|item| {
                (
                    item["command"].as_str().unwrap(),
                    item["state"].as_str().unwrap(),
                )
            })
            .collect();
        assert_eq!(
            keys,
            [(ANSWER, "Connected"), (ANSWER, "Ringing")],
            "{language}: {found}"
        );
        for item in refused {
            assert!(
                item["count"].as_u64().is_some_and(|count| count >= 1),
                "{language}: {found}"
            );
        }
        assert_eq!(
            strings(&found["excludedOutcomes"]),
            [
                "explorerefused.desk.Answer/answered",
                "explorerefused.desk.Answer/not-open"
            ],
            "{language}: {found}"
        );
        assert!(
            strings(&found["reached"]).contains(&"explorerefused.desk.Answer/picked-up"),
            "{language}: {found}"
        );
        assert_eq!(
            found["excluded"],
            serde_json::json!([]),
            "{language}: {found}"
        );
        assert_eq!(
            lane.asserts["ringing-connected"],
            "failed: explore: 2 declared outcome(s) of excluded commands were never tried:",
            "{language}"
        );
    }
}

/// `Answer` refused in every state the run reached (only `Ringing`, because `Hold` is refused
/// everywhere and nothing else leaves it) is driven in none, so it is excluded with its refusal's
/// reason, beside `Hold`, and nothing is reported in `refused`.
#[test]
fn a_command_refused_in_every_reached_state_is_excluded_not_refused() {
    for (language, lane) in each_lane() {
        let found = &lane.results["hold-everywhere-answer-ringing"];
        assert!(found.get("failure").is_none(), "{language}: {found}");
        assert!(found.get("refused").is_none(), "{language}: {found}");
        assert_eq!(
            found["excluded"],
            serde_json::json!([
                {"subject": ANSWER, "reason": "the target does not expose it: Answer has no callback while the call rings"},
                {"subject": HOLD, "reason": "the target does not expose it: Hold has no callback"},
            ]),
            "{language}: {found}"
        );
        assert_eq!(
            strings(&found["excludedOutcomes"]),
            [
                "explorerefused.desk.Answer/answered",
                "explorerefused.desk.Answer/not-open",
                "explorerefused.desk.Answer/picked-up",
                "explorerefused.desk.Hold/held",
                "explorerefused.desk.Hold/not-ringing",
            ],
            "{language}: {found}"
        );
        assert!(
            strings(&found["unreached"]).is_empty(),
            "{language}: {found}"
        );
    }
}

/// Every case reports the same result and assertion in both languages.
#[test]
fn both_languages_report_the_same_result_for_every_refusal_case() {
    let (typescript, go) = lanes();
    assert_eq!(typescript.results.len(), cases().as_array().unwrap().len());
    for (name, result) in &typescript.results {
        assert_eq!(
            result, &go.results[name],
            "`{name}` differs between TypeScript and Go"
        );
        assert_eq!(typescript.asserts[name], go.asserts[name], "`{name}`");
    }
}
