//! Source-pinned normalization checks behavior, not merely generated declarations.

use schema_contract::bundle::{import, Bundle, Dialect};
use schema_contract::realize::normalize::{Plan, Root, FORMAT};
use serde_json::{json, Value};

fn bundle(schemas: Value, roots: &[&str]) -> Bundle {
    let mut source = json!({"openapi":"3.0.0", "components":{"schemas":null}});
    source["components"]["schemas"] = schemas;
    import(
        &source.to_string(),
        &roots.iter().map(|name| (*name).to_owned()).collect(),
        Dialect::Draft202012,
    )
    .unwrap()
}

fn stage(bundle: &Bundle, input: &str, output: &str, value: Value) -> Value {
    let mut stage = json!({"input": Root::pin(bundle, input).unwrap(),
        "output": Root::pin(bundle, output).unwrap(), "requires": [], "value": null});
    stage["value"] = value;
    stage
}

fn recipe(stages: Vec<Value>) -> Value {
    let mut recipe = json!({"format": FORMAT, "branches": {"primary": null}});
    recipe["branches"]["primary"] = Value::Array(stages);
    recipe
}

fn read(path: &[&str]) -> Value {
    json!({"op": "read", "scope": "input", "path": path})
}

fn integer(value: i64) -> Value {
    json!({"op": "integer", "value": value})
}

fn fallback(value: Value, replacement: Value, on_null: bool) -> Value {
    let mut fallback = json!({"op":"fallback", "on_null":on_null});
    fallback["value"] = value;
    fallback["fallback"] = replacement;
    fallback
}

fn check(value: &Value, bundle: &Bundle) -> Plan {
    Plan::read(&value.to_string(), std::slice::from_ref(bundle)).unwrap()
}

fn refuses(value: &Value, bundle: &Bundle, rule: &str) {
    let errors = Plan::read(&value.to_string(), std::slice::from_ref(bundle)).unwrap_err();
    assert!(
        errors.0.iter().any(|error| error.rule == rule),
        "{errors:?}"
    );
}

fn arithmetic(left: Value, right: Value, overflow: &str) -> Value {
    let mut arithmetic = json!({"op":"arithmetic", "operation":"multiply", "overflow":overflow});
    arithmetic["left"] = left;
    arithmetic["right"] = right;
    arithmetic
}

#[test]
fn fallback_preserves_false_zero_empty_null_and_absence_independently() {
    let schemas = bundle(
        json!({
            "Input": {"type": "object", "additionalProperties": false, "properties": {
                "label": {"type": ["string", "null"]}, "count": {"type": "integer"},
                "enabled": {"type": "boolean"}}},
            "Output": {"type": "object", "additionalProperties": false,
                "required": ["label", "count", "enabled"], "properties": {
                "label": {"type": ["string", "null"]}, "copy": {"type": ["string", "null"]},
                "count": {"type": "integer"}, "enabled": {"type": "boolean"}}}
        }),
        &["Input", "Output"],
    );
    let value = recipe(vec![stage(
        &schemas,
        "Input",
        "Output",
        json!({"op": "record", "fields": {
            "label": fallback(read(&["label"]), json!({"op": "string", "value": "seed"}), false),
            "copy": read(&["label"]), "count": fallback(read(&["count"]), integer(7), false),
            "enabled": fallback(read(&["enabled"]), json!({"op": "boolean", "value": true}), false)
        }}),
    )]);
    let plan = check(&value, &schemas);
    assert_eq!(
        plan.run("primary", &json!({})).unwrap(),
        json!({"label":"seed", "count":7, "enabled":true})
    );
    let original = json!({"label":"", "count":0, "enabled":false});
    let before = original.to_string();
    assert_eq!(
        plan.run("primary", &original).unwrap(),
        json!({"label":"", "copy":"", "count":0, "enabled":false})
    );
    assert_eq!(original.to_string(), before);
    assert_eq!(
        plan.run("primary", &json!({"label":null})).unwrap()["copy"],
        Value::Null
    );
    assert_eq!(
        plan.run("primary", &json!({"label":null})).unwrap()["label"],
        Value::Null
    );
    let mut replace_null = value;
    replace_null["branches"]["primary"][0]["value"]["fields"]["label"]["on_null"] = json!(true);
    assert_eq!(
        check(&replace_null, &schemas)
            .run("primary", &json!({"label":null}))
            .unwrap()["label"],
        "seed"
    );
}

#[test]
fn nullable_object_and_list_fallbacks_remain_usable_for_field_access_and_mapping() {
    let schemas = bundle(
        json!({
            "Input": {"type":"object", "additionalProperties":false, "properties": {
                "nested": {"anyOf": [{"type":"null"}, {"type":"object", "additionalProperties":false,
                    "properties":{"label":{"type":"string"}}, "required":["label"]}]},
                "items": {"type":["array", "null"], "items":{"type":"string"}}}},
            "Output": {"type":"object", "additionalProperties":false, "required":["label", "items"],
                "properties":{"label":{"type":"string"}, "items":{"type":"array", "items":{"type":"string"}}}}
        }),
        &["Input", "Output"],
    );
    let value = recipe(vec![stage(
        &schemas,
        "Input",
        "Output",
        json!({"op":"record", "fields": {
            "label": {"op":"field", "name":"label", "object": fallback(read(&["nested"]),
                json!({"op":"record", "fields":{"label":{"op":"string", "value":"default"}}}), true)},
            "items": {"op":"map", "list":fallback(read(&["items"]), json!({"op":"list", "items":[]}), true),
                "value":{"op":"read", "scope":"item", "path":[]}}
        }}),
    )]);
    let plan = check(&value, &schemas);
    assert_eq!(
        plan.run("primary", &json!({"nested":null, "items":null}))
            .unwrap(),
        json!({"label":"default", "items":[]})
    );
    assert_eq!(
        plan.run(
            "primary",
            &json!({"nested":{"label":"given"}, "items":["b", "a"]})
        )
        .unwrap(),
        json!({"label":"given", "items":["b", "a"]})
    );
    assert_eq!(
        plan.run("primary", &json!({})).unwrap(),
        json!({"label":"default", "items":[]})
    );
    let mut unsafe_path = value;
    unsafe_path["branches"]["primary"][0]["value"]["fields"]["label"] = read(&["nested", "label"]);
    refuses(&unsafe_path, &schemas, "object_type");
}

#[test]
fn stages_check_each_boundary_and_dispatch_has_no_implicit_default() {
    let schemas = bundle(
        json!({
            "Input":{"type":"object", "additionalProperties":false, "properties":{"count":{"type":"integer"}}},
            "Normalized":{"type":"integer"}, "Validated":{"type":"integer", "minimum":1, "maximum":10}
        }),
        &["Input", "Normalized", "Validated"],
    );
    let first = stage(
        &schemas,
        "Input",
        "Normalized",
        fallback(read(&["count"]), integer(1), false),
    );
    let second = stage(&schemas, "Normalized", "Validated", read(&[]));
    let mut value = recipe(vec![first.clone(), second.clone()]);
    value["branches"]["alias"] = json!([first, second]);
    let plan = check(&value, &schemas);
    assert_eq!(plan.run("alias", &json!({})).unwrap(), 1);
    assert_eq!(plan.run("primary", &json!({"count":3})).unwrap(), 3);
    assert_eq!(
        plan.run("unknown", &json!({})).unwrap_err().0[0].rule,
        "unknown_dispatch"
    );
    let invalid = json!({"count":0});
    let before = invalid.to_string();
    let errors = plan.run("primary", &invalid).unwrap_err();
    assert_eq!(errors.0[0].rule, "schema_validation");
    assert!(errors.0[0]
        .pointer
        .starts_with("/branches/primary/1/output"));
    assert_eq!(invalid.to_string(), before);
    let bytes = plan.to_json();
    let replayed_bundle = Bundle::read(&schemas.to_json().unwrap()).unwrap();
    let replayed = Plan::read(&bytes, &[replayed_bundle]).unwrap();
    assert_eq!(replayed.to_json(), bytes);
    assert_eq!(replayed.run("alias", &json!({})).unwrap(), 1);
    value["branches"]["primary"][1]["input"] = json!(Root::pin(&schemas, "Input").unwrap());
    refuses(&value, &schemas, "stage_identity");
}

#[test]
fn distinct_count_uses_selected_categories_and_mapping_retains_order() {
    let schemas = bundle(
        json!({
            "Input":{"type":"array", "items":{"type":"object", "additionalProperties":false,
                "required":["key","enabled"], "properties":{"key":{"type":"string"}, "enabled":{"type":"boolean"}}}},
            "Output":{"type":"object", "additionalProperties":false, "required":["count","keys"],
                "properties":{"count":{"type":"integer"}, "keys":{"type":"array", "items":{"type":"string"}}}}
        }),
        &["Input", "Output"],
    );
    let key = json!({"op":"read", "scope":"item", "path":["key"]});
    let value = recipe(vec![stage(
        &schemas,
        "Input",
        "Output",
        json!({"op":"record", "fields":{
            "count":{"op":"distinct_count", "list":read(&[]), "condition":{"op":"boolean",
                "value":{"op":"read", "scope":"item", "path":["enabled"]}}, "key":key},
            "keys":{"op":"map", "list":read(&[]), "value":key}
        }}),
    )]);
    let input = json!([{"key":"second", "enabled":true}, {"key":"first", "enabled":false},
        {"key":"second", "enabled":true}, {"key":"third", "enabled":true}]);
    assert_eq!(
        check(&value, &schemas).run("primary", &input).unwrap(),
        json!({"count":2, "keys":["second", "first", "second", "third"]})
    );
}

#[test]
fn arithmetic_requires_exact_signed_tokens_and_explicit_overflow_policy() {
    let schemas = bundle(json!({"Integer":{"type":"integer"}}), &["Integer"]);
    let value = recipe(vec![stage(
        &schemas,
        "Integer",
        "Integer",
        arithmetic(read(&[]), integer(1000), "reject"),
    )]);
    let plan = check(&value, &schemas);
    assert_eq!(plan.run("primary", &json!(3)).unwrap(), 3000);
    assert_eq!(
        plan.run("primary", &json!(i64::MAX)).unwrap_err().0[0].rule,
        "integer_overflow"
    );
    for token in ["1.0", "1e0", "9223372036854775808"] {
        let input = serde_json::from_str(token).unwrap();
        assert_eq!(
            plan.run("primary", &input).unwrap_err().0[0].rule,
            "integer_representation"
        );
    }
    let wrapping = recipe(vec![stage(
        &schemas,
        "Integer",
        "Integer",
        arithmetic(read(&[]), integer(2), "wrap"),
    )]);
    assert_eq!(
        check(&wrapping, &schemas)
            .run("primary", &json!(i64::MAX))
            .unwrap(),
        -2
    );
    let mut missing_policy = value;
    missing_policy["branches"]["primary"][0]["value"]
        .as_object_mut()
        .unwrap()
        .remove("overflow");
    refuses(&missing_policy, &schemas, "recipe_syntax");
}

#[test]
fn chosen_branches_and_conditions_are_lazy_but_all_are_type_checked() {
    let schemas = bundle(json!({"Integer":{"type":"integer"}}), &["Integer"]);
    let overflow = arithmetic(integer(i64::MAX), integer(2), "reject");
    let dangerous = json!({"op":"greater", "left":overflow, "right":integer(0)});
    let selected = json!({"op":"choose", "condition":{"op":"any", "conditions":[
        {"op":"all", "conditions":[]}, dangerous]}, "then_value":integer(7), "else_value":overflow});
    let value = recipe(vec![stage(&schemas, "Integer", "Integer", selected)]);
    assert_eq!(
        check(&value, &schemas).run("primary", &json!(0)).unwrap(),
        7
    );
    let mut wrong_branch = value;
    wrong_branch["branches"]["primary"][0]["value"]["else_value"] =
        json!({"op":"string", "value":"wrong"});
    refuses(&wrong_branch, &schemas, "output_type");
}

#[test]
fn explicit_requirements_run_before_transformation_and_null_is_present() {
    let schemas = bundle(
        json!({
            "Input":{"type":"object", "additionalProperties":false, "properties":{"label":{"type":["string","null"]}}},
            "Output":{"type":"integer"}
        }),
        &["Input", "Output"],
    );
    let mut value = recipe(vec![stage(&schemas, "Input", "Output", integer(1))]);
    value["branches"]["primary"][0]["requires"] = json!([
        {"op":"present", "value":read(&["label"])},
        {"op":"equal", "left":read(&["label"]), "right":{"op":"null"}}
    ]);
    let plan = check(&value, &schemas);
    assert_eq!(plan.run("primary", &json!({"label":null})).unwrap(), 1);
    assert_eq!(
        plan.run("primary", &json!({})).unwrap_err().0[0].pointer,
        "/branches/primary/0/requires/0"
    );
    assert_eq!(
        plan.run("primary", &json!({"label":"x"})).unwrap_err().0[0].pointer,
        "/branches/primary/0/requires/1"
    );
}

#[test]
fn strict_envelope_root_identity_and_every_branch_must_check() {
    let schemas = bundle(
        json!({
            "Input":{"type":"object", "additionalProperties":false, "properties":{"n":{"$ref":"#/components/schemas/Hidden"}}},
            "Hidden":{"type":"integer"}, "Output":{"type":"integer"}
        }),
        &["Input", "Output"],
    );
    let value = recipe(vec![stage(&schemas, "Input", "Output", integer(1))]);
    for (path, replacement, rule) in [
        ("/format", json!("ess-normalization/999"), "recipe_format"),
        (
            "/branches/primary/0/input/bundle_digest",
            json!("wrong"),
            "unknown_bundle",
        ),
        (
            "/branches/primary/0/input/root",
            json!("Hidden"),
            "root_selection",
        ),
        (
            "/branches/primary/0/value",
            read(&["unknown"]),
            "unknown_field",
        ),
        ("/branches/primary/0/value", read(&["n"]), "output_type"),
        (
            "/branches/primary/0/value",
            json!({"op":"read", "scope":"item", "path":[]}),
            "item_scope",
        ),
        ("/branches/primary", json!([]), "empty_pipeline"),
        ("/branches", json!({}), "empty_dispatch"),
    ] {
        let mut invalid = value.clone();
        *invalid.pointer_mut(path).unwrap() = replacement;
        refuses(&invalid, &schemas, rule);
    }
    let mut unknown = value.clone();
    unknown["branches"]["primary"][0]["value"]["extra"] = json!(true);
    refuses(&unknown, &schemas, "recipe_syntax");
    let mut other = value;
    other["branches"]["unused"] = json!([stage(&schemas, "Input", "Output", read(&["unknown"]))]);
    refuses(&other, &schemas, "unknown_field");
}

#[test]
fn additional_properties_cannot_bypass_a_named_output_field_type() {
    let schemas = bundle(
        json!({
            "Input":{"type":"object", "additionalProperties":{"type":"integer"}},
            "Output":{"type":"object", "properties":{"label":{"type":"string"}}}
        }),
        &["Input", "Output"],
    );
    refuses(
        &recipe(vec![stage(&schemas, "Input", "Output", read(&[]))]),
        &schemas,
        "output_type",
    );
}

#[test]
fn scalar_equality_has_no_target_dependent_structural_or_numeric_coercion() {
    let schemas = bundle(
        json!({
            "Input":{"type":"object", "additionalProperties":false, "properties": {
                "left":{"type":["string", "integer", "boolean", "null"]},
                "right":{"type":["string", "integer", "boolean", "null"]}}},
            "Output":{"type":"boolean"}
        }),
        &["Input", "Output"],
    );
    let value = recipe(vec![stage(
        &schemas,
        "Input",
        "Output",
        json!({
            "op":"choose", "condition":{"op":"equal", "left":read(&["left"]), "right":read(&["right"])},
            "then_value":{"op":"boolean", "value":true}, "else_value":{"op":"boolean", "value":false}
        }),
    )]);
    let plan = check(&value, &schemas);
    for (input, expected) in [
        (json!({}), false),
        (json!({"left":null}), false),
        (json!({"left":null, "right":null}), true),
        (json!({"left":true, "right":1}), false),
        (json!({"left":"1", "right":1}), false),
        (json!({"left":i64::MIN, "right":i64::MIN}), true),
    ] {
        assert_eq!(plan.run("primary", &input).unwrap(), expected);
    }
    assert_eq!(
        plan.run("primary", &json!({"left":1.0, "right":1}))
            .unwrap_err()
            .0[0]
            .rule,
        "integer_representation"
    );
    for operand in [read(&[]), json!({"op":"list", "items":[]})] {
        let mut invalid = value.clone();
        invalid["branches"]["primary"][0]["value"]["condition"]["left"] = operand;
        refuses(&invalid, &schemas, "equality_type");
    }
}

#[test]
fn duplicate_keys_cannot_hide_a_dispatch_branch_or_record_expression() {
    let schemas = bundle(json!({"Input":{"type":"object"}}), &["Input"]);
    let valid = stage(&schemas, "Input", "Input", read(&[])).to_string();
    let duplicate_branch =
        format!(r#"{{"format":"{FORMAT}","branches":{{"primary":[],"primary":[{valid}]}}}}"#);
    let invalid_record = r#"{"op":"record","fields":{"x":{"op":"read","scope":"item","path":[]},"x":{"op":"integer","value":1}}}"#;
    let duplicate_field = valid.replace(&read(&[]).to_string(), invalid_record);
    let duplicate_record =
        format!(r#"{{"format":"{FORMAT}","branches":{{"primary":[{duplicate_field}]}}}}"#);
    for text in [duplicate_branch, duplicate_record] {
        let errors = Plan::read(&text, std::slice::from_ref(&schemas)).unwrap_err();
        assert_eq!(errors.0[0].rule, "recipe_syntax");
        assert!(
            errors.0[0]
                .detail
                .contains("duplicate normalization map key"),
            "{errors:?}"
        );
    }
}

#[test]
fn raw_json_execution_refuses_precision_loss_before_even_an_unused_value_is_discarded() {
    let schemas = bundle(
        json!({"Input":true, "Output":{"type":"integer"}}),
        &["Input", "Output"],
    );
    let plan = check(
        &recipe(vec![stage(&schemas, "Input", "Output", integer(1))]),
        &schemas,
    );
    for text in [
        "18446744073709551617",
        "-9223372036854775809",
        "0.10000000000000001",
        "1e-999",
        "1e999",
        "{\"extra\":[18446744073709551617]}",
    ] {
        let errors = plan.run_json("primary", text).unwrap_err();
        assert_eq!(errors.0[0].rule, "input_number", "{text}: {errors:?}");
        assert!(!errors.0[0].detail.contains(text));
    }
    assert_eq!(
        plan.run_json("primary", r#"{"number":18446744073709551615}"#)
            .unwrap(),
        1
    );
}

#[test]
fn raw_json_execution_preserves_exact_values_and_integer_lexical_distinctions() {
    let schemas = bundle(
        json!({"Input":true, "Integer":{"type":"integer"}}),
        &["Input", "Integer"],
    );
    let copy = check(
        &recipe(vec![stage(&schemas, "Input", "Input", read(&[]))]),
        &schemas,
    );
    for text in [
        "-9223372036854775808",
        "9007199254740993",
        "18446744073709551615",
        "0.1",
        "0.125",
        "1e20",
        "-0.0",
        r#"{"n":0.125,"text":"1e999"}"#,
    ] {
        let out = copy.run_json("primary", text).unwrap();
        assert_eq!(out, serde_json::from_str::<Value>(text).unwrap());
    }
    let multiply = check(
        &recipe(vec![stage(
            &schemas,
            "Integer",
            "Integer",
            arithmetic(read(&[]), integer(2), "reject"),
        )]),
        &schemas,
    );
    assert_eq!(multiply.run_json("primary", "-0").unwrap(), 0);
    for text in ["-0.0", "0e0", "1.0", "1e0"] {
        assert_eq!(
            multiply.run_json("primary", text).unwrap_err().0[0].rule,
            "integer_representation"
        );
    }
}

#[test]
fn raw_json_execution_refuses_duplicate_keys_trailing_data_and_excessive_depth() {
    let schemas = bundle(json!({"Input":true}), &["Input"]);
    let plan = check(
        &recipe(vec![stage(&schemas, "Input", "Input", read(&[]))]),
        &schemas,
    );
    for text in [
        r#"{"key":1,"key":2}"#,
        r#"[{"key":1,"k\u0065y":2}]"#,
        "true false",
        "[1,]",
    ] {
        assert_eq!(
            plan.run_json("primary", text).unwrap_err().0[0].rule,
            "input_syntax"
        );
    }
    let deep = format!("{}0{}", "[".repeat(65), "]".repeat(65));
    assert_eq!(
        plan.run_json("primary", &deep).unwrap_err().0[0].rule,
        "input_depth"
    );
}

#[test]
fn prefix_selection_is_case_sensitive_and_does_not_parse_or_normalize_text() {
    let schemas = bundle(
        json!({
            "Input":{"type":"object", "additionalProperties":false, "required":["value", "prefix"],
                "properties":{"value":{"type":"string"}, "prefix":{"type":"string"}, "optional":{"type":["string", "null"]}}},
            "Output":{"type":"boolean"}
        }),
        &["Input", "Output"],
    );
    let recipe = recipe(vec![stage(
        &schemas,
        "Input",
        "Output",
        json!({
            "op":"choose", "condition":{"op":"starts_with", "value":read(&["value"]), "prefix":read(&["prefix"])},
            "then_value":{"op":"boolean", "value":true}, "else_value":{"op":"boolean", "value":false}
        }),
    )]);
    let plan = check(&recipe, &schemas);
    for (value, prefix, expected) in [
        ("proto:destination", "proto", true),
        ("protocol", "proto", true),
        ("Proto:destination", "proto", false),
        (" proto", "proto", false),
        ("", "", true),
        ("any", "", true),
        ("", "a", false),
        ("\u{00e9}x", "\u{00e9}", true),
        ("e\u{0301}x", "\u{00e9}", false),
    ] {
        assert_eq!(
            plan.run("primary", &json!({"value":value, "prefix":prefix}))
                .unwrap(),
            expected
        );
    }
    for operand in [read(&["optional"]), integer(0), json!({"op":"null"})] {
        let mut invalid = recipe.clone();
        invalid["branches"]["primary"][0]["value"]["condition"]["prefix"] = operand;
        refuses(&invalid, &schemas, "string_type");
    }
}
