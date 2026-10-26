//! Model and bundle identity are distinct; root failures never publish partial libraries; `--check`
//! compares a generated library with the files the same command would write (beyond10x/ess#654).

use ess_cli::TemporaryDirectory;
use serde_json::Value;
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
    time::SystemTime,
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

/// Every file under `root`, with its bytes and modification time (as `generate_check.rs` takes it).
fn snapshot(root: &Path) -> BTreeMap<PathBuf, (Vec<u8>, SystemTime)> {
    fn visit(root: &Path, dir: &Path, out: &mut BTreeMap<PathBuf, (Vec<u8>, SystemTime)>) {
        for entry in fs::read_dir(dir).expect("readable directory") {
            let path = entry.expect("directory entry").path();
            let meta = fs::symlink_metadata(&path).expect("metadata");
            if meta.is_dir() {
                visit(root, &path, out);
            } else {
                out.insert(
                    path.strip_prefix(root).expect("under root").to_path_buf(),
                    (
                        fs::read(&path).expect("readable file"),
                        meta.modified().expect("mtime"),
                    ),
                );
            }
        }
    }
    let mut files = BTreeMap::new();
    visit(root, root, &mut files);
    files
}

/// A Rust library of the fixture's record, the arguments that generate it into `out`.
fn rust_library(out: &str) -> Vec<&str> {
    vec![
        "--root",
        "sample.data.Record",
        "--target",
        "rust",
        "--package",
        "model_types",
        "--out",
        out,
    ]
}

fn checked<'a>(args: &[&'a str]) -> Vec<&'a str> {
    let mut args = args.to_vec();
    args.push("--check");
    args
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

/// The one `--check` line naming `file`.
fn drift_line<'a>(err: &'a str, file: &str) -> &'a str {
    err.lines()
        .find(|line| line.contains(file))
        .unwrap_or_else(|| panic!("stderr names {file}: {err}"))
}

#[test]
fn generate_types_check_passes_on_fresh_output_and_writes_nothing() {
    let fixture = Fixture::new();
    let args = rust_library("rust");
    assert!(fixture.run(&args).status.success());
    let before = snapshot(&fixture.0.join("rust"));
    let check = fixture.run(&checked(&args));
    assert_eq!(
        check.status.code(),
        Some(0),
        "current output passes: {check:?}"
    );
    assert!(
        !stderr(&check).contains("ess generate types"),
        "no drift line on current output: {}",
        stderr(&check)
    );
    assert_eq!(
        snapshot(&fixture.0.join("rust")),
        before,
        "--check changed no file's bytes or modification time"
    );
}

#[test]
fn generate_types_check_names_a_drifted_file() {
    let fixture = Fixture::new();
    let args = rust_library("rust");
    assert!(fixture.run(&args).status.success());
    fs::write(fixture.0.join("rust/types.rs"), "edited by hand\n").unwrap();
    let before = snapshot(&fixture.0.join("rust"));
    let check = fixture.run(&checked(&args));
    assert_eq!(check.status.code(), Some(1), "drift exits 1: {check:?}");
    let err = stderr(&check);
    let line = drift_line(&err, "types.rs");
    assert!(
        line.contains("ess generate types"),
        "the remedy names the command: {line}"
    );
    assert_eq!(
        err.lines().filter(|line| line.contains("types.rs")).count(),
        1,
        "one line for the drifted file: {err}"
    );
    assert_eq!(
        snapshot(&fixture.0.join("rust")),
        before,
        "--check wrote nothing"
    );
}

#[test]
fn generate_types_check_names_a_missing_file_and_one_no_longer_generated() {
    let fixture = Fixture::new();
    let args = rust_library("rust");
    assert!(fixture.run(&args).status.success());
    fs::remove_file(fixture.0.join("rust/types-report.json")).unwrap();
    let before = snapshot(&fixture.0.join("rust"));
    let check = fixture.run(&checked(&args));
    assert_eq!(check.status.code(), Some(1), "drift exits 1: {check:?}");
    let err = stderr(&check);
    let line = drift_line(&err, "types-report.json");
    assert!(
        line.contains("is missing") && line.contains("regenerate it with `ess generate types`"),
        "{line}"
    );
    assert_eq!(
        snapshot(&fixture.0.join("rust")),
        before,
        "--check wrote nothing"
    );

    let again = rust_library("mixed");
    assert!(fixture.run(&again).status.success());
    let before = snapshot(&fixture.0.join("mixed"));
    let check = fixture.run(&[
        "--root",
        "sample.data.Record",
        "--target",
        "typescript",
        "--out",
        "mixed",
        "--check",
    ]);
    assert_eq!(check.status.code(), Some(1), "drift exits 1: {check:?}");
    let err = stderr(&check);
    let line = drift_line(&err, "Cargo.toml");
    assert!(
        line.contains("is recorded in .ess-output but no longer generated")
            && line.contains("ess generate types"),
        "{line}"
    );
    assert_eq!(
        snapshot(&fixture.0.join("mixed")),
        before,
        "--check wrote nothing"
    );
}

#[test]
fn generate_types_check_passes_on_a_copied_output_root() {
    fn copy(from: &Path, to: &Path) {
        fs::create_dir_all(to).unwrap();
        for entry in fs::read_dir(from).unwrap() {
            let entry = entry.unwrap();
            let target = to.join(entry.file_name());
            if entry.file_type().unwrap().is_dir() {
                copy(&entry.path(), &target);
            } else {
                fs::copy(entry.path(), &target).unwrap();
            }
        }
    }
    let fixture = Fixture::new();
    assert!(fixture.run(&rust_library("original")).status.success());
    copy(&fixture.0.join("original"), &fixture.0.join("copied"));
    fs::write(fixture.0.join("copied/NOTES.md"), "kept by hand\n").unwrap();
    let check = fixture.run(&checked(&rust_library("copied")));
    assert_eq!(
        check.status.code(),
        Some(0),
        "a copied root is current: {check:?}"
    );
    assert!(
        !stderr(&check).contains("NOTES.md"),
        "an unrecorded extra file is not drift: {}",
        stderr(&check)
    );
}

#[test]
fn generate_types_check_requires_out() {
    let fixture = Fixture::new();
    let check = fixture.run(&[
        "--root",
        "sample.data.Record",
        "--target",
        "typescript",
        "--check",
    ]);
    assert_eq!(
        check.status.code(),
        Some(2),
        "the argument parser refuses --check without --out: {check:?}"
    );
    let err = stderr(&check);
    assert!(
        err.contains("required arguments were not provided") && err.contains("--out"),
        "the refusal names --out: {err}"
    );
}

/// Adversary pass 1 (regression guard): a library generated before the model moved is drift
/// through the regeneration route, not the re-enroll one, and the line names this command.
#[test]
fn adv_g_types_check_after_a_model_change_names_regeneration_by_this_command() {
    let fixture = Fixture::new();
    let args = rust_library("rust");
    assert!(fixture.run(&args).status.success());
    let model = fixture.0.join("model/system.yaml");
    let text = fs::read_to_string(&model).unwrap();
    let moved = text.replacen(
        "      - name: score\n",
        "      - name: added\n        type: String\n      - name: score\n",
        1,
    );
    assert_ne!(moved, text, "the fixture still declares `score`");
    fs::write(&model, moved).unwrap();
    let before = snapshot(&fixture.0.join("rust"));
    let check = fixture.run(&checked(&args));
    assert_eq!(check.status.code(), Some(1), "drift exits 1: {check:?}");
    let err = stderr(&check);
    let line = drift_line(&err, "types.rs");
    assert!(
        line.contains("regenerate it with `ess generate types`"),
        "the regeneration remedy names this command: {line}"
    );
    assert!(
        !line.contains("move it aside"),
        "an unedited file is not re-enrolled: {line}"
    );
    assert_eq!(
        snapshot(&fixture.0.join("rust")),
        before,
        "--check wrote nothing"
    );
}

/// Adversary pass 1 (regression guard): `--check` against an `--out` nothing was generated into
/// names the files as missing and leaves no directory behind.
#[test]
fn adv_g_types_check_on_an_absent_out_creates_nothing() {
    let fixture = Fixture::new();
    let check = fixture.run(&checked(&rust_library("never")));
    assert_eq!(
        check.status.code(),
        Some(1),
        "absent output is drift: {check:?}"
    );
    let err = stderr(&check);
    assert!(drift_line(&err, "types.rs").contains("is missing"), "{err}");
    assert!(
        !fixture.0.join("never").exists(),
        "--check created the output directory"
    );
}
