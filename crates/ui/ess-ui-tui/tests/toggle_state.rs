//! A toggle in the terminal shows the row state it flips. The terminal offers a row's actions in
//! the hint line of the cursor row; a toggle there carries an on/off mark from the state
//! `ess_ui::toggle_state` resolves, the same expression the React project is handed. A column
//! `as: toggle` shows an on/off mark, not `true` or `false`.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use ess_ui_tui::{App, DataAdapter, Options, ReadRequest, ReadResult};
use ess_yaml::Value;

/// A page listing flags, with the collection's `columns` and `row_actions` lines written in.
fn document(columns: &str, row_actions: &str) -> String {
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
        columns: {columns}
{row_actions}
"
    )
}

const ROWS: &str = "[{id: f-1, title: Alpha, enabled: true}, \
                    {id: f-2, title: Beta, enabled: true}, \
                    {id: f-3, title: Gamma, enabled: false}]";

/// Answers every read with the three rows; every command is accepted.
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
        .join("ess-ui-tui-toggle-state")
        .join(test);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("a stale dir is removed");
    }
    std::fs::create_dir_all(&dir).expect("the dir is created");
    dir
}

fn app(test: &str, columns: &str, row_actions: &str) -> App {
    let yaml = document(columns, row_actions);
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

/// The hint line offering the cursor row's actions.
fn hint(screen: &str) -> &str {
    screen
        .lines()
        .find(|line| line.contains("Enabled"))
        .unwrap_or_else(|| panic!("no hint offers the toggle:\n{screen}"))
}

#[test]
fn a_command_toggle_shows_the_row_state() {
    let mut app = app(
        "row-toggle",
        "[title]",
        "        row_actions:\n          - {name: enabled, as: toggle, label: Enabled, does: flags.SetFlag, bind: {id: row.id, enabled: not row.enabled}}",
    );
    app.focus_section("list");
    let first = app.render_text(120, 24);
    assert!(
        hint(&first).contains("[x] Enabled"),
        "the cursor row is on, and its toggle shows it:\n{first}"
    );
    app.keys("jj");
    let third = app.render_text(120, 24);
    assert!(
        hint(&third).contains("[ ] Enabled"),
        "the cursor row is off, and its toggle shows it:\n{third}"
    );
}

#[test]
fn a_toggle_column_shows_a_switch_mark() {
    let app = app(
        "column",
        "[title, {field: enabled, as: toggle, label: Enabled}]",
        "",
    );
    let screen = app.render_text(120, 24);
    let line = |name: &str| {
        screen
            .lines()
            .find(|line| line.contains(name))
            .unwrap_or_else(|| panic!("no row {name}:\n{screen}"))
            .to_owned()
    };
    assert!(line("Alpha").contains("[x]"), "{screen}");
    assert!(line("Beta").contains("[x]"), "{screen}");
    assert!(line("Gamma").contains("[ ]"), "{screen}");
    assert!(
        !screen.contains("true") && !screen.contains("false"),
        "the column shows a mark, not the words:\n{screen}"
    );
}
