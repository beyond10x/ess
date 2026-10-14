//! A read-only command's `unknown_instance:` answer (beyond10x/ess#535), executed against the
//! generated Rust and Go targets.
//!
//! The issue's `Ring` — a `when_subject:` refusal and a `preserves:` branch addressing the seat its
//! input names — declares `no-seat` for an identity no seat carries. The model, served by one
//! network component, is emitted, built as its generated entry point with no hand-written code, and
//! driven over a real socket by the suite the same model synthesizes, which sends `Ring` an identity
//! nothing stores and expects `no-seat` with `demo.desk.NoSeat`.
//!
//! A faulty control answers the unknown identity with the preserving branch instead: the suite must
//! fail it.

#[allow(dead_code)]
mod related_guard_served;
use related_guard_served as served;

use ess_synth::{CapabilityKind, SynthesisDisposition, Target};

const READ_ONLY: &str =
    include_str!("../../../specify/ess-compiler/tests/fixtures/read-only-unknown-instance.yaml");

fn model() -> ess_compiler::ir::EssIr {
    served::model(&served::served(READ_ONLY, "desk"))
}

/// The first `no-seat` answer of `Ring`'s generated behaviour — where the addressed row is read and
/// found absent — and the preserving answer the faulty control puts in its place.
fn unknown_answer(target: Target) -> (&'static str, &'static str) {
    match target {
        Target::Rust => (
            "return Ok(crate::desk::RingOutcome::NoSeat { error: crate::desk::NoSeat });",
            "return Ok(crate::desk::RingOutcome::Rung);",
        ),
        Target::Go => (
            "return desk.RingOutcomeNoSeat{Error: desk.NoSeat{}}, nil",
            "return desk.RingOutcomeRung{}, nil",
        ),
        _ => unreachable!("the read-only case emits Rust and Go"),
    }
}

fn read_only_served(target: Target) {
    let ir = model();
    let synthesis = ess_synth::synthesize_for(&ir, target).expect("the model synthesizes");
    assert_eq!(
        synthesis
            .plan
            .disposition_of(CapabilityKind::CommandBehavior, "demo.desk.Ring"),
        Some(&SynthesisDisposition::Generated),
        "{target:?}:\n{}",
        synthesis.plan.to_markdown()
    );
    let (root, _) = served::emit(&ir, target, "read-only-unknown");
    let binary = served::build(&root, target, "demo", "desk", false);
    let statuses = served::run(&ir, &binary);
    served::assert_passes(
        &format!("{target:?} read-only"),
        &statuses,
        &[
            "Ring/outcome/no-seat",
            "Ring/outcome/closed",
            "Ring/outcome/rung",
        ],
    );
    let _ = std::fs::remove_dir_all(&root);

    // Faulty control: an unknown identity answered as if the seat were there.
    let (root, _) = served::emit(&ir, target, "read-only-unknown-faulty");
    let (from, to) = unknown_answer(target);
    served::mutate(&served::behaviour_path(&root, target, "demo"), from, to);
    let binary = served::build(&root, target, "demo", "desk", false);
    let statuses = served::run(&ir, &binary);
    assert_ne!(
        statuses.get("demo.desk.Ring/outcome/no-seat"),
        Some(&ess_conformance::report::Status::Passed),
        "{target:?}: the faulty target fails the unknown-identity scenario: {statuses:#?}"
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn a_read_only_command_answers_an_unknown_identity_in_the_generated_rust_target() {
    read_only_served(Target::Rust);
}

#[test]
fn a_read_only_command_answers_an_unknown_identity_in_the_generated_go_target() {
    assert!(
        served::go_available(),
        "Go is required: the generated Go target is unchecked without it"
    );
    read_only_served(Target::Go);
}
