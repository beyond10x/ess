//! Actual format-6 targets execute positional and inherited Binary64 corpora serially.

use fixture::binary64;
#[path = "fixtures/normalization_positional.rs"]
mod fixture;

use schema_contract::realize::normalize::Plan;
use serde_json::json;
#[cfg(feature = "go-typecheck")]
use serde_json::Value;
use std::{fs, path::Path, process::Command};

fn binary64_plan() -> Plan {
    let (model, mut recipe) = binary64::fixture();
    recipe["format"] = json!("ess-normalization/6");
    Plan::check_with_models(serde_json::from_value(recipe).unwrap(), &[], &[model]).unwrap()
}

fn write_target(root: &Path, files: std::collections::BTreeMap<String, String>) {
    for (path, source) in files {
        let path = root.join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, source).unwrap();
    }
}

#[test]
fn native_rust_executes_position_and_binary64_contracts() {
    for (name, plan, cases, tests, internal) in [
        (
            "positional",
            fixture::plan(),
            fixture::cases(),
            include_str!("fixtures/normalization_positional_rust_tests.rs.txt"),
            true,
        ),
        (
            "mixed-six",
            fixture::mixed_plan(),
            fixture::mixed_cases(),
            include_str!("fixtures/normalization_positional_rust_tests.rs.txt"),
            true,
        ),
        (
            "binary64-six",
            binary64_plan(),
            binary64::cases(),
            include_str!("fixtures/normalization_binary64_rust_tests.rs.txt"),
            false,
        ),
    ] {
        let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join(format!("normalization-{name}-rust-{}", std::process::id()));
        let target = plan.rust("normalization_adapter").unwrap();
        assert_eq!(json!(target.report)["format"], "ess-normalization-target/3");
        write_target(&root, target.files);
        fs::write(root.join("cases.json"), json!(cases).to_string()).unwrap();
        if internal {
            let path = root.join("src/lib.rs");
            let mut source = fs::read_to_string(&path).unwrap();
            source.push_str(tests);
            fs::write(path, source).unwrap();
        } else {
            fs::create_dir_all(root.join("tests")).unwrap();
            fs::write(root.join("tests/cases.json"), json!(cases).to_string()).unwrap();
            fs::write(root.join("tests/behavior.rs"), tests).unwrap();
        }
        let lock = Command::new(env!("CARGO"))
            .args(["generate-lockfile", "--offline", "--manifest-path"])
            .arg(root.join("Cargo.toml"))
            .output()
            .unwrap();
        fs::write(
            root.join("native-lock.log"),
            format!(
                "{}\n{}",
                String::from_utf8_lossy(&lock.stdout),
                String::from_utf8_lossy(&lock.stderr)
            ),
        )
        .unwrap();
        assert!(
            lock.status.success(),
            "{}",
            String::from_utf8_lossy(&lock.stderr)
        );
        for features in [None, Some("serde_json/arbitrary_precision")] {
            let mut command = Command::new(env!("CARGO"));
            command
                .args([
                    "test",
                    "--offline",
                    "--locked",
                    "--quiet",
                    "--manifest-path",
                ])
                .arg(root.join("Cargo.toml"))
                .env(
                    "CARGO_TARGET_DIR",
                    Path::new(env!("CARGO_TARGET_TMPDIR")).join("normalization-rust-target"),
                );
            if let Some(features) = features {
                command.args(["--features", features]);
            }
            command.args(["--", "--nocapture", "--test-threads=1"]);
            let output = command.output().unwrap();
            let log = format!(
                "{}\n{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
            fs::write(
                root.join(if features.is_some() {
                    "native-arbitrary.log"
                } else {
                    "native-default.log"
                }),
                &log,
            )
            .unwrap();
            assert!(output.status.success(), "{name} {features:?}: {log}");
            eprintln!(
                "native Rust {name} {features:?}: {} corpus vectors, exit {}\n{log}",
                cases.len(),
                output.status
            );
        }
    }
}

#[cfg(feature = "go-typecheck")]
#[test]
fn native_go_executes_position_and_binary64_contracts() {
    let compiler = std::path::PathBuf::from(
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
            .and_then(|s| s.split('-').next()),
        Some("go1.26.5")
    );
    // Go has no decoded-value API. Those six explicit provenance cases execute in
    // reference and native Rust; every text/base64 case executes here without skips.
    let positional_cases = fixture::cases()
        .into_iter()
        .filter(|case| case["mode"] != "value")
        .collect::<Vec<_>>();
    for (name, plan, cases, tests) in [
        (
            "positional",
            fixture::plan(),
            positional_cases,
            include_str!("fixtures/normalization_positional_go_tests.go.txt"),
        ),
        (
            "mixed-six",
            fixture::mixed_plan(),
            fixture::mixed_cases(),
            include_str!("fixtures/normalization_positional_go_tests.go.txt"),
        ),
        (
            "binary64-six",
            binary64_plan(),
            binary64::cases(),
            include_str!("fixtures/normalization_binary64_go_tests.go.txt"),
        ),
    ] {
        let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
            .join(format!("normalization-{name}-go-{}", std::process::id()));
        let target = plan
            .go(
                "normalization_adapter",
                "example.invalid/normalization-adapter",
            )
            .unwrap();
        assert_eq!(json!(target.report)["format"], "ess-normalization-target/3");
        write_target(&root, target.files);
        fs::write(root.join("cases.json"), json!(cases).to_string()).unwrap();
        fs::write(root.join("behavior_test.go"), tests).unwrap();
        let output = Command::new(&compiler)
            .args([
                "test",
                "-json",
                "-count=1",
                "-race",
                "-mod=readonly",
                "-p",
                "1",
                "./...",
            ])
            .current_dir(&root)
            .env("GOPROXY", "off")
            .env("GOSUMDB", "off")
            .env("GOTOOLCHAIN", "local")
            .env("GOFLAGS", "")
            .env("GOMAXPROCS", "4")
            .env(
                "GOCACHE",
                Path::new(env!("CARGO_TARGET_TMPDIR")).join("positional-go-cache"),
            )
            .output()
            .unwrap();
        fs::write(root.join("native-go.jsonl"), &output.stdout).unwrap();
        assert!(
            output.status.success(),
            "{name}: {}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        let passed = String::from_utf8_lossy(&output.stdout)
            .lines()
            .filter_map(|line| serde_json::from_str::<Value>(line).ok())
            .filter(|row| {
                row["Action"] == "pass"
                    && row["Test"]
                        .as_str()
                        .is_some_and(|test| test.starts_with("TestCorpus/"))
            })
            .count();
        assert_eq!(passed, cases.len());
        eprintln!(
            "native Go {name}: executed {passed} corpus vectors, exit {}",
            output.status
        );
    }
}
