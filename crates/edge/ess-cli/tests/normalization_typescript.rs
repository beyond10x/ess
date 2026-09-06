//! Executable TypeScript publication uses the existing atomic CLI boundary.

use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

use schema_contract::bundle::{import, Dialect};
use schema_contract::realize::normalize::Root;
use serde_json::{json, Value};

#[allow(dead_code)]
#[path = "../../../generate/schema-contract/tests/fixtures/normalization_model.rs"]
mod model;

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!(
            "normalization-typescript-cli-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&path).unwrap();
        let bundle = import(
            &json!({"components":{"schemas":{"Text":{"type":"string"}}}}).to_string(),
            &["Text".to_owned()].into_iter().collect(),
            Dialect::Draft202012,
        )
        .unwrap();
        let root = Root::pin(&bundle, "Text").unwrap();
        let recipe = json!({"format":"ess-normalization/1","branches":{"copy":[{
            "input":root,"output":root,"requires":[],"value":{"op":"read","scope":"input","path":[]}
        }]}});
        fs::write(path.join("source.bundle.json"), bundle.to_json().unwrap()).unwrap();
        fs::write(path.join("recipe.json"), recipe.to_string()).unwrap();
        Self(path)
    }

    fn generate(&self, extra: &[&str]) -> Output {
        self.generate_from("recipe.json", "source.bundle.json", extra)
    }

    fn generate_from(&self, recipe: &str, bundle: &str, extra: &[&str]) -> Output {
        Command::new(env!("CARGO_BIN_EXE_ess"))
            .current_dir(&self.0)
            .args([
                "generate",
                "schema",
                "normalize-generate",
                "--recipe",
                recipe,
                "--bundle",
                bundle,
                "--target",
                "typescript",
                "--package",
                "normalization-adapter",
                "--out",
                "generated",
            ])
            .args(extra)
            .output()
            .unwrap()
    }
}

#[test]
fn typescript_cli_emits_an_accounted_executable_package() {
    let fixture = Fixture::new();
    let result = fixture.generate(&[]);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let report: Value = serde_json::from_slice(
        &fs::read(fixture.0.join("generated/normalization-report.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(
        report["configuration"],
        json!({"language":"typescript","package":"normalization-adapter"})
    );
    for name in [
        "src/index.ts",
        "src/input.ts",
        "src/schemas.ts",
        "schema-profile.json",
        "package.json",
    ] {
        assert!(report["files"].get(name).is_some(), "{name}");
    }
    assert!(fixture.generate(&["--check"]).status.success());
}

#[test]
fn every_planned_file_participates_in_read_only_drift_checks() {
    let fixture = Fixture::new();
    assert!(fixture.generate(&[]).status.success());
    let root = fixture.0.join("generated");
    let report: Value =
        serde_json::from_slice(&fs::read(root.join("normalization-report.json")).unwrap()).unwrap();
    let paths = report["files"]
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .chain(std::iter::once("normalization-report.json"))
        .collect::<Vec<_>>();
    for path in &paths {
        let path = root.join(path);
        let original = fs::read(&path).unwrap();
        fs::write(&path, "independent drift").unwrap();
        assert!(!fixture.generate(&["--check"]).status.success());
        assert_eq!(fs::read_to_string(&path).unwrap(), "independent drift");
        fs::write(path, original).unwrap();
    }
    fs::remove_file(root.join("src/value.ts")).unwrap();
    assert!(!fixture.generate(&["--check"]).status.success());
    assert!(!root.join("src/value.ts").exists());
    fs::write(root.join("unrelated.txt"), "retained").unwrap();
    assert!(fixture.generate(&[]).status.success());
    assert_eq!(
        fs::read_to_string(root.join("unrelated.txt")).unwrap(),
        "retained"
    );
    assert!(fixture.generate(&["--check"]).status.success());
    eprintln!(
        "TypeScript CLI drift: {} individual planned-file mutations and a missing late file",
        paths.len()
    );
}

#[test]
fn module_and_late_path_conflicts_refuse_before_any_publication() {
    let fixture = Fixture::new();
    let module = fixture.generate(&["--module", "example.invalid/adapter"]);
    assert!(!module.status.success());
    assert!(String::from_utf8_lossy(&module.stderr).contains("only supported for the Go target"));
    assert!(!fixture.0.join("generated").exists());
    assert!(!fixture.generate(&["--check"]).status.success());
    assert!(!fixture.0.join("generated").exists());
    let root = fixture.0.join("generated");
    fs::create_dir_all(root.join("src/value.ts")).unwrap();
    fs::write(root.join("package.json"), "untouched").unwrap();
    assert!(!fixture.generate(&[]).status.success());
    assert_eq!(
        fs::read_to_string(root.join("package.json")).unwrap(),
        "untouched"
    );
    assert!(!root.join("src/index.ts").exists());
}

#[test]
fn retained_recipe_and_bundle_inputs_cannot_be_overwritten() {
    let fixture = Fixture::new();
    let root = fixture.0.join("generated");
    fs::create_dir_all(&root).unwrap();
    let original_recipe = fs::read(fixture.0.join("recipe.json")).unwrap();
    fs::write(root.join("source.recipe.json"), &original_recipe).unwrap();
    let result = fixture.generate_from("generated/source.recipe.json", "source.bundle.json", &[]);
    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("must not replace"));
    assert_eq!(
        fs::read(root.join("source.recipe.json")).unwrap(),
        original_recipe
    );
    assert!(!root.join("package.json").exists());
    let bundle = fs::read(fixture.0.join("source.bundle.json")).unwrap();
    fs::write(root.join("package.json"), &bundle).unwrap();
    assert!(!fixture
        .generate_from("recipe.json", "generated/package.json", &[])
        .status
        .success());
    assert_eq!(fs::read(root.join("package.json")).unwrap(), bundle);
}

#[cfg(unix)]
#[test]
fn generated_file_and_parent_links_refuse_without_touching_their_destinations() {
    use std::os::unix::fs::symlink;
    for parent_link in [false, true] {
        let fixture = Fixture::new();
        let root = fixture.0.join("generated");
        fs::create_dir_all(&root).unwrap();
        let destination = fixture.0.join("outside");
        if parent_link {
            fs::create_dir_all(&destination).unwrap();
            fs::write(destination.join("value.ts"), "untouched").unwrap();
            symlink(&destination, root.join("src")).unwrap();
        } else {
            fs::write(&destination, "untouched").unwrap();
            symlink(&destination, root.join("schema-profile.json")).unwrap();
        }
        assert!(!fixture.generate(&[]).status.success());
        assert!(!fixture.generate(&["--check"]).status.success());
        assert!(!root.join("package.json").exists());
        assert_eq!(
            fs::read_to_string(if parent_link {
                destination.join("value.ts")
            } else {
                destination
            })
            .unwrap(),
            "untouched"
        );
    }
}

#[test]
fn model_inputs_are_recompiled_pinned_and_protected_before_typescript_generation() {
    let fixture = Fixture::new();
    fs::create_dir(fixture.0.join("model")).unwrap();
    fs::write(fixture.0.join("model/system.yaml"), model::SOURCE).unwrap();
    let (_, recipe) = model::fixture();
    fs::write(fixture.0.join("recipe.json"), recipe.to_string()).unwrap();
    let generate = |out: &str| {
        Command::new(env!("CARGO_BIN_EXE_ess"))
            .current_dir(&fixture.0)
            .args([
                "generate",
                "schema",
                "normalize-generate",
                "--recipe",
                "recipe.json",
                "--model",
                "model",
                "--target",
                "typescript",
                "--package",
                "adapter",
                "--out",
                out,
            ])
            .output()
            .unwrap()
    };
    assert!(generate("generated").status.success());
    let before = fs::read(fixture.0.join("generated/normalization-report.json")).unwrap();
    let report: Value = serde_json::from_slice(&before).unwrap();
    assert_eq!(report["format"], "ess-normalization-target/2");
    assert_eq!(report["roots"].as_array().unwrap().len(), 2);
    assert!(!generate("model/generated").status.success());
    assert!(!fixture.0.join("model/generated").exists());
    assert_eq!(
        fs::read_to_string(fixture.0.join("model/system.yaml")).unwrap(),
        model::SOURCE
    );
    fs::write(
        fixture.0.join("model/system.yaml"),
        model::SOURCE.replace("version: v1", "version: v2"),
    )
    .unwrap();
    assert!(!generate("generated").status.success());
    assert_eq!(
        fs::read(fixture.0.join("generated/normalization-report.json")).unwrap(),
        before
    );
}

#[test]
fn unqualified_profile_refuses_with_source_pointer_and_no_partial_files() {
    let fixture = Fixture::new();
    let bundle = import(
        &json!({"components":{"schemas":{"Input":{"type":"array","uniqueItems":true}}}})
            .to_string(),
        &["Input".to_owned()].into_iter().collect(),
        Dialect::Draft202012,
    )
    .unwrap();
    let root = Root::pin(&bundle, "Input").unwrap();
    fs::write(
        fixture.0.join("source.bundle.json"),
        bundle.to_json().unwrap(),
    )
    .unwrap();
    fs::write(fixture.0.join("recipe.json"), json!({"format":"ess-normalization/6","branches":{"copy":[{"input":root,"output":root,"requires":[],"value":{"op":"read","scope":"input","path":[]}}]}}).to_string()).unwrap();
    let result = fixture.generate(&[]);
    assert!(!result.status.success());
    let error = String::from_utf8_lossy(&result.stderr);
    assert!(
        error.contains("typescript_schema_profile") && error.contains("/Input/uniqueItems"),
        "{error}"
    );
    assert!(!fixture.0.join("generated").exists());
}
