//! A row action's inline confirm is addressed under its row,
//! `<collection>/rows/<key>/row_actions/<action>/confirm/overlay`, in the terminal and in the
//! Playwright spec; confirming runs the action, whether or not the confirm declares `does`.

use std::path::{Path, PathBuf};

use ess_ui_test::{Outcome, Status};

const DOCUMENT: &str = r"format: ess-ui/1
app: probe
model: probe.system
placement_profile: fat
fixtures: {views: {things.Page: things.yaml}}
shells:
  app:
    regions:
      nav:     {kind: navigation}
      main:    {kind: page_outlet}
      overlay: {kind: overlay_outlet}
navigation:
  home: things.list
  sections: [{name: all, pages: [things.list]}]
pages:
  things.list:
    kind: list_page
    title: Things
    sections:
      - name: list
        component: collection
        reads: {view: things.Page}
        columns: [name]
        row_actions:
          - {name: remove, does: things.Remove, bind: {thing: row.id}, label: Remove, confirm: {title: Remove thing}}
          - {name: drop, does: things.Drop, bind: {thing: row.id}, label: Drop, confirm: ask}
    overlays:
      ask: {kind: dialog, component: confirm, title: Drop thing}
";

/// Each test writes its own copy: a test that rewrote a shared `ui.yaml` while another read it
/// handed that reader an empty document.
fn document(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("ess-ui-test-confirm")
        .join(name);
    std::fs::create_dir_all(&dir).expect("the dir is made");
    std::fs::write(
        dir.join("things.yaml"),
        "views:\n  things.Page:\n    rows:\n      - {id: th-1, name: Anvil}\n      - {id: th-2, name: Bolt}\n",
    )
    .expect("the fixture is written");
    std::fs::write(dir.join("ui.yaml"), DOCUMENT).expect("the document is written");
    dir.join("ui.yaml")
}

fn file(name: &str, steps: &[&str]) -> ess_ui_test::TestFile {
    let mut text = format!(
        "format: ess-ui-test/1\ndocument: {}\ntests:\n- name: confirm\n  steps:\n",
        document(name).display()
    );
    for step in steps {
        text.push_str("  - ");
        text.push_str(step);
        text.push('\n');
    }
    ess_ui_test::parse_str(&text, Path::new("confirm.yaml"))
        .unwrap_or_else(|error| panic!("{error}\n{text}"))
}

fn run(name: &str, steps: &[&str]) -> Outcome {
    let report = ess_ui_test::execute(&document(name), &[file(name, steps)])
        .unwrap_or_else(|error| panic!("{error}"));
    report.tests.into_iter().next().expect("one outcome")
}

fn spec(name: &str, steps: &[&str]) -> String {
    ess_ui_test::playwright(
        &[file(name, steps)],
        &ess_ui::load_path(&document(name)).expect("the document loads"),
    )
}

const CONFIRM: &str =
    "pages/things.list/sections/list/rows/th-2/row_actions/remove/confirm/overlay";

#[test]
fn an_inline_confirm_is_addressed_under_its_row_and_confirming_runs_the_action() {
    let steps = [
        "open: things.list",
        "act: pages/things.list/sections/list/rows/th-2/row_actions/remove",
        &format!("expect: {{at: {CONFIRM}, text: Remove thing}}"),
        &format!("act: {CONFIRM}"),
        "expect_command: {command: things.Remove, input: {thing: th-2}}",
    ];
    let outcome = run("inline", &steps);
    assert_eq!(outcome.status, Status::Passed, "{outcome:?}");
    let spec = spec("inline", &steps);
    let at = format!("[data-ui-path=\"{CONFIRM}\"]");
    for wanted in [
        format!("await expect(page.locator('{at}')).toContainText(\"Remove thing\");"),
        format!("await page.locator('{at} button.ui-emphasis-danger, {at} button.ui-emphasis-primary').last().click();"),
    ] {
        assert!(spec.contains(&wanted), "{wanted}\n\n{spec}");
    }
    assert!(!spec.contains("test.fixme"), "{spec}");
}

#[test]
fn another_rows_confirm_is_not_open() {
    let outcome = run("another-row", &[
        "open: things.list",
        "act: pages/things.list/sections/list/rows/th-2/row_actions/remove",
        "expect: {at: pages/things.list/sections/list/rows/th-1/row_actions/remove/confirm/overlay, text: Remove thing}",
    ]);
    assert_eq!(outcome.step, Some(3), "{outcome:?}");
    assert!(
        outcome.message.unwrap_or_default().contains("is not open"),
        "a confirm opened from th-2 is not th-1's"
    );
}

#[test]
fn a_row_actions_confirm_without_its_row_is_refused_in_both_renderers() {
    let steps = [
        "open: things.list",
        "act: pages/things.list/sections/list/rows/th-2/row_actions/remove",
        "act: pages/things.list/sections/list/row_actions/remove/confirm/overlay",
    ];
    let outcome = run("without-row", &steps);
    assert_eq!(outcome.step, Some(3), "{outcome:?}");
    let message = outcome.message.unwrap_or_default();
    assert!(message.contains("under its row"), "{message}");
    assert!(spec("without-row", &steps).contains("under its row"));
}

/// A view keyed by `reads.key`. `rows` is the fixture's rows, one per line.
fn keyed_document(name: &str, rows: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("ess-ui-test-confirm-keyed")
        .join(name);
    std::fs::create_dir_all(&dir).expect("the dir is made");
    std::fs::write(
        dir.join("things.yaml"),
        format!("views:\n  things.Page:\n    rows:\n{rows}"),
    )
    .expect("the fixture is written");
    std::fs::write(
        dir.join("ui.yaml"),
        DOCUMENT
            .replace(
                "reads: {view: things.Page}",
                "reads: {view: things.Page, key: thing_key}",
            )
            .replace("row.id", "row.thing_key"),
    )
    .expect("the document is written");
    dir.join("ui.yaml")
}

fn run_keyed(name: &str, rows: &str, steps: &[&str]) -> Outcome {
    let document = keyed_document(name, rows);
    let mut text = format!(
        "format: ess-ui-test/1\ndocument: {}\ntests:\n- name: keyed confirm\n  steps:\n",
        document.display()
    );
    for step in steps {
        text.push_str("  - ");
        text.push_str(step);
        text.push('\n');
    }
    let file = ess_ui_test::parse_str(&text, Path::new("confirm.yaml"))
        .unwrap_or_else(|error| panic!("{error}\n{text}"));
    let report = ess_ui_test::execute(&document, &[file]).unwrap_or_else(|error| panic!("{error}"));
    report.tests.into_iter().next().expect("one outcome")
}

const NO_ID: &str = "      - {thing_key: k-1, name: Anvil}\n      - {thing_key: k-2, name: Bolt}\n";
const WITH_ID: &str =
    "      - {id: th-1, thing_key: k-1, name: Anvil}\n      - {id: th-2, thing_key: k-2, name: Bolt}\n";
const KEYED_CONFIRM: &str =
    "pages/things.list/sections/list/rows/k-2/row_actions/remove/confirm/overlay";

#[test]
fn a_confirm_on_a_view_keyed_by_reads_key_without_id_is_found_and_accepted() {
    let outcome = run_keyed(
        "no-id-accept",
        NO_ID,
        &[
            "open: things.list",
            "act: pages/things.list/sections/list/rows/k-2/row_actions/remove",
            &format!("expect: {{at: {KEYED_CONFIRM}, text: Remove thing}}"),
            &format!("act: {KEYED_CONFIRM}"),
            "expect_command: {command: things.Remove, input: {thing: k-2}}",
        ],
    );
    assert_eq!(
        outcome.status,
        Status::Passed,
        "story:a-row-actions-confirm-is-addressed-by-the-views-key {outcome:?}"
    );
}

#[test]
fn a_confirm_on_a_view_keyed_by_reads_key_without_id_is_found_and_dismissed() {
    let steps = [
        "open: things.list",
        "act: pages/things.list/sections/list/rows/k-2/row_actions/remove",
        &format!("expect: {{at: {KEYED_CONFIRM}, text: Remove thing}}"),
        "select: pages/things.list",
        &format!("expect: {{at: {KEYED_CONFIRM}, text: Remove thing}}"),
    ];
    let outcome = run_keyed("no-id-dismiss", NO_ID, &steps);
    assert_eq!(outcome.step, Some(5), "{outcome:?}");
    assert!(
        outcome.message.unwrap_or_default().contains("is not open"),
        "a dismissed confirm is closed"
    );
    let outcome = run_keyed(
        "no-id-dismissed-sends-nothing",
        NO_ID,
        &[
            "open: things.list",
            "act: pages/things.list/sections/list/rows/k-2/row_actions/remove",
            &format!("expect: {{at: {KEYED_CONFIRM}, text: Remove thing}}"),
            "select: pages/things.list",
            "expect_command: things.Remove",
        ],
    );
    assert_eq!(
        outcome.step,
        Some(5),
        "story:a-row-actions-confirm-is-addressed-by-the-views-key: a dismissed confirm sends no \
         command {outcome:?}"
    );
}

#[test]
fn a_view_with_id_and_reads_key_addresses_its_confirm_by_reads_key() {
    let outcome = run_keyed(
        "with-id",
        WITH_ID,
        &[
            "open: things.list",
            "act: pages/things.list/sections/list/rows/k-2/row_actions/remove",
            &format!("expect: {{at: {KEYED_CONFIRM}, text: Remove thing}}"),
            &format!("act: {KEYED_CONFIRM}"),
            "expect_command: {command: things.Remove, input: {thing: k-2}}",
        ],
    );
    assert_eq!(
        outcome.status,
        Status::Passed,
        "story:a-row-actions-confirm-is-addressed-by-the-views-key {outcome:?}"
    );
    let outcome = run_keyed(
        "with-id-not-by-id",
        WITH_ID,
        &[
            "open: things.list",
            "act: pages/things.list/sections/list/rows/k-2/row_actions/remove",
            "expect: {at: pages/things.list/sections/list/rows/th-2/row_actions/remove/confirm/overlay, text: Remove thing}",
        ],
    );
    assert_eq!(outcome.step, Some(3), "{outcome:?}");
}

#[test]
fn a_confirm_overlay_without_does_runs_the_opening_action() {
    let outcome = run(
        "without-does",
        &[
            "open: things.list",
            "act: pages/things.list/sections/list/rows/th-2/row_actions/drop",
            "act: pages/things.list/overlays/ask",
            "expect_command: {command: things.Drop, input: {thing: th-2}}",
        ],
    );
    assert_eq!(outcome.status, Status::Passed, "{outcome:?}");
}
