//! The model-based explorer, executed in both emitted languages.
//!
//! `docs/design/mutation-audit-and-model-runner.md`, Part 2, and its deciding checks P2-1 to P2-8.
//! The fixture is `tests/fixtures/explore.yaml`; the targets are `explore-target.mjs` and
//! `explore_target.go`, each switched by the mutant name; the drivers run one exploration per case
//! and write its result, so this file can hold the two lanes to each other as well as to the pins.
//!
//! A missing `tsc`, `node` or `go` panics rather than skipping: a lane that ran nothing and exited
//! zero is the failure this repository already knows by name.

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

const FIXTURE: &str = include_str!("fixtures/explore.yaml");

fn ir() -> EssIr {
    let mut sources = SourceMap::new();
    sources.insert("explore.yaml", FIXTURE);
    let raw = RawSpecFile::parse(FIXTURE).expect("the fixture parses");
    let specification = Specification::assemble(vec![(Source::new("explore.yaml"), raw)])
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

fn scratch(label: &str) -> support_scratch::Scratch {
    let root = support_scratch::Scratch::adopt(
        std::env::temp_dir().join(format!("ess-explore-{}-{label}", std::process::id())),
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

/// What one lane reported: each case's result and assertion, the five draws, and the runner log.
#[derive(Debug)]
struct Lane {
    results: BTreeMap<String, Option<Value>>,
    asserts: BTreeMap<String, String>,
    mulberry32: String,
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
    let mulberry32 = std::fs::read_to_string(out.join("mulberry32"))
        .unwrap_or_else(|_| panic!("the lane wrote no draws:\n{log}"))
        .trim_end()
        .to_owned();
    Lane {
        results,
        asserts,
        mulberry32,
        log,
    }
}

/// The TypeScript lane: the emitted package, compiled with `tsc --noCheck`, run with `node --test`.
fn typescript(
    root: &Path,
    ir: &EssIr,
    cases: &Value,
    edit: impl Fn(&str, String) -> String,
) -> Lane {
    let package = root.join("typescript");
    for artifact in ess_conformance::ts::emit_with_model(&suite(ir), ir).unwrap() {
        let contents = edit(&artifact.path, artifact.contents);
        write(&package.join(&artifact.path), &contents);
    }
    let ts = package.join("essconform");
    for name in ["explore-target.mjs", "explore-driver.mjs"] {
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
        .args(["--test", "explore-driver.mjs"])
        .env("ESS_EXPLORE_CASES", cases.to_string())
        .env("ESS_EXPLORE_OUT", &out)
        .env_remove("ESS_EXPLORE_MUTANT")
        .env_remove("ESS_EXPLORE_GRADE")
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

/// The Go lane: the emitted package as a module of its own, run with `go test`.
fn go(root: &Path, ir: &EssIr, cases: &Value, edit: impl Fn(&str, String) -> String) -> Lane {
    let module = root.join("go");
    for artifact in ess_conformance::go::emit_with_model(&suite(ir), ir).unwrap() {
        let contents = edit(&artifact.path, artifact.contents);
        write(&module.join(&artifact.path), &contents);
    }
    write(
        &module.join("go.mod"),
        "module example.invalid/explore\n\ngo 1.24\n",
    );
    std::fs::copy(
        fixture("explore_target.go"),
        module.join("essconform/explore_target_test.go"),
    )
    .unwrap();
    std::fs::copy(
        fixture("explore_driver_test.go"),
        module.join("essconform/explore_driver_test.go"),
    )
    .unwrap();
    let out = root.join("out-go");
    std::fs::create_dir_all(&out).unwrap();
    let run = Command::new("go")
        .args([
            "test",
            "./essconform",
            "-run",
            "TestExplore",
            "-count=1",
            "-v",
        ])
        .env("ESS_EXPLORE_CASES", cases.to_string())
        .env("ESS_EXPLORE_OUT", &out)
        .env_remove("ESS_EXPLORE_MUTANT")
        .env_remove("ESS_EXPLORE_GRADE")
        .env("GOWORK", "off")
        .current_dir(&module)
        .output()
        .expect("`go` is on PATH: the explorer lane does not skip");
    let log = printed(&run);
    assert!(run.status.success(), "{log}");
    for case in cases.as_array().unwrap() {
        let line = format!("--- PASS: TestExplore/{}", case["name"].as_str().unwrap());
        assert!(
            log.lines().any(|it| it.trim_start().starts_with(&line)),
            "`{line}` is not in the go log:\n{log}"
        );
    }
    read_lane(&out, cases, log)
}

/// The seed each mutant first fails under, from the first exploration of 200 × 60.
const FAILING_SEEDS: &[(&str, u64)] = &[
    ("view-cap-5", 1),
    ("wrong-state-applies-sets", 1),
    ("fourth-create-reuses-first", 1),
    ("third-return-refused", 4),
];

fn cases() -> Value {
    let mut cases = vec![
        serde_json::json!({"name": "correct"}),
        serde_json::json!({"name": "grade-high", "grade": "high"}),
        serde_json::json!({"name": "close-unsupported", "mutant": "close-unsupported"}),
        serde_json::json!({"name": "close-unsupported-allowed", "mutant": "close-unsupported", "allowExcluded": true}),
        serde_json::json!({"name": "one-step", "options": {"seeds": 1, "steps": 1}}),
    ];
    for (mutant, seed) in FAILING_SEEDS {
        cases.push(serde_json::json!({"name": mutant, "mutant": mutant}));
        cases.push(serde_json::json!({"name": format!("{mutant}-replayed"), "mutant": mutant, "options": {"seed": seed}}));
    }
    Value::Array(cases)
}

/// Both lanes, run once for every test below.
fn lanes() -> &'static (Lane, Lane) {
    static LANES: OnceLock<(Lane, Lane)> = OnceLock::new();
    LANES.get_or_init(|| {
        let ir = ir();
        let cases = cases();
        let root = scratch("lanes");
        let lanes = (
            typescript(&root, &ir, &cases, |_, contents| contents),
            go(&root, &ir, &cases, |_, contents| contents),
        );
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
        .expect("a list")
        .iter()
        .map(|item| item.as_str().expect("a string"))
        .collect()
}

/// Every declared outcome of the fixture.
const DECLARED: &[&str] = &[
    "explore.desk.CloseTicket/closed",
    "explore.desk.CloseTicket/wrong-state",
    "explore.desk.Grade/high",
    "explore.desk.Grade/low",
    "explore.desk.Grade/ungraded",
    "explore.desk.HoldTicket/held",
    "explore.desk.HoldTicket/wrong-state",
    "explore.desk.OpenTicket/opened",
    "explore.desk.OpenTicket/rejected",
    "explore.desk.ReleaseTicket/released",
    "explore.desk.ReleaseTicket/wrong-state",
    "explore.desk.Restock/refused",
    "explore.desk.Restock/restocked",
];

// ---- P2-1: the correct target passes and reaches every declared outcome ---------------------------

#[test]
fn the_correct_target_passes_and_reaches_every_declared_outcome() {
    for lane in [&lanes().0, &lanes().1] {
        let correct = result(lane, "correct");
        assert_eq!(lane.asserts["correct"], "ok", "{correct}");
        assert!(correct.get("failure").is_none(), "{correct}");
        assert_eq!(strings(&correct["reached"]), DECLARED);
        assert_eq!(strings(&correct["unreached"]).len(), 0);
        assert_eq!(correct["sequences"], 200);
        assert_eq!(correct["steps"], 60);
        assert_eq!(correct["executed"], 12000);
        assert!(strings(&correct["undetermined"]).is_empty(), "{correct}");
        // A specification without an `external:` branch keeps the result's bytes: no `external`.
        assert!(correct.get("external").is_none(), "{correct}");
    }
}

// ---- P2-2: each engine mutant is caught, with a shrunk trace that fails again ---------------------

/// The four mutants: the message class each fails with, and the shrunk trace it is pinned to.
const MUTANTS: &[(&str, &str, &[&str])] = &[
    (
        "view-cap-5",
        "view-rows: `explore.desk.TicketsByItems` holds 5 row(s), the specification says 6",
        &[
            r#"explore.desk.OpenTicket {"items":2}"#,
            r#"explore.desk.OpenTicket {"items":2941}"#,
            r#"explore.desk.OpenTicket {"items":0}"#,
            r#"explore.desk.OpenTicket {"items":2}"#,
            r#"explore.desk.OpenTicket {"items":3}"#,
            r#"explore.desk.OpenTicket {"items":0}"#,
        ],
    ),
    (
        "wrong-state-applies-sets",
        "view-field: `explore.desk.TicketsByItems`.priority of \"00000000-0000-4000-8000-000000000001\" is 100, the specification says 1",
        &[
            r#"explore.desk.OpenTicket {"items":2}"#,
            r#"explore.desk.HoldTicket {"priority":1,"ticket_id":"00000000-0000-4000-8000-000000000001"}"#,
            r#"explore.desk.OpenTicket {"items":2941}"#,
            r#"explore.desk.OpenTicket {"items":0}"#,
            r#"explore.desk.HoldTicket {"priority":100,"ticket_id":"00000000-0000-4000-8000-000000000001"}"#,
            r#"explore.desk.Restock {"items":8343,"ticket_id":"00000000-0000-4000-8000-000000000003"}"#,
        ],
    ),
    (
        "fourth-create-reuses-first",
        "identity: the target created \"00000000-0000-4000-8000-000000000001\" again, over an existing record",
        &[
            r#"explore.desk.OpenTicket {"items":2}"#,
            r#"explore.desk.OpenTicket {"items":2941}"#,
            r#"explore.desk.OpenTicket {"items":0}"#,
            r#"explore.desk.OpenTicket {"items":2}"#,
        ],
    ),
    (
        "third-return-refused",
        "outcome: the target answered `wrong-state`, the specification says `released`",
        &[
            r#"explore.desk.OpenTicket {"items":3}"#,
            r#"explore.desk.HoldTicket {"priority":5283,"ticket_id":"00000000-0000-4000-8000-000000000001"}"#,
            r#"explore.desk.ReleaseTicket {"ticket_id":"00000000-0000-4000-8000-000000000001"}"#,
            r#"explore.desk.HoldTicket {"priority":9796,"ticket_id":"00000000-0000-4000-8000-000000000001"}"#,
            r#"explore.desk.ReleaseTicket {"ticket_id":"00000000-0000-4000-8000-000000000001"}"#,
        ],
    ),
];

#[test]
fn each_engine_mutant_is_caught_with_a_shrunk_trace_that_fails_again_under_its_seed() {
    for lane in [&lanes().0, &lanes().1] {
        for (mutant, class, trace) in MUTANTS {
            let found = result(lane, mutant);
            let failure = &found["failure"];
            let message = failure["message"].as_str().unwrap_or_default();
            assert!(message.starts_with(class), "{mutant}: {message}");
            assert!(
                lane.asserts[*mutant].starts_with("failed: explore: seed "),
                "{mutant}"
            );
            assert_eq!(strings(&failure["trace"]), *trace, "{mutant}: {found}");
            let original = failure["originalLength"].as_u64().unwrap();
            assert!(trace.len() as u64 <= original, "{mutant}: {found}");
            assert_eq!(failure["shrinkComplete"], true, "{mutant}");

            let seed = FAILING_SEEDS
                .iter()
                .find(|(name, _)| name == mutant)
                .unwrap()
                .1;
            assert_eq!(failure["seed"], seed, "{mutant}");
            let replayed = result(lane, &format!("{mutant}-replayed"));
            assert_eq!(replayed["sequences"], 1, "{mutant}");
            assert_eq!(
                replayed["failure"], *failure,
                "{mutant}: replaying seed {seed} fails the same way"
            );
        }
    }
}

// ---- P2-3: one seed names one sequence in both languages ------------------------------------------

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

// ---- P2-4: excluded outcomes fail by default; one step leaves outcomes unreached -----------------

#[test]
fn an_unsupported_command_fails_unless_its_outcomes_are_explicitly_accepted() {
    for lane in [&lanes().0, &lanes().1] {
        let unsupported = result(lane, "close-unsupported");
        assert_eq!(
            strings(&unsupported["excludedOutcomes"]),
            [
                "explore.desk.CloseTicket/closed",
                "explore.desk.CloseTicket/wrong-state"
            ]
        );
        assert_eq!(
            unsupported["excluded"],
            serde_json::json!([{
                "reason": "the target does not expose it: CloseTicket is not exposed",
                "subject": "explore.desk.CloseTicket"
            }])
        );
        assert!(unsupported.get("failure").is_none(), "{unsupported}");
        assert_eq!(
            lane.asserts["close-unsupported"],
            "failed: explore: 2 declared outcome(s) of excluded commands were never tried:"
        );
        assert_eq!(lane.asserts["close-unsupported-allowed"], "ok");

        let short = result(lane, "one-step");
        assert_eq!(short["executed"], 1);
        assert_ne!(strings(&short["unreached"]).len(), 0);
        assert!(
            lane.asserts["one-step"].starts_with("failed: explore: ")
                && lane.asserts["one-step"].ends_with("declared outcome(s) no sequence reached:"),
            "{}",
            lane.asserts["one-step"]
        );
    }
}

// ---- P2-5: ir.json is bound to suite.json ---------------------------------------------------------

#[test]
fn one_changed_byte_in_ir_json_is_refused_in_both_languages() {
    let ir = ir();
    let cases = serde_json::json!([{"name": "correct"}]);
    let edit = |path: &str, contents: String| {
        if path.ends_with("/ir.json") {
            contents.replacen("explore.desk.Grade", "explore.desk.Grada", 1)
        } else {
            contents
        }
    };
    let root = scratch("digest");
    let refusal = "refused: `ir.json` and `suite.json` come from different specifications; regenerate the package";
    assert_eq!(
        typescript(&root, &ir, &cases, edit).asserts["correct"],
        refusal
    );
    assert_eq!(go(&root, &ir, &cases, edit).asserts["correct"], refusal);
    std::fs::remove_dir_all(&root).ok();
}

// ---- P2-6: the model does not choose between overlapping guards ----------------------------------

#[test]
fn either_answer_to_overlapping_guards_passes_and_the_draw_is_reported_ambiguous() {
    for lane in [&lanes().0, &lanes().1] {
        for name in ["correct", "grade-high"] {
            assert_eq!(lane.asserts[name], "ok", "{name}");
            assert_eq!(
                strings(&result(lane, name)["ambiguous"]),
                ["explore.desk.Grade: high, low"],
                "{name}"
            );
        }
    }
}

// ---- P2-7: the generator is mulberry32, bit for bit ------------------------------------------------

/// mulberry32, as the design writes it, in Rust.
fn mulberry32(seed: u32, count: usize) -> Vec<u32> {
    let mut state = seed;
    (0..count)
        .map(|_| {
            state = state.wrapping_add(0x6d2b_79f5);
            let mut t = state;
            t = (t ^ (t >> 15)).wrapping_mul(t | 0x1);
            t ^= t.wrapping_add((t ^ (t >> 7)).wrapping_mul(t | 0x3d));
            t ^ (t >> 14)
        })
        .collect()
}

const SEED_ONE: [u32; 5] = [
    2_693_262_067,
    11_749_833,
    2_265_367_787,
    4_213_581_821,
    4_159_151_403,
];

#[test]
fn seed_one_draws_the_pinned_outputs_in_rust_typescript_and_go() {
    assert_eq!(mulberry32(1, 5), SEED_ONE);
    let expected = SEED_ONE.map(|it| it.to_string()).join(" ");
    assert_eq!(lanes().0.mulberry32, expected);
    assert_eq!(lanes().1.mulberry32, expected);
}

// ---- P2-8: ir.json is the compact IR, bound to the suite's digest ----------------------------------

#[test]
fn the_emitted_ir_json_is_the_compact_ir_and_hashes_to_the_suites_digest() {
    use sha2::{Digest as _, Sha256};
    use std::fmt::Write as _;

    let ir = ir();
    let suite = suite(&ir);
    let ts = ess_conformance::ts::emit_with_model(&suite, &ir).unwrap();
    let go = ess_conformance::go::emit_with_model(&suite, &ir).unwrap();
    let ts_ir = ts
        .iter()
        .find(|file| file.path == "essconform/ir.json")
        .expect("ts ir.json");
    let go_ir = go
        .iter()
        .find(|file| file.path == "essconform/ir.json")
        .expect("go ir.json");
    assert_eq!(ts_ir.contents, format!("{}\n", ir.to_compact_json()));
    assert_eq!(go_ir.contents, ts_ir.contents);
    let mut digest = String::new();
    for byte in Sha256::digest(ts_ir.contents.trim_end_matches('\n').as_bytes()) {
        let _ = write!(digest, "{byte:02x}");
    }
    assert_eq!(digest, suite.provenance.spec_digest.to_string());
}

#[test]
fn the_packages_with_a_model_add_the_explorer_and_nothing_else_moves() {
    let ir = ir();
    let suite = suite(&ir);
    let plain = ess_conformance::ts::emit(&suite).unwrap();
    let with = ess_conformance::ts::emit_with_model(&suite, &ir).unwrap();
    let paths: Vec<&str> = with.iter().map(|file| file.path.as_str()).collect();
    assert_eq!(
        paths,
        [
            "essconform/src/runtime.ts",
            "essconform/src/predicate.ts",
            "essconform/src/response.ts",
            "essconform/src/direct_response.ts",
            "essconform/src/one_time_response.ts",
            "essconform/src/fixtures.ts",
            "essconform/src/reading.ts",
            "essconform/src/coordinate.ts",
            "essconform/src/explore.ts",
            "essconform/src/index.ts",
            "essconform/suite.json",
            "essconform/ir.json",
            "essconform/package.json",
            "essconform/tsconfig.json",
            "essconform/README.md",
        ]
    );
    for file in &plain {
        let added = with.iter().find(|it| it.path == file.path).unwrap();
        match file.path.as_str() {
            "essconform/src/index.ts" => {
                assert_eq!(
                    added.contents,
                    format!("{}export * from './explore.js';\n", file.contents)
                );
            }
            "essconform/README.md" => {
                assert!(added.contents.starts_with(&file.contents));
                assert!(added.contents.contains("## Random command sequences"));
            }
            _ => assert_eq!(added, file),
        }
    }

    let plain = ess_conformance::go::emit(&suite).unwrap();
    let with = ess_conformance::go::emit_with_model(&suite, &ir).unwrap();
    let paths: Vec<&str> = with.iter().map(|file| file.path.as_str()).collect();
    assert_eq!(
        paths,
        [
            "essconform/runtime.go",
            "essconform/predicate.go",
            "essconform/suite.go",
            "essconform/explore.go",
            "essconform/suite.json",
            "essconform/ir.json",
            "essconform/README.md",
        ]
    );
    for file in &plain {
        let added = with.iter().find(|it| it.path == file.path).unwrap();
        if file.path == "essconform/README.md" {
            assert!(added.contents.starts_with(&file.contents));
            assert!(added.contents.contains("## Random command sequences"));
        } else {
            assert_eq!(added, file);
        }
    }
}

// ---- beyond10x/ess#536: a command that preserves a supplied subject is drawn ---------------------

const PRESERVES_FIXTURE: &str = include_str!("fixtures/explore-preserves.yaml");

fn preserves_ir() -> EssIr {
    let mut sources = SourceMap::new();
    sources.insert("explore-preserves.yaml", PRESERVES_FIXTURE);
    let raw = RawSpecFile::parse(PRESERVES_FIXTURE).expect("the fixture parses");
    let specification = Specification::assemble(vec![(Source::new("explore-preserves.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("the fixture validates:\n{errors}"));
    compile(&specification, &sources).unwrap_or_else(|diagnostics| panic!("{diagnostics}"))
}

fn read_fixture(name: &str) -> String {
    std::fs::read_to_string(fixture(name)).expect("the fixture reads")
}

/// One lane over `explore-preserves.yaml`: each case's result and assertion, and the runner log.
#[derive(Debug)]
struct KeepLane {
    results: BTreeMap<String, Value>,
    asserts: BTreeMap<String, String>,
    /// The whole text of each failed assertion, by case.
    messages: BTreeMap<String, String>,
    log: String,
}

fn read_keep_lane(out: &Path, cases: &Value, log: String) -> KeepLane {
    let mut results = BTreeMap::new();
    let mut asserts = BTreeMap::new();
    let mut messages = BTreeMap::new();
    for case in cases.as_array().expect("a case list") {
        let name = case["name"].as_str().expect("a case name").to_owned();
        let assert = std::fs::read_to_string(out.join(format!("{name}.assert")))
            .unwrap_or_else(|_| panic!("`{name}` wrote no assertion:\n{log}"));
        let text = std::fs::read_to_string(out.join(format!("{name}.json")))
            .unwrap_or_else(|_| panic!("`{name}` wrote no result ({assert}):\n{log}"));
        results.insert(name.clone(), serde_json::from_str(&text).expect("JSON"));
        if let Ok(message) = std::fs::read_to_string(out.join(format!("{name}.message"))) {
            messages.insert(name.clone(), message.trim_end().to_owned());
        }
        asserts.insert(name, assert.trim_end().to_owned());
    }
    KeepLane {
        results,
        asserts,
        messages,
        log,
    }
}

/// The TypeScript lane over `explore-preserves.yaml`, run by the preconditions driver with the
/// target swapped in.
fn keep_typescript(root: &Path, ir: &EssIr, cases: &Value) -> KeepLane {
    driven_typescript(root, ir, cases, "explore-preserves-target.mjs")
}

/// The TypeScript lane over `ir`, run by the preconditions driver with the fixture `target` swapped
/// in for its own.
fn driven_typescript(root: &Path, ir: &EssIr, cases: &Value, target: &str) -> KeepLane {
    let package = root.join("typescript");
    for artifact in ess_conformance::ts::emit_with_model(&suite(ir), ir).unwrap() {
        write(&package.join(&artifact.path), &artifact.contents);
    }
    let ts = package.join("essconform");
    std::fs::copy(fixture(target), ts.join(target)).expect("the target copies");
    let driver = read_fixture("explore-preconditions-driver.mjs")
        .replace("./explore-preconditions-target.mjs", &format!("./{target}"));
    write(&ts.join("preserves-driver.mjs"), &driver);
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
        .args(["--test", "preserves-driver.mjs"])
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
    read_keep_lane(&out, cases, log)
}

/// The Go lane over `explore-preserves.yaml`, run by the preconditions driver with the target
/// swapped in.
fn keep_go(root: &Path, ir: &EssIr, cases: &Value) -> KeepLane {
    driven_go(
        root,
        ir,
        cases,
        "explore_preserves_target.go",
        "newExploreKeepTarget",
    )
}

/// The Go lane over `ir`, run by the preconditions driver with the fixture `target` swapped in for
/// its own, built by the Go function `constructor`.
fn driven_go(root: &Path, ir: &EssIr, cases: &Value, target: &str, constructor: &str) -> KeepLane {
    let module = root.join("go");
    for artifact in ess_conformance::go::emit_with_model(&suite(ir), ir).unwrap() {
        write(&module.join(&artifact.path), &artifact.contents);
    }
    write(
        &module.join("go.mod"),
        "module example.invalid/explorekeep\n\ngo 1.24\n",
    );
    let stem = target.trim_end_matches(".go");
    std::fs::copy(
        fixture(target),
        module.join(format!("essconform/{stem}_test.go")),
    )
    .expect("the target copies");
    let driver = read_fixture("explore_preconditions_driver_test.go").replace(
        "newExplorePreTarget(one.Mode)",
        &format!("{constructor}(one.Mode)"),
    );
    write(
        &module.join("essconform/explore_preserves_driver_test.go"),
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
    read_keep_lane(&out, cases, log)
}

/// Every mode of the preserving target, run once per lane: `(typescript, go)`. `Purge` deletes its
/// subject and stays excluded, so every case accepts excluded outcomes.
fn keep_lanes() -> &'static (KeepLane, KeepLane) {
    static LANES: OnceLock<(KeepLane, KeepLane)> = OnceLock::new();
    LANES.get_or_init(|| {
        let ir = preserves_ir();
        let cases = serde_json::json!([
            {"name": "correct", "allowExcluded": true},
            {"name": "again-resumes", "mode": "again-resumes", "allowExcluded": true},
            {"name": "paused-relabels", "mode": "paused-relabels", "allowExcluded": true},
        ]);
        let root = scratch("preserves");
        let lanes = (
            keep_typescript(&root, &ir, &cases),
            keep_go(&root, &ir, &cases),
        );
        std::fs::remove_dir_all(&root).ok();
        lanes
    })
}

fn keep_each_lane() -> [(&'static str, &'static KeepLane); 2] {
    [("typescript", &keep_lanes().0), ("go", &keep_lanes().1)]
}

/// Every declared outcome of `explore-preserves.yaml` but `Purge`'s.
const KEPT_REACHED: &[&str] = &[
    "explorekeep.desk.Pause/already-paused",
    "explorekeep.desk.Pause/paused",
    "explorekeep.desk.PauseAgain/not-paused",
    "explorekeep.desk.PauseAgain/still-paused",
    "explorekeep.desk.Resume/not-active",
    "explorekeep.desk.Resume/resumed",
    "explorekeep.desk.Start/started",
];

fn assert_preserving_commands_drawn(language: &str, lane: &KeepLane) {
    let found = &lane.results["correct"];
    assert_eq!(
        lane.asserts["correct"], "ok",
        "{language}: {found}\n{}",
        lane.log
    );
    assert!(found.get("failure").is_none(), "{language}: {found}");
    // `Excluded` names neither `Pause` nor `PauseAgain`: only `Purge`, with today's reason.
    assert_eq!(
        found["excluded"],
        serde_json::json!([{
            "reason": "outcome `purged` deletes its subject from `supplied`",
            "subject": "explorekeep.desk.Purge"
        }]),
        "{language}: {found}"
    );
    assert_eq!(
        strings(&found["excludedOutcomes"]),
        ["explorekeep.desk.Purge/purged"],
        "{language}: {found}"
    );
    assert_eq!(
        strings(&found["reached"]),
        KEPT_REACHED,
        "{language}: {found}"
    );
    assert!(
        strings(&found["unreached"]).is_empty(),
        "{language}: {found}"
    );
}

#[test]
fn a_preserving_command_on_a_supplied_subject_is_drawn() {
    assert_preserving_commands_drawn("go", &keep_lanes().1);
}

#[test]
fn a_target_that_changes_a_preserved_subject_fails_exploration() {
    for (language, lane) in keep_each_lane() {
        for (mode, command, view) in [
            (
                "again-resumes",
                "explorekeep.desk.PauseAgain",
                "explorekeep.desk.PausedSessions",
            ),
            (
                "paused-relabels",
                "explorekeep.desk.Pause",
                "explorekeep.desk.PausedSessions",
            ),
        ] {
            let found = &lane.results[mode];
            assert!(
                lane.asserts[mode].starts_with("failed: explore: seed "),
                "{language}: `{mode}` passed: {}\n{found}",
                lane.asserts[mode]
            );
            let failure = &found["failure"];
            let message = failure["message"].as_str().unwrap_or_default();
            assert!(
                message.contains(view),
                "{language}: `{mode}` names no `{view}`: {failure}"
            );
            let trace = strings(&failure["trace"]);
            assert!(
                trace.last().is_some_and(|step| step.starts_with(command)),
                "{language}: `{mode}` ends on no `{command}` step: {failure}"
            );
            let step = format!("(step {}: {command} ", trace.len());
            assert!(
                message.contains(&step),
                "{language}: `{mode}` names no `{step}`: {failure}"
            );
        }
    }
}

#[test]
fn the_typescript_explorer_draws_a_preserving_command() {
    assert_preserving_commands_drawn("typescript", &keep_lanes().0);
    for name in ["correct", "again-resumes", "paused-relabels"] {
        assert_eq!(
            keep_lanes().0.results[name],
            keep_lanes().1.results[name],
            "`{name}` differs between TypeScript and Go"
        );
        assert_eq!(
            keep_lanes().0.asserts[name],
            keep_lanes().1.asserts[name],
            "`{name}`"
        );
    }
}

// ---- beyond10x/ess#538: the model applies a selected-rows effect and draws a `Decimal` -----------

const ROWS_FIXTURE: &str = include_str!("fixtures/explore-selected-rows.yaml");

fn rows_ir() -> EssIr {
    let mut sources = SourceMap::new();
    sources.insert("explore-selected-rows.yaml", ROWS_FIXTURE);
    let raw = RawSpecFile::parse(ROWS_FIXTURE).expect("the fixture parses");
    let specification =
        Specification::assemble(vec![(Source::new("explore-selected-rows.yaml"), raw)])
            .unwrap_or_else(|errors| panic!("the fixture validates:\n{errors}"));
    compile(&specification, &sources).unwrap_or_else(|diagnostics| panic!("{diagnostics}"))
}

/// Every mode of the selected-rows target, run once per lane: `(typescript, go)`.
fn rows_lanes() -> &'static (KeepLane, KeepLane) {
    static LANES: OnceLock<(KeepLane, KeepLane)> = OnceLock::new();
    LANES.get_or_init(|| {
        let ir = rows_ir();
        let cases = serde_json::json!([
            {"name": "correct"},
            {"name": "close-skips-one", "mode": "close-skips-one"},
            {"name": "close-every-team", "mode": "close-every-team"},
            {"name": "dial-adds-one", "mode": "dial-adds-one"},
        ]);
        let root = scratch("selected-rows");
        let lanes = (
            driven_typescript(&root, &ir, &cases, "explore-selected-rows-target.mjs"),
            driven_go(
                &root,
                &ir,
                &cases,
                "explore_selected_rows_target.go",
                "newExploreRowsTarget",
            ),
        );
        std::fs::remove_dir_all(&root).ok();
        lanes
    })
}

fn rows_each_lane() -> [(&'static str, &'static KeepLane); 2] {
    [("typescript", &rows_lanes().0), ("go", &rows_lanes().1)]
}

/// Every declared outcome of `explore-selected-rows.yaml`.
const ROWS_DECLARED: &[&str] = &[
    "explorerows.desk.Close/closed",
    "explorerows.desk.Dial/dialled",
    "explorerows.desk.Note/noted",
    "explorerows.desk.Reopen/not-closed",
    "explorerows.desk.Reopen/reopened",
    "explorerows.desk.Start/started",
];

/// The correct target passes with nothing excluded and every declared outcome reached.
fn assert_rows_correct(language: &str, lane: &KeepLane) {
    let found = &lane.results["correct"];
    assert_eq!(
        lane.asserts["correct"], "ok",
        "{language}: {found}\n{}",
        lane.log
    );
    assert!(found.get("failure").is_none(), "{language}: {found}");
    assert_eq!(
        found["excluded"],
        serde_json::json!([]),
        "{language}: {found}"
    );
    assert_eq!(
        strings(&found["reached"]),
        ROWS_DECLARED,
        "{language}: {found}"
    );
    assert!(
        strings(&found["undetermined"]).is_empty(),
        "{language}: {found}"
    );
}

/// `mode` fails exploration on a step of `command`, the last of its shrunk trace, with a message
/// naming `needle`; the trace is returned.
fn assert_rows_fail<'a>(
    language: &str,
    lane: &'a KeepLane,
    mode: &str,
    command: &str,
    needle: &str,
) -> Vec<&'a str> {
    let found = &lane.results[mode];
    assert!(
        lane.asserts[mode].starts_with("failed: explore: seed "),
        "{language}: `{mode}` passed: {}\n{found}",
        lane.asserts[mode]
    );
    let failure = &found["failure"];
    let message = failure["message"].as_str().unwrap_or_default();
    assert!(
        message.contains(needle),
        "{language}: `{mode}` names no `{needle}`: {failure}"
    );
    let trace = strings(&failure["trace"]);
    assert!(
        trace.last().is_some_and(|step| step.starts_with(command)),
        "{language}: `{mode}` ends on no `{command}` step: {failure}"
    );
    let step = format!("(step {}: {command} ", trace.len());
    assert!(
        message.contains(&step),
        "{language}: `{mode}` names no `{step}`: {failure}"
    );
    trace
}

#[test]
fn the_model_moves_and_sets_every_selected_row() {
    for (language, lane) in rows_each_lane() {
        assert_rows_correct(language, lane);
        assert_rows_fail(
            language,
            lane,
            "close-skips-one",
            "explorerows.desk.Close",
            "view-field: `explorerows.desk.Seats`.last_reason of ",
        );
    }
}

#[test]
fn a_selected_row_effect_leaves_unselected_rows_alone() {
    for (language, lane) in rows_each_lane() {
        // A model that moved or set every row would fail the correct target here.
        assert_rows_correct(language, lane);
        assert_rows_fail(
            language,
            lane,
            "close-every-team",
            "explorerows.desk.Close",
            "view-field: `explorerows.desk.Seats`.",
        );
    }
}

#[test]
fn a_decimal_input_is_drawn_sequentially() {
    for (language, lane) in rows_each_lane() {
        assert_rows_correct(language, lane);
        let trace = assert_rows_fail(
            language,
            lane,
            "dial-adds-one",
            "explorerows.desk.Dial",
            "view-field: `explorerows.desk.Seats`.level of ",
        );
        // `Decimal` declares no scale of its own: the drawn values are whole numbers, as an
        // `Integer`'s are, which every scale holds exactly.
        let mut drawn = 0;
        for step in trace {
            let (command, input) = step.split_once(' ').expect("a step names its input");
            let input: Value = serde_json::from_str(input).expect("the input is JSON");
            let field = match command {
                "explorerows.desk.Dial" => "dial_level",
                "explorerows.desk.Start" => "level",
                _ => continue,
            };
            let value = &input[field];
            assert!(
                value.as_number().is_some_and(serde_json::Number::is_i64),
                "{language}: {command}.{field} drew {value}"
            );
            drawn += 1;
        }
        assert!(drawn >= 2, "{language}: no decimal was drawn");
    }
}

#[test]
fn the_typescript_explorer_agrees_on_selected_rows_and_decimals() {
    let (typescript, go) = rows_lanes();
    assert_eq!(typescript.results.len(), 4);
    for (name, result) in &typescript.results {
        assert_eq!(
            result, &go.results[name],
            "`{name}` differs between TypeScript and Go"
        );
        assert_eq!(typescript.asserts[name], go.asserts[name], "`{name}`");
    }
}

/// Which unwritten stored fields a selected-rows filter may read (adversary pass 1 on
/// beyond10x/ess#538). An `Optional` field no outcome wrote is absent, as the interpreter's
/// `only_absent` reads it: `SweepTagged` selects no seat and is never reported. A required field no
/// outcome wrote has no value the model holds: `Sweep` beside a seat is reported in `undetermined`,
/// under a message that no longer claims no command sets the field.
#[test]
fn a_selected_rows_filter_reads_an_unwritten_optional_as_absent() {
    const UNKNOWN: &str = include_str!("fixtures/explore-selected-rows-unknown.yaml");
    let mut sources = SourceMap::new();
    sources.insert("explore-selected-rows-unknown.yaml", UNKNOWN);
    let raw = RawSpecFile::parse(UNKNOWN).expect("the fixture parses");
    let specification = Specification::assemble(vec![(
        Source::new("explore-selected-rows-unknown.yaml"),
        raw,
    )])
    .unwrap_or_else(|errors| panic!("the fixture validates:\n{errors}"));
    let ir =
        compile(&specification, &sources).unwrap_or_else(|diagnostics| panic!("{diagnostics}"));
    let cases = serde_json::json!([{"name": "correct"}]);
    let root = scratch("selected-rows-unknown");
    let lanes = [
        (
            "typescript",
            driven_typescript(
                &root,
                &ir,
                &cases,
                "explore-selected-rows-unknown-target.mjs",
            ),
        ),
        (
            "go",
            driven_go(
                &root,
                &ir,
                &cases,
                "explore_selected_rows_unknown_target.go",
                "newExploreUnkTarget",
            ),
        ),
    ];
    std::fs::remove_dir_all(&root).ok();
    for (language, lane) in &lanes {
        let found = &lane.results["correct"];
        assert_eq!(
            lane.asserts["correct"], "ok",
            "{language}: {found}\n{}",
            lane.log
        );
        assert_eq!(
            strings(&found["undetermined"]),
            [
                "exploreunk.desk.Sweep filter of `swept` reads zone, whose value the model does \
                 not hold"
            ],
            "{language}: {found}"
        );
        assert_eq!(
            strings(&found["reached"]),
            [
                "exploreunk.desk.Start/started",
                "exploreunk.desk.Sweep/swept",
                "exploreunk.desk.SweepTagged/swept-tagged"
            ],
            "{language}: {found}"
        );
    }
}

// ---- beyond10x/ess#537: an unsupported step excludes the command only in that state -------------

const REFUSED_FIXTURE: &str = include_str!("fixtures/explore-refused.yaml");

/// Every mode of the refusing target, run once per lane: `(typescript, go)`.
fn refused_lanes() -> &'static (KeepLane, KeepLane) {
    static LANES: OnceLock<(KeepLane, KeepLane)> = OnceLock::new();
    LANES.get_or_init(|| {
        let mut sources = SourceMap::new();
        sources.insert("explore-refused.yaml", REFUSED_FIXTURE);
        let raw = RawSpecFile::parse(REFUSED_FIXTURE).expect("the fixture parses");
        let specification =
            Specification::assemble(vec![(Source::new("explore-refused.yaml"), raw)])
                .unwrap_or_else(|errors| panic!("the fixture validates:\n{errors}"));
        let ir =
            compile(&specification, &sources).unwrap_or_else(|diagnostics| panic!("{diagnostics}"));
        let cases = serde_json::json!([
            {"name": "correct"},
            {"name": "ringing", "mode": "ringing"},
            {"name": "ringing-allowed", "mode": "ringing", "allowExcluded": true},
            {"name": "answer", "mode": "answer"},
            {"name": "answer-allowed", "mode": "answer", "allowExcluded": true},
        ]);
        let root = scratch("refused");
        let lanes = (
            driven_typescript(&root, &ir, &cases, "explore-refused-target.mjs"),
            driven_go(
                &root,
                &ir,
                &cases,
                "explore_refused_target.go",
                "newExploreRefusedTarget",
            ),
        );
        std::fs::remove_dir_all(&root).ok();
        lanes
    })
}

/// The view every mode refuses, excluded per view as before #537.
fn assert_view_excluded(language: &str, found: &Value) -> Vec<Value> {
    let excluded = found["excluded"].as_array().expect("a list").clone();
    let view: Vec<&Value> = excluded
        .iter()
        .filter(|item| item["subject"] == "explorerefused.desk.RingingCalls")
        .collect();
    assert_eq!(view.len(), 1, "{language}: {found}");
    assert!(
        view[0]["reason"].as_str().is_some_and(|reason| reason
            .starts_with("the target does not expose it: ")
            && reason.ends_with("is not served")),
        "{language}: {found}"
    );
    excluded
        .into_iter()
        .filter(|item| item["subject"] != "explorerefused.desk.RingingCalls")
        .collect()
}

/// `Answer` refused while ringing is drawn in `Waiting` and `Connected`: `picked-up` and `not-open`
/// are reached, one refusal is reported for (`Answer`, `Ringing`) with its reason and count, and only
/// `answered`, which no step outside `Ringing` can take, is left to `AllowExcluded`.
fn assert_narrowed(language: &str, lane: &KeepLane) {
    let correct = &lane.results["correct"];
    assert_eq!(
        lane.asserts["correct"], "ok",
        "{language}: {correct}\n{}",
        lane.log
    );
    assert!(correct.get("refused").is_none(), "{language}: {correct}");
    assert!(
        assert_view_excluded(language, correct).is_empty(),
        "{language}: {correct}"
    );

    let found = &lane.results["ringing"];
    assert!(found.get("failure").is_none(), "{language}: {found}");
    assert!(
        assert_view_excluded(language, found).is_empty(),
        "{language}: `Answer` is excluded for the whole run: {found}"
    );
    let reached = strings(&found["reached"]);
    for outcome in [
        "explorerefused.desk.Answer/picked-up",
        "explorerefused.desk.Answer/not-open",
    ] {
        assert!(
            reached.contains(&outcome),
            "{language}: {outcome} unreached: {found}"
        );
    }
    let refused = found["refused"]
        .as_array()
        .unwrap_or_else(|| panic!("{language}: {found}"));
    assert_eq!(refused.len(), 1, "{language}: {found}");
    assert_eq!(
        refused[0]["command"], "explorerefused.desk.Answer",
        "{language}: {found}"
    );
    assert_eq!(refused[0]["state"], "Ringing", "{language}: {found}");
    assert_eq!(
        refused[0]["reason"],
        "the target does not expose it: Answer has no callback while the call rings",
        "{language}: {found}"
    );
    assert!(
        refused[0]["count"].as_u64().is_some_and(|count| count >= 1),
        "{language}: {found}"
    );
    assert_eq!(
        strings(&found["excludedOutcomes"]),
        ["explorerefused.desk.Answer/answered"],
        "{language}: {found}"
    );
    assert!(
        strings(&found["unreached"]).is_empty(),
        "{language}: {found}"
    );
    assert_eq!(
        lane.asserts["ringing"],
        "failed: explore: 1 declared outcome(s) of excluded commands were never tried:",
        "{language}"
    );
    assert_eq!(lane.asserts["ringing-allowed"], "ok", "{language}");
}

/// `Answer` refused in every state ends in `excluded`, as before #537, with every outcome left to
/// `AllowExcluded` and no refusal reported beside it.
fn assert_still_excluded(language: &str, lane: &KeepLane) {
    let found = &lane.results["answer"];
    assert!(found.get("failure").is_none(), "{language}: {found}");
    assert_eq!(
        assert_view_excluded(language, found),
        [serde_json::json!({
            "reason": "the target does not expose it: Answer has no callback",
            "subject": "explorerefused.desk.Answer"
        })],
        "{language}: {found}"
    );
    assert!(found.get("refused").is_none(), "{language}: {found}");
    assert_eq!(
        strings(&found["excludedOutcomes"]),
        [
            "explorerefused.desk.Answer/answered",
            "explorerefused.desk.Answer/not-open",
            "explorerefused.desk.Answer/picked-up",
        ],
        "{language}: {found}"
    );
    assert!(
        strings(&found["unreached"]).is_empty(),
        "{language}: {found}"
    );
    assert_eq!(
        lane.asserts["answer"],
        "failed: explore: 3 declared outcome(s) of excluded commands were never tried:",
        "{language}"
    );
    assert_eq!(lane.asserts["answer-allowed"], "ok", "{language}");
}

#[test]
fn an_unsupported_step_excludes_the_command_only_in_that_state() {
    assert_narrowed("go", &refused_lanes().1);
}

#[test]
fn a_command_unsupported_in_every_state_is_still_excluded() {
    assert_still_excluded("go", &refused_lanes().1);
}

#[test]
fn the_typescript_explorer_narrows_an_unsupported_step() {
    let (typescript, go) = refused_lanes();
    assert_narrowed("typescript", typescript);
    assert_still_excluded("typescript", typescript);
    assert_eq!(typescript.results.len(), 5);
    for (name, result) in &typescript.results {
        assert_eq!(
            result, &go.results[name],
            "`{name}` differs between TypeScript and Go"
        );
        assert_eq!(typescript.asserts[name], go.asserts[name], "`{name}`");
    }
}

/// The lines of `message` from the one equal to `heading` up to the next `accept them explicitly`
/// line, without either.
fn section<'a>(message: &'a str, heading: &str) -> Vec<&'a str> {
    message
        .lines()
        .skip_while(|line| *line != heading)
        .skip(1)
        .take_while(|line| !line.starts_with("accept them explicitly"))
        .collect()
}

/// A refusal has its own heading in the assertion, naming the command, the held state, the
/// target's reason and the steps not sent; the excluded list names only what was excluded, and an
/// excluded outcome that only a refused step would have taken says so.
fn assert_refusal_lines(language: &str, lane: &KeepLane) {
    let message = &lane.messages["ringing"];
    let count = lane.results["ringing"]["refused"][0]["count"]
        .as_u64()
        .expect("a count");
    let outcomes = section(
        message,
        "explore: 1 declared outcome(s) of excluded commands were never tried:",
    );
    assert_eq!(
        outcomes.first(),
        Some(
            &"  explorerefused.desk.Answer/answered (only a step the target refused would have taken it)"
        ),
        "{language}:\n{message}"
    );
    let excluded: Vec<&str> = outcomes
        .iter()
        .skip_while(|line| **line != "excluded:")
        .copied()
        .collect();
    assert!(
        excluded.len() > 1
            && excluded[1..]
                .iter()
                .all(|line| line.starts_with("  explorerefused.desk.RingingCalls: ")),
        "{language}: a refusal is listed as an exclusion:\n{message}"
    );
    assert_eq!(
        section(
            message,
            "explore: 1 (command, held state) pair(s) the target refused were never tried again:"
        ),
        [format!(
            "  explorerefused.desk.Answer in Ringing: the target does not expose it: Answer has no \
             callback while the call rings ({count} step(s) not sent)"
        )],
        "{language}:\n{message}"
    );

    let message = &lane.messages["answer"];
    assert!(
        !message.contains("the target refused were never tried again"),
        "{language}: an exclusion is listed as a refusal:\n{message}"
    );
    assert!(
        message
            .lines()
            .any(|line| line
                == "  explorerefused.desk.Answer: the target does not expose it: Answer has no callback"),
        "{language}:\n{message}"
    );
    assert!(
        !message.contains("(only a step the target refused"),
        "{language}:\n{message}"
    );
    for allowed in ["ringing-allowed", "answer-allowed"] {
        assert!(
            !lane.messages.contains_key(allowed),
            "{language}: {allowed}"
        );
    }
}

#[test]
fn a_refusal_has_its_own_lines_in_the_assertion_in_both_languages() {
    let (typescript, go) = refused_lanes();
    assert_refusal_lines("go", go);
    assert_refusal_lines("typescript", typescript);
    let without_option = |message: &str| -> Vec<String> {
        message
            .lines()
            .filter(|line| !line.starts_with("accept them explicitly"))
            .map(str::to_owned)
            .collect()
    };
    for (name, message) in &go.messages {
        assert_eq!(
            without_option(message),
            without_option(&typescript.messages[name]),
            "`{name}` differs between TypeScript and Go"
        );
    }
    assert_eq!(
        go.messages.keys().collect::<Vec<_>>(),
        typescript.messages.keys().collect::<Vec<_>>()
    );
}
