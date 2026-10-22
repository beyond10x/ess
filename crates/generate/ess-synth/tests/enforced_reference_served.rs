//! A command reading a stored reference with no `exists: false` branch, the row's presence derived
//! from the specification (`ess/30`, beyond10x/ess#509), executed against the generated Rust and
//! Go targets.
//!
//! `Rename` reads the node its port's stored `node_id` names: `AddPort` refuses a missing node
//! before storing one, and no command removes a node, so `Rename` declares no missing-node branch.
//! The model, served by one network component, is emitted, built as its generated entry point with
//! no hand-written code, and driven over a real socket by the suite the same model synthesizes.
//!
//! A faulty control reads no node for the port, so a sealed node is never seen: the suite must fail
//! it.

#[allow(dead_code)]
mod related_guard_served;
use related_guard_served as served;

use ess_synth::{CapabilityKind, SynthesisDisposition, Target};

const BOARD: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/enforced-reference.yaml");

/// `BOARD` with an `unknown_instance:` answer on `Rename`, which the generated behaviour needs for
/// a port no row carries. Synthesis arranges no unknown port for a command reading a stored
/// reference (`ESS-SYNTH-008`), so no scenario witnesses that answer here.
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

/// The edit that makes `Rename`'s generated behaviour read no node for the port.
fn reads_no_node(target: Target) -> (&'static str, &'static str) {
    match target {
        Target::Rust => (
            "let reference = Some(&held.data.node_id);",
            "let reference = Some(&held.data.node_id).filter(|_| false);",
        ),
        Target::Go => (
            "reference := &held.Data.NodeId\n\tif reference != nil {",
            "reference := &held.Data.NodeId\n\tif reference != nil && false {",
        ),
        _ => unreachable!("the case emits Rust and Go"),
    }
}

fn enforced_reference_served(target: Target) {
    let ir = served::model(&served::served(&answered(), "board"));
    let synthesis = ess_synth::synthesize_for(&ir, target).expect("the model synthesizes");
    for command in ["integrity.board.Rename", "integrity.board.AddPort"] {
        assert_eq!(
            synthesis
                .plan
                .disposition_of(CapabilityKind::CommandBehavior, command),
            Some(&SynthesisDisposition::Generated),
            "{target:?} {command}:\n{}",
            synthesis.plan.to_markdown()
        );
    }
    let (root, _) = served::emit(&ir, target, "enforced-reference");
    let binary = served::build(&root, target, "integrity", "board", false);
    let statuses = served::run(&ir, &binary);
    served::assert_passes(
        &format!("{target:?} a stored reference the specification keeps present"),
        &statuses,
        &[
            "Rename/outcome/node-sealed",
            "Rename/outcome/renamed",
            "Port/invariant/after/integrity.board.Rename/renamed",
        ],
    );
    let _ = std::fs::remove_dir_all(&root);

    // Faulty control: the port's node is never read, so a sealed node is renamed past.
    let (root, _) = served::emit(&ir, target, "enforced-reference-faulty");
    let (from, to) = reads_no_node(target);
    served::mutate(
        &served::behaviour_path(&root, target, "integrity"),
        from,
        to,
    );
    let binary = served::build(&root, target, "integrity", "board", false);
    let statuses = served::run(&ir, &binary);
    assert_ne!(
        statuses.get("integrity.board.Rename/outcome/node-sealed"),
        Some(&ess_conformance::report::Status::Passed),
        "{target:?}: the faulty target fails the node-sealed scenario: {statuses:#?}"
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn a_derived_reference_is_read_by_the_generated_rust_target() {
    enforced_reference_served(Target::Rust);
}

#[test]
fn a_derived_reference_is_read_by_the_generated_go_target() {
    assert!(
        served::go_available(),
        "Go is required: the generated Go target is unchecked without it"
    );
    enforced_reference_served(Target::Go);
}
