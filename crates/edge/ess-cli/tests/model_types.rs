//! Model and bundle identity are distinct; root failures never publish partial libraries.

use ess_cli::TemporaryDirectory;
use serde_json::Value;
use std::{
    fs,
    process::{Command, Output},
};

struct Fixture(TemporaryDirectory);
impl Fixture {
    fn new() -> Self {
        let path = TemporaryDirectory::create("ess-model-types").unwrap();
        fs::create_dir_all(path.join("model")).unwrap();
        fs::write(
            path.join("model/system.yaml"),
            include_str!("../../../generate/ess-gen/tests/fixtures/model-types.yaml"),
        )
        .unwrap();
        Self(path)
    }
    fn run(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_ess"))
            .current_dir(&self.0)
            .args(["generate", "types", "--path", "model"])
            .args(args)
            .output()
            .unwrap()
    }
}
#[test]
fn short_names_reach_the_generate_types_cli_and_the_default_stays_qualified() {
    let fixture = Fixture::new();
    for (directory, names, expected) in [
        (
            "qualified",
            &[][..],
            [
                "SampleDataChoice",
                "SampleDataId",
                "SampleDataMode",
                "SampleDataRecord",
            ],
        ),
        (
            "short",
            &["--names", "short"][..],
            ["Choice", "Id", "Mode", "Record"],
        ),
    ] {
        let mut args = vec![
            "--root",
            "sample.data.Record",
            "--target",
            "typescript",
            "--out",
            directory,
        ];
        args.extend_from_slice(names);
        let output = fixture.run(&args);
        assert!(output.status.success(), "{output:?}");
        let report: Value = serde_json::from_slice(
            &fs::read(fixture.0.join(directory).join("types-report.json")).unwrap(),
        )
        .unwrap();
        let mut declarations: Vec<&str> = report["declarations"]
            .as_object()
            .unwrap()
            .values()
            .map(|value| value.as_str().unwrap())
            .collect();
        declarations.sort_unstable();
        assert_eq!(declarations, expected, "{report}");
    }
}

#[test]
fn each_target_retains_the_same_model_selection_and_distinct_input_provenance() {
    let fixture = Fixture::new();
    for (target, extension, native) in [
        ("typescript", "ts", vec![]),
        ("rust", "rs", vec!["--package", "model_types"]),
        (
            "go",
            "go",
            vec![
                "--package",
                "modeltypes",
                "--module",
                "example.invalid/modeltypes",
            ],
        ),
    ] {
        let mut args = vec![
            "--root",
            "sample.data.Record",
            "--target",
            target,
            "--out",
            target,
        ];
        args.extend(native);
        let output = fixture.run(&args);
        assert!(output.status.success(), "{output:?}");
        let report: Value = serde_json::from_slice(
            &fs::read(fixture.0.join(target).join("types-report.json")).unwrap(),
        )
        .unwrap();
        let source: Value = serde_json::from_slice(
            &fs::read(fixture.0.join(target).join("source.schema.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(report["format"], "ess-types-report/3");
        assert_eq!(report["input"]["kind"], "model");
        assert_eq!(report["input"]["system"], "sample");
        assert_eq!(
            report["source_digest"],
            source["x-ess-provenance"]["source_digest"]
        );
        assert_eq!(report["declarations"].as_object().unwrap().len(), 4);
        assert!(source["$defs"].get("sample.data.OtherId").is_none());
        assert!(report.get("bundle_digest").is_none());
        assert!(!fixture.0.join(target).join("source.bundle.json").exists());
        let bytes = fs::read(fixture.0.join(target).join(format!("types.{extension}"))).unwrap();
        assert!(fixture.run(&args).status.success());
        assert_eq!(
            bytes,
            fs::read(fixture.0.join(target).join(format!("types.{extension}"))).unwrap()
        );
        for finding in report["annotations"]
            .as_array()
            .unwrap()
            .iter()
            .chain(report["obligations"].as_array().unwrap())
        {
            let pointer = finding["pointer"].as_str().unwrap();
            assert!(
                pointer == "/" || source.pointer(pointer).is_some(),
                "{finding}"
            );
        }
    }
}

#[test]
fn root_selection_and_output_refusals_preserve_existing_files() {
    let fixture = Fixture::new();
    for selection in [
        vec![],
        vec!["--root", "missing.Type"],
        vec!["--all-types", "--root", "sample.data.Id"],
    ] {
        let mut args = vec!["--target", "typescript", "--out", "out"];
        args.extend(selection);
        assert!(!fixture.run(&args).status.success());
        assert!(!fixture.0.join("out").exists());
    }
    let result = fixture.run(&[
        "--all-types",
        "--target",
        "typescript",
        "--out",
        "model/generated",
    ]);
    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("specification input"));
    assert!(!fixture.0.join("model/generated").exists());
    fs::create_dir_all(fixture.0.join("out/types-report.json")).unwrap();
    fs::write(fixture.0.join("out/types.ts"), "sentinel").unwrap();
    assert!(!fixture
        .run(&["--all-types", "--target", "typescript", "--out", "out"])
        .status
        .success());
    assert_eq!(
        fs::read_to_string(fixture.0.join("out/types.ts")).unwrap(),
        "sentinel"
    );
    assert!(!fixture.0.join("out/source.schema.json").exists());
}

#[test]
fn all_type_binary64_libraries_publish_finite_codecs_with_atomic_preflight() {
    let fixture = Fixture::new();
    fs::write(fixture.0.join("model/system.yaml"), "format: ess/2\nsystem: sample\nversion: v1\ndomains: [sample.float]\ndomain: sample.float\ntypes:\n  - {name: sample.float.Scalar, kind: newtype, of: Binary64}\n  - name: sample.float.Record\n    kind: struct\n    fields:\n      - {name: number, type: sample.float.Scalar}\n      - {name: values, type: 'Map<String, Binary64>'}\n").unwrap();
    // This CLI lane checks publication and needs no ambient Go or Rust compiler.
    for (target, extension, native) in [
        ("rust", "rs", vec!["--package", "float_types"]),
        (
            "go",
            "go",
            vec![
                "--package",
                "float_types",
                "--module",
                "example.invalid/floattypes",
            ],
        ),
    ] {
        let mut args = vec!["--all-types", "--target", target, "--out", target];
        args.extend(native);
        let output = fixture.run(&args);
        assert!(output.status.success(), "{output:?}");
        let directory = fixture.0.join(target);
        let report: Value =
            serde_json::from_slice(&fs::read(directory.join("types-report.json")).unwrap())
                .unwrap();
        assert_eq!(report["format"], "ess-types-report/3");
        assert_eq!(report["roots"].as_array().unwrap().len(), 2);
        assert!(!report["obligations"]
            .as_array()
            .unwrap()
            .iter()
            .any(|e| e["rule"] == "model_binary64"));
        let source = directory.join(format!("types.{extension}"));
        assert!(fs::read_to_string(&source).unwrap().contains("EssBinary64"));
        fs::remove_file(directory.join("types-report.json")).unwrap();
        fs::create_dir(directory.join("types-report.json")).unwrap();
        fs::write(&source, "sentinel").unwrap();
        assert!(!fixture.run(&args).status.success());
        assert_eq!(fs::read_to_string(&source).unwrap(), "sentinel");
    }
}
