//! Accounted Go target generation and an explicit native execution lane.

#[path = "fixtures/normalization_numeric.rs"]
mod fixture_numeric;
#[path = "fixtures/normalization_v1.rs"]
mod fixture_v1;
#[path = "fixtures/normalization_v2.rs"]
mod fixture_v2;

use schema_contract::realize::normalize::Plan;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

fn digest(text: &str) -> String {
    use std::fmt::Write as _;
    let mut out = String::new();
    for byte in Sha256::digest(text) {
        write!(out, "{byte:02x}").unwrap();
    }
    out
}

fn plans() -> Vec<(Plan, Value)> {
    let first = fixture_v1::plan();
    let mut old_cases = fixture_v1::cases(&first).as_array().unwrap().clone();
    for input in [
        r#""\ud800""#,
        r#"{"key":"\ud800"}"#,
        r#"{"\ud800":0}"#,
        r#"{"a":0.10000000000000001,"z":1,"z":2}"#,
    ] {
        let errors = first.run_json("copy", input).unwrap_err().0;
        old_cases.push(json!({"branch":"copy","input":input,"errors":errors}));
    }
    for input in [
        "{}",
        r#"{"seconds":"bad","address":false,"items":[{}, {"key":3,"enabled":null}],"nested":{},"extra":1}"#,
        r#"{"seconds":1,"address":"x","items":[],"nested":{"other":1}}"#,
    ] {
        let errors = first.run_json("primary", input).unwrap_err().0;
        old_cases.push(json!({"branch":"primary","input":input,"errors":errors}));
    }
    let (bundle, recipe) = fixture_v2::fixture();
    let second = Plan::read(&recipe.to_string(), &[bundle]).unwrap();
    let mut ordered = fixture_v2::expected()
        .into_iter()
        .map(|(branch, input, value)| json!({"branch":branch,"input":input,"value":value}))
        .collect::<Vec<_>>();
    let mut selected: Value = serde_json::from_str(fixture_v2::INPUT).unwrap();
    selected["items"][1]["enabled"] = json!(true);
    let input = selected.to_string();
    assert_eq!(second.run_json("find", &input).unwrap(), json!(3));
    ordered.push(json!({"branch":"find","input":input,"value":3}));
    let errors = second.run_json("select", &input).unwrap_err().0;
    ordered.push(json!({"branch":"select","input":input,"errors":errors}));
    let (bundle, recipe) = fixture_numeric::fixture();
    let third = Plan::read(&recipe.to_string(), &[bundle]).unwrap();
    let mut numeric = fixture_numeric::expected()
        .into_iter()
        .map(|(branch, input, value)| json!({"branch":branch,"input":input,"value":value}))
        .collect::<Vec<_>>();
    for (branch, input) in [
        ("truncate", "9223372036854775807"),
        ("truncate", "9223372036854775808"),
        ("truncate", "-9223372036854777856"),
        ("truncate", "1e999"),
        ("overflow", "2"),
        ("copy", r#"{"exact":0.10000000000000001}"#),
        ("copy", r#"{"value":1,"value":2}"#),
    ] {
        let errors = third.run_json(branch, input).unwrap_err().0;
        numeric.push(json!({"branch":branch,"input":input,"errors":errors}));
    }
    vec![
        (first, Value::Array(old_cases)),
        (second, Value::Array(ordered)),
        (third, Value::Array(numeric)),
    ]
}

#[test]
fn go_target_retains_source_identity_and_accounts_for_every_file() {
    for (plan, cases) in plans() {
        assert!(!cases.as_array().unwrap().is_empty());
        let result = plan
            .go(
                "normalization_adapter",
                "example.invalid/normalization-adapter",
            )
            .unwrap();
        assert_eq!(
            result,
            plan.go(
                "normalization_adapter",
                "example.invalid/normalization-adapter"
            )
            .unwrap()
        );
        let report = serde_json::to_value(&result.report).unwrap();
        assert_eq!(report["format"], "ess-normalization-target/1");
        assert_eq!(report["configuration"]["package"], "normalization_adapter");
        assert_eq!(report["recipe_digest"], digest(&plan.to_json()));
        for (path, expected_digest) in report["files"].as_object().unwrap() {
            assert_eq!(expected_digest, &digest(&result.files[path]));
        }
        assert_eq!(
            report["files"].as_object().unwrap().len() + 1,
            result.files.len()
        );
        assert_eq!(result.files["source.recipe.json"], plan.to_json());
        assert!(result.files["go.mod"].contains("jsonschema/v6 v6.0.2"));
        assert!(result.files["go.sum"].contains("nxP4pPoyqOAgX8lYDFCfl3DyKeXErCvSvhcyzwGV9CE="));
    }
}

#[test]
fn go_target_refuses_invalid_library_identity() {
    let plan = fixture_v1::plan();
    for (package, module) in [
        ("main", "example.invalid/test"),
        ("type", "example.invalid/test"),
        ("valid", "../escape"),
        ("quote\"", "example.invalid/test"),
    ] {
        assert_eq!(
            plan.go(package, module).unwrap_err().0[0].rule,
            "go_configuration"
        );
    }
}

#[test]
fn go_target_refuses_referenced_pattern_semantics_before_emitting_files() {
    use schema_contract::bundle::{import, Dialect};
    use schema_contract::realize::normalize::Root;
    let source = json!({"components":{"schemas":{
        "Input":{"type":"object","properties":{"value":{"$ref":"#/components/schemas/Text"}}},
        "Text":{"type":"string","pattern":"^(?=a)a$"},
        "Plain":{"type":"string"}
    }}});
    let roots = ["Input", "Plain"].map(str::to_owned).into_iter().collect();
    let bundle = import(&source.to_string(), &roots, Dialect::Draft202012).unwrap();
    let make = |name| {
        let root = Root::pin(&bundle, name).unwrap();
        Plan::read(&json!({"format":"ess-normalization/1","branches":{"copy":[{
            "input":root,"output":root,"requires":[],"value":{"op":"read","scope":"input","path":[]}
        }]}}).to_string(), std::slice::from_ref(&bundle)).unwrap()
    };
    let plan = make("Input");
    let errors = plan.go("adapter", "example.invalid/adapter").unwrap_err().0;
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].rule, "go_schema_pattern");
    assert!(errors[0].pointer.starts_with("/bundles/"));
    assert!(errors[0].pointer.ends_with("/Text/pattern"));
    assert!(plan.run_json("copy", r#"{"value":"a"}"#).is_ok());
    assert!(plan.run_json("copy", r#"{"value":"b"}"#).is_err());
    // An unselected schema in the same retained bundle cannot widen the refusal.
    assert!(make("Plain")
        .go("adapter", "example.invalid/adapter")
        .is_ok());
}

#[cfg(feature = "go-typecheck")]
#[test]
fn native_go_executes_old_ordered_and_numeric_recipes() {
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::process::Command;

    let compiler = PathBuf::from(
        std::env::var_os("ESS_GO_COMPILER").expect("go-typecheck requires ESS_GO_COMPILER"),
    );
    assert!(compiler.is_absolute() && compiler.is_file());
    let version = Command::new(&compiler)
        .arg("version")
        .env("GOTOOLCHAIN", "local")
        .output()
        .unwrap();
    assert!(version.status.success());
    assert_eq!(
        String::from_utf8_lossy(&version.stdout)
            .split_whitespace()
            .nth(2)
            .and_then(|version| version.split('-').next()),
        Some("go1.26.5")
    );
    for (index, (plan, cases)) in plans().into_iter().enumerate() {
        let generated = plan
            .go(
                "normalization_adapter",
                "example.invalid/normalization-adapter",
            )
            .unwrap();
        let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join(format!("normalization-go-{}-{index}", std::process::id()));
        for (path, source) in generated.files {
            let path = root.join(path);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, source).unwrap();
        }
        fs::write(root.join("cases.json"), cases.to_string()).unwrap();
        fs::write(
            root.join("behavior_test.go"),
            include_str!("fixtures/normalization_go_tests.go.txt"),
        )
        .unwrap();
        let output = Command::new(&compiler)
            .args(["test", "-count=1", "-race", "-mod=readonly", "./..."])
            .current_dir(&root)
            .env("GOPROXY", "off")
            .env("GOSUMDB", "off")
            .env("GOTOOLCHAIN", "local")
            .env("GOFLAGS", "")
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "fixture {index}: {}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
}
