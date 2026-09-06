use schema_contract::bundle::{import, Dialect};
use schema_contract::realize::normalize::{Plan, Root};
use serde_json::{json, Value};
use std::collections::BTreeSet;

fn input(key: &str) -> Value {
    json!({"op":"read", "scope":"input", "path":[key]})
}

fn item(key: &str) -> Value {
    json!({"op":"read", "scope":"item", "path":[key]})
}

fn integer(value: i64) -> Value {
    json!({"op":"integer", "value":value})
}

fn text(value: &str) -> Value {
    json!({"op":"string", "value":value})
}

pub fn plan() -> Plan {
    let source = json!({"components":{"schemas":{
        "Input":{"type":"object", "additionalProperties":false, "required":["seconds", "address", "items"], "properties":{
            "seconds":{"type":"integer"}, "address":{"type":"string"}, "optional":{"type":["string", "null"]},
            "nested":{"type":["object", "null"], "required":["label"], "properties":{"label":{"type":"string"}}, "additionalProperties":false},
            "items":{"type":"array", "items":{"type":"object", "required":["key", "enabled"],
                "properties":{"key":{"type":"string"}, "enabled":{"type":"boolean"}}, "additionalProperties":false}}}},
        "Output":{"type":"object", "additionalProperties":false, "required":["ns", "label", "count", "keys", "kind", "nil", "flag", "manual"], "properties":{
            "ns":{"type":"integer"}, "label":{"type":"string"}, "count":{"type":"integer"}, "keys":{"type":"array", "items":{"type":"string"}},
            "kind":{"type":"string"}, "nil":{"type":"null"}, "flag":{"type":"boolean"}, "manual":{"type":"array", "items":{"type":"integer"}},
            "optional":{"type":["string", "null"]}}},
        "Integer":{"type":"integer"}, "Bounded":{"type":"integer", "minimum":1, "maximum":10}, "Any":true,
        "Format":{"type":"string", "format":"email"}
    }}});
    let roots = ["Input", "Output", "Integer", "Bounded", "Any", "Format"]
        .into_iter()
        .map(str::to_owned)
        .collect::<BTreeSet<_>>();
    let bundle = import(&source.to_string(), &roots, Dialect::Draft202012).unwrap();
    let stage = |input: &str, output: &str, value: &Value| {
        json!({
            "input":Root::pin(&bundle, input).unwrap(), "output":Root::pin(&bundle, output).unwrap(), "requires":[], "value":value
        })
    };
    let record = json!({"op":"record", "fields":{
        "ns":{"op":"arithmetic", "operation":"multiply", "overflow":"wrap", "left":input("seconds"), "right":integer(1_000_000_000)},
        "label":{"op":"field", "name":"label", "object":{"op":"fallback", "value":input("nested"),
            "on_null":true, "fallback":{"op":"record", "fields":{"label":text("seed")}}}},
        "optional":input("optional"), "nil":{"op":"null"}, "flag":{"op":"boolean", "value":false},
        "manual":{"op":"list", "items":[integer(1),integer(2)]},
        "keys":{"op":"map", "list":input("items"), "value":item("key")},
        "count":{"op":"arithmetic", "operation":"add", "overflow":"reject", "left":integer(1), "right":{
            "op":"distinct_count", "list":input("items"), "key":item("key"),
            "condition":{"op":"boolean", "value":item("enabled")}}},
        "kind":{"op":"choose", "condition":{"op":"starts_with", "value":input("address"), "prefix":text("proto")},
            "then_value":text("uri"), "else_value":text("number")}
    }});
    let whole = json!({"op":"read", "scope":"input", "path":[]});
    let mut first = stage("Input", "Output", &record);
    first["requires"] = json!([{"op":"not", "condition":{"op":"all", "conditions":[
        {"op":"present", "value":input("optional")},
        {"op":"equal", "left":input("optional"), "right":text("deny")}
    ]}}]);
    let mut second = stage("Output", "Output", &whole);
    second["requires"] = json!([{"op":"greater", "left":input("ns"), "right":integer(0)}]);
    let overflow = json!({"op":"arithmetic", "operation":"add", "overflow":"reject", "left":whole, "right":integer(1)});
    let lazy = json!({"op":"choose", "condition":{"op":"any", "conditions":[
        {"op":"all", "conditions":[]}, {"op":"greater", "left":overflow, "right":integer(0)}]},
        "then_value":whole, "else_value":overflow});
    let recipe = json!({"format":"ess-normalization/1", "branches":{
        "primary":[first, second], "alias":[first, second], "scalar":[stage("Integer", "Integer", &overflow)],
        "bounded":[stage("Integer", "Bounded", &whole)], "copy":[stage("Any", "Any", &whole)],
        "lazy":[stage("Integer", "Integer", &lazy)], "format":[stage("Format", "Format", &whole)],
        "odd\"\nbranch/segment~":[stage("Any", "Any", &whole)]
    }});
    Plan::read(&recipe.to_string(), &[bundle]).unwrap()
}

pub fn cases(plan: &Plan) -> Value {
    let primary = json!({"seconds":2, "address":"protocol", "items":[
        {"key":"second", "enabled":true}, {"key":"first", "enabled":false},
        {"key":"second", "enabled":true}, {"key":"third", "enabled":true}]})
    .to_string();
    let mut cases = vec![("primary", primary.clone()), ("alias", primary)];
    for input in [
        r#"{"seconds":1,"address":"Proto:x","items":[],"nested":null,"optional":null}"#,
        r#"{"seconds":1,"address":"other","items":[],"nested":{"label":""},"optional":""}"#,
        r#"{"seconds":1,"address":"other","items":[],"optional":"deny"}"#,
        r#"{"seconds":0,"address":"other","items":[]}"#,
        r#"{"seconds":9223372036854775807,"address":"other","items":[]}"#,
        r#"{"seconds":1.0,"address":"other","items":[]}"#,
        r#"{"seconds":"wrong","address":"other","items":[]}"#,
        r#"{"seconds":1,"address":"other","items":[{"key":"x","enabled":null}]}"#,
    ] {
        cases.push(("primary", input.to_owned()));
    }
    for (branch, input) in [
        ("scalar", "9223372036854775807"),
        ("scalar", "-9223372036854775808"),
        ("scalar", "-0"),
        ("scalar", "1e0"),
        ("scalar", "18446744073709551615"),
        ("lazy", "9223372036854775807"),
        ("bounded", "0"),
        ("bounded", "11"),
        ("bounded", "10"),
        ("format", r#""not an email""#),
        ("copy", "0.125"),
        ("copy", "0.1"),
        ("copy", "1e20"),
        ("copy", "-0.0"),
        ("copy", "18446744073709551615"),
        ("copy", "18446744073709551617"),
        ("copy", "0.10000000000000001"),
        ("copy", "1e-999"),
        ("copy", "1e999"),
        ("copy", r#"{"x":1,"x":2}"#),
        ("copy", "true false"),
        ("unknown", "{}"),
        ("odd\"\nbranch/segment~", "null"),
    ] {
        cases.push((branch, input.to_owned()));
    }
    Value::Array(
        cases
            .into_iter()
            .map(|(branch, input)| {
                let result = plan.run_json(branch, &input);
                match result {
                    Ok(value) => json!({"branch":branch, "input":input, "value":value}),
                    Err(error) => json!({"branch":branch, "input":input, "errors":error.0}),
                }
            })
            .collect(),
    )
}
