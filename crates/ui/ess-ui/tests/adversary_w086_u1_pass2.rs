//! Adversary pass 2 on story:a-value-shows-its-declared-label-beside-its-tone, after fix round 1.
//!
//! The fix makes `labels` a value-to-label map only in a mapping that holds `field:` as text or
//! `primitive: badge` (`holds_labels`), and lets the four walkers skip `labels` only there. These
//! cases drive that predicate from both sides: every place a field or a badge can stand (shorthand
//! columns, widget bodies, widget arguments, nested widgets, tabs, records, items, expand, page
//! kinds, `same_as` overlays, selectable columns) must still resolve `labels: <name>`, and every
//! mapping of names in which `field` and `labels` are both ordinary names must load as before.

use std::collections::BTreeMap;

use ess_ui::{Body, Document, LabelMap, NodeRef, Primitive};
use ess_yaml::Value;

const BASE: &str = r"format: ess-ui/1
app: t
model: t.system
placement_profile: fat
shells:
  app: {regions: {main: {kind: page_outlet}}}
";

const NAV_P: &str = "navigation: {home: p, sections: [{name: all, pages: [p]}]}\n";

const RUN_STATE: &str = "label_maps:\n  run_state: {in_progress: In progress, done: Done}\n";

fn load(parts: &[&str]) -> Result<Document, String> {
    let mut text = BASE.to_owned();
    for part in parts {
        text.push_str(part);
    }
    ess_ui::load_str(&text).map_err(|error| format!("{error}\n---\n{text}"))
}

fn loaded(parts: &[&str]) -> Document {
    load(parts).unwrap_or_else(|error| panic!("{error}"))
}

fn run_state() -> LabelMap {
    [("done", "Done"), ("in_progress", "In progress")]
        .iter()
        .map(|(value, label)| ((*value).to_owned(), (*label).to_owned()))
        .collect()
}

/// Every field and badge that carries `labels`, by path: its map and the name it was resolved
/// from.
fn labelled(document: &Document) -> BTreeMap<String, (LabelMap, Option<String>)> {
    let mut found = BTreeMap::new();
    for located in document.nodes() {
        let labels = match located.node {
            NodeRef::Field(field) => field.labels.as_ref(),
            NodeRef::Node(node) => match &node.body {
                Body::Primitive(Primitive::Badge(badge)) => badge.labels.as_ref(),
                _ => None,
            },
            _ => None,
        };
        if let Some(labels) = labels {
            found.insert(
                located.path.to_string(),
                (labels.map.clone(), labels.name.clone()),
            );
        }
    }
    found
}

fn all_paths(document: &Document) -> Vec<String> {
    document
        .nodes()
        .into_iter()
        .map(|located| located.path.to_string())
        .collect()
}

/// Asserts that `path` carries `run_state`, resolved from its name.
fn assert_run_state(document: &Document, path: &str) {
    let found = labelled(document);
    let Some((map, name)) = found.get(path) else {
        panic!(
            "no labels at `{path}`; labelled: {:?}; every path: {:#?}",
            found.keys().collect::<Vec<_>>(),
            all_paths(document)
        );
    };
    assert_eq!(
        map,
        &run_state(),
        "`labels: run_state` resolved at `{path}`"
    );
    assert_eq!(name.as_deref(), Some("run_state"), "at `{path}`");
}

// ── a field or badge wherever it may stand still resolves `labels: <name>` ─────────────────────

#[test]
fn a_shorthand_column_beside_a_labelled_column_loads() {
    let page = "pages:
  p:
    kind: list_page
    title: Runs
    sections:
      - name: runs
        component: collection
        reads: t.Runs
        columns:
          - state
          - {field: kind, as: badge, labels: run_state}
          - owner
";
    let document = loaded(&[NAV_P, RUN_STATE, page]);
    assert_run_state(&document, "pages/p/sections/runs/columns/kind");
    let found = labelled(&document);
    assert!(
        !found.contains_key("pages/p/sections/runs/columns/state"),
        "{found:?}"
    );
}

#[test]
fn a_badge_written_with_text_takes_labels_by_name() {
    // The schema's own badge example is written with `text:`, not `field:`; only the
    // `primitive: badge` arm of `holds_labels` reaches it.
    let page = "pages:
  p:
    kind: detail_page
    title: Run
    sections:
      - name: summary
        reads: t.ById
        children:
          - {name: state, primitive: badge, text: row.state, labels: run_state}
";
    let document = loaded(&[NAV_P, RUN_STATE, page]);
    assert_run_state(&document, "pages/p/sections/summary/children/state");
}

#[test]
fn a_field_in_a_widget_body_takes_labels_by_name() {
    let widgets = "widgets:
  run_table:
    summary: Runs as a table.
    params: {}
    body:
      - name: runs
        component: collection
        reads: t.Runs
        columns:
          - state
          - {field: kind, as: badge, labels: run_state}
";
    let page = "pages:
  p:
    kind: detail_page
    title: Run
    sections:
      - name: summary
        reads: t.ById
        children:
          - {name: table, component: run_table, args: {}}
";
    let document = loaded(&[NAV_P, RUN_STATE, widgets, page]);
    assert_run_state(&document, "widgets/run_table/body/runs/columns/kind");
    assert_run_state(
        &document,
        "pages/p/sections/summary/children/table/body/runs/columns/kind",
    );
}

#[test]
fn a_text_badge_reached_through_widget_arguments_takes_labels_both_forms() {
    let widgets = "widgets:
  status_badge:
    summary: A status as a badge.
    params:
      status: {type: string, required: true, note: the status}
      labels: {type: {ref: label_map}, required: true, note: the labels}
    body:
      - {name: badge, primitive: badge, text: args.status, labels: args.labels}
  written_badge:
    summary: A status as a badge, its labels written out.
    params:
      status: {type: string, required: true, note: the status}
      labels: {type: {map: {key: string, value: string}}, required: true, note: the labels}
    body:
      - {name: badge, primitive: badge, text: args.status, labels: args.labels}
";
    let page = "pages:
  p:
    kind: detail_page
    title: Run
    sections:
      - name: summary
        reads: t.ById
        children:
          - {name: named, component: status_badge, args: {status: row.state, labels: run_state}}
          - {name: written, component: written_badge, args: {status: row.state, labels: {in_progress: In progress, done: Done}}}
";
    let document = loaded(&[NAV_P, RUN_STATE, widgets, page]);
    assert_run_state(
        &document,
        "pages/p/sections/summary/children/named/body/badge",
    );
    let found = labelled(&document);
    let (map, name) = &found["pages/p/sections/summary/children/written/body/badge"];
    assert_eq!(map, &run_state());
    assert_eq!(name, &None, "a map passed written out names no label map");
}

#[test]
fn a_nested_widget_passes_a_label_map_name_through() {
    let widgets = "widgets:
  state_badge:
    summary: A state as a badge.
    params:
      labels: {type: {ref: label_map}, required: true, note: the labels}
    body:
      - {name: badge, primitive: badge, field: state, labels: args.labels}
  run_card:
    summary: A run as a card.
    params:
      labels: {type: {ref: label_map}, required: true, note: the labels}
    body:
      - {name: title, primitive: text, field: title}
      - {name: state, component: state_badge, args: {labels: args.labels}}
";
    let page = "pages:
  p:
    kind: detail_page
    title: Run
    sections:
      - name: summary
        reads: t.ById
        children:
          - {name: card, component: run_card, args: {labels: run_state}}
";
    let document = loaded(&[NAV_P, RUN_STATE, widgets, page]);
    assert_run_state(
        &document,
        "pages/p/sections/summary/children/card/body/state/body/badge",
    );
}

#[test]
fn labels_nested_in_tabs_items_expand_and_selectable_columns_resolve() {
    let page = "pages:
  p:
    kind: detail_page
    title: Run
    sections:
      - name: summary
        component: record
        reads: t.ById
        tabs:
          - name: main
            fields: [title, {field: state, as: badge, labels: run_state}]
        item:
          - {name: pill, primitive: badge, field: state, labels: run_state}
      - name: steps
        component: collection
        reads: t.Steps
        columns:
          binds: state.shown
          all: [title, {field: state, as: badge, labels: run_state}]
        expand:
          component: record
          fields: [{field: state, as: badge, labels: run_state}]
        item:
          - {name: pill, primitive: badge, text: row.state, labels: run_state}
    state:
      shown: {type: {list: string}, class: page_state}
";
    let document = loaded(&[NAV_P, RUN_STATE, page]);
    let found = labelled(&document);
    for (path, (map, name)) in &found {
        assert_eq!(map, &run_state(), "at `{path}`");
        assert_eq!(name.as_deref(), Some("run_state"), "at `{path}`");
    }
    assert_run_state(&document, "pages/p/sections/summary/item/pill");
    assert_run_state(&document, "pages/p/sections/steps/item/pill");
    assert_eq!(
        found.len(),
        5,
        "a tab field, two items, a selectable column and an expanded field: {:?}",
        found.keys().collect::<Vec<_>>()
    );
}

#[test]
fn a_page_kind_column_and_a_same_as_overlay_resolve_labels_by_name() {
    let kinds = "page_kinds:
  runs_list:
    extends: list_page
    sections:
      - name: runs
        component: collection
        reads: t.Runs
        columns: [title, {field: state, as: badge, labels: run_state}]
";
    let page = "navigation: {home: runs, sections: [{name: all, pages: [runs, other]}]}
pages:
  runs:
    kind: runs_list
    title: Runs
    overlays:
      show: {kind: drawer, component: record, reads: t.ById, fields: [{field: state, as: badge, labels: run_state}]}
  other:
    kind: detail_page
    title: Other
    sections:
      - name: summary
        reads: t.ById
        children:
          - {name: title, primitive: text, field: title}
    overlays:
      show: {kind: drawer, component: record, same_as: runs.show, title: Again}
";
    let document = loaded(&[RUN_STATE, kinds, page]);
    assert_run_state(&document, "pages/runs/sections/runs/columns/state");
    let found = labelled(&document);
    let overlays: Vec<&String> = found
        .keys()
        .filter(|path| path.starts_with("pages/other/overlays/show"))
        .collect();
    assert_eq!(overlays.len(), 1, "{:?}", found.keys().collect::<Vec<_>>());
    assert_run_state(&document, overlays[0]);
}

// ── `field` and `labels` as ordinary names, side by side ───────────────────────────────────────

#[test]
fn a_page_state_named_field_beside_one_named_labels_loads() {
    let page = "pages:
  p:
    kind: list_page
    title: Runs
    state:
      field: {type: string, class: page_state}
      labels: {type: string, class: page_state}
    sections:
      - name: runs
        component: collection
        reads: t.Runs
        columns: [{field: state, as: badge}]
";
    let document = load(&[NAV_P, page]).unwrap_or_else(|error| {
        panic!("page states named `field` and `labels` are not a field and its labels: {error}")
    });
    assert!(document.pages["p"].state.contains_key("field"));
    assert!(document.pages["p"].state.contains_key("labels"));
}

#[test]
fn a_section_state_named_field_beside_one_named_labels_loads() {
    let page = "pages:
  p:
    kind: list_page
    title: Runs
    sections:
      - name: runs
        component: collection
        reads: t.Runs
        state:
          field: {type: string, class: page_state}
          labels: {type: string, class: page_state}
        columns: [{field: state, as: badge}]
";
    load(&[NAV_P, page]).unwrap_or_else(|error| {
        panic!("section states named `field` and `labels` load as before: {error}")
    });
}

#[test]
fn pages_named_field_and_labels_load() {
    let pages = "navigation: {home: field, sections: [{name: all, pages: [field, labels]}]}
pages:
  field:
    kind: list_page
    title: Fields
    sections:
      - {name: rows, component: collection, reads: t.Runs, columns: [{field: state, as: badge, labels: run_state}]}
  labels:
    kind: list_page
    title: Labels
    sections:
      - {name: rows, component: collection, reads: t.Runs, columns: [{field: state, as: badge, labels: run_state}]}
";
    let document = load(&[RUN_STATE, pages])
        .unwrap_or_else(|error| panic!("pages named `field` and `labels` load as before: {error}"));
    assert_run_state(&document, "pages/field/sections/rows/columns/state");
    assert_run_state(&document, "pages/labels/sections/rows/columns/state");
}

#[test]
fn widgets_named_field_and_labels_load_and_expand() {
    let widgets = "widgets:
  field:
    summary: A state badge.
    params: {}
    body:
      - {name: badge, primitive: badge, field: state, labels: run_state}
  labels:
    summary: A text badge.
    params: {}
    body:
      - {name: badge, primitive: badge, text: row.state, labels: run_state}
";
    let page = "pages:
  p:
    kind: detail_page
    title: Run
    sections:
      - name: summary
        reads: t.ById
        children:
          - {name: one, component: field, args: {}}
          - {name: two, component: labels, args: {}}
";
    let document = load(&[NAV_P, RUN_STATE, widgets, page]).unwrap_or_else(|error| {
        panic!("widgets named `field` and `labels` load as before: {error}")
    });
    assert_run_state(&document, "widgets/field/body/badge");
    assert_run_state(&document, "widgets/labels/body/badge");
    assert_run_state(
        &document,
        "pages/p/sections/summary/children/one/body/badge",
    );
    assert_run_state(
        &document,
        "pages/p/sections/summary/children/two/body/badge",
    );
}

#[test]
fn overlays_named_field_and_labels_load() {
    let page = "pages:
  p:
    kind: list_page
    title: Runs
    sections:
      - {name: runs, component: collection, reads: t.Runs, columns: [{field: state, as: badge}]}
    overlays:
      field: {kind: drawer, component: record, reads: t.ById, fields: [title]}
      labels: {kind: drawer, component: record, reads: t.ById, fields: [{field: state, as: badge, labels: run_state}]}
";
    let document = load(&[NAV_P, RUN_STATE, page]).unwrap_or_else(|error| {
        panic!("overlays named `field` and `labels` load as before: {error}")
    });
    let found = labelled(&document);
    assert_eq!(found.len(), 1, "{:?}", found.keys().collect::<Vec<_>>());
}

// ── the walkers leave a label map's keys alone, and tones still resolve ────────────────────────

/// A label map whose values are spelled like document structure: a widget name under
/// `component`, a view under `reads`, a string under `tone_by`. No walker may read it as
/// structure, on a field, on a field-text badge, on a text badge, or through a widget argument.
#[test]
fn a_label_map_keyed_by_structural_words_stays_as_written() {
    let words = "{component: card, reads: t.Runs, tone_by: Toned, children: Kids, columns: Cols, \
                 args: Args, body: Body, field: Field, primitive: badge, labels: Labels, type: Type}";
    let widgets = format!(
        "widgets:
  card:
    summary: A card.
    params: {{}}
    body:
      - {{name: t, primitive: text, field: title}}
  passed:
    summary: A badge whose labels are passed.
    params:
      labels: {{type: {{map: {{key: string, value: string}}}}, required: true, note: the labels}}
    body:
      - {{name: badge, primitive: badge, text: row.state, labels: args.labels}}
"
    );
    let page = format!(
        "pages:
  p:
    kind: detail_page
    title: Run
    sections:
      - name: summary
        component: record
        reads: t.ById
        fields: [{{field: state, as: badge, labels: {words}}}]
        item:
          - {{name: by_field, primitive: badge, field: state, labels: {words}}}
          - {{name: by_text, primitive: badge, text: row.state, labels: {words}}}
          - {{name: passed, component: passed, args: {{labels: {words}}}}}
"
    );
    let document = loaded(&[NAV_P, &widgets, &page]);
    let expected: LabelMap = [
        ("args", "Args"),
        ("body", "Body"),
        ("children", "Kids"),
        ("columns", "Cols"),
        ("component", "card"),
        ("field", "Field"),
        ("labels", "Labels"),
        ("primitive", "badge"),
        ("reads", "t.Runs"),
        ("tone_by", "Toned"),
        ("type", "Type"),
    ]
    .iter()
    .map(|(value, label)| ((*value).to_owned(), (*label).to_owned()))
    .collect();
    let found = labelled(&document);
    for path in [
        "pages/p/sections/summary/fields/state",
        "pages/p/sections/summary/item/by_field",
        "pages/p/sections/summary/item/by_text",
        "pages/p/sections/summary/item/passed/body/badge",
    ] {
        let Some((map, _)) = found.get(path) else {
            panic!(
                "no labels at `{path}`: {:?}",
                found.keys().collect::<Vec<_>>()
            );
        };
        assert_eq!(map, &expected, "at `{path}`");
    }
}

#[test]
fn a_badge_with_tones_and_labels_by_name_resolves_both_wherever_it_stands() {
    let maps = "tone_maps:\n  run_tone: {in_progress: info, done: success}\n";
    let badge = "{name: state, primitive: badge, field: state, tone_by: {value: row.state, tones: run_tone}, labels: run_state}";
    let text_badge = "{name: text, primitive: badge, text: row.state, tone_by: {value: row.state, tones: run_tone}, labels: run_state}";
    let widgets = format!(
        "widgets:
  labels:
    summary: A toned badge.
    params: {{}}
    body:
      - {badge}
      - {text_badge}
"
    );
    let pages = format!(
        "navigation: {{home: labels, sections: [{{name: all, pages: [labels, field]}}]}}
pages:
  labels:
    kind: detail_page
    title: Labels
    sections:
      - name: summary
        reads: t.ById
        children:
          - {badge}
          - {text_badge}
          - {{name: used, component: labels, args: {{}}}}
  field:
    kind: detail_page
    title: Field
    sections:
      - name: summary
        reads: t.ById
        children:
          - {badge}
"
    );
    let document = loaded(&[RUN_STATE, maps, &widgets, &pages]);
    let mut toned = 0;
    for located in document.nodes() {
        let NodeRef::Node(node) = located.node else {
            continue;
        };
        let Body::Primitive(Primitive::Badge(badge)) = &node.body else {
            continue;
        };
        let path = located.path.to_string();
        let tone_by = badge
            .tone_by
            .as_ref()
            .unwrap_or_else(|| panic!("a tone_by at `{path}`"));
        assert_eq!(tone_by.tones.as_deref(), Some("run_tone"), "at `{path}`");
        assert_eq!(
            tone_by.map.get("in_progress"),
            Some(&Value::from("info")),
            "`tones: run_tone` resolved at `{path}`: {:?}",
            tone_by.map
        );
        assert_run_state(&document, &path);
        toned += 1;
    }
    assert_eq!(
        toned, 7,
        "two in the widget declaration, two on page `labels`, two in its use, one on page `field`"
    );
}
