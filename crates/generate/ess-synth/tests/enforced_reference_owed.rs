//! A command reading a stored reference with no `exists: false` branch, the row's presence derived
//! from the specification (`ess/29`, beyond10x/ess#509), stays owed by generated Rust and Go,
//! named by the plan: the generated read of the related row is bound by that branch.

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_synth::{synthesize_for, CapabilityKind, SynthesisDisposition, Target};

const BOARD: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/enforced-reference.yaml");

fn model(text: &str) -> EssIr {
    let spec = Specification::assemble([(
        Source::new("board.yaml"),
        RawSpecFile::parse(text).expect("well formed"),
    )])
    .unwrap_or_else(|errors| panic!("validates: {errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|errors| panic!("compiles: {errors}"))
}

/// `BOARD` with an `unknown_instance:` answer on `Rename`, without which the command is owed for
/// that first.
fn answered() -> String {
    let body = BOARD
        .replacen(
            "  - {name: integrity.board.NodeSealed,",
            "  - {name: integrity.board.PortNotFound, summary: No port carries this identity., \
             fields: []}\n  - {name: integrity.board.NodeSealed,",
            1,
        )
        .replacen(
            "      - name: node-sealed\n",
            "      - {name: unknown-port, unknown_instance: true, error: \
             integrity.board.PortNotFound}\n      - name: node-sealed\n",
            1,
        );
    assert_eq!(body.matches("PortNotFound").count(), 2, "both edits apply");
    body
}

#[test]
fn generated_behaviour_names_the_derived_reference_owed() {
    let ir = model(&answered());
    for target in [Target::Rust, Target::Go] {
        let synthesis = synthesize_for(&ir, target).expect("a realizable target");
        let reason = match synthesis
            .plan
            .disposition_of(CapabilityKind::CommandBehavior, "integrity.board.Rename")
        {
            Some(SynthesisDisposition::Obligation(owed)) => owed.reason.describes(),
            other => panic!("{target:?}: `Rename` is owed, not {other:?}"),
        };
        assert!(
            reason.contains("no `exists: false` branch") && reason.contains("ess/29"),
            "{target:?}: the obligation names the construct: {reason}"
        );
        // The writer that checks the row is still generated.
        assert_eq!(
            synthesis
                .plan
                .disposition_of(CapabilityKind::CommandBehavior, "integrity.board.AddPort"),
            Some(&SynthesisDisposition::Generated),
            "{target:?}"
        );
    }
}
