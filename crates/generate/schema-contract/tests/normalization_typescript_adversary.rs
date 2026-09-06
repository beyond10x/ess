//! Independent contract boundaries beyond the inherited TypeScript corpus.
use schema_contract::bundle::{import, Bundle, Dialect};
use schema_contract::realize::normalize::{Plan, Root};
use serde_json::{json, Value};

#[cfg(feature = "typescript-typecheck")]
#[path = "support/normalization_typescript.rs"]
mod native;

fn bundle(schemas: &Value) -> Bundle {
    let roots = schemas.as_object().unwrap().keys().cloned().collect();
    import(
        &json!({"components":{"schemas":schemas}}).to_string(),
        &roots,
        Dialect::Draft202012,
    )
    .unwrap()
}

fn stage(bundle: &Bundle, input: &str, output: &str, value: Value) -> Value {
    let mut stage = json!({"input":Root::pin(bundle,input).unwrap(),"output":Root::pin(bundle,output).unwrap(),"requires":[]});
    stage["value"] = value;
    stage
}

fn read() -> Value {
    json!({"op":"read","scope":"input","path":[]})
}

#[test]
fn referenced_property_names_stop_at_plan_admission_and_annotation_decoys_stay_inert() {
    let bundle = bundle(&json!({
        "Good":{"type":"string","default":{"pattern":"not a schema"},"examples":[{"uniqueItems":true}]},
        "Map":{"type":"object","additionalProperties":{"type":"string"},"propertyNames":{"$ref":"#/components/schemas/Key~1~0"}},
        "Key/~":{"type":"string","pattern":"^x$"}
    }));
    let recipe = json!({"format":"ess-normalization/6","branches":{
        "used":[stage(&bundle,"Good","Good",read())],
        "unused":[stage(&bundle,"Map","Map",read())]
    }});
    // The public checker does not admit this qualified-bundle shape. This is a
    // boundary control, not evidence that TypeScript sees this private schema.
    let errors = Plan::read(&recipe.to_string(), std::slice::from_ref(&bundle))
        .unwrap_err()
        .0;
    assert_eq!(errors.len(), 2);
    for (error, side) in errors.iter().zip(["input", "output"]) {
        assert_eq!(error.rule, "root_selection");
        assert_eq!(error.pointer, format!("/branches/unused/0/{side}"));
        assert_eq!(
            error.detail,
            "/components/schemas/Map/propertyNames: propertyNames"
        );
    }
    let safe = json!({"format":"ess-normalization/6","branches":{"used":[stage(&bundle,"Good","Good",read())]}});
    assert!(Plan::read(&safe.to_string(), &[bundle])
        .unwrap()
        .typescript("@scope/adapter")
        .is_ok());
}

#[test]
fn compiler_owned_integer_map_key_pattern_is_not_implicitly_qualified() {
    use ess_compiler::{resolve::compile, source::SourceMap};
    use ess_domain::{
        spec::{RawSpecFile, Specification},
        system::Source,
    };
    use ess_gen::schema::ModelTypes;
    let source = "format: ess/1\nsystem: sample\nversion: v1\ndomains: [sample.keys]\ndomain: sample.keys\ntypes:\n  - name: sample.keys.Row\n    kind: struct\n    fields:\n      - name: lookup\n        wire: lookup/~\n        type: Map<Integer, String>\n";
    let mut sources = SourceMap::new();
    sources.insert(Source::DOCUMENT, source.to_owned());
    let specification =
        Specification::assemble([(Source::document(), RawSpecFile::parse(source).unwrap())])
            .unwrap();
    let ir = compile(&specification, &sources).unwrap();
    let model =
        ModelTypes::select(&ir, &["sample.keys.Row".to_owned()].into_iter().collect()).unwrap();
    let root = Root::pin_model(&model, "sample.keys.Row").unwrap();
    let recipe = json!({"format":"ess-normalization/6","branches":{"unused":[{"input":root,"output":root,"requires":[],"value":read()}]}});
    let plan =
        Plan::check_with_models(serde_json::from_value(recipe).unwrap(), &[], &[model]).unwrap();
    let errors = plan.typescript("adapter").unwrap_err().0;
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].rule, "typescript_schema_profile");
    let Root::Model { model, .. } = root else {
        panic!("expected a model root")
    };
    let digest = model.projection_digest;
    assert_eq!(
        errors[0].pointer,
        format!(
            "/models/{digest}/$defs/sample.keys.Row/properties/lookup~1~0/propertyNames/pattern"
        )
    );
}

#[cfg(feature = "typescript-typecheck")]
#[test]
fn native_bounds_compare_the_exact_binary_fraction_not_its_shortest_decimal() {
    // This literal's binary value is exactly 1_000_000_000_000_000_128;
    // its shortest decimal text denotes 1_000_000_000_000_000_100 instead.
    let bound = 1.000_000_000_000_000_1e18_f64;
    assert_eq!(format!("{bound:.0}"), "1000000000000000128");
    let bundle = bundle(&json!({
        "Min":{"type":"number","minimum":bound},
        "Max":{"type":"number","maximum":bound},
        "Any":true
    }));
    let recipe = json!({"format":"ess-normalization/6","branches":{
        "min":[stage(&bundle,"Min","Any",read())],
        "max":[stage(&bundle,"Max","Any",read())]
    }});
    let plan = Plan::read(&recipe.to_string(), &[bundle]).unwrap();
    let mut cases = Vec::new();
    for (branch, input, accepted) in [
        ("min", 1_000_000_000_000_000_100_u64, false),
        ("min", 1_000_000_000_000_000_127, false),
        ("min", 1_000_000_000_000_000_128, true),
        ("min", 1_000_000_000_000_000_129, true),
        ("max", 1_000_000_000_000_000_100, true),
        ("max", 1_000_000_000_000_000_127, true),
        ("max", 1_000_000_000_000_000_128, true),
        ("max", 1_000_000_000_000_000_129, false),
    ] {
        let mut case = json!({"branch":branch,"input":input.to_string()});
        if accepted {
            case["value"] = json!(input);
        } else {
            case["error"] = json!("schema_validation");
        }
        cases.push(case);
    }
    assert_eq!(native::run_cases("adversary-exact-bound", &plan, &cases), 8);
}

#[cfg(feature = "typescript-typecheck")]
#[test]
fn native_retained_reentry_and_escaped_nested_position_authority() {
    use base64::{engine::general_purpose::STANDARD, Engine as _};
    let pair = json!({"type":"array","prefixItems":[{"type":"string"},{"type":"string"}],"items":false,"minItems":2,"maxItems":2});
    let bundle = bundle(&json!({
        "Text":{"type":"string"},"Any":true,
        "Input":{"type":"object","properties":{"provider":{"type":"string"},"pair":pair},"required":["pair"],"additionalProperties":false},
        "Matrix":{"type":"array","items":{"type":"array","items":pair}}
    }));
    let position = |value: Value, index| json!({"op":"position","value":value,"index":index});
    let field = |name| json!({"op":"read","scope":"input","path":[name]});
    let item = json!({"op":"read","scope":"item","path":[]});
    let recipe = json!({"format":"ess-normalization/6","branches":{
        "retain":[stage(&bundle,"Text","Text",read())],
        "adapt":[stage(&bundle,"Input","Input",read()),stage(&bundle,"Input","Any",json!({"op":"record","fields":{
            "provider":field("provider"),"left":position(field("pair"),0),"right":position(field("pair"),1)
        }}))],
        "pair~/":[stage(&bundle,"Matrix","Any",json!({"op":"map","list":read(),"value":{
            "op":"map","list":item,"value":{"op":"record","fields":{"a/b~":position(item.clone(),0)}}
        }}))]
    },"raw_json_inputs":{"retain":[[]],"adapt":[[{"kind":"field","name":"provider"}]]},
    "positional_inputs":{"adapt":[{"path":[{"kind":"field","name":"pair"}],"kind":"fixed_string_array","length":2,"missing":"preserve","null":"zero","short":"zero_pad","extra":"discard","null_element":"zero"}]}});
    let plan = Plan::read(&recipe.to_string(), &[bundle]).unwrap();
    let provider = r#"{ "x":1, "x":1e999, "literal":"😀", "escaped":"\ud83d\ude00" }"#;
    let cases = vec![
        json!({"input":format!(" \n {{\"provider\": {provider} , \"pair\":[null,\"r\",{{\"k\":1,\"k\":1e999}}]}} \t"),"output":json!({"provider":STANDARD.encode(provider),"left":"","right":"r"}).to_string()}),
        json!({"input":r#"{"provider":"","pair":[]}"#,"output":r#"{"left":"","provider":"IiI=","right":""}"#}),
        json!({"input":r#"{"pair":null}"#,"output":r#"{"left":"","right":""}"#}),
        json!({"input":r#"{"provider":null,"pair":["l"]}"#,"output":r#"{"left":"l","provider":"bnVsbA==","right":""}"#}),
        json!({"input":r#"{"provider":1,"provider":2,"pair":[]}"#,"pointer":"/input","rule":"input_syntax"}),
        json!({"input":r#"{"provider":"\ud800","pair":[]}"#,"pointer":"/input","rule":"input_syntax"}),
        json!({"input":r#"{"provider":{},"pair":[false,"r",1e999]}"#,"pointer":"/input/pair/0","rule":"positional_element_type"}),
        json!({"input":r#"{"provider":{},"pair":[null,"r",{"x":"\ud800"}]}"#,"pointer":"/input","rule":"input_syntax"}),
    ];
    for case in &cases {
        let result = plan
            .run_json("retain", case["input"].as_str().unwrap())
            .and_then(|encoded| plan.run_base64_json("adapt", encoded.as_str().unwrap()));
        if let Some(expected) = case["output"].as_str() {
            assert_eq!(result.unwrap().to_string(), expected, "{case}");
        } else {
            let errors = result.unwrap_err().0;
            assert_eq!(errors.len(), 1, "{case}");
            assert_eq!(errors[0].pointer, case["pointer"], "{case}");
            assert_eq!(errors[0].rule, case["rule"], "{case}");
        }
    }
    assert_eq!(
        plan.run_json("pair~/", r#"[[["a","b"],["c","d"]],[["e","f"]]]"#)
            .unwrap(),
        json!([[{"a/b~":"a"},{"a/b~":"c"}],[{"a/b~":"e"}]])
    );
    let root = native::write_package("adversary-reentry-position", &plan);
    let source = include_str!("fixtures/normalization_typescript_adversary.ts.txt")
        .replace("__CASES__", &serde_json::to_string(&cases).unwrap());
    std::fs::write(root.join("src/qualification.ts"), source).unwrap();
    native::compile(&root);
    native::run(&root);
}
