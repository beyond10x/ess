use schema_contract::bundle::{import, Bundle, Dialect};
use schema_contract::realize::normalize::Root;
use serde_json::{json, Value};

pub fn fixture() -> (Bundle, Value) {
    let source = json!({"components":{"schemas":{
        "Number":{"type":"number"}, "Integer":{"type":"integer"},
        "Record":{"type":"object","additionalProperties":false,"properties":{
            "value":{"type":["number","null"]}, "exact":{"type":"number"}, "integer":{"type":"integer"},
            "rows":{"type":["array","null"],"items":{"type":["object","null"],"properties":{"value":{"type":["number","null"]}},"additionalProperties":false}}
        }}
    }}});
    let roots = ["Number", "Integer", "Record"]
        .into_iter()
        .map(str::to_owned)
        .collect();
    let bundle = import(&source.to_string(), &roots, Dialect::Draft202012).unwrap();
    let read = json!({"op":"read","scope":"input","path":[]});
    let converted = |steps: Value| json!({"op":"binary64_to_integer","value":read,"steps":steps,"out_of_range":"reject"});
    let scaled = |steps: Value| json!({"op":"arithmetic","operation":"multiply","overflow":"wrap","left":converted(steps),"right":{"op":"integer","value":1_000_000}});
    let stage = |input: &str, output: &str, value: Value| {
        json!([{
            "input":Root::pin(&bundle,input).unwrap(),"output":Root::pin(&bundle,output).unwrap(),"requires":[],"value":value
        }])
    };
    let recipe = json!({"format":"ess-normalization/2","branches":{
        "truncate":stage("Number","Integer",converted(json!([]))),
        "duration":stage("Number","Integer",scaled(json!([{"op":"multiply","value":"1000"}]))),
        "clamped":stage("Number","Integer",scaled(json!([{"op":"maximum","value":"3"},{"op":"multiply","value":"1000"}]))),
        "upper":stage("Number","Integer",converted(json!([{"op":"minimum","value":"10"}]))),
        "overflow":stage("Number","Integer",converted(json!([{"op":"multiply","value":"1e308"}]))),
        "copy":stage("Record","Record",read)
    },"binary64_inputs":{
        "truncate":[[]],"duration":[[]],"clamped":[[]],"upper":[[]],"overflow":[[]],
        "copy":[[{"kind":"field","name":"value"}],[{"kind":"field","name":"rows"},{"kind":"items"},{"kind":"field","name":"value"}]]
    }});
    (bundle, recipe)
}

pub fn expected() -> Vec<(&'static str, &'static str, Value)> {
    vec![
        ("truncate", "0", json!(0)),
        ("truncate", "-0", json!(0)),
        ("truncate", "1.9", json!(1)),
        ("truncate", "-1.9", json!(-1)),
        ("truncate", "1e0", json!(1)),
        ("truncate", "1e-999", json!(0)),
        ("truncate", "-1e-999", json!(0)),
        ("truncate", "5e-324", json!(0)),
        (
            "truncate",
            "9007199254740993",
            json!(9_007_199_254_740_992_i64),
        ),
        (
            "truncate",
            "9007199254740995",
            json!(9_007_199_254_740_996_i64),
        ),
        ("truncate", "-9223372036854775808", json!(i64::MIN)),
        (
            "truncate",
            "9223372036854774784",
            json!(9_223_372_036_854_774_784_i64),
        ),
        ("duration", "1.001", json!(1_000_000_000_i64)),
        ("duration", "-1.001", json!(-1_000_000_000_i64)),
        ("duration", "0.0019", json!(1_000_000)),
        ("duration", "-0.0019", json!(-1_000_000)),
        ("duration", "1e-999", json!(0)),
        // Go binary64 multiply rounds to 9223372036854773760 before the wrapping integer scale.
        ("duration", "9223372036854774", json!(-2_048_000_000_i64)),
        ("clamped", "-1.001", json!(3_000_000_000_i64)),
        ("clamped", "0", json!(3_000_000_000_i64)),
        ("clamped", "2.9999999999999999", json!(3_000_000_000_i64)),
        ("clamped", "4.25", json!(4_250_000_000_i64)),
        ("upper", "1e308", json!(10)),
        ("upper", "9.999", json!(9)),
        ("copy", "{}", json!({})),
        (
            "copy",
            r#"{"value":null,"rows":null}"#,
            json!({"value":null,"rows":null}),
        ),
        (
            "copy",
            r#"{"value":0.10000000000000001,"integer":9007199254740993}"#,
            json!({"value":0.1,"integer":9_007_199_254_740_993_i64}),
        ),
        (
            "copy",
            r#"{"rows":[null,{}, {"value":null}, {"value":0.10000000000000001}]}"#,
            json!({"rows":[null,{}, {"value":null}, {"value":0.1}]}),
        ),
    ]
}
