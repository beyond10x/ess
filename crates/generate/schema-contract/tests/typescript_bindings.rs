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

#[path = "support/empty_struct.rs"]
mod empty_struct;

/// The declaration and report an open `fields: []` struct realizes as (ess/32, beyond10x/ess#649):
/// an object type with a string index, so `{}` and `{"x": 1}` are both values of it.
#[test]
fn an_open_empty_struct_decodes_any_object() {
    let result = empty_struct::plan().typescript();
    let declarations = &result.declarations;
    assert!(
        declarations.contains("export type DemoAckOpen = ({\n} & { [key: string]: "),
        "{declarations}"
    );
}

/// A closed `fields: []` struct is the object every key of which is `never`, so no string, number
/// or array is one. A decoded value reaches the type by a cast, so the report still names the
/// runtime check of its keys as the `closed_object` obligation.
#[test]
fn a_closed_empty_struct_decodes_only_the_empty_object() {
    let result = empty_struct::plan().typescript();
    let declarations = &result.declarations;
    assert!(
        declarations.contains("export type DemoAckClosed = Record<string, never>;"),
        "{declarations}"
    );
    let report = serde_json::to_value(&result.report).unwrap();
    assert!(
        report["obligations"]
            .as_array()
            .unwrap()
            .iter()
            .any(|entry| entry["rule"] == "closed_object"
                && entry["pointer"]
                    .as_str()
                    .is_some_and(|pointer| pointer.contains("demo.ack.Closed"))),
        "{report}"
    );
}
