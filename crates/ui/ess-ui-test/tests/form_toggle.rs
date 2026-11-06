//! `expect: {at: <form field>, checked: true | false}` holds a form toggle's state, judged on the
//! control in both renderers: in the terminal on the field's line, in the Playwright spec on the
//! field's checkbox. The terminal draws a toggle as `[[x]]` or `[[ ]]` and the browser draws a
//! checkbox with no text, so before this check no test could state a toggle's value in both.

use std::path::{Path, PathBuf};

use ess_ui_test::{Outcome, Status};

const DOCUMENT: &str = r"format: ess-ui/1
app: probe
model: probe.system
placement_profile: fat
fixtures: {views: {settings.Current: settings.yaml}}
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
    header: {actions: [{name: more, opens: more, label: More}]}
    sections:
      - name: form
        component: form
        loads: {view: settings.Current}
        does: settings.Save
        fields:
          - {field: notify, as: toggle, label: Notify}
          - {field: archive, as: toggle, label: Archive}
          - {field: owner, label: Owner}
        groups:
          - {name: audit, label: Audit, fields: [{field: audit_on, as: toggle, label: Audit on}]}
    overlays:
      more:
        kind: drawer
        component: form
        title: More
        does: settings.SaveMore
        fields: [{field: verbose, as: toggle, label: Verbose}]
";

const DATA: &str = "views:\n  settings.Current:\n    rows:\n      - {id: s-1, notify: true, archive: false, owner: Ada, audit_on: true}\n";

fn document(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("ess-ui-test-form-toggle")
        .join(name);
    std::fs::create_dir_all(&dir).expect("the dir is made");
    std::fs::write(dir.join("settings.yaml"), DATA).expect("the fixture is written");
    std::fs::write(dir.join("ui.yaml"), DOCUMENT).expect("the document is written");
    dir.join("ui.yaml")
}

fn file(name: &str, steps: &[&str]) -> ess_ui_test::TestFile {
    let mut text = format!(
        "format: ess-ui-test/1\ndocument: {}\ntests:\n- name: toggle\n  steps:\n",
        document(name).display()
    );
    for step in steps {
        text.push_str("  - ");
        text.push_str(step);
        text.push('\n');
    }
    ess_ui_test::parse_str(&text, Path::new("form_toggle.yaml"))
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

const FORM: &str = "pages/settings.edit/sections/form";

#[test]
fn a_form_toggles_state_is_checked_in_both_renderers() {
    let steps = [
        "open: settings.edit",
        &format!("expect: {{at: {FORM}/fields/notify, checked: true}}"),
        &format!("expect: {{at: {FORM}/fields/archive, checked: false}}"),
        &format!("expect: {{at: {FORM}/groups/audit/fields/audit_on, checked: true}}"),
    ];
    let outcome = run("state", &steps);
    assert_eq!(outcome.status, Status::Passed, "{outcome:?}");
    let spec = spec("state", &steps);
    for wanted in [
        format!("await expect(page.locator('[data-ui-path=\"{FORM}/fields/notify\"] input[type=\"checkbox\"]')).toBeChecked();"),
        format!("await expect(page.locator('[data-ui-path=\"{FORM}/fields/archive\"] input[type=\"checkbox\"]')).not.toBeChecked();"),
    ] {
        assert!(spec.contains(&wanted), "{wanted}\n\n{spec}");
    }
    assert!(!spec.contains("test.fixme"), "{spec}");
}

#[test]
fn a_wrong_state_fails_naming_what_the_toggle_shows() {
    let outcome = run(
        "wrong",
        &[
            "open: settings.edit",
            &format!("expect: {{at: {FORM}/fields/archive, checked: true}}"),
        ],
    );
    assert_eq!(outcome.step, Some(2), "{outcome:?}");
    let message = outcome.message.unwrap_or_default();
    assert!(
        message.contains("expected") && message.contains("on") && message.contains("off"),
        "{message}"
    );
}

#[test]
fn a_toggle_in_an_open_overlay_form_is_checked_there() {
    let outcome = run(
        "overlay",
        &[
            "open: settings.edit",
            "act: pages/settings.edit/header/actions/more",
            "expect: {at: pages/settings.edit/overlays/more/fields/verbose, checked: false}",
        ],
    );
    assert_eq!(outcome.status, Status::Passed, "{outcome:?}");
}

#[test]
fn checked_is_refused_on_a_field_that_is_not_a_toggle() {
    let steps = [
        "open: settings.edit",
        &format!("expect: {{at: {FORM}/fields/owner, checked: true}}"),
    ];
    let outcome = run("not-toggle", &steps);
    assert_eq!(outcome.step, Some(2), "{outcome:?}");
    let message = outcome.message.unwrap_or_default();
    assert!(message.contains("a form field `as: toggle`"), "{message}");
}

#[test]
fn checked_must_be_a_boolean() {
    let text = format!(
        "format: ess-ui-test/1\ndocument: {}\ntests:\n- name: toggle\n  steps:\n  - open: settings.edit\n  - expect: {{at: {FORM}/fields/notify, checked: yes please}}\n",
        document("parse").display()
    );
    let error = ess_ui_test::parse_str(&text, Path::new("form_toggle.yaml"))
        .expect_err("a non-boolean is refused");
    assert!(
        error.to_string().contains("`checked` is true or false"),
        "{error}"
    );
}
