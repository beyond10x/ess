use ess_compiler::{resolve::compile, source::SourceMap};
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source,
};
use ess_gen::schema::ModelTypes;
use schema_contract::realize::normalize::{Plan, Root, FORMAT_V3};
use serde_json::{json, Value};
use std::collections::BTreeSet;

pub const SOURCE: &str = r"format: ess/1
system: sample
version: v1
domains: [sample.settings]
domain: sample.settings
types:
  - name: sample.settings.Id
    kind: newtype
    of: String
  - name: sample.settings.Milliseconds
    kind: newtype
    of: Integer
  - name: sample.settings.Mode
    kind: enum
    variants: [ready, paused]
  - name: sample.settings.Decoded
    kind: struct
    fields:
      - name: id
        type: sample.settings.Id
        wire: identifier
      - name: seconds
        type: Integer
      - name: label
        type: Optional<String>
  - name: sample.settings.Runtime
    kind: struct
    fields:
      - name: mode
        type: sample.settings.Mode
      - name: id
        type: sample.settings.Id
        wire: key
      - name: elapsed
        type: sample.settings.Milliseconds
        wire: elapsed-ms
      - name: label
        type: Optional<String>
  - name: sample.settings.Guarded
    kind: struct
    fields:
      - name: count
        type: Integer
    invariants:
      - count >= 0
";

pub fn selection(source: &str, roots: &[&str]) -> ModelTypes {
    let mut sources = SourceMap::new();
    sources.insert(Source::DOCUMENT, source.to_owned());
    let specification =
        Specification::assemble([(Source::document(), RawSpecFile::parse(source).unwrap())])
            .unwrap();
    let ir = compile(&specification, &sources).unwrap();
    ModelTypes::select(
        &ir,
        &roots
            .iter()
            .map(|root| (*root).to_owned())
            .collect::<BTreeSet<_>>(),
    )
    .unwrap()
}

pub fn fixture() -> (ModelTypes, Value) {
    let model = selection(
        SOURCE,
        &["sample.settings.Decoded", "sample.settings.Runtime"],
    );
    let read = |field| json!({"op":"read", "scope":"input", "path":[field]});
    let recipe = json!({"format":FORMAT_V3, "branches":{"primary":[{
        "input":Root::pin_model(&model, "sample.settings.Decoded").unwrap(),
        "output":Root::pin_model(&model, "sample.settings.Runtime").unwrap(),
        "requires":[], "value":{"op":"record", "fields":{
            "mode":{"op":"string","value":"ready"},
            "key":read("identifier"), "label":read("label"),
            "elapsed-ms":{"op":"arithmetic", "operation":"multiply", "overflow":"reject",
                "left":read("seconds"), "right":{"op":"integer", "value":1000}}
        }}
    }]}});
    (model, recipe)
}

pub fn plan() -> Plan {
    let (model, recipe) = fixture();
    Plan::check_with_models(serde_json::from_value(recipe).unwrap(), &[], &[model]).unwrap()
}

pub fn cases() -> Value {
    let plan = plan();
    let mut cases = vec![
        json!({"branch":"primary", "input":r#"{"identifier":"x","seconds":3}"#, "value":{"key":"x","elapsed-ms":3000}}),
        json!({"branch":"primary", "input":r#"{"identifier":"","seconds":0,"label":""}"#, "value":{"key":"","elapsed-ms":0,"label":""}}),
        json!({"branch":"primary", "input":r#"{"identifier":"x","seconds":-2,"label":"a"}"#, "value":{"key":"x","elapsed-ms":-2000,"label":"a"}}),
    ];
    for input in [
        r#"{"id":"x","seconds":3}"#,
        r#"{"identifier":"x","seconds":3,"label":null}"#,
        r#"{"identifier":"x","seconds":9223372036854775807}"#,
    ] {
        cases.push(json!({"branch":"primary", "input":input, "errors":plan.run_json("primary", input).unwrap_err().0}));
    }
    for seconds in [
        -9_223_372_036_854_776_i64,
        -9_223_372_036_854_775,
        -2,
        -1,
        0,
        1,
        2,
        9_223_372_036_854_775,
        9_223_372_036_854_776,
    ] {
        for label in [None, Some(""), Some("label")] {
            let mut input = json!({"identifier":"x","seconds":seconds});
            if let Some(label) = label {
                input["label"] = json!(label);
            }
            if let Some(elapsed) = seconds.checked_mul(1000) {
                let mut value = json!({"key":"x","elapsed-ms":elapsed,"mode":"ready"});
                if let Some(label) = label {
                    value["label"] = json!(label);
                }
                assert_eq!(plan.run("primary", &input).unwrap(), value);
                cases.push(json!({"branch":"primary","input":input.to_string(),"value":value}));
            } else {
                let errors = plan.run("primary", &input).unwrap_err().0;
                assert_eq!(errors[0].rule, "integer_overflow");
                cases.push(json!({"branch":"primary","input":input.to_string(),"errors":errors}));
            }
        }
    }
    for case in &mut cases {
        if let Some(value) = case.get_mut("value") {
            value["mode"] = json!("ready");
        }
    }
    json!(cases)
}
