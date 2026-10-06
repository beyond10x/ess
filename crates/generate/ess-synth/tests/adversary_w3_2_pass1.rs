//! Adversary pass 1 on unit W3-2 (beyond10x/ess#445): the typed binding constants the generated
//! Rust and Go adapters now pass are held to the compilers, not only to the text that spells them.
//! The unit's own case (`binding_literal_scalar.rs`) asserts substrings of the emission.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_synth::{synthesize_for, Synthesis, Target};

const CALLS: &str = "format: ess/22
system: demo
version: v1
domain: demo.calls
types:
  - {name: demo.calls.Weight, kind: newtype, of: Integer}
  - {name: demo.calls.Bridged, kind: newtype, of: Boolean}
  - {name: demo.calls.Ratio, kind: newtype, of: Decimal}
  - {name: demo.calls.TemplateId, kind: newtype, of: String}
events:
  - name: demo.calls.LegJoined
    fields:
      - {name: leg_id, type: String}
  - name: demo.calls.LegRecorded
    fields:
      - {name: leg_id, type: String}
commands:
  - name: demo.calls.JoinLeg
    input:
      - {name: leg_id, type: String}
    outcomes:
      - name: joined
        emits: [demo.calls.LegJoined]
        payload:
          demo.calls.LegJoined: {leg_id: input.leg_id}
  - name: demo.calls.RecordLeg
    input:
      - {name: leg_id, type: String}
      - {name: is_bridged, type: Boolean}
      - {name: bridged, type: demo.calls.Bridged}
      - {name: weight, type: demo.calls.Weight}
      - {name: count, type: Integer}
      - {name: share, type: Decimal}
      - {name: ratio, type: demo.calls.Ratio}
      - {name: template, type: demo.calls.TemplateId}
    outcomes:
      - name: recorded
        emits: [demo.calls.LegRecorded]
        payload:
          demo.calls.LegRecorded: {leg_id: input.leg_id}
components:
  - component: calls-service
    summary: Records the legs of a call.
    owns:
      domains: [demo.calls]
    accepts:
      commands: [demo.calls.JoinLeg, demo.calls.RecordLeg]
    publishes:
      events: [demo.calls.LegJoined, demo.calls.LegRecorded]
bindings:
  - id: joined
    when: {event: demo.calls.LegJoined}
    invoke: {command: demo.calls.RecordLeg}
    mapping:
      leg_id: event.leg_id
      is_bridged: true
      bridged: false
      weight: 3
      count: -3
      share: 0.5
      ratio: 1
      template: invoice-created
    delivery: at_least_once
    on_failure: drop
";

fn ir() -> EssIr {
    let raw = RawSpecFile::parse(CALLS).unwrap();
    let spec = Specification::assemble([(Source::new("calls.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap()
}

fn scratch(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("adversary-w3-2-{name}-{}", std::process::id()))
}

fn write_tree(directory: &Path, synthesis: &Synthesis) {
    let _ = std::fs::remove_dir_all(directory);
    for artifact in synthesis.artifacts.values() {
        let path = directory.join(&artifact.path);
        std::fs::create_dir_all(path.parent().expect("a file has a parent")).expect("mkdir");
        std::fs::write(&path, &artifact.contents).expect("write");
    }
}

fn shown(output: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

#[test]
fn adv_w3_2_typed_binding_constants_compile_in_rust_and_go() {
    let ir = ir();
    let root = scratch("typed-constants");
    let _ = std::fs::remove_dir_all(&root);

    let rust = synthesize_for(&ir, Target::Rust).expect("the model synthesizes to Rust");
    let tree = root.join("rust");
    write_tree(&tree, &rust);
    let emitted: String = rust
        .artifacts
        .values()
        .map(|artifact| artifact.contents.clone())
        .collect();
    assert!(
        emitted.contains("count: -3,"),
        "the Rust adapter passes the constant"
    );
    let output = Command::new(std::env::var_os("CARGO").expect("Cargo supplies its executable"))
        .args(["check", "--offline", "--workspace", "--target-dir"])
        .arg(root.join("target"))
        .current_dir(&tree)
        .env_remove("CARGO_TARGET_DIR")
        .env_remove("CARGO_ENCODED_RUSTFLAGS")
        .env("RUSTC_WRAPPER", "")
        .env("CARGO_BUILD_RUSTC_WRAPPER", "")
        .env("RUSTC_WORKSPACE_WRAPPER", "")
        .env("CARGO_INCREMENTAL", "0")
        .output()
        .expect("cargo runs");
    let _ = std::fs::remove_dir_all(root.join("target"));
    assert!(
        output.status.success(),
        "the generated Rust workspace compiles: {}",
        shown(&output)
    );
    assert!(
        shown(&output).contains("Checking"),
        "cargo checked a crate: {}",
        shown(&output)
    );

    if Command::new("go")
        .arg("version")
        .output()
        .is_ok_and(|output| output.status.success())
    {
        let go = synthesize_for(&ir, Target::Go).expect("the model synthesizes to Go");
        let tree = root.join("go");
        write_tree(&tree, &go);
        let emitted: String = go
            .artifacts
            .values()
            .map(|artifact| artifact.contents.clone())
            .collect();
        assert!(
            emitted.contains("int64(-3)"),
            "the Go adapter passes the constant"
        );
        let output = Command::new("go")
            .args(["vet", "./..."])
            .current_dir(&tree)
            .env("GOFLAGS", "-mod=mod")
            .env("GOPROXY", "off")
            .env("GOWORK", "off")
            .env("CGO_ENABLED", "0")
            .output()
            .expect("go runs");
        assert!(
            output.status.success(),
            "the generated Go module compiles: {}",
            shown(&output)
        );
    }
    let _ = std::fs::remove_dir_all(&root);
}
