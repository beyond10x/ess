#![allow(dead_code)]

use ess_compiler::{resolve::compile, source::SourceMap};
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source,
};
use ess_gen::schema::ModelTypes;
use schema_contract::realize::normalize::{Plan, Root};
use serde_json::{json, Value};

pub const SOURCE: &str = r"format: ess/2
system: probe
version: v1
domains: [probe.data]
domain: probe.data
types:
  - {name: probe.data.Maybe, kind: newtype, of: 'Optional<Binary64>'}
  - name: probe.data.Input
    kind: struct
    fields:
      - {name: values, wire: 'a/~', type: 'List<Optional<List<Optional<Binary64>>>>'}
      - {name: count, type: Integer}
      - {name: raw, type: Bytes}
      - {name: ratio, type: 'Optional<probe.data.Maybe>'}
  - name: probe.data.Output
    kind: struct
    fields:
      - {name: values, wire: 'a/~', type: 'List<Optional<List<Optional<Binary64>>>>'}
      - {name: count, type: Integer}
      - {name: raw, type: Bytes}
      - {name: ratio, type: Binary64}
";

pub fn fixture() -> (ModelTypes, Value) {
    let mut sources = SourceMap::new();
    sources.insert(Source::DOCUMENT, SOURCE.to_owned());
    let spec = Specification::assemble([(Source::document(), RawSpecFile::parse(SOURCE).unwrap())])
        .unwrap();
    let ir = compile(&spec, &sources).unwrap();
    let model = ModelTypes::select(
        &ir,
        &std::collections::BTreeSet::from([
            "probe.data.Input".to_owned(),
            "probe.data.Output".to_owned(),
        ]),
    )
    .unwrap();
    let root = |name| Root::pin_model(&model, name).unwrap();
    let read = |name| json!({"op":"read","scope":"input","path":[name]});
    let literal = |token| json!({"op":"binary64_literal","value":token});
    let recipe = json!({"format":"ess-normalization/5",
    "binary64_inputs":{"primary":[[{"kind":"field","name":"a/~"},{"kind":"items"},{"kind":"items"}],[{"kind":"field","name":"ratio"}]]},
    "raw_json_inputs":{"primary":[[{"kind":"field","name":"raw"}]]},
    "branches":{"primary":[
      {"input":root("probe.data.Input"),"output":root("probe.data.Output"),"requires":[],"value":{"op":"record","fields":{
        "a/~":read("a/~"),"count":read("count"),"raw":read("raw"),
        "ratio":{"op":"fallback","value":read("ratio"),"fallback":literal("-0.0"),"on_null":true}
      }}},
      {"input":root("probe.data.Output"),"output":root("probe.data.Output"),"requires":[{"op":"equal","left":literal("-0"),"right":literal("0.0")}],"value":{"op":"read","scope":"input","path":[]}}
    ]}});
    (model, recipe)
}

pub fn plan() -> Plan {
    let (model, recipe) = fixture();
    Plan::check_with_models(serde_json::from_value(recipe).unwrap(), &[], &[model]).unwrap()
}

pub fn cases() -> Value {
    json!([
      {"input":"{\"a/~\":[null,[-0,null,5e-324,9007199254740995]],\"count\":9007199254740993,\"raw\": {\"x\":1e999, \"x\":-0}}", "expected":"{\"a/~\":[null,[-0.0,null,5e-324,9007199254740996.0]],\"count\":9007199254740993,\"raw\":\"eyJ4IjoxZTk5OSwgIngiOi0wfQ==\",\"ratio\":-0.0}"},
      {"input":"{\"a/~\":[],\"count\":0,\"raw\":null,\"ratio\":null}","expected":"{\"a/~\":[],\"count\":0,\"raw\":\"bnVsbA==\",\"ratio\":-0.0}"},
      {"input":"{\"a/~\":[[],[null,-1e-999]],\"count\":0,\"raw\":\"\\u0041\",\"ratio\":0}","expected":"{\"a/~\":[[],[null,-0.0]],\"count\":0,\"raw\":\"Ilx1MDA0MSI=\",\"ratio\":0.0}"},
      {"input":"{\"a/~\":[[1e999]],\"count\":0,\"raw\":null}","error":"input_number","pointer":"/input/a~1~0/0/0"},
      {"input":"{\"a/~\":[],\"count\":0.10000000000000001,\"raw\":null}","error":"input_number","pointer":"/input/count"},
      {"input":"{\"a/~\":[],\"count\":0,\"raw\":null,\"ratio\":\"0\"}","error":"schema_validation"},
      {"input":"{\"a/~\":[[\"0\"]],\"count\":0,\"raw\":null}","error":"schema_validation"},
      {"input":"{\"a/~\":[],\"count\":0,\"raw\":null,\"ratio\":0,\"ratio\":1}","error":"input_syntax","pointer":"/input"}
    ])
}

pub fn assert_case(case: &Value, result: Result<Value, schema_contract::realize::Refused>) {
    if let Some(rule) = case["error"].as_str() {
        let errors = result.unwrap_err();
        assert_eq!(errors.0[0].rule, rule, "{case}: {errors:?}");
        if let Some(pointer) = case["pointer"].as_str() {
            assert_eq!(errors.0[0].pointer, pointer, "{case}");
        }
        return;
    }
    let actual = result.unwrap_or_else(|e| panic!("{case}: {e:?}"));
    let expected: Value = serde_json::from_str(case["expected"].as_str().unwrap()).unwrap();
    assert_value(&actual, &expected);
}

pub fn assert_value(actual: &Value, expected: &Value) {
    match (actual, expected) {
        (Value::Number(a), Value::Number(e)) if e.is_f64() => {
            assert!(a.is_f64(), "floating identity: {actual}");
            assert_eq!(
                a.as_f64().unwrap().to_bits(),
                e.as_f64().unwrap().to_bits(),
                "{actual}: {expected}"
            );
        }
        (Value::Array(a), Value::Array(e)) => {
            assert_eq!(a.len(), e.len());
            for (a, e) in a.iter().zip(e) {
                assert_value(a, e);
            }
        }
        (Value::Object(a), Value::Object(e)) => {
            assert_eq!(a.keys().collect::<Vec<_>>(), e.keys().collect::<Vec<_>>());
            for (k, e) in e {
                assert_value(&a[k], e);
            }
        }
        _ => assert_eq!(actual, expected),
    }
}
