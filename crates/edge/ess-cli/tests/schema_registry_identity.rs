//! Actual CLI characterization of adopter resource IDs and separate selector envelopes.

use std::cell::Cell;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;

use ess_cli::TemporaryDirectory;
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
    // Unset `ESS_SCHEMA_IDENTITY_EVIDENCE` keeps the evidence in TMPDIR scratch, removed on drop.
    _scratch: Option<TemporaryDirectory>,
}

impl Fixture {
    fn new(label: &str) -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let (scratch, base) = std::env::var_os("ESS_SCHEMA_IDENTITY_EVIDENCE").map_or_else(
            || {
                let scratch = TemporaryDirectory::create("ess-schema-identity").unwrap();
                let base = scratch.to_path_buf();
                (Some(scratch), base)
            },
            |base| (None, PathBuf::from(base)),
        );
        let root = base.join(format!(
            "schema-identity-{label}-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&root).unwrap();
        Self {
            root,
            next: Cell::new(0),
            _scratch: scratch,
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
fn typescript_uses_root_id_and_checks_equal_output_and_refuses_edited_output() {
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
    // An owned file whose bytes `.ess-output` does not record refuses the check, naming the file
    // and the re-enroll route rather than calling it stale, since regeneration refuses it too
    // (beyond10x/ess#484); nothing is written.
    fixture.write("generated/widget.ts", b"sentinel stale\n");
    let output = fixture.typescript("urn:example:widget:1", "generated/widget.ts", true);
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    for named in ["widget.ts", "ess generate output adopt"] {
        assert!(
            stderr.contains(named),
            "the refusal names {named}: {stderr}"
        );
    }
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

/// ESS's own schema tools accept what `ess generate --kind schema` writes: an identified
/// projection is a registry resource, a YAML instance validates as the JSON it denotes, and the
/// importer takes a projected document as it stands (beyond10x/ess#590, #591, #593).
mod projected {
    use std::fs;
    use std::process::Output;

    use serde_json::json;

    use super::{report, repository, Fixture};

    const BASE: &str = "https://schemas.example.org/billing/";
    const MONEY: &str =
        "https://schemas.example.org/billing/types/billing.invoice.Money.schema.json";
    const ENVELOPE: &str = "urn:example:money-submission:1";

    fn generate(fixture: &Fixture, extra: &[&str]) -> Output {
        let billing = repository().join("examples/billing");
        let mut args = vec![
            "generate",
            "--path",
            billing.to_str().unwrap(),
            "--kind",
            "schema",
            "--out",
            "gen",
        ];
        args.extend_from_slice(extra);
        fixture.run(&args)
    }

    /// Every projected type document, as written, beside an adopter's envelope that selects one by
    /// the identity the projection minted.
    fn registry(fixture: &Fixture) {
        let generated = generate(fixture, &["--schema-id-base", BASE]);
        assert_eq!(generated.status.code(), Some(0), "{generated:?}");
        for entry in fs::read_dir(fixture.root.join("gen/schema/types")).unwrap() {
            let entry = entry.unwrap();
            fixture.write(
                &format!("registry/{}", entry.file_name().to_str().unwrap()),
                fs::read(entry.path()).unwrap(),
            );
        }
        fixture.json(
            "registry/envelope.schema.json",
            &json!({
                "$schema": "https://json-schema.org/draft/2020-12/schema",
                "$id": ENVELOPE,
                "type": "object",
                "required": ["schema", "payload"],
                "properties": {"schema": {"const": ENVELOPE}, "payload": {"$ref": MONEY}},
                "additionalProperties": false
            }),
        );
    }

    #[test]
    fn issue_590_the_registry_validates_against_identified_projections() {
        let fixture = Fixture::new("projected-ids");
        registry(&fixture);
        let money: serde_json::Value = serde_json::from_slice(
            &fs::read(
                fixture
                    .root
                    .join("gen/schema/types/billing.invoice.Money.schema.json"),
            )
            .unwrap(),
        )
        .unwrap();
        assert_eq!(money["$id"], MONEY);

        fixture.json(
            "valid.json",
            &json!({"schema": ENVELOPE, "payload": {"amount": "12.5", "currency": "EUR"}}),
        );
        let accepted = report(&fixture.validate("valid.json"), 0);
        assert_eq!(accepted["valid"][0]["schema_id"], ENVELOPE, "{accepted}");
        assert_eq!(accepted["issues"], json!([]));

        fixture.json(
            "invalid.json",
            &json!({"schema": ENVELOPE, "payload": {"amount": 12.5, "currency": "EUR"}}),
        );
        let refused = report(&fixture.validate("invalid.json"), 1);
        assert_eq!(
            refused["issues"][0]["code"], "invalid_instance",
            "{refused}"
        );
        assert_eq!(refused["issues"][0]["instance_path"], "/payload/amount");

        let same = generate(&fixture, &["--schema-id-base", BASE, "--check"]);
        assert_eq!(same.status.code(), Some(0), "{same:?}");
        let unidentified = generate(&fixture, &["--check"]);
        assert_eq!(
            unidentified.status.code(),
            Some(1),
            "the identities are part of the projected bytes: {unidentified:?}"
        );
    }

    #[test]
    fn issue_590_a_schema_id_base_needs_kind_schema() {
        let fixture = Fixture::new("projected-id-refusals");
        let billing = repository().join("examples/billing");
        let billing = billing.to_str().unwrap();
        for (args, says) in [
            (
                vec![
                    "generate", "--path", billing, "--kind", "openapi", "--out", "gen",
                ],
                "requires --kind schema",
            ),
            (
                vec!["generate", "--path", billing, "--out", "gen"],
                "requires --kind schema",
            ),
            (
                vec![
                    "generate", "--path", billing, "--kind", "schema", "--out", "gen",
                ],
                "does not end in",
            ),
        ] {
            let base = if says == "does not end in" {
                "https://schemas.example.org/billing"
            } else {
                BASE
            };
            let mut args = args;
            args.extend(["--schema-id-base", base]);
            let output = fixture.run(&args);
            assert_ne!(output.status.code(), Some(0), "{output:?}");
            let stderr = String::from_utf8_lossy(&output.stderr);
            assert!(stderr.contains(says), "{args:?}: {stderr}");
            assert!(!fixture.root.join("gen").exists(), "{args:?} wrote nothing");
        }
    }

    #[test]
    fn issue_591_a_yaml_instance_validates_as_the_json_it_denotes() {
        let fixture = Fixture::new("projected-yaml");
        registry(&fixture);
        fixture.write(
            "instances/valid.yaml",
            format!("schema: {ENVELOPE}\npayload:\n  amount: \"12.5\"\n  currency: EUR\n"),
        );
        fixture.write(
            "instances/invalid.yml",
            format!("schema: {ENVELOPE}\npayload:\n  amount: 12.5\n  currency: EUR\n"),
        );
        let value = report(&fixture.validate("instances"), 1);
        assert_eq!(value["valid"].as_array().unwrap().len(), 1, "{value}");
        assert_eq!(value["valid"][0]["instance"], "instances/valid.yaml");
        assert_eq!(value["issues"][0]["document"], "instances/invalid.yml");
        assert_eq!(value["issues"][0]["code"], "invalid_instance");
        assert_eq!(value["issues"][0]["instance_path"], "/payload/amount");
    }

    #[test]
    fn issue_591_a_yaml_instance_outside_json_is_refused_by_file() {
        let fixture = Fixture::new("projected-yaml-refusals");
        registry(&fixture);
        for (name, text, says) in [
            ("tagged.yaml", "schema: !thing x\n", "tag"),
            ("keyed.yaml", "1: one\n", "not a string"),
            ("infinite.yaml", "amount: .inf\n", "not finite"),
            ("twice.yaml", "schema: a\nschema: b\n", "schema"),
            ("two.yaml", "a: 1\n---\na: 2\n", "more than one document"),
        ] {
            fixture.write(name, text);
            let output = fixture.validate(name);
            assert_ne!(output.status.code(), Some(0), "{name}: {output:?}");
            let stderr = String::from_utf8_lossy(&output.stderr);
            assert!(
                stderr.contains(name) && stderr.contains(says),
                "{name}: {stderr}"
            );
        }
    }

    /// The issue's newtype, projected with identities and validated through an envelope: a value
    /// outside the alphabet is refused by the pattern the projection now carries.
    #[test]
    fn issue_592_validate_refuses_a_value_outside_the_alphabet() {
        const HEX: &str =
            "https://schemas.example.org/codes/types/example.codes.Sha256Hex.schema.json";
        const SUBMISSION: &str = "urn:example:digest-submission:1";
        let fixture = Fixture::new("projected-alphabet");
        fixture.write(
            "codes.yaml",
            "format: ess/15\nsystem: example\nversion: v1\ndomain: example.codes\ntypes:\n  - name: \
             example.codes.Sha256Hex\n    kind: newtype\n    of: String\n    alphabet: \
             \"0123456789abcdef\"\n",
        );
        let generated = fixture.run(&[
            "generate",
            "--path",
            "codes.yaml",
            "--kind",
            "schema",
            "--out",
            "gen",
            "--schema-id-base",
            "https://schemas.example.org/codes/",
        ]);
        assert_eq!(generated.status.code(), Some(0), "{generated:?}");
        fixture.write(
            "registry/example.codes.Sha256Hex.schema.json",
            fs::read(
                fixture
                    .root
                    .join("gen/schema/types/example.codes.Sha256Hex.schema.json"),
            )
            .unwrap(),
        );
        fixture.json(
            "registry/envelope.schema.json",
            &json!({
                "$schema": "https://json-schema.org/draft/2020-12/schema",
                "$id": SUBMISSION,
                "type": "object",
                "required": ["schema", "payload"],
                "properties": {"schema": {"const": SUBMISSION}, "payload": {"$ref": HEX}},
                "additionalProperties": false
            }),
        );
        fixture.json(
            "inside.json",
            &json!({"schema": SUBMISSION, "payload": "0123456789abcdef"}),
        );
        let accepted = report(&fixture.validate("inside.json"), 0);
        assert_eq!(accepted["issues"], json!([]), "{accepted}");

        fixture.json(
            "outside.json",
            &json!({"schema": SUBMISSION, "payload": "ZZZb0123456789abcdef"}),
        );
        let refused = report(&fixture.validate("outside.json"), 1);
        assert_eq!(
            refused["issues"][0]["code"], "invalid_instance",
            "{refused}"
        );
        assert_eq!(refused["issues"][0]["instance_path"], "/payload");
        assert!(
            refused["issues"][0]["message"]
                .as_str()
                .unwrap_or_default()
                .contains("^[0-9a-f]*$"),
            "the refusal names the pattern: {refused}"
        );
    }

    /// A document projected under `--schema-id-base` carries a root `$id` and imports as one
    /// without it: the identity changes nothing the bundle validates (review 2026-10-19, B3).
    #[test]
    fn issue_590_an_identified_projection_imports_as_a_bundle() {
        let fixture = Fixture::new("projected-identified-bundle");
        let generated = generate(&fixture, &["--schema-id-base", BASE]);
        assert_eq!(generated.status.code(), Some(0), "{generated:?}");
        let document = "gen/schema/types/billing.invoice.Money.schema.json";
        let projected: serde_json::Value =
            serde_json::from_slice(&fs::read(fixture.root.join(document)).unwrap()).unwrap();
        assert_eq!(
            projected["$id"],
            format!("{BASE}types/billing.invoice.Money.schema.json")
        );
        let imported = fixture.run(&[
            "generate",
            "schema",
            "import-document",
            "--path",
            document,
            "--root",
            "Money",
            "--dialect",
            "draft-2020-12",
            "--out",
            "bundle.json",
        ]);
        assert_eq!(imported.status.code(), Some(0), "{imported:?}");
        fixture.write("money.yaml", "amount: \"12.5\"\ncurrency: EUR\n");
        fixture.write("loose.yaml", "amount: 12.5\ncurrency: EUR\n");
        let validated = fixture.run(&[
            "generate",
            "schema",
            "validate-bundle",
            "--bundle",
            "bundle.json",
            "--root",
            "Money",
            "money.yaml",
            "loose.yaml",
        ]);
        assert_eq!(validated.status.code(), Some(1), "{validated:?}");
        let stdout = String::from_utf8_lossy(&validated.stdout);
        assert!(stdout.contains("2 instance(s), 1 valid"), "{stdout}");
        assert!(stdout.contains("loose.yaml"), "{stdout}");
    }

    #[test]
    fn issue_593_a_projected_document_imports_and_validates_a_yaml_instance() {
        let fixture = Fixture::new("projected-bundle");
        let generated = generate(&fixture, &[]);
        assert_eq!(generated.status.code(), Some(0), "{generated:?}");
        let imported = fixture.run(&[
            "generate",
            "schema",
            "import-document",
            "--path",
            "gen/schema/types/billing.invoice.Money.schema.json",
            "--root",
            "Money",
            "--dialect",
            "draft-2020-12",
            "--out",
            "bundle.json",
        ]);
        assert_eq!(imported.status.code(), Some(0), "{imported:?}");
        fixture.write("money.yaml", "amount: \"12.5\"\ncurrency: EUR\n");
        fixture.write(
            "loose.yaml",
            "amount: \"12.5\"\ncurrency: EUR\nextra: true\n",
        );
        let validated = fixture.run(&[
            "generate",
            "schema",
            "validate-bundle",
            "--bundle",
            "bundle.json",
            "--root",
            "Money",
            "money.yaml",
            "loose.yaml",
        ]);
        assert_eq!(validated.status.code(), Some(1), "{validated:?}");
        let stdout = String::from_utf8_lossy(&validated.stdout);
        assert!(stdout.contains("2 instance(s), 1 valid"), "{stdout}");
        assert!(stdout.contains("loose.yaml"), "{stdout}");
        assert!(!stdout.contains("money.yaml"), "{stdout}");
    }
}
