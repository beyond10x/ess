//! Adversary pass 2 on story:a-toggle-shows-the-row-state-it-flips, library side, against fix
//! round 1: the state is the negation's operand written back by `canonical`. The renderers read
//! that text with the display grammar (`expr.ts.tmpl` `evaluate`, the terminal's `expr::eval`),
//! where a bare root name (`draft`, `state`, `row`, ...) is a path and a dotted word whose first
//! segment is not a root is literal text.

use ess_ui::{Action, Document, NodeRef, ToggleSource};

fn document(actions: &str) -> String {
    format!(
        r"
format: ess-ui/1
app: posts
model: posts.system
placement_profile: fat
shells:
  app: {{regions: {{main: {{kind: page_outlet}}}}}}
navigation:
  home: posts.list
  sections: [{{name: all, pages: [posts.list]}}]
pages:
  posts.list:
    kind: detail_page
    title: Posts
    state:
      paused: {{type: boolean, class: component_state, default: true}}
    sections:
      - name: list
        component: collection
        reads: {{view: posts.All}}
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
    let path = format!("pages/posts.list/sections/list/row_actions/{name}");
    document
        .nodes()
        .into_iter()
        .find_map(|located| match located.node {
            NodeRef::Action(action) if located.path.to_string() == path => Some(action),
            _ => None,
        })
        .unwrap_or_else(|| panic!("no action at {path}"))
}

/// The operand compares with the text `'draft'`. Written back bare, `row.stage == draft` compares
/// with the `draft` root in the React runtime (`ROOTS` in `expr.ts.tmpl` lists `draft`), so the
/// state no longer means the operand the toggle flips.
#[test]
fn a_quoted_word_that_spells_a_root_keeps_its_quotes_in_the_state() {
    let document = load(&document(
        r#"          - {name: draft, as: toggle, label: Draft, does: posts.SetDraft, bind: {id: row.id, draft: "not (row.stage == 'draft')"}}"#,
    ));
    let state = ess_ui::toggle_state(action(&document, "draft")).expect("a state");
    assert_eq!(
        state.expr.0, "row.stage == 'draft'",
        "the state must compare with the text draft, as the bind's operand does"
    );
}

/// The same for every root the React runtime knows: a quoted root name is text, a bare one a path.
#[test]
fn no_quoted_root_name_loses_its_quotes() {
    let roots = [
        "state",
        "params",
        "args",
        "row",
        "selection",
        "draft",
        "section",
        "channel",
        "shell",
        "url",
        "actor",
        "widget",
    ];
    let mut bare = Vec::new();
    for root in roots {
        let document = load(&document(&format!(
            r#"          - {{name: kind, as: toggle, label: Kind, does: posts.SetKind, bind: {{id: row.id, on: "not (row.kind == '{root}')"}}}}"#,
        )));
        let state = ess_ui::toggle_state(action(&document, "kind")).expect("a state");
        if state.expr.0 == format!("row.kind == {root}") {
            bare.push(root);
        }
    }
    assert_eq!(
        bare,
        Vec::<&str>::new(),
        "these quoted root names were written back bare, as paths"
    );
}

/// `item.enabled` reads no root: both renderers evaluate it as the literal text `item.enabled`,
/// truthy for every row, and the module's own rule is that a `not` over literal text names no
/// state (so `toggle_state_unknown` warns).
#[test]
fn a_not_over_a_dotted_word_that_reads_no_root_names_no_state() {
    let document = load(&document(
        "          - {name: enabled, as: toggle, label: Enabled, does: posts.SetEnabled, bind: {id: row.id, enabled: not item.enabled}}",
    ));
    let state = ess_ui::toggle_state(action(&document, "enabled"));
    assert_eq!(
        state, None,
        "`not item.enabled` is literal text to both renderers, so it names no state"
    );
}

/// `checked:` wins over a `sets` key and over a `not` bind entry alike.
#[test]
fn checked_wins_over_sets_and_over_a_flipped_bind() {
    let document = load(&document(
        "          - {name: a, as: toggle, label: A, sets: {state.paused: toggle}, checked: row.locked}\n          - {name: b, as: toggle, label: B, does: posts.SetB, bind: {id: row.id, on: not row.on}, checked: row.locked}",
    ));
    for name in ["a", "b"] {
        let state = ess_ui::toggle_state(action(&document, name)).expect("a state");
        assert_eq!(state.source, ToggleSource::Checked, "{name}");
        assert_eq!(state.expr.0, "row.locked", "{name}");
    }
}

/// Precedence under `not`: the written-back operand groups as the bind's operand does.
#[test]
fn a_mixed_and_or_operand_keeps_its_grouping() {
    for (bind, state) in [
        ("not (row.a or row.b and row.c)", "row.a or row.b and row.c"),
        (
            "not ((row.a or row.b) and row.c)",
            "(row.a or row.b) and row.c",
        ),
        ("not (row.a and not row.b)", "row.a and not row.b"),
        ("not (not row.a and row.b)", "not row.a and row.b"),
        (
            "not (row.n in [1, 'x y', row.m])",
            "row.n in [1, 'x y', row.m]",
        ),
    ] {
        let document = load(&document(&format!(
            r#"          - {{name: t, as: toggle, label: T, does: posts.SetT, bind: {{id: row.id, on: "{bind}"}}}}"#,
        )));
        let got = ess_ui::toggle_state(action(&document, "t")).expect("a state");
        assert_eq!(got.expr.0, state, "{bind}");
    }
}
