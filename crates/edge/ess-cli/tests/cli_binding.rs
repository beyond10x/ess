//! Public CLI routes resolve first and obey existing generated-output ownership.

use std::{
    fs,
    path::Path,
    process::{Command, Output},
};

const MODEL: &str = include_str!("../../../specify/ess-cli-contract/tests/fixtures/model.yaml");
const BINDING: &str = include_str!("../../../specify/ess-cli-contract/tests/fixtures/cli.yaml");

fn fixture() -> tempfile::TempDir {
    let temp = tempfile::tempdir().unwrap();
    fs::write(temp.path().join("model.yaml"), MODEL).unwrap();
    fs::write(temp.path().join("cli.yaml"), BINDING).unwrap();
    temp
}

fn ess(root: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ess"))
        .current_dir(root)
        .args(args)
        .output()
        .unwrap()
}

fn generate(root: &Path, check: bool) -> Output {
    let mut args = vec![
        "generate",
        "cli",
        "--path",
        "model.yaml",
        "--binding",
        "cli.yaml",
        "--out",
        "generated",
    ];
    if check {
        args.push("--check");
    }
    ess(root, &args)
}

#[test]
fn binding_routes_validate_generate_compare_and_refuse_drift() {
    let temp = fixture();
    let valid = ess(
        temp.path(),
        &[
            "specify",
            "cli",
            "--path",
            "model.yaml",
            "--binding",
            "cli.yaml",
            "--format",
            "json",
        ],
    );
    assert!(
        valid.status.success(),
        "{}",
        String::from_utf8_lossy(&valid.stderr)
    );
    serde_json::from_slice::<serde_json::Value>(&valid.stdout).unwrap();
    let generated = generate(temp.path(), false);
    assert!(
        generated.status.success(),
        "{}",
        String::from_utf8_lossy(&generated.stderr)
    );
    let before = fs::read(temp.path().join("generated/src/lib.rs")).unwrap();
    assert!(generate(temp.path(), true).status.success());
    fs::write(temp.path().join("generated/src/lib.rs"), "changed").unwrap();
    assert!(!generate(temp.path(), true).status.success());
    assert_eq!(
        fs::read(temp.path().join("generated/src/lib.rs")).unwrap(),
        b"changed"
    );
    // Generation replaces only bytes it recorded (beyond10x/ess#484): an edited owned file
    // refuses, naming the file and the re-enroll route, and nothing is written. Moved aside, the
    // owned file is missing, and generation recreates it.
    let refused = generate(temp.path(), false);
    let stderr = String::from_utf8_lossy(&refused.stderr);
    assert!(
        !refused.status.success(),
        "an edited owned file was replaced"
    );
    for named in ["src/lib.rs", "ess generate output adopt --ownership-root"] {
        assert!(
            stderr.contains(named),
            "the refusal names {named}: {stderr}"
        );
    }
    assert_eq!(
        fs::read(temp.path().join("generated/src/lib.rs")).unwrap(),
        b"changed"
    );
    fs::rename(
        temp.path().join("generated/src/lib.rs"),
        temp.path().join("lib.rs.aside"),
    )
    .unwrap();
    assert!(generate(temp.path(), false).status.success());
    assert_eq!(
        fs::read(temp.path().join("generated/src/lib.rs")).unwrap(),
        before
    );
    fs::write(
        temp.path().join("generated/authored.txt"),
        "authored neighbour",
    )
    .unwrap();
    assert!(generate(temp.path(), false).status.success());
    assert_eq!(
        fs::read(temp.path().join("generated/authored.txt")).unwrap(),
        b"authored neighbour"
    );
}

#[test]
fn invalid_binding_leaves_both_absent_and_existing_destinations_untouched() {
    let temp = fixture();
    fs::write(
        temp.path().join("cli.yaml"),
        BINDING.replace("ess-cli/1", "ess-cli/999"),
    )
    .unwrap();
    assert!(!generate(temp.path(), false).status.success());
    assert!(!temp.path().join("generated").exists());
    fs::create_dir(temp.path().join("generated")).unwrap();
    fs::write(temp.path().join("generated/keep.txt"), "authored").unwrap();
    assert!(!generate(temp.path(), false).status.success());
    assert_eq!(
        fs::read_dir(temp.path().join("generated")).unwrap().count(),
        1
    );
    assert_eq!(
        fs::read(temp.path().join("generated/keep.txt")).unwrap(),
        b"authored"
    );
}

#[test]
fn generating_within_model_input_is_refused_without_mutation() {
    let temp = fixture();
    fs::create_dir(temp.path().join("model")).unwrap();
    fs::rename(
        temp.path().join("model.yaml"),
        temp.path().join("model/system.yaml"),
    )
    .unwrap();
    let output = ess(
        temp.path(),
        &[
            "generate",
            "cli",
            "--path",
            "model",
            "--binding",
            "cli.yaml",
            "--out",
            "model/generated",
        ],
    );
    assert!(!output.status.success());
    assert!(!temp.path().join("model/generated").exists());
}

/// beyond10x/ess#481: the issue's state-only binding, as `ess-cli/2`.
const STATE_ONLY: &str = "format: ess-cli/2\nbinary: demo\nabout: The demo records\nglobals:\n  state: state-dir\ncallables:\n  show:\n    target: {kind: local, owner: demo.cli, action: show}\n    input: null\n    result: demo.Stored\ncommands:\n  - path: [show]\n    callable: show\n    about: Show\n    arguments: []\n";

fn specify(root: &Path) -> Output {
    ess(
        root,
        &[
            "specify",
            "cli",
            "--path",
            "model.yaml",
            "--binding",
            "cli.yaml",
            "--format",
            "json",
        ],
    )
}

#[test]
fn a_state_only_ess_cli_2_binding_validates_and_generates_without_the_absent_flags() {
    let temp = fixture();
    fs::write(temp.path().join("cli.yaml"), STATE_ONLY).unwrap();
    let valid = specify(temp.path());
    assert!(
        valid.status.success(),
        "{}",
        String::from_utf8_lossy(&valid.stderr)
    );
    let plan: serde_json::Value = serde_json::from_slice(&valid.stdout).unwrap();
    assert_eq!(plan["format"], "ess-cli-plan/2");
    assert_eq!(plan["globals"], serde_json::json!({"state": "state-dir"}));
    let generated = generate(temp.path(), false);
    assert!(
        generated.status.success(),
        "{}",
        String::from_utf8_lossy(&generated.stderr)
    );
    let help = fs::read_to_string(temp.path().join("generated/help.txt")).unwrap();
    assert!(help.contains("--state-dir"), "{help}");
    for absent in ["--config", "--output"] {
        assert!(!help.contains(absent), "help lists {absent}: {help}");
    }
    assert!(generate(temp.path(), true).status.success());
}

#[test]
fn a_null_or_empty_global_is_refused_naming_it_and_ess_cli_1_names_the_version_that_omits_it() {
    let temp = fixture();
    for (format, spelling) in [
        ("ess-cli/1", "null"),
        ("ess-cli/1", "~"),
        ("ess-cli/1", "\"\""),
        ("ess-cli/2", "null"),
        ("ess-cli/2", "~"),
        ("ess-cli/2", "\"\""),
    ] {
        let text = BINDING
            .replace("ess-cli/1", format)
            .replace("{config: config,", &format!("{{config: {spelling},"));
        fs::write(temp.path().join("cli.yaml"), text).unwrap();
        let refused = specify(temp.path());
        let stderr = String::from_utf8_lossy(&refused.stderr);
        assert!(
            !refused.status.success(),
            "{format} admitted `config: {spelling}`"
        );
        assert!(
            stderr.contains("globals.config"),
            "{format} `config: {spelling}`: {stderr}"
        );
    }
    fs::write(
        temp.path().join("cli.yaml"),
        STATE_ONLY.replace("ess-cli/2", "ess-cli/1"),
    )
    .unwrap();
    let refused = specify(temp.path());
    let stderr = String::from_utf8_lossy(&refused.stderr);
    assert!(!refused.status.success(), "ess-cli/1 admitted state alone");
    for named in ["`config`", "ess-cli/2"] {
        assert!(
            stderr.contains(named),
            "the refusal names {named}: {stderr}"
        );
    }
}

#[cfg(unix)]
#[test]
fn drift_check_refuses_linked_generated_files() {
    let temp = fixture();
    assert!(generate(temp.path(), false).status.success());
    let output = temp.path().join("generated/src/lib.rs");
    let outside = temp.path().join("outside.rs");
    fs::rename(&output, &outside).unwrap();
    std::os::unix::fs::symlink(outside, output).unwrap();
    assert!(!generate(temp.path(), true).status.success());
}
