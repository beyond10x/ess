//! Execute the actual standalone Rust adapter against reference results and refusals.

use std::fs;
use std::path::Path;
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

use schema_contract::realize::normalize::Plan;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

#[path = "fixtures/normalization_v1.rs"]
mod fixture_v1;
use fixture_v1::{cases, plan};

#[path = "fixtures/normalization_v2.rs"]
mod fixture_v2;

#[path = "fixtures/normalization_numeric.rs"]
mod fixture_numeric;

fn source_digest(source: &str) -> String {
    use std::fmt::Write as _;
    let mut out = String::new();
    for byte in Sha256::digest(source.as_bytes()) {
        write!(out, "{byte:02x}").unwrap();
    }
    out
}

#[test]
fn standalone_rust_target_preserves_reference_behavior_and_feature_unification() {
    let plan = plan();
    // Canonical bytes independently reproduced by the released 0.19.0 reader.
    assert_eq!(
        source_digest(&plan.to_json()),
        "8c99019a9eb7111f0e025772d1c29a82acea2215c1fcaf0b6200f825bcca611b"
    );
    assert_target(&plan, &cases(&plan));
}

#[test]
fn standalone_rust_target_executes_version_two_ordered_operations() {
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
    assert_target(&plan, &Value::Array(cases));
}

#[test]
fn standalone_rust_target_executes_declared_binary64_conversion() {
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
    assert_target(&plan, &Value::Array(cases));
}

fn assert_target(plan: &Plan, cases: &Value) {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let target = plan.rust("normalization_adapter").unwrap();
    assert_eq!(target, plan.rust("normalization_adapter").unwrap());
    let report = serde_json::to_value(&target.report).unwrap();
    assert_eq!(report["format"], "ess-normalization-target/1");
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
    let root = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!(
        "normalization-rust-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    for (path, contents) in &target.files {
        let path = root.join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, contents).unwrap();
    }
    fs::create_dir_all(root.join("tests")).unwrap();
    fs::write(root.join("tests/cases.json"), cases.to_string()).unwrap();
    fs::write(
        root.join("tests/behavior.rs"),
        include_str!("fixtures/normalization_rust_tests.rs.txt"),
    )
    .unwrap();
    for features in [None, Some("serde_json/arbitrary_precision")] {
        let mut command = Command::new(env!("CARGO"));
        command
            .args(["test", "--offline", "--quiet", "--manifest-path"])
            .arg(root.join("Cargo.toml"))
            .env(
                "CARGO_TARGET_DIR",
                Path::new(env!("CARGO_TARGET_TMPDIR")).join("normalization-rust-target"),
            );
        if let Some(features) = features {
            command.args(["--features", features]);
        }
        let result = command.output().unwrap();
        assert!(
            result.status.success(),
            "{features:?}: {}\n{}",
            String::from_utf8_lossy(&result.stdout),
            String::from_utf8_lossy(&result.stderr)
        );
    }
}

#[test]
fn rust_target_refuses_invalid_or_reserved_package_names() {
    let plan = plan();
    for package in ["", "Upper", "../escape", "serde", "jsonschema", "name\"\n"] {
        assert_eq!(plan.rust(package).unwrap_err().0[0].rule, "rust_package");
    }
}
