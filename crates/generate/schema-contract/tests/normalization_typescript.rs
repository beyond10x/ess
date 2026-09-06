//! Deterministic executable files and source retention across every recipe family.
#[path = "fixtures/normalization_typescript.rs"]
mod fixture;

use serde_json::{json, Value};
use sha2::{Digest, Sha256};

fn digest(text: &str) -> String {
    use std::fmt::Write as _;
    let mut result = String::new();
    for byte in Sha256::digest(text) {
        write!(result, "{byte:02x}").unwrap();
    }
    result
}

#[test]
fn every_recipe_emits_a_complete_deterministic_accounted_typescript_package() {
    for (name, plan, _) in fixture::corpora() {
        let first = plan.typescript("@sample/normalization-adapter").unwrap();
        assert_eq!(
            first,
            plan.typescript("@sample/normalization-adapter").unwrap(),
            "{name}"
        );
        let report = serde_json::to_value(&first.report).unwrap();
        assert_eq!(
            report["configuration"],
            json!({"language":"typescript","package":"@sample/normalization-adapter"})
        );
        let recipe: Value = serde_json::from_str(&plan.to_json()).unwrap();
        assert_eq!(
            report["format"],
            match recipe["format"].as_str().unwrap() {
                "ess-normalization/1" | "ess-normalization/2" => "ess-normalization-target/1",
                "ess-normalization/3" => "ess-normalization-target/2",
                _ => "ess-normalization-target/3",
            }
        );
        assert_eq!(first.files["source.recipe.json"], plan.to_json());
        assert_eq!(
            first.files.len(),
            report["files"].as_object().unwrap().len() + 1
        );
        for (path, digest) in report["files"].as_object().unwrap() {
            assert_eq!(digest, &self::digest(&first.files[path]), "{name}: {path}");
        }
        assert!(!first.files["package.json"].contains("dependencies"));
        assert!(first.files["tsconfig.json"].contains("ES2022"));
        assert!(first.files["schema-profile.json"].contains("uniqueItems"));
        assert!(!first.files["src/bindings.ts"].contains("JSON.parse"));
        for root in report["roots"].as_array().unwrap() {
            assert!(first
                .files
                .contains_key(root["schema_path"].as_str().unwrap()));
            assert_eq!(
                root["schema_digest"],
                digest(&first.files[root["schema_path"].as_str().unwrap()])
            );
        }
    }
}

#[test]
fn package_identity_is_bounded_and_never_interpreted_as_a_path_or_identifier() {
    let plan = fixture::raw::plan();
    for package in ["adapter", "a.b_c-2", "@scope/name"] {
        assert!(plan.typescript(package).is_ok());
    }
    for package in [
        "",
        ".hidden",
        "../escape",
        "A",
        "@scope",
        "@scope/name/more",
        "a\n",
        "a\"",
        "a b",
        "_a",
        "@/name",
    ] {
        let error = plan.typescript(package).unwrap_err();
        assert_eq!(error.0[0].rule, "typescript_package", "{package:?}");
    }
    assert!(plan.typescript(&"a".repeat(214)).is_ok());
    assert!(plan.typescript(&"a".repeat(215)).is_err());
}
