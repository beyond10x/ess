//! Adversarial composition at finite-number, raw-token and model-authority boundaries.

#[path = "fixtures/binary64_adversary.rs"]
mod fixture;

use schema_contract::realize::normalize::Plan;
use serde_json::json;

#[test]
fn nested_nullable_floats_raw_capture_and_exact_siblings_survive_two_stages() {
    let plan = fixture::plan();
    for case in fixture::cases().as_array().unwrap() {
        fixture::assert_case(
            case,
            plan.run_json("primary", case["input"].as_str().unwrap()),
        );
    }
    eprintln!("8 independently authored nested/raw/exact/default/error-order vectors");
}

#[test]
fn optional_aliases_require_complete_policies_and_exact_model_pins() {
    let (model, recipe) = fixture::fixture();
    for index in 0..2 {
        let mut value = recipe.clone();
        value["binary64_inputs"]["primary"]
            .as_array_mut()
            .unwrap()
            .remove(index);
        let errors = Plan::check_with_models(
            serde_json::from_value(value).unwrap(),
            &[],
            std::slice::from_ref(&model),
        )
        .unwrap_err();
        assert!(
            errors
                .0
                .iter()
                .any(|e| e.rule == "model_binary64_policy"
                    && e.pointer == "/binary64_inputs/primary"),
            "{errors:?}"
        );
    }
    for part in ["source_digest", "contract_digest", "projection_digest"] {
        let mut value = recipe.clone();
        value["branches"]["primary"][0]["input"]["model"][part] = json!("0".repeat(64));
        let errors = Plan::check_with_models(
            serde_json::from_value(value).unwrap(),
            &[],
            std::slice::from_ref(&model),
        )
        .unwrap_err();
        assert!(
            errors
                .0
                .iter()
                .any(|e| e.rule == "unknown_model" && e.pointer == "/branches/primary/0/input"),
            "{errors:?}"
        );
    }
    eprintln!("2 omitted numeric policies and 3 separately corrupted model pins refuse");
}

#[test]
fn lazy_defaults_and_later_stages_cannot_erase_float_identity() {
    let (model, recipe) = fixture::fixture();
    for expression in [
        json!({"op":"integer","value":0}),
        json!({"op":"string","value":"0.0"}),
    ] {
        let mut value = recipe.clone();
        value["branches"]["primary"][0]["value"]["fields"]["ratio"]["fallback"] = expression;
        let errors = Plan::check_with_models(
            serde_json::from_value(value).unwrap(),
            &[],
            std::slice::from_ref(&model),
        )
        .unwrap_err();
        assert!(
            errors.0.iter().any(|e| e.rule == "output_type"),
            "{errors:?}"
        );
    }
    let mut value = recipe;
    value["branches"]["primary"][1]["requires"][0]["right"] =
        json!({"op":"read","scope":"input","path":["count"]});
    let errors =
        Plan::check_with_models(serde_json::from_value(value).unwrap(), &[], &[model]).unwrap_err();
    assert!(
        errors.0.iter().any(|e| e.rule == "equality_type"),
        "{errors:?}"
    );
    eprintln!("2 wrong-kind lazy defaults and later-stage mixed numeric equality refuse");
}

fn write_target(
    name: &str,
    files: std::collections::BTreeMap<String, String>,
) -> std::path::PathBuf {
    let root = std::path::Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("binary64-adversary-{name}-{}", std::process::id()));
    for (path, source) in files {
        let path = root.join(path);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, source).unwrap();
    }
    root
}

#[test]
fn native_rust_composition_preserves_identity_in_both_number_feature_modes() {
    use std::{fs, path::Path, process::Command};
    let root = write_target(
        "rust",
        fixture::plan().rust("normalization_adapter").unwrap().files,
    );
    fs::create_dir_all(root.join("tests")).unwrap();
    fs::write(root.join("tests/cases.json"), fixture::cases().to_string()).unwrap();
    fs::write(
        root.join("tests/adversary.rs"),
        include_str!("fixtures/binary64_adversary_rust.rs.txt"),
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
        command.args(["--", "--nocapture"]);
        let output = command.output().unwrap();
        let text = format!(
            "{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        fs::write(
            root.join(if features.is_some() {
                "arbitrary.log"
            } else {
                "default.log"
            }),
            &text,
        )
        .unwrap();
        eprintln!(
            "native Rust {features:?}, actual exit {}:\n{text}",
            output.status
        );
        assert!(output.status.success());
    }
}

#[cfg(feature = "go-typecheck")]
#[test]
fn native_go_composition_preserves_identity_and_atomic_refusals() {
    use std::{fs, path::Path, process::Command};
    let compiler = std::path::PathBuf::from(std::env::var_os("ESS_GO_COMPILER").unwrap());
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
            .and_then(|text| text.split('-').next()),
        Some("go1.26.5")
    );
    let root = write_target(
        "go",
        fixture::plan()
            .go(
                "normalization_adapter",
                "example.invalid/binary64-adversary",
            )
            .unwrap()
            .files,
    );
    fs::write(root.join("cases.json"), fixture::cases().to_string()).unwrap();
    fs::write(
        root.join("adversary_test.go"),
        include_str!("fixtures/binary64_adversary_go.go.txt"),
    )
    .unwrap();
    let output = Command::new(compiler)
        .args(["test", "-v", "-count=1", "-race", "-mod=readonly", "./..."])
        .current_dir(&root)
        .env("GOPROXY", "off")
        .env("GOSUMDB", "off")
        .env("GOTOOLCHAIN", "local")
        .env("GOFLAGS", "")
        .env(
            "GOCACHE",
            Path::new(env!("CARGO_TARGET_TMPDIR")).join("binary64-go-cache"),
        )
        .env("GOMAXPROCS", "4")
        .output()
        .unwrap();
    let text = format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    fs::write(root.join("native-go.log"), &text).unwrap();
    eprintln!("native Go actual exit {}:\n{text}", output.status);
    assert!(output.status.success());
}
