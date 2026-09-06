//! A sealed model, not an imported projection's annotations, owns each model root.

#[path = "fixtures/normalization_model.rs"]
mod fixture;

use schema_contract::bundle::{import, Dialect};
use schema_contract::realize::normalize::{Plan, Recipe, Root};
use serde_json::{json, Value};

#[test]
fn checked_models_preserve_wire_names_units_and_absent_fields() {
    let plan = fixture::plan();
    for case in fixture::cases().as_array().unwrap() {
        let result = plan.run_json("primary", case["input"].as_str().unwrap());
        if let Some(value) = case.get("value") {
            assert_eq!(&result.unwrap(), value);
        } else {
            assert_eq!(
                serde_json::to_value(result.unwrap_err().0).unwrap(),
                case["errors"]
            );
        }
    }
    let (model, _) = fixture::fixture();
    let parsed: Recipe = serde_json::from_str(&plan.to_json()).unwrap();
    assert_eq!(
        plan.to_json(),
        Plan::check_with_models(parsed, &[], &[model])
            .unwrap()
            .to_json()
    );
}

#[test]
fn every_model_identity_coordinate_is_rechecked() {
    let (model, recipe) = fixture::fixture();
    for field in [
        "system",
        "specification_version",
        "source_digest",
        "contract_digest",
        "projection_digest",
        "roots",
    ] {
        let mut changed = recipe.clone();
        changed["branches"]["primary"][0]["input"]["model"][field] = if field == "roots" {
            json!(["sample.settings.Decoded"])
        } else {
            json!("stale")
        };
        let errors = Plan::check_with_models(
            serde_json::from_value(changed).unwrap(),
            &[],
            std::slice::from_ref(&model),
        )
        .unwrap_err()
        .0;
        assert!(
            errors.iter().any(|error| error.rule == "unknown_model"),
            "{field}: {errors:?}"
        );
    }
    assert!(Plan::read(&recipe.to_string(), &[])
        .unwrap_err()
        .0
        .iter()
        .any(|error| error.rule == "unknown_model"));
    assert!(Root::pin_model(&model, "sample.settings.Id").is_err());
    let mut changed = recipe;
    changed["branches"]["primary"][0]["input"]["root"] = json!("sample.settings.Id");
    assert!(
        Plan::check_with_models(serde_json::from_value(changed).unwrap(), &[], &[model])
            .unwrap_err()
            .0
            .iter()
            .any(|error| error.rule == "unselected_root")
    );
}

#[test]
fn old_envelopes_and_old_root_reader_refuse_model_identity() {
    #[derive(serde::Deserialize)]
    #[serde(deny_unknown_fields)]
    struct LegacyRoot {
        #[serde(rename = "bundle_digest")]
        _bundle_digest: String,
        #[serde(rename = "root")]
        _root: String,
    }
    let (model, recipe) = fixture::fixture();
    assert!(serde_json::from_value::<LegacyRoot>(
        recipe["branches"]["primary"][0]["input"].clone()
    )
    .is_err());
    for format in ["ess-normalization/1", "ess-normalization/2"] {
        let mut changed = recipe.clone();
        changed["format"] = json!(format);
        assert!(Plan::check_with_models(
            serde_json::from_value(changed).unwrap(),
            &[],
            std::slice::from_ref(&model)
        )
        .unwrap_err()
        .0
        .iter()
        .any(|error| error.rule == "root_version"));
    }
    let mut ambiguous = recipe["branches"]["primary"][0]["input"].clone();
    ambiguous["bundle_digest"] = json!("forged");
    assert!(serde_json::from_value::<Root>(ambiguous).is_err());
}

#[test]
fn unevaluated_invariants_cannot_become_a_successful_normalizer() {
    let model = fixture::selection(fixture::SOURCE, &["sample.settings.Guarded"]);
    let root = Root::pin_model(&model, "sample.settings.Guarded").unwrap();
    let recipe = json!({"format":"ess-normalization/3", "branches":{"copy":[{"input":root,"output":root,"requires":[],"value":{"op":"read","scope":"input","path":[]}}]}});
    let errors = Plan::check_with_models(serde_json::from_value(recipe).unwrap(), &[], &[model])
        .unwrap_err()
        .0;
    assert!(errors.iter().any(|error| error.rule == "model_invariants"
        && error
            .pointer
            .ends_with("/sample.settings.Guarded/x-ess-invariants")));
}

#[test]
fn checked_model_enums_retain_membership_without_flattening_imported_intersections() {
    let model = fixture::selection(fixture::SOURCE, &["sample.settings.Mode"]);
    let root = Root::pin_model(&model, "sample.settings.Mode").unwrap();
    let recipe = json!({"format":"ess-normalization/3", "branches":{"copy":[{"input":root,"output":root,"requires":[],"value":{"op":"read","scope":"input","path":[]}}]}});
    let plan = Plan::check_with_models(
        serde_json::from_value(recipe.clone()).unwrap(),
        &[],
        &[model],
    )
    .unwrap();
    for member in ["ready", "paused"] {
        assert_eq!(plan.run("copy", &json!(member)).unwrap(), json!(member));
    }
    for value in [json!("other"), json!(null), json!(0)] {
        assert_eq!(
            plan.run("copy", &value).unwrap_err().0[0].rule,
            "schema_validation"
        );
    }
    let source =
        json!({"components":{"schemas":{"Mode":{"type":"string","enum":["ready","paused"]}}}});
    let bundle = import(
        &source.to_string(),
        &["Mode".to_owned()].into_iter().collect(),
        Dialect::Draft202012,
    )
    .unwrap();
    let root = Root::pin(&bundle, "Mode").unwrap();
    let mut imported = recipe;
    imported["branches"]["copy"][0]["input"] = json!(root);
    imported["branches"]["copy"][0]["output"] = json!(root);
    assert!(Plan::read(&imported.to_string(), &[bundle])
        .unwrap_err()
        .0
        .iter()
        .any(|error| error.rule == "normalization_shape"));
}

#[test]
fn qualified_sources_can_normalize_into_model_owned_records() {
    let (model, mut recipe) = fixture::fixture();
    let source = json!({"components":{"schemas":{"Source":{"type":"object","additionalProperties":false,"required":["identifier","seconds"],"properties":{"identifier":{"type":"string"},"seconds":{"type":"integer"},"label":{"type":"string"}}}}}});
    let bundle = import(
        &source.to_string(),
        &["Source".to_owned()].into_iter().collect(),
        Dialect::Draft202012,
    )
    .unwrap();
    recipe["branches"]["primary"][0]["input"] = json!(Root::pin(&bundle, "Source").unwrap());
    let plan =
        Plan::check_with_models(serde_json::from_value(recipe).unwrap(), &[bundle], &[model])
            .unwrap();
    assert_eq!(
        plan.run_json("primary", r#"{"identifier":"x","seconds":4}"#)
            .unwrap(),
        json!({"key":"x","elapsed-ms":4000,"mode":"ready"})
    );
    for target in [
        plan.rust("adapter").unwrap(),
        plan.go("adapter", "example.invalid/adapter").unwrap(),
    ] {
        let report: Value = serde_json::to_value(&target.report).unwrap();
        assert_eq!(report["format"], "ess-normalization-target/2");
        assert!(target
            .files
            .keys()
            .any(|name| name.ends_with(".bundle.json")));
        let (_, model_source) = target
            .files
            .iter()
            .find(|(name, _)| name.starts_with("sources/model-"))
            .unwrap();
        let model_source: Value = serde_json::from_str(model_source).unwrap();
        assert_eq!(
            model_source["$defs"]["sample.settings.Id"]["x-ess-kind"],
            "newtype"
        );
        assert!(model_source.get("x-ess-provenance").is_some());
    }
}
