//! Model and bundle identity are distinct; root failures never publish partial libraries.

use serde_json::Value;
use std::{
    fs,
    path::PathBuf,
    process::{Command, Output},
    sync::atomic::{AtomicUsize, Ordering},
};

struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "ess-model-types-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
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
impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
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
