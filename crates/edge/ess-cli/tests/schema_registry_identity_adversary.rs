//! Additional actual-CLI probes for the published schema resource workflow.

use std::cell::Cell;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

use serde_json::{json, Value};

const DIALECT: &str = "https://json-schema.org/draft/2020-12/schema";

fn repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

fn fixture_value(path: &str) -> Value {
    serde_json::from_slice(
        &fs::read(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("tests/fixtures/schema-resource-identity")
                .join(path),
        )
        .unwrap(),
    )
    .unwrap()
}

fn source(kind: &str) -> (PathBuf, Value) {
    let path = repository().join(match kind {
        "invoice" => "generated/schema/commands/billing.invoice.CreateInvoice.schema.json",
        "source" => "schemas/generated/ess.schema.json",
        _ => unreachable!(),
    });
    let value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    (path, value)
}

fn envelope(id: &str, reference: &str) -> Value {
    json!({
        "$schema": DIALECT, "$id": id, "type": "object",
        "required": ["schema", "payload"], "additionalProperties": false,
        "properties": {"schema": {"const": id}, "payload": {"$ref": reference}}
    })
}

struct Case {
    root: PathBuf,
    next: Cell<usize>,
}

impl Case {
    fn new(label: &str) -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let base = std::env::var_os("ESS_SCHEMA_IDENTITY_ADVERSARY_EVIDENCE")
            .map_or_else(std::env::temp_dir, PathBuf::from);
        let root = base.join(format!(
            "schema-id-attack-{label}-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(root.join("registry")).unwrap();
        Self {
            root,
            next: Cell::new(0),
        }
    }

    fn write(&self, path: &str, bytes: impl AsRef<[u8]>) {
        let path = self.root.join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, bytes).unwrap();
    }

    fn json(&self, path: &str, value: &Value) {
        self.write(path, serde_json::to_string_pretty(value).unwrap() + "\n");
    }

    fn copied_pair(&self, kind: &str) -> (Value, Value) {
        let (original, value) = source(kind);
        let bytes = fs::read(&original).unwrap();
        let resource_id = format!("urn:example:adversary-{kind}:1");
        let selected_id = format!("urn:example:adversary-{kind}-envelope:1");
        assert!(value.get("$id").is_none());
        let mut resource = value.clone();
        resource["$id"] = json!(resource_id);
        self.json("registry/a-unrelated.schema.json", &resource);
        let selected = envelope(&selected_id, &resource_id);
        self.json("registry/z-unrelated.schema.json", &selected);
        let name = if kind == "invoice" {
            "instances/create-invoice.json"
        } else {
            "instances/ess-source.json"
        };
        let mut instance = fixture_value(name);
        instance["schema"] = json!(selected_id);
        self.json("instances/one.json", &instance);
        resource.as_object_mut().unwrap().remove("$id");
        assert_eq!(resource, value);
        assert_eq!(fs::read(original).unwrap(), bytes);
        (selected, instance)
    }

    fn run(&self, args: &[&str]) -> std::process::Output {
        let index = self.next.get();
        self.next.set(index + 1);
        let call = self.root.join(format!("calls/{index:02}"));
        fs::create_dir_all(&call).unwrap();
        for name in ["registry", "instances"] {
            let input = self.root.join(name);
            if input.exists() {
                snapshot(&input, &call.join("inputs").join(name));
            }
        }
        let started = Instant::now();
        let output = Command::new(env!("CARGO_BIN_EXE_ess"))
            .current_dir(&self.root)
            .args(args)
            .output()
            .unwrap();
        fs::write(call.join("stdout"), &output.stdout).unwrap();
        fs::write(call.join("stderr"), &output.stderr).unwrap();
        fs::write(
            call.join("command.json"),
            serde_json::to_vec_pretty(&json!({
                "binary": env!("CARGO_BIN_EXE_ess"), "cwd": self.root, "argv": args,
                "exit": output.status.code(), "elapsed_seconds": started.elapsed().as_secs_f64(),
                "inputs": "inputs", "stdout": "stdout", "stderr": "stderr"
            }))
            .unwrap(),
        )
        .unwrap();
        println!(
            "CLI receipt {} exit {:?}",
            call.display(),
            output.status.code()
        );
        output
    }

    fn validate(&self, expected: i32) -> Value {
        let output = self.run(&[
            "generate",
            "schema",
            "validate",
            "instances",
            "--schemas",
            "registry",
            "--format",
            "json",
        ]);
        assert_eq!(output.status.code(), Some(expected), "{output:?}");
        assert!(output.stderr.is_empty(), "{output:?}");
        serde_json::from_slice(&output.stdout).unwrap()
    }
}

fn snapshot(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap();
    for entry in fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let destination = to.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            snapshot(&entry.path(), &destination);
        } else {
            fs::copy(entry.path(), destination).unwrap();
        }
    }
}

fn issue(report: &Value, code: &str, pointer: Option<&str>) {
    assert_eq!(report["valid"], json!([]), "{report}");
    assert!(
        report["issues"].as_array().unwrap().iter().any(|entry| {
            entry["code"] == code
                && pointer.is_none_or(|expected| entry["instance_path"] == expected)
        }),
        "expected {code} at {pointer:?}: {report}"
    );
}

#[test]
fn an_unselected_invalid_resource_blocks_both_documented_pairs() {
    for kind in ["invoice", "source"] {
        for variant in ["unresolved", "invalid", "relative"] {
            let case = Case::new(&format!("{kind}-{variant}"));
            case.copied_pair(kind);
            let mut unused = json!({"$schema": DIALECT, "$id": "urn:example:unused:1"});
            let code = match variant {
                "unresolved" => {
                    unused["$ref"] = json!("https://example.invalid/not-supplied.json");
                    "unresolved_schema"
                }
                "invalid" => {
                    unused["type"] = json!(17);
                    "invalid_schema"
                }
                "relative" => {
                    unused["$id"] = json!("relative.json");
                    "missing_schema_id"
                }
                _ => unreachable!(),
            };
            case.json("registry/m-unused.schema.json", &unused);
            issue(&case.validate(1), code, None);
        }
    }
}

#[test]
fn envelope_definitions_cannot_shadow_either_payload_resource_root() {
    for kind in ["invoice", "source"] {
        let case = Case::new(&format!("{kind}-root"));
        let (mut selected, mut instance) = case.copied_pair(kind);
        selected["$defs"] = json!({"billing.invoice.Money": false});
        selected["definitions"] = json!({"QualifiedName": false});
        case.json("registry/z-unrelated.schema.json", &selected);
        let valid = case.validate(0);
        assert_eq!(
            valid["valid"][0]["schema"],
            "registry/z-unrelated.schema.json"
        );
        let pointer = if kind == "invoice" {
            instance["payload"]["amount"]["currency"] = json!(false);
            "/payload/amount/currency"
        } else {
            instance["payload"]["domains"][0] = json!(false);
            "/payload/domains/0"
        };
        case.json("instances/one.json", &instance);
        issue(&case.validate(1), "invalid_instance", Some(pointer));
    }
}

#[test]
fn the_selected_schema_checks_its_selector_and_nonobject_payload_as_one_instance() {
    for (label, shape, valid, invalid) in [
        (
            "text",
            json!({"type":"string", "minLength":1}),
            json!("kept"),
            json!(""),
        ),
        (
            "array",
            json!({"type":"array", "items":{"type":"integer"}, "minItems":1}),
            json!([1]),
            json!([1, "bad"]),
        ),
        ("null", json!({"type":"null"}), Value::Null, json!(false)),
        (
            "boolean",
            json!({"type":"boolean"}),
            json!(false),
            json!("false"),
        ),
    ] {
        let case = Case::new(label);
        let mut resource = shape;
        resource["$schema"] = json!(DIALECT);
        resource["$id"] = json!("urn:example:scalar:1");
        case.json("registry/value.schema.json", &resource);
        let selected = envelope("urn:example:wrapper:1", "urn:example:scalar:1");
        case.json("registry/wrapper.schema.json", &selected);
        let mut instance = json!({"schema":"urn:example:wrapper:1", "payload":valid});
        case.json("instances/one.json", &instance);
        assert_eq!(case.validate(0)["valid"].as_array().unwrap().len(), 1);
        instance["payload"] = invalid;
        case.json("instances/one.json", &instance);
        issue(&case.validate(1), "invalid_instance", None);
        let mut changed = selected;
        changed["$id"] = json!("urn:example:changed-wrapper:1");
        case.json("registry/wrapper.schema.json", &changed);
        instance["schema"] = json!("urn:example:changed-wrapper:1");
        instance["payload"] = valid;
        case.json("instances/one.json", &instance);
        issue(&case.validate(1), "invalid_instance", Some("/schema"));
    }
}

#[test]
fn decoded_duplicate_ids_are_collisions_and_filenames_supply_no_identity() {
    let case = Case::new("decoded-ids");
    case.copied_pair("invoice");
    let bytes = fs::read_to_string(case.root.join("registry/a-unrelated.schema.json")).unwrap();
    let escaped = bytes.replace(
        "urn:example:adversary-invoice:1",
        "urn:example:adversary-invoice:\\u0031",
    );
    assert_ne!(bytes, escaped);
    let left: Value = serde_json::from_str(&bytes).unwrap();
    let right: Value = serde_json::from_str(&escaped).unwrap();
    assert_eq!(left["$id"], right["$id"]);
    case.write("registry/duplicate.schema.json", escaped);
    issue(&case.validate(1), "duplicate_schema_id", Some("/$id"));

    let renamed = Case::new("renamed");
    let (_, mut instance) = renamed.copied_pair("invoice");
    let resource = fs::read(renamed.root.join("registry/a-unrelated.schema.json")).unwrap();
    renamed.write("registry/deep/not-an-id.schema.json", resource);
    // Keep one indexed resource: this case was constructed independently of the collision input.
    fs::rename(
        renamed.root.join("registry/a-unrelated.schema.json"),
        renamed.root.join("original-resource.txt"),
    )
    .unwrap();
    let valid = renamed.validate(0);
    assert_eq!(valid["valid"][0]["schema_id"], instance["schema"]);
    instance["schema"] = json!("registry/z-unrelated.schema.json");
    renamed.json("instances/one.json", &instance);
    issue(&renamed.validate(1), "unknown_schema", Some("/schema"));
}

#[test]
fn inlining_idless_generated_payloads_breaks_their_original_root_references() {
    for kind in ["invoice", "source"] {
        let case = Case::new(&format!("{kind}-inline"));
        let (_, generated) = source(kind);
        let mut selected = envelope("urn:example:inline:1", "urn:example:unused:1");
        selected["properties"]["payload"] = generated;
        case.json("registry/wrapper.schema.json", &selected);
        let name = if kind == "invoice" {
            "instances/create-invoice.json"
        } else {
            "instances/ess-source.json"
        };
        let mut instance = fixture_value(name);
        instance["schema"] = json!("urn:example:inline:1");
        case.json("instances/one.json", &instance);
        issue(&case.validate(1), "unresolved_schema", None);
    }
}

#[test]
fn registry_admission_and_selected_typescript_projection_have_distinct_boundaries() {
    let case = Case::new("typescript-boundaries");
    let widget = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/schema-contract/registry/widget.schema.json");
    case.write("registry/widget.schema.json", fs::read(widget).unwrap());
    case.json(
        "registry/unselected.schema.json",
        &json!({
            "$schema":DIALECT, "$id":"urn:example:unselected:1", "type":17
        }),
    );
    case.json(
        "instances/one.json",
        &json!({"schema":"urn:example:widget:1","name":"Ada"}),
    );
    issue(&case.validate(1), "invalid_schema", None);
    let projected = case.run(&[
        "generate",
        "schema",
        "typescript",
        "urn:example:widget:1",
        "--root",
        "Widget",
        "--schemas",
        "registry",
    ]);
    assert_eq!(projected.status.code(), Some(0), "{projected:?}");
    assert!(projected.stderr.is_empty());
    let expected = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/schema-contract/expected/widget.ts");
    assert_eq!(projected.stdout, fs::read(expected).unwrap());

    let refused = Case::new("typescript-source");
    refused.copied_pair("source");
    refused.write("sentinel.ts", b"preserved\n");
    for destination in ["sentinel.ts", "absent/output.ts"] {
        let output = refused.run(&[
            "generate",
            "schema",
            "typescript",
            "urn:example:adversary-source:1",
            "--root",
            "Source",
            "--schemas",
            "registry",
            "--out",
            destination,
            "--check",
        ]);
        assert_eq!(output.status.code(), Some(1), "{output:?}");
        assert!(
            String::from_utf8_lossy(&output.stderr).contains("unsupported"),
            "{output:?}"
        );
        assert_eq!(
            fs::read(refused.root.join("sentinel.ts")).unwrap(),
            b"preserved\n"
        );
        assert!(!refused.root.join("absent").exists());
    }
}
