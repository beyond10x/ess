//! A lifecycle state may declare its wire spelling, `states: [{name: Indexing, wire: indexing}]`,
//! from `ess/32` (beyond10x/ess#639). The state keeps its declared name everywhere a state is
//! named; the spelling lowers onto the variant of `<Entity>.State`. From `ess/32` no two states of
//! one lifecycle, and no two variants of one enum, share a wire spelling.

use super::{assemble, model, variants_of};
use ess_domain::{entity::StateName, TypeBody};
use ess_primitives::error::ValidationCode;

/// The job queue the projection tests share, its `Job` lifecycle written as `states` (a YAML flow
/// sequence) under `ess/<format>`.
const FIXTURE: &str = include_str!("../../../../generate/ess-gen/tests/fixtures/state-wire.yaml");

const SPELLED: &str = "[{name: Indexing, wire: indexing}, Ready]";

fn lifecycle(format: u32, states: &str) -> String {
    assert!(FIXTURE.contains("format: ess/32\n") && FIXTURE.contains(SPELLED));
    FIXTURE
        .replacen("format: ess/32\n", &format!("format: ess/{format}\n"), 1)
        .replacen(SPELLED, states, 1)
}

#[test]
fn a_lifecycle_state_declares_its_wire_name_from_ess_32() {
    let spec = assemble(&lifecycle(32, SPELLED)).expect("ess/32 admits a state's wire name");
    let job = &spec.entities()[&"jobs.queue.Job".parse().unwrap()];
    let indexing = StateName::new("Indexing").unwrap();
    assert!(
        job.states.states.contains(&indexing),
        "the state keeps its declared name: {:?}",
        job.states.states
    );
    assert_eq!(job.states.initial, indexing);
    let state_type = job.state_type();
    let TypeBody::Enum { variants } = &state_type.body else {
        panic!("the state type is an enum: {state_type:?}");
    };
    assert_eq!(
        variants
            .iter()
            .map(|variant| (variant.name(), variant.wire()))
            .collect::<Vec<_>>(),
        vec![("Indexing", "indexing"), ("Ready", "Ready")],
        "the spelling lowers onto the state enum's variant"
    );
}

#[test]
fn a_state_wire_name_below_ess_32_is_refused_naming_ess_32() {
    let errors = assemble(&lifecycle(31, SPELLED))
        .expect_err("a state's wire name is not admitted below ess/32");
    assert!(
        errors.contains(&ValidationCode::UnsupportedFormatVersion.to_string()),
        "{errors}"
    );
    assert!(errors.contains("ess/32"), "names the format: {errors}");
    assert!(
        errors.contains("jobs.queue.Job.states.Indexing"),
        "located at the state: {errors}"
    );
    assert!(
        !errors.contains("invalid type: map"),
        "no whole-file parse error: {errors}"
    );
}

#[test]
fn two_states_with_one_wire_spelling_are_refused() {
    for states in [
        "[{name: Indexing, wire: ready}, {name: Ready, wire: ready}]",
        "[{name: Indexing, wire: Ready}, Ready]",
    ] {
        let errors = assemble(&lifecycle(32, states))
            .expect_err("two states with one spelling cannot be told apart on the wire");
        assert!(
            errors.contains(&ValidationCode::ConflictingDeclaration.to_string()),
            "{states}: {errors}"
        );
        assert!(
            errors.contains("Indexing") && errors.contains("Ready"),
            "{states}: names both states: {errors}"
        );
    }
}

const SHARED: &str =
    "      - {name: Indexing, wire: indexing}\n      - {name: Ready, wire: indexing}\n";

#[test]
fn two_enum_variants_with_one_wire_spelling_are_refused_from_ess_32() {
    for (variants, spelling) in [
        (SHARED, "indexing"),
        (
            "      - Ready\n      - {name: Indexing, wire: Ready}\n",
            "Ready",
        ),
    ] {
        let errors = assemble(&model("ess/32", variants))
            .expect_err("two variants with one spelling cannot be told apart on the wire");
        assert!(
            errors.contains(&ValidationCode::ConflictingDeclaration.to_string()),
            "{variants}: {errors}"
        );
        assert!(
            errors.contains(&format!("`{spelling}`")),
            "{variants}: names the spelling: {errors}"
        );
    }
}

#[test]
fn two_enum_variants_with_one_wire_spelling_still_validate_below_ess_32() {
    let spec = assemble(&model("ess/31", SHARED)).expect("below ess/32 the enum keeps validating");
    assert_eq!(variants_of(&spec).len(), 2);
}
