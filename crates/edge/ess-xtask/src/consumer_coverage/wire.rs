//! Closed Draft 7 structural inventory; annotations are removed only at schema positions.
use anyhow::{bail, Context, Result};
use serde_json::{json, Map, Value};
use std::collections::BTreeMap;

pub(super) fn extract(root: &Value) -> Result<Value> {
    let canonical = schema(root, "")?;
    let mut obligations = BTreeMap::new();
    let mut references = Vec::new();
    inventory(&canonical, "", &mut obligations);
    refs(&canonical, "", &canonical, &mut references)?;
    Ok(json!({"obligations":obligations,"references":references}))
}
fn escape(s: &str) -> String {
    s.replace('~', "~0").replace('/', "~1")
}
fn schema(value: &Value, path: &str) -> Result<Value> {
    if value.is_boolean() {
        return Ok(value.clone());
    }
    let object = value
        .as_object()
        .with_context(|| format!("wire {path}: expected schema object or boolean"))?;
    let mut result = Map::new();
    for (key, v) in object {
        let at = format!("{path}/{}", escape(key));
        let next = match key.as_str() {
            "description" | "title" | "examples" | "$comment" => continue,
            "$schema" => {
                if v != "http://json-schema.org/draft-07/schema#" {
                    bail!("wire {at}: unsupported schema dialect");
                }
                v.clone()
            }
            "$id" | "$ref" | "format" | "pattern" => {
                if !v.is_string() {
                    bail!("wire {at}: expected string");
                }
                v.clone()
            }
            "definitions" | "properties" | "patternProperties" => {
                let map = v
                    .as_object()
                    .with_context(|| format!("wire {at}: expected schema map"))?;
                let mut out = Map::new();
                for (name, child) in map {
                    out.insert(
                        name.clone(),
                        schema(child, &format!("{at}/{}", escape(name)))?,
                    );
                }
                Value::Object(out)
            }
            "additionalProperties"
            | "additionalItems"
            | "contains"
            | "propertyNames"
            | "not"
            | "if"
            | "then"
            | "else" => schema(v, &at)?,
            "items" => {
                if let Some(items) = v.as_array() {
                    Value::Array(
                        items
                            .iter()
                            .enumerate()
                            .map(|(i, x)| schema(x, &format!("{at}/{i}")))
                            .collect::<Result<_>>()?,
                    )
                } else {
                    schema(v, &at)?
                }
            }
            "allOf" | "anyOf" | "oneOf" => {
                let items = v
                    .as_array()
                    .filter(|x| !x.is_empty())
                    .with_context(|| format!("wire {at}: expected nonempty schema array"))?;
                Value::Array(
                    items
                        .iter()
                        .enumerate()
                        .map(|(i, x)| schema(x, &format!("{at}/{i}")))
                        .collect::<Result<_>>()?,
                )
            }
            "dependencies" => {
                let map = v
                    .as_object()
                    .with_context(|| format!("wire {at}: expected dependency map"))?;
                let mut out = Map::new();
                for (name, x) in map {
                    let x = if x.as_array().is_some_and(|a| a.iter().all(Value::is_string)) {
                        x.clone()
                    } else {
                        schema(x, &format!("{at}/{}", escape(name)))?
                    };
                    out.insert(name.clone(), x);
                }
                Value::Object(out)
            }
            _ => scalar_keyword(key, v, &at)?,
        };
        result.insert(key.clone(), next);
    }
    Ok(Value::Object(result))
}
fn scalar_keyword(key: &str, v: &Value, at: &str) -> Result<Value> {
    Ok(match key {
        "type" => {
            let valid = |x: &Value| {
                matches!(
                    x.as_str(),
                    Some("null" | "boolean" | "object" | "array" | "number" | "string" | "integer")
                )
            };
            if !(valid(v)
                || v.as_array()
                    .is_some_and(|x| !x.is_empty() && x.iter().all(valid)))
            {
                bail!("wire {at}: unknown type");
            }
            v.clone()
        }
        "required" => {
            if !v.as_array().is_some_and(|a| a.iter().all(Value::is_string)) {
                bail!("wire {at}: expected names");
            }
            v.clone()
        }
        "minimum" | "maximum" | "exclusiveMinimum" | "exclusiveMaximum" | "multipleOf" => {
            if !v.is_number() {
                bail!("wire {at}: expected number");
            }
            v.clone()
        }
        "minLength" | "maxLength" | "minItems" | "maxItems" | "minProperties" | "maxProperties" => {
            if !v.is_u64() {
                bail!("wire {at}: expected unsigned integer");
            }
            v.clone()
        }
        "uniqueItems" | "readOnly" | "writeOnly" => {
            if !v.is_boolean() {
                bail!("wire {at}: expected boolean");
            }
            v.clone()
        }
        "enum" => {
            if v.as_array().is_none_or(Vec::is_empty) {
                bail!("wire {at}: expected nonempty literal array");
            }
            v.clone()
        }
        "const" | "default" => v.clone(),

        _ => bail!("wire {at}: unknown schema keyword"),
    })
}
fn inventory(v: &Value, path: &str, out: &mut BTreeMap<String, String>) {
    out.insert(format!("wire:RawSpecFile#{path}"), super::hash_json(v));
    match v {
        Value::Object(m) => {
            for (key, x) in m {
                inventory(x, &format!("{path}/{}", escape(key)), out);
            }
        }
        Value::Array(a) => {
            for (i, x) in a.iter().enumerate() {
                inventory(x, &format!("{path}/{i}"), out);
            }
        }
        _ => {}
    }
}
// Walk schema positions only: a literal object's "$ref" is data, never a reference.
fn refs(v: &Value, path: &str, root: &Value, out: &mut Vec<Value>) -> Result<()> {
    let Some(m) = v.as_object() else {
        return Ok(());
    };
    for (key, x) in m {
        let at = format!("{path}/{}", escape(key));
        match key.as_str() {
            "$ref" => {
                let r = x.as_str().context("reference must be a string")?;
                let pointer = r
                    .strip_prefix("#/definitions/")
                    .filter(|s| !s.is_empty() && !s.contains('/'))
                    .with_context(|| format!("wire {at}: unsupported reference {r}"))?;
                let resolved = root
                    .pointer(&format!("/definitions/{pointer}"))
                    .with_context(|| format!("wire {at}: unresolved reference {r}"))?;
                if !resolved.is_object() && !resolved.is_boolean() {
                    bail!("wire {at}: reference target is not a schema");
                }
                out.push(json!({"source":format!("wire:RawSpecFile#{at}"),"target":format!("wire:RawSpecFile{}",r)}));
            }
            "properties" | "patternProperties" | "definitions" => {
                for (n, s) in x.as_object().context("schema map")? {
                    refs(s, &format!("{at}/{}", escape(n)), root, out)?;
                }
            }
            "dependencies" => {
                for (n, s) in x.as_object().context("dependency map")? {
                    if !s.is_array() {
                        refs(s, &format!("{at}/{}", escape(n)), root, out)?;
                    }
                }
            }
            "allOf" | "anyOf" | "oneOf" => {
                for (i, s) in x.as_array().context("schema array")?.iter().enumerate() {
                    refs(s, &format!("{at}/{i}"), root, out)?;
                }
            }
            "items" => {
                if let Some(a) = x.as_array() {
                    for (i, s) in a.iter().enumerate() {
                        refs(s, &format!("{at}/{i}"), root, out)?;
                    }
                } else {
                    refs(x, &at, root, out)?;
                }
            }
            "additionalProperties"
            | "additionalItems"
            | "contains"
            | "propertyNames"
            | "not"
            | "if"
            | "then"
            | "else" => refs(x, &at, root, out)?,
            _ => {}
        }
    }
    Ok(())
}
