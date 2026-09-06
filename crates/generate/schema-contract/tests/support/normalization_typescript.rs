//! Rust owns native execution and lossless expected values; no Node type dependency.
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

use schema_contract::realize::normalize::Plan;
use serde_json::{json, Value};

pub fn compiler() -> &'static Path {
    static COMPILER: OnceLock<PathBuf> = OnceLock::new();
    COMPILER.get_or_init(|| {
        let path = PathBuf::from(std::env::var_os("ESS_TYPESCRIPT_COMPILER")
            .expect("typescript-typecheck requires ESS_TYPESCRIPT_COMPILER"));
        assert!(path.is_absolute() && path.is_file());
        let output = Command::new("node").arg(&path).arg("--version").output().unwrap();
        assert!(output.status.success());
        assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "Version 6.0.3");
        let engine = Command::new("node").args(["-p", "JSON.stringify({node:process.version,v8:process.versions.v8,execPath:process.execPath})"]).output().unwrap();
        assert!(engine.status.success());
        let identity: Value = serde_json::from_slice(&engine.stdout).unwrap();
        assert_eq!(identity["node"], "v22.23.1");
        assert_eq!(identity["v8"], "12.4.254.21-node.56");
        eprintln!("qualified engine {identity}; compiler {}", path.display());
        path
    })
}

pub fn write_package(name: &str, plan: &Plan) -> PathBuf {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!(
        "normalization-typescript-{}-{name}",
        std::process::id()
    ));
    for (path, source) in plan.typescript("normalization-adapter").unwrap().files {
        let path = root.join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, source).unwrap();
    }
    root
}

pub fn compile(root: &Path) {
    let output = Command::new("node")
        .arg(compiler())
        .args(["--pretty", "false", "--project", "tsconfig.json"])
        .current_dir(root)
        .output()
        .unwrap();
    fs::write(root.join("typescript-compile.stdout.log"), &output.stdout).unwrap();
    fs::write(root.join("typescript-compile.stderr.log"), &output.stderr).unwrap();
    assert!(
        output.status.success(),
        "{}: compiler {}\n{}\n{}",
        root.display(),
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

pub fn run(root: &Path) {
    let output = Command::new("node")
        .arg("dist/qualification.js")
        .current_dir(root)
        .output()
        .unwrap();
    fs::write(root.join("node.stdout.log"), &output.stdout).unwrap();
    fs::write(root.join("node.stderr.log"), &output.stderr).unwrap();
    assert!(
        output.status.success(),
        "{}: native {}\n{}\n{}",
        root.display(),
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    eprint!("{}", String::from_utf8_lossy(&output.stdout));
}

pub fn run_cases(name: &str, plan: &Plan, cases: &[Value]) -> usize {
    let cases = cases
        .iter()
        .map(|case| {
            let mut case = case.clone();
            let branch = case["branch"].as_str().unwrap();
            let input = case["input"].as_str().unwrap();
            let result = if case["mode"] == "base64" {
                plan.run_base64_json(branch, input)
            } else {
                plan.run_json(branch, input)
            };
            match result {
                Ok(actual) => {
                    if let Some(expected) = case.get("value") {
                        assert_eq!(expected, &actual, "{case}");
                    }
                    if let Some(bits) = case["bits"].as_str() {
                        assert_eq!(format!("{:016x}", actual.as_f64().unwrap().to_bits()), bits);
                    }
                    // Expected text comes from the independent stored value where available.
                    // Bit-only stored cases retain their independent bit assertions below.
                    let expected = case.get("value").unwrap_or(&actual).to_string();
                    case["expected_json"] = json!(expected);
                }
                Err(mut errors) => {
                    if errors.0.iter().all(|f| f.rule == "schema_validation") {
                        errors.0.sort_by(|a, b| a.pointer.cmp(&b.pointer));
                    }
                    if let Some(expected) = case.get("errors") {
                        let mut expected = expected.as_array().unwrap().clone();
                        if expected.iter().all(|f| f["rule"] == "schema_validation") {
                            expected.sort_by(|a, b| {
                                a["pointer"]
                                    .as_str()
                                    .unwrap()
                                    .cmp(b["pointer"].as_str().unwrap())
                            });
                        }
                        assert_eq!(json!(expected), json!(errors.0), "{case}");
                    } else {
                        assert_eq!(case["error"], errors.0[0].rule, "{case}");
                    }
                    case["expected_errors"] = json!(errors.0);
                }
            }
            case.as_object_mut().unwrap().remove("value");
            case
        })
        .collect::<Vec<_>>();
    let root = write_package(name, plan);
    fs::write(
        root.join("cases.json"),
        serde_json::to_string_pretty(&cases).unwrap(),
    )
    .unwrap();
    let source = include_str!("../fixtures/normalization_typescript_tests.ts.txt")
        .replace(
            "__CASES_JSON_STRING__",
            &serde_json::to_string(&serde_json::to_string(&cases).unwrap()).unwrap(),
        )
        .replace("__CASE_LABEL__", &serde_json::to_string(name).unwrap());
    fs::write(root.join("src/qualification.ts"), source).unwrap();
    compile(&root);
    run(&root);
    cases.len()
}
