//! Independent executable CLI controls; no optional native tool is required.

use schema_contract::bundle::{import, Dialect};
use schema_contract::realize::normalize::Root;
use serde_json::{json, Value};
use std::{
    fs,
    path::PathBuf,
    process::{Command, Output},
};

struct Fixture {
    root: PathBuf,
    recipe: Value,
}

impl Fixture {
    fn new(name: &str) -> Self {
        let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!(
            "positional-adversary-cli-{name}-{}",
            std::process::id()
        ));
        fs::create_dir_all(&root).unwrap();
        let source = json!({"components":{"schemas":{"Pair":{"type":"array","prefixItems":[{"type":"string"},{"type":"string"}],"items":false,"minItems":2,"maxItems":2},"String":{"type":"string"}}}});
        let bundle = import(
            &source.to_string(),
            &["Pair".to_owned(), "String".to_owned()]
                .into_iter()
                .collect(),
            Dialect::Draft202012,
        )
        .unwrap();
        fs::write(root.join("source.bundle.json"), bundle.to_json().unwrap()).unwrap();
        let recipe = json!({"format":"ess-normalization/6","branches":{"pick/~":[{"input":Root::pin(&bundle,"Pair").unwrap(),"output":Root::pin(&bundle,"String").unwrap(),"requires":[],"value":{"op":"position","value":{"op":"read","scope":"input","path":[]},"index":1}}]},"positional_inputs":{"pick/~":[{"path":[],"kind":"fixed_string_array","length":2,"missing":"preserve","null":"zero","short":"zero_pad","extra":"discard","null_element":"zero"}]}});
        fs::write(root.join("recipe.json"), recipe.to_string()).unwrap();
        fs::write(
            root.join("input.json"),
            r#"[null,"literal",{"x":1e999,"x":0}]"#,
        )
        .unwrap();
        fs::write(root.join("sentinel.json"), b"preserved\n").unwrap();
        Self { root, recipe }
    }

    fn command(&self, op: &str, args: &[&str]) -> Output {
        let output = Command::new(env!("CARGO_BIN_EXE_ess"))
            .current_dir(&self.root)
            .args([
                "generate",
                "schema",
                op,
                "--recipe",
                "recipe.json",
                "--bundle",
                "source.bundle.json",
            ])
            .args(args)
            .output()
            .unwrap();
        eprintln!(
            "{op} {args:?}: exit {}\nstdout: {}\nstderr: {}",
            output.status,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        output
    }
}

#[test]
fn cli_runtime_grammar_controls_preserve_existing_output() {
    let fixture = Fixture::new("runtime");
    let args = ["--branch", "pick/~", "--input", "input.json"];
    let positive = fixture.command("normalize-run", &args);
    assert!(positive.status.success());
    assert_eq!(positive.stdout, b"\"literal\"\n");
    for (input, rule, pointer) in [
        (r#"[false,null,{"broken":}]"#, "input_syntax", "/input"),
        (
            r#"["ok",{"bad":"\ud800"}]"#,
            "positional_element_type",
            "/input/1",
        ),
        (
            r#"[null,null,{"x":1e999,"x":"\ud800"}]"#,
            "input_syntax",
            "/input/2",
        ),
    ] {
        fs::write(fixture.root.join("input.json"), input).unwrap();
        let mut arguments = args.to_vec();
        arguments.extend(["--out", "sentinel.json"]);
        let result = fixture.command("normalize-run", &arguments);
        assert!(!result.status.success() && result.stdout.is_empty());
        let stderr = String::from_utf8_lossy(&result.stderr);
        assert!(
            stderr.contains(rule) && stderr.contains(pointer),
            "{stderr}"
        );
        assert_eq!(
            fs::read(fixture.root.join("sentinel.json")).unwrap(),
            b"preserved\n"
        );
        assert_eq!(
            fs::read_to_string(fixture.root.join("input.json")).unwrap(),
            input
        );
    }
}

#[test]
fn cli_invalid_policy_blocks_all_publication_and_valid_metadata_is_drift_checked() {
    let fixture = Fixture::new("publication");
    let mut invalid = fixture.recipe.clone();
    invalid["positional_inputs"]["pick/~"][0]["length"] = json!(u64::MAX);
    fs::write(fixture.root.join("recipe.json"), invalid.to_string()).unwrap();
    for (target, extra) in [
        ("rust", vec![]),
        ("go", vec!["--module", "example.invalid/adversary"]),
    ] {
        let mut args = vec![
            "--target",
            target,
            "--package",
            "adversary",
            "--out",
            target,
        ];
        args.extend(extra);
        let refused = fixture.command("normalize-generate", &args);
        assert!(!refused.status.success() && refused.stdout.is_empty());
        assert!(String::from_utf8_lossy(&refused.stderr).contains("positional_schema"));
        assert!(!fixture.root.join(target).exists());
    }
    let refused = fixture.command("normalize-check", &["--out", "sentinel.json"]);
    assert!(!refused.status.success());
    assert_eq!(
        fs::read(fixture.root.join("sentinel.json")).unwrap(),
        b"preserved\n"
    );
    fs::write(fixture.root.join("recipe.json"), fixture.recipe.to_string()).unwrap();
    for (target, binding, extra) in [
        ("rust", "src/schemas.rs", vec![]),
        (
            "go",
            "bindings.go",
            vec!["--module", "example.invalid/adversary"],
        ),
    ] {
        let mut args = vec![
            "--target",
            target,
            "--package",
            "adversary",
            "--out",
            target,
        ];
        args.extend(extra);
        assert!(fixture
            .command("normalize-generate", &args)
            .status
            .success());
        let report: Value = serde_json::from_slice(
            &fs::read(fixture.root.join(target).join("normalization-report.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(report["format"], "ess-normalization-target/3");
        args.push("--check");
        assert!(fixture
            .command("normalize-generate", &args)
            .status
            .success());
        let path = fixture.root.join(target).join(binding);
        let mut bytes = fs::read(&path).unwrap();
        bytes.extend(b"\n// independent metadata drift\n");
        fs::write(&path, &bytes).unwrap();
        let drift = fixture.command("normalize-generate", &args);
        assert!(!drift.status.success());
        assert_eq!(fs::read(&path).unwrap(), bytes);
    }
}
