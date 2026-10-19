//! The generated Go and TypeScript runtimes expand a suite/52 that states its event catalog once
//! (beyond10x/ess#570) into exactly the scenarios the Rust writer held.
//!
//! The suite is the beyond10x/ess#550 model with two further declared events, so it carries every
//! compact spelling: compact one-time traces, absent-event runs and an unpublished-event list. Each
//! runtime admits the compact document and hands back the document it admitted; that must be the
//! complete document Rust writes for the same suite under suite/50, scenario by scenario.
use std::fmt::Write as _;
mod support_scratch;

use std::process::Command;

use ess_compiler::{resolve::compile, source::SourceMap};
use ess_conformance::ConformanceSuite;
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use serde_json::Value;

const WITH_CALLER: &str = include_str!("fixtures/one-time-caller/with-caller.yaml");

fn suite() -> ConformanceSuite {
    let mut text = WITH_CALLER.to_owned();
    for index in 1..3 {
        write!(
            text,
            "  - name: keys.api.Audited{index}\n    fields:\n      - {{name: key_id, type: Uuid}}\n"
        )
        .unwrap();
    }
    let spec = Specification::assemble([(
        Source::new("system.yaml"),
        RawSpecFile::parse(&text).unwrap(),
    )])
    .unwrap();
    let ir = compile(&spec, &SourceMap::new()).unwrap();
    let mut suite = ess_conformance::synthesize(&ir).suite;
    suite.select_fresh_format_for(&ir);
    suite
}

/// A document with its version removed and every scenario's `source` in one order, since a
/// runtime appends the catalog events it puts back where Rust keeps a sorted set.
fn normalized(mut document: Value) -> Value {
    document["provenance"]
        .as_object_mut()
        .unwrap()
        .remove("suite_version");
    for scenario in document["scenarios"].as_object_mut().unwrap().values_mut() {
        let source = scenario["source"].as_array_mut().unwrap();
        source.sort_by_key(ToString::to_string);
    }
    document
}

/// The compact document, and the complete one Rust writes for the same suite.
fn documents() -> (String, Value) {
    let suite = suite();
    let compact = suite.to_canonical_json().unwrap();
    for spelling in [
        "\"events\"",
        "\"absent_events\"",
        "\"unpublished_events\"",
        "\"within_ms\"",
    ] {
        assert!(compact.contains(spelling), "the fixture carries {spelling}");
    }
    let mut complete = suite;
    complete.provenance.suite_version = "ess-conformance/50".parse().unwrap();
    let complete = serde_json::from_str(&complete.to_canonical_json().unwrap()).unwrap();
    (compact, normalized(complete))
}

fn scratch(label: &str) -> support_scratch::Scratch {
    let directory = support_scratch::Scratch::adopt(std::env::temp_dir().join(format!(
        "ess-compact-{label}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    )));
    std::fs::create_dir_all(&*directory).unwrap();
    directory
}

const GO_TEST: &str = r#"package essconform

import (
	"encoding/json"
	"os"
	"testing"
)

func TestCompactExpansion(t *testing.T) {
	raw, err := os.ReadFile(os.Getenv("COMPACT_DOCUMENT"))
	if err != nil {
		t.Fatal(err)
	}
	suite, err := admitSuite(string(raw))
	if err != nil {
		t.Fatal(err)
	}
	if len(suite.Scenarios) == 0 {
		t.Fatal("no scenario decoded")
	}
	for id, scenario := range suite.Scenarios {
		written := suite.document["scenarios"].(map[string]any)[id].(map[string]any)["steps"].([]any)
		if len(scenario.Steps) != len(written) {
			t.Fatalf("%s decodes %d steps of %d", id, len(scenario.Steps), len(written))
		}
	}
	out, err := json.Marshal(suite.document)
	if err != nil {
		t.Fatal(err)
	}
	if err := os.WriteFile(os.Getenv("COMPACT_OUT"), out, 0o644); err != nil {
		t.Fatal(err)
	}
}
"#;

#[test]
fn go_expands_the_compact_suite_into_the_scenarios_rust_held() {
    let (compact, complete) = documents();
    let suite = ConformanceSuite::from_json(&compact).unwrap();
    let directory = scratch("go");
    for artifact in ess_conformance::go::emit(&suite).unwrap() {
        let path = directory.join(artifact.path);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, artifact.contents).unwrap();
    }
    std::fs::write(
        directory.join("go.mod"),
        "module example.invalid/compact\n\ngo 1.21\n",
    )
    .unwrap();
    std::fs::write(
        directory.join("essconform").join("compact_test.go"),
        GO_TEST,
    )
    .unwrap();
    let document = directory.join("compact.json");
    let out = directory.join("expanded.json");
    std::fs::write(&document, &compact).unwrap();
    let output = Command::new("go")
        .args([
            "test",
            "./essconform",
            "-run",
            "^TestCompactExpansion$",
            "-count=1",
        ])
        .env("GOWORK", "off")
        .env("GOMAXPROCS", "2")
        .env("COMPACT_DOCUMENT", &document)
        .env("COMPACT_OUT", &out)
        .current_dir(&*directory)
        .output()
        .expect("required Go toolchain executes");
    assert!(
        output.status.success(),
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let expanded: Value = serde_json::from_str(&std::fs::read_to_string(out).unwrap()).unwrap();
    assert_eq!(normalized(expanded), complete);
}

#[test]
fn typescript_expands_the_compact_suite_into_the_scenarios_rust_held() {
    let (compact, complete) = documents();
    let suite = ConformanceSuite::from_json(&compact).unwrap();
    let directory = scratch("ts");
    for artifact in ess_conformance::ts::emit(&suite).unwrap() {
        let path = directory.join(artifact.path);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, artifact.contents).unwrap();
    }
    let package = directory.join(ess_conformance::ts::PACKAGE);
    std::fs::write(
        package.join("compact.tsconfig.json"),
        r#"{"extends":"./tsconfig.json","compilerOptions":{"types":[],"noCheck":true}}"#,
    )
    .unwrap();
    let compiled = Command::new("tsc")
        .args(["--project", "compact.tsconfig.json"])
        .current_dir(&package)
        .output()
        .expect("TypeScript compiler is required");
    assert!(
        compiled.status.success(),
        "{}{}",
        String::from_utf8_lossy(&compiled.stdout),
        String::from_utf8_lossy(&compiled.stderr)
    );
    std::fs::write(
        package.join("expand.mjs"),
        "import { readFileSync, writeFileSync } from 'node:fs';\n\
         import { admitSuite, goMarshal } from './dist/runtime.js';\n\
         const suite = admitSuite(readFileSync(process.argv[2], 'utf8'));\n\
         if (Object.keys(suite.scenarios).length === 0) throw new Error('no scenario decoded');\n\
         for (const [id, scenario] of Object.entries(suite.scenarios))\n\
           if (scenario.steps.length !== suite.document.scenarios[id].steps.length) throw new Error(id);\n\
         writeFileSync(process.argv[3], goMarshal(suite.document, false));\n",
    )
    .unwrap();
    let document = directory.join("compact.json");
    let out = directory.join("expanded.json");
    std::fs::write(&document, &compact).unwrap();
    let run = Command::new("node")
        .arg("expand.mjs")
        .arg(&document)
        .arg(&out)
        .current_dir(&package)
        .output()
        .expect("node runs");
    assert!(
        run.status.success(),
        "{}{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    let expanded: Value = serde_json::from_str(&std::fs::read_to_string(out).unwrap()).unwrap();
    assert_eq!(normalized(expanded), complete);
}
