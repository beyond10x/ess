//! In fixture mode a confirmed command changes the fixture rows the way the model says it does
//! (story:fixture-mode-applies-a-confirmed-commands-effect): approving `r1` through its confirm
//! takes it out of the pending list, without a reload and without a timed script.

use std::path::{Path, PathBuf};

use ess_ui_test::{execute_with, load, Status};

fn dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/fixture-command-effect")
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

#[test]
fn an_approved_request_leaves_the_pending_list() {
    let ui = dir().join("ui");
    let files = [load(&ui.join("approve.test.yaml")).expect("the tests read")];
    let model = model();
    let report = execute_with(&ui.join("desk.ui.yaml"), &files, Some(&model)).expect("it runs");
    assert!(report.passed(), "{:#?}", report.tests);
    assert_eq!(report.tests.len(), 1);
}

/// Without a model the effect is not known, so the rows stay as written.
#[test]
fn without_a_model_the_rows_stay() {
    let ui = dir().join("ui");
    let files = [load(&ui.join("approve.test.yaml")).expect("the tests read")];
    let report = execute_with(&ui.join("desk.ui.yaml"), &files, None).expect("it runs");
    let outcome = &report.tests[0];
    assert_eq!(outcome.status, Status::Failed);
    assert_eq!(outcome.step, Some(7), "{outcome:?}");
    let message = outcome.message.as_deref().unwrap_or_default();
    assert!(message.contains("expected 1 rows"), "{message}");
}
