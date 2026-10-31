//! The generated Go and TypeScript runtimes admit an entity fixture from `ess-conformance/60` and
//! pass its entity and state to the provider
//! (story:a-fixture-may-be-a-row-the-deployment-already-holds).
//!
//! Each runtime runs the synthesized suite of `fixtures/provided-related-row.yaml` against a
//! provider standing in for the deployment, which checks the entity, the state and the type it is
//! handed and answers the row's identity; both branches of the related guard pass. Each also admits
//! an entity fixture under /60, refuses it under /58, and refuses one naming its entity without its
//! state.
mod support_scratch;

use ess_compiler::{resolve::compile, source::SourceMap};
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source,
};

const LINKS: &str = include_str!("fixtures/provided-related-row.yaml");

fn suite() -> ess_conformance::ConformanceSuite {
    let raw = RawSpecFile::parse(LINKS).expect("the fixture parses");
    let spec = Specification::assemble([(Source::new("links.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    let ir = compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"));
    let synthesis = ess_conformance::synthesize::synthesize(&ir);
    assert!(synthesis.refusals.is_empty(), "{:#?}", synthesis.refusals);
    assert_eq!(
        synthesis.suite.provenance.suite_version.to_string(),
        "ess-conformance/60"
    );
    synthesis.suite
}

/// The emitted Go module and TypeScript package with the provider cases beside them; the
/// TypeScript package compiled. Returns the TypeScript package directory.
fn packages(root: &std::path::Path) -> std::path::PathBuf {
    let suite = suite();
    std::fs::create_dir_all(root).unwrap();
    for artifact in ess_conformance::go::emit(&suite).unwrap_or_else(|error| panic!("{error}")) {
        let path = root.join(artifact.path);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, artifact.contents).unwrap();
    }
    std::fs::write(
        root.join("go.mod"),
        "module example.invalid/provided\n\ngo 1.24\n",
    )
    .unwrap();
    std::fs::write(
        root.join("essconform/provided_test.go"),
        include_str!("fixtures/provided-rows-runtime.go"),
    )
    .unwrap();
    for artifact in ess_conformance::ts::emit(&suite).unwrap_or_else(|error| panic!("{error}")) {
        let path = root.join("typescript").join(artifact.path);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, artifact.contents).unwrap();
    }
    let ts = root.join("typescript/essconform");
    std::fs::write(
        ts.join("provided.mjs"),
        include_str!("fixtures/provided-rows-runtime.mjs"),
    )
    .unwrap();
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
fn both_runtimes_pass_the_entity_and_state_to_the_provider() {
    let root =
        support_scratch::Scratch::new(format!("ess-provided-rows-runtimes-{}", std::process::id()));
    let ts = packages(&root);
    for (tool, args, directory) in [
        (
            "go",
            vec!["test", "./essconform", "-count=1", "-v"],
            root.path(),
        ),
        ("node", vec!["--test", "provided.mjs"], ts.as_path()),
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
            log.contains("provided linking.links.App Active"),
            "{tool}: the provider was handed the entity and the state: {log}"
        );
        assert!(
            log.contains("entity fixture admission checked"),
            "{tool}: {log}"
        );
    }
}
