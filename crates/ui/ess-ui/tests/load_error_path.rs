//! A refused node is reported at its own path, whatever names earlier nodes carry
//! (story:a-refused-node-is-reported-at-its-own-path, beyond10x/ess#723).
//!
//! When a whole document is refused, the loader walks the expanded value again and reads each
//! node as the type its position implies. A list entry's segment is its `name` or `field`: a
//! name, never a container key, so an earlier form field `{field: action}` is never read as an
//! action and never hides the node really at fault.

use ess_ui::{load_str, LoadError, Shape};

/// A document whose first page has a create form with a field named `field` (with a `label_from`
/// map below it), and whose second page's `files` section ends with `tail` (its later keys,
/// written in the section's flow mapping).
fn document(field: &str, tail: &str) -> String {
    format!(
        "format: ess-ui/1
app: desk
model: desk.system
placement_profile: fat
shells:
  app:
    regions:
      main: {{kind: page_outlet}}
      overlay: {{kind: overlay_outlet}}
navigation:
  home: notes
  sections: [{{name: all, pages: [notes, files]}}]
pages:
  notes:
    kind: list_page
    title: Notes
    sections:
      - {{name: list, component: collection, reads: {{view: desk.Notes}}, columns: [title]}}
    overlays:
      create:
        kind: drawer
        component: form
        title: Add note
        does: desk.CreateNote
        fields:
          - {{field: title, label: Title}}
          - {{field: {field}, label: Other, label_from: {{view: desk.Kinds, field: name}}}}
  files:
    kind: list_page
    title: Files
    sections:
      - {{name: list, component: collection, selection: multiple, reads: {{view: desk.Files}}, columns: [name]{tail}}}
"
    )
}

fn refused(text: &str) -> LoadError {
    match load_str(text) {
        Ok(_) => panic!("the document loads, but it has a refused key:\n{text}"),
        Err(error) => error,
    }
}

/// Asserts the refusal names `path` (or a node below it) and the key `key`.
fn reported_at(error: &LoadError, path: &str, key: &str, text: &str) {
    let at = error.path().to_string();
    assert!(
        at == path || at.starts_with(&format!("{path}/")),
        "the refusal is reported at `{at}`, not at `{path}`: {error}\n{text}"
    );
    assert!(
        error.message().contains(&format!("`{key}`")),
        "the refusal does not name `{key}`: {error}"
    );
}

const BULK_EXPORT: &str = ", bulk_actions: [{name: download, label: Download, \
                           export: {reads: desk.Bundle, as: zip, colour: red}}]";

#[test]
fn a_form_field_named_action_is_not_read_as_an_action() {
    let text = document("action", ", colour: red");
    let error = refused(&text);
    assert!(
        !error.path().to_string().contains("fields/action"),
        "the valid field `action` is reported: {error}"
    );
    reported_at(&error, "pages/files/sections/list", "colour", &text);
}

#[test]
fn a_refused_key_in_an_empty_state_names_the_empty_state() {
    let text = document(
        "action",
        ", states: {empty: {message: Nothing here, colour: red}}",
    );
    let error = refused(&text);
    reported_at(
        &error,
        "pages/files/sections/list/states/empty",
        "colour",
        &text,
    );
}

#[test]
fn a_refused_key_in_a_bulk_export_names_the_export() {
    let text = document("action", BULK_EXPORT);
    let error = refused(&text);
    reported_at(
        &error,
        "pages/files/sections/list/bulk_actions/download",
        "colour",
        &text,
    );
}

/// Every key the schema holds an action under, single or listed, as the name of an earlier form
/// field. The field's `label_from` map sits one level below the name, where a listed position
/// would be read.
#[test]
fn a_field_named_like_any_action_position_is_read_as_a_field() {
    let positions = &ess_ui::positions().actions;
    let single: Vec<&str> = positions
        .iter()
        .filter(|(_, shape)| *shape == Shape::One)
        .map(|(key, _)| key.as_str())
        .collect();
    assert!(
        single.contains(&"action") && single.len() > 1,
        "the schema's single-action positions are not the ones this case was written for: \
         {positions:?}"
    );
    let mut wrong = Vec::new();
    for (key, shape) in positions {
        let text = document(key, BULK_EXPORT);
        let error = refused(&text);
        let at = error.path().to_string();
        if !at.starts_with("pages/files/sections/list/bulk_actions/download")
            || !error.message().contains("`colour`")
        {
            wrong.push(format!("field `{key}` ({shape:?}): {error}"));
        }
    }
    assert!(
        wrong.is_empty(),
        "an earlier field's name hides the refused node:\n{}",
        wrong.join("\n")
    );
}

/// A key of a map of named nodes is a name too: a page state named `states` is a state, not a
/// section's `states`, and hides no later refusal.
#[test]
fn a_state_named_like_a_container_is_read_as_a_state() {
    let text = document("action", BULK_EXPORT).replace(
        "    title: Notes\n",
        "    title: Notes\n    state:\n      states: {type: string, class: page_state, default: ''}\n",
    );
    assert!(text.contains("      states: {type"), "{text}");
    let error = refused(&text);
    reported_at(
        &error,
        "pages/files/sections/list/bulk_actions/download",
        "colour",
        &text,
    );
    let fine = text.replace(", colour: red", "");
    if let Err(error) = load_str(&fine) {
        panic!("the document with a state `states` does not load: {error}\n{fine}");
    }
}

/// The same documents load once the refused key is gone, so each case above is refused for its
/// one key only.
#[test]
fn the_documents_load_without_the_refused_key() {
    for (key, _) in &ess_ui::positions().actions {
        let text = document(key, "");
        if let Err(error) = load_str(&text) {
            panic!("the document with a field `{key}` does not load: {error}\n{text}");
        }
    }
}
