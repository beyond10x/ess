//! beyond10x/ess#328: `choose` finds an option of a choice over a view the way the terminal
//! renderer lists it — the row field `value` names is sent, the one `label` names is shown, and
//! a row without the value field offers nothing — in a form field and in a filter bar.

use std::path::{Path, PathBuf};

use ess_ui_test::{Outcome, Status};

const DOCUMENT: &str = r"
format: ess-ui/1
app: releases
model: release.system
placement_profile: fat
shells:
  app: {regions: {main: {kind: page_outlet}}}
navigation:
  home: releases.new
  sections: [{name: all, pages: [releases.new]}]
pages:
  releases.new:
    kind: form_page
    title: New release
    state:
      repository: {type: string, class: page_state, store: url}
    sections:
      - {name: form, remove: true}
      - name: bar
        component: filter_bar
        binds: [state.repository]
        choices:
          - {name: repository, component: choice, reads: {placeholder: release.Repositories, fixture: repositories.yaml}, value: repository_id, label: location}
    header: {actions: [{name: cut, opens: cut, label: Cut}]}
    overlays:
      cut:
        kind: drawer
        component: form
        does: release.Cut
        fields:
          - field: chosen
            as: choice
            choice: {component: choice, reads: {placeholder: release.Repositories, fixture: repositories.yaml}, value: repository_id, label: location}
";

const FIXTURE: &str = r"
view: release.Repositories
rows:
  - {location: nowhere, id: wrong}
  - {repository_id: repo-1, location: example/one, id: wrong-1}
  - {repository_id: repo-2, location: example/two, id: wrong-2}
";

/// The document in a directory of its own for test `name`: tests run in parallel.
fn document(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("ess-ui-test-choice-projection")
        .join(name);
    std::fs::create_dir_all(&dir).expect("the dir is created");
    std::fs::write(dir.join("repositories.yaml"), FIXTURE).expect("the fixture is written");
    let path = dir.join("ui.yaml");
    std::fs::write(&path, DOCUMENT).expect("the document is written");
    path
}

fn run(name: &str, tests: &str) -> Outcome {
    let document = document(name);
    let text = format!(
        "format: ess-ui-test/1\ndocument: {}\ntests:\n{tests}",
        document.display()
    );
    let file = ess_ui_test::parse_str(&text, Path::new("choices.yaml"))
        .unwrap_or_else(|error| panic!("{error}\n{text}"));
    let report = ess_ui_test::execute(&document, &[file]).unwrap_or_else(|error| panic!("{error}"));
    report.tests.into_iter().next().expect("one outcome")
}

#[test]
fn a_form_choice_is_chosen_by_its_label_and_sends_its_named_value() {
    let outcome = run(
        "form",
        "- name: form\n  steps:\n\
         \x20 - open: releases.new\n\
         \x20 - act: pages/releases.new/header/actions/cut\n\
         \x20 - choose: {at: pages/releases.new/overlays/cut/fields/chosen, option: example/two}\n\
         \x20 - act: pages/releases.new/overlays/cut\n\
         \x20 - expect_command: {command: release.Cut, input: {chosen: repo-2}}\n",
    );
    assert_eq!(outcome.status, Status::Passed, "{outcome:?}");
}

#[test]
fn a_filter_bar_choice_is_chosen_by_its_label() {
    let outcome = run("bar", "- name: bar\n  steps:\n\
         \x20 - open: releases.new\n\
         \x20 - choose: {at: pages/releases.new/sections/bar/choices/repository, option: example/two}\n\
         \x20 - expect: {at: pages/releases.new/sections/bar, text: (•) example/two}\n");
    assert_eq!(outcome.status, Status::Passed, "{outcome:?}");
}
