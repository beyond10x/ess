//! Adversary, ESS wave D357 unit U2, pass 1.
//!
//! `story:concurrent-history-records-inputs`: the same history in `ess-history/3` with its input
//! removed still reads and is `Linearizable`.
//!
//! The pass's scanner cases for `story:a-marked-region-is-not-a-scan-of-what-runs` live beside the
//! scan they hold, as the `the_startup_scan_*` cases in `coverage_browser.rs`, and its close-frame
//! case in `browser_startup_refusal_boundary.rs`, so this binary includes no browser fixture.

use std::process::{Command, Output};

/// `story:concurrent-history-records-inputs`: "the same history without inputs still reads". The
/// unit's case drops to `ess-history/1` for that half; this keeps `ess-history/3` and removes only
/// the input, so the `Violation` the unit pins is shown to come from the input and not the format.
#[test]
fn adversary_the_same_format_3_history_without_its_input_reads_and_is_linearizable() {
    let directory = tempfile::tempdir().unwrap();
    let source = r"
format: ess/18
system: pay
version: v1
domain: pay.core
errors:
  - {name: pay.core.InvalidAmount, summary: The amount is not positive.}
events:
  - {name: pay.core.Charged, fields: [{name: amount, type: Integer}]}
commands:
  - name: pay.core.Charge
    input: [{name: amount, type: Integer}]
    outcomes:
      - name: accepted
        when: amount >= 1
        emits: [pay.core.Charged]
        payload: {pay.core.Charged: {amount: input.amount}}
      - name: rejected
        error: pay.core.InvalidAmount
";
    let specification = ess_domain::spec::Specification::assemble([(
        ess_domain::system::Source::new("pay.yaml"),
        ess_domain::spec::RawSpecFile::parse(source).unwrap(),
    )])
    .unwrap();
    let ir =
        ess_compiler::resolve::compile(&specification, &ess_compiler::source::SourceMap::new())
            .unwrap();
    let digest = ess_conformance::scenario::SuiteProvenance::of(&ir).spec_digest;
    let path = directory.path().join("pay.yaml");
    std::fs::write(&path, source).unwrap();
    let operation = serde_json::json!({
        "operation_id":"00000000-0000-4000-8000-000000000002", "client":0,
        "command":"pay.core.Charge", "subject_key":"",
        "invoked_at":1, "returned_at":2, "completion":"Returned", "outcome":"accepted"
    });
    let document = serde_json::json!({
        "format":"ess-history/3", "history_id":"00000000-0000-4000-8000-000000000001",
        "spec_digest":digest, "seed":1, "clients":1, "operations":[operation]
    });
    let recorded = directory.path().join("no-input-format-3.json");
    std::fs::write(&recorded, serde_json::to_vec(&document).unwrap()).unwrap();
    let output: Output = Command::new(env!("CARGO_BIN_EXE_ess"))
        .args(["verify", "conform", "check-history", "--path"])
        .arg(&path)
        .arg("--history")
        .arg(&recorded)
        .output()
        .expect("the `ess` binary runs");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(
        output.status.code(),
        Some(0),
        "story:concurrent-history-records-inputs: ess-history/3 without the input: {stdout}{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        stdout.starts_with("verdict: Linearizable\n"),
        "story:concurrent-history-records-inputs: {stdout}"
    );
}
