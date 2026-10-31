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

const TRANSITION: &str = "{name: finish, from: [Indexing], to: Ready}";

/// The fixture under `ess/<format>`, its states written as `states` and its one transition as
/// `transition` (`story:a-transition-and-a-state-may-carry-a-summary`).
fn summarised(format: u32, states: &str, transition: &str) -> String {
    assert!(FIXTURE.contains(TRANSITION));
    lifecycle(format, states).replacen(TRANSITION, transition, 1)
}

const SUMMARISED_STATES: &str =
    "[{name: Indexing, wire: indexing, summary: Being indexed}, {name: Ready, summary: Searchable}]";
const SUMMARISED_TRANSITION: &str =
    "{name: finish, from: [Indexing], to: Ready, summary: The index is written}";

#[test]
fn a_lifecycle_state_and_transition_carry_a_summary_from_ess_33() {
    let spec = assemble(&summarised(33, SUMMARISED_STATES, SUMMARISED_TRANSITION))
        .expect("ess/33 admits a summary on a state and on a transition");
    let job = &spec.entities()[&"jobs.queue.Job".parse().unwrap()];
    assert_eq!(
        job.states.transitions[0].summary.as_deref(),
        Some("The index is written"),
        "the transition keeps its summary"
    );
    let ready = StateName::new("Ready").unwrap();
    assert!(
        job.states.states.contains(&ready),
        "{:?}",
        job.states.states
    );
    assert_eq!(
        job.states.wire(&ready),
        "Ready",
        "a summary-only state declares no wire spelling"
    );
    let state_type = job.state_type();
    let TypeBody::Enum { variants } = &state_type.body else {
        panic!("the state type is an enum: {state_type:?}");
    };
    assert_eq!(
        variants
            .iter()
            .map(|variant| (
                variant.name(),
                variant.wire(),
                variant.naming.summary.as_deref()
            ))
            .collect::<Vec<_>>(),
        vec![
            ("Indexing", "indexing", Some("Being indexed")),
            ("Ready", "Ready", Some("Searchable")),
        ],
        "each state's summary reaches its variant of the state enum"
    );
}

#[test]
fn a_lifecycle_summary_below_ess_33_is_refused_naming_ess_33() {
    for (states, transition, located) in [
        (
            "[Indexing, {name: Ready, summary: Searchable}]",
            TRANSITION,
            "jobs.queue.Job.states.Ready",
        ),
        (
            "[{name: Indexing, wire: indexing, summary: Being indexed}, Ready]",
            TRANSITION,
            "jobs.queue.Job.states.Indexing",
        ),
        (
            "[Indexing, Ready]",
            SUMMARISED_TRANSITION,
            "jobs.queue.Job.transitions[0]",
        ),
    ] {
        let errors = assemble(&summarised(32, states, transition))
            .expect_err("a lifecycle summary is not admitted below ess/33");
        assert!(
            errors.contains(&ValidationCode::UnsupportedFormatVersion.to_string()),
            "{states} {transition}: {errors}"
        );
        assert!(errors.contains("ess/33"), "names the format: {errors}");
        assert!(errors.contains(located), "located at {located}: {errors}");
        assert!(
            !errors.contains("unknown field"),
            "no whole-file parse error: {errors}"
        );
        assert!(
            !errors.contains("requires specification format ess/32"),
            "an ess/32 document spells a state's wire name: {errors}"
        );
    }
}

#[test]
fn one_state_with_two_summaries_is_refused() {
    let errors = assemble(&summarised(
        33,
        "[Indexing, {name: Ready, summary: Searchable}, {name: Ready, summary: Done}]",
        TRANSITION,
    ))
    .expect_err("two summaries of one state are two claims about it");
    assert!(
        errors.contains(&ValidationCode::ConflictingDeclaration.to_string()),
        "{errors}"
    );
    assert!(errors.contains("jobs.queue.Job.states.Ready"), "{errors}");
}

#[test]
fn a_state_mapping_with_nothing_beside_its_name_is_refused_naming_the_bare_form() {
    let errors = assemble(&summarised(33, "[Indexing, {name: Ready}]", TRANSITION))
        .expect_err("a state with nothing to declare is written as its name");
    assert!(
        errors.contains("write the state as the name"),
        "names the bare form: {errors}"
    );
}
