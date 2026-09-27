//! The ess/15 outcome shapes in a semantic diff (`docs/design/outcome-shapes.md`): the state a
//! creation lands in, a removal, an unknown-instance marker and an accepted no-op are each outcome
//! changes, and a model that does not use them diffs as it always did.

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_diff::{diff, CommandChange, SemanticChange};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

const MODEL: &str =
    include_str!("../../../specify/ess-compiler/tests/fixtures/outcome-shapes.yaml");

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap();
    let spec = Specification::assemble([(Source::new("outcome-shapes.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap()
}

fn command_changes(before: &str, after: &str) -> Vec<CommandChange> {
    diff(&ir(before), &ir(after))
        .unwrap()
        .changes()
        .iter()
        .filter_map(|change| match change {
            SemanticChange::Command { changed, .. } => Some(changed.clone()),
            _ => None,
        })
        .collect()
}

#[test]
fn the_state_a_creation_lands_in_is_an_outcome_subject_change() {
    let changes = command_changes(MODEL, &MODEL.replace("into: Ringing", "into: Connected"));
    assert!(
        changes.iter().any(|change| matches!(change,
            CommandChange::OutcomeSubjectChanged { outcome, before: Some(before), after: Some(after) }
                if outcome == "offered"
                    && before.contains("into Ringing")
                    && after.contains("into Connected"))),
        "{changes:#?}"
    );
}

#[test]
fn a_removal_and_an_update_differ() {
    let changes = command_changes(
        MODEL,
        &MODEL.replace("deletes: example.call.Call", "updates: example.call.Call"),
    );
    assert!(
        changes.iter().any(|change| matches!(change,
            CommandChange::OutcomeSubjectChanged { outcome, before: Some(before), .. }
                if outcome == "ended" && before.starts_with("deletes "))),
        "{changes:#?}"
    );
}

#[test]
fn an_unknown_instance_marker_is_an_outcome_condition() {
    let changes = command_changes(
        MODEL,
        &MODEL.replace(
            "      - {name: no-such-call, unknown_instance: true, error: example.call.CallNotFound}\n      - {name: not-ringing, wrong_state: true, refuses: false}\n",
            "      - {name: not-ringing, wrong_state: true, refuses: false}\n",
        ),
    );
    assert!(
        changes.iter().any(|change| matches!(change,
            CommandChange::OutcomeRemoved { outcome } if outcome == "no-such-call")),
        "{changes:#?}"
    );
}

#[test]
fn a_changed_precondition_is_not_silent() {
    let other_user = MODEL.replace(
        "00000000-0000-4000-8000-000000000152",
        "00000000-0000-4000-8000-000000000153",
    );
    assert!(!diff(&ir(MODEL), &ir(&other_user)).unwrap().is_empty());
}

#[test]
fn identical_models_diff_empty() {
    assert!(diff(&ir(MODEL), &ir(MODEL)).unwrap().is_empty());
}
