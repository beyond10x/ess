//! The protocol sidecar is admitted explicitly, and never silently read as ordinary ESS.
use std::{
    fs,
    path::Path,
    process::{Command, Output},
};

const MODEL: &str = r"
format: ess-protospec/1
name: flush-session
bounds: {max_steps: 20, max_states: 100, max_time_ms: 1000}
participants:
  - name: client
    initial: ready
    states: [ready, done]
    inputs: [{name: finish, fields: []}]
    transitions:
      - name: finish
        from: ready
        to: done
        trigger: {kind: input, name: finish}
        effects:
          - {kind: flush, channel: responses}
          - {kind: close}
  - name: server
    initial: ready
    states: [ready]
    transitions: []
messages: []
channels:
  - {name: responses, from: client, to: server, ordering: fifo, capacity: 2, loss: false, duplication: false}
properties:
  - {kind: flush_before_close, name: response-before-close, participant: client, channel: responses}
";
fn ess(root: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ess"))
        .current_dir(root)
        .args(args)
        .output()
        .unwrap()
}
fn fixture() -> tempfile::TempDir {
    let temp = tempfile::tempdir().unwrap();
    fs::write(temp.path().join("protocol.yaml"), MODEL).unwrap();
    temp
}
#[test]
fn protocol_routes_admit_compile_and_refuse_ordinary_source_projection() {
    let temp = fixture();
    let valid = ess(
        temp.path(),
        &["specify", "protocol", "validate", "--path", "protocol.yaml"],
    );
    assert!(
        valid.status.success(),
        "{}",
        String::from_utf8_lossy(&valid.stderr)
    );
    assert!(String::from_utf8_lossy(&valid.stdout).contains("flush-session"));
    let compiled = ess(
        temp.path(),
        &["specify", "protocol", "compile", "--path", "protocol.yaml"],
    );
    assert!(
        compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    let json: serde_json::Value = serde_json::from_slice(&compiled.stdout).unwrap();
    assert_eq!(json["format"], "ess-protospec/1");
    assert_eq!(json["name"], "flush-session");
    assert!(!ess(
        temp.path(),
        &["specify", "validate", "--path", "protocol.yaml"]
    )
    .status
    .success());
}
#[test]
fn rejected_model_cannot_overwrite_an_output() {
    let temp = fixture();
    fs::write(temp.path().join("owned.json"), "user contents").unwrap();
    let out = ess(
        temp.path(),
        &[
            "specify",
            "protocol",
            "compile",
            "--path",
            "protocol.yaml",
            "--out",
            "owned.json",
        ],
    );
    assert!(!out.status.success());
    assert_eq!(
        fs::read_to_string(temp.path().join("owned.json")).unwrap(),
        "user contents"
    );
    fs::write(
        temp.path().join("protocol.yaml"),
        MODEL.replace("ess-protospec/1", "ess-protospec/999"),
    )
    .unwrap();
    let invalid = ess(
        temp.path(),
        &[
            "specify",
            "protocol",
            "compile",
            "--path",
            "protocol.yaml",
            "--out",
            "absent.json",
        ],
    );
    assert!(!invalid.status.success());
    assert!(!temp.path().join("absent.json").exists());
}

#[test]
fn cli_simulation_replay_and_mutation_have_distinct_exit_codes() {
    let temp = fixture();
    fs::write(
        temp.path().join("actions.json"),
        r#"[{"kind":"input","participant":"client","name":"finish","payload":{}}]"#,
    )
    .unwrap();
    let run = ess(
        temp.path(),
        &[
            "verify",
            "protocol",
            "run",
            "--path",
            "protocol.yaml",
            "--actions",
            "actions.json",
            "--out",
            "trace.json",
        ],
    );
    assert!(
        run.status.success(),
        "{}",
        String::from_utf8_lossy(&run.stderr)
    );
    assert!(String::from_utf8_lossy(&run.stderr).contains("model simulation only"));
    let replay = || {
        ess(
            temp.path(),
            &[
                "verify",
                "protocol",
                "replay",
                "--path",
                "protocol.yaml",
                "--trace",
                "trace.json",
            ],
        )
    };
    let good = replay();
    assert!(good.status.success());
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&good.stdout).unwrap()["verdict"],
        "passed"
    );
    let mut trace: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(temp.path().join("trace.json")).unwrap()).unwrap();
    trace["complete"] = false.into();
    fs::write(
        temp.path().join("trace.json"),
        serde_json::to_string(&trace).unwrap(),
    )
    .unwrap();
    assert_eq!(replay().status.code(), Some(2));
    trace["complete"] = true.into();
    trace["steps"][0]["observations"]
        .as_array_mut()
        .unwrap()
        .swap(0, 1);
    fs::write(
        temp.path().join("trace.json"),
        serde_json::to_string(&trace).unwrap(),
    )
    .unwrap();
    assert_eq!(replay().status.code(), Some(1));
    let explored = ess(
        temp.path(),
        &["verify", "protocol", "explore", "--path", "protocol.yaml"],
    );
    assert!(explored.status.success());
    let report: serde_json::Value = serde_json::from_slice(&explored.stdout).unwrap();
    assert_eq!(report["explored_transitions"], 1);
    assert_eq!(report["explored_states"], 2);
}
