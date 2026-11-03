//! Adversary pass 1 on story:a-toggle-shows-the-row-state-it-flips, terminal side. The story's
//! Decisions: one library function names the state expression and both renderers evaluate it,
//! "so the two cannot disagree". A bind entry `not (row.enabled)` is one negation; the library
//! names the state `(row.enabled)`, which the React runtime evaluates to the row value.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use ess_ui_tui::{App, DataAdapter, Options, ReadRequest, ReadResult};
use ess_yaml::Value;

fn document(row_actions: &str) -> String {
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
      paused: {{type: boolean, class: component_state, default: true}}
    sections:
      - name: list
        component: collection
        reads: {{view: flags.All}}
        columns: [title]
{row_actions}
"
    )
}

const ROWS: &str = "[{id: f-1, title: Alpha, enabled: true, locked: false}, \
                    {id: f-2, title: Beta, enabled: false, locked: true}]";

struct Table {
    rows: Vec<Value>,
}

impl DataAdapter for Table {
    fn read(&self, _: &ReadRequest) -> Result<ReadResult, String> {
        Ok(ReadResult {
            rows: self.rows.clone(),
            total: None,
        })
    }
    fn run(&mut self, _: &str, _: &BTreeMap<String, Value>) -> ess_ui::binding::Answer {
        ess_ui::binding::Answer::Accepted
    }
    fn load_state(&self, _: &str) -> Option<Value> {
        None
    }
    fn store_state(&mut self, _: &str, _: Value) {}
}

fn dir(test: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("ess-ui-tui-adversary-w086-u2")
        .join(test);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("a stale dir is removed");
    }
    std::fs::create_dir_all(&dir).expect("the dir is created");
    dir
}

fn app(test: &str, row_actions: &str) -> App {
    let yaml = document(row_actions);
    let document = ess_ui::load_str(&yaml).unwrap_or_else(|error| panic!("{error}\n{yaml}"));
    let adapter = Box::new(Table {
        rows: ess_yaml::from_str(ROWS).expect("the rows parse"),
    });
    App::with_adapter(
        document,
        adapter,
        Vec::new(),
        Options::new(dir(test).join("state")),
    )
    .unwrap_or_else(|error| panic!("{error}"))
}

fn hint(screen: &str) -> &str {
    screen
        .lines()
        .find(|line| line.contains("Enabled"))
        .unwrap_or_else(|| panic!("no hint offers the toggle:\n{screen}"))
}

#[test]
fn a_parenthesised_not_bind_shows_the_row_state() {
    let mut app = app(
        "parenthesised",
        "        row_actions:\n          - {name: enabled, as: toggle, label: Enabled, does: flags.SetFlag, bind: {id: row.id, enabled: not (row.enabled)}}",
    );
    let state = ess_ui::toggle_state(&document_action(
        "        row_actions:\n          - {name: enabled, as: toggle, label: Enabled, does: flags.SetFlag, bind: {id: row.id, enabled: not (row.enabled)}}",
    ))
    .expect("the library names a state, so no warning is raised");
    assert_eq!(
        state.expr.0, "row.enabled",
        "story:a-toggle-shows-the-row-state-it-flips fix round 1 (F2/F3): the state is the \
         negation's operand in canonical text, without redundant parentheses"
    );
    app.focus_section("list");
    let first = app.render_text(120, 24);
    assert!(
        hint(&first).contains("[x] Enabled"),
        "the cursor row is on and the library named its state, yet the terminal shows:\n{first}"
    );
    app.keys("j");
    let second = app.render_text(120, 24);
    assert!(hint(&second).contains("[ ] Enabled"), "{second}");
}

/// The `sets:` path keeps working in the terminal: the key it sets, `state.paused`, defaults on.
#[test]
fn a_sets_row_toggle_shows_its_state_key() {
    let mut app = app(
        "sets",
        "        row_actions:\n          - {name: pause, as: toggle, label: Enabled, sets: {state.paused: toggle}}",
    );
    app.focus_section("list");
    let first = app.render_text(120, 24);
    assert!(hint(&first).contains("[x] Enabled"), "{first}");
}

/// An explicit `checked:` wins in the terminal, as in React.
#[test]
fn an_explicit_checked_wins_in_the_terminal() {
    let mut app = app(
        "checked",
        "        row_actions:\n          - {name: enabled, as: toggle, label: Enabled, does: flags.SetFlag, bind: {id: row.id, enabled: not row.enabled}, checked: row.locked}",
    );
    app.focus_section("list");
    let first = app.render_text(120, 24);
    assert!(hint(&first).contains("[ ] Enabled"), "{first}");
    app.keys("j");
    let second = app.render_text(120, 24);
    assert!(hint(&second).contains("[x] Enabled"), "{second}");
}

fn document_action(row_actions: &str) -> ess_ui::Action {
    let yaml = document(row_actions);
    let document = ess_ui::load_str(&yaml).unwrap_or_else(|error| panic!("{error}\n{yaml}"));
    document
        .nodes()
        .into_iter()
        .find_map(|located| match located.node {
            ess_ui::NodeRef::Action(action) if action.name == "enabled" => Some(action.clone()),
            _ => None,
        })
        .expect("the toggle is located")
}
