use base64::{engine::general_purpose::STANDARD, Engine as _};
use schema_contract::bundle::{import, Bundle, Dialect};
use schema_contract::realize::normalize::{Plan, Root};
use serde_json::{json, Value};

#[allow(dead_code)]
#[path = "normalization_binary64.rs"]
pub(super) mod binary64;

pub fn field(name: &str) -> Value {
    json!({"kind":"field","name":name})
}

pub fn policy(path: Value) -> Value {
    let mut policy = json!({"kind":"fixed_string_array","length":2,"missing":"preserve","null":"zero","short":"zero_pad","extra":"discard","null_element":"zero"});
    policy["path"] = path;
    policy
}

pub fn position(value: Value, index: u64) -> Value {
    let mut expression = json!({"op":"position","index":index});
    expression["value"] = value;
    expression
}

pub fn read(path: &[&str]) -> Value {
    json!({"op":"read","scope":"input","path":path})
}

pub fn fixture() -> (Bundle, Value) {
    let pair = json!({"type":"array","prefixItems":[{"type":"string"},{"type":"string"}],"items":false,"minItems":2,"maxItems":2});
    let triple = json!({"type":"array","prefixItems":[{"type":"string"},{"type":"string"},{"type":"string"}],"items":false,"minItems":3,"maxItems":3});
    let record = json!({"type":"object","additionalProperties":false,"properties":{
        "operands":pair,"wire/~":pair,"raw":{"type":"string"},"number":{"type":"number"},
        "groups":{"type":["array","null"],"items":{"type":["object","null"],"additionalProperties":false,"properties":{"pairs":{"type":["array","null"],"items":{"type":"object","additionalProperties":false,"properties":{"operands":pair}}}}}}
    }});
    let source = json!({"components":{"schemas":{
        "Pair":pair,"Record":record,
        "Required":{"type":"object","required":["operands"],"additionalProperties":false,"properties":{"operands":pair}},
        "Named":{"type":"object","required":["left","right"],"additionalProperties":false,"properties":{"left":{"type":"string"},"right":{"type":"string"}}},
        "Mixed":{"type":"array","prefixItems":[{"type":"string"},{"type":["boolean","null"]}],"items":false,"minItems":2,"maxItems":2},
        "NullableBool":{"type":["boolean","null"]},
        "Rows":{"type":"array","items":pair},"Strings":{"type":"array","items":{"type":"string"}},
        "Scopes":{"type":"object","additionalProperties":false,"required":["rows","fallback","matrix"],"properties":{"rows":{"type":"array","items":pair},"fallback":triple,"matrix":{"type":"array","items":{"type":"array","items":pair}}}},
        "Any":true
    }}});
    let roots = source["components"]["schemas"]
        .as_object()
        .unwrap()
        .keys()
        .cloned()
        .collect();
    let bundle = import(&source.to_string(), &roots, Dialect::Draft202012).unwrap();
    let stage = |input: &str, output: &str, value: Value| json!({"input":Root::pin(&bundle,input).unwrap(),"output":Root::pin(&bundle,output).unwrap(),"requires":[],"value":value});
    let slot = |index| json!({"op":"fallback","value":position(read(&["operands"]),index),"fallback":{"op":"string","value":""},"on_null":false});
    let named = json!({"op":"record","fields":{"left":slot(0),"right":slot(1)}});
    let record_policies = json!([
        policy(json!([field("operands")])),
        policy(json!([field("wire/~")])),
        policy(
            json!([field("groups"),{"kind":"items"},field("pairs"),{"kind":"items"},field("operands")])
        )
    ]);
    let item = |index| position(json!({"op":"read","scope":"item","path":[]}), index);
    let selected = json!({"op":"equal","left":item(0),"right":{"op":"string","value":"a"}});
    let scope_value = json!({"op":"record","fields":{
        "select/~":{"op":"select_map","list":read(&["rows"]),"condition":selected,"value":item(1)},
        "count":{"op":"distinct_count","list":read(&["rows"]),"condition":{"op":"present","value":item(0)},"key":item(1)},
        "find":{"op":"find","list":read(&["rows"]),"condition":selected,"value":item(1),"otherwise":position(read(&["fallback"]),2)},
        "nested":{"op":"map","list":read(&["matrix"]),"value":{"op":"map","list":{"op":"read","scope":"item","path":[]},"value":item(1)}}
    }});
    let mut scope_stage = stage("Scopes", "Any", scope_value);
    scope_stage["requires"] = json!([{"op":"present","value":position(read(&["fallback"]),0)}]);
    let mut triple_policy = policy(json!([field("fallback")]));
    triple_policy["length"] = json!(3);
    let recipe = json!({"format":"ess-normalization/6","branches":{
        "pair":[stage("Record","Named",named)],
        "root":[stage("Pair","Pair",read(&[]))],
        "record":[stage("Record","Record",read(&[])),stage("Record","Record",read(&[]))],
        "required":[stage("Required","Required",read(&[]))],
        "pure":[stage("Mixed","NullableBool",position(read(&[]),1))],
        "map/~":[stage("Rows","Strings",json!({"op":"map","list":read(&[]),"value":position(json!({"op":"read","scope":"item","path":[]}),0)}))],
        "plain":[stage("Any","Any",read(&[]))]
        ,"scopes/~":[scope_stage,stage("Any","Any",read(&[]))]
    },"positional_inputs":{
        "pair":[policy(json!([field("operands")]))],"root":[policy(json!([]))],"record":record_policies,
        "required":[policy(json!([field("operands")]))],"map/~":[policy(json!([{"kind":"items"}]))],"plain":[]
        ,"scopes/~":[policy(json!([field("rows"),{"kind":"items"}])),triple_policy,policy(json!([field("matrix"),{"kind":"items"},{"kind":"items"}]))]
    },"raw_json_inputs":{"record":[[field("raw")]]},"binary64_inputs":{"record":[[field("number")]]}});
    (bundle, recipe)
}

pub fn plan() -> Plan {
    let (bundle, recipe) = fixture();
    Plan::read(&recipe.to_string(), &[bundle]).unwrap()
}

pub fn mixed_plan() -> Plan {
    let (bundle, _) = fixture();
    let (model, _) = binary64::fixture();
    let root = |name: &str| Root::pin_model(&model, format!("sample.float.{name}")).unwrap();
    let literal = |value| json!({"op":"binary64_literal","value":value});
    let value = json!({"op":"binary64","value":{"op":"fallback","value":read(&["number"]),"fallback":literal("-0.0"),"on_null":false},"steps":[]});
    let recipe = json!({"format":"ess-normalization/6","branches":{"mixed":[
        {"input":Root::pin(&bundle,"Record").unwrap(),"output":root("Input"),"requires":[{"op":"present","value":position(read(&["operands"]),0)}],"value":{"op":"record","fields":{"a/b~":value}}},
        {"input":root("Input"),"output":root("Output"),"requires":[],"value":{"op":"fallback","value":read(&["a/b~"]),"fallback":literal("-0.0"),"on_null":false}},
        {"input":root("Output"),"output":root("Flag"),"requires":[],"value":{"op":"choose","condition":{"op":"equal","left":read(&[]),"right":literal("0.0")},"then_value":{"op":"boolean","value":true},"else_value":{"op":"boolean","value":false}}}
    ]},"raw_json_inputs":{"mixed":[[field("raw")]]},"binary64_inputs":{"mixed":[[field("number")]]},"positional_inputs":{"mixed":[policy(json!([field("operands")]))]}});
    Plan::check_with_models(serde_json::from_value(recipe).unwrap(), &[bundle], &[model]).unwrap()
}

pub fn mixed_cases() -> Vec<Value> {
    vec![
        json!({"branch":"mixed","input":r#"{"operands":["a",null,1e999],"raw":{"duplicate":0,"duplicate":1e999},"number":-0}"#,"value":true}),
        json!({"branch":"mixed","input":r#"{"operands":null,"number":0.10000000000000001}"#,"value":false}),
        json!({"branch":"mixed","input":r#"{"operands":[]}"#,"value":true}),
    ]
}

fn failure(branch: &str, input: String, pointer: &str, rule: &str, detail: &str) -> Value {
    let mut case =
        json!({"branch":branch,"errors":[{"pointer":pointer,"rule":rule,"detail":detail}]});
    case["input"] = input.into();
    case
}

pub fn cases() -> Vec<Value> {
    let mut cases = Vec::new();
    success_cases(&mut cases);
    type_refusals(&mut cases);
    lexical_refusals(&mut cases);
    entrypoint_cases(&mut cases);
    cases
}

fn success_cases(cases: &mut Vec<Value>) {
    for (input, pair) in [
        ("null", json!(["", ""])),
        ("[]", json!(["", ""])),
        (r#"["one"]"#, json!(["one", ""])),
        (r#"["one","two"]"#, json!(["one", "two"])),
        (r#"[null,"two"]"#, json!(["", "two"])),
        (r#"["one",null]"#, json!(["one", ""])),
        ("[null,null]", json!(["", ""])),
        (r#"["","same"]"#, json!(["", "same"])),
        (r#"["same","same"]"#, json!(["same", "same"])),
        (r#"["\u0000","é😀"]"#, json!(["\0", "é😀"])),
        (r#"["[\\\"]","\ud83d\ude00"]"#, json!(["[\\\"]", "😀"])),
    ] {
        cases.push(json!({"branch":"root","input":format!(" \n{input}\t "),"value":pair}));
        cases.push(json!({"branch":"pair","input":format!(r#"{{"operands":{input}}}"#),"value":{"left":pair[0],"right":pair[1]}}));
    }
    for tail in [
        "null",
        "true",
        "false",
        "0",
        "-0.0",
        "1e999",
        "1e-999",
        "184467440737095516170001",
        r#""text""#,
        "[]",
        "{}",
        r#"{"a":1,"\u0061":2}"#,
        r#"[1e999,{"x":0,"x":1}]"#,
    ]
    .into_iter()
    .map(str::to_owned)
    .chain([format!("1e{}", "9".repeat(500)), "9".repeat(500)])
    {
        cases.push(
            json!({"branch":"root","input":format!(r#"["a","b",{tail}]"#),"value":["a","b"]}),
        );
    }
    cases.push(json!({"branch":"pair","input":"{}","value":{"left":"","right":""}}));
    for (input, value) in [
        ("{}", json!({})),
        (r#"{"groups":null}"#, json!({"groups":null})),
        (
            r#"{"groups":[null,{}, {"pairs":null}, {"pairs":[{}, {"operands":null}]}]}"#,
            json!({"groups":[null,{}, {"pairs":null}, {"pairs":[{}, {"operands":["",""]}]}]}),
        ),
        (
            r#"{"wire\u002f~":["x"],"number":0.10000000000000001,"raw": { "x":1e999,"x":2 }}"#,
            json!({"wire/~":["x",""],"number":0.1,"raw":STANDARD.encode(br#"{ "x":1e999,"x":2 }"#)}),
        ),
    ] {
        cases.push(json!({"branch":"record","input":input,"value":value}));
    }
    cases.push(json!({"branch":"pure","input":r#"["x",true]"#,"value":true}));
    cases.push(json!({"branch":"pure","input":r#"["x",null]"#,"value":null}));
    cases.push(
        json!({"branch":"map/~","input":r#"[["a"],null,["c","d",1e999]]"#,"value":["a","","c"]}),
    );
    cases.push(json!({"branch":"scopes/~","input":r#"{"rows":[["a","one"],["b","one"],["a","two"]],"fallback":["x","y","z"],"matrix":[[["a"],["b","c",1e999]],[],[null]]}"#,"value":{"select/~":["one","two"],"count":2,"find":"one","nested":[["","c"],[],[""]]}}));
    cases.push(json!({"branch":"scopes/~","input":r#"{"rows":[["b"]],"fallback":["x","y","z"],"matrix":[]}"#,"value":{"select/~":[],"count":1,"find":"z","nested":[]}}));
    cases.push(json!({"branch":"scopes/~","input":r#"{"rows":[],"fallback":null,"matrix":[]}"#,"value":{"select/~":[],"count":0,"find":"","nested":[]}}));
}

fn type_refusals(cases: &mut Vec<Value>) {
    for input in [
        "true",
        "false",
        "0",
        "1e999",
        r#""x""#,
        "{}",
        r#"{"x":1e999}"#,
    ] {
        cases.push(failure(
            "root",
            input.into(),
            "/input",
            "positional_input_type",
            "fixed_string_array requires an array or null",
        ));
    }
    for index in 0..2 {
        for token in [
            "true",
            "false",
            "0",
            "1e999",
            "1e-999",
            "{}",
            "[]",
            r#"{"x":1e999,"x":2}"#,
        ] {
            let input = if index == 0 {
                format!(r#"[{token},"ok"]"#)
            } else {
                format!(r#"["ok",{token}]"#)
            };
            cases.push(failure(
                "root",
                input,
                &format!("/input/{index}"),
                "positional_element_type",
                "fixed_string_array element must be a string or null",
            ));
        }
    }
}

fn lexical_refusals(cases: &mut Vec<Value>) {
    for input in [
        r#"[true,"ok",]"#,
        r#"["a","b",{"x":}]"#,
        r#"["a","b"] null"#,
        r#"["a","b",NaN]"#,
        r#"["a","b",01]"#,
    ] {
        cases.push(failure(
            "root",
            input.into(),
            "/input",
            "input_syntax",
            "expected one complete JSON value",
        ));
    }
    for (input, at, detail) in [
        (
            r#"["\ud800","b"]"#,
            "/input/0",
            "positional string contains invalid Unicode",
        ),
        (
            r#"["a","\udc00"]"#,
            "/input/1",
            "positional string contains invalid Unicode",
        ),
        (
            r#"["a","b",{"\ud800":0}]"#,
            "/input/2",
            "discarded positional token contains invalid Unicode",
        ),
        (
            r#"["a","b",["\ud800"]]"#,
            "/input/2",
            "discarded positional token contains invalid Unicode",
        ),
    ] {
        cases.push(failure("root", input.into(), at, "input_syntax", detail));
    }
    cases.push(failure(
        "root",
        r#"[true,"b","\ud800"]"#.into(),
        "/input/0",
        "positional_element_type",
        "fixed_string_array element must be a string or null",
    ));
    cases.push(failure(
        "pair",
        r#"{"operands":[],"\u006fperands":null}"#.into(),
        "/input",
        "input_syntax",
        "JSON object keys must be unique",
    ));
    for depth in [63, 64, 10001] {
        let tail = format!("{}0{}", "[".repeat(depth), "]".repeat(depth));
        let input = format!(r#"["a","b",{tail}]"#);
        if depth == 63 {
            cases.push(json!({"branch":"root","input":input,"value":["a","b"]}));
        } else {
            cases.push(failure(
                "root",
                input.clone(),
                "/input/2",
                "input_depth",
                "JSON input exceeds 64 levels",
            ));
        }
        cases.push(failure(
            "root",
            format!("{input} false"),
            "/input",
            "input_syntax",
            "expected one complete JSON value",
        ));
    }
}

fn entrypoint_cases(cases: &mut Vec<Value>) {
    for branch in ["pair", "root", "required", "map/~"] {
        let mut case = failure(
            branch,
            "null".into(),
            "/input",
            "input_positional_provenance",
            "positional input decoding requires original JSON text; use a text input entrypoint",
        );
        case["mode"] = json!("value");
        cases.push(case);
    }
    let mut raw = failure(
        "record",
        "{}".into(),
        "/input",
        "input_capture_provenance",
        "raw JSON capture requires original token bytes; use a text input entrypoint",
    );
    raw["mode"] = json!("value");
    cases.push(raw);
    cases.push(json!({"branch":"plain","mode":"value","input":"{}","value":{}}));
    cases.push(json!({"branch":"root","mode":"base64","input":STANDARD.encode(br#"["a",null,1e999]"#),"value":["a",""]}));
    for (input, rule, detail) in [
        ("***", "input_base64", "expected canonical standard base64"),
        ("/w==", "input_utf8", "retained JSON bytes must be UTF-8"),
        ("", "input_syntax", "expected one complete JSON value"),
    ] {
        let mut case = failure("root", input.into(), "/input", rule, detail);
        case["mode"] = json!("base64");
        cases.push(case);
    }
    cases.push(failure(
        "missing",
        "null".into(),
        "/branches",
        "unknown_dispatch",
        "external discriminator has no declared branch",
    ));
}
