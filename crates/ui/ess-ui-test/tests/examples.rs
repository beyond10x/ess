//! The example application's own tests, `examples/partner-portal/tests/`, run headless against
//! the terminal renderer and pass.

use std::path::{Path, PathBuf};

use ess_ui_test::{Status, TestFile};

fn example_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../examples/partner-portal")
}

fn example_tests() -> Vec<TestFile> {
    let mut files: Vec<PathBuf> = std::fs::read_dir(example_dir().join("tests"))
        .expect("the example has tests")
        .map(|entry| entry.expect("a directory entry").path())
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension == "yaml")
        })
        .collect();
    files.sort();
    files
        .iter()
        .map(|file| ess_ui_test::load(file).unwrap_or_else(|error| panic!("{error}")))
        .collect()
}

#[test]
fn every_example_test_passes() {
    let files = example_tests();
    let report = ess_ui_test::execute(&example_dir().join("ui.yaml"), &files)
        .unwrap_or_else(|error| panic!("{error}"));
    let failed: Vec<String> = report
        .tests
        .iter()
        .filter(|outcome| outcome.status == Status::Failed)
        .map(|outcome| {
            format!(
                "{} (step {:?}): {}",
                outcome.name,
                outcome.step,
                outcome.message.clone().unwrap_or_default()
            )
        })
        .collect();
    assert!(failed.is_empty(), "{}", failed.join("\n\n"));
    // A green run that ran nothing is not green: every declared test ran.
    let declared: usize = files.iter().map(|file| file.tests.len()).sum();
    assert_eq!(report.tests.len(), declared);
    assert!(declared >= 7, "{declared}");
    assert!(report.passed());
}

#[test]
fn the_examples_cover_a_list_page_a_live_update_and_a_stale_channel() {
    let files = example_tests();
    let steps: Vec<String> = files
        .iter()
        .flat_map(|file| &file.tests)
        .flat_map(|test| test.steps.iter().map(|step| step.keyword().to_owned()))
        .collect();
    for keyword in [
        "open",
        "type",
        "choose",
        "act",
        "page",
        "expect",
        "play",
        "advance",
        "expect_command",
    ] {
        assert!(steps.iter().any(|step| step == keyword), "{keyword}");
    }
    let names: Vec<&str> = files
        .iter()
        .flat_map(|file| &file.tests)
        .map(|test| test.name.as_str())
        .collect();
    assert!(
        names.iter().any(|name| name.contains("filtered list")),
        "{names:?}"
    );
    assert!(names.iter().any(|name| name.contains("stale")), "{names:?}");
    assert!(
        names.iter().any(|name| name.contains("deleting")),
        "{names:?}"
    );
}

#[test]
fn a_test_file_names_its_document_and_a_different_document_is_refused() {
    let files = example_tests();
    assert!(files.iter().all(|file| file.document == "../ui.yaml"));
    let error = ess_ui_test::execute(&example_dir().join("fixtures").join("portal.yaml"), &files)
        .expect_err("a test file for another document is refused");
    assert!(error.to_string().contains("ui.yaml"), "{error}");
}
