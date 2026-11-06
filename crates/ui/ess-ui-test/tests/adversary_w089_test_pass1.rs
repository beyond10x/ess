//! Adversary pass on the wave's ride-along `ess-ui-test` pull requests: beyond10x/ess#743 (a page
//! header's help is a node a test acts on; its runner hunk was merged by hand against 0.88.0's
//! `act_on_form_or_column`) and #744 (`expect: {checked}` at a form toggle).

use std::path::{Path, PathBuf};

use ess_ui_test::{Outcome, Status};

const DOCUMENT: &str = r"format: ess-ui/1
app: probe
model: probe.system
placement_profile: fat
fixtures: {views: {settings.Current: settings.yaml, settings.Items: settings.yaml}}
shells:
  app:
    regions:
      main:    {kind: page_outlet}
      overlay: {kind: overlay_outlet}
navigation:
  home: settings.edit
  sections: [{name: all, pages: [settings.edit]}]
pages:
  settings.edit:
    kind: form_page
    title: Settings
    header:
      help: {text: Settings apply to the whole account}
    sections:
      - name: add
        component: form
        does: settings.Add
        fields:
          - {field: title, label: Title}
      - name: form
        component: form
        loads: {view: settings.Current}
        does: settings.Save
        fields:
          - {field: notify, as: toggle, label: Notify}
          - {field: archive, as: toggle, label: Archive}
      - name: prefs
        component: form
        loads: {view: settings.Current}
        does: settings.SavePrefs
        tabs:
          - {name: look, label: Look, fields: [{field: dark, as: toggle, label: Dark}]}
          - {name: more, label: More, fields: [{field: compact, as: toggle, label: Compact}]}
      - name: list
        component: collection
        reads: {view: settings.Items}
        columns: [{field: name, sortable: true}]
";

const DATA: &str = "views:
  settings.Current:
    rows:
      - {id: s-1, notify: true, archive: false, dark: true, compact: false}
  settings.Items:
    rows:
      - {id: i-2, name: Bolt}
      - {id: i-1, name: Anvil}
";

fn document(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("ess-ui-test-adversary-w089")
        .join(name);
    std::fs::create_dir_all(&dir).expect("the dir is made");
    std::fs::write(dir.join("settings.yaml"), DATA).expect("the fixture is written");
    std::fs::write(dir.join("ui.yaml"), DOCUMENT).expect("the document is written");
    dir.join("ui.yaml")
}

fn run(name: &str, steps: &[&str]) -> Outcome {
    let mut text = format!(
        "format: ess-ui-test/1\ndocument: {}\ntests:\n- name: probe\n  steps:\n",
        document(name).display()
    );
    for step in steps {
        text.push_str("  - ");
        text.push_str(step);
        text.push('\n');
    }
    let file = ess_ui_test::parse_str(&text, Path::new("adversary.yaml"))
        .unwrap_or_else(|error| panic!("{error}\n{text}"));
    let report =
        ess_ui_test::execute(&document(name), &[file]).unwrap_or_else(|error| panic!("{error}"));
    report.tests.into_iter().next().expect("one outcome")
}

const PAGE: &str = "pages/settings.edit";

/// #743's runner hunk sits before `act_on_form_or_column`: a help act, a form submit and a column
/// sort in one journey all still act.
#[test]
fn a_help_act_a_form_submit_and_a_column_sort_act_in_one_journey() {
    let outcome = run(
        "acts",
        &[
            "open: settings.edit",
            &format!("act: {PAGE}/header/help"),
            &format!(
                "expect: {{at: {PAGE}/header/help, text: Settings apply to the whole account}}"
            ),
            &format!("act: {PAGE}/sections/list/columns/name"),
            &format!("expect: {{at: {PAGE}/sections/list, rows: [i-1, i-2]}}"),
            &format!("type: {{at: {PAGE}/sections/add/fields/title, text: Fresh}}"),
            &format!("act: {PAGE}/sections/add"),
            "expect_command: {command: settings.Add, input: {title: Fresh}}",
        ],
    );
    assert_eq!(
        outcome.status,
        Status::Passed,
        "a help act and a form or column act must all still act after #743: {outcome:?}"
    );
}

/// #744 names `…/tabs/<t>/fields/<f>` as a place `checked` reads, and a toggle a test sets with
/// `choose` must read back as set.
#[test]
fn checked_reads_a_tab_toggle_and_a_toggle_a_test_set() {
    let outcome = run(
        "tabs",
        &[
            "open: settings.edit",
            &format!("expect: {{at: {PAGE}/sections/prefs/tabs/look/fields/dark, checked: true}}"),
            &format!("choose: {{at: {PAGE}/sections/form/fields/archive, option: 'true'}}"),
            &format!("expect: {{at: {PAGE}/sections/form/fields/archive, checked: true}}"),
            &format!("choose: {{at: {PAGE}/sections/form/fields/notify, option: 'false'}}"),
            &format!("expect: {{at: {PAGE}/sections/form/fields/notify, checked: false}}"),
        ],
    );
    assert_eq!(
        outcome.status,
        Status::Passed,
        "#744: `checked` must read a first-tab toggle and a toggle set by `choose`: {outcome:?}"
    );
}
