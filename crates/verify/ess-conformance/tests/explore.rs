//! The model-based explorer, executed in both emitted languages.
//!
//! `docs/design/mutation-audit-and-model-runner.md`, Part 2, and its deciding checks P2-1 to P2-8.
//! The fixture is `tests/fixtures/explore.yaml`; the targets are `explore-target.mjs` and
//! `explore_target.go`, each switched by the mutant name; the drivers run one exploration per case
//! and write its result, so this file can hold the two lanes to each other as well as to the pins.
//!
//! A missing `tsc`, `node` or `go` panics rather than skipping: a lane that ran nothing and exited
//! zero is the failure this repository already knows by name.

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

fn scratch(label: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("ess-explore-{}-{label}", std::process::id()));
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
        assert!(strings(&correct["unreached"]).is_empty());
        assert_eq!(correct["sequences"], 200);
        assert_eq!(correct["steps"], 60);
        assert_eq!(correct["executed"], 12000);
        assert!(strings(&correct["undetermined"]).is_empty(), "{correct}");
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
        assert!(!strings(&short["unreached"]).is_empty());
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
    let refusal =
        "refused: `ir.json` and `suite.json` come from different specifications; regenerate the package";
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
