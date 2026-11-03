//! Value labels (story:a-value-shows-its-declared-label-beside-its-tone): the words a document
//! declares for a value, on a field (`labels:`) and on a `badge` primitive, written out or named
//! from the document's `label_maps`, which the loader resolves as it resolves `tone_by.tones`.

use std::collections::BTreeMap;

use ess_ui::{Badge, Body, Document, Field, LabelMap, Labels, NodeRef, Primitive};
use ess_yaml::Value;

const BASE: &str = r"format: ess-ui/1
app: t
model: t.system
placement_profile: fat
shells:
  app: {regions: {main: {kind: page_outlet}}}
navigation: {home: p, sections: [{name: all, pages: [p]}]}
";

/// A collection of runs whose columns are `columns`, written one per line.
fn columns(columns: &[&str]) -> String {
    let mut text = "pages:\n  p:\n    kind: list_page\n    title: Runs\n    sections:\n      \
                    - name: runs\n        component: collection\n        reads: t.Runs\n        \
                    columns:\n"
        .to_owned();
    for column in columns {
        text.push_str("          - ");
        text.push_str(column);
        text.push('\n');
    }
    text
}

fn load(parts: &[&str]) -> Document {
    let mut text = BASE.to_owned();
    for part in parts {
        text.push_str(part);
    }
    ess_ui::load_str(&text).unwrap_or_else(|error| panic!("{error}\n{text}"))
}

fn map(entries: &[(&str, &str)]) -> LabelMap {
    entries
        .iter()
        .map(|(value, label)| ((*value).to_owned(), (*label).to_owned()))
        .collect::<BTreeMap<_, _>>()
}

fn field<'a>(document: &'a Document, path: &str) -> &'a Field {
    let located = document
        .nodes()
        .into_iter()
        .find(|located| located.path.to_string() == path)
        .unwrap_or_else(|| panic!("no node at `{path}`"));
    match located.node {
        NodeRef::Field(field) => field,
        other => panic!("not a field at `{path}`: {other:?}"),
    }
}

fn badge<'a>(document: &'a Document, path: &str) -> &'a Badge {
    let located = document
        .nodes()
        .into_iter()
        .find(|located| located.path.to_string() == path)
        .unwrap_or_else(|| panic!("no node at `{path}`"));
    match located.node {
        NodeRef::Node(node) => match &node.body {
            Body::Primitive(Primitive::Badge(badge)) => badge,
            other => panic!("not a badge at `{path}`: {other:?}"),
        },
        other => panic!("not a node at `{path}`: {other:?}"),
    }
}

fn labels(labels: Option<&Labels>) -> &Labels {
    labels.expect("the node carries labels")
}

#[test]
fn a_field_takes_labels_written_out() {
    let document = load(&[&columns(&[
        "{field: state, as: badge, labels: {in_progress: In progress}}",
    ])]);
    let state = field(&document, "pages/p/sections/runs/columns/state");
    let labels = labels(state.labels.as_ref());
    assert_eq!(labels.map, map(&[("in_progress", "In progress")]));
    assert_eq!(labels.name, None, "a map written out names no label map");
    assert_eq!(
        labels.label(&Value::from("in_progress")),
        Some("In progress")
    );
}

#[test]
fn a_field_takes_labels_by_name() {
    let document = load(&[
        "label_maps:\n  run_state: {in_progress: In progress, no_answer: No answer}\n",
        &columns(&["{field: state, as: badge, labels: run_state}"]),
    ]);
    let declared = map(&[("in_progress", "In progress"), ("no_answer", "No answer")]);
    assert_eq!(document.label_maps["run_state"], declared);
    let state = field(&document, "pages/p/sections/runs/columns/state");
    let labels = labels(state.labels.as_ref());
    assert_eq!(
        labels.map, declared,
        "`labels: run_state` is resolved to `label_maps.run_state` by the loader"
    );
    assert_eq!(labels.name.as_deref(), Some("run_state"));
    assert_eq!(labels.label(&Value::from("no_answer")), Some("No answer"));
}

#[test]
fn a_badge_takes_labels() {
    let page = "pages:\n  p:\n    kind: detail_page\n    title: P\n    sections:\n      \
                - name: summary\n        reads: t.ById\n        children:\n          \
                - {name: written, primitive: badge, field: state, labels: {in_progress: In progress}}\n          \
                - {name: named, primitive: badge, field: state, labels: run_state}\n";
    let document = load(&["label_maps:\n  run_state: {in_progress: Running}\n", page]);
    let written = labels(
        badge(&document, "pages/p/sections/summary/children/written")
            .labels
            .as_ref(),
    );
    assert_eq!(written.map, map(&[("in_progress", "In progress")]));
    assert_eq!(written.name, None);
    let named = labels(
        badge(&document, "pages/p/sections/summary/children/named")
            .labels
            .as_ref(),
    );
    assert_eq!(named.map, map(&[("in_progress", "Running")]));
    assert_eq!(named.name.as_deref(), Some("run_state"));
}

/// A widget parameter typed `{ref: label_map}` carries a name, resolved at each use, as a
/// parameter typed `{ref: tone_map}` is; the declaration keeps the unbound argument.
#[test]
fn a_widget_argument_names_a_label_map() {
    let widgets = "widgets:\n  state_badge:\n    summary: A state as a badge.\n    params:\n      \
                   labels: {type: {ref: label_map}, required: true, note: the labels}\n    \
                   body:\n      - {name: badge, primitive: badge, field: state, labels: args.labels}\n";
    let page = "pages:\n  p:\n    kind: detail_page\n    title: P\n    sections:\n      \
                - name: summary\n        reads: t.ById\n        children:\n          \
                - {name: state, component: state_badge, args: {labels: run_state}}\n";
    let document = load(&[
        "label_maps:\n  run_state: {in_progress: In progress}\n",
        widgets,
        page,
    ]);
    let used = labels(
        badge(
            &document,
            "pages/p/sections/summary/children/state/body/badge",
        )
        .labels
        .as_ref(),
    );
    assert_eq!(used.map, map(&[("in_progress", "In progress")]));
    assert_eq!(used.name.as_deref(), Some("run_state"));
    let declared = labels(
        badge(&document, "widgets/state_badge/body/badge")
            .labels
            .as_ref(),
    );
    assert!(declared.map.is_empty(), "{declared:?}");
    assert_eq!(declared.name.as_deref(), Some("args.labels"));
}

#[test]
fn boolean_values_are_keyed_true_and_false() {
    let document = load(&[&columns(&[
        r#"{field: enabled, as: badge, labels: {"true": Active, "false": Inactive}}"#,
    ])]);
    let enabled = labels(
        field(&document, "pages/p/sections/runs/columns/enabled")
            .labels
            .as_ref(),
    );
    assert_eq!(
        enabled.map,
        map(&[("false", "Inactive"), ("true", "Active")])
    );
    assert_eq!(enabled.label(&Value::Bool(true)), Some("Active"));
    assert_eq!(enabled.label(&Value::Bool(false)), Some("Inactive"));
    assert_eq!(
        enabled.label(&Value::Null),
        None,
        "a value the map does not name has no label"
    );
}
