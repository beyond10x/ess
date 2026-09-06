//! Explicit fixed-array input policy and checked positional reads.

use fixture::binary64;
#[path = "fixtures/normalization_positional.rs"]
mod fixture;
#[allow(dead_code)]
#[path = "../src/realize/normalize/legacy_v5/recipe.rs.txt"]
mod old_reader;

use schema_contract::realize::normalize::Plan;
use serde_json::{json, Value};

#[test]
fn positional_corpus_has_independent_expected_values_and_findings() {
    let plan = fixture::plan();
    let cases = fixture::cases();
    for (index, case) in cases.iter().enumerate() {
        let branch = case["branch"].as_str().unwrap();
        let input = case["input"].as_str().unwrap();
        let result = match case["mode"].as_str() {
            Some("base64") => plan.run_base64_json(branch, input),
            Some("value") => plan.run(branch, &serde_json::from_str(input).unwrap()),
            _ => plan.run_json(branch, input),
        };
        match result {
            Ok(value) => assert_eq!(Some(&value), case.get("value"), "case {index}: {case}"),
            Err(error) => assert_eq!(
                Some(json!(error.0)),
                case.get("errors").cloned(),
                "case {index}: {case}"
            ),
        }
    }
    eprintln!("positional corpus executed {} cases", cases.len());
}

#[test]
fn old_formats_refuse_even_empty_positional_declarations() {
    let (bundle, _) = fixture::fixture();
    let root = schema_contract::realize::normalize::Root::pin(&bundle, "Any").unwrap();
    for version in 1..=5 {
        let recipe = json!({"format":format!("ess-normalization/{version}"),"branches":{"main":[{"input":root,"output":root,"requires":[],"value":fixture::read(&[])}]},"positional_inputs":{}});
        let errors = Plan::read(&recipe.to_string(), std::slice::from_ref(&bundle)).unwrap_err();
        assert_eq!(
            json!(errors.0),
            json!([{"pointer":"/positional_inputs","rule":"operation_version","detail":"positional input declarations require ess-normalization/6"}])
        );
    }
}

#[test]
fn pure_tuple_mapping_does_not_require_original_text() {
    assert_eq!(
        fixture::plan().run("pure", &json!(["x", null])).unwrap(),
        json!(null)
    );
}

#[test]
fn lexical_preparation_composes_with_three_modeled_binary64_stages() {
    let plan = fixture::mixed_plan();
    for case in fixture::mixed_cases() {
        assert_eq!(
            plan.run_json("mixed", case["input"].as_str().unwrap())
                .unwrap(),
            case["value"]
        );
    }
}

fn schema_plan(
    schema: Value,
    value: Value,
    policy: Option<Value>,
) -> Result<Plan, schema_contract::realize::Refused> {
    let mut source = json!({"components":{"schemas":{"Output":true}}});
    source["components"]["schemas"]["Input"] = schema;
    let bundle = schema_contract::bundle::import(
        &source.to_string(),
        &["Input".to_owned(), "Output".to_owned()]
            .into_iter()
            .collect(),
        schema_contract::bundle::Dialect::Draft202012,
    )
    .unwrap();
    let root = |name| schema_contract::realize::normalize::Root::pin(&bundle, name).unwrap();
    let mut recipe = json!({"format":"ess-normalization/6","branches":{"main":[{"input":root("Input"),"output":root("Output"),"requires":[]}]}});
    recipe["branches"]["main"][0]["value"] = value;
    if let Some(policy) = policy {
        recipe["positional_inputs"] = json!({"main":[policy]});
    }
    Plan::read(&recipe.to_string(), &[bundle])
}

fn tuple() -> Value {
    json!({"type":"array","prefixItems":[{"type":"string"},{"type":"string"}],"items":false,"minItems":2,"maxItems":2})
}

#[test]
fn policy_and_position_require_exact_tuples_without_guessing() {
    let schemas = [
        json!({"type":"array","items":{"type":"string"}}),
        json!({"type":"array","items":false,"maxItems":0}),
        json!({"type":"array","prefixItems":[{"type":"string"},{"type":"string"}],"items":true,"minItems":2,"maxItems":2}),
        json!({"type":"array","prefixItems":[{"type":"string"},{"type":"string"}],"items":false,"maxItems":2}),
        json!({"type":"array","prefixItems":[{"type":"string"},{"type":"string"}],"items":false,"minItems":2}),
        json!({"anyOf":[tuple(),{"type":"null"}]}),
        json!({"anyOf":[tuple(),tuple()]}),
    ];
    for schema in schemas {
        let errors = schema_plan(
            schema.clone(),
            fixture::position(fixture::read(&[]), 0),
            None,
        )
        .unwrap_err();
        assert!(
            errors
                .0
                .iter()
                .any(|e| e.rule == "position_type" && e.pointer == "/branches/main/0/value/value"),
            "{schema}: {errors:?}"
        );
        let errors = schema_plan(
            schema.clone(),
            json!({"op":"null"}),
            Some(fixture::policy(json!([]))),
        )
        .unwrap_err();
        assert!(
            errors
                .0
                .iter()
                .any(|e| e.rule == "positional_schema"
                    && e.pointer == "/positional_inputs/main/0/path"),
            "{schema}: {errors:?}"
        );
    }
    for slot in [
        json!({"type":"number"}),
        json!({"type":["string","null"]}),
        json!(true),
    ] {
        let mut schema = tuple();
        schema["prefixItems"][0] = slot;
        let errors = schema_plan(
            schema,
            json!({"op":"null"}),
            Some(fixture::policy(json!([]))),
        )
        .unwrap_err();
        assert!(
            errors.0.iter().any(|e| e.rule == "positional_schema"),
            "{errors:?}"
        );
    }
    for length in [0, 1, 3, u64::MAX] {
        let mut policy = fixture::policy(json!([]));
        policy["length"] = json!(length);
        let errors = schema_plan(tuple(), json!({"op":"null"}), Some(policy)).unwrap_err();
        let rule = if length == 0 {
            "positional_length"
        } else {
            "positional_schema"
        };
        assert_eq!(errors.0[0].rule, rule);
    }
    for index in [2, u64::MAX] {
        let errors =
            schema_plan(tuple(), fixture::position(fixture::read(&[]), index), None).unwrap_err();
        assert_eq!(errors.0[0].rule, "position_index");
        assert_eq!(errors.0[0].pointer, "/branches/main/0/value/index");
    }
}

#[test]
fn tuple_reads_preserve_type_and_do_not_join_homogeneous_operations() {
    for value in [
        json!({"op":"map","list":fixture::read(&[]),"value":{"op":"string","value":"x"}}),
        json!({"op":"join","list":fixture::read(&[]),"separator":{"op":"string","value":""}}),
        json!({"op":"concat_lists","lists":[fixture::read(&[])]}),
        json!({"op":"select_map","list":fixture::read(&[]),"condition":{"op":"all","conditions":[]},"value":{"op":"string","value":"x"}}),
        json!({"op":"find","list":fixture::read(&[]),"condition":{"op":"all","conditions":[]},"value":{"op":"string","value":"x"},"otherwise":{"op":"string","value":""}}),
    ] {
        assert!(schema_plan(tuple(), value, None)
            .unwrap_err()
            .0
            .iter()
            .any(|e| e.rule == "collection_type"));
    }
    let mixed = json!({"type":"array","prefixItems":[{"type":"string"},{"type":"boolean"}],"items":false,"minItems":2,"maxItems":2});
    assert_eq!(
        schema_plan(mixed, fixture::position(fixture::read(&[]), 1), None)
            .unwrap()
            .run_json("main", r#"["a",true]"#)
            .unwrap(),
        json!(true)
    );
}

#[test]
fn source_tuple_alternatives_are_not_erased_by_member_or_fallback_typing() {
    let alternatives = json!({"anyOf":[tuple(),tuple()]});
    let fallback = json!({"op":"fallback","value":fixture::read(&[]),"fallback":fixture::read(&[]),"on_null":true});
    assert!(
        schema_plan(alternatives, fixture::position(fallback, 0), None)
            .unwrap_err()
            .0
            .iter()
            .any(|e| e.rule == "position_type")
    );
    let object = json!({"type":"object","additionalProperties":false,"required":["operands"],"properties":{"operands":tuple()}});
    let union = json!({"anyOf":[object,object]});
    assert!(schema_plan(
        union.clone(),
        fixture::position(fixture::read(&["operands"]), 0),
        None
    )
    .unwrap_err()
    .0
    .iter()
    .any(|e| e.rule == "position_type"));
    assert!(schema_plan(
        union,
        json!({"op":"null"}),
        Some(fixture::policy(json!([fixture::field("operands")])))
    )
    .unwrap_err()
    .0
    .iter()
    .any(|e| e.rule == "positional_schema"));
    let array = json!({"type":"array","items":tuple()});
    let union = json!({"anyOf":[array,array]});
    let mapped = json!({"op":"map","list":fixture::read(&[]),"value":fixture::position(json!({"op":"read","scope":"item","path":[]}),0)});
    assert!(schema_plan(union.clone(), mapped, None)
        .unwrap_err()
        .0
        .iter()
        .any(|e| e.rule == "position_type"));
    assert!(schema_plan(
        union,
        json!({"op":"null"}),
        Some(fixture::policy(json!([{"kind":"items"}])))
    )
    .unwrap_err()
    .0
    .iter()
    .any(|e| e.rule == "positional_schema"));
    // An expression choosing between the same independently exact tuple type
    // has one arity; refusing ambiguous source unions must not erase this proof.
    let choose = json!({"op":"choose","condition":{"op":"all","conditions":[]},"then_value":fixture::read(&[]),"else_value":fixture::read(&[])});
    assert_eq!(
        schema_plan(tuple(), fixture::position(choose, 1), None)
            .unwrap()
            .run_json("main", r#"["a","b"]"#)
            .unwrap(),
        json!("b")
    );
}

#[test]
fn positional_declarations_are_closed() {
    let (bundle, recipe) = fixture::fixture();
    for value in [
        json!(null),
        json!([]),
        json!({"pair":null}),
        json!({"pair":[{}]}),
    ] {
        let mut recipe = recipe.clone();
        recipe["positional_inputs"] = value;
        assert_eq!(
            Plan::read(&recipe.to_string(), std::slice::from_ref(&bundle))
                .unwrap_err()
                .0[0]
                .rule,
            "recipe_syntax"
        );
    }
    for field in [
        "path",
        "kind",
        "length",
        "missing",
        "null",
        "short",
        "extra",
        "null_element",
    ] {
        let mut recipe = recipe.clone();
        recipe["positional_inputs"]["pair"][0]
            .as_object_mut()
            .unwrap()
            .remove(field);
        assert_eq!(
            Plan::read(&recipe.to_string(), std::slice::from_ref(&bundle))
                .unwrap_err()
                .0[0]
                .rule,
            "recipe_syntax",
            "{field}"
        );
    }
    for (field, value) in [
        ("kind", json!("other")),
        ("missing", json!("zero")),
        ("null", json!("preserve")),
        ("short", json!("refuse")),
        ("extra", json!("keep")),
        ("null_element", json!("preserve")),
        ("length", json!(-1)),
        ("length", json!(2.0)),
        ("unknown", json!(true)),
        ("path", json!([{"kind":"items","name":"x"}])),
    ] {
        let mut recipe = recipe.clone();
        recipe["positional_inputs"]["pair"][0][field] = value;
        assert_eq!(
            Plan::read(&recipe.to_string(), std::slice::from_ref(&bundle))
                .unwrap_err()
                .0[0]
                .rule,
            "recipe_syntax",
            "{field}"
        );
    }
}

#[test]
fn paths_keep_exact_conflict_order() {
    let (bundle, recipe) = fixture::fixture();
    for (paths, rule, index) in [
        (
            json!([[fixture::field("operands")], [fixture::field("operands")]]),
            "duplicate_positional_path",
            1,
        ),
        (
            json!([[], [fixture::field("operands")]]),
            "overlapping_positional_path",
            1,
        ),
        (
            json!([[fixture::field("operands")], []]),
            "overlapping_positional_path",
            1,
        ),
        (json!([[fixture::field("raw")]]), "input_policy_overlap", 0),
        (
            json!([[fixture::field("number")]]),
            "input_policy_overlap",
            0,
        ),
        (json!([[fixture::field("unknown")]]), "unknown_field", 0),
        (
            json!([[fixture::field("operands"),{"kind":"items"}]]),
            "collection_type",
            0,
        ),
    ] {
        let mut recipe = recipe.clone();
        recipe["positional_inputs"]["record"] = Value::Array(
            paths
                .as_array()
                .unwrap()
                .iter()
                .cloned()
                .map(fixture::policy)
                .collect(),
        );
        let errors = Plan::read(&recipe.to_string(), std::slice::from_ref(&bundle)).unwrap_err();
        assert!(
            errors.0.iter().any(|e| e.rule == rule
                && e.pointer
                    .starts_with(&format!("/positional_inputs/record/{index}/path"))),
            "{errors:?}"
        );
    }
    let mut bad = recipe.clone();
    bad["positional_inputs"]["absent"] = json!([]);
    assert!(Plan::read(&bad.to_string(), std::slice::from_ref(&bundle))
        .unwrap_err()
        .0
        .iter()
        .any(|e| e.rule == "unknown_branch" && e.pointer == "/positional_inputs/absent"));
    let duplicate = recipe.to_string().replacen(
        "\"positional_inputs\":{",
        "\"positional_inputs\":{\"pair\":[],",
        1,
    );
    assert_eq!(
        Plan::read(&duplicate, &[bundle]).unwrap_err().0[0].rule,
        "recipe_syntax"
    );
}

#[test]
fn refinements_and_requiredness_run_after_preparation() {
    let mut schema = tuple();
    schema["prefixItems"][1]["minLength"] = json!(1);
    let plan = schema_plan(schema, fixture::read(&[]), Some(fixture::policy(json!([])))).unwrap();
    assert_eq!(
        plan.run_json("main", r#"["a"]"#).unwrap_err().0[0].rule,
        "schema_validation"
    );
    assert_eq!(
        fixture::plan().run_json("required", "{}").unwrap_err().0[0].rule,
        "schema_validation"
    );
    assert_eq!(
        plan.run_json("main", r#"["a","b",1e999]"#).unwrap(),
        json!(["a", "b"])
    );
}

#[test]
fn typed_construction_and_old_readers_cannot_bypass_version_or_path_checks() {
    let (bundle, recipe) = fixture::fixture();
    assert!(serde_json::from_value::<old_reader::Recipe>(recipe.clone()).is_err());
    let root = schema_contract::realize::normalize::Root::pin(&bundle, "Any").unwrap();
    let mut plain = json!({"format":"ess-normalization/6","branches":{"unused":[{"input":root,"output":root,"requires":[],"value":fixture::read(&[])}]}});
    let old: old_reader::Recipe = serde_json::from_value(plain.clone()).unwrap();
    assert!(![
        old_reader::FORMAT,
        old_reader::FORMAT_V2,
        old_reader::FORMAT_V3,
        old_reader::FORMAT_V4,
        old_reader::FORMAT_V5
    ]
    .contains(&old.format.as_str()));
    plain["branches"]["unused"][0]["value"] = fixture::position(fixture::read(&[]), u64::MAX);
    for version in 1..=5 {
        let mut typed: schema_contract::realize::normalize::Recipe =
            serde_json::from_value(plain.clone()).unwrap();
        typed.format = format!("ess-normalization/{version}");
        let errors = Plan::check(typed, std::slice::from_ref(&bundle)).unwrap_err();
        assert_eq!(
            json!(errors.0),
            json!([{"pointer":"/branches/unused/0/value","rule":"operation_version","detail":"position requires ess-normalization/6"}])
        );
    }
    let mut typed: schema_contract::realize::normalize::Recipe =
        serde_json::from_value(recipe.clone()).unwrap();
    typed
        .positional_inputs
        .as_mut()
        .unwrap()
        .get_mut("pair")
        .unwrap()[0]
        .length = 0;
    let errors = Plan::check(typed, std::slice::from_ref(&bundle)).unwrap_err();
    assert!(errors
        .0
        .iter()
        .any(|e| e.rule == "positional_length" && e.pointer == "/positional_inputs/pair/0/length"));
    for version in 1..=5 {
        let mut typed: schema_contract::realize::normalize::Recipe =
            serde_json::from_value(recipe.clone()).unwrap();
        typed.format = format!("ess-normalization/{version}");
        typed.positional_inputs = None;
        let errors = Plan::check(typed, std::slice::from_ref(&bundle)).unwrap_err();
        assert!(
            errors
                .0
                .iter()
                .any(|e| e.rule == "normalization_shape" && e.detail.contains("explicit tuple")),
            "{errors:?}"
        );
    }
    for index in [json!(-1), json!(0.0), json!("0")] {
        plain["branches"]["unused"][0]["value"]["index"] = index;
        assert_eq!(
            Plan::read(&plain.to_string(), std::slice::from_ref(&bundle))
                .unwrap_err()
                .0[0]
                .rule,
            "recipe_syntax"
        );
    }
}

#[test]
fn format_six_inherits_all_existing_binary64_vectors() {
    let (model, mut recipe) = binary64::fixture();
    recipe["format"] = json!("ess-normalization/6");
    let plan =
        Plan::check_with_models(serde_json::from_value(recipe).unwrap(), &[], &[model]).unwrap();
    let cases = binary64::cases();
    for case in &cases {
        let result = plan.run_json(
            case["branch"].as_str().unwrap(),
            case["input"].as_str().unwrap(),
        );
        if let Some(rule) = case["error"].as_str() {
            assert!(
                result.unwrap_err().0.iter().any(|error| error.rule == rule),
                "{case}"
            );
        } else {
            let value = result.unwrap();
            if let Some(bits) = case["bits"].as_str() {
                assert_eq!(
                    value.as_f64().unwrap().to_bits(),
                    u64::from_str_radix(bits, 16).unwrap(),
                    "{case}"
                );
                assert!(value.as_i64().is_none());
            } else if let Some(bits) = case["array_bits"].as_array() {
                for (value, bits) in value.as_array().unwrap().iter().zip(bits) {
                    if bits.is_null() {
                        assert!(value.is_null());
                    } else {
                        assert_eq!(
                            value.as_f64().unwrap().to_bits(),
                            u64::from_str_radix(bits.as_str().unwrap(), 16).unwrap()
                        );
                    }
                }
            } else {
                assert_eq!(Some(&value), case.get("value"), "{case}");
            }
        }
    }
    assert!(!plan.to_json().contains("position_arities"));
    eprintln!(
        "format6 inherited Binary64 corpus executed {} cases",
        cases.len()
    );
}
