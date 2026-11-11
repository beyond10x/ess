//! Adversary cases for story:fixture-mode-applies-a-confirmed-commands-effect, terminal renderer
//! (`ess ui test --model`): what the model says a confirmed command does, beyond the repro.

use std::path::{Path, PathBuf};

use ess_ui_test::{execute_with, load};

fn dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/adversary-t3-fixture-command-effect")
}

fn model() -> ess_ui_check::Model {
    let sources: Vec<(String, String)> = ["system.yaml", "components.yaml", "requests.yaml"]
        .iter()
        .map(|file| {
            let text =
                std::fs::read_to_string(dir().join("spec").join(file)).expect("the model reads");
            ((*file).to_owned(), text)
        })
        .collect();
    ess_ui_check::model_from_sources(&sources, Path::new("spec")).expect("the model compiles")
}

fn passes(test_file: &str) {
    let ui = dir().join("ui");
    let files = [load(&ui.join(test_file)).expect("the tests read")];
    let model = model();
    let report = execute_with(&ui.join("desk.ui.yaml"), &files, Some(&model)).expect("it runs");
    assert_eq!(report.tests.len(), 1);
    assert!(report.passed(), "{:#?}", report.tests);
}

/// `UrgentRequests` filters on `priority == 1`; approving `r1` sets only its state, so the filter
/// still holds and the row stays (patched). The React runtime compares `String(1)` with
/// `String(1.0)` and keeps it.
#[test]
fn a_row_whose_numeric_filter_still_holds_stays_in_its_view() {
    passes("numeric-filter.test.yaml");
}

/// Approving `r1` makes `state == Approved` hold: the approved list, which reads the same entity,
/// lists it too (the story's outcome: every section and page that reads the entity shows the
/// change).
#[test]
#[ignore = "backlog: a changed row does not enter a view it did not list; fixture rows are per view"]
fn an_approved_request_enters_the_approved_view() {
    passes("enters-view.test.yaml");
}

/// `reject` leaves only `Pending`, and `Approved` is terminal: the model's `wrong_state` outcome
/// answers rejecting `r3`. Fixture mode must not show `r3` as `Rejected`.
#[test]
fn a_move_the_lifecycle_does_not_allow_from_the_rows_state_is_not_applied() {
    passes("forbidden-move.test.yaml");
}

/// A command the test declares refused changes no row.
#[test]
fn a_declared_refusal_applies_no_effect() {
    passes("refused.test.yaml");
}
