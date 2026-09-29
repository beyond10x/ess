//! `ess synthesize --layout` (`story:single-crate-rust-layout`), through both CLI spellings.

use std::path::{Path, PathBuf};
use std::process::Command;

fn gatepass() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../examples/gatepass")
}

fn run(verb: &[&str], extra: &[&str], out: &Path) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_ess"))
        .args(verb)
        .arg("--path")
        .arg(gatepass())
        .args(extra)
        .arg("--out")
        .arg(out)
        .output()
        .expect("the ess binary runs")
}

#[test]
fn the_rust_target_writes_one_crate_at_the_output_root() {
    for verb in [&["synthesize"][..], &["generate", "synthesize"][..]] {
        let dir = tempfile::tempdir().unwrap();
        let out = dir.path().join("gatepass");
        let output = run(verb, &["--layout", "crate"], &out);
        assert!(output.status.success(), "{verb:?}: {output:?}");
        assert!(out.join("Cargo.toml").is_file(), "{verb:?}");
        assert!(out.join("src/lib.rs").is_file(), "{verb:?}");
        assert!(out.join("src/ports/pass_service.rs").is_file(), "{verb:?}");
        assert!(out.join("src/server/http.rs").is_file(), "{verb:?}");
        assert!(!out.join("crates").exists(), "{verb:?}");
        let plan = std::fs::read_to_string(out.join("plan.json")).unwrap();
        assert!(plan.contains("\"layout\": \"crate\""), "{plan}");
    }
}

#[test]
fn the_default_layout_is_the_workspace() {
    let dir = tempfile::tempdir().unwrap();
    let explicit = dir.path().join("explicit");
    let default = dir.path().join("default");
    assert!(run(&["synthesize"], &["--layout", "workspace"], &explicit)
        .status
        .success());
    assert!(run(&["synthesize"], &[], &default).status.success());
    assert!(default.join("crates/gatepass-types/Cargo.toml").is_file());
    for relative in ["Cargo.toml", "plan.json", "PLAN.md"] {
        assert_eq!(
            std::fs::read(explicit.join(relative)).unwrap(),
            std::fs::read(default.join(relative)).unwrap(),
            "{relative}"
        );
    }
}

#[test]
fn every_other_target_refuses_the_crate_layout_and_writes_nothing() {
    for target in ["go", "web", "clap"] {
        for verb in [&["synthesize"][..], &["generate", "synthesize"][..]] {
            let dir = tempfile::tempdir().unwrap();
            let out = dir.path().join("out");
            let output = run(verb, &["--target", target, "--layout", "crate"], &out);
            assert_eq!(
                output.status.code(),
                Some(2),
                "{target} {verb:?}: {output:?}"
            );
            let stderr = String::from_utf8_lossy(&output.stderr);
            assert!(
                stderr.contains(&format!("the `{target}` target has no `crate` layout")),
                "{target}: {stderr}"
            );
            assert!(!out.exists(), "{target}: nothing is written");
        }
    }
}
