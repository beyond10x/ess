//! `ess verify conform synthesize --target typescript|go` emits the model-based explorer.
//!
//! `docs/design/mutation-audit-and-model-runner.md`, Part 2, "What is emitted": the explorer and
//! the compact `ir.json` it interprets are part of every TypeScript and Go package, with no flag,
//! for the ordinary suite and for the coverage carrier alike. The explorer itself is executed in
//! `crates/verify/ess-conformance/tests/explore.rs`.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use sha2::{Digest as _, Sha256};

const FIXTURE: &str = "crates/verify/ess-conformance/tests/fixtures/explore.yaml";

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("the workspace root exists")
}

fn ess(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ess"))
        .current_dir(root())
        .args(args)
        .output()
        .expect("the `ess` binary runs")
}

fn scratch(name: &str) -> PathBuf {
    let directory = root()
        .join("target/explore-package")
        .join(format!("{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&directory);
    std::fs::create_dir_all(&directory).expect("a scratch directory");
    directory
}

fn hex(bytes: &[u8]) -> String {
    use std::fmt::Write as _;
    Sha256::digest(bytes)
        .iter()
        .fold(String::new(), |mut out, byte| {
            let _ = write!(out, "{byte:02x}");
            out
        })
}

/// The emitted `ir.json` is one line plus LF, and it hashes to the suite's `spec_digest`.
fn assert_bound(package: &Path) {
    let ir = std::fs::read_to_string(package.join("ir.json")).expect("ir.json is emitted");
    assert!(
        ir.ends_with('\n') && !ir[..ir.len() - 1].contains('\n'),
        "compact JSON plus one LF"
    );
    let suite: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(package.join("suite.json")).unwrap())
            .unwrap();
    assert_eq!(
        suite["provenance"]["spec_digest"].as_str(),
        Some(hex(ir.trim_end_matches('\n').as_bytes()).as_str())
    );
}

#[test]
fn the_typescript_package_carries_the_explorer_and_its_model() {
    let out = scratch("typescript");
    let output = ess(&[
        "verify",
        "conform",
        "synthesize",
        "--path",
        FIXTURE,
        "--target",
        "typescript",
        "--out",
        out.to_str().unwrap(),
    ]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let package = out.join("essconform");
    assert!(package.join("src/explore.ts").is_file());
    let index = std::fs::read_to_string(package.join("src/index.ts")).unwrap();
    assert!(
        index.ends_with("export * from './explore.js';\n"),
        "{index}"
    );
    let readme = std::fs::read_to_string(package.join("README.md")).unwrap();
    assert!(readme.contains("## Random command sequences"), "{readme}");
    assert_bound(&package);
}

#[test]
fn the_go_package_carries_the_explorer_and_its_model() {
    let out = scratch("go");
    let output = ess(&[
        "verify",
        "conform",
        "synthesize",
        "--path",
        FIXTURE,
        "--target",
        "go",
        "--out",
        out.to_str().unwrap(),
    ]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let package = out.join("essconform");
    let explore = std::fs::read_to_string(package.join("explore.go")).expect("explore.go");
    assert!(
        explore.contains("//go:embed ir.json"),
        "the explorer embeds its model"
    );
    assert_bound(&package);
}

#[test]
fn the_coverage_carrier_packages_carry_the_explorer_too() {
    for (target, source) in [("typescript", "src/explore.ts"), ("go", "explore.go")] {
        let out = scratch(&format!("coverage-{target}"));
        let output = ess(&[
            "verify",
            "conform",
            "synthesize",
            "--path",
            FIXTURE,
            "--target",
            target,
            "--suite-format",
            "5",
            "--out",
            out.to_str().unwrap(),
        ]);
        assert!(
            output.status.success(),
            "{target}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let package = out.join("essconform");
        assert!(package.join(source).is_file(), "{target}");
        assert!(
            package.join("input.json").is_file(),
            "{target}: still the coverage carrier"
        );
        assert_bound(&package);
    }
}
