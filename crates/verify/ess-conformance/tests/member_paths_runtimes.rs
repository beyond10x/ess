//! The generated Go and TypeScript runtimes compare a response member path alone and refuse it
//! below `ess-conformance/62` (story:an-authored-claim-may-compare-a-nested-field).
//!
//! Each runtime runs the suite of `fixtures/nested-response-checkpoint-scenario.yaml`, whose
//! `response:` claims `checkpoint.checkpoint_id` alone, against a target whose checkpoint carries
//! other members too; the scenario passes. Each then admits the scenario's own response contract,
//! passes a return whose other members differ, fails one whose named member differs, refuses the
//! contract under /61, and refuses a path that names no member or goes through a primitive: the
//! verdicts the Rust runner gives in `tests/direct_returns.rs`.
mod support_scratch;

use ess_compiler::{resolve::compile, source::SourceMap};
use ess_conformance::authored;
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source,
};

const CHECKPOINT: &str = include_str!("fixtures/nested-response-checkpoint.yaml");
const SCENARIO: &str = include_str!("fixtures/nested-response-checkpoint-scenario.yaml");

fn suite() -> ess_conformance::ConformanceSuite {
    let raw = RawSpecFile::parse(CHECKPOINT).expect("the fixture parses");
    let spec = Specification::assemble([(Source::new("checkpoint.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    let ir = compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"));
    let authored = authored::compile(&ir, &[authored::Source::new("defer.yaml", SCENARIO)]);
    assert!(authored.refusals.is_empty(), "{:#?}", authored.refusals);
    let mut suite = ess_conformance::synthesize(&ir).suite;
    suite.scenarios = authored.scenarios;
    suite.select_fresh_format();
    assert_eq!(
        suite.provenance.suite_version.to_string(),
        "ess-conformance/62"
    );
    suite
}

/// The `response` member of the suite's one direct-response step, as the suite carries it.
fn contract(suite: &ess_conformance::ConformanceSuite) -> String {
    let document: serde_json::Value =
        serde_json::from_str(&suite.to_canonical_json().unwrap()).unwrap();
    let step = document["scenarios"]
        .as_object()
        .unwrap()
        .values()
        .flat_map(|scenario| scenario["steps"].as_array().unwrap().iter())
        .find(|step| step["step"] == "expect_direct_response")
        .expect("the scenario observes its direct response")
        .clone();
    assert_eq!(
        step["response"]["expected"],
        serde_json::json!({"checkpoint.checkpoint_id": "cp-1"})
    );
    serde_json::to_string(&step["response"]).unwrap()
}

/// The emitted Go module and TypeScript package with the cases beside them; the TypeScript
/// package compiled. Returns the TypeScript package directory.
fn packages(root: &std::path::Path) -> std::path::PathBuf {
    let suite = suite();
    let contract = contract(&suite);
    std::fs::create_dir_all(root).unwrap();
    for artifact in ess_conformance::go::emit(&suite).unwrap_or_else(|error| panic!("{error}")) {
        let path = root.join(artifact.path);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, artifact.contents).unwrap();
    }
    std::fs::write(
        root.join("go.mod"),
        "module example.invalid/paths\n\ngo 1.24\n",
    )
    .unwrap();
    std::fs::write(
        root.join("essconform/paths_test.go"),
        include_str!("fixtures/member-paths-runtime.go"),
    )
    .unwrap();
    std::fs::write(root.join("essconform/contract.json"), &contract).unwrap();
    for artifact in ess_conformance::ts::emit(&suite).unwrap_or_else(|error| panic!("{error}")) {
        let path = root.join("typescript").join(artifact.path);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, artifact.contents).unwrap();
    }
    let ts = root.join("typescript/essconform");
    std::fs::write(
        ts.join("paths.mjs"),
        include_str!("fixtures/member-paths-runtime.mjs"),
    )
    .unwrap();
    std::fs::write(ts.join("contract.json"), &contract).unwrap();
    std::fs::write(
        ts.join("runtime-test.tsconfig.json"),
        r#"{"extends":"./tsconfig.json","compilerOptions":{"types":[],"noCheck":true}}"#,
    )
    .unwrap();
    let compiled = std::process::Command::new("tsc")
        .args(["--project", "runtime-test.tsconfig.json"])
        .current_dir(&ts)
        .output()
        .expect("the TypeScript compiler runs");
    assert!(
        compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled.stdout)
    );
    ts
}

#[test]
fn both_runtimes_compare_a_response_member_path_alone() {
    let root =
        support_scratch::Scratch::new(format!("ess-member-paths-runtimes-{}", std::process::id()));
    let ts = packages(&root);
    for (tool, args, directory) in [
        (
            "go",
            vec!["test", "./essconform", "-count=1", "-v"],
            root.path(),
        ),
        ("node", vec!["--test", "paths.mjs"], ts.as_path()),
    ] {
        let output = std::process::Command::new(tool)
            .args(args)
            .env("ESS_REPORT_FORMAT", "2")
            .env("GOWORK", "off")
            .env_remove("ESS_REPORT_OUT")
            .current_dir(directory)
            .output()
            .unwrap_or_else(|error| panic!("{tool} runs: {error}"));
        let log = format!(
            "{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(output.status.success(), "{tool}: {log}");
        assert!(
            log.contains("deferred with format f-1"),
            "{tool}: the target answered the authored scenario: {log}"
        );
        assert!(
            log.contains("member path admission checked"),
            "{tool}: {log}"
        );
    }
}
