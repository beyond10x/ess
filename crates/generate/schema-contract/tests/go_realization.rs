//! Explicit Go compiler and wire-codec verification for generated data libraries.

#[path = "support/model.rs"]
mod model;

#[test]
fn model_wire_mapping_builds_with_native_roundtrips() {
    let plan = model::plan();
    let result = plan
        .go("contract_types", "example.invalid/modeltypes")
        .unwrap();
    assert_eq!(
        result,
        plan.go("contract_types", "example.invalid/modeltypes")
            .unwrap()
    );
    compile(
        &result.supporting["go.mod"],
        &result.declarations,
        include_str!("fixtures/model_wire_tests.go.txt"),
    );
}

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

use schema_contract::bundle::{import, Dialect};
use schema_contract::realize::Plan;
use serde_json::{json, Value};

fn plan(schemas: &Value, selected: &[&str]) -> Plan {
    let roots = selected
        .iter()
        .map(|name| (*name).to_owned())
        .collect::<BTreeSet<_>>();
    let bundle = import(
        &json!({"components": {"schemas": schemas}}).to_string(),
        &roots,
        Dialect::Draft202012,
    )
    .unwrap();
    Plan::from_bundle(&bundle, &roots).unwrap()
}

#[test]
fn generated_go_library_preserves_wire_values_and_native_presence() {
    let plan = plan(
        &json!({
            "Record": {"type": "object", "required": ["requiredNullable", "requiredValue"], "properties": {
                "requiredNullable": {"type": ["string", "null"]}, "requiredValue": true,
                "optionalNullable": {"type": ["string", "null"]}, "defaultValue": {"type": "string", "default": "seed"}
            }},
            "Closed": {"type": "object", "additionalProperties": false, "required": ["type"], "properties": {"type": {"type": "boolean"}}},
            "Choice": {"anyOf": [{"$ref": "#/components/schemas/Record"}, {"$ref": "#/components/schemas/Closed"}]},
            "Tag": {"type": "string", "enum": ["ready", "paused"]},
            "Narrow": {"$ref": "#/components/schemas/Tag", "const": "ready", "type": "string"},
            "Tuple": {"type": "array", "prefixItems": [{"$ref": "#/components/schemas/Tag"}, true, {"type": "string"}], "minItems": 3, "maxItems": 3},
            "Node": {"type": "object", "additionalProperties": false, "properties": {"children": {"type": "array", "items": {"$ref": "#/components/schemas/Node"}}}},
            "Dictionary": {"type": "object", "additionalProperties": {"type": "integer"}},
            "List": {"type": "array", "items": {"type": "string"}},
            "Impossible": false
        }),
        &[
            "Record",
            "Closed",
            "Choice",
            "Tag",
            "Narrow",
            "Tuple",
            "Node",
            "Dictionary",
            "List",
            "Impossible",
        ],
    );
    let result = plan
        .go("contract_types", "example.invalid/contracttypes")
        .unwrap();
    assert_eq!(
        result,
        plan.go("contract_types", "example.invalid/contracttypes")
            .unwrap()
    );
    let report = serde_json::to_value(&result.report).unwrap();
    assert_eq!(
        report["configuration"],
        json!({"language": "go", "package": "contract_types", "module": "example.invalid/contracttypes"})
    );
    compile(
        &result.supporting["go.mod"],
        &result.declarations,
        include_str!("fixtures/native_wire_tests.go.txt"),
    );
}

#[test]
fn go_refusals_cover_native_shapes_identifiers_and_compound_bindings() {
    let plan = plan(
        &json!({
            "Intersection": {"allOf": [{"type": "string"}, {"type": "boolean"}]},
            "Tuple": {"type": "array", "prefixItems": [{"type": "string"}]},
            "Collision": {"type": "object", "properties": {"a-b": true, "a_b": true, "marshalJSON": true}},
            "Tag": {"enum": ["x"]}, "TagV0": {"type": "string"}
        }),
        &["Intersection", "Tuple", "Collision", "Tag", "TagV0"],
    );
    let errors = plan
        .go("contract_types", "example.invalid/contracttypes")
        .unwrap_err();
    for rule in [
        "go_intersection_or_literal",
        "go_prefix_layout",
        "go_field_collision",
        "go_declaration_collision",
    ] {
        assert!(errors.0.iter().any(|error| error.rule == rule), "{rule}");
    }
    for (package, module) in [
        ("type", "example.invalid/types"),
        ("types", "../escape"),
        ("types", "x\"\nreplace"),
    ] {
        assert!(plan.go(package, module).is_err());
    }
}

#[test]
fn untrusted_go_wire_names_cannot_introduce_declarations() {
    let name = "name\nvar injected = unknownSymbol";
    let plan = plan(
        &json!({name: {"type": "object", "properties": {"quote\"\nkey": {"type": "string"}}}}),
        &[name],
    );
    let result = plan
        .go("quoted_types", "example.invalid/quotedtypes")
        .unwrap();
    assert!(!result.declarations.contains("\nvar injected"));
    compile(&result.supporting["go.mod"], &result.declarations, "");
}

fn compile(manifest: &str, source: &str, tests: &str) {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let compiler = PathBuf::from(
        std::env::var_os("ESS_GO_COMPILER").expect("go-typecheck requires ESS_GO_COMPILER"),
    );
    assert!(
        compiler.is_file(),
        "ESS_GO_COMPILER must name an installed Go compiler"
    );
    let version = Command::new(&compiler)
        .arg("version")
        .env("GOTOOLCHAIN", "local")
        .output()
        .unwrap();
    assert!(version.status.success());
    assert!(
        String::from_utf8_lossy(&version.stdout)
            .split_whitespace()
            .nth(2)
            .and_then(|version| version.split('-').next())
            == Some("go1.26.5"),
        "the native lane is pinned to Go 1.26.5"
    );
    let root = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!(
        "go-types-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir_all(&root).unwrap();
    fs::write(root.join("go.mod"), manifest).unwrap();
    fs::write(root.join("types.go"), source).unwrap();
    if !tests.is_empty() {
        fs::write(root.join("wire_test.go"), tests).unwrap();
    }
    let result = Command::new(compiler)
        .args(["test", "./..."])
        .current_dir(&root)
        .env("GOTOOLCHAIN", "local")
        .env("GOPROXY", "off")
        .env("GOSUMDB", "off")
        .env("GOWORK", "off")
        .env("GOFLAGS", "")
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
}
