//! Final bounded regression attempts against the corrected checked import boundary.

use ess_openapi::{import, project_import, read_import};
use serde_json::{json, Value};

fn source(schema: &Value) -> String {
    json!({
        "openapi": "3.1.0", "info": {"title": "pass-two", "version": "v1"},
        "paths": {}, "components": {"schemas": {"A": schema, "B": schema}}
    })
    .to_string()
}

#[test]
fn null_empty_and_escaped_unknown_unit_fields_refuse() {
    for kind in ["integer", "number", "boolean"] {
        let report = import(&source(&json!({"type": kind}))).unwrap();
        let original: Value = serde_json::from_str(&report.to_canonical_json()).unwrap();
        for (key, value, escaped) in [
            ("", Value::Null, ""),
            ("extra~/", json!({}), "extra~0~1"),
            ("kind~", json!([]), "kind~0"),
            ("constraint", json!(false), "constraint"),
        ] {
            let mut wire = original.clone();
            wire["interface"]["types"]["A"][key] = value;
            for bytes in [wire.to_string(), serde_yaml::to_string(&wire).unwrap()] {
                let errors = read_import(&bytes).unwrap_err();
                assert!(
                    errors.iter().any(|error| {
                        error.pointer == format!("/interface/types/A/{escaped}")
                            && error.message.contains("unknown field")
                    }),
                    "{errors:?}"
                );
            }
        }
    }
}

#[test]
fn yaml_aliases_cannot_hide_unknown_unit_fields() {
    for kind in ["integer", "number", "boolean"] {
        let report = import(&source(&json!({"type": kind}))).unwrap();
        let bytes: Value = serde_json::from_str(&report.to_canonical_json()).unwrap();
        let bytes = bytes.to_string();
        let node = format!("{{\"kind\":\"{kind}\"}}");
        assert_eq!(bytes.matches(&node).count(), 2);
        let control = bytes
            .replacen(&node, &format!("&unit {{kind: {kind}}}"), 1)
            .replacen(&node, "*unit", 1);
        assert_eq!(read_import(&control).unwrap(), report);
        let unknown = control.replacen(
            &format!("&unit {{kind: {kind}}}"),
            &format!("&unit {{kind: {kind}, extra: null}}"),
            1,
        );
        let errors = read_import(&unknown).unwrap_err();
        assert!(
            errors
                .iter()
                .any(|error| error.pointer == "/interface/types/A/extra"
                    && error.message.contains("unknown field")),
            "{errors:?}"
        );
    }
}

#[test]
fn schema_like_annotation_literals_do_not_trigger_wire_preflight() {
    let report = import(&source(&json!({
        "type": "string", "enum": ["integer", "number", "boolean"],
        "default": {"kind": "integer", "extra": null},
        "examples": [{"kind": "boolean", "extra": []}]
    })))
    .unwrap();
    assert_eq!(report.accounting().normalizations.len(), 4);
    assert!(report.accounting().coverage_gaps.is_empty());
    let checked = read_import(&report.to_canonical_json()).unwrap();
    assert_eq!(checked, report);
    assert!(project_import(&checked).is_ok());
}

#[test]
fn duplicate_source_keys_refuse_even_when_values_agree() {
    let original = source(&json!({"type": "integer"}));
    for (needle, replacement) in [
        (
            "\"openapi\":\"3.1.0\"",
            "\"openapi\":\"3.1.0\",\"openapi\":\"3.1.0\"",
        ),
        (
            "\"type\":\"integer\"",
            "\"type\":\"integer\",\"ty\\u0070e\":\"integer\"",
        ),
    ] {
        let duplicate = original.replacen(needle, replacement, 1);
        assert_ne!(duplicate, original);
        let errors = import(&duplicate).unwrap_err();
        assert!(
            errors
                .iter()
                .any(|error| error.message.contains("duplicate")),
            "{errors:?}"
        );
    }
    let yaml = "openapi: 3.1.0\ninfo: {title: pass-two, version: v1}\npaths: {}\ncomponents:\n  schemas:\n    A:\n      type: integer\n      type: integer\n";
    let errors = import(yaml).unwrap_err();
    assert!(
        errors
            .iter()
            .any(|error| error.message.contains("duplicate")),
        "{errors:?}"
    );
}

#[test]
fn exact_source_identity_requires_a_consistent_new_envelope() {
    let original = import(&source(&json!({"type": "integer", "minimum": 0}))).unwrap();
    let changed_source = format!(
        "# same interpretation, new source bytes\n{}",
        original.source().text
    );
    let changed = import(&changed_source).unwrap();
    assert_eq!(original.interface(), changed.interface());
    assert_eq!(original.accounting(), changed.accounting());
    assert_ne!(original.source().sha256, changed.source().sha256);
    let mut wire: Value = serde_json::from_str(&original.to_canonical_json()).unwrap();
    wire["source"]["text"] = json!(changed_source);
    assert!(read_import(&wire.to_string()).is_err());
    wire["source"]["sha256"] = json!(changed.source().sha256);
    assert_eq!(read_import(&wire.to_string()).unwrap(), changed);
    wire["source"]["sha256"] = json!(changed.source().sha256.to_uppercase());
    assert!(read_import(&wire.to_string()).is_err());
}

#[test]
fn null_schema_nodes_cannot_disappear_behind_optional_carriers() {
    let mut sources = Vec::new();
    let component: Value = serde_json::from_str(&source(&Value::Null)).unwrap();
    sources.push((component, "/components/schemas/A"));
    let property: Value = serde_json::from_str(&source(&json!({
        "type": "object", "properties": {"p": null}, "additionalProperties": false
    })))
    .unwrap();
    sources.push((property, "/components/schemas/A/properties/p"));
    let items: Value =
        serde_json::from_str(&source(&json!({"type": "array", "items": null}))).unwrap();
    sources.push((items, "/components/schemas/A/items"));
    for request in [false, true] {
        let mut message: Value = serde_json::from_str(&source(&json!({"type": "string"}))).unwrap();
        message["paths"] = json!({"/probe": {"post": {
            "operationId": "probe", "responses": {"200": {"description": "ok"}}
        }}});
        let pointer = if request {
            message["paths"]["/probe"]["post"]["requestBody"] =
                json!({"content": {"application/json": {"schema": null}}});
            "/paths/~1probe/post/requestBody/content/application~1json/schema"
        } else {
            message["paths"]["/probe"]["post"]["responses"]["200"]["content"] =
                json!({"application/json": {"schema": null}});
            "/paths/~1probe/post/responses/200/content/application~1json/schema"
        };
        sources.push((message, pointer));
    }
    for (document, pointer) in sources {
        let errors = import(&document.to_string()).unwrap_err();
        assert!(
            errors.iter().any(|error| error.pointer == pointer),
            "{errors:?}"
        );
    }
}
