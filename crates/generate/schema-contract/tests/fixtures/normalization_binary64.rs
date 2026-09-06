use ess_gen::schema::ModelTypes;
use schema_contract::realize::normalize::{Plan, Root};
use serde_json::{json, Value};

#[allow(dead_code)]
#[path = "normalization_model.rs"]
pub(super) mod model;
#[allow(dead_code)]
#[path = "normalization_numeric.rs"]
mod numeric;

pub const SOURCE: &str = "format: ess/2\nsystem: sample\nversion: v1\ndomains: [sample.float]\ndomain: sample.float\ntypes:\n  - name: sample.float.Input\n    kind: struct\n    fields:\n      - {name: ratio, type: Optional<sample.float.Output>, wire: 'a/b~'}\n  - name: sample.float.Output\n    kind: newtype\n    of: Binary64\n  - {name: sample.float.Count, kind: newtype, of: Integer}\n  - {name: sample.float.Flag, kind: newtype, of: Boolean}\n  - {name: sample.float.Items, kind: newtype, of: 'List<Optional<Binary64>>'}\n  - {name: sample.float.Retained, kind: newtype, of: Bytes}\n";

pub fn fixture() -> (ModelTypes, Value) {
    let model = model::selection(
        SOURCE,
        &[
            "sample.float.Input",
            "sample.float.Output",
            "sample.float.Count",
            "sample.float.Flag",
            "sample.float.Items",
            "sample.float.Retained",
        ],
    );
    let root = |name: &str| Root::pin_model(&model, format!("sample.float.{name}")).unwrap();
    let read = json!({"op":"read","scope":"input","path":[]});
    let literal = |value| json!({"op":"binary64_literal","value":value});
    let fallback = json!({"op":"fallback","value":{"op":"read","scope":"input","path":["a/b~"]},"fallback":literal("-0.0"),"on_null":false});
    let stage = |input, output, value: Value| json!([{"input":root(input),"output":root(output),"requires":[],"value":value}]);
    let mut recipe = json!({"format":"ess-normalization/5", "binary64_inputs":{
        "main":[[{"kind":"field","name":"a/b~"}]], "scalar":[[]], "equal":[[]], "items":[[{"kind":"items"}]], "lazy":[[{"kind":"field","name":"a/b~"}]]
    },"raw_json_inputs":{"capture":[[]]},"branches":{
        "main":stage("Input","Output",fallback),
        "scalar":stage("Output","Output",read.clone()),
        "cast":stage("Count","Output",json!({"op":"binary64","value":read,"steps":[]})),
        "equal":stage("Output","Flag",json!({"op":"choose","condition":{"op":"equal","left":read,"right":literal("0.0")},"then_value":{"op":"boolean","value":true},"else_value":{"op":"boolean","value":false}})),
        "items":stage("Items","Items",read.clone()),
        "capture":stage("Retained","Retained",read.clone()),
        "lazy":stage("Input","Output",json!({"op":"fallback","value":{"op":"read","scope":"input","path":["a/b~"]},"fallback":{"op":"binary64","value":literal("1e308"),"steps":[{"op":"multiply","value":"1e308"}]},"on_null":false})),
        "minimum":stage("Count","Output",json!({"op":"binary64","value":literal("0.0"),"steps":[{"op":"minimum","value":"-0.0"}]})),
        "maximum":stage("Count","Output",json!({"op":"binary64","value":literal("-0.0"),"steps":[{"op":"maximum","value":"0.0"}]}))
    }});
    // Reuse the existing independently specified ordered conversion recipe and its
    // expected arithmetic results, changing only its checked numeric root authority.
    let (_, old) = numeric::fixture();
    for (name, stages) in old["branches"].as_object().unwrap() {
        if name == "copy" {
            continue;
        }
        let mut stages = stages.clone();
        stages[0]["input"] = json!(root("Output"));
        stages[0]["output"] = json!(root("Count"));
        recipe["branches"][name] = stages;
        recipe["binary64_inputs"][name] = json!([[]]);
    }
    (model, recipe)
}

pub fn plan() -> Plan {
    let (model, recipe) = fixture();
    Plan::check_with_models(serde_json::from_value(recipe).unwrap(), &[], &[model]).unwrap()
}

pub fn cases() -> Vec<Value> {
    let mut cases = Vec::new();
    for (token, bits) in [
        ("0", 0_u64),
        ("-0", 0x8000_0000_0000_0000),
        ("0.0", 0),
        ("-0.0", 0x8000_0000_0000_0000),
        ("1e-999", 0),
        ("-1e-999", 0x8000_0000_0000_0000),
        ("5e-324", 1),
        ("-5e-324", 0x8000_0000_0000_0001),
        ("2.2250738585072014e-308", 0x0010_0000_0000_0000),
        ("9007199254740993", 0x4340_0000_0000_0000),
        ("9007199254740995", 0x4340_0000_0000_0002),
        ("1.7976931348623157e308", 0x7fef_ffff_ffff_ffff),
        ("0.10000000000000001", 0x3fb9_9999_9999_999a),
    ] {
        for (branch, input) in [
            ("scalar", token.to_owned()),
            ("main", format!(r#"{{"a/b~":{token}}}"#)),
        ] {
            cases.push(json!({"branch":branch,"input":input,"bits":format!("{bits:016x}")}));
        }
    }
    for (branch, input, bits) in [
        ("main", "{}", "8000000000000000"),
        ("lazy", r#"{"a/b~":1}"#, "3ff0000000000000"),
        ("cast", "0", "0000000000000000"),
        ("cast", "9007199254740993", "4340000000000000"),
        ("minimum", "0", "8000000000000000"),
        ("maximum", "0", "0000000000000000"),
    ] {
        cases.push(json!({"branch":branch,"input":input,"bits":bits}));
    }
    for (token, expected) in [("0", true), ("-0", true), ("5e-324", false), ("1", false)] {
        cases.push(json!({"branch":"equal","input":token,"value":expected}));
    }
    cases.push(json!({"branch":"items","input":"[-0, null, 5e-324]","array_bits":["8000000000000000",null,"0000000000000001"]}));
    cases.push(json!({"branch":"capture","input":" {\"x\":1e999,\"x\":-0} ","value":"eyJ4IjoxZTk5OSwieCI6LTB9"}));
    for (branch, input, value) in numeric::expected() {
        if branch != "copy" {
            cases.push(json!({"branch":branch,"input":input,"value":value}));
        }
    }
    for (branch, input, rule) in [
        ("scalar", "1e999", "input_number"),
        ("scalar", "\"1.0\"", "schema_validation"),
        ("scalar", "null", "schema_validation"),
        ("main", "{\"a/b~\":null}", "schema_validation"),
        ("main", "{\"a/b~\":0,\"a/b~\":1}", "input_syntax"),
        ("lazy", "{}", "binary64_overflow"),
        ("overflow", "2", "binary64_overflow"),
        ("truncate", "9223372036854775808", "binary64_range"),
    ] {
        cases.push(json!({"branch":branch,"input":input,"error":rule}));
    }
    cases
}
