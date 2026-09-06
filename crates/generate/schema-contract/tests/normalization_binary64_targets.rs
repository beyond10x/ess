//! Native format-5 adapters execute the same independent lexical corpus, serially.

#[path = "fixtures/normalization_binary64.rs"]
mod fixture;

use serde_json::json;
#[cfg(feature = "go-typecheck")]
use serde_json::Value;
use std::{fs, path::Path, process::Command};

fn write_target(root: &Path, files: std::collections::BTreeMap<String, String>) {
    for (path, source) in files {
        let path = root.join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, source).unwrap();
    }
}

#[test]
fn native_rust_executes_finite_binary64_and_retained_documents() {
    let plan = fixture::plan();
    let target = plan.rust("normalization_adapter").unwrap();
    assert_eq!(json!(target.report)["format"], "ess-normalization-target/3");
    let root = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!(
        "normalization-binary64-rust-{}",
        std::process::id()
    ));
    write_target(&root, target.files);
    fs::create_dir_all(root.join("tests")).unwrap();
    fs::write(
        root.join("tests/cases.json"),
        json!(fixture::cases()).to_string(),
    )
    .unwrap();
    fs::write(
        root.join("tests/behavior.rs"),
        include_str!("fixtures/normalization_binary64_rust_tests.rs.txt"),
    )
    .unwrap();
    for features in [None, Some("serde_json/arbitrary_precision")] {
        let mut command = Command::new(env!("CARGO"));
        command
            .args(["test", "--offline", "--quiet", "--manifest-path"])
            .arg(root.join("Cargo.toml"))
            .env(
                "CARGO_TARGET_DIR",
                Path::new(env!("CARGO_TARGET_TMPDIR")).join("normalization-rust-target"),
            );
        if let Some(features) = features {
            command.args(["--features", features]);
        }
        command.args(["--", "--nocapture"]);
        let output = command.output().unwrap();
        fs::write(
            root.join(if features.is_some() {
                "native-arbitrary.log"
            } else {
                "native-default.log"
            }),
            format!(
                "{}\n{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            ),
        )
        .unwrap();
        assert!(
            output.status.success(),
            "{features:?}: {}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        eprintln!(
            "native Rust {features:?}: {} cases, exit {}",
            fixture::cases().len(),
            output.status
        );
        eprintln!(
            "{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[cfg(feature = "go-typecheck")]
#[test]
fn native_go_executes_finite_binary64_and_retained_documents() {
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
    let target = fixture::plan()
        .go(
            "normalization_adapter",
            "example.invalid/normalization-adapter",
        )
        .unwrap();
    assert_eq!(json!(target.report)["format"], "ess-normalization-target/3");
    let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("normalization-binary64-go-{}", std::process::id()));
    write_target(&root, target.files);
    let cases = fixture::cases();
    fs::write(root.join("cases.json"), json!(cases).to_string()).unwrap();
    fs::write(
        root.join("behavior_test.go"),
        include_str!("fixtures/normalization_binary64_go_tests.go.txt"),
    )
    .unwrap();
    let output = Command::new(&compiler)
        .args([
            "test",
            "-json",
            "-count=1",
            "-race",
            "-mod=readonly",
            "./...",
        ])
        .current_dir(&root)
        .env("GOPROXY", "off")
        .env("GOSUMDB", "off")
        .env("GOTOOLCHAIN", "local")
        .env("GOFLAGS", "")
        .env(
            "GOCACHE",
            Path::new(env!("CARGO_TARGET_TMPDIR")).join("binary64-go-cache"),
        )
        .env("GOMAXPROCS", "4")
        .output()
        .unwrap();
    fs::write(root.join("native-go.jsonl"), &output.stdout).unwrap();
    assert!(
        output.status.success(),
        "{}\n{}",
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
        "native Go: {passed} corpus cases plus retained-document helper, exit {}",
        output.status
    );
}
