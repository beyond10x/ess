//! `unknown_instance:` beside a `when_related:` guard (source format `ess/27`, downstream report,
//! 2026-10-18), executed against the generated Rust and Go targets.
//!
//! `ChangeTier` addresses a seat by its input and reads the tier its input names: an unknown tier
//! answers first, then an unknown seat, then a closed tier, then the change. The model, served by
//! one network component, is emitted, built as its generated entry point with no hand-written code,
//! and driven over a real socket by the suite the same model synthesizes, whose `unknown-seat`
//! scenario names a closed tier.
//!
//! A faulty control skips the seat's existence check made before the tier's refusal, so it reads
//! the closed tier first: the suite must fail it.

#[allow(dead_code)]
mod related_guard_served;
use related_guard_served as served;

use ess_synth::{CapabilityKind, SynthesisDisposition, Target};

const MODEL: &str = include_str!(
    "../../../verify/ess-conformance/tests/fixtures/related-guard-unknown-instance.yaml"
);

fn model() -> ess_compiler::ir::EssIr {
    served::model(&served::served(MODEL, "seats"))
}

/// The existence check of `ChangeTier`'s generated behaviour made before the tier's refusal, and a
/// replacement that always finds a row.
fn existence_check(target: Target) -> (&'static str, &'static str) {
    match target {
        Target::Rust => (
            "let Some(held) = SeatStorage::get(&self.ports, &input.seat_id) else {",
            "let Some(held) = Some(()) else {",
        ),
        Target::Go => (
            "held, found := b.ports.SeatStorage.Get(input.SeatId)",
            "held, found := 0, true",
        ),
        _ => unreachable!("the case emits Rust and Go"),
    }
}

fn unknown_instance_served(target: Target) {
    let ir = model();
    let synthesis = ess_synth::synthesize_for(&ir, target).expect("the model synthesizes");
    assert_eq!(
        synthesis
            .plan
            .disposition_of(CapabilityKind::CommandBehavior, "demo.seats.ChangeTier"),
        Some(&SynthesisDisposition::Generated),
        "{target:?}:\n{}",
        synthesis.plan.to_markdown()
    );
    let (root, _) = served::emit(&ir, target, "unknown-instance");
    let binary = served::build(&root, target, "demo", "seats", false);
    let statuses = served::run(&ir, &binary);
    served::assert_passes(
        &format!("{target:?} unknown instance beside a related guard"),
        &statuses,
        &[
            "ChangeTier/outcome/unknown-seat",
            "ChangeTier/outcome/unknown-tier",
            "ChangeTier/outcome/tier-closed",
            "ChangeTier/outcome/changed",
        ],
    );
    let _ = std::fs::remove_dir_all(&root);

    // Faulty control: the closed tier read before the seat's existence.
    let (root, _) = served::emit(&ir, target, "unknown-instance-faulty");
    let (from, to) = existence_check(target);
    served::mutate(&served::behaviour_path(&root, target, "demo"), from, to);
    let binary = served::build(&root, target, "demo", "seats", false);
    let statuses = served::run(&ir, &binary);
    assert_ne!(
        statuses.get("demo.seats.ChangeTier/outcome/unknown-seat"),
        Some(&ess_conformance::report::Status::Passed),
        "{target:?}: the faulty target fails the unknown-seat scenario: {statuses:#?}"
    );
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn unknown_instance_beside_a_related_guard_in_the_generated_rust_target() {
    unknown_instance_served(Target::Rust);
}

#[test]
fn unknown_instance_beside_a_related_guard_in_the_generated_go_target() {
    assert!(
        served::go_available(),
        "Go is required: the generated Go target is unchecked without it"
    );
    unknown_instance_served(Target::Go);
}
