//! `ess specify compose` over an `ess-composition/4` document with `roles:`
//! (<https://github.com/beyond10x/ess/issues/525>, `docs/design/composition-contract-roles.md`).
//!
//! The library checks the roles (`crates/specify/ess-composition/tests/roles.rs`); this holds the
//! command to reading `/4` through it, writing the roles into the compiled composition and
//! printing a role refusal by its code.

use ess_cli::TemporaryDirectory;
use std::{
    fs,
    path::PathBuf,
    process::{Command, Output},
};

fn fixture(system: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../specify/ess-composition/tests/fixtures/contract-role")
        .join(system)
}

fn ess(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ess"))
        .args(args)
        .output()
        .unwrap()
}

fn text(output: &Output) -> String {
    format!(
        "status {:?}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

/// The compiled-model digest `ess verify diff` reports for one fixture system.
fn digest(system: &str) -> String {
    let path = fixture(system);
    let path = path.to_str().unwrap();
    let output = ess(&[
        "verify", "diff", "--from", path, "--to", path, "--format", "json",
    ]);
    assert!(output.status.success(), "{}", text(&output));
    let diff: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    diff["after"]["spec_digest"]
        .as_str()
        .unwrap_or_else(|| panic!("no digest in {diff}"))
        .to_owned()
}

/// Composes `roles` under `/4` over the issue's two systems and returns the command's output.
fn compose(directory: &TemporaryDirectory, roles: &str) -> Output {
    let document = directory.join("composition.yaml");
    fs::write(
        &document,
        format!(
            "format: ess-composition/4\n\
             composition: demo-wire\n\
             services:\n  \
               - {{key: demo, system: demo, version: v1, source_digest: {}, component: relay}}\n  \
               - {{key: wire, system: wire, version: v1, source_digest: {}, component: wire-server}}\n\
             {roles}",
            digest("demo"),
            digest("wire"),
        ),
    )
    .unwrap();
    let demo = format!("demo={}", fixture("demo").display());
    let wire = format!("wire={}", fixture("wire").display());
    ess(&[
        "specify",
        "compose",
        "--path",
        document.to_str().unwrap(),
        "--service",
        &demo,
        "--service",
        &wire,
        "--format",
        "json",
    ])
}

#[test]
fn compose_writes_a_checked_contract_role_into_the_composition() {
    let directory = TemporaryDirectory::create("ess-compose-roles").unwrap();
    let output = compose(
        &directory,
        "roles:\n  \
           - service: demo\n    \
             plays: wire\n    \
             commands:\n      \
               - {local: demo.gate.Admit, stands: gates, contract: wire.server.Subscribe}\n",
    );
    assert!(output.status.success(), "{}", text(&output));
    let ir: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(ir["format"], "ess-composition/4");
    let role = &ir["roles"][0];
    assert_eq!(role["service"]["component"], "relay");
    assert_eq!(role["service"]["source_digest"], digest("demo").as_str());
    assert_eq!(role["plays"]["component"], "wire-server");
    assert_eq!(role["plays"]["source_digest"], digest("wire").as_str());
    assert_eq!(
        role["commands"],
        serde_json::json!([
            {"local": "demo.gate.Admit", "stands": "gates", "contract": "wire.server.Subscribe"}
        ])
    );
}

#[test]
fn compose_refuses_an_implements_entry_whose_inputs_drift() {
    let directory = TemporaryDirectory::create("ess-compose-roles-drift").unwrap();
    let output = compose(
        &directory,
        "roles:\n  \
           - service: demo\n    \
             plays: wire\n    \
             commands:\n      \
               - {local: demo.gate.Count, stands: implements, contract: wire.server.Subscribe}\n",
    );
    assert_eq!(output.status.code(), Some(1), "{}", text(&output));
    let diagnostics: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(diagnostics[0]["code"], "role_input_drift", "{diagnostics}");
    let detail = diagnostics[0]["detail"].as_str().unwrap();
    assert!(detail.starts_with("roles[0].commands[0]"), "{detail}");
    assert!(detail.contains("field `channel`"), "{detail}");
}
