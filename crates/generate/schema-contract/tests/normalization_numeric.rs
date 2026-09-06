//! Explicit numeric decoding is source-selected, ordered and separate from integer arithmetic.

#[path = "fixtures/normalization_numeric.rs"]
mod fixture;

use schema_contract::realize::normalize::Plan;
use serde_json::{json, Value};

#[test]
fn rounded_conversion_preserves_pipeline_order_and_exact_unselected_integers() {
    let (bundle, recipe) = fixture::fixture();
    let plan = Plan::read(&recipe.to_string(), &[bundle]).unwrap();
    for (branch, input, expected) in fixture::expected() {
        assert_eq!(
            plan.run_json(branch, input).unwrap(),
            expected,
            "{branch}: {input}"
        );
    }
    let input = json!({"value":9_007_199_254_740_993_i64,"integer":9_007_199_254_740_993_i64});
    let before = input.clone();
    let output = plan.run("copy", &input).unwrap();
    assert_eq!(
        output["value"].as_f64().unwrap().to_bits(),
        9_007_199_254_740_992_f64.to_bits()
    );
    assert_eq!(output["integer"], input["integer"]);
    assert_eq!(input, before);
    assert!(
        plan.run_json("copy", r#"{"value":-1e-999}"#).unwrap()["value"]
            .as_f64()
            .unwrap()
            .is_sign_negative()
    );
}

#[test]
fn finite_range_and_undeclared_precision_loss_refuse_without_partial_results() {
    let (bundle, recipe) = fixture::fixture();
    let plan = Plan::read(&recipe.to_string(), &[bundle]).unwrap();
    for (branch, input, rule) in [
        ("truncate", "9223372036854775807", "binary64_range"),
        ("truncate", "9223372036854775808", "binary64_range"),
        ("truncate", "-9223372036854777856", "binary64_range"),
        ("truncate", "1e999", "input_number"),
        ("overflow", "2", "binary64_overflow"),
        ("copy", r#"{"exact":0.10000000000000001}"#, "input_number"),
        ("copy", r#"{"value":1,"value":2}"#, "input_syntax"),
    ] {
        assert_eq!(
            plan.run_json(branch, input).unwrap_err().0[0].rule,
            rule,
            "{branch}: {input}"
        );
    }
}

#[test]
fn decoding_paths_and_tokens_are_checked_not_guessed() {
    for (paths, rule) in [
        (json!({"missing":[[]]}), "unknown_branch"),
        (
            json!({"copy":[[{"kind":"field","name":"missing"}]]}),
            "unknown_field",
        ),
        (json!({"copy":[[]]}), "numeric_input_type"),
        (json!({"copy":[[{"kind":"items"}]]}), "collection_type"),
        (json!({"truncate":[[],[]]}), "duplicate_numeric_path"),
    ] {
        let (bundle, mut recipe) = fixture::fixture();
        recipe["binary64_inputs"] = paths;
        assert!(Plan::read(&recipe.to_string(), &[bundle])
            .unwrap_err()
            .0
            .iter()
            .any(|e| e.rule == rule));
    }
    for token in [
        "NaN", "Infinity", "1e999", "01", " 1", "1 ", "null", "\"1\"",
    ] {
        let (bundle, mut recipe) = fixture::fixture();
        recipe["branches"]["upper"][0]["value"]["steps"][0]["value"] = json!(token);
        assert!(
            Plan::read(&recipe.to_string(), &[bundle])
                .unwrap_err()
                .0
                .iter()
                .any(|e| e.rule == "binary64_literal"),
            "{token}"
        );
    }
}

#[test]
fn version_one_refuses_even_an_empty_numeric_declaration_and_conversion() {
    for policy in [json!({}), json!(null)] {
        let (bundle, mut recipe) = fixture::fixture();
        recipe["format"] = json!("ess-normalization/1");
        recipe["branches"] = json!({"copy":recipe["branches"]["copy"]});
        recipe["binary64_inputs"] = policy.clone();
        let expected = if policy.is_null() {
            "recipe_syntax"
        } else {
            "operation_version"
        };
        assert_eq!(
            Plan::read(&recipe.to_string(), &[bundle]).unwrap_err().0[0].rule,
            expected
        );
    }
    let (bundle, mut recipe) = fixture::fixture();
    recipe["format"] = json!("ess-normalization/1");
    recipe.as_object_mut().unwrap().remove("binary64_inputs");
    let errors = Plan::read(&recipe.to_string(), &[bundle]).unwrap_err();
    assert!(errors.0.iter().all(|e| e.rule == "operation_version"));
}

#[test]
fn numeric_recipe_round_trip_preserves_authored_decimal_tokens() {
    let (bundle, recipe) = fixture::fixture();
    let plan = Plan::read(&recipe.to_string(), std::slice::from_ref(&bundle)).unwrap();
    assert_eq!(
        serde_json::from_str::<Value>(&plan.to_json()).unwrap(),
        recipe
    );
    assert_eq!(
        Plan::read(&plan.to_json(), &[bundle]).unwrap().to_json(),
        plan.to_json()
    );
}
