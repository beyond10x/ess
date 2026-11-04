//! Adversary pass 2 on story:a-toggle-shows-the-row-state-it-flips, terminal side, against fix
//! round 1. The story's Decisions: one library function names the state and both renderers
//! evaluate it, "so the two cannot disagree". The library now derives a state from any single
//! negation (`and`, `or`, `in` under `not` included) and the React runtime evaluates all of them;
//! the terminal's `expr::eval` reads `not`, `==` and `!=` only.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use ess_ui_tui::{App, DataAdapter, Options, ReadRequest, ReadResult};
use ess_yaml::Value;

fn document(row_actions: &str) -> String {
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
{row_actions}
"
    )
}

/// The first row satisfies every operand below, the second none.
const ROWS: &str = "[{id: p-1, title: Alpha, enabled: true, ready: true, kind: a, stage: draft}, \
                    {id: p-2, title: Beta, enabled: false, ready: false, kind: z, stage: live}]";

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
        .join("ess-ui-tui-adversary-w086-u2-pass2")
        .join(test);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("a stale dir is removed");
    }
    std::fs::create_dir_all(&dir).expect("the dir is created");
    dir
}

fn app(test: &str, bind: &str) -> App {
    let yaml = document(&format!(
        r#"          - {{name: t, as: toggle, label: Switch, does: posts.SetT, bind: {{id: row.id, on: "{bind}"}}}}"#
    ));
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
        .find(|line| line.contains("Switch"))
        .unwrap_or_else(|| panic!("no hint offers the toggle:\n{screen}"))
}

/// The marks the terminal shows for the first and the second row.
fn marks(test: &str, bind: &str) -> (bool, bool, String) {
    let mut app = app(test, bind);
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

#[test]
fn a_flipped_conjunction_shows_the_row_state() {
    let (first, second, screens) = marks("and", "not (row.enabled and row.ready)");
    assert_eq!(
        (first, second),
        (true, false),
        "state `row.enabled and row.ready`; React draws on, off:\n{screens}"
    );
}

#[test]
fn a_flipped_disjunction_shows_the_row_state() {
    let (first, second, screens) = marks("or", "not (row.enabled or row.ready)");
    assert_eq!(
        (first, second),
        (true, false),
        "state `row.enabled or row.ready`; React draws on, off:\n{screens}"
    );
}

#[test]
fn a_flipped_membership_shows_the_row_state() {
    let (first, second, screens) = marks("in", "not (row.kind in [a, b])");
    assert_eq!(
        (first, second),
        (true, false),
        "state `row.kind in [a, b]`; React draws on, off:\n{screens}"
    );
}

/// The terminal reads `row.stage == draft` as a comparison with the text `draft`; recorded so the
/// React case of the same document can be compared with it.
#[test]
fn a_flipped_comparison_with_a_root_word_shows_the_row_state() {
    let (first, second, screens) = marks("draft", "not (row.stage == 'draft')");
    assert_eq!((first, second), (true, false), "{screens}");
}
