//! Adversary pass 1 over wave D325: the Go and TypeScript runtimes read an entity fixture
//! (`ess-conformance/60`) as the Rust reader does.
//!
//! An entity fixture naming the empty entity and the empty state is refused by the Rust reader
//! (neither is a name). Each generated runtime must refuse it too, not read it as a type-only
//! fixture or hand an empty entity to its provider.
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
    synthesis.suite
}

#[test]
#[allow(clippy::too_many_lines)]
fn an_entity_fixture_naming_the_empty_entity_is_refused_by_every_reader() {
    // The Rust reader: the synthesized suite with its entity fixture's entity and state emptied.
    let suite = suite();
    let mut document: serde_json::Value =
        serde_json::from_str(&suite.to_canonical_json().unwrap()).unwrap();
    let mut edited = false;
    for scenario in document["scenarios"].as_object_mut().unwrap().values_mut() {
        for step in scenario["steps"].as_array_mut().unwrap() {
            if step["step"] == "resolve_fixtures" {
                for field in step["fixtures"]["fields"].as_array_mut().unwrap() {
                    field["entity"] = serde_json::Value::String(String::new());
                    field["state"] = serde_json::Value::String(String::new());
                    edited = true;
                }
            }
        }
    }
    assert!(edited);
    let rust =
        ess_conformance::AdmittedSuite::from_json(&serde_json::to_string(&document).unwrap());
    assert!(rust.is_err(), "the Rust reader refuses the empty entity");

    // The generated runtimes.
    let root = support_scratch::Scratch::new(format!(
        "ess-adversary-d325-runtimes-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&root).unwrap();
    for artifact in ess_conformance::go::emit(&suite).unwrap_or_else(|error| panic!("{error}")) {
        let path = root.join(artifact.path);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, artifact.contents).unwrap();
    }
    std::fs::write(
        root.join("go.mod"),
        "module example.invalid/adversary\n\ngo 1.24\n",
    )
    .unwrap();
    std::fs::write(
        root.join("essconform/adversary_test.go"),
        include_str!("fixtures/adversary-d325-runtime.go"),
    )
    .unwrap();
    for artifact in ess_conformance::ts::emit(&suite).unwrap_or_else(|error| panic!("{error}")) {
        let path = root.join("typescript").join(artifact.path);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, artifact.contents).unwrap();
    }
    let ts = root.join("typescript/essconform");
    std::fs::write(
        ts.join("adversary.mjs"),
        include_str!("fixtures/adversary-d325-runtime.mjs"),
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
    let mut admitted = Vec::new();
    for (tool, args, directory) in [
        (
            "go",
            vec![
                "test",
                "./essconform",
                "-count=1",
                "-v",
                "-run",
                "TestAdversaryD325EmptyEntityFixture",
            ],
            root.path(),
        ),
        ("node", vec!["--test", "adversary.mjs"], ts.as_path()),
    ] {
        let output = std::process::Command::new(tool)
            .args(args)
            .env("GOWORK", "off")
            .current_dir(directory)
            .output()
            .unwrap_or_else(|error| panic!("{tool} runs: {error}"));
        let log = format!(
            "{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        if !(output.status.success() && log.contains("empty entity fixture refused")) {
            admitted.push(format!("{tool}:\n{log}"));
        }
    }
    assert!(
        admitted.is_empty(),
        "every runtime refuses an entity fixture naming the empty entity:\n{}",
        admitted.join("\n")
    );
}
