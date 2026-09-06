use base64::{engine::general_purpose::STANDARD, Engine as _};
use schema_contract::bundle::{import, Bundle, Dialect};
use schema_contract::realize::normalize::{Plan, Root};
use serde_json::{json, Value};

#[allow(dead_code)]
#[path = "normalization_model.rs"]
mod model;

pub fn field(name: &str) -> Value {
    json!({"kind":"field", "name":name})
}

pub fn fixture() -> (Bundle, Value) {
    let source = json!({"components":{"schemas":{
        "Text":{"type":"string"},
        "Record":{"type":"object", "additionalProperties":false, "properties":{
            "payload":{"type":"string"},
            "items":{"type":["array","null"],"items":{"type":"string"}},
            "nested":{"type":["object","null"],"additionalProperties":false,"properties":{"body":{"type":"string"}}},
            "a/b~":{"type":"string"}, "number":{"type":"number"}}},
        "Number":{"type":"number"}, "Null":{"type":"null"}, "Any":true
    }}});
    let bundle = import(
        &source.to_string(),
        &["Text", "Record", "Number", "Null", "Any"]
            .map(str::to_owned)
            .into_iter()
            .collect(),
        Dialect::Draft202012,
    )
    .unwrap();
    let stage = |name| {
        let root = Root::pin(&bundle, name).unwrap();
        json!({"input":root,"output":root,"requires":[],"value":{"op":"read","scope":"input","path":[]}})
    };
    let recipe = json!({"format":"ess-normalization/4","branches":{
        "root":[stage("Text")], "record":[stage("Record"),stage("Record")],
        "plain":[stage("Any")], "number":[stage("Number")], "null":[stage("Null")]
    }, "raw_json_inputs":{"root":[[]],"record":[[field("payload")],[field("items"),{"kind":"items"}],[field("nested"),field("body")],[field("a/b~")]],"plain":[]},
    "binary64_inputs":{"record":[[field("number")]]}});
    (bundle, recipe)
}

pub fn plan() -> Plan {
    let (bundle, mut recipe) = fixture();
    let model = model::selection("format: ess/1\nsystem: sample\nversion: v1\ndomains: [sample.raw]\ndomain: sample.raw\ntypes:\n  - name: sample.raw.Retained\n    kind: newtype\n    of: Bytes\n  - name: sample.raw.Input\n    kind: struct\n    fields:\n      - name: payload\n        wire: raw/data~\n        type: Optional<sample.raw.Retained>\n      - name: items\n        type: List<sample.raw.Retained>\n", &["sample.raw.Input"]);
    let root = Root::pin_model(&model, "sample.raw.Input").unwrap();
    recipe["branches"]["model"] = json!([{"input":root,"output":root,"requires":[],"value":{"op":"read","scope":"input","path":[]}}]);
    recipe["raw_json_inputs"]["model"] =
        json!([[field("raw/data~")],[field("items"),{"kind":"items"}]]);
    Plan::check_with_models(serde_json::from_value(recipe).unwrap(), &[bundle], &[model]).unwrap()
}

pub fn cases() -> Vec<Value> {
    let mut cases = Vec::new();
    capture_cases(&mut cases);
    outside_refusals(&mut cases);
    lexical_refusals(&mut cases);
    depth_cases(&mut cases);
    entrypoint_cases(&mut cases);
    cases
}

fn capture_cases(cases: &mut Vec<Value>) {
    let tokens = [
        "null",
        "true",
        "false",
        "0",
        "-0",
        "1.0",
        "1e0",
        "-0.0",
        "1e999",
        "1e-999",
        "184467440737095516170000000000000000001",
        r#""\u0061""#,
        r#""a""#,
        r#""\ud83d\ude00""#,
        "\"é😀\"",
        "[]",
        "{}",
        r#"{ "n": 1e0, "n":1.0 }"#,
        r#"{"a":1,"\u0061":2}"#,
        r#"[ null ,"\u0061",{"z":-0.0} ]"#,
    ];
    for token in tokens.iter().map(|s| (*s).to_owned()).chain([
        format!("1e{}", "9".repeat(500)),
        format!("-{}", "9".repeat(500)),
    ]) {
        let retained = STANDARD.encode(token.as_bytes());
        for (branch, input, value) in [
            ("root", format!(" \n{token}\t "), json!(retained)),
            (
                "record",
                format!(
                    r#"{{"payload": {token} , "items":[{token}], "nested":{{"body":{token}}}, "a/b~":{token},"number":0.10000000000000001}}"#
                ),
                json!({"payload":retained,"items":[retained],"nested":{"body":retained},"a/b~":retained,"number":0.1}),
            ),
            (
                "model",
                format!(r#"{{"raw/data~":{token},"items":[{token}]}}"#),
                json!({"raw/data~":retained,"items":[retained]}),
            ),
        ] {
            cases.push(json!({"branch":branch,"input":input,"value":value}));
        }
    }
    for (input, value) in [
        ("{}", json!({})),
        (
            r#"{"items":null,"nested":null}"#,
            json!({"items":null,"nested":null}),
        ),
        (
            r#"{"nested":{},"items":[]}"#,
            json!({"nested":{},"items":[]}),
        ),
    ] {
        cases.push(json!({"branch":"record","input":input,"value":value}));
    }
}

fn outside_refusals(cases: &mut Vec<Value>) {
    for (branch, input, at, rule, detail) in [
        (
            "root",
            "",
            "/input",
            "input_syntax",
            "expected one complete JSON value",
        ),
        (
            "root",
            "true false",
            "/input",
            "input_syntax",
            "expected one complete JSON value",
        ),
        (
            "record",
            r#"{"payload":1,"payload":2}"#,
            "/input",
            "input_syntax",
            "JSON object keys must be unique",
        ),
        (
            "record",
            r#"{"payload":1,"\u0070ayload":2}"#,
            "/input",
            "input_syntax",
            "JSON object keys must be unique",
        ),
        (
            "record",
            r#"{"payload":1,"extra":{"n":1,"n":2}}"#,
            "/input/extra",
            "input_syntax",
            "JSON object keys must be unique",
        ),
        (
            "record",
            r#"{"payload":"\ud800","\ud800":0}"#,
            "/input",
            "input_syntax",
            "JSON object keys must be unique",
        ),
        (
            "record",
            r#"{"payload":"\ud800","extra":1e999}"#,
            "/input/extra",
            "input_number",
            "JSON number is outside the supported representation",
        ),
        (
            "plain",
            "1e999",
            "/input",
            "input_number",
            "JSON number is outside the supported representation",
        ),
        (
            "missing",
            "1e999",
            "/input",
            "input_number",
            "JSON number is outside the supported representation",
        ),
        (
            "missing",
            "null",
            "/branches",
            "unknown_dispatch",
            "external discriminator has no declared branch",
        ),
    ] {
        cases.push(refusal(branch, input, at, rule, detail));
    }
}

fn lexical_refusals(cases: &mut Vec<Value>) {
    for token in [
        r#""\ud800""#,
        r#""\udc00""#,
        r#""\ud800x""#,
        r#""\ud800\ud800""#,
        r#"{"\ud800":0}"#,
        r#"{"ok":["\udc00"]}"#,
    ] {
        for (branch, input, at) in [
            ("root", token.to_owned(), "/input"),
            (
                "record",
                format!(r#"{{"payload":{token}}}"#),
                "/input/payload",
            ),
            (
                "record",
                format!(r#"{{"items":[0,{token}]}}"#),
                "/input/items/1",
            ),
        ] {
            cases.push(refusal(
                branch,
                &input,
                at,
                "input_syntax",
                "captured JSON token contains invalid Unicode",
            ));
        }
    }
    for token in [
        "NaN",
        "Infinity",
        "01",
        "1.",
        "1e",
        "[1,]",
        "{\"n\":1,}",
        "/*x*/0",
        r#""\x00""#,
        r#""\uZZZZ""#,
        "\"\n\"",
    ] {
        cases.push(refusal(
            "root",
            token,
            "/input",
            "input_syntax",
            "expected one complete JSON value",
        ));
        cases.push(refusal(
            "record",
            &format!(r#"{{"payload":{token}}}"#),
            "/input",
            "input_syntax",
            "expected one complete JSON value",
        ));
    }
}

fn depth_cases(cases: &mut Vec<Value>) {
    for depth in [63, 64, 65, 128, 10_001] {
        let token = format!("{}null{}", "[".repeat(depth), "]".repeat(depth));
        if depth <= 64 {
            cases.push(json!({"branch":"root","input":token,"value":STANDARD.encode(&token)}));
        } else {
            cases.push(refusal(
                "root",
                &token,
                "/input",
                "input_depth",
                "JSON input exceeds 64 levels",
            ));
        }
        let input = format!(r#"{{"payload":{token}}}"#);
        if depth < 64 {
            cases.push(json!({"branch":"record","input":input,"value":{"payload":STANDARD.encode(&token)}}));
        } else {
            cases.push(refusal(
                "record",
                &input,
                "/input/payload",
                "input_depth",
                "JSON input exceeds 64 levels",
            ));
        }
        if depth == 10_001 {
            cases.push(refusal(
                "root",
                &format!("{token} trailing"),
                "/input",
                "input_syntax",
                "expected one complete JSON value",
            ));
            cases.push(refusal(
                "root",
                &format!("[{},]", "[".repeat(depth)),
                "/input",
                "input_syntax",
                "expected one complete JSON value",
            ));
        }
    }
    let deep = format!("{}null{}", "[".repeat(65), "]".repeat(65));
    for (input, rule, detail) in [
        (
            format!(r#"{{"first":{deep},"\ud800":0}}"#),
            "input_depth",
            "JSON input exceeds 64 levels",
        ),
        (
            format!(r#"{{"\ud800":0,"later":{deep}}}"#),
            "input_syntax",
            "captured JSON token contains invalid Unicode",
        ),
        (
            format!(r#"[{deep},"\ud800"]"#),
            "input_depth",
            "JSON input exceeds 64 levels",
        ),
        (
            format!(r#"["\ud800",{deep}]"#),
            "input_syntax",
            "captured JSON token contains invalid Unicode",
        ),
        (
            format!("{deep} trailing"),
            "input_syntax",
            "expected one complete JSON value",
        ),
    ] {
        cases.push(refusal("root", &input, "/input", rule, detail));
    }
}

fn entrypoint_cases(cases: &mut Vec<Value>) {
    for branch in ["root", "record"] {
        for input in ["null", "{}", "\"already encoded\""] {
            let mut case = refusal(
                branch,
                input,
                "/input",
                "input_capture_provenance",
                "raw JSON capture requires original token bytes; use a text input entrypoint",
            );
            case["mode"] = json!("value");
            cases.push(case);
        }
    }
    cases.push(json!({"branch":"plain","input":"null","value":null,"mode":"value"}));
    for encoded in [
        "Zg", "Zh==", "Zm9=", "Zg===", "Zg==\n", " Zg==", "_w==", "-w==", "!!!!",
    ] {
        let mut case = refusal(
            "missing",
            encoded,
            "/input",
            "input_base64",
            "expected canonical standard base64",
        );
        case["mode"] = json!("base64");
        cases.push(case);
    }
    for (branch, encoded, rule, detail) in [
        (
            "missing",
            "/w==",
            "input_utf8",
            "retained JSON bytes must be UTF-8",
        ),
        (
            "root",
            "",
            "input_syntax",
            "expected one complete JSON value",
        ),
    ] {
        let mut case = refusal(branch, encoded, "/input", rule, detail);
        case["mode"] = json!("base64");
        cases.push(case);
    }
    cases.push(json!({"branch":"root","input":"bnVsbA==","mode":"base64","value":"bnVsbA=="}));
    cases.push(json!({"branch":"record","input":STANDARD.encode(r#"{"payload": { "n":1e999,"n":2 }}"#),"mode":"base64","value":{"payload":STANDARD.encode(r#"{ "n":1e999,"n":2 }"#)}}));
}

fn refusal(branch: &str, input: &str, at: &str, rule: &str, detail: &str) -> Value {
    json!({"branch":branch,"input":input,"errors":[{"pointer":at,"rule":rule,"detail":detail}]})
}
