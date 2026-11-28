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
fn the_crate_takes_a_package_name_and_leaves_out_its_workspace_table_on_request() {
    for verb in [&["synthesize"][..], &["generate", "synthesize"][..]] {
        let dir = tempfile::tempdir().unwrap();
        let out = dir.path().join("member");
        let output = run(
            verb,
            &[
                "--layout",
                "crate",
                "--package",
                "gatepass-model",
                "--no-workspace-table",
            ],
            &out,
        );
        assert!(output.status.success(), "{verb:?}: {output:?}");
        let manifest = std::fs::read_to_string(out.join("Cargo.toml")).unwrap();
        assert!(
            manifest.contains("\nname = \"gatepass-model\"\n"),
            "{manifest}"
        );
        assert!(!manifest.contains("[workspace]"), "{manifest}");
        let plan: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(out.join("plan.json")).unwrap()).unwrap();
        assert_eq!(plan["scope"]["package"], "gatepass-model", "{plan}");
        assert_eq!(plan["scope"]["no_workspace_table"], true, "{plan}");
    }
}

#[test]
fn the_manifest_options_are_refused_outside_the_crate_layout_and_write_nothing() {
    for (extra, refusal) in [
        (
            &["--package", "gatepass-model"][..],
            "`--package` is an option of the one crate",
        ),
        (
            &["--no-workspace-table"][..],
            "`--no-workspace-table` is an option of the one crate",
        ),
        (
            &["--layout", "crate", "--package", "Gatepass"][..],
            "`Gatepass` cannot be the crate's `--package`",
        ),
    ] {
        let dir = tempfile::tempdir().unwrap();
        let out = dir.path().join("out");
        let output = run(&["generate", "synthesize"], extra, &out);
        assert_eq!(output.status.code(), Some(2), "{extra:?}: {output:?}");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains(refusal), "{extra:?}: {stderr}");
        assert!(!out.exists(), "{extra:?}: nothing is written");
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
