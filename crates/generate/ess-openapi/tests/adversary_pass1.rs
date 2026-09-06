//! Counterexamples written before execution for the frozen F04 first adversary pass.

use ess_openapi::{import, project_import, read_import, AccountingCode, SCHEMA_DIALECT};
use serde_json::{json, Value};

fn document(schema: &Value) -> Value {
    json!({
        "openapi": "3.1.0", "info": {"title": "adversary", "version": "v1"},
        "paths": {}, "components": {"schemas": {"A": schema}}
    })
}

fn at_site(schema: Value, site: usize) -> (Value, &'static str, &'static str) {
    let mut source = document(&json!({"type": "string"}));
    match site {
        0 => {
            source["components"]["schemas"]["A"] = schema;
            (source, "/components/schemas/A", "/interface/types/A")
        }
        1 => {
            source["components"]["schemas"]["A"] = json!({
                "type": "object", "additionalProperties": false,
                "properties": {"p~/": schema}, "required": ["p~/"]
            });
            (
                source,
                "/components/schemas/A/properties/p~0~1",
                "/interface/types/A/properties/p~0~1",
            )
        }
        2 | 3 => {
            source["paths"] = json!({"/probe": {"post": {
                "operationId": "probe", "responses": {"200": {"description": "ok"}}
            }}});
            let (source_pointer, wire_pointer) = if site == 2 {
                source["paths"]["/probe"]["post"]["requestBody"] = json!({
                    "content": {"application/json": {"schema": schema}}
                });
                (
                    "/paths/~1probe/post/requestBody/content/application~1json/schema",
                    "/interface/operations/probe/request/schema",
                )
            } else {
                source["paths"]["/probe"]["post"]["responses"]["200"]["content"] =
                    json!({"application/json": {"schema": schema}});
                (
                    "/paths/~1probe/post/responses/200/content/application~1json/schema",
                    "/interface/operations/probe/responses/200/schema",
                )
            };
            (source, source_pointer, wire_pointer)
        }
        4 => {
            source["components"]["schemas"]["A"] = json!({"type": "array", "items": schema});
            (
                source,
                "/components/schemas/A/items",
                "/interface/types/A/items",
            )
        }
        _ => unreachable!(),
    }
}

fn assert_closed_unit(kind: &str) {
    let mut accepted = Vec::new();
    for site in 0..5 {
        let (source, _, wire_pointer) = at_site(json!({"type": kind}), site);
        let report = import(&source.to_string()).unwrap();
        assert!(project_import(&report).is_ok());
        let mut wire: Value = serde_json::from_str(&report.to_canonical_json()).unwrap();
        wire.pointer_mut(wire_pointer).unwrap()["unexpected_constraint"] = json!({"const": 7});
        for (encoding, bytes) in [
            ("json", wire.to_string()),
            ("yaml", serde_yaml::to_string(&wire).unwrap()),
        ] {
            match read_import(&bytes) {
                Err(refusals) => println!("refused {encoding} {wire_pointer}: {refusals:?}"),
                Ok(checked) => {
                    let erased = !checked
                        .to_canonical_json()
                        .contains("unexpected_constraint");
                    let projected = project_import(&checked).is_ok();
                    println!("ADMITTED {encoding} {wire_pointer}/unexpected_constraint; erased={erased}; checked_projection_ok={projected}");
                    accepted.push(format!("{encoding} {wire_pointer}"));
                }
            }
        }
    }
    assert!(
        accepted.is_empty(),
        "closed import envelope admitted unknown fields in {kind} unit variants: {accepted:?}"
    );
}

#[test]
fn closed_import_rejects_unknown_integer_variant_fields() {
    assert_closed_unit("integer");
}

#[test]
fn closed_import_rejects_unknown_number_variant_fields() {
    assert_closed_unit("number");
}

#[test]
fn closed_import_rejects_unknown_boolean_variant_fields() {
    assert_closed_unit("boolean");
}

#[test]
fn nonunit_schema_variants_reject_unknown_fields() {
    for schema in [
        json!({"type": "string"}),
        json!({"type": "array", "items": {"type": "string"}}),
        json!({"type": "object", "properties": {}, "additionalProperties": false}),
        json!({"$ref": "#/components/schemas/A"}),
    ] {
        let source = document(&schema);
        let report = import(&source.to_string()).unwrap();
        let mut wire: Value = serde_json::from_str(&report.to_canonical_json()).unwrap();
        wire["interface"]["types"]["A"]["unexpected_constraint"] = json!(7);
        assert!(read_import(&wire.to_string()).is_err(), "{wire}");
    }
}

#[test]
fn unconsumed_variant_keywords_remain_gaps_after_replay() {
    for (schema, keywords) in [
        (
            json!({"type": "integer", "format": "int64", "pattern": "x", "items": {"type": "string"}}),
            vec!["format", "pattern", "items"],
        ),
        (
            json!({"type": "number", "enum": [1.5], "const": 1.5, "exclusiveMinimum": 0}),
            vec!["enum", "const", "exclusiveMinimum"],
        ),
        (
            json!({"type": "boolean", "enum": [true], "const": true, "properties": {}}),
            vec!["enum", "const", "properties"],
        ),
        (
            json!({"type": "string", "minLength": 2, "readOnly": true, "writeOnly": true, "x-ess-invariants": ["guard"]}),
            vec!["minLength", "readOnly", "writeOnly", "x-ess-invariants"],
        ),
        (
            json!({"type": "array", "items": {"type": "integer"}, "contains": {"const": 3}, "uniqueItems": true}),
            vec!["contains", "uniqueItems"],
        ),
        (
            json!({"$ref": "#/components/schemas/A", "type": "string", "const": "a", "properties": {}}),
            vec!["type", "const", "properties"],
        ),
    ] {
        for site in 0..5 {
            let (source, pointer, _) = at_site(schema.clone(), site);
            let report = import(&source.to_string()).unwrap();
            let checked = read_import(&report.to_canonical_json()).unwrap();
            for keyword in &keywords {
                let expected = format!("{pointer}/{keyword}");
                assert!(
                    checked
                        .accounting()
                        .coverage_gaps
                        .iter()
                        .any(|entry| entry.pointer == expected),
                    "missing {expected}"
                );
            }
            assert!(project_import(&checked).is_err());
        }
    }
}

#[test]
fn contradictory_string_enum_and_const_survive_both_boundaries() {
    for site in 0..5 {
        let (source, source_pointer, _) = at_site(json!({"enum": ["a"], "const": "b"}), site);
        let report = import(&source.to_string()).unwrap();
        let checked = read_import(&report.to_canonical_json()).unwrap();
        let projected = project_import(&checked).unwrap();
        let value: Value = serde_yaml::from_str(&projected).unwrap();
        let schema = value.pointer(source_pointer).unwrap();
        assert_eq!(schema["enum"], json!(["a"]));
        assert_eq!(schema["const"], "b");
        assert_eq!(schema["type"], "string");
    }
}

#[test]
fn unsupported_items_cannot_disappear_at_nested_or_message_sites() {
    for items in [
        Value::Bool(true),
        Value::Bool(false),
        json!({}),
        json!({"type": ["string"]}),
    ] {
        for site in 0..5 {
            let (source, pointer, _) = at_site(json!({"type": "array", "items": items}), site);
            let errors = import(&source.to_string()).unwrap_err();
            assert!(
                errors
                    .iter()
                    .any(|error| error.pointer.starts_with(&format!("{pointer}/items"))),
                "{errors:?}"
            );
        }
    }
}

#[test]
fn schema_resource_and_dialect_features_refuse_but_annotation_literals_do_not() {
    for site in 0..5 {
        for key in ["$schema", "$id", "$anchor", "$dynamicRef", "$vocabulary"] {
            let mut schema = json!({"type": "string"});
            schema[key] = json!("https://example.invalid/unsupported");
            let (source, pointer, _) = at_site(schema, site);
            let errors = import(&source.to_string()).unwrap_err();
            assert!(
                errors
                    .iter()
                    .any(|error| error.pointer == format!("{pointer}/{key}")),
                "{errors:?}"
            );
        }
        let (mut source, pointer, _) = at_site(
            json!({
                "type": "string", "$schema": SCHEMA_DIALECT,
                "default": {"$schema": "unsupported", "$ref": "external", "$id": "literal"},
                "examples": [{"type": "array"}]
            }),
            site,
        );
        source["jsonSchemaDialect"] = json!(SCHEMA_DIALECT);
        let report = import(&source.to_string()).unwrap();
        let checked = read_import(&report.to_canonical_json()).unwrap();
        assert!(checked.accounting().coverage_gaps.is_empty());
        assert!(checked
            .accounting()
            .normalizations
            .iter()
            .any(|entry| entry.pointer == format!("{pointer}/default")
                && entry.code == AccountingCode::AnnotationOmitted));
        assert!(project_import(&checked).is_ok());
    }
}

#[test]
fn reference_escape_identity_is_not_decoded_twice() {
    let source = json!({
        "openapi": "3.1.0", "info": {"title": "adversary", "version": "v1"}, "paths": {},
        "components": {"schemas": {
            "~1": {"type": "string"},
            "A": {"$ref": "#/components/schemas/~01"},
            "B": {"$ref": "#/components/schemas/Missing~01"},
            "C": {"type": "array", "items": {"$ref": "#/components/schemas/Missing~01"}}
        }}
    });
    let report = import(&source.to_string()).unwrap();
    let checked = read_import(&report.to_canonical_json()).unwrap();
    let unresolved = &checked.accounting().unresolved_references;
    assert_eq!(unresolved.len(), 2);
    assert!(unresolved.iter().all(|entry| entry.target == "Missing~1"));
    assert_eq!(unresolved[0].pointer, "/components/schemas/B/$ref");
    assert_eq!(unresolved[1].pointer, "/components/schemas/C/items/$ref");
    assert!(project_import(&checked).is_err());
}

#[test]
fn accounting_order_duplicates_codes_and_missing_arrays_cannot_be_normalized_away() {
    let report = import(
        &document(
            &json!({"type": "integer", "minimum": 0, "maximum": 9, "description": "literal"}),
        )
        .to_string(),
    )
    .unwrap();
    let original: Value = serde_json::from_str(&report.to_canonical_json()).unwrap();
    let mut mutations = Vec::new();
    let mut wire = original.clone();
    wire["accounting"]["coverage_gaps"]
        .as_array_mut()
        .unwrap()
        .reverse();
    mutations.push(wire);
    let mut wire = original.clone();
    let duplicate = wire["accounting"]["coverage_gaps"][0].clone();
    wire["accounting"]["coverage_gaps"]
        .as_array_mut()
        .unwrap()
        .push(duplicate);
    mutations.push(wire);
    let mut wire = original.clone();
    wire["accounting"]["coverage_gaps"][0]["code"] = json!("annotation-omitted");
    mutations.push(wire);
    for name in ["normalizations", "coverage_gaps", "unresolved_references"] {
        let mut wire = original.clone();
        wire["accounting"].as_object_mut().unwrap().remove(name);
        mutations.push(wire);
    }
    for wire in mutations {
        assert!(read_import(&wire.to_string()).is_err(), "{wire}");
    }
}

#[test]
fn duplicate_escaped_map_keys_are_rejected_before_replay() {
    let report = import(&document(&json!({"type": "number"})).to_string()).unwrap();
    let bytes = report.to_canonical_json();
    let changed = bytes.replacen(
        "\"kind\": \"number\"",
        r#""kind": "number", "ki\u006ed": "number""#,
        1,
    );
    assert_ne!(bytes, changed);
    let errors = read_import(&changed).unwrap_err();
    assert!(
        errors
            .iter()
            .any(|error| error.message.contains("duplicate")),
        "{errors:?}"
    );
}
