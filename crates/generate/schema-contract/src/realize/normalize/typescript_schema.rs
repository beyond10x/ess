//! Closed source-schema qualification and lossless private validator bindings.

use std::collections::BTreeSet;

use serde_json::{json, Value};

use super::{finding, path, Finding, Plan, Refused, Root};

pub(super) const BASE64_PATTERN: &str =
    "^(?:[A-Za-z0-9+/]{4})*(?:[A-Za-z0-9+/]{2}==|[A-Za-z0-9+/]{3}=)?$";

pub(super) fn qualify(plan: &Plan) -> Result<(), Refused> {
    let roots = plan
        .recipe
        .branches
        .values()
        .flatten()
        .flat_map(|stage| [&stage.input, &stage.output])
        .collect::<BTreeSet<_>>();
    let mut found = BTreeSet::new();
    for root in roots {
        match root {
            Root::Bundle {
                bundle_digest,
                root,
            } => {
                let bundle = &plan.bundles[bundle_digest];
                let definitions = bundle
                    .root_definitions(root)
                    .expect("checked selected root");
                for (name, schema) in definitions {
                    visit_schema(
                        schema,
                        &format!("/bundles/{bundle_digest}{}", bundle.source_pointer(name)),
                        &mut found,
                    );
                }
            }
            Root::Model { model, .. } => {
                for (name, schema) in plan.models[model].definitions() {
                    visit_schema(
                        schema,
                        &path(&format!("/models/{}/$defs", model.projection_digest), name),
                        &mut found,
                    );
                }
            }
        }
    }
    if found.is_empty() {
        Ok(())
    } else {
        Err(Refused(found.into_iter().collect()))
    }
}

fn visit_schema(schema: &Value, at: &str, found: &mut BTreeSet<Finding>) {
    let Some(object) = schema.as_object() else {
        return;
    };
    for (key, value) in object {
        let location = path(at, key);
        match key.as_str() {
            "$ref"
                if value
                    .as_str()
                    .and_then(crate::realize::local_definition)
                    .is_some() => {}
            "type" | "required" | "enum" | "const" | "minimum" | "maximum" | "exclusiveMinimum"
            | "exclusiveMaximum" | "minLength" | "maxLength" | "minItems" | "maxItems"
            | "$schema" | "$id" | "$comment" | "title" | "description" | "default" | "examples"
            | "deprecated" | "readOnly" | "writeOnly" | "format" | "contentEncoding"
            | "contentMediaType" | "x-ess-name" | "x-ess-kind" | "x-ess-field"
            | "x-ess-map-key" | "x-ess-union-tag" | "x-ess-provenance" => {}
            "pattern" if value.as_str() == Some(BASE64_PATTERN) => {}
            "uniqueItems" if value == &Value::Bool(false) => {}
            "properties" | "$defs" => {
                if let Some(children) = value.as_object() {
                    for (name, child) in children {
                        visit_schema(child, &path(&location, name), found);
                    }
                }
            }
            "additionalProperties" | "propertyNames" | "items" => {
                visit_schema(value, &location, found);
            }
            "prefixItems" | "anyOf" | "oneOf" | "allOf" => {
                if let Some(children) = value.as_array() {
                    for (index, child) in children.iter().enumerate() {
                        visit_schema(child, &format!("{location}/{index}"), found);
                    }
                }
            }
            _ => {
                found.insert(finding(
                    &location,
                    "typescript_schema_profile",
                    "constraint is outside the qualified TypeScript source-schema profile",
                ));
            }
        }
    }
}

pub(super) fn profile() -> String {
    format!("{}\n", serde_json::to_string_pretty(&json!({
        "dialect":"https://json-schema.org/draft/2020-12/schema",
        "validation":["boolean","$ref","type","const","enum","properties","required","additionalProperties","propertyNames","items","prefixItems","minimum","maximum","exclusiveMinimum","exclusiveMaximum","minLength","maxLength","minItems","maxItems","anyOf","oneOf","allOf"],
        "pattern":BASE64_PATTERN,
        "annotations":["$schema","$id","$comment","title","description","default","examples","deprecated","readOnly","writeOnly","format","contentEncoding","contentMediaType","x-ess-name","x-ess-kind","x-ess-field","x-ess-map-key","x-ess-union-tag","x-ess-provenance"],
        "references":"checked local definition references; conjunctive siblings",
        "source_admission":"existing checked Plan; this keyword list does not broaden normalization shape admission",
        "uniqueItems":"false only; true refuses during generation",
        "other_constraints":"refuse during generation at the source-qualified pointer",
        "numeric_comparison":"exact represented bigint and finite binary64 values",
        "diagnostics":"complete reference multiset sorted by Unicode-scalar instance pointer"
    })).expect("fixed profile serializes"))
}

pub(super) fn quote(text: &str) -> String {
    serde_json::to_string(text).expect("string serializes")
}

pub(super) fn value_literal(value: &Value) -> String {
    match value {
        Value::Number(number) if number.is_f64() => {
            format!(
                "fromBits(0x{:016x}n)",
                number.as_f64().expect("finite source number").to_bits()
            )
        }
        Value::Number(number) => format!("{number}n"),
        Value::Array(items) => format!(
            "[{}]",
            items
                .iter()
                .map(value_literal)
                .collect::<Vec<_>>()
                .join(",")
        ),
        Value::Object(fields) => format!(
            "new Map<string, Value>([{}])",
            fields
                .iter()
                .map(|(key, value)| format!("[{},{}]", quote(key), value_literal(value)))
                .collect::<Vec<_>>()
                .join(",")
        ),
        _ => value.to_string(),
    }
}

pub(super) fn lower_schema(schema: &Value) -> String {
    let Some(object) = schema.as_object() else {
        return schema.to_string();
    };
    let mut fields = Vec::new();
    for (key, value) in object {
        let (name, lowered) = match key.as_str() {
            "$ref" => (
                "ref",
                quote(
                    &crate::realize::local_definition(value.as_str().expect("reference string"))
                        .expect("qualified reference"),
                ),
            ),
            "type" => (
                "types",
                if value.is_array() {
                    value.to_string()
                } else {
                    format!("[{value}]")
                },
            ),
            "const" => ("constant", value_literal(value)),
            "enum" => ("enumeration", value_literal(value)),
            "required" => ("required", value.to_string()),
            "properties" => (
                "properties",
                format!(
                    "new Map<string, Schema>([{}])",
                    value
                        .as_object()
                        .expect("properties object")
                        .iter()
                        .map(|(key, value)| format!("[{},{}]", quote(key), lower_schema(value)))
                        .collect::<Vec<_>>()
                        .join(",")
                ),
            ),
            "additionalProperties" => ("additional", lower_schema(value)),
            "propertyNames" => ("names", lower_schema(value)),
            "items" => ("items", lower_schema(value)),
            "prefixItems" | "anyOf" | "oneOf" | "allOf" => (
                if key == "prefixItems" {
                    "prefix"
                } else {
                    key.as_str()
                },
                format!(
                    "[{}]",
                    value
                        .as_array()
                        .expect("schema list")
                        .iter()
                        .map(lower_schema)
                        .collect::<Vec<_>>()
                        .join(",")
                ),
            ),
            "minimum" | "maximum" | "exclusiveMinimum" | "exclusiveMaximum" | "minLength"
            | "maxLength" | "minItems" | "maxItems" => (key.as_str(), value_literal(value)),
            "pattern" => ("base64", "true".to_owned()),
            _ => continue, // Qualified annotations and schema envelopes have no validation action.
        };
        fields.push(format!("{name}:{lowered}"));
    }
    format!("{{{}}}", fields.join(","))
}

pub(super) fn root_binding(document: &Value) -> String {
    let definitions = document["$defs"]
        .as_object()
        .expect("retained schema definitions");
    format!(
        "{{root:{},definitions:new Map<string, Schema>([{}])}}",
        lower_schema(document),
        definitions
            .iter()
            .map(|(name, schema)| format!("[{},{}]", quote(name), lower_schema(schema)))
            .collect::<Vec<_>>()
            .join(",")
    )
}
