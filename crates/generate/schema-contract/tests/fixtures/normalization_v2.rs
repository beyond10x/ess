use schema_contract::bundle::{import, Bundle, Dialect};
use schema_contract::realize::normalize::{Root, FORMAT_V2};
use serde_json::{json, Value};

pub fn fixture() -> (Bundle, Value) {
    let source = json!({"components":{"schemas":{
        "Input":{"type":"object", "additionalProperties":false, "required":["prefix","items","fallback"], "properties":{
            "prefix":{"type":"string"}, "fallback":{"type":"integer"}, "optional":{"type":"string"},
            "items":{"type":"array", "items":{"type":"object", "additionalProperties":false,
                "required":["key","enabled","n","children"], "properties":{
                    "key":{"type":"string"}, "enabled":{"type":"boolean"}, "n":{"type":"integer"},
                    "children":{"type":"array", "items":{"type":"integer"}}}}}}},
        "Output":{"type":"object", "additionalProperties":false, "required":["labels","joined","nested"], "properties":{
            "labels":{"type":"array", "items":{"type":"string"}}, "joined":{"type":"string"},
            "nested":{"type":"array", "items":{"type":"array", "items":{"type":"integer"}}}}},
        "Integer":{"type":"integer"}, "String":{"type":"string"},
        "Integers":{"type":"array", "items":{"type":"integer"}}
    }}});
    let roots = ["Input", "Output", "Integer", "String", "Integers"]
        .map(str::to_owned)
        .into_iter()
        .collect();
    let bundle = import(&source.to_string(), &roots, Dialect::Draft202012).unwrap();
    let read = |scope: &str, path: &[&str]| json!({"op":"read", "scope":scope, "path":path});
    let list = read("input", &["items"]);
    let enabled = json!({"op":"boolean", "value":read("item", &["enabled"])});
    let index = json!({"op":"item_index"});
    let text = |value: &str| json!({"op":"string", "value":value});
    let record = json!({"op":"record", "fields":{
        "labels":{"op":"concat_lists", "lists":[
            {"op":"list", "items":[read("input", &["prefix"])]},
            {"op":"select_map", "list":list, "condition":enabled, "value":{
                "op":"concat", "parts":[text("item-"), {"op":"integer_string", "value":index},
                    text(":"), read("item", &["key"])]}}]},
        "joined":{"op":"join", "list":{"op":"map", "list":list, "value":read("item", &["key"])},
            "separator":text("|")},
        "nested":{"op":"map", "list":list, "value":{"op":"concat_lists", "lists":[
            {"op":"list", "items":[index]},
            {"op":"map", "list":read("item", &["children"]), "value":index},
            {"op":"list", "items":[index]}]}}
    }});
    let add = json!({"op":"arithmetic", "operation":"add", "overflow":"reject",
        "left":read("item", &["n"]), "right":{"op":"integer", "value":1}});
    let selected = json!({"op":"select_map", "list":list, "condition":enabled, "value":add});
    let found = json!({"op":"find", "list":list, "condition":enabled, "value":add,
        "otherwise":read("input", &["fallback"])});
    let outer = json!({"op":"map", "list":list, "value":{"op":"find",
        "list":read("item", &["children"]), "condition":{"op":"any", "conditions":[]},
        "value":index, "otherwise":index}});
    let stage = |input: &str, output: &str, value: Value| {
        json!({
            "input":Root::pin(&bundle, input).unwrap(), "output":Root::pin(&bundle, output).unwrap(),
            "requires":[], "value":value
        })
    };
    let recipe = json!({"format":FORMAT_V2, "branches":{
        "primary":[stage("Input", "Output", record)],
        "select":[stage("Input", "Integers", selected)],
        "find":[stage("Input", "Integer", found)],
        "outer":[stage("Input", "Integers", outer)],
        "integer":[stage("Integer", "String", json!({"op":"integer_string", "value":read("input", &[])}))],
        "empty_text":[stage("Integer", "String", json!({"op":"concat", "parts":[]}))],
        "empty_lists":[stage("Integer", "Integers", json!({"op":"concat_lists", "lists":[]}))],
        "empty_join":[stage("Integer", "String", json!({"op":"join", "list":{"op":"list","items":[]}, "separator":text("|")}))]
    }});
    (bundle, recipe)
}

pub const INPUT: &str = r#"{"prefix":"start","fallback":-1,"items":[{"key":"z","enabled":true,"n":2,"children":[4,5]},{"key":"","enabled":false,"n":9223372036854775807,"children":[]},{"key":"z","enabled":true,"n":4,"children":[7]}]}"#;
pub const EMPTY: &str = r#"{"prefix":"","fallback":-7,"items":[]}"#;

pub fn expected() -> Vec<(&'static str, &'static str, Value)> {
    vec![
        (
            "primary",
            INPUT,
            json!({"labels":["start","item-0:z","item-2:z"], "joined":"z||z", "nested":[[0,0,1,0],[1,1],[2,0,2]]}),
        ),
        (
            "primary",
            EMPTY,
            json!({"labels":[""], "joined":"", "nested":[]}),
        ),
        ("select", INPUT, json!([3, 5])),
        ("find", INPUT, json!(3)),
        ("find", EMPTY, json!(-7)),
        ("outer", INPUT, json!([0, 1, 2])),
        (
            "integer",
            "-9223372036854775808",
            json!("-9223372036854775808"),
        ),
        (
            "integer",
            "9223372036854775807",
            json!("9223372036854775807"),
        ),
        ("integer", "-0", json!("0")),
        ("empty_text", "0", json!("")),
        ("empty_lists", "0", json!([])),
        ("empty_join", "0", json!("")),
    ]
}
