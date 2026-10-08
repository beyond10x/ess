//! Second adversary pass on the explorer's external branches (beyond10x/ess#156), both languages.
//!
//! `explore-external-exits.yaml` drives the two decision exits the correction round changed without
//! tests: the no-move-state exit (`Close` from `Closed`, no `wrong_state`) and the
//! overlapping-guards exit (`Rate`, `low` and `high` both hold for 1..3). The no-outcome-holds exit
//! is not driven: the validator refuses a command without an `otherwise` (`non_exhaustive_branches`).
//!
//! 1. With a double that arranges, each exit takes its external branch: nothing at those exits is
//!    reported ambiguous, and nothing disagrees.
//! 2. Reverting each exit to its pre-fix `ambiguous` answer is caught by (1)'s assertion — a mutant
//!    per exit, applied to the emitted explorer.
//! 3. With a double that cannot arrange, a draw at those exits is neither executed nor redrawn as
//!    ambiguous: it vanishes. `ambiguous` is documented as "draws the specification does not
//!    decide, which were redrawn rather than executed", and these are exactly that.
//!
//! A missing `tsc`, `node` or `go` panics rather than skipping.

mod support_scratch;

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

const NO_MOVE: &str = "exploreexit.desk.Close: no outcome for state Closed";
const OVERLAP: &str = "exploreexit.desk.Rate: low, high";

fn ir() -> EssIr {
    let name = "explore-external-exits.yaml";
    let text = std::fs::read_to_string(fixture(name)).expect("the fixture reads");
    let mut sources = SourceMap::new();
    sources.insert(name, &text);
    let raw = RawSpecFile::parse(&text).expect("the fixture parses");
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

fn scratch(label: &str) -> support_scratch::Scratch {
    let root = support_scratch::Scratch::adopt(std::env::temp_dir().join(format!(
        "ess-explore-external-exits-{}-{label}",
        std::process::id()
    )));
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

type Mutation = fn(&str, String) -> String;

fn typescript(root: &Path, ir: &EssIr, mutate: Mutation, cases: &Value) -> Lane {
    let package = root.join("typescript");
    for artifact in ess_conformance::ts::emit_with_model(&suite(ir), ir).unwrap() {
        let contents = mutate(&artifact.path, artifact.contents.clone());
        write(&package.join(&artifact.path), &contents);
    }
    let ts = package.join("essconform");
    std::fs::copy(
        fixture("explore-external-exits-target.mjs"),
        ts.join("explore-external-exits-target.mjs"),
    )
    .unwrap();
    let driver = read_fixture("explore-external-driver.mjs").replace(
        "./explore-external-target.mjs",
        "./explore-external-exits-target.mjs",
    );
    write(&ts.join("exits-driver.mjs"), &driver);
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
        .args(["--test", "exits-driver.mjs"])
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
        "module example.invalid/exploreexit\n\ngo 1.24\n",
    );
    std::fs::copy(
        fixture("explore_external_exits_target.go"),
        module.join("essconform/exits_target_test.go"),
    )
    .unwrap();
    let driver = read_fixture("explore_external_driver_test.go").replace(
        "newExploreExtTarget(one.Mode)",
        "newExploreExitTarget(one.Mode)",
    );
    write(&module.join("essconform/exits_driver_test.go"), &driver);
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

fn both(label: &str, mutate: Mutation, cases: &Value) -> [(&'static str, Lane); 2] {
    let ir = ir();
    let root = scratch(label);
    let lanes = [
        ("typescript", typescript(&root, &ir, mutate, cases)),
        ("go", go(&root, &ir, mutate, cases)),
    ];
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

fn unchanged(_: &str, contents: String) -> String {
    contents
}

fn replace_first(path: &str, contents: &str, from: &str, to: &str, count: usize) -> String {
    assert_eq!(
        contents.matches(from).count(),
        count,
        "the mutation site moved in {path}"
    );
    contents.replacen(from, to, 1)
}

/// The no-move-state exit answers `ambiguous` again, as before the correction round.
fn revert_no_move(path: &str, contents: String) -> String {
    if path.ends_with("explore.go") {
        replace_first(
            path,
            &contents,
            "return orExternal(exploreDecision{kind: \"ambiguous\", names: []string{fmt.Sprintf(\"no outcome for state",
            "return (exploreDecision{kind: \"ambiguous\", names: []string{fmt.Sprintf(\"no outcome for state",
            1,
        )
    } else if path.ends_with("explore.ts") {
        replace_first(
            path,
            &contents,
            "return orExternal({\n      kind: 'ambiguous',\n      names: [`no outcome for state",
            "return ({\n      kind: 'ambiguous',\n      names: [`no outcome for state",
            1,
        )
    } else {
        contents
    }
}

/// The overlapping-guards exit answers `ambiguous` again, as before the correction round.
fn revert_overlap(path: &str, contents: String) -> String {
    if path.ends_with("explore.go") {
        // The first of two identical lines is the overlapping-guards exit; the second is the
        // ordinary-branch-cannot-move exit that pass 1 covered.
        replace_first(
            path,
            &contents,
            "return orExternal(exploreDecision{kind: \"ambiguous\", names: names})",
            "return (exploreDecision{kind: \"ambiguous\", names: names})",
            2,
        )
    } else if path.ends_with("explore.ts") {
        replace_first(
            path,
            &contents,
            "return orExternal({ kind: 'ambiguous', names: holding.map(",
            "return ({ kind: 'ambiguous', names: holding.map(",
            1,
        )
    } else {
        contents
    }
}

#[test]
fn each_exit_without_an_ordinary_branch_takes_its_external_branch() {
    let cases = json!([{"name": "arranges"}]);
    let lanes = both("arranges", unchanged, &cases);
    for (language, lane) in &lanes {
        let found = &lane.results["arranges"];
        assert!(
            found.get("failure").is_none(),
            "{language}: {found}\n{}",
            lane.log
        );
        let ambiguous = strings(&found["ambiguous"]);
        for exit in [NO_MOVE, OVERLAP] {
            assert!(
                !ambiguous.contains(&exit),
                "{language}: `{exit}` was redrawn although an external branch was eligible\n{found}"
            );
        }
        for branch in &["bounced", "throttled"] {
            assert!(
                found["external"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|reach| reach["outcome"].as_str().unwrap().ends_with(branch)
                        && reach["reach"] == "reached"),
                "{language}: `{branch}` was not reached\n{found}"
            );
        }
    }
    assert_eq!(
        lanes[0].1.results["arranges"], lanes[1].1.results["arranges"],
        "typescript and go explore the same seed differently"
    );
}

fn mutant_is_caught(label: &str, mutate: Mutation, exit: &str) {
    let cases = json!([{"name": label}]);
    for (language, lane) in both(label, mutate, &cases) {
        let found = &lane.results[label];
        assert!(
            strings(&found["ambiguous"]).contains(&exit),
            "{language}: the mutant that reverts `{exit}` is not caught by \
             each_exit_without_an_ordinary_branch_takes_its_external_branch\n{found}"
        );
    }
}

#[test]
fn reverting_the_no_move_state_exit_is_caught() {
    mutant_is_caught("mutant-no-move", revert_no_move, NO_MOVE);
}

#[test]
fn reverting_the_overlapping_guards_exit_is_caught() {
    mutant_is_caught("mutant-overlap", revert_overlap, OVERLAP);
}

#[test]
fn a_draw_at_an_exit_whose_external_branch_cannot_be_arranged_is_reported_ambiguous() {
    let cases = json!([{"name": "cannot-arrange", "mode": "cannot-arrange"}]);
    for (language, lane) in both("cannot-arrange", unchanged, &cases) {
        let found = &lane.results["cannot-arrange"];
        assert!(found.get("failure").is_none(), "{language}: {found}");
        let ambiguous = strings(&found["ambiguous"]);
        let missing: Vec<&str> = [NO_MOVE, OVERLAP]
            .into_iter()
            .filter(|exit| !ambiguous.contains(exit))
            .collect();
        assert!(
            missing.is_empty(),
            "{language}: draws the specification does not decide, redrawn rather than executed, \
             are missing from `ambiguous`: {missing:?}\n{found}"
        );
    }
}
