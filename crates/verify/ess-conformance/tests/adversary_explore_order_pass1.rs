//! Adversary pass 1 on `story:explorers-decide-input-guards-before-wrong-state`
//! (beyond10x/ess#235), both emitted languages.
//!
//! `tests/fixtures/explore-adv-order.yaml` puts the new order beside external branches, which
//! `explore-guards-first.yaml` has none of and `explore-external-exits.yaml` (after this unit
//! turned its refusals into accepting branches) has no input-guarded refusal beside.
//!
//! 1. `Close`: an arranged external branch that moves nothing (`bounced`) is a branch Entity
//!    Runtime selects before the default, so it answers in `Closed` too; the wrong-state answer is
//!    only for the default's move. The explorer answers `wrong-state` there. See
//!    `ess-entity-runtime/tests/adversary_explorer_order_pass1.rs` for the Entity Runtime half.
//! 2. `Rate`: an input-guarded refusal answers before an arranged external verdict. Nothing else in
//!    the suite pins that; a mutant that lets the external branch answer beside the refusal is
//!    caught here.
//!
//! A missing `tsc`, `node` or `go` panics rather than skipping.
#![allow(clippy::too_many_lines)]

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

const FIXTURE: &str = include_str!("fixtures/explore-adv-order.yaml");

type Mutation = fn(&str, String) -> String;

fn ir() -> EssIr {
    ir_of("explore-adv-order.yaml", FIXTURE)
}

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
        "ess-adv-explore-order-{}-{label}",
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

fn typescript(root: &Path, ir: &EssIr, mutate: Mutation, cases: &Value) -> Lane {
    let package = root.join("typescript");
    for artifact in ess_conformance::ts::emit_with_model(&suite(ir), ir).unwrap() {
        let contents = mutate(&artifact.path, artifact.contents.clone());
        write(&package.join(&artifact.path), &contents);
    }
    let ts = package.join("essconform");
    std::fs::copy(
        fixture("explore-adv-order-target.mjs"),
        ts.join("explore-adv-order-target.mjs"),
    )
    .expect("the target copies");
    let driver = read_fixture("explore-preconditions-driver.mjs").replace(
        "./explore-preconditions-target.mjs",
        "./explore-adv-order-target.mjs",
    );
    write(&ts.join("adv-order-driver.mjs"), &driver);
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
        .args(["--test", "adv-order-driver.mjs"])
        .env("ESS_EXPLORE_CASES", cases.to_string())
        .env("ESS_EXPLORE_OUT", &out)
        .current_dir(&ts)
        .output()
        .expect("`node` is on PATH");
    let log = printed(&run);
    assert!(run.status.success(), "{log}");
    read_lane(&out, cases, log)
}

fn go(root: &Path, ir: &EssIr, mutate: Mutation, cases: &Value) -> Lane {
    let module = root.join("go");
    for artifact in ess_conformance::go::emit_with_model(&suite(ir), ir).unwrap() {
        let contents = mutate(&artifact.path, artifact.contents.clone());
        write(&module.join(&artifact.path), &contents);
    }
    write(
        &module.join("go.mod"),
        "module example.invalid/exploreadv\n\ngo 1.24\n",
    );
    std::fs::copy(
        fixture("explore_adv_order_target.go"),
        module.join("essconform/explore_adv_order_target_test.go"),
    )
    .expect("the target copies");
    let driver = read_fixture("explore_preconditions_driver_test.go").replace(
        "newExplorePreTarget(one.Mode)",
        "newExploreAdvTarget(one.Mode)",
    );
    write(
        &module.join("essconform/explore_adv_order_driver_test.go"),
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
        .expect("`go` is on PATH");
    let log = printed(&run);
    assert!(run.status.success(), "{log}");
    read_lane(&out, cases, log)
}

fn both(label: &str, mutate: Mutation, cases: &Value) -> [(&'static str, Lane); 2] {
    both_of(&ir(), label, mutate, cases)
}

fn both_of(ir: &EssIr, label: &str, mutate: Mutation, cases: &Value) -> [(&'static str, Lane); 2] {
    let root = scratch(label);
    let lanes = [
        ("typescript", typescript(&root, ir, mutate, cases)),
        ("go", go(&root, ir, mutate, cases)),
    ];
    std::fs::remove_dir_all(&root).ok();
    lanes
}

fn unchanged(_: &str, contents: String) -> String {
    contents
}

fn replace_once(path: &str, contents: &str, from: &str, to: &str) -> String {
    assert_eq!(
        contents.matches(from).count(),
        1,
        "the mutation site moved in {path}"
    );
    contents.replacen(from, to, 1)
}

/// The input-guarded refusal no longer shuts out the eligible external branches: an arranged
/// verdict answers beside it.
fn refusal_beside_externals(path: &str, contents: String) -> String {
    if path.ends_with("explore.go") {
        replace_once(
            path,
            &contents,
            "\t\t\treturn exploreDecision{kind: \"take\", outcome: outcome}\n",
            "\t\t\texternals, _ := exploreEligible(command, outcomes, source, record)\n\t\t\treturn exploreDecision{kind: \"take\", outcome: outcome, externals: externals}\n",
        )
    } else if path.ends_with("explore.ts") {
        replace_once(
            path,
            &contents,
            "if (truth === TruthTrue) return { kind: 'take', outcome, externals: [] };",
            "if (truth === TruthTrue) return { kind: 'take', outcome, externals: eligibleExternals(command, outcomes, source, record) as Node[] };",
        )
    } else {
        contents
    }
}

fn reached(found: &Value) -> Vec<&str> {
    found["reached"]
        .as_array()
        .unwrap_or_else(|| panic!("a reached list, not {found}"))
        .iter()
        .map(|item| item.as_str().expect("a string"))
        .collect()
}

/// Entity Runtime's order selects an arranged `bounced` (a guard over the provider's verdict,
/// sorted before the default) in `Closed` as in `Open`; `bounced` moves nothing, so `admit_state`
/// admits it, and `wrong-state` is not the answer. The explorer, whose own rule is "a branch that
/// moves nothing answers in every state", answers `wrong-state` there whatever is arranged.
#[test]
fn an_arranged_external_that_moves_nothing_answers_in_a_state_no_move_starts_from() {
    let cases = json!([{"name": "entity-runtime-order"}]);
    for (language, lane) in both("er-order", unchanged, &cases) {
        let found = &lane.results["entity-runtime-order"];
        assert_eq!(
            lane.asserts["entity-runtime-order"], "ok",
            "{language}: a target in Entity Runtime's order is reported as disagreeing\n{found}"
        );
    }
}

/// The unmutated explorer passes a target in Entity Runtime's order over `Rate`, reaching the
/// refusal and the external branch; the mutant that lets an arranged verdict answer beside the
/// refusal is caught, in both languages.
#[test]
fn a_refusal_answers_before_an_arranged_external_verdict() {
    let cases = json!([{"name": "rate", "mode": "no-close", "allowExcluded": true}]);
    for (language, lane) in both("rate", unchanged, &cases) {
        let found = &lane.results["rate"];
        assert_eq!(
            lane.asserts["rate"], "ok",
            "{language}: {found}\n{}",
            lane.log
        );
        let reached = reached(found);
        for outcome in [
            "exploreadv.desk.Rate/too-high",
            "exploreadv.desk.Rate/rated",
        ] {
            assert!(
                reached.contains(&outcome),
                "{language}: `{outcome}` unreached\n{found}"
            );
        }
        assert!(
            found["external"].as_array().unwrap().iter().any(|reach| {
                reach["outcome"] == "exploreadv.desk.Rate/throttled" && reach["reach"] == "reached"
            }),
            "{language}: `throttled` was never arranged\n{found}"
        );
    }
    for (language, lane) in both("rate-mutant", refusal_beside_externals, &cases) {
        let found = &lane.results["rate"];
        assert!(
            lane.asserts["rate"].starts_with("failed: "),
            "{language}: the mutant that answers an arranged verdict beside a refusal passed\n{found}"
        );
        assert!(
            found["failure"]
                .to_string()
                .contains("exploreadv.desk.Rate"),
            "{language}: {found}"
        );
    }
}

/// Origin probe: the same case against the base commit's explorer sources. Ignored by default; run
/// with `--ignored` from inside the worktree (reads `git show 300bfd3f5:…`).
#[test]
#[ignore = "origin probe against the base commit's explorer"]
fn origin_probe_base_explorer() {
    fn base(path: &str, contents: String) -> String {
        let source = if path.ends_with("explore.go") {
            "crates/verify/ess-conformance/src/go/explore.go"
        } else if path.ends_with("explore.ts") {
            "crates/verify/ess-conformance/src/ts/explore.ts"
        } else {
            return contents;
        };
        let shown = Command::new("git")
            .args(["show", &format!("300bfd3f5:{source}")])
            .current_dir(env!("CARGO_MANIFEST_DIR"))
            .output()
            .expect("git runs");
        assert!(shown.status.success(), "{}", printed(&shown));
        String::from_utf8(shown.stdout).expect("UTF-8")
    }
    let cases = json!([{"name": "entity-runtime-order", "mode": "no-rate", "allowExcluded": true}]);
    for (language, lane) in both("er-order-base", base, &cases) {
        let found = &lane.results["entity-runtime-order"];
        assert_eq!(
            lane.asserts["entity-runtime-order"], "ok",
            "{language} (base 300bfd3f5): {found}"
        );
    }
}

/// The overlap exit that decides nothing: `stranded && every holding branch moves` answers
/// `wrong-state` again as an ambiguous draw.
fn drop_all_move_overlap(path: &str, contents: String) -> String {
    if path.ends_with("explore.go") {
        replace_once(
            path,
            &contents,
            "if stranded && exploreAllMove(holding) {",
            "if false && stranded && exploreAllMove(holding) {",
        )
    } else if path.ends_with("explore.ts") {
        replace_once(
            path,
            &contents,
            "if (stranded && holding.every(",
            "if (false && stranded && holding.every(",
        )
    } else {
        contents
    }
}

const OVERLAP: &str = include_str!("fixtures/explore-adv-overlap.yaml");

/// `Resolve/wrong-state` is reached only through two accepting guards that both hold and both move
/// on a `Closed` ticket — the `stranded && all move` branch this unit added. The unmutated
/// explorer reaches it without disagreeing; the mutant that drops the branch leaves it unreached.
#[test]
fn two_moving_overlapping_guards_on_a_stranded_subject_answer_wrong_state() {
    let ir = ir_of("explore-adv-overlap.yaml", OVERLAP);
    let cases = json!([{"name": "overlap"}]);
    let wanted = "exploreadv.desk.Resolve/wrong-state";
    for (language, lane) in both_of(&ir, "overlap", unchanged, &cases) {
        let found = &lane.results["overlap"];
        assert!(found.get("failure").is_none(), "{language}: {found}");
        assert!(reached(found).contains(&wanted), "{language}: {found}");
    }
    for (language, lane) in both_of(&ir, "overlap-mutant", drop_all_move_overlap, &cases) {
        let found = &lane.results["overlap"];
        assert!(
            !reached(found).contains(&wanted),
            "{language}: the mutant still reaches `{wanted}`\n{found}"
        );
    }
}
