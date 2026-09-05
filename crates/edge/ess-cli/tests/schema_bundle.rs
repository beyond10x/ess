//! The schema-only CLI never converts an incomplete import into successful output.

use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

use serde_json::{json, Value};

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let root = std::env::temp_dir().join(format!(
            "ess-schema-bundle-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&root).unwrap();
        let source = json!({"openapi": "3.0.0", "info": {"title": "", "version": ""}, "paths": {},
            "components": {"schemas": {"Root": {"type": "object", "required": ["n"],
                "properties": {"n": {"type": "integer", "minimum": 1}}, "additionalProperties": false}}}});
        fs::write(root.join("source.json"), source.to_string()).unwrap();
        fs::write(root.join("instance.json"), "{\"n\":2}\n").unwrap();
        Self(root)
    }

    fn run(&self, args: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_ess"))
            .current_dir(&self.0)
            .args(["generate", "schema"])
            .args(args)
            .output()
            .unwrap()
    }

    fn import(&self) {
        let output = self.run(&[
            "import-bundle",
            "--path",
            "source.json",
            "--component",
            "Root",
            "--dialect",
            "draft-2020-12",
            "--out",
            "result.bundle.json",
        ]);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

#[test]
fn document_root_survives_reload_projection_and_each_type_target() {
    let fixture = Fixture::new();
    let source = json!({"$schema": "https://json-schema.org/draft/2020-12/schema",
        "type": "object", "required": ["settings"],
        "properties": {"settings": {"$ref": "#/$defs/Settings"}},
        "$defs": {"Settings": {"type": "string"}, "Other": {"type": "boolean"}}});
    fs::write(fixture.0.join("document.json"), source.to_string()).unwrap();
    let arguments = [
        "import-document",
        "--path",
        "document.json",
        "--root",
        "Record",
        "--definition",
        "Other",
        "--dialect",
        "draft-2020-12",
        "--out",
        "document.bundle.json",
    ];
    let result = fixture.run(&arguments);
    assert!(result.status.success(), "{result:?}");
    let before = fs::read(fixture.0.join("document.bundle.json")).unwrap();
    assert!(fixture.run(&arguments).status.success());
    assert_eq!(
        before,
        fs::read(fixture.0.join("document.bundle.json")).unwrap()
    );
    let bundle: Value = serde_json::from_slice(&before).unwrap();
    assert_eq!(bundle["format"], "ess-schema-bundle/2");
    assert_eq!(bundle["source"], source.to_string());
    assert_eq!(bundle["document_root"], "Record");
    let result = fixture.run(&[
        "project-bundle",
        "--bundle",
        "document.bundle.json",
        "--root",
        "Record",
        "--schema-id",
        "urn:example:record",
        "--out",
        "record.schema.json",
    ]);
    assert!(result.status.success(), "{result:?}");
    let schema: Value =
        serde_json::from_slice(&fs::read(fixture.0.join("record.schema.json")).unwrap()).unwrap();
    assert_eq!(
        schema["$defs"]["Record"]["properties"]["settings"]["$ref"],
        "#/$defs/Settings"
    );
    for target in ["typescript", "go", "rust"] {
        let mut args = vec![
            "types-bundle",
            "--bundle",
            "document.bundle.json",
            "--root",
            "Record",
            "--target",
            target,
            "--out",
            target,
        ];
        if target != "typescript" {
            args.extend(["--package", "record_types"]);
        }
        if target == "go" {
            args.extend(["--module", "example.invalid/recordtypes"]);
        }
        let result = fixture.run(&args);
        assert!(result.status.success(), "{result:?}");
        let report: Value = serde_json::from_slice(
            &fs::read(fixture.0.join(target).join("types-report.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(
            report["declarations"],
            json!({"Record": "Record", "Settings": "Settings"})
        );
        assert_eq!(
            fs::read(fixture.0.join(target).join("source.bundle.json")).unwrap(),
            before
        );
    }
}

#[test]
fn document_import_refusals_preserve_source_and_existing_output() {
    let fixture = Fixture::new();
    let source = json!({"type": "object", "$defs": {"Duplicate": {"type": "string"}}}).to_string();
    fs::write(fixture.0.join("document.json"), &source).unwrap();
    fs::write(fixture.0.join("sentinel.json"), "sentinel").unwrap();
    for (root, out, dialect, expected) in [
        ("Record", "sentinel.json", false, "--dialect"),
        ("Duplicate", "sentinel.json", true, "collides"),
        ("Record", "document.json", true, "must not replace"),
    ] {
        let mut args = vec![
            "import-document",
            "--path",
            "document.json",
            "--root",
            root,
            "--out",
            out,
        ];
        if dialect {
            args.extend(["--dialect", "draft-2020-12"]);
        }
        let result = fixture.run(&args);
        assert!(!result.status.success());
        assert!(
            String::from_utf8_lossy(&result.stderr).contains(expected),
            "{result:?}"
        );
        assert_eq!(
            fs::read_to_string(fixture.0.join("document.json")).unwrap(),
            source
        );
        assert_eq!(
            fs::read_to_string(fixture.0.join("sentinel.json")).unwrap(),
            "sentinel"
        );
    }
    fs::write(
        fixture.0.join("document.json"),
        r#"{"$ref":"https://example.invalid/schema"}"#,
    )
    .unwrap();
    let result = fixture.run(&[
        "import-document",
        "--path",
        "document.json",
        "--root",
        "Record",
        "--dialect",
        "draft-2020-12",
        "--out",
        "absent/result.json",
    ]);
    assert!(!result.status.success());
    assert!(!fixture.0.join("absent").exists());
}

#[test]
fn types_bundle_is_deterministic_and_never_replaces_its_input() {
    let fixture = Fixture::new();
    fixture.import();
    let arguments = [
        "types-bundle",
        "--bundle",
        "result.bundle.json",
        "--root",
        "Root",
        "--target",
        "typescript",
        "--out",
        "types",
    ];
    let result = fixture.run(&arguments);
    assert!(result.status.success(), "{result:?}");
    let files = ["types.ts", "types-report.json", "source.bundle.json"];
    let before = files.map(|name| fs::read(fixture.0.join("types").join(name)).unwrap());
    assert!(fixture.run(&arguments).status.success());
    assert_eq!(
        before,
        files.map(|name| fs::read(fixture.0.join("types").join(name)).unwrap())
    );
    let report: Value = serde_json::from_slice(&before[1]).unwrap();
    assert_eq!(report["format"], "ess-types-report/3");
    assert_eq!(report["declarations"], json!({"Root": "Root"}));
    assert!(!report["obligations"].as_array().unwrap().is_empty());
    let input = fixture.0.join("types/source.bundle.json");
    let result = fixture.run(&[
        "types-bundle",
        "--bundle",
        "types/source.bundle.json",
        "--root",
        "Root",
        "--target",
        "typescript",
        "--out",
        "types",
    ]);
    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("must not replace its source"));
    assert_eq!(fs::read(input).unwrap(), before[2]);
}

#[test]
fn type_planning_and_output_refusals_leave_no_partial_library() {
    let fixture = Fixture::new();
    fixture.import();
    let result = fixture.run(&[
        "types-bundle",
        "--bundle",
        "result.bundle.json",
        "--root",
        "Missing",
        "--target",
        "typescript",
        "--out",
        "absent",
    ]);
    assert!(!result.status.success());
    assert!(!fixture.0.join("absent").exists());
    fs::create_dir_all(fixture.0.join("collision/types-report.json")).unwrap();
    let result = fixture.run(&[
        "types-bundle",
        "--bundle",
        "result.bundle.json",
        "--root",
        "Root",
        "--target",
        "typescript",
        "--out",
        "collision",
    ]);
    assert!(!result.status.success());
    assert!(!fixture.0.join("collision/types.ts").exists());
    assert!(!fixture.0.join("collision/source.bundle.json").exists());
}

#[test]
fn native_bundle_targets_require_identity_and_emit_build_metadata() {
    let fixture = Fixture::new();
    fixture.import();
    for (target, file, manifest) in [
        ("rust", "types.rs", "Cargo.toml"),
        ("go", "types.go", "go.mod"),
    ] {
        let mut args = vec![
            "types-bundle",
            "--bundle",
            "result.bundle.json",
            "--root",
            "Root",
            "--target",
            target,
            "--out",
            target,
        ];
        assert!(!fixture.run(&args).status.success());
        assert!(!fixture.0.join(target).exists());
        args.extend(["--package", "contract_types"]);
        if target == "go" {
            assert!(!fixture.run(&args).status.success());
            assert!(!fixture.0.join(target).exists());
            args.extend(["--module", "example.invalid/contracttypes"]);
        }
        let result = fixture.run(&args);
        assert!(result.status.success(), "{result:?}");
        assert!(fixture.0.join(target).join(file).is_file());
        assert!(fixture.0.join(target).join(manifest).is_file());
        let report: Value = serde_json::from_slice(
            &fs::read(fixture.0.join(target).join("types-report.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(report["target"], target);
        assert_eq!(report["configuration"]["package"], "contract_types");
        let before = fs::read(fixture.0.join(target).join(file)).unwrap();
        assert!(fixture.run(&args).status.success());
        assert_eq!(before, fs::read(fixture.0.join(target).join(file)).unwrap());
    }
}

#[test]
fn import_reload_projection_and_instance_validation_keep_original_data() {
    let fixture = Fixture::new();
    let original = fs::read(fixture.0.join("source.json")).unwrap();
    fixture.import();
    let first = fs::read(fixture.0.join("result.bundle.json")).unwrap();
    fixture.import();
    assert_eq!(
        first,
        fs::read(fixture.0.join("result.bundle.json")).unwrap()
    );
    let output = fixture.run(&[
        "project-bundle",
        "--bundle",
        "result.bundle.json",
        "--root",
        "Root",
        "--schema-id",
        "urn:example:root",
        "--out",
        "result.schema.json",
    ]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let schema: Value =
        serde_json::from_slice(&fs::read(fixture.0.join("result.schema.json")).unwrap()).unwrap();
    assert_eq!(schema["x-ess-source"]["declared_dialect"], "3.0.0");
    let output = fixture.run(&[
        "validate-bundle",
        "--bundle",
        "result.bundle.json",
        "--root",
        "Root",
        "instance.json",
    ]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        fs::read_to_string(fixture.0.join("instance.json")).unwrap(),
        "{\"n\":2}\n"
    );
    assert_eq!(fs::read(fixture.0.join("source.json")).unwrap(), original);
    fs::write(fixture.0.join("instance.json"), "{\"n\":0}").unwrap();
    let invalid = fixture.run(&[
        "validate-bundle",
        "--bundle",
        "result.bundle.json",
        "--root",
        "Root",
        "instance.json",
    ]);
    assert!(!invalid.status.success());
    assert!(String::from_utf8_lossy(&invalid.stdout).contains("instance.json/n"));
}

#[test]
fn missing_dialect_and_incomplete_import_leave_existing_output_alone() {
    let fixture = Fixture::new();
    fs::write(fixture.0.join("result.bundle.json"), "sentinel").unwrap();
    for args in [
        vec![
            "import-bundle",
            "--path",
            "source.json",
            "--component",
            "Root",
            "--out",
            "result.bundle.json",
        ],
        vec![
            "import-bundle",
            "--path",
            "source.json",
            "--component",
            "Absent",
            "--dialect",
            "draft-2020-12",
            "--out",
            "result.bundle.json",
        ],
    ] {
        let output = fixture.run(&args);
        assert!(!output.status.success());
        assert_eq!(
            fs::read_to_string(fixture.0.join("result.bundle.json")).unwrap(),
            "sentinel"
        );
    }
}

#[test]
fn corrupted_import_cannot_be_projected_and_output_cannot_replace_source() {
    let fixture = Fixture::new();
    fixture.import();
    let mut saved: Value =
        serde_json::from_slice(&fs::read(fixture.0.join("result.bundle.json")).unwrap()).unwrap();
    saved["definitions"]["Root"] = json!(true);
    fs::write(fixture.0.join("result.bundle.json"), saved.to_string()).unwrap();
    fs::write(fixture.0.join("result.schema.json"), "sentinel").unwrap();
    let output = fixture.run(&[
        "project-bundle",
        "--bundle",
        "result.bundle.json",
        "--root",
        "Root",
        "--schema-id",
        "urn:example:root",
        "--out",
        "result.schema.json",
    ]);
    assert!(!output.status.success());
    assert_eq!(
        fs::read_to_string(fixture.0.join("result.schema.json")).unwrap(),
        "sentinel"
    );
    let original = fs::read(fixture.0.join("source.json")).unwrap();
    let output = fixture.run(&[
        "import-bundle",
        "--path",
        "source.json",
        "--component",
        "Root",
        "--dialect",
        "draft-2020-12",
        "--out",
        "source.json",
    ]);
    assert!(!output.status.success());
    assert_eq!(fs::read(fixture.0.join("source.json")).unwrap(), original);
}
