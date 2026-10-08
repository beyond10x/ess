//! The precedence plan of a compiled command (`docs/design/selection-plan.md`): one test per phase
//! and per composition that moves a branch, over models the conformance suite already runs.
use ess_compiler::ir::{EssIr, PrecedencePlan};
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_domain::command::precedence::Phase;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

const ABSENT_INPUT: &str = include_str!("fixtures/absent-input.yaml");
const UPSERT: &str = include_str!("fixtures/upsert-by-existence.yaml");
const MULTIPLE: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/related-guard-multiple.yaml");
const RELEASE: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/related-guard-release.yaml");
const OPTIONAL: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/related-guard-optional.yaml");
const STORED: &str = include_str!(
    "../../../verify/ess-conformance/tests/fixtures/related-guard-stored-reference.yaml"
);
const ROW_SETS: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/filtered-related-reads.yaml");
const UNIQUE: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/unique-within-scope.yaml");
const ROTATE: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/refusal-beside-state.yaml");
const PAUSE: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/held-state-input-refusal.yaml");
const PICK: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/external-beside-held-guard.yaml");

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}"));
    let spec = Specification::assemble([(Source::new("model.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|errors| panic!("{errors:?}"))
}

/// The non-empty phases of `command`'s plan, in order, each with its branches in order.
fn plan(text: &str, command: &str) -> Vec<(Phase, Vec<String>)> {
    let ir = ir(text);
    let command = &ir.commands()[&command.parse().unwrap()];
    let plan = PrecedencePlan::new(command, ir.format());
    let mut every: Vec<String> = plan
        .phases()
        .iter()
        .flat_map(|planned| {
            planned
                .branches
                .iter()
                .map(|outcome| outcome.name.to_string())
        })
        .collect();
    every.sort();
    let mut declared: Vec<String> = command
        .outcomes
        .iter()
        .map(|outcome| outcome.name.to_string())
        .collect();
    declared.sort();
    assert_eq!(every, declared, "every branch is planned exactly once");
    assert_eq!(
        plan.phases()
            .iter()
            .map(|planned| planned.phase)
            .collect::<Vec<_>>(),
        Phase::PRECEDENCE,
        "every phase, in the precedence order"
    );
    for planned in plan.phases() {
        for outcome in &planned.branches {
            assert_eq!(plan.phase_of(&outcome.name), Some(planned.phase));
        }
        assert_eq!(plan.branches(planned.phase), planned.branches.as_slice());
    }
    plan.phases()
        .iter()
        .filter(|planned| !planned.branches.is_empty())
        .map(|planned| {
            (
                planned.phase,
                planned
                    .branches
                    .iter()
                    .map(|outcome| outcome.name.to_string())
                    .collect(),
            )
        })
        .collect()
}

fn expect(phases: &[(Phase, &[&str])]) -> Vec<(Phase, Vec<String>)> {
    phases
        .iter()
        .map(|(phase, names)| (*phase, names.iter().map(ToString::to_string).collect()))
        .collect()
}

#[test]
fn a_missing_document_answers_before_every_step() {
    assert_eq!(
        plan(ABSENT_INPUT, "demo.notes.SubmitNote"),
        expect(&[
            (Phase::InputAbsent, &["body-missing"]),
            (Phase::Default, &["submitted"]),
        ])
    );
}

#[test]
fn step_one_reads_missing_input_rows_in_declaration_order_and_step_five_their_refusals() {
    assert_eq!(
        plan(MULTIPLE, "demo.run.StartRun"),
        expect(&[
            (Phase::RelatedRow, &["no-such-switch", "no-such-capability"]),
            (
                Phase::PresentRelated,
                &["switch-paused", "capability-revoked"]
            ),
            (Phase::Default, &["started"]),
        ])
    );
}

#[test]
fn existing_instance_answers_at_step_one_beside_a_row_set() {
    assert_eq!(
        plan(UNIQUE, "demo.binding.BindIdentity"),
        expect(&[
            (Phase::RelatedRow, &["already-bound"]),
            (Phase::PresentRelated, &["claims-taken"]),
            (Phase::Default, &["bound"]),
        ])
    );
}

#[test]
fn an_input_refusal_answers_at_step_two_before_the_held_state() {
    assert_eq!(
        plan(ROTATE, "demo.secrets.RotateSecret"),
        expect(&[
            (Phase::InputRefusal, &["too-short"]),
            (Phase::HeldState, &["rotated"]),
            (Phase::Default, &["not-configured"]),
        ])
    );
}

#[test]
fn existence_answers_at_step_three_on_a_command_reading_no_related_row() {
    assert_eq!(
        plan(UPSERT, "demo.items.PutItem"),
        expect(&[
            (Phase::Existence, &["created"]),
            (Phase::Default, &["updated"]),
        ])
    );
    assert_eq!(
        plan(UPSERT, "demo.items.BookSlot"),
        expect(&[
            (Phase::Existence, &["already-booked"]),
            (Phase::Default, &["booked"]),
        ])
    );
}

#[test]
fn a_held_state_input_refusal_and_wrong_state_answer_at_step_four() {
    assert_eq!(
        plan(PAUSE, "demo.session.Pause"),
        expect(&[
            (Phase::Existence, &["no-session"]),
            (Phase::HeldState, &["invalid-code", "not-active"]),
            (Phase::Default, &["paused"]),
        ])
    );
}

#[test]
fn a_held_state_branch_answers_before_an_external_branch_declared_after_it() {
    assert_eq!(
        plan(PICK, "demo.desk.CheckPick"),
        expect(&[
            (Phase::HeldState, &["stale", "wrong-state"]),
            (Phase::Accepting, &["unlisted"]),
            (Phase::Default, &["accepted"]),
        ])
    );
}

#[test]
fn a_present_related_refusal_moves_to_step_five_beside_wrong_state_from_ess_22() {
    assert_eq!(
        plan(OPTIONAL, "demo.release.PublishRelease"),
        expect(&[
            (Phase::RelatedRow, &["no-candidate"]),
            (Phase::HeldState, &["wrong-state"]),
            (Phase::PresentRelated, &["not-accepted"]),
            (Phase::Default, &["published"]),
        ])
    );
    // Below ess/22, with no `wrong_state`, it keeps declaration order among the step-6 branches.
    assert_eq!(
        plan(RELEASE, "demo.release.PublishRelease"),
        expect(&[
            (Phase::RelatedRow, &["no-candidate"]),
            (Phase::Accepting, &["not-accepted"]),
            (Phase::Default, &["published"]),
        ])
    );
}

#[test]
fn a_stored_reference_answers_entirely_at_step_five() {
    assert_eq!(
        plan(STORED, "demo.tasks.CompleteTask"),
        expect(&[
            (Phase::HeldState, &["wrong-state"]),
            (Phase::PresentRelated, &["blocker-missing", "blocked"]),
            (Phase::Default, &["completed"]),
        ])
    );
}

#[test]
fn a_row_set_refusal_answers_at_step_five_and_an_accepting_row_set_at_step_six() {
    assert_eq!(
        plan(ROW_SETS, "demo.jobs.Retry"),
        expect(&[
            (Phase::PresentRelated, &["ambiguous"]),
            (Phase::Accepting, &["started"]),
            (Phase::Default, &["retried"]),
        ])
    );
    assert_eq!(
        plan(ROW_SETS, "demo.jobs.Close"),
        expect(&[
            (Phase::Existence, &["unknown-attempt"]),
            (Phase::HeldState, &["already-closed"]),
            (Phase::PresentRelated, &["crowded"]),
            (Phase::Default, &["closed"]),
        ])
    );
    assert_eq!(
        plan(ROW_SETS, "demo.jobs.CheckLimit"),
        expect(&[
            (Phase::Accepting, &["within-limit"]),
            (Phase::Default, &["over-limit"]),
        ])
    );
}

const UPSERT_UNKNOWN: &str = include_str!("fixtures/precedence-upsert-unknown-instance.yaml");
const UPDATING: &str = include_str!("fixtures/precedence-row-set-updating-branch.yaml");

#[test]
fn unknown_instance_closes_step_five_on_a_row_set_upsert() {
    assert_eq!(
        plan(UPSERT_UNKNOWN, "demo.shelf.Shelve"),
        expect(&[
            (Phase::PresentRelated, &["refused", "unknown-book"]),
            (Phase::Accepting, &["added"]),
            (Phase::Default, &["replaced"]),
        ])
    );
    assert_eq!(
        plan(UPSERT_UNKNOWN, "demo.shelf.Restock"),
        expect(&[
            (Phase::Existence, &["unknown-book"]),
            (Phase::PresentRelated, &["refused"]),
            (Phase::Default, &["restocked"]),
        ])
    );
}

#[test]
fn wrong_state_stays_at_step_four_beside_an_accepting_branch_that_updates() {
    assert_eq!(
        plan(UPDATING, "demo.jobs.Close"),
        expect(&[
            (Phase::Existence, &["unknown-attempt"]),
            (Phase::HeldState, &["already-closed"]),
            (Phase::PresentRelated, &["crowded"]),
            (Phase::Accepting, &["annotated"]),
            (Phase::Default, &["closed"]),
        ])
    );
}
