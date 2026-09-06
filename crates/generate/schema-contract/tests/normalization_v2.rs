//! Ordered text and collection behavior with strict version and scope boundaries.

#[path = "fixtures/normalization_v2.rs"]
mod fixture;

use schema_contract::realize::normalize::{Plan, FORMAT};
use serde_json::{json, Value};

#[test]
fn ordered_operations_preserve_source_indices_and_exact_construction() {
    let (bundle, recipe) = fixture::fixture();
    let plan = Plan::read(&recipe.to_string(), std::slice::from_ref(&bundle)).unwrap();
    assert_eq!(
        plan.to_json(),
        Plan::read(&plan.to_json(), &[bundle]).unwrap().to_json()
    );
    for (branch, input, expected) in fixture::expected() {
        assert_eq!(
            plan.run_json(branch, input).unwrap(),
            expected,
            "{branch}: {input}"
        );
    }
    let mut input: Value = serde_json::from_str(fixture::INPUT).unwrap();
    input["items"][0]["key"] = json!("a\u{301}\\[]");
    let before = input.clone();
    assert_eq!(
        plan.run("primary", &input).unwrap()["joined"],
        "a\u{301}\\[]||z"
    );
    assert_eq!(input, before);
}

#[test]
fn first_match_and_filter_do_not_evaluate_unselected_values() {
    let (bundle, recipe) = fixture::fixture();
    let plan = Plan::read(&recipe.to_string(), &[bundle]).unwrap();
    let mut input: Value = serde_json::from_str(fixture::INPUT).unwrap();
    input["items"][2]["n"] = json!(i64::MAX);
    let before = input.clone();
    assert_eq!(plan.run("find", &input).unwrap(), 3);
    assert_eq!(
        plan.run("select", &input).unwrap_err().0[0].rule,
        "integer_overflow"
    );
    assert_eq!(input, before);
    for item in input["items"].as_array_mut().unwrap() {
        item["enabled"] = json!(false);
    }
    assert_eq!(plan.run("find", &input).unwrap(), -1);
    assert_eq!(plan.run("select", &input).unwrap(), json!([]));
    assert_eq!(
        plan.run_json("integer", "1.0").unwrap_err().0[0].rule,
        "integer_representation"
    );
}

#[test]
fn version_one_cannot_smuggle_extended_operations_through_nested_expressions() {
    let (bundle, mut recipe) = fixture::fixture();
    recipe["format"] = json!(FORMAT);
    let errors = Plan::read(&recipe.to_string(), std::slice::from_ref(&bundle)).unwrap_err();
    assert!(errors
        .0
        .iter()
        .all(|error| error.rule == "operation_version"));
    assert!(errors.0.iter().any(|error| error
        .pointer
        .starts_with("/branches/primary/0/value/fields/")));
    recipe["branches"] = json!({"identity":[{
        "input":recipe["branches"]["integer"][0]["input"],
        "output":recipe["branches"]["integer"][0]["input"], "requires":[],
        "value":{"op":"read", "scope":"input", "path":[]}
    }]});
    let plan = Plan::read(&recipe.to_string(), &[bundle]).unwrap();
    assert_eq!(
        serde_json::from_str::<Value>(&plan.to_json()).unwrap(),
        recipe
    );
    assert_eq!(plan.run_json("identity", "7").unwrap(), 7);
}

#[test]
fn every_extended_operation_requires_version_two() {
    for replacement in [
        json!({"op":"concat","parts":[]}),
        json!({"op":"join","list":{"op":"list","items":[]},"separator":{"op":"string","value":""}}),
        json!({"op":"integer_string","value":{"op":"integer","value":0}}),
        json!({"op":"concat_lists","lists":[]}),
        json!({"op":"item_index"}),
        json!({"op":"select_map","list":{"op":"list","items":[]},"condition":{"op":"all","conditions":[]},"value":{"op":"null"}}),
        json!({"op":"find","list":{"op":"list","items":[]},"condition":{"op":"all","conditions":[]},"value":{"op":"null"},"otherwise":{"op":"null"}}),
    ] {
        let (bundle, recipe) = fixture::fixture();
        let mut stage = recipe["branches"]["integer"][0].clone();
        stage["value"] = replacement;
        let recipe = json!({"format":FORMAT,"branches":{"primary":[stage]}});
        let errors = Plan::read(&recipe.to_string(), &[bundle]).unwrap_err();
        assert_eq!(errors.0.len(), 1);
        assert_eq!(errors.0[0].rule, "operation_version");
        assert_eq!(errors.0[0].pointer, "/branches/primary/0/value");
    }
}

#[test]
fn extended_operands_and_scopes_check_before_any_execution() {
    for (replacement, rule) in [
        (json!({"op":"item_index"}), "item_scope"),
        (
            json!({"op":"concat", "parts":[{"op":"integer","value":1}]}),
            "string_type",
        ),
        (
            json!({"op":"join", "list":{"op":"list","items":[{"op":"null"}]}, "separator":{"op":"string","value":""}}),
            "string_type",
        ),
        (
            json!({"op":"concat_lists", "lists":[{"op":"string","value":"x"}]}),
            "collection_type",
        ),
        (
            json!({"op":"integer_string", "value":{"op":"string","value":"1"}}),
            "integer_type",
        ),
    ] {
        let (bundle, mut recipe) = fixture::fixture();
        recipe["branches"]["primary"][0]["value"] = replacement;
        let errors = Plan::read(&recipe.to_string(), &[bundle]).unwrap_err();
        assert!(
            errors.0.iter().any(|error| error.rule == rule),
            "{errors:?}"
        );
    }
    let (bundle, mut recipe) = fixture::fixture();
    recipe["branches"]["find"][0]["value"]["otherwise"] = json!({"op":"item_index"});
    let errors = Plan::read(&recipe.to_string(), &[bundle]).unwrap_err();
    assert!(errors
        .0
        .iter()
        .any(|error| error.rule == "item_scope" && error.pointer.ends_with("/otherwise")));
}

#[test]
fn selected_values_must_be_present_even_when_predicate_is_always_false() {
    let (bundle, mut recipe) = fixture::fixture();
    recipe["branches"]["select"][0]["value"]["condition"] = json!({"op":"any","conditions":[]});
    recipe["branches"]["select"][0]["value"]["value"] =
        json!({"op":"read","scope":"input","path":["optional"]});
    let errors = Plan::read(&recipe.to_string(), &[bundle]).unwrap_err();
    assert!(errors
        .0
        .iter()
        .any(|error| error.rule == "missing_value" && error.pointer.ends_with("/value")));
}
