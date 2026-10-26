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

/// The model types of a union mixing a unit variant with a payload variant (ess/22,
/// beyond10x/ess#418), selected through the event that carries it.
fn unit_variant_plan() -> Plan {
    let text =
        include_str!("../../../specify/ess-compiler/tests/fixtures/union-unit-variants.yaml");
    let specification = ess_domain::spec::Specification::assemble([(
        ess_domain::system::Source::new("work.yaml"),
        ess_domain::spec::RawSpecFile::parse(text).unwrap(),
    )])
    .unwrap();
    let ir =
        ess_compiler::resolve::compile(&specification, &ess_compiler::source::SourceMap::new())
            .unwrap();
    let selected = ess_gen::schema::ModelTypes::select(
        &ir,
        &BTreeSet::from(["demo.work.StatusReported".to_owned()]),
    )
    .unwrap();
    Plan::from_model(&selected).unwrap()
}

/// A unit variant is the tag alone: both variants round-trip, and a unit variant written with a
/// content member, a payload variant written without one, and an undeclared tag are refused.
const UNIT_VARIANT_WIRE: &str = r#"
#[cfg(test)]
mod unit_variant_wire {
    use model_types::*;
    use serde_json::json;

    #[test]
    fn a_unit_variant_round_trips_as_its_tag_alone() {
        for status in [json!({"kind": "Open"}), json!({"kind": "Complete", "value": {"outcome": "shipped"}})] {
            let decoded: DemoWorkStatus = serde_json::from_value(status.clone()).unwrap();
            assert_eq!(serde_json::to_value(&decoded).unwrap(), status);
            let event = json!({"status": status});
            let decoded: DemoWorkStatusReported = serde_json::from_value(event.clone()).unwrap();
            assert_eq!(serde_json::to_value(&decoded).unwrap(), event);
        }
        for malformed in [
            json!({"kind": "Open", "value": {"outcome": "shipped"}}),
            json!({"kind": "Open", "value": null}),
            json!({"kind": "Complete"}),
            json!({"kind": "Closed"}),
        ] {
            assert!(serde_json::from_value::<DemoWorkStatus>(malformed.clone()).is_err(), "{malformed}");
        }
    }
}
"#;

#[test]
fn model_unit_variant_round_trips_natively_and_refuses_the_malformed_shapes() {
    let plan = unit_variant_plan();
    let result = plan.rust("model_types").unwrap();
    compile(
        &result.supporting["Cargo.toml"],
        &result.declarations,
        UNIT_VARIANT_WIRE,
    );
}

/// A command's input and response and a view's row selected as roots (beyond10x/ess#656): the
/// generated library decodes the bodies the `OpenAPI` projection accepts for them, the input's
/// being the one its published examples spell, and refuses the bodies that projection refuses.
const HTTP_BODY_WIRE: &str = r#"
#[cfg(test)]
mod http_body_wire {
    use model_types::*;
    use serde_json::{json, Value};

    fn round_trips<T: serde::de::DeserializeOwned + serde::Serialize>(body: &Value) {
        let decoded: T = serde_json::from_value(body.clone()).unwrap();
        assert_eq!(&serde_json::to_value(&decoded).unwrap(), body);
        for (key, bad) in [("extra", json!(1)), ("note_id", json!(7))] {
            let mut refused = body.clone();
            refused[key] = bad;
            assert!(serde_json::from_value::<T>(refused).is_err(), "{key}");
        }
        let mut missing = body.clone();
        missing.as_object_mut().unwrap().remove("note_id");
        assert!(serde_json::from_value::<T>(missing).is_err());
    }

    #[test]
    fn the_types_match_the_http_projection() {
        round_trips::<DemoNotesCreateInput>(&json!({"note_id": "n-1", "title": "Buy milk", "isPinned": true}));
        round_trips::<DemoNotesCreateResponse>(&json!({"note_id": "n-1", "titleLength": 8}));
        round_trips::<DemoNotesNotesRow>(&json!({"note_id": "n-1", "title": "Buy milk"}));
        round_trips::<DemoNotesSummariesRow>(&json!({"note_id": "n-1", "title": "Buy milk"}));
    }
}
"#;

#[test]
fn the_types_match_the_http_projection() {
    let text = include_str!("../../ess-gen/tests/fixtures/command-view-roots.yaml");
    let specification = ess_domain::spec::Specification::assemble([(
        ess_domain::system::Source::new("notes.yaml"),
        ess_domain::spec::RawSpecFile::parse(text).unwrap(),
    )])
    .unwrap();
    let ir =
        ess_compiler::resolve::compile(&specification, &ess_compiler::source::SourceMap::new())
            .unwrap();
    let selected = ess_gen::schema::ModelTypes::select(
        &ir,
        &BTreeSet::from(
            [
                "demo.notes.Create.input",
                "demo.notes.Create.response",
                "demo.notes.Notes.row",
                "demo.notes.Summaries.row",
            ]
            .map(str::to_owned),
        ),
    )
    .unwrap();
    let result = Plan::from_model(&selected)
        .unwrap()
        .rust("model_types")
        .unwrap();
    compile(
        &result.supporting["Cargo.toml"],
        &result.declarations,
        HTTP_BODY_WIRE,
    );
}

#[path = "support/empty_struct.rs"]
mod empty_struct;

/// The generated decoders of a struct with `fields: []` (ess/32, beyond10x/ess#649).
const EMPTY_STRUCT_WIRE: &str = r#"
#[cfg(test)]
mod empty_struct_wire {
    use model_types::*;
    use serde_json::{json, Value};

    fn round_trips<T: serde::de::DeserializeOwned + serde::Serialize>(body: Value) {
        let decoded: T = serde_json::from_value(body.clone()).unwrap();
        assert_eq!(serde_json::to_value(&decoded).unwrap(), body);
    }

    #[test]
    fn an_open_empty_struct_decodes_any_object() {
        round_trips::<DemoAckOpen>(json!({}));
        round_trips::<DemoAckOpen>(json!({"x": 1}));
        assert!(serde_json::from_value::<DemoAckOpen>(json!("x")).is_err());
    }

    #[test]
    fn a_closed_empty_struct_decodes_only_the_empty_object() {
        round_trips::<DemoAckClosed>(json!({}));
        assert!(serde_json::from_value::<DemoAckClosed>(json!({"x": 1})).is_err());
        assert!(serde_json::from_value::<DemoAckClosed>(json!("x")).is_err());
    }
}
"#;

#[test]
fn an_open_empty_struct_decodes_any_object() {
    let result = empty_struct::plan().rust("model_types").unwrap();
    compile(
        &result.supporting["Cargo.toml"],
        &result.declarations,
        EMPTY_STRUCT_WIRE,
    );
}

#[test]
fn a_closed_empty_struct_decodes_only_the_empty_object() {
    let result = empty_struct::plan().rust("model_types").unwrap();
    assert!(
        result
            .declarations
            .contains("#[serde(deny_unknown_fields)]\npub struct DemoAckClosed {"),
        "{}",
        result.declarations
    );
    compile(
        &result.supporting["Cargo.toml"],
        &result.declarations,
        EMPTY_STRUCT_WIRE,
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
