//! Modeled finite floating values retain identity through explicit construction.

#[path = "fixtures/normalization_binary64.rs"]
mod fixture;
use fixture::model;

use schema_contract::realize::normalize::Plan;
use serde_json::{json, Value};

#[test]
fn modeled_defaults_and_input_tokens_preserve_finite_bits() {
    let plan = fixture::plan();
    let cases = fixture::cases();
    for case in &cases {
        let branch = case["branch"].as_str().unwrap();
        let result = plan.run_json(branch, case["input"].as_str().unwrap());
        if let Some(rule) = case["error"].as_str() {
            assert_eq!(result.unwrap_err().0[0].rule, rule, "{case}");
            continue;
        }
        let value = result.unwrap_or_else(|error| panic!("{case}: {error:?}"));
        check_value(case, &value);
        if let Some(bits) = case["bits"].as_str() {
            assert_eq!(
                plan.run_json("scalar", &value.to_string())
                    .unwrap()
                    .as_f64()
                    .unwrap()
                    .to_bits(),
                u64::from_str_radix(bits, 16).unwrap()
            );
        }
    }
    eprintln!(
        "reference modeled Binary64: {} independently specified corpus cases",
        cases.len()
    );
}

fn check_value(case: &Value, value: &Value) {
    if let Some(bits) = case["bits"].as_str() {
        assert_eq!(
            value.as_f64().unwrap().to_bits(),
            u64::from_str_radix(bits, 16).unwrap(),
            "{case}"
        );
        assert!(value.as_i64().is_none(), "floating representation: {value}");
        if bits == "8000000000000000" {
            assert_eq!(value.to_string(), "-0.0");
        }
    } else if let Some(expected) = case["array_bits"].as_array() {
        for (value, bits) in value.as_array().unwrap().iter().zip(expected) {
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
        assert_eq!(*value, case["value"], "{case}");
    }
}

#[test]
fn modeled_numeric_policy_is_required_even_for_unused_optional_fields() {
    let (model, mut recipe) = fixture::fixture();
    recipe["binary64_inputs"]["main"] = json!([]);
    recipe["branches"]["main"][0]["value"] = json!({"op":"binary64_literal","value":"0.0"});
    let errors = Plan::check_with_models(serde_json::from_value(recipe).unwrap(), &[], &[model])
        .err()
        .unwrap();
    assert!(
        errors.0.iter().any(|e| e.rule == "model_binary64_policy"
            && e.pointer == "/binary64_inputs/main"
            && e.detail.contains("a/b~")),
        "{errors:?}"
    );
}

#[test]
fn floating_construction_requires_the_new_recipe_format() {
    let (model, recipe) = fixture::fixture();
    for version in 1..=4 {
        let mut recipe = recipe.clone();
        recipe["format"] = json!(format!("ess-normalization/{version}"));
        let errors = Plan::check_with_models(
            serde_json::from_value(recipe).unwrap(),
            &[],
            std::slice::from_ref(&model),
        )
        .unwrap_err();
        assert!(
            errors.0.iter().any(|e| e.rule == "model_binary64_version"),
            "{errors:?}"
        );
    }
    let selected = model::selection(
        fixture::SOURCE,
        &["sample.float.Count", "sample.float.Output"],
    );
    let input =
        schema_contract::realize::normalize::Root::pin_model(&selected, "sample.float.Count")
            .unwrap();
    let output =
        schema_contract::realize::normalize::Root::pin_model(&selected, "sample.float.Output")
            .unwrap();
    for version in 1..=4 {
        let recipe = json!({"format":format!("ess-normalization/{version}"),"branches":{"only_output":[{"input":input,"output":output,"requires":[],"value":{"op":"binary64_literal","value":"0.0"}}]}});
        let errors = Plan::check_with_models(
            serde_json::from_value(recipe).unwrap(),
            &[],
            std::slice::from_ref(&selected),
        )
        .unwrap_err();
        assert!(
            errors
                .0
                .iter()
                .any(|error| error.rule == "model_binary64_version"),
            "output-only selection: {errors:?}"
        );
    }
}

#[test]
fn literal_admission_and_numeric_assignment_are_checked_in_lazy_branches() {
    let (model, recipe) = fixture::fixture();
    for token in ["1e999", "NaN", "Infinity", "01", " 0", "0 ", "+0", "0.0x"] {
        let mut recipe = recipe.clone();
        recipe["branches"]["main"][0]["value"]["fallback"]["value"] = json!(token);
        let errors = Plan::check_with_models(
            serde_json::from_value(recipe).unwrap(),
            &[],
            std::slice::from_ref(&model),
        )
        .unwrap_err();
        assert!(
            errors.0.iter().any(|e| e.rule == "binary64_literal"
                && e.pointer == "/branches/main/0/value/fallback/value"),
            "{token}: {errors:?}"
        );
    }
    for expression in [
        json!({"op":"integer","value":0}),
        json!({"op":"string","value":"0.0"}),
    ] {
        let mut recipe = recipe.clone();
        recipe["branches"]["main"][0]["value"] = expression;
        assert!(Plan::check_with_models(
            serde_json::from_value(recipe).unwrap(),
            &[],
            std::slice::from_ref(&model)
        )
        .unwrap_err()
        .0
        .iter()
        .any(|e| e.rule == "output_type"));
    }
    let mut mixed = recipe;
    mixed["branches"]["equal"][0]["value"]["condition"]["right"] =
        json!({"op":"integer","value":0});
    assert!(
        Plan::check_with_models(serde_json::from_value(mixed).unwrap(), &[], &[model])
            .unwrap_err()
            .0
            .iter()
            .any(|e| e.rule == "equality_type")
    );
}

#[test]
fn floating_integral_results_do_not_become_integer_operands() {
    let (model, mut recipe) = fixture::fixture();
    for expression in [
        json!({"op":"binary64_literal","value":"1.0"}),
        json!({"op":"binary64","value":{"op":"integer","value":1},"steps":[]}),
    ] {
        recipe["branches"]["cast"][0]["value"] = json!({"op":"arithmetic","operation":"multiply","overflow":"reject","left":expression,"right":{"op":"integer","value":2}});
        assert!(Plan::check_with_models(
            serde_json::from_value(recipe.clone()).unwrap(),
            &[],
            std::slice::from_ref(&model)
        )
        .unwrap_err()
        .0
        .iter()
        .any(|e| e.rule == "integer_type"));
    }
}

#[test]
fn compiler_metadata_and_finite_codecs_cannot_be_minted_by_schema_annotations() {
    let (model, _) = fixture::fixture();
    assert!(model
        .binary64_locations()
        .contains("/$defs/sample.float.Output"));
    assert!(model
        .binary64_locations()
        .contains("/$defs/sample.float.Items/items/anyOf/0"));
    let structural = schema_contract::realize::Plan::from_model(&model).unwrap();
    assert!(structural
        .rust("adapter")
        .unwrap()
        .declarations
        .contains("pub struct EssBinary64(f64)"));
    assert!(structural
        .go("adapter", "example.invalid/adapter")
        .unwrap()
        .declarations
        .contains("type EssBinary64 struct"));
    let report = json!(structural.typescript().report);
    assert!(report["obligations"]
        .as_array()
        .unwrap()
        .iter()
        .any(|e| e["rule"] == "model_binary64"));
    assert_eq!(model.definitions()["sample.float.Output"]["type"], "number");
    let selected = std::collections::BTreeSet::from(["Plain".to_owned()]);
    let source = json!({"openapi":"3.0.0","components":{"schemas":{"Plain":{"type":"number","x-ess-binary64":true}}}});
    assert!(
        schema_contract::bundle::import(
            &source.to_string(),
            &selected,
            schema_contract::bundle::Dialect::Draft202012
        )
        .is_err(),
        "unsupported metadata must not become authority"
    );
    let source = json!({"openapi":"3.0.0","components":{"schemas":{"Plain":{"type":"number","examples":[{"x-ess-binary64":true}]}}}});
    let imported = schema_contract::bundle::import(
        &source.to_string(),
        &selected,
        schema_contract::bundle::Dialect::Draft202012,
    )
    .unwrap();
    let plain = schema_contract::realize::Plan::from_bundle(&imported, &selected).unwrap();
    assert!(!json!(plain.typescript().report)["obligations"]
        .as_array()
        .unwrap()
        .iter()
        .any(|e| e["rule"] == "model_binary64"));
    plain.rust("plain").unwrap();
    plain.go("plain", "example.invalid/plain").unwrap();
}

#[test]
fn new_expressions_refuse_old_formats_without_any_binary64_model_in_the_recipe() {
    let selected = std::collections::BTreeSet::from(["Plain".to_owned()]);
    let source = json!({"openapi":"3.0.0","components":{"schemas":{"Plain":{"type":"number"}}}});
    let bundle = schema_contract::bundle::import(
        &source.to_string(),
        &selected,
        schema_contract::bundle::Dialect::Draft202012,
    )
    .unwrap();
    let root = schema_contract::realize::normalize::Root::pin(&bundle, "Plain").unwrap();
    let base = json!({"format":"ess-normalization/1","branches":{"primary":[{"input":root,"output":root,"requires":[],"value":null}]}});
    for version in 1..=4 {
        for expression in [
            json!({"op":"binary64_literal","value":"0.0"}),
            json!({"op":"binary64","value":{"op":"integer","value":0},"steps":[]}),
        ] {
            let mut recipe = base.clone();
            recipe["format"] = json!(format!("ess-normalization/{version}"));
            recipe["branches"]["primary"][0]["value"] = expression;
            let errors =
                Plan::read(&recipe.to_string(), std::slice::from_ref(&bundle)).unwrap_err();
            assert!(
                errors.0.iter().any(|e| e.rule == "operation_version"),
                "{errors:?}"
            );
        }
    }
}

#[test]
fn inaccessible_map_and_union_input_policies_refuse_before_generation() {
    use schema_contract::realize::normalize::Root;
    for body in [
        "kind: newtype\n    of: Map<String, Binary64>",
        "kind: union\n    tag: kind\n    variants: {left: Binary64, right: String}",
    ] {
        let source = format!("format: ess/2\nsystem: sample\nversion: v1\ndomains: [sample.float]\ndomain: sample.float\ntypes:\n  - name: sample.float.Input\n    {body}\n");
        let model = model::selection(&source, &["sample.float.Input"]);
        let root = Root::pin_model(&model, "sample.float.Input").unwrap();
        let recipe = json!({"format":"ess-normalization/5","branches":{"main":[{"input":root,"output":root,"requires":[],"value":{"op":"read","scope":"input","path":[]}}]}});
        let errors =
            Plan::check_with_models(serde_json::from_value(recipe).unwrap(), &[], &[model])
                .unwrap_err();
        assert!(
            errors.0.iter().any(|e| e.rule == "model_binary64_path"),
            "{errors:?}"
        );
    }
}
