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

fn document() -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("ess-ui-test-confirm");
    std::fs::create_dir_all(&dir).expect("the dir is made");
    std::fs::write(
        dir.join("things.yaml"),
        "views:\n  things.Page:\n    rows:\n      - {id: th-1, name: Anvil}\n      - {id: th-2, name: Bolt}\n",
    )
    .expect("the fixture is written");
    std::fs::write(dir.join("ui.yaml"), DOCUMENT).expect("the document is written");
    dir.join("ui.yaml")
}

fn file(steps: &[&str]) -> ess_ui_test::TestFile {
    let mut text = format!(
        "format: ess-ui-test/1\ndocument: {}\ntests:\n- name: confirm\n  steps:\n",
        document().display()
    );
    for step in steps {
        text.push_str("  - ");
        text.push_str(step);
        text.push('\n');
    }
    ess_ui_test::parse_str(&text, Path::new("confirm.yaml"))
        .unwrap_or_else(|error| panic!("{error}\n{text}"))
}

fn run(steps: &[&str]) -> Outcome {
    let report =
        ess_ui_test::execute(&document(), &[file(steps)]).unwrap_or_else(|error| panic!("{error}"));
    report.tests.into_iter().next().expect("one outcome")
}

fn spec(steps: &[&str]) -> String {
    ess_ui_test::playwright(
        &[file(steps)],
        &ess_ui::load_path(&document()).expect("the document loads"),
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
    let outcome = run(&steps);
    assert_eq!(outcome.status, Status::Passed, "{outcome:?}");
    let spec = spec(&steps);
    let at = format!("[data-ui-path=\"{CONFIRM}\"]");
    for wanted in [
        format!("await expect(page.locator('{at}')).toContainText(\"Remove thing\");"),
        format!("await page.locator('{at} button.ui-tone-danger, {at} button.ui-tone-primary').last().click();"),
    ] {
        assert!(spec.contains(&wanted), "{wanted}\n\n{spec}");
    }
    assert!(!spec.contains("test.fixme"), "{spec}");
}

#[test]
fn another_rows_confirm_is_not_open() {
    let outcome = run(&[
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
    let outcome = run(&steps);
    assert_eq!(outcome.step, Some(3), "{outcome:?}");
    let message = outcome.message.unwrap_or_default();
    assert!(message.contains("under its row"), "{message}");
    assert!(spec(&steps).contains("under its row"));
}

#[test]
fn a_confirm_overlay_without_does_runs_the_opening_action() {
    let outcome = run(&[
        "open: things.list",
        "act: pages/things.list/sections/list/rows/th-2/row_actions/drop",
        "act: pages/things.list/overlays/ask",
        "expect_command: {command: things.Drop, input: {thing: th-2}}",
    ]);
    assert_eq!(outcome.status, Status::Passed, "{outcome:?}");
}
