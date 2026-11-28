//! Execute the actual standalone Rust adapter against reference results and refusals.
//!
//! No test builds an adapter: `task schema-contract-native-crates` builds every crate of
//! [`native_crates`] once, before the tests (`support/native_crate.rs`).

use schema_contract::realize::normalize::Plan;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

#[path = "support/native_crate.rs"]
mod native_crate;
use native_crate::NativeCrate;

#[path = "fixtures/normalization_v1.rs"]
mod fixture_v1;
use fixture_v1::{cases, plan};

#[path = "fixtures/normalization_v2.rs"]
mod fixture_v2;

#[path = "fixtures/normalization_numeric.rs"]
mod fixture_numeric;

#[path = "fixtures/normalization_base64.rs"]
mod fixture_base64;
#[path = "fixtures/normalization_model.rs"]
mod fixture_model;

/// Every adapter a test of this binary runs.
fn native_crates() -> Vec<NativeCrate> {
    vec![
        base64_adapter(),
        model_adapter(),
        reference_adapter(),
        version_two_adapter(),
        binary64_adapter(),
    ]
}

/// Writes the sources of every crate of [`native_crates`] for `task
/// schema-contract-native-crates` to build. A step of that task, never a test of the suite.
#[test]
#[ignore = "generator: `task schema-contract-native-crates` runs it before the tests"]
fn write_the_sources_of_every_native_crate() {
    native_crate::write_sources(native_crates());
}

fn base64_adapter() -> NativeCrate {
    let (plan, cases) = fixture_base64::fixture();
    adapter("normalization_rust/base64", &plan, &cases)
}

#[test]
fn standalone_rust_target_matches_the_qualified_base64_language() {
    base64_adapter().run(&[]);
}

fn model_adapter() -> NativeCrate {
    adapter(
        "normalization_rust/model",
        &fixture_model::plan(),
        &fixture_model::cases(),
    )
}

#[test]
fn standalone_rust_target_executes_model_owned_wire_records() {
    model_adapter().run(&[]);
}

fn source_digest(source: &str) -> String {
    use std::fmt::Write as _;
    let mut out = String::new();
    for byte in Sha256::digest(source.as_bytes()) {
        write!(out, "{byte:02x}").unwrap();
    }
    out
}

fn reference_adapter() -> NativeCrate {
    let plan = plan();
    // Canonical bytes independently reproduced by the released 0.19.0 reader.
    assert_eq!(
        source_digest(&plan.to_json()),
        "8c99019a9eb7111f0e025772d1c29a82acea2215c1fcaf0b6200f825bcca611b"
    );
    adapter("normalization_rust/reference", &plan, &cases(&plan))
}

#[test]
fn standalone_rust_target_preserves_reference_behavior_and_feature_unification() {
    reference_adapter().run(&[]);
}

fn version_two_adapter() -> NativeCrate {
    let (bundle, recipe) = fixture_v2::fixture();
    let plan = Plan::read(&recipe.to_string(), &[bundle]).unwrap();
    let mut cases = fixture_v2::expected()
        .into_iter()
        .map(|(branch, input, value)| json!({"branch":branch,"input":input,"value":value}))
        .collect::<Vec<_>>();
    let mut input: Value = serde_json::from_str(fixture_v2::INPUT).unwrap();
    input["items"][2]["n"] = json!(i64::MAX);
    cases.push(json!({"branch":"find","input":input.to_string(),"value":3}));
    let errors = plan.run("select", &input).unwrap_err().0;
    assert_eq!(errors[0].rule, "integer_overflow");
    cases.push(json!({"branch":"select","input":input.to_string(),"errors":errors}));
    for input in ["1.0", "1e0", "9223372036854775808"] {
        let errors = plan.run_json("integer", input).unwrap_err().0;
        cases.push(json!({"branch":"integer","input":input,"errors":errors}));
    }
    for mask in 0..8 {
        let mut input: Value = serde_json::from_str(fixture_v2::INPUT).unwrap();
        for (index, item) in input["items"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .enumerate()
        {
            item["enabled"] = json!(mask & (1 << index) != 0);
        }
        for branch in ["find", "select"] {
            let overflow = mask & 2 != 0 && (branch == "select" || mask & 1 == 0);
            if overflow {
                let errors = plan.run(branch, &input).unwrap_err().0;
                assert_eq!(errors[0].rule, "integer_overflow");
                cases.push(json!({"branch":branch,"input":input.to_string(),"errors":errors}));
            } else {
                let selected = [3, 0, 5]
                    .into_iter()
                    .enumerate()
                    .filter_map(|(index, value)| (mask & (1 << index) != 0).then_some(value))
                    .collect::<Vec<_>>();
                let value = if branch == "find" {
                    json!(selected.first().copied().unwrap_or(-1))
                } else {
                    json!(selected)
                };
                assert_eq!(plan.run(branch, &input).unwrap(), value);
                cases.push(json!({"branch":branch,"input":input.to_string(),"value":value}));
            }
        }
    }
    adapter(
        "normalization_rust/version_two",
        &plan,
        &Value::Array(cases),
    )
}

#[test]
fn standalone_rust_target_executes_version_two_ordered_operations() {
    version_two_adapter().run(&[]);
}

fn binary64_adapter() -> NativeCrate {
    let (bundle, recipe) = fixture_numeric::fixture();
    let plan = Plan::read(&recipe.to_string(), &[bundle]).unwrap();
    let mut cases = fixture_numeric::expected()
        .into_iter()
        .map(|(branch, input, value)| json!({"branch":branch,"input":input,"value":value}))
        .collect::<Vec<_>>();
    for (branch, input, rule) in [
        ("truncate", "9223372036854775807", "binary64_range"),
        ("truncate", "9223372036854775808", "binary64_range"),
        ("truncate", "-9223372036854777856", "binary64_range"),
        ("truncate", "1e999", "input_number"),
        ("overflow", "2", "binary64_overflow"),
        ("copy", r#"{"exact":0.10000000000000001}"#, "input_number"),
        ("copy", r#"{"value":1,"value":2}"#, "input_syntax"),
    ] {
        let errors = plan.run_json(branch, input).unwrap_err().0;
        assert_eq!(errors[0].rule, rule);
        cases.push(json!({"branch":branch,"input":input,"errors":errors}));
    }
    let input = json!({"value":9_007_199_254_740_993_i64,"integer":9_007_199_254_740_993_i64});
    cases.push(json!({"branch":"copy","input":input.to_string(),"decoded":true,"value":plan.run("copy",&input).unwrap()}));
    adapter("normalization_rust/binary64", &plan, &Value::Array(cases))
}

#[test]
fn standalone_rust_target_executes_declared_binary64_conversion() {
    binary64_adapter().run(&[]);
}

/// The adapter `plan` generates, its report checked, with `cases` and the behaviour tests under
/// `tests/`.
fn adapter(label: &str, plan: &Plan, cases: &Value) -> NativeCrate {
    let target = plan.rust("normalization_adapter").unwrap();
    assert_eq!(target, plan.rust("normalization_adapter").unwrap());
    let report = serde_json::to_value(&target.report).unwrap();
    let recipe: Value = serde_json::from_str(&plan.to_json()).unwrap();
    assert_eq!(
        report["format"],
        if recipe["format"] == "ess-normalization/3" {
            "ess-normalization-target/2"
        } else {
            "ess-normalization-target/1"
        }
    );
    assert_eq!(report["recipe_digest"], source_digest(&plan.to_json()));
    for (path, digest) in report["files"].as_object().unwrap() {
        assert_eq!(digest, &source_digest(&target.files[path]));
    }
    assert!(!report["files"]
        .as_object()
        .unwrap()
        .contains_key("normalization-report.json"));
    assert_eq!(
        report["files"].as_object().unwrap().len() + 1,
        target.files.len()
    );
    let mut files = target.files;
    files.insert("tests/cases.json".to_owned(), cases.to_string());
    files.insert(
        "tests/behavior.rs".to_owned(),
        include_str!("fixtures/normalization_rust_tests.rs.txt").to_owned(),
    );
    NativeCrate::new(label, files)
}

#[test]
fn rust_target_refuses_invalid_or_reserved_package_names() {
    let plan = plan();
    for package in ["", "Upper", "../escape", "serde", "jsonschema", "name\"\n"] {
        assert_eq!(plan.rust(package).unwrap_err().0[0].rule, "rust_package");
    }
}
