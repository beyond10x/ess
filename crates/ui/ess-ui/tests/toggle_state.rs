//! The state a toggle action shows, computed once in this crate and handed to both renderers: an
//! explicit `checked:` expression, else for a toggle that runs a command the one `bind` entry
//! written `not <expr>` (the state is `<expr>`), else for a toggle that sets UI state the key it
//! sets. No row field named like the action is read.

use ess_ui::{Action, ActionAs, Document, Expr, NodeRef, ToggleSource};

/// A list of flags whose row actions are written as `actions`.
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
    state:
      paused: {{type: boolean, class: component_state}}
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

/// The row action named `name`.
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

fn expr(text: &str) -> Expr {
    Expr(text.to_owned())
}

#[test]
fn an_action_takes_a_checked_expression() {
    let document = load(&document(
        "          - {name: enabled, as: toggle, does: flags.SetFlag, bind: {id: row.id, on: row.wanted}, checked: row.enabled}",
    ));
    let enabled = action(&document, "enabled");
    assert_eq!(enabled.action_as, Some(ActionAs::Toggle));
    assert_eq!(
        enabled.checked,
        Some(expr("row.enabled")),
        "the typed action carries `checked`"
    );
    let state = ess_ui::toggle_state(enabled).expect("a toggle with `checked` has a state");
    assert_eq!(state.expr, expr("row.enabled"));
    assert_eq!(state.source, ToggleSource::Checked);
}

#[test]
fn a_command_toggle_derives_its_state_from_the_bind_it_flips() {
    let document = load(&document(concat!(
        "          - {name: enabled, as: toggle, does: flags.SetFlag, bind: {id: row.id, enabled: not row.enabled}}\n",
        "          - {name: both, as: toggle, does: flags.SetFlag, bind: {id: row.id, enabled: not row.enabled}, checked: row.locked}\n",
        "          - {name: twice, as: toggle, does: flags.SetFlag, bind: {a: not row.a, b: not row.b}}\n",
        "          - {name: joined, as: toggle, does: flags.SetFlag, bind: {a: not row.a and row.b}}\n",
        "          - {name: prose, as: toggle, does: flags.SetFlag, bind: {note: not yet sent}}\n",
        "          - {name: flagged, as: toggle, does: flags.SetFlag, bind: {id: row.id}}\n",
        "          - {name: flip, does: flags.SetFlag, bind: {enabled: not row.enabled}}",
    )));

    let state = ess_ui::toggle_state(action(&document, "enabled"))
        .expect("a toggle flipping one bound value has a state");
    assert_eq!(
        state.expr,
        expr("row.enabled"),
        "`enabled: not row.enabled` gives the state `row.enabled`"
    );
    assert_eq!(
        state.source,
        ToggleSource::Flipped {
            input: "enabled".to_owned()
        }
    );

    let both = ess_ui::toggle_state(action(&document, "both")).expect("a state");
    assert_eq!(
        (both.expr, both.source),
        (expr("row.locked"), ToggleSource::Checked),
        "`checked:` wins over the derived state"
    );

    for name in ["twice", "joined", "prose"] {
        assert_eq!(
            ess_ui::toggle_state(action(&document, name)),
            None,
            "`{name}`: not exactly one entry that is `not <expr>` as a whole, so no state"
        );
    }
    assert_eq!(
        ess_ui::toggle_state(action(&document, "flagged")),
        None,
        "no fallback on a row field named like the action"
    );
    assert_eq!(
        ess_ui::toggle_state(action(&document, "flip")),
        None,
        "an action that is not a toggle shows no state"
    );
}

#[test]
fn a_sets_toggle_keeps_its_state() {
    let document = load(&document(
        "          - {name: pause, as: toggle, sets: {state.paused: toggle}}",
    ));
    let pause = action(&document, "pause");
    let state = ess_ui::toggle_state(pause).expect("a `sets` toggle has a state");
    assert_eq!(
        state.expr,
        expr("state.paused"),
        "the state is the key the toggle sets, as before"
    );
    assert_eq!(
        state.source,
        ToggleSource::Sets {
            key: "state.paused".to_owned()
        }
    );
}
