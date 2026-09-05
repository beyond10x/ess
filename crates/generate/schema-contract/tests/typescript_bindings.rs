//! Boundary cases for the public declaration allocator, without requiring an external compiler.

use schema_contract::typescript::{project, ProjectionError};
use serde_json::json;

#[test]
fn escaped_and_non_ascii_definition_keys_refuse_root_collisions_deterministically() {
    for (key, root) in [
        ("a~b", "AB"),
        ("node/name", "NodeName"),
        ("__item__", "Item"),
        ("café", "Caf"),
        ("foo.$bar", "FooBar"),
    ] {
        let reference = key.replace('~', "~0").replace('/', "~1");
        let source = json!({
            "$id": "urn:example:adversary:bindings",
            "$defs": {
                "other": {"type": "number"},
                key: {"type": "string"},
                "z-final": {"type": "boolean"}
            },
            "$ref": format!("#/$defs/{reference}")
        });
        let before = serde_json::to_vec(&source).unwrap();
        let first = project(&source, root).expect_err("normalized root collision");
        assert!(
            matches!(&first, ProjectionError::InvalidShape { pointer, message }
            if pointer == &format!("/$defs/{reference}") && message.contains(root))
        );
        assert_eq!(project(&source, root), Err(first));
        assert_eq!(serde_json::to_vec(&source).unwrap(), before);
    }
}

#[test]
fn array_helpers_in_alternatives_and_unused_definitions_cannot_be_shadowed() {
    for definition in [
        json!({"type": ["null", "array"], "items": {"type": "string"}}),
        json!({"type": "object", "additionalProperties": false,
        "properties": {"children": {"type": "array", "items": {
            "type": "array", "items": {"type": "number"}
        }}}}),
    ] {
        let source = json!({
            "$id": "urn:example:adversary:array",
            "$defs": {"unused": definition},
            "type": "string"
        });
        let first = project(&source, "Array").expect_err("root shadows a rendered helper");
        assert_eq!(project(&source, "Array"), Err(first));
        let mut named = source;
        named["$defs"]["_array_"] = json!({"type": "string"});
        let first = project(&named, "Root").expect_err("normalized definition shadows helper");
        assert_eq!(project(&named, "Root"), Err(first));
    }
}

#[test]
fn property_names_and_unemitted_items_do_not_reserve_bindings() {
    let source = json!({
        "$id": "urn:example:adversary:properties",
        "$defs": {"array": {"type": "string", "items": {
            "type": "array", "items": {"type": "string"}
        }}},
        "type": "object",
        "additionalProperties": false,
        "properties": {
            "Array": {"$ref": "#/$defs/array"},
            "await": {"type": "string"},
            "constructor": {"type": "string"},
            "foo/bar": {"type": "string"}
        }
    });
    let before = source.clone();
    let rendered = project(&source, "namespace").expect("unused items do not emit a helper");
    assert!(rendered.contains("export type Array = string;\n"));
    assert!(rendered.contains("  Array?: Array;\n"));
    assert!(rendered.contains("  await?: string;\n"));
    assert!(rendered.contains("  constructor?: string;\n"));
    assert!(rendered.contains("  \"foo/bar\"?: string;\n"));
    assert_eq!(source, before);
    assert_eq!(project(&source, "namespace").unwrap(), rendered);
}
