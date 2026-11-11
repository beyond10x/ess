//! A struct with a shorthand (`ess/27`) that holds a `Binary64` field realizes on every target
//! (story:a-shorthand-struct-binary64-location-resolves).
//!
//! The Binary64 locations the compiler owns point into the published `oneOf [value, object]`. That
//! the generated Rust decodes `"x"` and `{"label": "x", "value": 1.5}` is held by the Binary64
//! wire corpus `schema-contract`'s `binary64_structural.rs` builds once. `tracker.ops.Plain`, a
//! struct without a shorthand, keeps the bytes the code before the change wrote: the fixtures under
//! `fixtures/shorthand-binary64/plain` are that code's output, with its generator version.

use ess_cli::TemporaryDirectory;
use serde_json::Value;
use std::{
    fs,
    path::Path,
    process::{Command, Output},
};

const SPEC: &str = include_str!("fixtures/shorthand-binary64/spec.yaml");

/// The generator version the `plain` fixtures were written by.
const FIXTURE_VERSION: &str = "0.93.0";

struct Fixture(TemporaryDirectory);
impl Fixture {
    fn new() -> Self {
        let path = TemporaryDirectory::create("ess-shorthand-binary64").unwrap();
        fs::create_dir_all(path.join("model")).unwrap();
        fs::write(path.join("model/system.yaml"), SPEC).unwrap();
        Self(path)
    }
    fn generate(&self, root: &str, target: &str) -> Output {
        let mut command = Command::new(env!("CARGO_BIN_EXE_ess"));
        command
            .current_dir(&self.0)
            .args(["generate", "types", "--path", "model", "--root", root])
            .args(["--target", target, "--out", target]);
        match target {
            "rust" => command.args(["--package", "demo"]),
            "go" => command.args(["--package", "demo", "--module", "example.com/demo"]),
            _ => &mut command,
        };
        command.output().unwrap()
    }
    fn read(&self, path: &str) -> String {
        fs::read_to_string(self.0.join(path)).unwrap()
    }
}

#[test]
fn a_shorthand_struct_with_a_binary64_field_realizes_on_every_target() {
    let fixture = Fixture::new();
    for root in ["tracker.ops.Reading", "tracker.ops.Gauge"] {
        for target in ["rust", "go", "typescript"] {
            let output = fixture.generate(root, target);
            if (root, target) == ("tracker.ops.Gauge", "rust") {
                // The Rust target refuses every struct written as a number alone
                // (`rust_shorthand_number`); the Binary64 location is not what refuses it.
                let stderr = String::from_utf8_lossy(&output.stderr);
                assert!(!output.status.success(), "{root} {target}: {output:?}");
                assert!(
                    stderr.contains("/$defs/tracker.ops.Gauge: rust_shorthand_number"),
                    "{root} {target}: {output:?}"
                );
                assert!(!stderr.contains("model_binary64_layout"), "{stderr}");
                continue;
            }
            assert!(output.status.success(), "{root} {target}: {output:?}");
            let types = fixture.read(&format!(
                "{target}/types.{}",
                match target {
                    "rust" => "rs",
                    "go" => "go",
                    _ => "ts",
                }
            ));
            let name = format!("TrackerOps{}", root.rsplit('.').next().unwrap());
            assert!(types.contains(&name), "{root} {target}: {types}");
        }
    }
}

fn binary64_pointers(report: &str) -> Vec<String> {
    let report: Value = serde_json::from_str(report).unwrap();
    let mut found: Vec<String> = report["obligations"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|item| item["rule"] == "model_binary64")
        .map(|item| item["pointer"].as_str().unwrap().to_owned())
        .collect();
    found.sort();
    found
}

#[test]
fn the_types_report_names_binary64_inside_the_published_one_of() {
    let fixture = Fixture::new();
    for (root, expected) in [
        (
            "tracker.ops.Reading",
            vec!["/$defs/tracker.ops.Reading/oneOf/1/properties/value"],
        ),
        (
            "tracker.ops.Gauge",
            vec![
                "/$defs/tracker.ops.Gauge/oneOf/0",
                "/$defs/tracker.ops.Gauge/oneOf/1/properties/level",
            ],
        ),
    ] {
        let output = fixture.generate(root, "typescript");
        assert!(output.status.success(), "{root}: {output:?}");
        let report = fixture.read("typescript/types-report.json");
        assert_eq!(binary64_pointers(&report), expected, "{root}: {report}");
        let schema: Value = serde_json::from_str(&fixture.read("typescript/source.schema.json"))
            .expect("the published schema");
        for at in expected {
            assert_eq!(
                schema.pointer(at).map(|node| &node["type"]),
                Some(&Value::from("number")),
                "{at}"
            );
        }
    }
}

/// The generated text with the generator version the fixtures were written by in place of this
/// build's: the version is the only byte that moves with a release.
fn at_fixture_version(text: &str) -> String {
    let current = env!("CARGO_PKG_VERSION");
    text.replacen(
        &format!("ESS {current};"),
        &format!("ESS {FIXTURE_VERSION};"),
        1,
    )
    .replacen(
        &format!("\"generator_version\": \"{current}\""),
        &format!("\"generator_version\": \"{FIXTURE_VERSION}\""),
        1,
    )
}

#[test]
fn a_struct_without_a_shorthand_keeps_its_bytes() {
    let fixture = Fixture::new();
    let expected =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/shorthand-binary64/plain");
    let mut compared = 0;
    for (target, manifest) in [
        ("rust", Some(("Cargo.toml", "Cargo.toml.fixture"))),
        ("go", Some(("go.mod", "go.mod.fixture"))),
        ("typescript", None),
    ] {
        let output = fixture.generate("tracker.ops.Plain", target);
        assert!(output.status.success(), "{target}: {output:?}");
        let directory = match target {
            "typescript" => "ts",
            other => other,
        };
        let mut files = vec![
            (
                format!("types.{}", directory.replace("rust", "rs")),
                format!("types.{}", directory.replace("rust", "rs")),
            ),
            (
                "types-report.json".to_owned(),
                "types-report.json".to_owned(),
            ),
            (
                "source.schema.json".to_owned(),
                "source.schema.json".to_owned(),
            ),
        ];
        if let Some((written, kept)) = manifest {
            files.push((written.to_owned(), kept.to_owned()));
        }
        for (written, kept) in files {
            let bytes = fs::read_to_string(expected.join(directory).join(&kept)).unwrap();
            let actual = at_fixture_version(&fixture.read(&format!("{target}/{written}")));
            assert_eq!(actual, bytes, "{target}/{written}");
            compared += 1;
        }
    }
    assert_eq!(compared, 11);
}
