//! Command-boundary checks protect authored inputs and never publish partial results.

use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

use schema_contract::bundle::{import, Bundle, Dialect};
use schema_contract::realize::normalize::Root;
use serde_json::{json, Value};

#[path = "../../../generate/schema-contract/tests/fixtures/normalization_binary64.rs"]
mod fixture_binary64;
use fixture_binary64::model as fixture_model;

fn positional_cli_fixture() -> (Fixture, Value) {
    let fixture = Fixture::new();
    let input = Fixture::bundle(
        "Input",
        &json!({"type":"array","prefixItems":[{"type":"string"},{"type":"string"}],"items":false,"minItems":2,"maxItems":2}),
    );
    let output = Fixture::bundle(
        "Output",
        &json!({"type":"object","additionalProperties":false,"required":["left","right"],"properties":{"left":{"type":"string"},"right":{"type":"string"}}}),
    );
    fs::write(
        fixture.0.join("input.bundle.json"),
        input.to_json().unwrap(),
    )
    .unwrap();
    fs::write(
        fixture.0.join("output.bundle.json"),
        output.to_json().unwrap(),
    )
    .unwrap();
    let position = |index| json!({"op":"position","value":{"op":"read","scope":"input","path":[]},"index":index});
    let recipe = json!({"format":"ess-normalization/6","positional_inputs":{"primary":[{"path":[],"kind":"fixed_string_array","length":2,"missing":"preserve","null":"zero","short":"zero_pad","extra":"discard","null_element":"zero"}]},"branches":{"primary":[{"input":Root::pin(&input,"Input").unwrap(),"output":Root::pin(&output,"Output").unwrap(),"requires":[],"value":{"op":"record","fields":{"left":position(0),"right":position(1)}}}]}});
    fs::write(fixture.0.join("recipe.json"), recipe.to_string()).unwrap();
    (fixture, recipe)
}

#[test]
fn positional_cli_prepares_text_and_refuses_before_publication() {
    let (fixture, recipe) = positional_cli_fixture();
    let checked = fixture.run("normalize-check", &[]);
    assert!(checked.status.success(), "{checked:?}");
    assert_eq!(
        serde_json::from_slice::<Value>(&checked.stdout).unwrap(),
        recipe
    );
    for (input, expected) in [
        ("null", json!({"left":"","right":""})),
        ("[]", json!({"left":"","right":""})),
        (r#"["a"]"#, json!({"left":"a","right":""})),
        (
            r#"[null,"b",1e999,{"x":0,"x":1}]"#,
            json!({"left":"","right":"b"}),
        ),
    ] {
        fs::write(fixture.0.join("instance.json"), input).unwrap();
        let result = fixture.run("normalize-run", &[]);
        assert!(result.status.success(), "{result:?}");
        assert_eq!(
            serde_json::from_slice::<Value>(&result.stdout).unwrap(),
            expected
        );
        assert_eq!(
            fs::read_to_string(fixture.0.join("instance.json")).unwrap(),
            input
        );
    }
    for (input, rule) in [
        (r#"[1e999,"b"]"#, "positional_element_type"),
        (r#"[true,"b",]"#, "input_syntax"),
        (r#"["a","b","\ud800"]"#, "input_syntax"),
        ("{}", "positional_input_type"),
    ] {
        fs::write(fixture.0.join("instance.json"), input).unwrap();
        let result = fixture.run("normalize-run", &["--out", "result.json"]);
        assert!(!result.status.success());
        assert!(
            String::from_utf8_lossy(&result.stderr).contains(rule),
            "{result:?}"
        );
        assert_eq!(
            fs::read_to_string(fixture.0.join("result.json")).unwrap(),
            "untouched"
        );
    }
    for target in ["rust", "go"] {
        let mut args = vec!["--target", target, "--package", "adapter", "--out", target];
        if target == "go" {
            args.extend(["--module", "example.invalid/adapter"]);
        }
        let result = fixture.run("normalize-generate", &args);
        assert!(result.status.success(), "{result:?}");
        let report: Value = serde_json::from_slice(
            &fs::read(fixture.0.join(target).join("normalization-report.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(report["format"], "ess-normalization-target/3");
        args.push("--check");
        assert!(fixture.run("normalize-generate", &args).status.success());
        let recipe_path = fixture.0.join(target).join("source.recipe.json");
        fs::write(&recipe_path, "drift").unwrap();
        assert!(!fixture.run("normalize-generate", &args).status.success());
        assert_eq!(fs::read_to_string(recipe_path).unwrap(), "drift");
    }
    eprintln!("CLI executed 8 positional text vectors and both publication/drift targets");
}

#[test]
fn positional_cli_refuses_unused_branches_before_publication() {
    let (fixture, mut recipe) = positional_cli_fixture();
    recipe["branches"]["unused"] = recipe["branches"]["primary"].clone();
    recipe["branches"]["unused"][0]["value"]["fields"]["left"]["index"] = json!(2);
    fs::write(fixture.0.join("recipe.json"), recipe.to_string()).unwrap();
    fs::write(fixture.0.join("instance.json"), "[]").unwrap();
    for (operation, args) in [
        ("normalize-check", vec!["--out", "result.json"]),
        ("normalize-run", vec!["--out", "result.json"]),
        (
            "normalize-generate",
            vec![
                "--target",
                "rust",
                "--package",
                "adapter",
                "--out",
                "refused-rust",
            ],
        ),
        (
            "normalize-generate",
            vec![
                "--target",
                "go",
                "--package",
                "adapter",
                "--module",
                "example.invalid/adapter",
                "--out",
                "refused-go",
            ],
        ),
    ] {
        let result = fixture.run(operation, &args);
        assert!(!result.status.success());
        assert!(
            String::from_utf8_lossy(&result.stderr).contains("position_index"),
            "{result:?}"
        );
        assert_eq!(
            fs::read_to_string(fixture.0.join("result.json")).unwrap(),
            "untouched"
        );
    }
    assert!(!fixture.0.join("refused-rust").exists());
    assert!(!fixture.0.join("refused-go").exists());
    eprintln!("CLI executed 4 unused-branch pre-publication refusals");
}

#[test]
fn binary64_cli_keeps_numeric_identity_and_emits_checked_format_five() {
    let fixture = Fixture::new();
    fs::create_dir(fixture.0.join("model")).unwrap();
    fs::write(
        fixture.0.join("model/system.yaml"),
        fixture_binary64::SOURCE,
    )
    .unwrap();
    let (_, recipe) = fixture_binary64::fixture();
    fs::write(fixture.0.join("recipe.json"), recipe.to_string()).unwrap();
    let invoke = |operation: &str, args: &[&str]| {
        Command::new(env!("CARGO_BIN_EXE_ess"))
            .current_dir(&fixture.0)
            .args([
                "generate",
                "schema",
                operation,
                "--recipe",
                "recipe.json",
                "--model",
                "model",
            ])
            .args(args)
            .output()
            .unwrap()
    };
    let checked = invoke("normalize-check", &[]);
    assert!(checked.status.success(), "{checked:?}");
    assert_eq!(
        String::from_utf8(checked.stdout).unwrap(),
        fixture_binary64::plan().to_json()
    );
    let cases = fixture_binary64::cases();
    for case in &cases {
        fs::write(
            fixture.0.join("instance.json"),
            case["input"].as_str().unwrap(),
        )
        .unwrap();
        let result = invoke(
            "normalize-run",
            &[
                "--branch",
                case["branch"].as_str().unwrap(),
                "--input",
                "instance.json",
            ],
        );
        if let Some(error) = case["error"].as_str() {
            assert!(!result.status.success());
            let details = format!(
                "{}{}",
                String::from_utf8_lossy(&result.stdout),
                String::from_utf8_lossy(&result.stderr)
            );
            assert!(details.contains(error), "{case}: {details}");
        } else {
            assert!(result.status.success(), "{case}: {result:?}");
            let value: Value = serde_json::from_slice(&result.stdout).unwrap();
            if let Some(bits) = case["bits"].as_str() {
                assert_eq!(
                    value.as_f64().unwrap().to_bits(),
                    u64::from_str_radix(bits, 16).unwrap()
                );
                assert!(value.as_i64().is_none());
            } else if let Some(bits) = case["array_bits"].as_array() {
                assert_eq!(value.as_array().unwrap().len(), bits.len());
                for (item, bits) in value.as_array().unwrap().iter().zip(bits) {
                    if bits.is_null() {
                        assert!(item.is_null());
                    } else {
                        assert_eq!(
                            item.as_f64().unwrap().to_bits(),
                            u64::from_str_radix(bits.as_str().unwrap(), 16).unwrap()
                        );
                    }
                }
            } else {
                assert_eq!(value, case["value"]);
            }
        }
    }
    for target in ["rust", "go"] {
        let mut args = vec!["--target", target, "--package", "adapter", "--out", target];
        if target == "go" {
            args.extend(["--module", "example.invalid/adapter"]);
        }
        let generated = invoke("normalize-generate", &args);
        assert!(generated.status.success(), "{generated:?}");
        args.push("--check");
        assert!(invoke("normalize-generate", &args).status.success());
    }
    eprintln!(
        "CLI executed {} independent Binary64 vectors and both target publication/drift checks",
        cases.len()
    );
}

#[test]
fn model_sources_are_compiled_pinned_and_protected_by_the_cli() {
    let fixture = Fixture::new();
    let model_path = fixture.0.join("model");
    fs::create_dir(&model_path).unwrap();
    fs::write(model_path.join("system.yaml"), fixture_model::SOURCE).unwrap();
    let (_, recipe) = fixture_model::fixture();
    fs::write(fixture.0.join("recipe.json"), recipe.to_string()).unwrap();
    fs::write(
        fixture.0.join("instance.json"),
        r#"{"identifier":"x","seconds":3}"#,
    )
    .unwrap();
    let invoke = |operation: &str, extra: &[&str]| {
        Command::new(env!("CARGO_BIN_EXE_ess"))
            .current_dir(&fixture.0)
            .args([
                "generate",
                "schema",
                operation,
                "--recipe",
                "recipe.json",
                "--model",
                "model",
            ])
            .args(extra)
            .output()
            .unwrap()
    };
    let checked = invoke("normalize-check", &[]);
    assert!(checked.status.success(), "{checked:?}");
    assert_eq!(
        String::from_utf8(checked.stdout).unwrap(),
        fixture_model::plan().to_json()
    );
    for case in fixture_model::cases().as_array().unwrap() {
        fs::write(
            fixture.0.join("instance.json"),
            case["input"].as_str().unwrap(),
        )
        .unwrap();
        let run = invoke(
            "normalize-run",
            &["--branch", "primary", "--input", "instance.json"],
        );
        if let Some(value) = case.get("value") {
            assert!(run.status.success(), "{run:?}");
            assert_eq!(
                serde_json::from_slice::<Value>(&run.stdout).unwrap(),
                *value
            );
        } else {
            assert!(!run.status.success());
        }
    }
    for target in ["rust", "go"] {
        let mut args = vec!["--target", target, "--package", "adapter", "--out", target];
        if target == "go" {
            args.extend(["--module", "example.invalid/adapter"]);
        }
        let result = invoke("normalize-generate", &args);
        assert!(result.status.success(), "{result:?}");
        args.push("--check");
        assert!(invoke("normalize-generate", &args).status.success());
    }
    for (operation, args) in [
        ("normalize-check", vec!["--out", "model/recipe.json"]),
        (
            "normalize-generate",
            vec![
                "--target",
                "rust",
                "--package",
                "adapter",
                "--out",
                "model/output",
            ],
        ),
    ] {
        assert!(!invoke(operation, &args).status.success());
    }
    assert!(!model_path.join("recipe.json").exists());
    assert!(!model_path.join("output").exists());
    fs::write(
        model_path.join("system.yaml"),
        fixture_model::SOURCE.replace("version: v1", "version: v2"),
    )
    .unwrap();
    let stale = invoke("normalize-check", &[]);
    assert!(!stale.status.success());
    assert!(String::from_utf8_lossy(&stale.stderr).contains("unknown_model"));
}

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
fn raw_json_capture_runs_before_schema_and_keeps_failure_output_untouched() {
    let fixture = Fixture::new();
    let bundle = Fixture::bundle("Text", &json!({"type":"string"}));
    fs::write(
        fixture.0.join("input.bundle.json"),
        bundle.to_json().unwrap(),
    )
    .unwrap();
    let root = Root::pin(&bundle, "Text").unwrap();
    let recipe = json!({"format":"ess-normalization/4","raw_json_inputs":{"primary":[[]]},"branches":{"primary":[{"input":root,"output":root,"requires":[],"value":{"op":"read","scope":"input","path":[]}}]}});
    fs::write(fixture.0.join("recipe.json"), recipe.to_string()).unwrap();
    let input = " \n{\"n\":1e999,\"n\":2} \t";
    fs::write(fixture.0.join("instance.json"), input).unwrap();
    let run = fixture.run("normalize-run", &[]);
    assert!(run.status.success(), "{run:?}");
    assert_eq!(run.stdout, b"\"eyJuIjoxZTk5OSwibiI6Mn0=\"\n");
    assert_eq!(
        fs::read_to_string(fixture.0.join("instance.json")).unwrap(),
        input
    );
    let checked = fixture.run("normalize-check", &[]);
    assert!(checked.status.success(), "{checked:?}");
    let plan =
        schema_contract::realize::normalize::Plan::read(&recipe.to_string(), &[bundle]).unwrap();
    assert_eq!(checked.stdout, plan.to_json().as_bytes());
    for target in ["rust", "go"] {
        let mut args = vec!["--target", target, "--package", "adapter", "--out", target];
        if target == "go" {
            args.extend(["--module", "example.invalid/adapter"]);
        }
        let result = fixture.run("normalize-generate", &args);
        assert!(result.status.success(), "{result:?}");
        let report: Value = serde_json::from_slice(
            &fs::read(fixture.0.join(target).join("normalization-report.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(report["format"], "ess-normalization-target/3");
        args.push("--check");
        assert!(fixture.run("normalize-generate", &args).status.success());
    }
    for input in [r#""\ud800""#, "{\"n\":1,}"] {
        fs::write(fixture.0.join("instance.json"), input).unwrap();
        let result = fixture.run("normalize-run", &["--out", "result.json"]);
        assert!(!result.status.success());
        assert!(String::from_utf8_lossy(&result.stderr).contains("input_syntax"));
        assert_eq!(
            fs::read_to_string(fixture.0.join("result.json")).unwrap(),
            "untouched"
        );
    }
}

#[test]
fn generated_libraries_match_the_api_and_drift_check_never_repairs_files() {
    let fixture = Fixture::new();
    let bundles = ["input.bundle.json", "output.bundle.json"]
        .map(|path| Bundle::read(&fs::read_to_string(fixture.0.join(path)).unwrap()).unwrap());
    let plan =
        schema_contract::realize::normalize::Plan::read(&fixture.recipe().to_string(), &bundles)
            .unwrap();
    for target in ["rust", "go"] {
        let mut args = vec!["--target", target, "--package", "adapter", "--out", target];
        let generated = if target == "rust" {
            plan.rust("adapter").unwrap()
        } else {
            args.extend(["--module", "example.invalid/adapter"]);
            plan.go("adapter", "example.invalid/adapter").unwrap()
        };
        let result = fixture.run("normalize-generate", &args);
        assert!(result.status.success(), "{result:?}");
        let root = fixture.0.join(target);
        for (path, contents) in &generated.files {
            assert_eq!(fs::read(root.join(path)).unwrap(), contents.as_bytes());
        }
        args.push("--check");
        let result = fixture.run("normalize-generate", &args);
        assert!(result.status.success(), "{result:?}");
        assert!(String::from_utf8_lossy(&result.stdout).contains("file(s): current"));
        fs::write(root.join("source.recipe.json"), "stale").unwrap();
        fs::remove_file(root.join("normalization-report.json")).unwrap();
        fs::write(root.join("consumer.txt"), "unowned").unwrap();
        let result = fixture.run("normalize-generate", &args);
        assert!(!result.status.success());
        assert_eq!(
            result.stdout,
            b"normalization-report.json: missing\nsource.recipe.json: stale\n"
        );
        assert_eq!(fs::read(root.join("source.recipe.json")).unwrap(), b"stale");
        assert!(!root.join("normalization-report.json").exists());
        args.pop();
        assert!(fixture.run("normalize-generate", &args).status.success());
        args.push("--check");
        assert!(fixture.run("normalize-generate", &args).status.success());
        assert_eq!(fs::read(root.join("consumer.txt")).unwrap(), b"unowned");
    }
}

#[test]
fn generation_checks_refuse_before_creating_or_changing_destinations() {
    let fixture = Fixture::new();
    let base = [
        "--target",
        "rust",
        "--package",
        "adapter",
        "--out",
        "missing",
    ];
    let mut check = base.to_vec();
    check.push("--check");
    let result = fixture.run("normalize-generate", &check);
    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stdout).contains("Cargo.toml: missing"));
    assert!(!fixture.0.join("missing").exists());
    for options in [
        vec!["--target", "rust", "--package", "bad-name!"],
        vec![
            "--target",
            "rust",
            "--package",
            "adapter",
            "--module",
            "extra",
        ],
        vec!["--target", "go", "--package", "adapter"],
        vec![
            "--target",
            "go",
            "--package",
            "main",
            "--module",
            "example.invalid/a",
        ],
        vec!["--target", "javascript", "--package", "adapter"],
    ] {
        let mut args = options;
        args.extend(["--out", "missing"]);
        let result = fixture.run("normalize-generate", &args);
        assert!(!result.status.success(), "{result:?}");
        assert!(result.stdout.is_empty());
        assert!(!fixture.0.join("missing").exists());
    }
    let mut recipe = fixture.recipe();
    recipe["branches"]["unused"] = json!([]);
    fs::write(fixture.0.join("recipe.json"), recipe.to_string()).unwrap();
    let result = fixture.run("normalize-generate", &base);
    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("empty_pipeline"));
    assert!(!fixture.0.join("missing").exists());
}

#[test]
fn incompatible_later_destination_preserves_the_entire_existing_output() {
    let fixture = Fixture::new();
    let root = fixture.0.join("generated");
    fs::create_dir_all(root.join("src/schemas.rs")).unwrap();
    fs::write(root.join("Cargo.toml"), "untouched").unwrap();
    let result = fixture.run(
        "normalize-generate",
        &[
            "--target",
            "rust",
            "--package",
            "adapter",
            "--out",
            "generated",
        ],
    );
    assert!(!result.status.success());
    assert!(result.stdout.is_empty());
    assert_eq!(fs::read(root.join("Cargo.toml")).unwrap(), b"untouched");
    assert!(!root.join("source.recipe.json").exists());
    assert!(!root.join("normalization-report.json").exists());
    assert!(!root.join("src/lib.rs").exists());
}

#[cfg(unix)]
#[test]
fn generated_paths_cannot_replace_canonical_source_inputs_or_follow_links() {
    let fixture = Fixture::new();
    fs::rename(
        fixture.0.join("recipe.json"),
        fixture.0.join("source.recipe.json"),
    )
    .unwrap();
    std::os::unix::fs::symlink("source.recipe.json", fixture.0.join("recipe.json")).unwrap();
    let before = fs::read(fixture.0.join("source.recipe.json")).unwrap();
    let result = fixture.run(
        "normalize-generate",
        &["--target", "rust", "--package", "adapter", "--out", "."],
    );
    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("must not replace"));
    assert_eq!(
        fs::read(fixture.0.join("source.recipe.json")).unwrap(),
        before
    );
    assert!(!fixture.0.join("Cargo.toml").exists());
    let root = fixture.0.join("generated");
    fs::create_dir(&root).unwrap();
    std::os::unix::fs::symlink(&fixture.0, root.join("schemas")).unwrap();
    let result = fixture.run(
        "normalize-generate",
        &[
            "--target",
            "rust",
            "--package",
            "adapter",
            "--out",
            "generated",
        ],
    );
    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("symlink"));
    assert!(!root.join("Cargo.toml").exists());
    assert!(!root.join("source.recipe.json").exists());
}

#[test]
fn qualified_go_base64_pattern_is_published_without_decoding_the_value() {
    let fixture = Fixture::new();
    let pattern = "^(?:[A-Za-z0-9+/]{4})*(?:[A-Za-z0-9+/]{2}==|[A-Za-z0-9+/]{3}=)?$";
    let input = Fixture::bundle(
        "Input",
        &json!({"type":"string", "contentEncoding":"base64", "pattern":pattern}),
    );
    fs::write(
        fixture.0.join("input.bundle.json"),
        input.to_json().unwrap(),
    )
    .unwrap();
    let mut recipe = fixture.recipe();
    let root = serde_json::to_value(Root::pin(&input, "Input").unwrap()).unwrap();
    recipe["branches"]["primary"][0]["input"] = root.clone();
    recipe["branches"]["primary"][0]["output"] = root;
    recipe["branches"]["primary"][0]["value"] = json!({"op":"read", "scope":"input", "path":[]});
    fs::write(fixture.0.join("recipe.json"), recipe.to_string()).unwrap();
    // Nonzero unused pad bits satisfy the pattern even though canonical decoders refuse them.
    fs::write(fixture.0.join("instance.json"), r#""AB==""#).unwrap();
    let run = fixture.run("normalize-run", &[]);
    assert!(run.status.success(), "{run:?}");
    assert_eq!(
        serde_json::from_slice::<Value>(&run.stdout).unwrap(),
        "AB=="
    );
    let mut args = vec![
        "--target",
        "go",
        "--package",
        "adapter",
        "--module",
        "example.invalid/adapter",
        "--out",
        "generated",
    ];
    let generated = fixture.run("normalize-generate", &args);
    assert!(generated.status.success(), "{generated:?}");
    let report_path = fixture.0.join("generated/normalization-report.json");
    let report: Value = serde_json::from_slice(&fs::read(report_path).unwrap()).unwrap();
    assert_eq!(report["format"], "ess-normalization-target/1");
    args.push("--check");
    let checked = fixture.run("normalize-generate", &args);
    assert!(checked.status.success(), "{checked:?}");
}

#[test]
fn unsupported_go_pattern_has_no_successful_partial_artifact() {
    let fixture = Fixture::new();
    let input = Fixture::bundle("Input", &json!({"type":"string", "pattern":"^(?=a)a$"}));
    fs::write(
        fixture.0.join("input.bundle.json"),
        input.to_json().unwrap(),
    )
    .unwrap();
    let mut recipe = fixture.recipe();
    let root = serde_json::to_value(Root::pin(&input, "Input").unwrap()).unwrap();
    recipe["branches"]["primary"][0]["input"] = root.clone();
    recipe["branches"]["primary"][0]["output"] = root;
    recipe["branches"]["primary"][0]["value"] = json!({"op":"read", "scope":"input", "path":[]});
    fs::write(fixture.0.join("recipe.json"), recipe.to_string()).unwrap();
    let result = fixture.run(
        "normalize-generate",
        &[
            "--target",
            "go",
            "--package",
            "adapter",
            "--module",
            "example.invalid/adapter",
            "--out",
            "generated",
        ],
    );
    assert!(!result.status.success());
    assert!(result.stdout.is_empty());
    assert!(String::from_utf8_lossy(&result.stderr).contains("go_schema_pattern"));
    assert!(!fixture.0.join("generated").exists());
}

#[test]
fn explicit_binary64_inputs_run_through_the_cli_without_weakening_version_one() {
    let fixture = Fixture::new();
    let input = Fixture::bundle("Input", &json!({"type":"number"}));
    fs::write(
        fixture.0.join("input.bundle.json"),
        input.to_json().unwrap(),
    )
    .unwrap();
    let mut recipe = fixture.recipe();
    recipe["format"] = json!("ess-normalization/2");
    recipe["binary64_inputs"] = json!({"primary":[[]]});
    recipe["branches"]["primary"][0]["input"] =
        serde_json::to_value(Root::pin(&input, "Input").unwrap()).unwrap();
    recipe["branches"]["primary"][0]["value"] = json!({"op":"binary64_to_integer",
        "value":{"op":"read","scope":"input","path":[]},
        "steps":[{"op":"multiply","value":"1000"}],"out_of_range":"reject"});
    fs::write(fixture.0.join("recipe.json"), recipe.to_string()).unwrap();
    fs::write(fixture.0.join("instance.json"), "1.001").unwrap();
    assert!(fixture.run("normalize-check", &[]).status.success());
    let result = fixture.run("normalize-run", &[]);
    assert!(result.status.success(), "{result:?}");
    assert_eq!(result.stdout, b"1000\n");
    fs::write(fixture.0.join("instance.json"), "9223372036854775807").unwrap();
    let result = fixture.run("normalize-run", &["--out", "result.json"]);
    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("binary64_range"));
    assert_eq!(
        fs::read(fixture.0.join("result.json")).unwrap(),
        b"untouched"
    );
    recipe["format"] = json!("ess-normalization/1");
    fs::write(fixture.0.join("recipe.json"), recipe.to_string()).unwrap();
    let result = fixture.run("normalize-check", &[]);
    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("operation_version"));
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
