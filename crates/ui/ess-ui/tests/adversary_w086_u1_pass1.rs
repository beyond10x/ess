//! Adversary pass 1 on story:a-value-shows-its-declared-label-beside-its-tone.
//!
//! The story adds `labels:` on a field and on a badge primitive. `labels` is also an ordinary
//! word a document may use as the *name* of a page, a state or a widget: such a name is a map key
//! under `pages`, `state` or `widgets`, not a `labels:` construct key, and a document without the
//! new construct must load as it did before the story.

const BASE: &str = r"format: ess-ui/1
app: t
model: t.system
placement_profile: fat
shells:
  app: {regions: {main: {kind: page_outlet}}}
";

fn load(text: &str) -> Result<ess_ui::Document, String> {
    let text = format!("{BASE}{text}");
    ess_ui::load_str(&text).map_err(|error| format!("{error}\n---\n{text}"))
}

/// A list page named `name` with one plain column.
fn list_page(name: &str) -> String {
    format!(
        "navigation: {{home: {name}, sections: [{{name: all, pages: [{name}]}}]}}
pages:
  {name}:
    kind: list_page
    title: Runs
    sections:
      - name: runs
        component: collection
        reads: t.Runs
        columns:
          - {{field: state, as: badge}}
"
    )
}

#[test]
fn control_a_page_named_runs_loads() {
    let document = load(&list_page("runs")).unwrap_or_else(|error| panic!("{error}"));
    assert!(document.pages.contains_key("runs"));
}

#[test]
fn a_page_named_labels_loads() {
    let document = load(&list_page("labels")).unwrap_or_else(|error| {
        panic!("a page named `labels` is not a `labels:` construct key: {error}")
    });
    assert!(document.pages.contains_key("labels"));
}

#[test]
fn a_page_state_named_labels_loads() {
    let text = "navigation: {home: p, sections: [{name: all, pages: [p]}]}
pages:
  p:
    kind: list_page
    title: Runs
    state:
      labels: {type: string, class: page_state}
    sections:
      - name: runs
        component: collection
        reads: t.Runs
        columns:
          - {field: state, as: badge}
";
    let document = load(text).unwrap_or_else(|error| {
        panic!("a page state named `labels` is not a `labels:` construct key: {error}")
    });
    assert!(document.pages["p"].state.contains_key("labels"));
}

#[test]
fn a_widget_named_labels_loads_and_expands() {
    let text = "navigation: {home: p, sections: [{name: all, pages: [p]}]}
widgets:
  labels:
    summary: The labels of a run.
    params: {}
    body:
      - {name: badge, primitive: badge, field: state}
pages:
  p:
    kind: detail_page
    title: Run
    sections:
      - name: summary
        reads: t.ById
        children:
          - {name: tags, component: labels, args: {}}
";
    let document = load(text).unwrap_or_else(|error| {
        panic!("a widget named `labels` is not a `labels:` construct key: {error}")
    });
    let expanded = document.nodes().into_iter().any(|located| {
        located.path.to_string() == "pages/p/sections/summary/children/tags/body/badge"
    });
    assert!(expanded, "the widget `labels` is expanded where it is used");
}
