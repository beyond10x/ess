//! Each way a test can be wrong fails that test, at the step that is wrong, for that reason.

use std::path::{Path, PathBuf};

use ess_ui_test::{Outcome, Status};

fn document() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../examples/partner-portal/ui.yaml")
        .canonicalize()
        .expect("the example document exists")
}

/// Runs one test whose steps are `steps` (YAML list items, one per line).
fn run_steps(name: &str, steps: &[&str]) -> Outcome {
    let text = format!(
        "format: ess-ui-test/1\ndocument: {}\ntests:\n  - name: {name}\n    steps:\n{}\n",
        document().display(),
        steps
            .iter()
            .map(|step| format!("      - {step}"))
            .collect::<Vec<_>>()
            .join("\n")
    );
    let file = ess_ui_test::parse_str(&text, Path::new("red.yaml"))
        .unwrap_or_else(|error| panic!("{error}\n{text}"));
    let report =
        ess_ui_test::execute(&document(), &[file]).unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(report.tests.len(), 1);
    report.tests.into_iter().next().expect("one outcome")
}

fn failed_at(outcome: &Outcome, step: usize) -> &str {
    assert_eq!(outcome.status, Status::Failed, "{outcome:?}");
    assert_eq!(outcome.step, Some(step), "{outcome:?}");
    outcome.message.as_deref().expect("a failure says why")
}

#[test]
fn the_control_passes() {
    let outcome = run_steps(
        "control",
        &[
            "open: partners.list",
            "expect: {at: pages/partners.list/sections/list/rows/pt-003, text: Cedar Partners}",
            "expect: {at: pages/partners.list/sections/list, rows: 6}",
        ],
    );
    assert_eq!(outcome.status, Status::Passed, "{outcome:?}");
    assert_eq!(outcome.step, None);
    assert_eq!(outcome.message, None);
}

#[test]
fn a_wrong_expected_text_fails_naming_the_text_and_the_node() {
    let outcome = run_steps(
        "wrong text",
        &[
            "open: partners.list",
            "expect: {at: pages/partners.list/sections/list/rows/pt-003, text: Birch Channel}",
        ],
    );
    let message = failed_at(&outcome, 2);
    assert!(message.contains("\"Birch Channel\""), "{message}");
    assert!(
        message.contains("pages/partners.list/sections/list/rows/pt-003"),
        "{message}"
    );
    assert!(
        message.contains("Cedar Partners"),
        "the failure shows what the node does show: {message}"
    );
}

#[test]
fn a_wrong_row_count_fails_naming_both_counts() {
    let outcome = run_steps(
        "wrong count",
        &[
            "open: partners.list",
            "expect: {at: pages/partners.list/sections/list, rows: 5}",
        ],
    );
    let message = failed_at(&outcome, 2);
    assert!(message.contains("expected 5 rows"), "{message}");
    assert!(message.contains("shows 6"), "{message}");

    let outcome = run_steps(
        "wrong keys",
        &[
            "open: partners.list",
            "type: {at: pages/partners.list/sections/filters, text: cedar}",
            "expect: {at: pages/partners.list/sections/list, rows: [pt-001]}",
        ],
    );
    let message = failed_at(&outcome, 3);
    assert!(message.contains("[pt-001]"), "{message}");
    assert!(message.contains("[pt-003]"), "{message}");
}

#[test]
fn a_path_that_names_no_node_fails_naming_the_path() {
    for (step, path) in [
        (
            "select: pages/partners.list/sections/nope",
            "pages/partners.list/sections/nope",
        ),
        (
            "act: pages/partners.list/sections/list/row_actions/nope",
            "pages/partners.list/sections/list/row_actions/nope",
        ),
        ("expect: {at: pages/nowhere, text: x}", "pages/nowhere"),
    ] {
        let outcome = run_steps("no node", &["open: partners.list", step]);
        let message = failed_at(&outcome, 2);
        assert!(message.contains(&format!("no node at {path}")), "{message}");
    }
    // A row key the collection does not hold names the key.
    let outcome = run_steps(
        "no row",
        &[
            "open: partners.list",
            "act: pages/partners.list/sections/list/rows/pt-999/row_actions/delete",
        ],
    );
    let message = failed_at(&outcome, 2);
    assert!(message.contains("no row pt-999"), "{message}");
    // A page that does not exist is a node that does not exist.
    let outcome = run_steps("no page", &["open: nowhere"]);
    assert!(failed_at(&outcome, 1).contains("no node at pages/nowhere"));
}

#[test]
fn an_expected_command_never_sent_fails_naming_the_command() {
    let outcome = run_steps(
        "never sent",
        &[
            "open: partners.list",
            "act: pages/partners.list/sections/list/rows/pt-003/row_actions/delete",
            "expect_command: {command: partners.DeletePartner, input: {id: pt-003}}",
        ],
    );
    let message = failed_at(&outcome, 3);
    assert!(message.contains("partners.DeletePartner"), "{message}");
    assert!(message.contains("never sent"), "{message}");

    // Sent, but with other input: the failure shows what was sent.
    let outcome = run_steps(
        "other input",
        &[
            "open: partners.list",
            "act: pages/partners.list/sections/list/rows/pt-003/row_actions/delete",
            "act: pages/partners.list/overlays/delete",
            "expect_command: {command: partners.DeletePartner, input: {id: pt-001}}",
        ],
    );
    let message = failed_at(&outcome, 4);
    assert!(message.contains("pt-003"), "{message}");
    assert!(message.contains("pt-001"), "{message}");
}

#[test]
fn a_later_step_does_not_run_after_a_failure_and_the_next_test_still_runs() {
    let text = format!(
        "format: ess-ui-test/1\ndocument: {}\ntests:\n\
         - name: first\n  steps:\n  - open: nowhere\n  - open: partners.list\n\
         - name: second\n  steps:\n  - open: partners.list\n",
        document().display()
    );
    let file = ess_ui_test::parse_str(&text, Path::new("two.yaml")).expect("parses");
    let report = ess_ui_test::execute(&document(), &[file]).expect("runs");
    assert_eq!(report.tests[0].status, Status::Failed);
    assert_eq!(report.tests[0].step, Some(1));
    assert_eq!(report.tests[1].status, Status::Passed);
    assert!(!report.passed());
}

#[test]
fn a_malformed_test_file_is_refused_with_its_place() {
    for (text, wanted) in [
        ("format: ess-ui-test/2\ndocument: x\ntests: []\n", "ess-ui-test/1"),
        (
            "format: ess-ui-test/1\ndocument: x\ntests:\n- name: a\n  steps:\n  - teleport: home\n",
            "teleport",
        ),
        (
            "format: ess-ui-test/1\ndocument: x\ntests:\n- name: a\n  steps:\n  - advance: soon\n",
            "soon",
        ),
        (
            "format: ess-ui-test/1\ndocument: x\ntests:\n- name: a\n  steps:\n  - expect: {at: pages/x, state: sleepy}\n",
            "sleepy",
        ),
    ] {
        let error = ess_ui_test::parse_str(text, Path::new("bad.yaml"))
            .err()
            .unwrap_or_else(|| panic!("refused: {text}"));
        let error = error.to_string();
        assert!(error.contains("bad.yaml"), "{error}");
        assert!(error.contains(wanted), "{error}");
    }
}
