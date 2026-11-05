//! A journey clicks a row toggle in either state: the terminal writes a toggle's hint with its
//! state mark (`k [x] Enabled`, `k [ ] Enabled`), and the runner finds the toggle by its label
//! behind that mark. An action that is not a toggle is still found by its whole label only.

use std::path::{Path, PathBuf};

use ess_ui_test::{Outcome, Status};

const DOCUMENT: &str = r#"format: ess-ui/1
app: probe
model: probe.system
placement_profile: fat
fixtures: {views: {shelf.Items: items.yaml}}
shells:
  app:
    regions:
      main: {kind: page_outlet}
navigation:
  home: items
  sections: [{name: all, pages: [items]}]
pages:
  items:
    kind: list_page
    shell: app
    title: Items
    sections:
      - name: list
        component: collection
        reads: {view: shelf.Items}
        columns: [name]
        row_actions:
          - {name: enabled, does: shelf.SetEnabled, bind: {id: row.id, enabled: not row.enabled}, as: toggle, label: Enabled}
          - {name: pin, does: shelf.Pin, bind: {id: row.id}, label: "[x] Pinned"}
          - {name: enable, does: shelf.Enable, bind: {id: row.id}, label: Enabled, visible: "row.state == closed"}
"#;

const FIXTURE: &str = "views:\n  shelf.Items:\n    rows:\n      \
                       - {id: it-1, name: Anvil, enabled: true, state: open}\n      \
                       - {id: it-2, name: Bolt, enabled: false, state: open}\n";

const ROWS: &str = "pages/items/sections/list/rows";

/// Each test writes its own copy of the document, so no test reads another's half-written file.
fn document(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("ess-ui-test-row-toggle")
        .join(name);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("the old scratch dir is removed");
    }
    std::fs::create_dir_all(&dir).expect("the scratch dir is made");
    std::fs::write(dir.join("items.yaml"), FIXTURE).expect("the fixture is written");
    std::fs::write(dir.join("ui.yaml"), DOCUMENT).expect("the document is written");
    dir.join("ui.yaml")
}

fn run(name: &str, steps: &[String]) -> Outcome {
    let document = document(name);
    let mut text = format!(
        "format: ess-ui-test/1\ndocument: {}\ntests:\n- name: row toggle\n  steps:\n",
        document.display()
    );
    for step in steps {
        text.push_str("  - ");
        text.push_str(step);
        text.push('\n');
    }
    let file = ess_ui_test::parse_str(&text, Path::new("row_toggle.yaml"))
        .unwrap_or_else(|error| panic!("{error}\n{text}"));
    let report = ess_ui_test::execute(&document, &[file]).unwrap_or_else(|error| panic!("{error}"));
    report.tests.into_iter().next().expect("one outcome")
}

#[test]
fn a_journey_clicks_a_row_toggle_that_is_off() {
    let outcome = run(
        "off",
        &[
            "open: items".to_owned(),
            format!("select: {ROWS}/it-2"),
            "expect: {at: pages/items/sections/list, text: '[ ] Enabled'}".to_owned(),
            format!("act: {ROWS}/it-2/row_actions/enabled"),
            "expect_command: {command: shelf.SetEnabled, input: {id: it-2, enabled: true}}"
                .to_owned(),
        ],
    );
    assert_eq!(outcome.status, Status::Passed, "{outcome:?}");
}

#[test]
fn a_journey_clicks_a_row_toggle_that_is_on() {
    let outcome = run(
        "on",
        &[
            "open: items".to_owned(),
            "expect: {at: pages/items/sections/list, text: '[x] Enabled'}".to_owned(),
            format!("act: {ROWS}/it-1/row_actions/enabled"),
            "expect_command: {command: shelf.SetEnabled, input: {id: it-1, enabled: false}}"
                .to_owned(),
        ],
    );
    assert_eq!(outcome.status, Status::Passed, "{outcome:?}");
}

#[test]
fn a_row_action_whose_label_starts_with_brackets_still_matches_exactly() {
    // The row offers `e [x] Enabled · p [x] Pinned`: the action labelled `[x] Pinned` is found by
    // its whole label, and the toggle beside it by its label behind the mark.
    let outcome = run(
        "brackets",
        &[
            "open: items".to_owned(),
            format!("act: {ROWS}/it-1/row_actions/pin"),
            "expect_command: {command: shelf.Pin, input: {id: it-1}}".to_owned(),
            format!("act: {ROWS}/it-1/row_actions/enabled"),
            "expect_command: {command: shelf.SetEnabled, input: {id: it-1, enabled: false}}"
                .to_owned(),
        ],
    );
    assert_eq!(outcome.status, Status::Passed, "{outcome:?}");

    // `enable`, also labelled `Enabled` but not a toggle, is hidden on this row: the toggle's
    // `[x] Enabled` is not its hint, so the step is refused rather than pressing the toggle's key.
    let outcome = run(
        "hidden",
        &[
            "open: items".to_owned(),
            format!("act: {ROWS}/it-1/row_actions/enable"),
        ],
    );
    assert_eq!(outcome.status, Status::Failed, "{outcome:?}");
    assert_eq!(outcome.step, Some(2), "{outcome:?}");
    let message = outcome.message.unwrap_or_default();
    assert!(
        message.contains("the row does not offer Enabled"),
        "{message}"
    );
}
