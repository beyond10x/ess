//! Actual CLI composition of explicit finite floats with retained text and exact siblings.

#[path = "../../../generate/schema-contract/tests/fixtures/binary64_adversary.rs"]
mod fixture;

#[test]
fn cli_composition_obeys_the_independently_authored_vectors() {
    use std::{fs, path::Path, process::Command};
    let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("binary64-adversary-cli-{}", std::process::id()));
    fs::create_dir_all(root.join("model")).unwrap();
    fs::write(root.join("model/system.yaml"), fixture::SOURCE).unwrap();
    fs::write(root.join("recipe.json"), fixture::fixture().1.to_string()).unwrap();
    for (index, case) in fixture::cases().as_array().unwrap().iter().enumerate() {
        fs::write(root.join("input.json"), case["input"].as_str().unwrap()).unwrap();
        let output = Command::new(env!("CARGO_BIN_EXE_ess"))
            .current_dir(&root)
            .args([
                "generate",
                "schema",
                "normalize-run",
                "--recipe",
                "recipe.json",
                "--model",
                "model",
                "--input",
                "input.json",
                "--branch",
                "primary",
            ])
            .output()
            .unwrap();
        let text = format!(
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        fs::write(
            root.join(format!("case-{index}.log")),
            format!("exit {}\n{text}", output.status),
        )
        .unwrap();
        if let Some(rule) = case["error"].as_str() {
            assert!(!output.status.success(), "{case}: {text}");
            assert!(text.contains(rule), "{case}: {text}");
            if let Some(pointer) = case["pointer"].as_str() {
                assert!(text.contains(pointer), "{case}: {text}");
            }
        } else {
            assert!(output.status.success(), "{case}: {text}");
            fixture::assert_case(case, Ok(serde_json::from_slice(&output.stdout).unwrap()));
        }
    }
    eprintln!("8 actual CLI nested/raw/exact/default/error-order vectors");
}
