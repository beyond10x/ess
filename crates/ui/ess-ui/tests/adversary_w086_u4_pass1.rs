//! Adversary pass 1 on story:a-refused-node-is-reported-at-its-own-path (wave 0.86.0, unit U4).
//!
//! `locate` now reads a map under a key `states` as a section's `SectionStates`, whatever the
//! position: the match in `classify` looks at the last container key only. A map of names the
//! schema does not list under `NodePath.syntax.containers` (`tone_maps`, `label_maps`), and a
//! record field of an inline state type, are named by the author, so a valid tone map or state
//! type named `states` is refused first and hides the node really at fault.

use ess_ui::{load_str, LoadError};

/// A document with `before` at the top level ahead of `pages`, `state` lines under the page,
/// and `tail` written into the section's flow mapping.
fn document(before: &str, state: &str, tail: &str) -> String {
    format!(
        "format: ess-ui/1
app: desk
model: desk.system
placement_profile: fat
shells:
  app:
    regions:
      main: {{kind: page_outlet}}
navigation:
  home: files
  sections: [{{name: all, pages: [files]}}]
{before}pages:
  files:
    kind: list_page
    title: Files
{state}    sections:
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

fn loads(text: &str) {
    if let Err(error) = load_str(text) {
        panic!("the document without the refused key does not load: {error}\n{text}");
    }
}

fn reported_at(error: &LoadError, path: &str, key: &str, text: &str) {
    let at = error.path().to_string();
    assert!(
        (at == path || at.starts_with(&format!("{path}/")))
            && error.message().contains(&format!("`{key}`")),
        "story:a-refused-node-is-reported-at-its-own-path: the refusal of `{key}` is reported at \
         `{at}` ({}), not at `{path}`\n{text}",
        error.message()
    );
}

const TONE_MAP_STATES: &str = "tone_maps:\n  states: {open: info, closed: success}\n";
const LABEL_MAP_ACTION: &str = "label_maps:\n  action: {create: Created, delete: Deleted}\n";
const STATE_RECORD_STATES: &str = "    state:\n      \
     filter: {type: {record: {states: {enum: [open, closed]}}}, class: page_state}\n";

/// Introduced: before the unit `classify` read no map under `states`; now a tone map named
/// `states` is read as a section's states and refused (`unknown field open`).
#[test]
fn a_tone_map_named_states_hides_no_later_refusal() {
    let text = document(TONE_MAP_STATES, "", ", colour: red");
    loads(&document(TONE_MAP_STATES, "", ""));
    reported_at(
        &refused(&text),
        "pages/files/sections/list",
        "colour",
        &text,
    );
}

/// Introduced: a page state's record type with a field `states` is read as a section's states.
#[test]
fn a_state_type_field_named_states_hides_no_later_refusal() {
    let text = document("", STATE_RECORD_STATES, ", colour: red");
    loads(&document("", STATE_RECORD_STATES, ""));
    reported_at(
        &refused(&text),
        "pages/files/sections/list",
        "colour",
        &text,
    );
}

/// Out of scope, pre-existing: a label map named `action` (a single-action position) is read
/// as an `Action`, the same class of defect for a map the schema's `NodePath` does not list.
#[test]
fn out_of_scope_a_label_map_named_action_hides_no_later_refusal() {
    let text = document(LABEL_MAP_ACTION, "", ", colour: red");
    loads(&document(LABEL_MAP_ACTION, "", ""));
    reported_at(
        &refused(&text),
        "pages/files/sections/list",
        "colour",
        &text,
    );
}

/// Probes that hold today: each refusal is named at the state it is in, for an unknown key, a
/// missing required key and a wrong type.
#[test]
fn each_state_refusal_is_named_at_its_state() {
    let cases = [
        (
            ", states: {empty: {title: No files}}",
            "pages/files/sections/list/states/empty",
            "message",
        ),
        (
            ", states: {empty: {title: [No, files], message: M}}",
            "pages/files/sections/list/states/empty",
            "title",
        ),
        (
            ", states: {failed: {retry: true, colour: red}}",
            "pages/files/sections/list/states/failed",
            "colour",
        ),
        (
            ", states: {forbidden: {colour: red}}",
            "pages/files/sections/list/states/forbidden",
            "colour",
        ),
        (
            ", states: {stale: {mark: badge, colour: red}}",
            "pages/files/sections/list/states/stale",
            "colour",
        ),
        (
            ", states: {empty: {message: M, action: {name: add, label: Add, does: desk.Add, colour: red}}}",
            "pages/files/sections/list/states/empty/action",
            "colour",
        ),
    ];
    let mut wrong = Vec::new();
    for (tail, path, key) in cases {
        let text = document("", "", tail);
        let error = refused(&text);
        let at = error.path().to_string();
        let named = error.message().contains(&format!("`{key}`"))
            || (key == "title" && error.message().contains("string"));
        if !(at == path || at.starts_with(&format!("{path}/"))) || !named {
            wrong.push(format!("{tail}: {error}"));
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

/// Probe that holds today: an export's params map with a nested map is refused at the bulk
/// action, and an empty params map loads.
#[test]
fn export_params_boundaries() {
    let tail = |params: &str| {
        format!(
            ", bulk_actions: [{{name: download, label: Download, export: {{reads: desk.Bundle, \
             as: zip, params: {params}}}}}]"
        )
    };
    let nested = document("", "", &tail("{ids: {from: selection.id}}"));
    let error = refused(&nested);
    assert!(
        error
            .path()
            .to_string()
            .starts_with("pages/files/sections/list/bulk_actions/download"),
        "{error}"
    );
    loads(&document("", "", &tail("{}")));
}
