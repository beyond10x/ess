//! Adversary pass 1 on story:a-toggle-shows-the-row-state-it-flips, library side: the shapes of
//! a `bind` entry that is `not <expr>` as a whole (the filter grammar parses it as one negation)
//! and still names no state.

use ess_ui::{Action, Document, NodeRef, ToggleSource};

fn document(actions: &str) -> String {
    format!(
        r"
format: ess-ui/1
app: flags
model: flags.system
placement_profile: fat
shells:
  app: {{regions: {{main: {{kind: page_outlet}}}}}}
navigation:
  home: flags.list
  sections: [{{name: all, pages: [flags.list]}}]
pages:
  flags.list:
    kind: detail_page
    title: Flags
    sections:
      - name: list
        component: collection
        reads: {{view: flags.All}}
        columns: [title]
        row_actions:
{actions}
"
    )
}

fn load(text: &str) -> Document {
    ess_ui::load_str(text).unwrap_or_else(|error| panic!("the document loads: {error}\n{text}"))
}

fn action<'a>(document: &'a Document, name: &str) -> &'a Action {
    let path = format!("pages/flags.list/sections/list/row_actions/{name}");
    document
        .nodes()
        .into_iter()
        .find_map(|located| match located.node {
            NodeRef::Action(action) if located.path.to_string() == path => Some(action),
            _ => None,
        })
        .unwrap_or_else(|| panic!("no action at {path}"))
}

/// `not(row.enabled)` is one negation of a path for the filter grammar, the React expression
/// grammar and the check's expression grammar; the toggle flips it, so it has a state.
#[test]
fn a_not_bind_written_without_a_space_names_its_state() {
    let document = load(&document(
        "          - {name: enabled, as: toggle, does: flags.SetFlag, bind: {id: row.id, enabled: not(row.enabled)}}",
    ));
    let state = ess_ui::toggle_state(action(&document, "enabled"));
    assert_eq!(
        state.as_ref().map(|state| &state.source),
        Some(&ToggleSource::Flipped {
            input: "enabled".to_owned()
        }),
        "`enabled: not(row.enabled)` is one `not <expr>` entry, so the toggle has a state: {state:?}"
    );
}

/// `(not row.enabled)` is the same single negation, parenthesised as a whole.
#[test]
fn a_parenthesised_not_bind_names_its_state() {
    let document = load(&document(
        "          - {name: enabled, as: toggle, does: flags.SetFlag, bind: {id: row.id, enabled: (not row.enabled)}}",
    ));
    let state = ess_ui::toggle_state(action(&document, "enabled"));
    assert_eq!(
        state.as_ref().map(|state| &state.source),
        Some(&ToggleSource::Flipped {
            input: "enabled".to_owned()
        }),
        "`enabled: (not row.enabled)` is one `not <expr>` entry as a whole: {state:?}"
    );
}

/// Extra spaces around the operand keep the state the plain form names.
#[test]
fn a_not_bind_with_extra_spaces_names_the_same_state() {
    let document = load(&document(
        "          - {name: enabled, as: toggle, does: flags.SetFlag, bind: {id: row.id, enabled: '  not    row.enabled  '}}",
    ));
    let state = ess_ui::toggle_state(action(&document, "enabled")).expect("a state");
    assert_eq!(state.expr.0, "row.enabled");
}

/// `not not row.enabled` sends `row.enabled`, so the state it flips is `not row.enabled`.
#[test]
fn a_double_negation_names_the_negated_state() {
    let document = load(&document(
        "          - {name: enabled, as: toggle, does: flags.SetFlag, bind: {id: row.id, enabled: not not row.enabled}}",
    ));
    let state = ess_ui::toggle_state(action(&document, "enabled")).expect("a state");
    assert_eq!(state.expr.0, "not row.enabled");
}
