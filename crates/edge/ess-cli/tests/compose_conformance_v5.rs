//! `ess specify compose` over an `ess-composition/5` document whose conformances name an event, or
//! compare value constraints (`docs/design/composition-type-conformance.md`).
//!
//! The library compares the ends (`crates/specify/ess-composition/tests/payload_conformance.rs`);
//! this holds the command to reading `/5` through it, writing the payload ends into the compiled
//! composition and printing a drift by its code.

use ess_cli::TemporaryDirectory;
use std::{
    fs,
    path::PathBuf,
    process::{Command, Output},
};

fn fixture(set: &str, system: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../specify/ess-composition/tests/fixtures")
        .join(set)
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
fn digest(set: &str, system: &str) -> String {
    let path = fixture(set, system);
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

/// Composes `rest` under `format` over the fixture set's two systems.
fn compose(directory: &TemporaryDirectory, set: &str, format: &str, rest: &str) -> Output {
    let document = directory.join("composition.yaml");
    fs::write(
        &document,
        format!(
            "format: {format}\n\
             composition: demo-wire\n\
             services:\n  \
               - {{key: demo, system: demo, version: v1, source_digest: {}, component: relay}}\n  \
               - {{key: wire, system: wire, version: v1, source_digest: {}, component: wire-server}}\n\
             {rest}",
            digest(set, "demo"),
            digest(set, "wire"),
        ),
    )
    .unwrap();
    let demo = format!("demo={}", fixture(set, "demo").display());
    let wire = format!("wire={}", fixture(set, "wire").display());
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
fn compose_reads_an_event_conformance() {
    let directory = TemporaryDirectory::create("ess-compose-v5-event").unwrap();
    let output = compose(
        &directory,
        "payload-conformance",
        "ess-composition/5",
        "conformances:\n  \
           - local: {service: demo, event: demo.gate.Admitted}\n    \
             conforms_to: {service: wire, event: wire.server.Subscribed}\n",
    );
    assert!(output.status.success(), "{}", text(&output));
    let ir: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(ir["format"], "ess-composition/5");
    assert_eq!(
        ir["conformances"],
        serde_json::json!([{
            "local": {"service": "demo", "event": "demo.gate.Admitted"},
            "conforms_to": {"service": "wire", "event": "wire.server.Subscribed"},
        }])
    );

    let drift = compose(
        &directory,
        "payload-conformance",
        "ess-composition/5",
        "conformances:\n  \
           - local: {service: demo, event: demo.gate.Counted}\n    \
             conforms_to: {service: wire, event: wire.server.Subscribed}\n",
    );
    assert_eq!(drift.status.code(), Some(1), "{}", text(&drift));
    let all = text(&drift);
    assert!(all.contains("type_conformance_drift"), "{all}");
    assert!(all.contains("field `channel`"), "{all}");
}

#[test]
fn compose_exits_non_zero_on_an_invariant_drift_under_v5() {
    let directory = TemporaryDirectory::create("ess-compose-v5-invariant").unwrap();
    let rest = "conformances:\n  \
                - local: {service: demo, type: demo.gate.Code}\n    \
                  conforms_to: {service: wire, type: wire.server.Code}\n";
    let drift = compose(
        &directory,
        "constraint-conformance",
        "ess-composition/5",
        rest,
    );
    assert_eq!(drift.status.code(), Some(1), "{}", text(&drift));
    let all = text(&drift);
    assert!(all.contains("type_conformance_drift"), "{all}");
    assert!(all.contains("invariant"), "{all}");
    // `/4` compares no value constraint, so the same document composes.
    let four = compose(
        &directory,
        "constraint-conformance",
        "ess-composition/4",
        rest,
    );
    assert!(four.status.success(), "{}", text(&four));
}
