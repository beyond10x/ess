//! Adversary pass 3 on story:a-toggle-shows-the-row-state-it-flips, terminal side, against fix
//! round 2. `App::toggle_on` now reads a state with the filter grammar when it parses there with
//! admitted roots, and falls back to `expr::eval` otherwise; the outcome is judged by
//! `expr::truthy`. Each case has a React twin in `ess-ui-react`'s pass-3 file on the same rows,
//! whose expected marks are the ones written here.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use ess_ui_tui::{App, DataAdapter, Options, ReadRequest, ReadResult};
use ess_yaml::Value;

fn document(action: &str) -> String {
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
    sections:
      - name: list
        component: collection
        reads: {{view: posts.All}}
        columns: [title]
        row_actions:
          - {action}
"
    )
}

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
        .join("ess-ui-tui-adversary-w086-u2-pass3")
        .join(test);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("a stale dir is removed");
    }
    std::fs::create_dir_all(&dir).expect("the dir is created");
    dir
}

fn hint(screen: &str) -> &str {
    screen
        .lines()
        .find(|line| line.contains("Switch"))
        .unwrap_or_else(|| panic!("no hint offers the toggle:\n{screen}"))
}

/// The marks the terminal shows for the first and the second row.
fn marks(test: &str, action: &str, rows: &str) -> (bool, bool, String) {
    let yaml = document(action);
    let document = ess_ui::load_str(&yaml).unwrap_or_else(|error| panic!("{error}\n{yaml}"));
    let adapter = Box::new(Table {
        rows: ess_yaml::from_str(rows).expect("the rows parse"),
    });
    let mut app = App::with_adapter(
        document,
        adapter,
        Vec::new(),
        Options::new(dir(test).join("state")),
    )
    .unwrap_or_else(|error| panic!("{error}"));
    app.focus_section("list");
    let first = app.render_text(120, 24);
    app.keys("j");
    let second = app.render_text(120, 24);
    (
        hint(&first).contains("[x] Switch"),
        hint(&second).contains("[x] Switch"),
        format!("{first}\n---\n{second}"),
    )
}

/// Twin of the React red case: the terminal reads `-1` in the filter grammar.
#[test]
fn a_flipped_comparison_with_a_negative_number_shows_the_row_state() {
    let (first, second, screens) = marks(
        "negative",
        r#"{name: t, as: toggle, label: Switch, does: posts.SetT, bind: {id: row.id, on: "not (row.level == -1)"}}"#,
        "[{id: p-1, title: Alpha, level: -1}, {id: p-2, title: Beta, level: 3}]",
    );
    assert_eq!((first, second), (true, false), "{screens}");
}

/// `row.id in row.owners` does not parse in the filter grammar (its `in` needs a literal list),
/// so `toggle_on` falls back to `expr::eval`, which has no `in`: it walks the path
/// `row."id in row".owners`, finds null, and draws every row off. React draws on, off.
#[test]
fn a_checked_membership_in_a_row_list_shows_the_row_state() {
    let action = "{name: t, as: toggle, label: Switch, does: posts.SetT, bind: {id: row.id}, checked: row.id in row.owners}";
    let yaml = document(action);
    let report = ess_ui_check::check_source(
        &yaml,
        "posts.yaml",
        Path::new("."),
        None,
        &ess_ui_check::Options::default(),
    );
    let warned = report.findings.iter().any(|finding| {
        finding.check == "checked_unreadable"
            && finding.path == "pages/posts.list/sections/list/row_actions/t/checked"
            && finding.severity == ess_ui_check::Severity::Warning
            && finding.message.contains("terminal cannot evaluate")
    });
    assert!(
        warned,
        "story:a-toggle-shows-the-row-state-it-flips: the filter grammar is not widened for a \
         `checked:` with a path right of `in`; `ess ui check` warns that the terminal cannot \
         evaluate it (checked_unreadable) instead of the terminal drawing on, off: {:#?}",
        report.findings
    );
}

/// The state `row.enabled` is the text `false` on the first row. The filter grammar's own
/// truthiness (and React's) holds it true; `toggle_on` judges the outcome with `expr::truthy`,
/// which holds the text `false` false. React draws on, off.
#[test]
fn a_flipped_text_field_shows_the_row_state_as_the_runtime_reads_it() {
    let (first, second, screens) = marks(
        "text",
        r#"{name: t, as: toggle, label: Switch, does: posts.SetT, bind: {id: row.id, on: "not row.enabled"}}"#,
        "[{id: p-1, title: Alpha, enabled: 'false'}, {id: p-2, title: Beta, enabled: ''}]",
    );
    assert_eq!(
        (first, second),
        (true, false),
        "React draws on, off:\n{screens}"
    );
}

/// Missing fields and nulls compare as empty text on both sides.
#[test]
fn a_flipped_null_comparison_over_a_missing_field_shows_the_row_state() {
    let (first, second, screens) = marks(
        "null",
        r#"{name: t, as: toggle, label: Switch, does: posts.SetT, bind: {id: row.id, on: "not (row.gone == null and row.kind in [null, 'b c'])"}}"#,
        "[{id: p-1, title: Alpha, kind: null}, {id: p-2, title: Beta, gone: 1, kind: 'b c'}]",
    );
    assert_eq!((first, second), (true, false), "{screens}");
}
