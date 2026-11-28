//! Actual format-6 targets execute positional and inherited Binary64 corpora serially.
//!
//! No test builds an adapter: `task schema-contract-native-crates` builds every crate of
//! [`native_crates`] once, before the tests (`support/native_crate.rs`).

use fixture::binary64;
#[path = "fixtures/normalization_positional.rs"]
mod fixture;

#[path = "support/native_crate.rs"]
mod native_crate;
use native_crate::NativeCrate;

use schema_contract::realize::normalize::Plan;
use serde_json::json;
#[cfg(feature = "go-typecheck")]
use serde_json::Value;
#[cfg(feature = "go-typecheck")]
use std::{fs, path::Path, process::Command};

fn binary64_plan() -> Plan {
    let (model, mut recipe) = binary64::fixture();
    recipe["format"] = json!("ess-normalization/6");
    Plan::check_with_models(serde_json::from_value(recipe).unwrap(), &[], &[model]).unwrap()
}

#[cfg(feature = "go-typecheck")]
fn write_target(root: &Path, files: std::collections::BTreeMap<String, String>) {
    for (path, source) in files {
        let path = root.join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, source).unwrap();
    }
}

/// Every adapter a test of this binary runs, with the number of its corpus vectors.
fn native_crates() -> Vec<(NativeCrate, usize)> {
    [
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
    ]
    .into_iter()
    .map(|(name, plan, cases, tests, internal)| {
        let target = plan.rust("normalization_adapter").unwrap();
        assert_eq!(json!(target.report)["format"], "ess-normalization-target/3");
        let mut files = target.files;
        files.insert("cases.json".to_owned(), json!(cases).to_string());
        if internal {
            files
                .get_mut("src/lib.rs")
                .expect("a generated src/lib.rs")
                .push_str(tests);
        } else {
            files.insert("tests/cases.json".to_owned(), json!(cases).to_string());
            files.insert("tests/behavior.rs".to_owned(), tests.to_owned());
        }
        (
            NativeCrate::new(&format!("normalization_positional_targets/{name}"), files),
            cases.len(),
        )
    })
    .collect()
}

/// Writes the sources of every crate of [`native_crates`] for `task
/// schema-contract-native-crates` to build. A step of that task, never a test of the suite.
#[test]
#[ignore = "generator: `task schema-contract-native-crates` runs it before the tests"]
fn write_the_sources_of_every_native_crate() {
    native_crate::write_sources(
        native_crates()
            .into_iter()
            .map(|(native, _)| native)
            .collect(),
    );
}

#[test]
fn native_rust_executes_position_and_binary64_contracts() {
    for (native, cases) in native_crates() {
        let log = native.run(&["--nocapture", "--test-threads=1"]);
        eprintln!("native Rust: {cases} corpus vectors\n{log}");
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
