//! Publication cannot depend on an installed native toolchain.
use schema_contract::bundle::{import, Dialect};
use schema_contract::realize::normalize::Root;
use serde_json::json;
use std::fs;
use std::process::Command;

#[test]
fn generation_and_check_need_no_native_tools_and_module_refusal_preserves_bytes() {
    let root = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("typescript-adversary-cli-{}", std::process::id()));
    fs::create_dir_all(&root).unwrap();
    let bundle = import(
        &json!({"components":{"schemas":{"Text":{"type":"string"}}}}).to_string(),
        &["Text".to_owned()].into_iter().collect(),
        Dialect::Draft202012,
    )
    .unwrap();
    let pinned = Root::pin(&bundle, "Text").unwrap();
    let recipe = json!({"format":"ess-normalization/6","branches":{"copy":[{"input":pinned,"output":pinned,"requires":[],"value":{"op":"read","scope":"input","path":[]}}]}});
    fs::write(root.join("input.bundle.json"), bundle.to_json().unwrap()).unwrap();
    fs::write(root.join("input.recipe.json"), recipe.to_string()).unwrap();
    let run = |extra: &[&str]| {
        Command::new(env!("CARGO_BIN_EXE_ess"))
            .current_dir(&root)
            .env("PATH", "")
            .env_remove("ESS_TYPESCRIPT_COMPILER")
            .env_remove("ESS_GO_COMPILER")
            .args([
                "generate",
                "schema",
                "normalize-generate",
                "--recipe",
                "input.recipe.json",
                "--bundle",
                "input.bundle.json",
                "--target",
                "typescript",
                "--package",
                "@scope/adapter",
                "--out",
                "generated",
            ])
            .args(extra)
            .output()
            .unwrap()
    };
    let output = run(&[]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report: serde_json::Value = serde_json::from_slice(
        &fs::read(root.join("generated/normalization-report.json")).unwrap(),
    )
    .unwrap();
    let snapshot = report["files"]
        .as_object()
        .unwrap()
        .keys()
        .chain(std::iter::once(&"normalization-report.json".to_owned()))
        .map(|name| {
            (
                name.clone(),
                fs::read(root.join("generated").join(name)).unwrap(),
            )
        })
        .collect::<Vec<_>>();
    assert!(run(&["--check"]).status.success());
    let refused = run(&["--module", "example.invalid/unused"]);
    assert!(!refused.status.success());
    assert!(String::from_utf8_lossy(&refused.stderr).contains("--module"));
    for (name, before) in snapshot {
        assert_eq!(
            fs::read(root.join("generated").join(&name)).unwrap(),
            before,
            "{name}"
        );
    }
    assert!(run(&["--check"]).status.success());
    assert!(!root.join("generated/dist").exists());
    assert!(!root.join("generated/node_modules").exists());
}
