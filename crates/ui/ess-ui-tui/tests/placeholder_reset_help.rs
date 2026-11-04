//! The terminal draws the three presentation keys of
//! story:choices-filter-bars-and-forms-render-what-they-declare where it draws their node: a
//! choice with nothing chosen shows its `placeholder`, a filter bar's reset shows its `label`, and
//! `help` shows on a form group, a form field and a column.

use std::path::{Path, PathBuf};

use ess_ui_tui::{App, Options};

const DOC: &str = r"
format: ess-ui/1
app: shelf
model: shelf.system
placement_profile: fat
fixtures: {views: {shelf.Items: data.yaml}}
shells:
  app: {regions: {main: {kind: page_outlet}}}
navigation:
  home: items
  sections: [{name: all, pages: [items, settings]}]
pages:
  items:
    kind: detail_page
    title: Items
    state:
      stage: {type: string, class: page_state, store: memory}
    sections:
      - name: filters
        component: filter_bar
        binds: [state.stage]
        choices:
          - {name: stage, component: choice, options: [open, won], placeholder: Any stage}
        reset: {label: Clear filters}
      - name: list
        component: collection
        reads: {view: shelf.Items}
        columns:
          - {field: title, help: {text: The name shown on the shelf}}
  settings:
    kind: settings_page
    title: Settings
    sections:
      - name: settings
        component: form
        does: shelf.Configure
        fields:
          - {field: kind, as: choice, choice: {component: choice, options: [book, box], placeholder: Pick a kind}}
        groups:
          - name: profile
            label: Profile
            help: {text: Who sees the shelf}
            fields:
              - {field: owner, help: {text: The person who answers for it}}
";

const DATA: &str = r"
views:
  shelf.Items:
    rows:
      - {id: i-1, title: Alpha}
";

fn dir(test: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("ess-ui-tui-placeholder-reset-help")
        .join(test);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("a stale dir is removed");
    }
    std::fs::create_dir_all(&dir).expect("the dir is created");
    std::fs::write(dir.join("data.yaml"), DATA).expect("the fixture is written");
    dir
}

fn screen(test: &str, page: &str) -> String {
    let base = dir(test);
    let mut app = App::from_text(DOC, &base, Options::new(base.join("state")))
        .unwrap_or_else(|error| panic!("{error}"));
    app.open_page(page, &[]);
    app.render_text(140, 40)
}

#[test]
fn an_unchosen_choice_shows_its_placeholder() {
    let items = screen("bar_placeholder", "items");
    assert!(
        items.contains("Any stage"),
        "the filter choice shows its placeholder:\n{items}"
    );
    let settings = screen("field_placeholder", "settings");
    assert!(
        settings.contains("Pick a kind"),
        "the field choice shows its placeholder:\n{settings}"
    );
}

#[test]
fn a_reset_shows_its_label() {
    let items = screen("reset", "items");
    assert!(
        items.contains("Clear filters"),
        "the reset shows its label:\n{items}"
    );
}

#[test]
fn help_shows_on_a_group_a_field_and_a_column() {
    let items = screen("column_help", "items");
    assert!(
        items.contains("The name shown on the shelf"),
        "the column shows its help:\n{items}"
    );
    let settings = screen("form_help", "settings");
    for help in ["Who sees the shelf", "The person who answers for it"] {
        assert!(
            settings.contains(help),
            "the form shows `{help}`:\n{settings}"
        );
    }
}
