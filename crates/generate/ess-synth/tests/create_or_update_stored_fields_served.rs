//! A create-or-update command that selects its update branches by stored fields (ess/25,
//! beyond10x/ess#530), executed against the generated Rust and Go targets.
//!
//! The issue's `SignIn` creates the seat for an identity no seat carries (`first`) and otherwise
//! moves it by the stored row: `Busy` with a call reference (`reopened-busy`), `Closed` (`opened`),
//! every other seat (`reopened`). The model, served by one network component, is emitted, built as
//! its generated entry point with no hand-written code, and driven over a real socket by the suite
//! the same model synthesizes. A faulty control per target edits the emitted behaviour so the
//! stored call reference is never read: the suite must fail it.

#[allow(dead_code)]
mod related_guard_served;
use related_guard_served as served;

use ess_synth::{CapabilityKind, SynthesisDisposition, Target};

const MODEL: &str = include_str!(
    "../../../specify/ess-compiler/tests/fixtures/create-or-update-stored-fields.yaml"
);

fn model() -> ess_compiler::ir::EssIr {
    served::model(&served::served(MODEL, "desk"))
}

fn create_or_update_served(target: Target) {
    let ir = model();
    let synthesis = ess_synth::synthesize_for(&ir, target).expect("the model synthesizes");
    assert_eq!(
        synthesis
            .plan
            .disposition_of(CapabilityKind::CommandBehavior, "demo.desk.SignIn"),
        Some(&SynthesisDisposition::Generated),
        "{target:?}:\n{}",
        synthesis.plan.to_markdown()
    );
    let (root, _) = served::emit(&ir, target, "create-or-update-stored");
    let binary = served::build(&root, target, "demo", "desk", false);
    let statuses = served::run(&ir, &binary);
    served::assert_passes(
        &format!("{target:?} create-or-update"),
        &statuses,
        &[
            "SignIn/outcome/first",
            "SignIn/outcome/reopened-busy",
            "SignIn/outcome/opened",
            "SignIn/outcome/reopened",
        ],
    );
    let _ = std::fs::remove_dir_all(&root);

    // Faulty control: the stored call reference read the other way round, so a busy seat with a
    // call takes `reopened` and one without takes `reopened-busy`.
    let (root, _) = served::emit(&ir, target, "create-or-update-stored-faulty");
    let (from, to) = call_ref_test(target);
    served::mutate(&served::behaviour_path(&root, target, "demo"), from, to);
    let binary = served::build(&root, target, "demo", "desk", false);
    let statuses = served::run(&ir, &binary);
    served::assert_fails(&format!("{target:?} create-or-update faulty"), &statuses);
    let _ = std::fs::remove_dir_all(&root);
}

/// The `call_ref != ""` test of `reopened-busy` in the generated behaviour, and the faulty
/// control's `call_ref == ""` in its place.
fn call_ref_test(target: Target) -> (&'static str, &'static str) {
    match target {
        Target::Rust => (
            "Some(\"\".to_owned())).map(|value| !value)]",
            "Some(\"\".to_owned()))]",
        ),
        Target::Go => (
            "not(equal(some(held.Data.CallRef), some(\"\")))",
            "equal(some(held.Data.CallRef), some(\"\"))",
        ),
        _ => unreachable!("the create-or-update case emits Rust and Go"),
    }
}

#[test]
fn create_or_update_stored_field_runs_alike_on_the_generated_rust_target() {
    create_or_update_served(Target::Rust);
}

#[test]
fn create_or_update_stored_field_runs_alike_on_the_generated_go_target() {
    assert!(
        served::go_available(),
        "Go is required: the generated Go target is unchecked without it"
    );
    create_or_update_served(Target::Go);
}

/// The WebAssembly and command-line targets wrap the Rust system: each generates `SignIn` as the
/// Rust target does, rather than owing it or refusing the model.
#[test]
fn create_or_update_stored_field_generates_on_the_web_and_clap_targets() {
    let ir = model();
    for target in [Target::Web, Target::Clap] {
        let synthesis = ess_synth::synthesize_for(&ir, target)
            .unwrap_or_else(|failure| panic!("{target:?}: {}", failure.to_canonical_json()));
        assert_eq!(
            synthesis
                .plan
                .disposition_of(CapabilityKind::CommandBehavior, "demo.desk.SignIn"),
            Some(&SynthesisDisposition::Generated),
            "{target:?}:\n{}",
            synthesis.plan.to_markdown()
        );
    }
}
