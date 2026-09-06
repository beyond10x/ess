//! Actual CLI characterization of adopter resource IDs and separate selector envelopes.

use std::cell::Cell;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

use serde_json::{json, Value};

const INVOICE: &str = "urn:example:billing-create-invoice:1";
const SOURCE: &str = "urn:example:ess-source-syntax:1";
const INVOICE_ENVELOPE: &str = "urn:example:invoice-submission:1";
const SOURCE_ENVELOPE: &str = "urn:example:ess-source-submission:1";

fn repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/schema-resource-identity")
}

fn original(kind: &str) -> PathBuf {
    repository().join(match kind {
        "invoice" => "generated/schema/commands/billing.invoice.CreateInvoice.schema.json",
        "source" => "schemas/generated/ess.schema.json",
        _ => panic!("unknown fixture kind"),
    })
}

fn payload_id(kind: &str) -> &'static str {
    if kind == "invoice" {
        INVOICE
    } else {
        SOURCE
    }
}

fn envelope_id(kind: &str) -> &'static str {
    if kind == "invoice" {
        INVOICE_ENVELOPE
    } else {
        SOURCE_ENVELOPE
    }
}

fn instance(kind: &str) -> Value {
    let name = if kind == "invoice" {
        "create-invoice"
    } else {
        "ess-source"
    };
    serde_json::from_slice(&fs::read(fixtures().join(format!("instances/{name}.json"))).unwrap())
        .unwrap()
}

struct Fixture {
    root: PathBuf,
    next: Cell<usize>,
}

impl Fixture {
    fn new(label: &str) -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let base = std::env::var_os("ESS_SCHEMA_IDENTITY_EVIDENCE")
            .map_or_else(std::env::temp_dir, PathBuf::from);
        let root = base.join(format!(
            "schema-identity-{label}-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&root).unwrap();
        Self {
            root,
            next: Cell::new(0),
        }
    }

    fn write(&self, relative: &str, bytes: impl AsRef<[u8]>) {
        let path = self.root.join(relative);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, bytes).unwrap();
    }

    fn json(&self, relative: &str, value: &Value) {
        self.write(
            relative,
            serde_json::to_string_pretty(value).unwrap() + "\n",
        );
    }

    fn run(&self, args: &[&str]) -> Output {
        let mut command = Command::new(env!("CARGO_BIN_EXE_ess"));
        command.current_dir(&self.root).args(args);
        let started = Instant::now();
        let output = command.output().expect("actual ESS CLI starts");
        let index = self.next.get();
        self.next.set(index + 1);
        self.write(&format!("{index:02}.stdout"), &output.stdout);
        self.write(&format!("{index:02}.stderr"), &output.stderr);
        self.json(
            &format!("{index:02}.command.json"),
            &json!({
                "binary": env!("CARGO_BIN_EXE_ess"), "cwd": self.root, "argv": args,
                "exit": output.status.code(), "elapsed_seconds": started.elapsed().as_secs_f64(),
                "stdout": format!("{index:02}.stdout"), "stderr": format!("{index:02}.stderr"),
            }),
        );
        println!(
            "CLI evidence: {} [{index:02}] exit {:?}",
            self.root.display(),
            output.status.code()
        );
        output
    }

    fn validate(&self, path: &str) -> Output {
        self.run(&[
            "generate",
            "schema",
            "validate",
            path,
            "--schemas",
            "registry",
            "--format",
            "json",
        ])
    }

    fn pair(&self, kind: &str) {
        let source_path = original(kind);
        let bytes = fs::read(&source_path).unwrap();
        let source: Value = serde_json::from_slice(&bytes).unwrap();
        assert!(source.get("$id").is_none());
        let mut resource = source.clone();
        resource["$id"] = json!(payload_id(kind));
        self.json(&format!("registry/{kind}-resource.schema.json"), &resource);
        let envelope: Value = serde_json::from_slice(
            &fs::read(fixtures().join(format!("registry/{kind}-selector.schema.json"))).unwrap(),
        )
        .unwrap();
        assert_eq!(envelope["$id"], envelope_id(kind));
        assert_eq!(envelope["properties"]["schema"]["const"], envelope_id(kind));
        assert_eq!(envelope["properties"]["payload"]["$ref"], payload_id(kind));
        assert_eq!(envelope["required"], json!(["schema", "payload"]));
        assert_eq!(envelope["additionalProperties"], false);
        self.json(&format!("registry/{kind}-selector.schema.json"), &envelope);
        let copied: Value = serde_json::from_slice(
            &fs::read(
                self.root
                    .join(format!("registry/{kind}-resource.schema.json")),
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(copied["$id"], payload_id(kind));
        let mut recovered = copied.clone();
        recovered.as_object_mut().unwrap().remove("$id");
        assert_eq!(recovered, source);
        if kind == "invoice" {
            assert_eq!(
                copied["$schema"],
                "https://json-schema.org/draft/2020-12/schema"
            );
            assert_eq!(
                copied["properties"]["amount"]["$ref"],
                "#/$defs/billing.invoice.Money"
            );
            assert_eq!(copied["x-ess-provenance"], source["x-ess-provenance"]);
        } else {
            assert_eq!(copied["$schema"], "http://json-schema.org/draft-07/schema#");
            assert_eq!(
                copied["properties"]["domains"]["items"]["$ref"],
                "#/definitions/QualifiedName"
            );
        }
        assert_ne!(
            fs::read(
                self.root
                    .join(format!("registry/{kind}-resource.schema.json"))
            )
            .unwrap(),
            bytes
        );
        assert_eq!(fs::read(source_path).unwrap(), bytes);
    }

    fn typescript(&self, id: &str, out: &str, check: bool) -> Output {
        let mut args = vec![
            "generate",
            "schema",
            "typescript",
            id,
            "--root",
            "Widget",
            "--schemas",
            "registry",
            "--out",
            out,
        ];
        if check {
            args.push("--check");
        }
        self.run(&args)
    }
}

fn report(output: &Output, exit: i32) -> Value {
    assert_eq!(output.status.code(), Some(exit), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
    serde_json::from_slice(&output.stdout).expect("JSON validation report")
}

fn refused(output: &Output, code: &str, pointer: &str) -> Value {
    let value = report(output, 1);
    assert_eq!(value["valid"], json!([]));
    let issues = value["issues"].as_array().unwrap();
    assert!(
        issues
            .iter()
            .any(|issue| issue["code"] == code && issue["instance_path"] == pointer),
        "expected {code} at {pointer:?}: {value}"
    );
    value
}

#[test]
fn idless_generated_schemas_are_not_registry_resources() {
    for kind in ["invoice", "source"] {
        let fixture = Fixture::new(kind);
        let bytes = fs::read(original(kind)).unwrap();
        fixture.write("registry/original.schema.json", &bytes);
        fixture.json("instance.json", &instance(kind));
        let value = refused(&fixture.validate("instance.json"), "missing_schema_id", "");
        assert_eq!(value["schema_count"], 0);
        assert_eq!(
            value["issues"][0]["document"],
            "registry/original.schema.json"
        );
        assert_eq!(fs::read(original(kind)).unwrap(), bytes);
    }
}

#[test]
fn both_dialects_accept_separate_adopter_resources_and_report_the_envelope() {
    for kind in ["invoice", "source"] {
        let fixture = Fixture::new(kind);
        fixture.pair(kind);
        let selected = instance(kind);
        fixture.json("instance.json", &selected);
        let value = report(&fixture.validate("instance.json"), 0);
        assert_eq!(value["schema_count"], 2);
        assert_eq!(value["issues"], json!([]));
        assert_eq!(
            value["valid"],
            json!([{
                "instance": "instance.json", "schema_id": envelope_id(kind),
                "schema": format!("registry/{kind}-selector.schema.json")
            }])
        );
        let wrapped: Value =
            serde_json::from_slice(&fs::read(fixture.root.join("instance.json")).unwrap()).unwrap();
        assert_eq!(wrapped["payload"], selected["payload"]);
    }
    let fixture = Fixture::new("combined");
    fixture.pair("invoice");
    fixture.pair("source");
    fixture.json("instances/invoice.json", &instance("invoice"));
    fixture.json("instances/source.json", &instance("source"));
    let value = report(&fixture.validate("instances"), 0);
    assert_eq!(value["schema_count"], 4);
    assert_eq!(value["issues"], json!([]));
    assert_eq!(value["valid"].as_array().unwrap().len(), 2);
}

#[test]
fn nested_payload_constraints_are_checked_through_both_resource_roots() {
    let fixture = Fixture::new("nested");
    fixture.pair("invoice");
    fixture.pair("source");
    for (label, mut value, pointer) in [
        ("type", instance("invoice"), "/payload/amount/amount"),
        ("required", instance("invoice"), "/payload"),
        ("refinement", instance("invoice"), "/payload/account_id"),
        ("syntax", instance("source"), "/payload/domains/0"),
    ] {
        match label {
            "type" => value["payload"]["amount"]["amount"] = json!(7),
            "required" => {
                value["payload"]
                    .as_object_mut()
                    .unwrap()
                    .remove("customer_email");
            }
            "refinement" => value["payload"]["account_id"] = json!("not-a-uuid"),
            "syntax" => value["payload"]["domains"][0] = json!(99),
            _ => unreachable!(),
        }
        fixture.json("invalid.json", &value);
        refused(
            &fixture.validate("invalid.json"),
            "invalid_instance",
            pointer,
        );
    }
}

#[test]
fn selectors_and_strict_envelopes_do_not_modify_domain_payloads() {
    let fixture = Fixture::new("selectors");
    fixture.pair("invoice");
    for (label, code, pointer) in [
        ("missing", "missing_schema_selector", "/schema"),
        ("empty", "missing_schema_selector", "/schema"),
        ("number", "missing_schema_selector", "/schema"),
        ("unknown", "unknown_schema", "/schema"),
        ("filename", "unknown_schema", "/schema"),
        ("payload", "invalid_instance", ""),
        ("extra", "invalid_instance", ""),
        ("direct", "invalid_instance", ""),
    ] {
        let mut value = instance("invoice");
        match label {
            "missing" => {
                value.as_object_mut().unwrap().remove("schema");
            }
            "empty" => value["schema"] = json!(""),
            "number" => value["schema"] = json!(7),
            "unknown" => value["schema"] = json!("urn:example:unknown:1"),
            "filename" => value["schema"] = json!("invoice-selector.schema.json"),
            "payload" => {
                value.as_object_mut().unwrap().remove("payload");
            }
            "extra" => value["extra"] = json!(true),
            "direct" => {
                value = value["payload"].clone();
                value["schema"] = json!(INVOICE);
            }
            _ => unreachable!(),
        }
        fixture.json("invalid.json", &value);
        refused(&fixture.validate("invalid.json"), code, pointer);
    }
}

#[test]
fn offline_missing_references_and_exact_duplicate_ids_refuse_the_registry() {
    for variant in ["missing", "https", "duplicate"] {
        let fixture = Fixture::new(variant);
        fixture.json("instance.json", &instance("invoice"));
        if variant == "duplicate" {
            fixture.pair("invoice");
            let first = fixture.root.join("registry/invoice-resource.schema.json");
            fixture.write(
                "registry/zzz-unrelated.schema.json",
                fs::read(first).unwrap(),
            );
            let value = refused(
                &fixture.validate("instance.json"),
                "duplicate_schema_id",
                "/$id",
            );
            assert_eq!(
                value["issues"][0]["document"],
                "registry/zzz-unrelated.schema.json"
            );
            assert!(value["issues"][0]["message"]
                .as_str()
                .unwrap()
                .contains(INVOICE));
            assert!(value["issues"][0]["message"]
                .as_str()
                .unwrap()
                .contains("registry/invoice-resource.schema.json"));
        } else {
            let mut envelope: Value = serde_json::from_slice(
                &fs::read(fixtures().join("registry/invoice-selector.schema.json")).unwrap(),
            )
            .unwrap();
            if variant == "https" {
                envelope["properties"]["payload"]["$ref"] =
                    json!("https://example.invalid/unprovided.schema.json");
            }
            fixture.json("registry/only-envelope.schema.json", &envelope);
            refused(&fixture.validate("instance.json"), "unresolved_schema", "");
        }
    }
}

#[test]
fn syntax_acceptance_is_distinct_from_domain_roster_assembly() {
    let fixture = Fixture::new("assembly");
    fixture.pair("source");
    let raw = fs::read(fixtures().join("source/semantic-invalid.json")).unwrap();
    let source: Value = serde_json::from_slice(&raw).unwrap();
    let wrapped = instance("source");
    assert_eq!(wrapped["payload"], source);
    fixture.json("instance.json", &wrapped);
    assert_eq!(
        report(&fixture.validate("instance.json"), 0)["valid"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    fixture.write("semantic-invalid.json", &raw);
    let output = fixture.run(&[
        "specify",
        "validate",
        "--path",
        "semantic-invalid.json",
        "--format",
        "json",
    ]);
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    let diagnostic = String::from_utf8_lossy(&output.stdout);
    assert!(diagnostic.contains("shop.cart"), "{output:?}");
    assert!(diagnostic.contains("system.domains"), "{output:?}");
    assert_eq!(
        fs::read(fixture.root.join("semantic-invalid.json")).unwrap(),
        raw
    );
}

fn widget(fixture: &Fixture) {
    fixture.write(
        "registry/unrelated-resource.schema.json",
        fs::read(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("tests/fixtures/schema-contract/registry/widget.schema.json"),
        )
        .unwrap(),
    );
}

#[test]
fn typescript_uses_root_id_and_preserves_equal_and_stale_check_behavior() {
    let fixture = Fixture::new("typescript-positive");
    widget(&fixture);
    let expected = fs::read(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/schema-contract/expected/widget.ts"),
    )
    .unwrap();
    assert!(fixture
        .typescript("urn:example:widget:1", "generated/widget.ts", false)
        .status
        .success());
    assert_eq!(
        fs::read(fixture.root.join("generated/widget.ts")).unwrap(),
        expected
    );
    assert!(fixture
        .typescript("urn:example:widget:1", "generated/widget.ts", true)
        .status
        .success());
    assert_eq!(
        fs::read(fixture.root.join("generated/widget.ts")).unwrap(),
        expected
    );
    fixture.write("generated/widget.ts", b"sentinel stale\n");
    let output = fixture.typescript("urn:example:widget:1", "generated/widget.ts", true);
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stdout).contains(": stale"));
    assert_eq!(
        fs::read(fixture.root.join("generated/widget.ts")).unwrap(),
        b"sentinel stale\n"
    );
}

#[test]
fn typescript_id_and_projection_refusals_happen_before_output_writes() {
    for (variant, expected) in [
        ("missing", "found 0"),
        ("duplicate", "found 2"),
        ("generated", "unsupported"),
        (
            "external",
            "only local `#/$defs/...` references are projectable",
        ),
    ] {
        let fixture = Fixture::new(variant);
        let id = if matches!(variant, "generated" | "external") {
            fixture.pair("invoice");
            if variant == "generated" {
                INVOICE
            } else {
                INVOICE_ENVELOPE
            }
        } else {
            widget(&fixture);
            if variant == "duplicate" {
                fixture.write(
                    "registry/duplicate.schema.json",
                    fs::read(fixture.root.join("registry/unrelated-resource.schema.json")).unwrap(),
                );
                "urn:example:widget:1"
            } else {
                "urn:example:absent:1"
            }
        };
        fixture.write("sentinel.ts", b"unchanged sentinel\n");
        for destination in ["sentinel.ts", "absent-parent/output.ts"] {
            let output = fixture.typescript(id, destination, false);
            assert_eq!(output.status.code(), Some(1), "{output:?}");
            let diagnostic = String::from_utf8_lossy(&output.stderr);
            assert!(diagnostic.contains(expected), "{output:?}");
            if variant == "generated" {
                assert!(diagnostic.contains("x-ess-"), "{output:?}");
            }
            if variant == "external" {
                assert!(diagnostic.contains(INVOICE), "{output:?}");
            }
            assert_eq!(
                fs::read(fixture.root.join("sentinel.ts")).unwrap(),
                b"unchanged sentinel\n"
            );
            assert!(!fixture.root.join("absent-parent").exists());
        }
    }
}

#[test]
fn accepted_instances_can_coexist_with_later_selector_failures() {
    let fixture = Fixture::new("mixed-instances");
    fixture.pair("invoice");
    fixture.json("instances/a-valid.json", &instance("invoice"));
    fixture.json(
        "instances/z-unknown.json",
        &json!({"schema": "urn:example:unknown:1"}),
    );
    let value = report(&fixture.validate("instances"), 1);
    assert_eq!(
        value["valid"],
        json!([{
            "instance": "instances/a-valid.json", "schema_id": INVOICE_ENVELOPE,
            "schema": "registry/invoice-selector.schema.json"
        }])
    );
    assert_eq!(value["issues"].as_array().unwrap().len(), 1);
    assert_eq!(value["issues"][0]["code"], "unknown_schema");
}
