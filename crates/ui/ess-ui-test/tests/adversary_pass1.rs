//! Adversary pass 1: tests that pass although the UI is wrong, a Playwright spec that means
//! something else than the terminal run of the same file, and malformed files that panic.

use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::{Path, PathBuf};

use ess_ui_test::{Outcome, Status};

fn document() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../examples/partner-portal/ui.yaml")
        .canonicalize()
        .expect("the example document exists")
}

fn file(tests: &str) -> ess_ui_test::TestFile {
    let text = format!(
        "format: ess-ui-test/1\ndocument: {}\ntests:\n{tests}",
        document().display()
    );
    ess_ui_test::parse_str(&text, Path::new("adversary.yaml"))
        .unwrap_or_else(|error| panic!("{error}\n{text}"))
}

fn run(tests: &str) -> Outcome {
    let report =
        ess_ui_test::execute(&document(), &[file(tests)]).unwrap_or_else(|error| panic!("{error}"));
    report.tests.into_iter().next().expect("one outcome")
}

/// One test of `steps`, each a one-line YAML step.
fn steps(steps: &[&str]) -> String {
    let mut text = String::from("- name: adversary\n  steps:\n");
    for step in steps {
        text.push_str("  - ");
        text.push_str(step);
        text.push('\n');
    }
    text
}

/// An expectation about one overlay must not be satisfied while another overlay is open.
/// `require_overlay` accepts any open overlay when the wanted overlay's title appears anywhere on
/// the screen — here the header's "New partner" button behind the delete confirm.
#[test]
fn an_expectation_at_an_overlay_that_is_not_open_fails() {
    let outcome = run(&steps(&[
        "open: partners.list",
        "act: pages/partners.list/sections/list/rows/pt-003/row_actions/delete",
        // The delete confirm is open; the create form is not.
        "expect: {at: pages/partners.list/overlays/create, text: Delete partner}",
    ]));
    assert_eq!(
        outcome.status,
        Status::Failed,
        "the create overlay is not open, yet an expectation at it passed: {outcome:?}"
    );
    assert_eq!(outcome.step, Some(3), "{outcome:?}");
}

/// A row count must count the rows shown. Rows are recognised by string values unique to them;
/// two rows that show the same values are recognised as no row at all, so a list still showing
/// three rows passes `rows: 1` and `rows: [pt-203]` — the assertion a filter test makes.
#[test]
fn rows_that_show_the_same_values_are_still_counted() {
    let fixtures = "  fixtures:\n    views:\n      partners.Page:\n        rows:\n\
         \x20         - {id: pt-201, name: Acme Group, tier: silver, tags: []}\n\
         \x20         - {id: pt-202, name: Acme Group, tier: silver, tags: []}\n\
         \x20         - {id: pt-203, name: Birch Works, tier: gold, tags: []}\n";
    let wrong = format!(
        "- name: three rows are one\n{fixtures}  steps:\n\
         \x20 - open: partners.list\n\
         \x20 - expect: {{at: pages/partners.list/sections/list, rows: [pt-203]}}\n"
    );
    let outcome = run(&wrong);
    assert_eq!(
        outcome.status,
        Status::Failed,
        "three rows are shown, yet `rows: [pt-203]` passed: {outcome:?}"
    );
    let right = format!(
        "- name: three rows are three\n{fixtures}  steps:\n\
         \x20 - open: partners.list\n\
         \x20 - expect: {{at: pages/partners.list/sections/list, rows: 3}}\n"
    );
    let outcome = run(&right);
    assert_eq!(outcome.status, Status::Passed, "{outcome:?}");
}

/// `expect text` at a node inside a row or a section checks that node. The runner checks the
/// whole row line, the whole section box or the whole screen instead, so text shown by a sibling
/// satisfies it — while the Playwright spec of the same file checks the node's own element.
#[test]
fn text_at_a_nested_node_is_not_satisfied_by_its_siblings() {
    // The cells are a table's: a cards collection with an `item` (partners.list) renders no
    // column cells in the generated app, so a step at one is refused in both renderers.
    let cases = [
        // The priority column of tk-01 shows `high`, not the ticket's subject.
        (
            "open: tickets.list",
            "expect: {at: pages/tickets.list/sections/list/rows/tk-01/columns/priority, text: SSO login fails}",
        ),
        // The column (header) of the whole collection does not show a row's subject.
        (
            "open: tickets.list",
            "expect: {at: pages/tickets.list/sections/list/columns/priority, text: SSO login fails}",
        ),
        // The hint below the list shows its tip, not a partner.
        (
            "open: partners.list",
            "expect: {at: pages/partners.list/sections/list/children/hint, text: Cedar Partners}",
        ),
        // The header's `create` action shows `New partner`, not a partner.
        (
            "open: partners.list",
            "expect: {at: pages/partners.list/header/actions/create, text: Cedar Partners}",
        ),
    ];
    let passed: Vec<&str> = cases
        .iter()
        .filter(|(open, step)| run(&steps(&[open, step])).status == Status::Passed)
        .map(|(_, step)| *step)
        .collect();
    assert!(
        passed.is_empty(),
        "text shown by another node satisfied these:\n{}",
        passed.join("\n")
    );
    // Control: the cell is read, not refused.
    let control = run(&steps(&[
        "open: tickets.list",
        "expect: {at: pages/tickets.list/sections/list/rows/tk-01/columns/priority, text: high}",
    ]));
    assert_eq!(control.status, Status::Passed, "{control:?}");
    let spec = ess_ui_test::playwright(
        &[file(&steps(&[cases[0].0, cases[0].1]))],
        &ess_ui::load_path(&document()).expect("the document loads"),
    );
    assert!(
        spec.contains(
            "page.locator('[data-ui-path=\"pages/tickets.list/sections/list/rows/tk-01/columns/priority\"]')"
        ),
        "the browser checks the column's own element:\n{spec}"
    );
}

/// `page: {to}` names an absolute page. The terminal goes to it from wherever it is; the
/// Playwright spec clicks "next" `to - 1` times from wherever it is, so going back to page 1
/// emits no action at all and the browser stays on page 2.
#[test]
fn playwright_goes_back_to_an_earlier_page() {
    let rows = (1..=12)
        .map(|n| {
            format!("          - {{id: pt-{n:03}x, name: Partner {n:02}, tier: silver, tags: []}}")
        })
        .collect::<Vec<_>>()
        .join("\n")
        + "\n";
    let tests = format!(
        "- name: back\n  fixtures:\n    views:\n      partners.Page:\n        total: 12\n        rows:\n{rows}  steps:\n\
         \x20 - open: partners.list\n\
         \x20 - page: {{at: pages/partners.list/sections/list, to: 2}}\n\
         \x20 - page: {{at: pages/partners.list/sections/list, to: 1}}\n\
         \x20 - expect: {{at: pages/partners.list/sections/list, rows: 10}}\n"
    );
    // The terminal run holds: page 1 shows ten rows again.
    let outcome = run(&tests);
    assert_eq!(outcome.status, Status::Passed, "{outcome:?}");
    let spec = ess_ui_test::playwright(
        &[file(&tests)],
        &ess_ui::load_path(&document()).expect("the document loads"),
    );
    let lines: Vec<&str> = spec.lines().map(str::trim).collect();
    let back = lines
        .iter()
        .position(|line| *line == "// page 1 of pages/partners.list/sections/list")
        .unwrap_or_else(|| panic!("the page step is emitted:\n{spec}"));
    let next = lines.get(back + 1).copied().unwrap_or_default();
    assert!(
        next.contains("click"),
        "`page to: 1` after page 2 emits no navigation; the next line is `{next}`:\n{spec}"
    );
}

/// A malformed duration is refused, not a panic: `m` and `h` multiply seconds without a check.
#[test]
fn an_overflowing_duration_is_refused_not_a_panic() {
    for text in [
        "- advance: 307445734561825861m\n",
        "- advance: 5124095576030432h\n",
    ] {
        let parsed = catch_unwind(|| ess_ui_test::parse_steps(text));
        let parsed = parsed.unwrap_or_else(|_| panic!("parsing `{}` panicked", text.trim()));
        assert!(parsed.is_err(), "`{}` parsed: {parsed:?}", text.trim());
    }
}

/// Moving the clock past what a `Duration` holds fails the test, not the whole run. Every
/// channel's script is replaced by an empty one: with the document's looping scripts the same
/// file does not panic, it plays cycles until the process is killed.
#[test]
fn advancing_past_the_end_of_time_fails_the_test_not_the_run() {
    let tests = "- name: end of time\n  fixtures:\n    scripts:\n\
         \x20     activity: {events: []}\n\
         \x20     metrics: {events: []}\n\
         \x20     tickets: {events: []}\n\
         \x20     ticket_chat: {events: []}\n\
         \x20 steps:\n\
         \x20 - open: overview\n\
         \x20 - advance: 18446744073709551615s\n\
         \x20 - advance: 18446744073709551615s\n"
        .to_owned();
    let file = file(&tests);
    let report = catch_unwind(AssertUnwindSafe(|| {
        ess_ui_test::execute(&document(), &[file])
    }))
    .unwrap_or_else(|_| panic!("the run panicked instead of reporting a failed test"));
    let report = report.unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(report.tests[0].status, Status::Failed, "{report:?}");
}
