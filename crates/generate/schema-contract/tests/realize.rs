//! Shared structural planning and accounted target generation from checked imports.

use std::collections::BTreeSet;

use schema_contract::bundle::{import, Bundle, Dialect};
use schema_contract::realize::Plan;
use serde_json::{json, Value};

fn bundle(schemas: Value, roots: &[&str]) -> Bundle {
    let mut source = json!({"openapi": "3.0.0", "components": {"schemas": null}});
    source["components"]["schemas"] = schemas;
    import(
        &source.to_string(),
        &roots.iter().map(|name| (*name).to_owned()).collect(),
        Dialect::Draft202012,
    )
    .unwrap()
}

fn roots(names: &[&str]) -> BTreeSet<String> {
    names.iter().map(|name| (*name).to_owned()).collect()
}

#[test]
fn roots_select_their_exact_closure_and_keep_qualified_provenance() {
    let bundle = bundle(
        json!({
            "Record": {"type": "object", "properties": {"id": {"$ref": "#/components/schemas/Identifier"}}, "additionalProperties": false},
            "Identifier": {"type": "string"}, "Other": {"type": "integer"}
        }),
        &["Record", "Other"],
    );
    let original = bundle.to_json().unwrap();
    let selected = Plan::from_bundle(&bundle, &roots(&["Record"])).unwrap();
    assert_eq!(
        selected
            .declarations()
            .keys()
            .map(String::as_str)
            .collect::<Vec<_>>(),
        ["Identifier", "Record"]
    );
    let result = selected.typescript();
    assert_eq!(
        result,
        Plan::from_bundle(&Bundle::read(&original).unwrap(), &roots(&["Record"]))
            .unwrap()
            .typescript()
    );
    assert_eq!(bundle.to_json().unwrap(), original);
    let report = serde_json::to_value(&result.report).unwrap();
    assert_eq!(report["source_digest"], bundle.source_digest());
    assert_eq!(report["format"], "ess-types-report/3");
    assert_eq!(report["roots"], json!(["Record"]));
    assert_eq!(report["target"], "typescript");
    assert_eq!(report["input"]["kind"], "bundle");
    assert_eq!(report["input"]["bundle_digest"].as_str().unwrap().len(), 64);
    assert!(Plan::from_bundle(&bundle, &roots(&["Identifier"])).is_err());
    assert!(Plan::from_bundle(&bundle, &BTreeSet::new()).is_err());
}

#[test]
fn requiredness_nullability_defaults_and_unrestricted_json_are_distinct() {
    let bundle = bundle(
        json!({"Record": {"type": "object", "additionalProperties": false,
        "required": ["requiredNullable", "requiredValue"], "properties": {
            "requiredNullable": {"type": ["string", "null"]},
            "requiredValue": true,
            "optionalNullable": {"type": ["string", "null"]},
            "defaultValue": {"type": "string", "default": "seed"},
            "obsolete": {"deprecated": true}, "impossible": false
        }}}),
        &["Record"],
    );
    let result = Plan::from_bundle(&bundle, &roots(&["Record"]))
        .unwrap()
        .typescript();
    assert!(result
        .declarations
        .contains("\"requiredNullable\": (string | null);"));
    assert!(result
        .declarations
        .contains("\"optionalNullable\"?: (string | null);"));
    assert!(result
        .declarations
        .contains("\"requiredValue\": EssJsonValue;"));
    assert!(result.declarations.contains("\"defaultValue\"?: string;"));
    assert!(result.declarations.contains("\"obsolete\"?: EssJsonValue;"));
    assert!(result.declarations.contains("\"impossible\"?: never;"));
    assert!(!result.declarations.contains("unknown"));
    let report = serde_json::to_value(result.report).unwrap();
    assert!(report["annotations"]
        .as_array()
        .unwrap()
        .iter()
        .any(|item| item["rule"] == "default"));
}

#[test]
fn references_enums_consts_and_explicit_types_apply_as_intersections() {
    let bundle = bundle(
        json!({
            "Base": {"type": ["string", "null"]},
            "Narrow": {"$ref": "#/components/schemas/Base", "type": "string", "enum": ["x", "y"], "const": "x"},
            "All": {"allOf": [{"$ref": "#/components/schemas/Base"}, {"type": "null"}]}
        }),
        &["Narrow", "All"],
    );
    let result = Plan::from_bundle(&bundle, &roots(&["Narrow", "All"]))
        .unwrap()
        .typescript();
    assert!(result
        .declarations
        .contains("export type Narrow = (Base & \"x\" & (\"x\" | \"y\") & string);"));
    assert!(result
        .declarations
        .contains("export type All = (Base & null);"));
}

#[test]
fn tuple_positions_open_objects_and_exclusive_unions_are_accounted() {
    let bundle = bundle(
        json!({
            "Tuple": {"type": "array", "prefixItems": [{"type": "string"}, true, {"type": "boolean"}], "minItems": 3, "maxItems": 3},
            "OptionalTuple": {"type": "array", "prefixItems": [{"type": "string"}, {"type": "boolean"}], "minItems": 1, "items": {"type": "integer"}},
            "Empty": {"type": "array", "items": false},
            "Choice": {"oneOf": [{"type": "integer"}, {"type": "number"}]},
            "Open": {"type": "object", "required": ["label"], "properties": {"label": {"type": "string"}}, "additionalProperties": {"type": "integer"}}
        }),
        &["Tuple", "OptionalTuple", "Empty", "Choice", "Open"],
    );
    let result = Plan::from_bundle(&bundle, bundle.roots())
        .unwrap()
        .typescript();
    assert!(result
        .declarations
        .contains("export type Tuple = [(string), (EssJsonValue), (boolean)];"));
    assert!(result
        .declarations
        .contains("export type OptionalTuple = [(string), (boolean)?, ...(number)[]];"));
    assert!(result
        .declarations
        .contains("export type Empty = (never)[];"));
    let report = serde_json::to_value(result.report).unwrap();
    let rules = report["obligations"]
        .as_array()
        .unwrap()
        .iter()
        .map(|item| item["rule"].as_str().unwrap())
        .collect::<BTreeSet<_>>();
    for rule in [
        "oneOf",
        "integer",
        "json_number_precision",
        "additional_property_index",
        "minItems",
        "maxItems",
    ] {
        assert!(rules.contains(rule), "{rule}");
    }
}

#[test]
fn unsupported_shapes_are_accumulated_without_implicit_narrowing() {
    let bundle = bundle(
        json!({
            "Base": {"type": "object", "properties": {"child": {"type": "string"}}},
            "Conditional": {"$ref": "#/components/schemas/Base", "required": ["child"]},
            "Nested": {"$ref": "#/components/schemas/Base/properties/child"},
            "Negative": {"not": {"type": "null"}}
        }),
        &["Conditional", "Nested", "Negative"],
    );
    let refused = Plan::from_bundle(&bundle, bundle.roots()).unwrap_err();
    let rules = refused
        .0
        .iter()
        .map(|item| item.rule.as_str())
        .collect::<BTreeSet<_>>();
    assert_eq!(
        rules,
        BTreeSet::from([
            "conditional_shape",
            "unsupported_keyword",
            "unsupported_reference"
        ])
    );
    assert_eq!(
        Plan::from_bundle(&bundle, bundle.roots()).unwrap_err(),
        refused
    );
}

#[test]
fn allocation_and_recursion_refuse_only_the_unsupported_boundaries() {
    let colliding = bundle(
        json!({"a-b": true, "a_b": true, "Self": true}),
        &["a-b", "a_b", "Self"],
    );
    let errors = Plan::from_bundle(&colliding, colliding.roots()).unwrap_err();
    assert!(errors.0.iter().any(|item| item.rule == "name_collision"));
    assert!(errors.0.iter().any(|item| item.rule == "invalid_name"));
    let cyclic = bundle(
        json!({"A": {"$ref": "#/components/schemas/B"}, "B": {"$ref": "#/components/schemas/A"}}),
        &["A"],
    );
    assert!(Plan::from_bundle(&cyclic, cyclic.roots())
        .unwrap_err()
        .0
        .iter()
        .all(|item| item.rule == "unguarded_recursion"));
    let guarded = bundle(
        json!({"Node": {"type": "object", "properties": {"children": {"type": "array", "items": {"$ref": "#/components/schemas/Node"}}}}}),
        &["Node"],
    );
    assert!(Plan::from_bundle(&guarded, guarded.roots())
        .unwrap()
        .typescript()
        .declarations
        .contains("(Node)[]"));
    let escaped = bundle(
        json!({"node/name~": {"type": "string"}, "Root": {"$ref": "#/components/schemas/node~1name~0"}}),
        &["Root"],
    );
    assert!(Plan::from_bundle(&escaped, escaped.roots())
        .unwrap()
        .typescript()
        .declarations
        .contains("export type Root = NodeName;"));
}

#[test]
fn findings_resolve_in_original_source_and_floating_array_bounds_keep_positions() {
    let bundle = bundle(
        json!({
            "Open": {"type": "object", "properties": {"extra": {"type": "array"}}},
            "Tuple": {"type": "array", "minItems": 2.0, "maxItems": 2.0, "prefixItems": [{"type": "string"}, {"type": "boolean"}]}
        }),
        &["Open", "Tuple"],
    );
    let realization = Plan::from_bundle(&bundle, bundle.roots())
        .unwrap()
        .typescript();
    assert!(realization
        .declarations
        .contains("export type Tuple = [(string), (boolean)];"));
    let wire: Value = serde_json::from_str(&bundle.to_json().unwrap()).unwrap();
    let source: Value = serde_json::from_str(wire["source"].as_str().unwrap()).unwrap();
    let report = serde_json::to_value(realization.report).unwrap();
    for collection in ["annotations", "obligations"] {
        for item in report[collection].as_array().unwrap() {
            let pointer = item["pointer"].as_str().unwrap();
            if pointer.starts_with("/components/") {
                assert!(source.pointer(pointer).is_some(), "{pointer}");
            }
        }
    }
}
