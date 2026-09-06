//! Lexical token provenance, admission, and retained-document composition.

#[path = "fixtures/normalization_raw.rs"]
mod fixture;

#[allow(dead_code)]
#[path = "../src/realize/normalize/legacy_v1_v3/recipe.rs.txt"]
mod old_reader;

use schema_contract::realize::normalize::Plan;
use serde_json::json;

#[test]
fn lexical_corpus_matches_independent_expected_values_and_findings() {
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
    eprintln!("raw JSON corpus executed {} cases", cases.len());
}

#[test]
fn explicit_capture_preserves_tokens_before_the_first_schema() {
    let plan = fixture::plan();
    assert_eq!(
        plan.run_json("root", " \n {\"n\":1e999,\"n\":2} \t")
            .unwrap(),
        "eyJuIjoxZTk5OSwibiI6Mn0="
    );
    assert_eq!(
        plan.run_json(
            "record",
            r#"{"payload":null,"items":[null,"x"],"nested":null}"#
        )
        .unwrap(),
        json!({"payload":"bnVsbA==","items":["bnVsbA==","Ingi"],"nested":null})
    );
    assert_eq!(plan.run_json("record", "{}").unwrap(), json!({}));
    let refused = plan.run("record", &json!({})).unwrap_err();
    assert_eq!(refused.0[0].rule, "input_capture_provenance");
}

#[test]
fn old_formats_refuse_even_empty_capture_declarations() {
    let (bundle, mut recipe) = fixture::fixture();
    for format in [
        "ess-normalization/1",
        "ess-normalization/2",
        "ess-normalization/3",
    ] {
        recipe["format"] = json!(format);
        recipe["raw_json_inputs"] = json!({});
        recipe.as_object_mut().unwrap().remove("binary64_inputs");
        let errors = Plan::read(&recipe.to_string(), std::slice::from_ref(&bundle)).unwrap_err();
        assert!(
            errors
                .0
                .iter()
                .any(|e| e.rule == "operation_version" && e.pointer == "/raw_json_inputs"),
            "{errors:?}"
        );
    }
}

#[test]
fn capture_paths_are_qualified_and_conflicts_are_explicit() {
    let (bundle, recipe) = fixture::fixture();
    for (paths, rule, pointer) in [
        (
            json!([[fixture::field("missing")]]),
            "unknown_field",
            "/raw_json_inputs/record/0/0",
        ),
        (
            json!([[fixture::field("number")]]),
            "input_policy_overlap",
            "/raw_json_inputs/record/0",
        ),
        (
            json!([[fixture::field("payload")], [fixture::field("payload")]]),
            "duplicate_capture_path",
            "/raw_json_inputs/record/1",
        ),
        (
            json!([
                [fixture::field("nested")],
                [fixture::field("nested"), fixture::field("body")]
            ]),
            "overlapping_capture_path",
            "/raw_json_inputs/record/1",
        ),
    ] {
        let mut recipe = recipe.clone();
        recipe["raw_json_inputs"]["record"] = paths;
        let errors = Plan::read(&recipe.to_string(), std::slice::from_ref(&bundle)).unwrap_err();
        assert!(
            errors
                .0
                .iter()
                .any(|e| e.rule == rule && e.pointer == pointer),
            "{errors:?}"
        );
    }
}

#[test]
fn capture_envelope_is_closed_and_legacy_readers_refuse() {
    let (bundle, recipe) = fixture::fixture();
    assert!(serde_json::from_value::<old_reader::Recipe>(recipe.clone()).is_err());
    let mut without_capture = recipe.clone();
    without_capture
        .as_object_mut()
        .unwrap()
        .remove("raw_json_inputs");
    let old: old_reader::Recipe = serde_json::from_value(without_capture.clone()).unwrap();
    assert!(![
        old_reader::FORMAT,
        old_reader::FORMAT_V2,
        old_reader::FORMAT_V3
    ]
    .contains(&old.format.as_str()));
    assert!(Plan::read(&without_capture.to_string(), std::slice::from_ref(&bundle)).is_ok());
    for declaration in [
        json!(null),
        json!([]),
        json!({"record":null}),
        json!({"record":[[{"kind":"index","index":0}]]}),
        json!({"record":[[{"kind":"items","name":"x"}]]}),
    ] {
        let mut recipe = recipe.clone();
        recipe["raw_json_inputs"] = declaration;
        assert_eq!(
            Plan::read(&recipe.to_string(), std::slice::from_ref(&bundle))
                .unwrap_err()
                .0[0]
                .rule,
            "recipe_syntax"
        );
    }
    let duplicate = recipe.to_string().replacen(
        "\"raw_json_inputs\":",
        "\"raw_json_inputs\":{\"root\":[[]],\"root\":[[]]},\"unused\":",
        1,
    );
    let errors = Plan::read(&duplicate, std::slice::from_ref(&bundle)).unwrap_err();
    assert!(errors.0[0]
        .detail
        .contains("duplicate normalization map key"));
    let mut unknown = recipe.clone();
    unknown["raw_json_inputs"]["not/a~branch"] = json!([[]]);
    assert!(Plan::read(&unknown.to_string(), &[bundle])
        .unwrap_err()
        .0
        .iter()
        .any(|e| e.rule == "unknown_branch" && e.pointer == "/raw_json_inputs/not~1a~0branch"));
}

#[test]
fn root_paths_leaf_types_and_both_overlap_directions_are_checked() {
    let (bundle, recipe) = fixture::fixture();
    let check = |branch: &str, paths: serde_json::Value, numbers: Option<serde_json::Value>| {
        let mut recipe = recipe.clone();
        recipe["raw_json_inputs"] = json!({branch:paths});
        recipe.as_object_mut().unwrap().remove("binary64_inputs");
        if let Some(numbers) = numbers {
            recipe["binary64_inputs"] = json!({branch:numbers});
        }
        Plan::read(&recipe.to_string(), std::slice::from_ref(&bundle))
    };
    for branch in ["number", "null", "plain", "record"] {
        assert!(check(branch, json!([[]]), None)
            .unwrap_err()
            .0
            .iter()
            .any(|e| e.rule == "capture_input_type"));
    }
    for paths in [
        json!([[], [fixture::field("payload")]]),
        json!([[fixture::field("payload")], []]),
    ] {
        let errors = check("record", paths, None).unwrap_err();
        assert!(errors
            .0
            .iter()
            .any(|e| e.rule == "overlapping_capture_path" && e.pointer.ends_with("/1")));
        assert!(!errors
            .0
            .iter()
            .any(|e| e.rule == "capture_input_type" && e.pointer.ends_with("/1")));
    }
    for (capture, numeric) in [
        (json!([[]]), json!([[fixture::field("number")]])),
        (json!([[fixture::field("payload")]]), json!([[]])),
    ] {
        let errors = check("record", capture, Some(numeric)).unwrap_err();
        assert!(errors
            .0
            .iter()
            .any(|e| e.rule == "input_policy_overlap"
                && e.detail.contains("/binary64_inputs/record/0")));
    }
    for (paths, rule) in [
        (json!([[{"kind":"items"}]]), "collection_type"),
        (
            json!([[fixture::field("payload"), fixture::field("x")]]),
            "object_type",
        ),
    ] {
        assert!(check("record", paths, None)
            .unwrap_err()
            .0
            .iter()
            .any(|e| e.rule == rule));
    }
    assert!(check("root", json!([vec![fixture::field("x"); 65]]), None)
        .unwrap_err()
        .0
        .iter()
        .any(|e| e.rule == "normalization_depth"));
}

#[test]
fn absent_null_and_wrong_intermediates_reach_the_declared_schema() {
    let plan = fixture::plan();
    for input in [r#"{"nested":false}"#, r#"{"items":{}}"#] {
        let errors = plan.run_json("record", input).unwrap_err();
        assert!(errors.0.iter().all(|e| e.rule == "schema_validation"));
        assert!(errors
            .0
            .iter()
            .all(|e| e.pointer.starts_with("/branches/record/0/input")));
    }
    assert_eq!(
        plan.run_json("model", r#"{"items":[],"raw/data~":null}"#)
            .unwrap(),
        json!({"items":[],"raw/data~":"bnVsbA=="})
    );
    assert_eq!(
        plan.run_json("model", r#"{"items":[]}"#).unwrap(),
        json!({"items":[]})
    );
    let outer = plan
        .run_json("root", r#"{"payload": {"n":1e999,"n":2}}"#)
        .unwrap();
    assert_eq!(
        plan.run_base64_json("record", outer.as_str().unwrap())
            .unwrap(),
        json!({"payload":"eyJuIjoxZTk5OSwibiI6Mn0="})
    );
}

#[test]
fn capture_is_explicit_and_refinements_apply_to_the_encoded_representation() {
    use schema_contract::bundle::{import, Dialect};
    use schema_contract::realize::normalize::Root;
    let source = json!({"components":{"schemas":{"Text":{"type":"string","maxLength":4}}}});
    let bundle = import(
        &source.to_string(),
        &["Text".to_owned()].into_iter().collect(),
        Dialect::Draft202012,
    )
    .unwrap();
    let root = Root::pin(&bundle, "Text").unwrap();
    let mut recipe = json!({"format":"ess-normalization/4","branches":{"copy":[{"input":root,"output":root,"requires":[],"value":{"op":"read","scope":"input","path":[]}}]}});
    for declaration in [None, Some(json!({})), Some(json!({"copy":[]}))] {
        if let Some(value) = declaration {
            recipe["raw_json_inputs"] = value;
        }
        let plan = Plan::read(&recipe.to_string(), std::slice::from_ref(&bundle)).unwrap();
        assert_eq!(plan.run("copy", &json!("eA==")).unwrap(), "eA==");
        assert!(plan
            .run_json("copy", "null")
            .unwrap_err()
            .0
            .iter()
            .all(|e| e.rule == "schema_validation"));
    }
    recipe["raw_json_inputs"] = json!({"copy":[[]]});
    let plan = Plan::read(&recipe.to_string(), &[bundle]).unwrap();
    assert_eq!(plan.run_json("copy", "1").unwrap(), "MQ==");
    assert!(plan
        .run_json("copy", "null")
        .unwrap_err()
        .0
        .iter()
        .all(|e| e.rule == "schema_validation" && e.pointer == "/branches/copy/0/input"));
}
