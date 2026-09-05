//! Command-boundary checks protect authored inputs and never publish partial results.

use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

use schema_contract::bundle::{import, Bundle, Dialect};
use schema_contract::realize::normalize::Root;
use serde_json::{json, Value};

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let root = std::env::temp_dir().join(format!(
            "ess-normalization-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&root).unwrap();
        let input = Self::bundle(
            "Input",
            &json!({"type":"object", "additionalProperties":false,
            "properties":{"seconds":{"type":"integer"}}}),
        );
        let output = Self::bundle("Output", &json!({"type":"integer", "minimum":1}));
        let recipe = json!({"format":"ess-normalization/1", "branches":{"primary":[{
            "input":Root::pin(&input, "Input").unwrap(), "output":Root::pin(&output, "Output").unwrap(),
            "requires":[], "value":{"op":"arithmetic", "operation":"multiply", "overflow":"reject",
            "left":{"op":"fallback", "value":{"op":"read", "scope":"input", "path":["seconds"]},
                "fallback":{"op":"integer", "value":1}, "on_null":false},
            "right":{"op":"integer", "value":1000}}}]}});
        fs::write(root.join("input.bundle.json"), input.to_json().unwrap()).unwrap();
        fs::write(root.join("output.bundle.json"), output.to_json().unwrap()).unwrap();
        fs::write(root.join("recipe.json"), recipe.to_string()).unwrap();
        fs::write(root.join("instance.json"), r#"{"seconds":3}"#).unwrap();
        fs::write(root.join("result.json"), "untouched").unwrap();
        Self(root)
    }

    fn bundle(name: &str, schema: &Value) -> Bundle {
        import(
            &json!({"components":{"schemas":{name:schema}}}).to_string(),
            &[name.to_owned()].into_iter().collect(),
            Dialect::Draft202012,
        )
        .unwrap()
    }

    fn run(&self, operation: &str, extra: &[&str]) -> Output {
        let mut command = Command::new(env!("CARGO_BIN_EXE_ess"));
        command.current_dir(&self.0).args([
            "generate",
            "schema",
            operation,
            "--recipe",
            "recipe.json",
            "--bundle",
            "input.bundle.json",
            "--bundle",
            "output.bundle.json",
        ]);
        if operation == "normalize-run" {
            command.args(["--branch", "primary", "--input", "instance.json"]);
        }
        command.args(extra).output().unwrap()
    }

    fn recipe(&self) -> Value {
        serde_json::from_slice(&fs::read(self.0.join("recipe.json")).unwrap()).unwrap()
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

#[test]
fn checked_recipe_and_run_are_deterministic_with_json_only_stdout() {
    let fixture = Fixture::new();
    let first = fixture.run("normalize-check", &[]);
    assert!(first.status.success(), "{first:?}");
    assert_eq!(first.stdout, fixture.run("normalize-check", &[]).stdout);
    assert_eq!(
        serde_json::from_slice::<Value>(&first.stdout).unwrap(),
        fixture.recipe()
    );
    assert!(first.stderr.is_empty());
    let run = fixture.run("normalize-run", &[]);
    assert!(run.status.success(), "{run:?}");
    assert_eq!(run.stdout, b"3000\n");
    let written = fixture.run("normalize-run", &["--out", "result.json"]);
    assert!(written.status.success(), "{written:?}");
    assert_eq!(fs::read(fixture.0.join("result.json")).unwrap(), run.stdout);
}

#[test]
fn failed_check_or_execution_preserves_output_and_does_not_emit_a_result() {
    let fixture = Fixture::new();
    for input in [
        r#"{"seconds":0}"#,
        r#"{"seconds":9223372036854775807}"#,
        r#"{"seconds":1.0}"#,
        r#"{"seconds":18446744073709551617}"#,
        r#"{"seconds":1,"seconds":2}"#,
    ] {
        fs::write(fixture.0.join("instance.json"), input).unwrap();
        let result = fixture.run("normalize-run", &["--out", "result.json"]);
        assert!(!result.status.success());
        assert!(result.stdout.is_empty());
        assert_eq!(
            fs::read(fixture.0.join("result.json")).unwrap(),
            b"untouched"
        );
    }
    let mut recipe = fixture.recipe();
    recipe["branches"]["never_used"] = json!([]);
    fs::write(fixture.0.join("recipe.json"), recipe.to_string()).unwrap();
    for operation in ["normalize-check", "normalize-run"] {
        let result = fixture.run(operation, &["--out", "result.json"]);
        assert!(!result.status.success());
        assert!(String::from_utf8_lossy(&result.stderr).contains("empty_pipeline"));
        assert_eq!(
            fs::read(fixture.0.join("result.json")).unwrap(),
            b"untouched"
        );
    }
}

#[test]
fn output_cannot_replace_any_declared_input() {
    let fixture = Fixture::new();
    for file in [
        "recipe.json",
        "input.bundle.json",
        "output.bundle.json",
        "instance.json",
    ] {
        let before = fs::read(fixture.0.join(file)).unwrap();
        let result = fixture.run("normalize-run", &["--out", file]);
        assert!(!result.status.success(), "{file}: {result:?}");
        assert!(String::from_utf8_lossy(&result.stderr).contains("must not replace"));
        assert_eq!(fs::read(fixture.0.join(file)).unwrap(), before);
    }
}

#[cfg(unix)]
#[test]
fn output_symlinks_and_hardlinks_are_refused_without_mutation() {
    let fixture = Fixture::new();
    std::os::unix::fs::symlink(
        fixture.0.join("result.json"),
        fixture.0.join("symlink.json"),
    )
    .unwrap();
    fs::hard_link(
        fixture.0.join("result.json"),
        fixture.0.join("hardlink.json"),
    )
    .unwrap();
    for name in ["symlink.json", "hardlink.json"] {
        assert!(!fixture
            .run("normalize-run", &["--out", name])
            .status
            .success());
        assert_eq!(
            fs::read(fixture.0.join("result.json")).unwrap(),
            b"untouched"
        );
    }
}

#[test]
fn stale_bundle_missing_source_and_unknown_dispatch_refuse() {
    let fixture = Fixture::new();
    let mut recipe = fixture.recipe();
    recipe["branches"]["primary"][0]["input"]["bundle_digest"] = json!("wrong");
    fs::write(fixture.0.join("recipe.json"), recipe.to_string()).unwrap();
    let result = fixture.run("normalize-check", &[]);
    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("unknown_bundle"));
    let fixture = Fixture::new();
    let mut recipe = fixture.recipe();
    recipe["branches"]["other"] = recipe["branches"]["primary"].take();
    recipe["branches"]
        .as_object_mut()
        .unwrap()
        .remove("primary");
    fs::write(fixture.0.join("recipe.json"), recipe.to_string()).unwrap();
    let result = fixture.run("normalize-run", &[]);
    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("unknown_dispatch"));
    fs::remove_file(fixture.0.join("output.bundle.json")).unwrap();
    assert!(!fixture.run("normalize-check", &[]).status.success());
}
