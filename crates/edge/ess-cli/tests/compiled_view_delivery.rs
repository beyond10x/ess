//! A domain's view names resolve to complete declarations in the CLI's compiled IR.

use std::process::Command;

#[test]
fn compiled_views_retain_the_wire_and_projection_contract_on_stdout_and_disk() {
    let dir = tempfile::tempdir().unwrap();
    let source = include_str!("fixtures/entity-setup-model.yaml").replace(
        "    consistency: read_your_writes",
        "    naming: {wire: call_history}\n    consistency: read_your_writes",
    );
    let path = dir.path().join("system.yaml");
    let out = dir.path().join("ir.json");
    std::fs::write(&path, source).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_ess"))
        .args(["specify", "compile", "--path"])
        .arg(&path)
        .args(["--format", "json", "--out"])
        .arg(&out)
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    assert_eq!(std::fs::read(out).unwrap(), output.stdout);
    let ir: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let name = "calls.history.CallHistory";
    assert_eq!(ir["domains"]["calls.history"]["views"][0], name);
    let view = &ir["views"][name];
    assert_eq!(view["name"], name);
    assert_eq!(view["source"], "calls.history.CallRecord");
    assert!(ir["entities"]["calls.history.CallRecord"].is_object());
    assert_eq!(view["naming"]["wire"], "call_history");
    assert_eq!(view["consistency"], "read_your_writes");
    let fields = view["fields"].as_array().unwrap();
    assert_eq!(fields.len(), 3);
    assert_eq!(fields[0]["name"], "call_id");
    assert_eq!(fields[1]["name"], "started_at");
    assert_eq!(fields[2]["name"], "duration_seconds");
    assert!(fields.iter().all(|field| field.get("type_ref").is_some()));
}
