//! A binding constant over a `Boolean`, `Integer` or `Decimal` input (beyond10x/ess#445) is a
//! determined input: the generated Rust and Go binding adapters pass it as a typed constant, and
//! the plan raises no "is filled from the literal" obligation for it.

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_synth::{synthesize_for, Target};

const CALLS: &str = "format: ess/22
system: demo
version: v1
domain: demo.calls
types:
  - {name: demo.calls.Weight, kind: newtype, of: Integer}
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
      - {name: weight, type: demo.calls.Weight}
      - {name: share, type: Decimal}
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
      weight: 3
      share: 0.5
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

/// Every artifact a target writes, joined.
fn code(ir: &EssIr, target: Target) -> String {
    synthesize_for(ir, target)
        .unwrap_or_else(|failure| panic!("{target:?} generates the binding: {failure:?}"))
        .artifacts
        .values()
        .map(|artifact| artifact.contents.clone())
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn binding_literal_generated_rust_go_pass_typed_constant() {
    let ir = ir();
    for target in [Target::Rust, Target::Go] {
        let all = code(&ir, target);
        assert!(
            !all.contains("is filled from the literal"),
            "{target:?}: no typed constant is owed: {all}"
        );
    }

    let rust = code(&ir, Target::Rust);
    for expected in [
        "is_bridged: true,",
        "weight: demo_types::calls::Weight(3),",
        "share: demo_types::primitives::Decimal(\"0.5\".to_owned()),",
        // A text literal keeps its bytes.
        "template: demo_types::calls::TemplateId(\"invoice-created\".to_owned()),",
    ] {
        assert!(
            rust.contains(expected),
            "Rust is missing `{expected}`:\n{rust}"
        );
    }
    for refused in ["\"true\"", "\"3\"", "is_bridged: \"true\""] {
        assert!(
            !rust.contains(&format!("is_bridged: {refused}")),
            "Rust sends text: {rust}"
        );
    }

    let go = code(&ir, Target::Go);
    for expected in [
        "IsBridged: true,",
        "Weight: calls.NewWeight(int64(3)),",
        "Share: primitives.NewDecimal(\"0.5\"),",
        "Template: calls.NewTemplateId(\"invoice-created\"),",
    ] {
        assert!(go.contains(expected), "Go is missing `{expected}`:\n{go}");
    }
    assert!(!go.contains("IsBridged: \"true\""), "Go sends text: {go}");
}
