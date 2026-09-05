//! Compile generated native data libraries and exercise their actual wire serialization.

#[path = "support/model.rs"]
mod model;

#[test]
fn model_wire_mapping_builds_with_native_roundtrips() {
    let plan = model::plan();
    let result = plan.rust("model_types").unwrap();
    assert_eq!(result, plan.rust("model_types").unwrap());
    compile(
        &result.supporting["Cargo.toml"],
        &result.declarations,
        include_str!("fixtures/model_wire_tests.rs.txt"),
    );
}

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
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
fn generated_library_preserves_presence_open_data_unions_and_exact_numbers() {
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
            "Impossible",
        ],
    );
    let result = plan.rust("contract_types").unwrap();
    assert_eq!(result, plan.rust("contract_types").unwrap());
    let report = serde_json::to_value(&result.report).unwrap();
    assert_eq!(
        report["configuration"],
        json!({"language": "rust", "package": "contract_types"})
    );
    compile(
        &result.supporting["Cargo.toml"],
        &result.declarations,
        include_str!("fixtures/native_wire_tests.rs.txt"),
    );
}

#[test]
fn unsupported_native_shapes_and_names_refuse_before_emission() {
    let plan = plan(
        &json!({
            "Intersection": {"allOf": [{"type": "string"}, {"type": "boolean"}]},
            "Tuple": {"type": "array", "prefixItems": [{"type": "string"}]},
            "Collision": {"type": "object", "properties": {"a-b": true, "a_b": true}},
            "Literal": {"const": 3}
        }),
        &["Intersection", "Tuple", "Collision", "Literal"],
    );
    let errors = plan.rust("contract_types").unwrap_err();
    for rule in [
        "rust_intersection_or_literal",
        "rust_prefix_layout",
        "rust_field_collision",
    ] {
        assert!(errors.0.iter().any(|error| error.rule == rule), "{rule}");
    }
    for package in ["", "Upper", "../escape", "a\"\n", "serde", "self"] {
        assert!(plan.rust(package).is_err());
    }
}

#[test]
fn hostile_source_names_remain_comments_and_wire_strings() {
    let name = "name\ncompile_error!(\"injected\");";
    let plan = plan(
        &json!({name: {"type": "object", "properties": {"quote\"\nkey": {"type": "string"}}}}),
        &[name],
    );
    let result = plan.rust("quoted_types").unwrap();
    assert!(!result.declarations.contains("\ncompile_error!"));
    compile(&result.supporting["Cargo.toml"], &result.declarations, "");
}

fn compile(manifest: &str, source: &str, tests: &str) {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let root = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!(
        "native-types-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir_all(root.join("tests")).unwrap();
    fs::write(root.join("Cargo.toml"), manifest).unwrap();
    fs::write(root.join("types.rs"), source).unwrap();
    if !tests.is_empty() {
        fs::write(root.join("tests/wire.rs"), tests).unwrap();
    }
    let result = Command::new(env!("CARGO"))
        .args(["test", "--offline", "--quiet", "--manifest-path"])
        .arg(root.join("Cargo.toml"))
        .env(
            "CARGO_TARGET_DIR",
            Path::new(env!("CARGO_TARGET_TMPDIR")).join("native-types-target"),
        )
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
}
