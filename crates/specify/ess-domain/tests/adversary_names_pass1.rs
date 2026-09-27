//! Adversary pass 1 against `story:field-names-underscore-and-newtype-map-keys`.
//!
//! The unit changed the field-name rule (beyond10x/ess#141) at `Field::PATTERN` and the three
//! `schemars(regex)` attributes it knew of. `deserialize_field_name` is the parser behind more
//! members than those: a relation's `name:` and `via:` are read by it too, so they now admit `_owner`
//! while the committed document schema still publishes the old pattern for them. A schema that
//! refuses what the parser reads is an editor telling an author a valid document is wrong.

use ess_domain::entity::RelationSpec;
use serde_json::{json, Value};

fn document_schema() -> Value {
    let text = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../schemas/generated/ess.schema.json"),
    )
    .expect("the generated schema is committed");
    serde_json::from_str(&text).expect("the generated schema is JSON")
}

fn validator(definition: &str) -> jsonschema::Validator {
    let schema = document_schema();
    assert!(
        !schema["definitions"][definition].is_null(),
        "the document schema carries `{definition}`"
    );
    jsonschema::validator_for(&json!({
        "$schema": schema["$schema"],
        "definitions": schema["definitions"],
        "$ref": format!("#/definitions/{definition}"),
    }))
    .expect("a usable schema")
}

const RELATION: &str =
    "{name: _clients, kind: owns, target: crm.core.Client, cardinality: many, via: _parent}";

#[test]
fn a_relation_the_parser_reads_with_underscore_names_is_one_the_published_schema_admits() {
    let parsed: RelationSpec =
        serde_yaml::from_str(RELATION).expect("the parser admits `_clients` and `_parent`");
    assert_eq!(parsed.name, "_clients");
    assert_eq!(parsed.via, "_parent");

    let document: Value = serde_yaml::from_str(RELATION).unwrap();
    let validator = validator("RelationSpec");
    let errors = validator
        .iter_errors(&document)
        .map(|error| error.to_string())
        .collect::<Vec<_>>();
    assert!(
        errors.is_empty(),
        "the parser reads this relation and the committed schema refuses it: {errors:?}"
    );
}

#[test]
fn no_published_pattern_is_the_field_name_rule_from_before_141() {
    fn walk(value: &Value, path: &str, found: &mut Vec<String>) {
        match value {
            Value::Object(map) => {
                for (key, inner) in map {
                    if key == "pattern" && inner == "^[A-Za-z][A-Za-z0-9_]*$" {
                        found.push(path.to_owned());
                    }
                    walk(inner, &format!("{path}/{key}"), found);
                }
            }
            Value::Array(items) => {
                for (index, inner) in items.iter().enumerate() {
                    walk(inner, &format!("{path}/{index}"), found);
                }
            }
            _ => {}
        }
    }
    let mut found = Vec::new();
    walk(&document_schema(), "#", &mut found);
    assert!(
        found.is_empty(),
        "these members are read by `deserialize_field_name` (or claim to be field names) and \
         still publish the pre-#141 pattern: {found:?}"
    );
}

// ---- #143 boundaries the unit's cases do not name ---------------------------------------------

use ess_domain::types::{MapKeyNewtypes, Primitive, TypeRef};

fn keys(types: &str) -> MapKeyNewtypes {
    MapKeyNewtypes::from_documents([format!("types:\n{types}").as_str()])
}

#[test]
fn a_key_newtype_of_an_optional_an_enum_or_a_map_is_refused() {
    for (types, why) in [
        (
            "  - {name: demo.o.Id, kind: newtype, of: \"Optional<String>\"}\n",
            "an optional has a null that no key can spell",
        ),
        (
            "  - {name: demo.o.Id, kind: newtype, of: demo.o.Kind}\n  - {name: demo.o.Kind, kind: enum, variants: [a, b]}\n",
            "a newtype of an enum is not a newtype of a primitive",
        ),
        (
            "  - {name: demo.o.Id, kind: newtype, of: \"Map<String, String>\"}\n",
            "a map is not a key",
        ),
        (
            "  - {name: demo.o.Id, kind: newtype, of: demo.o.Raw}\n  - {name: demo.o.Raw, kind: newtype, of: Binary64}\n",
            "Binary64 through a chain is still Binary64",
        ),
    ] {
        let scope = keys(types);
        scope
            .scope(|| TypeRef::parse("Map<demo.o.Id, Boolean>"))
            .expect_err(why);
    }
}

#[test]
fn a_key_newtype_chain_as_long_as_the_declarations_resolves() {
    // Five declarations in a straight line: the bound in `get` must admit exactly this many steps.
    let types = "  - {name: demo.o.A, kind: newtype, of: demo.o.B}\n  - {name: demo.o.B, kind: newtype, of: demo.o.C}\n  - {name: demo.o.C, kind: newtype, of: demo.o.D}\n  - {name: demo.o.D, kind: newtype, of: demo.o.E}\n  - {name: demo.o.E, kind: newtype, of: Uuid}\n";
    assert_eq!(
        keys(types).get(&"demo.o.A".parse().unwrap()),
        Some(Primitive::Uuid)
    );
}

#[test]
fn a_key_newtype_nested_inside_other_generics_resolves() {
    let scope = keys("  - {name: demo.o.Id, kind: newtype, of: String}\n");
    assert_eq!(
        scope.scope(|| TypeRef::parse("Optional<List<Map<demo.o.Id, Map<demo.o.Id, Integer>>>>")),
        TypeRef::parse("Optional<List<Map<String, Map<String, Integer>>>>")
    );
}
