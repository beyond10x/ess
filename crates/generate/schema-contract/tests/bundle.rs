//! Qualified source import, persisted integrity and structural validation equivalence.

use std::collections::BTreeSet;

use schema_contract::bundle::{self, Bundle, Dialect, SCHEMA_DIALECT};
use serde_json::{json, Value};

#[test]
fn document_root_retains_typed_fields_recursive_references_and_original_locations() {
    let source = json!({"$schema": SCHEMA_DIALECT, "type":"object", "required":["settings", "nullable"],
        "properties":{"settings":{"$ref":"#/$defs/Settings"},"nullable":{"type":["string","null"]},
            "children":{"type":"array","items":{"$ref":"#"}}},
        "$defs":{"Settings":{"type":"object","required":["enabled"],"properties":{"enabled":{"type":"boolean"}}},
            "Unused":{"type":"integer"}}});
    let bytes = format!("{}\n", serde_json::to_string_pretty(&source).unwrap());
    let imported = bundle::import_document(
        &bytes,
        "Application",
        &BTreeSet::new(),
        Dialect::Draft202012,
    )
    .unwrap();
    assert_eq!(imported.definitions().len(), 2);
    assert_eq!(imported.source_pointer("Application"), "");
    assert_eq!(imported.source_pointer("Settings"), "/$defs/Settings");
    let wire: Value = serde_json::from_str(&imported.to_json().unwrap()).unwrap();
    assert_eq!(wire["format"], bundle::DOCUMENT_FORMAT);
    assert_eq!(wire["document_root"], "Application");
    assert_eq!(wire["source"], bytes);
    assert_eq!(
        Bundle::read(&imported.to_json().unwrap()).unwrap(),
        imported
    );
    assert!(imported
        .validate(
            "Application",
            &json!({"settings":{"enabled":true},"nullable":null,"children":[]})
        )
        .unwrap()
        .is_empty());
    for invalid in [
        json!({}),
        json!({"settings":{"enabled":true}}),
        json!({"settings":false,"nullable":null}),
        json!({"settings":{"enabled":true},"nullable":null,"children":[{}]}),
    ] {
        assert!(
            !imported
                .validate("Application", &invalid)
                .unwrap()
                .is_empty(),
            "{invalid}"
        );
    }
    let plan = schema_contract::realize::Plan::from_bundle(&imported, imported.roots()).unwrap();
    for result in [
        plan.typescript(),
        plan.rust("types").unwrap(),
        plan.go("types", "example.invalid/types").unwrap(),
    ] {
        let report = serde_json::to_value(result.report).unwrap();
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
fn document_import_refuses_collisions_dialect_changes_and_nonlocal_references() {
    for source in [
        json!({"$defs":{"Root":true}}),
        json!({"$schema":"http://json-schema.org/draft-07/schema#"}),
        json!({"$ref":"https://example.invalid/schema"}),
        json!({"$ref":"#/$defs/Missing"}),
        json!({"$defs":false}),
        json!({"type":"string","nullable":true}),
        json!({"$id":"https://example.invalid/schema"}),
    ] {
        assert!(
            bundle::import_document(
                &source.to_string(),
                "Root",
                &BTreeSet::new(),
                Dialect::Draft202012
            )
            .is_err(),
            "{source}"
        );
    }
    for source in ["true", "false"] {
        let imported =
            bundle::import_document(source, "Root", &BTreeSet::new(), Dialect::Draft202012)
                .unwrap();
        assert_eq!(
            imported.validate("Root", &json!(null)).unwrap().is_empty(),
            source == "true"
        );
    }
    let imported = bundle::import_document(
        r##"{"$defs":{"a/b":{"type":"string"}},"$ref":"#/$defs/a~1b"}"##,
        "Root",
        &BTreeSet::from(["a/b".to_owned()]),
        Dialect::Draft202012,
    )
    .unwrap();
    assert_eq!(imported.roots().len(), 2);
    assert!(imported
        .validate("Root", &json!("value"))
        .unwrap()
        .is_empty());
}

#[test]
fn document_identity_is_versioned_and_replayed_while_v1_bytes_remain_unchanged() {
    #[derive(serde::Deserialize, serde::Serialize)]
    #[serde(deny_unknown_fields)]
    struct OldWire {
        format: String,
        source: String,
        source_digest: String,
        declared_dialect: Option<String>,
        selected_dialect: Dialect,
        accounting: bundle::Accounting,
        roots: BTreeSet<String>,
        definitions: std::collections::BTreeMap<String, Value>,
    }
    let old = bundle::import(
        r#"{"components":{"schemas":{"Root":{"type":"string"}}}}"#,
        &BTreeSet::from(["Root".to_owned()]),
        Dialect::Draft202012,
    )
    .unwrap()
    .to_json()
    .unwrap();
    let parsed: OldWire = serde_json::from_str(&old).unwrap();
    assert_eq!(
        format!("{}\n", serde_json::to_string_pretty(&parsed).unwrap()),
        old
    );
    let new = bundle::import_document(
        r#"{"type":"string"}"#,
        "Root",
        &BTreeSet::new(),
        Dialect::Draft202012,
    )
    .unwrap()
    .to_json()
    .unwrap();
    assert!(serde_json::from_str::<OldWire>(&new).is_err());
    let original: Value = serde_json::from_str(&new).unwrap();
    for (key, value) in [
        ("format", json!(bundle::FORMAT)),
        ("document_root", json!(null)),
        ("document_root", json!("Changed")),
        ("roots", json!([])),
        ("source_digest", json!("0".repeat(64))),
    ] {
        let mut changed = original.clone();
        changed[key] = value;
        assert!(Bundle::read(&changed.to_string()).is_err(), "{key}");
    }
    let mut old_with_null: Value = serde_json::from_str(&old).unwrap();
    old_with_null["document_root"] = Value::Null;
    assert!(Bundle::read(&old_with_null.to_string()).is_err());
}

fn source() -> Value {
    json!({"openapi": "3.0.0", "info": {"title": "", "version": ""}, "paths": {},
    "components": {"schemas": {
        "Root": {"type": "object", "required": ["value"], "additionalProperties": false,
            "properties": {
                "value": {"anyOf": [{"$ref": "#/components/schemas/Record"}, {"$ref": "#/components/schemas/Tuple"}]},
                "optional": {"type": ["string", "null"], "minLength": 2, "default": null},
                "map": {"$ref": "#/components/schemas/Open"},
                "literal": {"enum": [true, null, "fixed", 3]},
                "never": false,
                "anything": true
            }},
        "Record": {"allOf": [
            {"type": "object", "properties": {"count": {"type": "integer", "minimum": 1}}, "required": ["count"]},
            {"type": "object", "properties": {"count": {"maximum": 3}}}
        ]},
        "Tuple": {"type": "array", "prefixItems": [{"const": "pair"}, {"type": "number"}], "minItems": 2, "maxItems": 2},
        "Open": {"type": "object", "additionalProperties": {"type": "integer"}},
        "Unselected": {"unknown-structural-rule": true}
    }}})
}

fn import(value: &Value, roots: &[&str]) -> Bundle {
    bundle::import(
        &value.to_string(),
        &roots.iter().map(|root| (*root).to_owned()).collect(),
        Dialect::Draft202012,
    )
    .unwrap()
}

#[test]
fn root_closure_keeps_qualification_without_inventing_service_metadata() {
    let value = source();
    let before = value.to_string();
    let bundle = import(&value, &["Root"]);
    assert_eq!(
        bundle
            .definitions()
            .keys()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        ["Open", "Record", "Root", "Tuple"]
    );
    assert_eq!(bundle.declared_dialect(), Some("3.0.0"));
    assert_eq!(bundle.source_digest().len(), 64);
    assert!(bundle.accounting().refusals.is_empty());
    assert!(bundle.accounting().coverage_gaps.is_empty());
    assert_eq!(bundle.accounting().unresolved_references, 0);
    assert_eq!(value.to_string(), before);
    let projected = bundle.schema("Root", "urn:example:root").unwrap();
    assert_eq!(projected["$schema"], SCHEMA_DIALECT);
    assert_eq!(projected["x-ess-source"]["declared_dialect"], "3.0.0");
    assert!(projected.get("paths").is_none());
    assert_eq!(Bundle::read(&bundle.to_json().unwrap()).unwrap(), bundle);
}

#[test]
fn source_and_projection_agree_under_the_explicitly_selected_dialect() {
    let value = source();
    let bundle = import(&value, &["Root"]);
    let mut interpreted_source = value;
    interpreted_source["$schema"] = json!(SCHEMA_DIALECT);
    interpreted_source["$ref"] = json!("#/components/schemas/Root");
    let validator = jsonschema::options()
        .with_draft(jsonschema::Draft::Draft202012)
        .should_validate_formats(false)
        .build(&interpreted_source)
        .unwrap();
    let cases = [
        (json!({"value": {"count": 2}}), true),
        (
            json!({"value": ["pair", 1.5], "optional": null, "map": {"key": 2}}),
            true,
        ),
        (
            json!({"value": {"count": 1}, "anything": [null, false, {}]}),
            true,
        ),
        (json!({"value": {"count": 0}}), false),
        (json!({"value": {"count": 4}}), false),
        (json!({"value": ["pair", 1, 2]}), false),
        (json!({"value": ["pair"]}), false),
        (json!({"value": {"count": 1}, "optional": "x"}), false),
        (
            json!({"value": {"count": 1}, "map": {"key": "wrong"}}),
            false,
        ),
        (json!({"value": {"count": 1}, "never": null}), false),
        (json!({"value": {"count": 1}, "unexpected": true}), false),
        (json!({}), false),
    ];
    for (instance, expected) in cases {
        let before = instance.clone();
        assert_eq!(
            validator.is_valid(&instance),
            expected,
            "source: {instance}"
        );
        assert_eq!(
            bundle.validate("Root", &instance).unwrap().is_empty(),
            expected,
            "projected: {instance}"
        );
        assert_eq!(instance, before, "validation never applies defaults");
    }
}

#[test]
fn annotations_are_not_walked_as_schemas_and_reference_siblings_survive() {
    let value = json!({"components": {"schemas": {
        "Root": {"$ref": "#/components/schemas/Text", "minLength": 3,
            "default": {"$ref": "#/components/schemas/NotAReference"},
            "examples": [{"$id": "literal", "unsupported-keyword": 1}]},
        "Text": {"type": "string"}
    }}});
    let bundle = import(&value, &["Root"]);
    assert_eq!(
        bundle.definitions()["Root"]["default"],
        value["components"]["schemas"]["Root"]["default"]
    );
    assert!(!bundle.validate("Root", &json!("ab")).unwrap().is_empty());
    assert!(bundle.validate("Root", &json!("abc")).unwrap().is_empty());
}

#[test]
fn recursive_schemas_and_escaped_component_names_keep_their_identity() {
    let value = json!({"components": {"schemas": {
        "Node/~ space": {"type": "object", "properties": {
            "next": {"anyOf": [{"type": "null"}, {"$ref": "#/components/schemas/Node~1~0%20space"}]}
        }}
    }}});
    let bundle = import(&value, &["Node/~ space"]);
    assert_eq!(bundle.definitions().len(), 1);
    assert!(bundle
        .validate("Node/~ space", &json!({"next": {"next": null}}))
        .unwrap()
        .is_empty());
    assert!(!bundle
        .validate("Node/~ space", &json!({"next": 1}))
        .unwrap()
        .is_empty());
}

#[test]
fn complete_persisted_accounting_is_rechecked_and_unknown_formats_refuse() {
    let bundle = import(&source(), &["Root"]);
    let original: Value = serde_json::from_str(&bundle.to_json().unwrap()).unwrap();
    for (pointer, replacement) in [
        ("/source_digest", json!("0".repeat(64))),
        ("/declared_dialect", json!("3.1.0")),
        ("/definitions/Root", json!(true)),
        ("/format", json!("ess-schema-bundle/0")),
        ("/selected_dialect", json!("guess")),
    ] {
        let mut changed = original.clone();
        *changed.pointer_mut(pointer).unwrap() = replacement;
        assert!(Bundle::read(&changed.to_string()).is_err(), "{pointer}");
    }
    let mut unknown = original;
    unknown["extra"] = json!(true);
    assert!(Bundle::read(&unknown.to_string()).is_err());
}

#[test]
fn one_of_is_exclusive_and_false_roots_accept_no_value() {
    let value = json!({"components": {"schemas": {
        "Exclusive": {"oneOf": [{"type": "integer"}, {"type": "number"}]},
        "Never": false
    }}});
    let bundle = import(&value, &["Exclusive", "Never"]);
    assert_eq!(
        bundle.schema("Never", "urn:example:never").unwrap()["$defs"]
            .as_object()
            .unwrap()
            .len(),
        1
    );
    assert!(!bundle.validate("Exclusive", &json!(1)).unwrap().is_empty());
    assert!(bundle
        .validate("Exclusive", &json!(1.5))
        .unwrap()
        .is_empty());
    assert!(!bundle.validate("Never", &Value::Null).unwrap().is_empty());
}

#[test]
fn unsupported_semantics_are_accumulated_at_source_pointers() {
    let value = json!({"components": {"schemas": {"Root": {"type": "object", "properties": {
        "external": {"$ref": "https://example.invalid/private.json"},
        "missing": {"$ref": "#/components/schemas/Absent"},
        "resource": {"$id": "urn:other:resource", "type": "string"},
        "unknown": {"nullable": true},
        "dialect": {"$schema": "http://json-schema.org/draft-07/schema#"}
    }}}}});
    let errors = bundle::import(
        &value.to_string(),
        &BTreeSet::from(["Root".to_owned()]),
        Dialect::Draft202012,
    )
    .unwrap_err();
    assert_eq!(errors.0.len(), 5, "{errors}");
    assert_eq!(errors.accounting().unresolved_references, 2);
    assert_eq!(errors.accounting().refusals.len(), 5);
    assert!(errors.0.iter().all(|error| error
        .pointer
        .starts_with("/components/schemas/Root/properties/")));
}

#[test]
fn malformed_schema_constraints_and_unselected_projection_roots_refuse() {
    let mut value = source();
    value["components"]["schemas"]["Record"]["allOf"][0]["properties"]["count"]["minimum"] =
        json!("one");
    assert!(bundle::import(
        &value.to_string(),
        &BTreeSet::from(["Root".to_owned()]),
        Dialect::Draft202012
    )
    .is_err());
    let bundle = import(&source(), &["Root"]);
    assert!(bundle.schema("Record", "urn:example:record").is_err());
    assert!(bundle.schema("Root", "relative.schema.json").is_err());
    assert!(bundle.schema("Root", "urn:example:root#fragment").is_err());
}

#[test]
fn original_byte_changes_remain_visible_even_when_structural_meaning_is_equal() {
    let value = source();
    let roots = BTreeSet::from(["Root".to_owned()]);
    let compact = bundle::import(&value.to_string(), &roots, Dialect::Draft202012).unwrap();
    let pretty = bundle::import(
        &serde_json::to_string_pretty(&value).unwrap(),
        &roots,
        Dialect::Draft202012,
    )
    .unwrap();
    assert_ne!(compact.source_digest(), pretty.source_digest());
    assert_eq!(compact.definitions(), pretty.definitions());
    assert_eq!(
        compact.to_json().unwrap(),
        bundle::import(&value.to_string(), &roots, Dialect::Draft202012)
            .unwrap()
            .to_json()
            .unwrap()
    );
}
