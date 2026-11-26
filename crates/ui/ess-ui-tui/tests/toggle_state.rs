//! A toggle in the terminal shows the row state it flips. The terminal offers a row's actions in
//! the hint line of the cursor row; a toggle there carries an on/off mark from the state
//! `ess_ui::toggle_state` resolves, the same expression the React project is handed. A column
//! `as: toggle` shows an on/off mark, not `true` or `false`.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::Duration;

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

/// The settings row `flags.Settings` answers, and every command sent, shared with the test.
struct Loaded {
    settings: Value,
    sent: Vec<(String, BTreeMap<String, Value>)>,
}

/// Answers `flags.Settings` with the shared settings row and every other read with the flags.
struct Settings {
    rows: Vec<Value>,
    shared: Rc<RefCell<Loaded>>,
}

impl DataAdapter for Settings {
    fn read(&self, request: &ReadRequest) -> Result<ReadResult, String> {
        let rows = if request.view == "flags.Settings" {
            vec![self.shared.borrow().settings.clone()]
        } else {
            self.rows.clone()
        };
        Ok(ReadResult { rows, total: None })
    }
    fn run(&mut self, command: &str, input: &BTreeMap<String, Value>) -> ess_ui::binding::Answer {
        self.shared
            .borrow_mut()
            .sent
            .push((command.to_owned(), input.clone()));
        // The effect was applied to the rows, so the terminal reads what is shown again.
        ess_ui::binding::Answer::FixtureApplied
    }
    fn load_state(&self, _: &str) -> Option<Value> {
        None
    }
    fn store_state(&mut self, _: &str, _: Value) {}
}

/// The key the hint line offers before `label`: `<key> [x] <label>` for a toggle, `<key> <label>`
/// for another action.
fn key_before(line: &str, label: &str) -> char {
    let at = line
        .find(label)
        .unwrap_or_else(|| panic!("{label} is not offered: {line}"));
    let before = line[..at]
        .trim_end_matches("[x] ")
        .trim_end_matches("[ ] ")
        .trim_end();
    before
        .chars()
        .next_back()
        .unwrap_or_else(|| panic!("no key before {label}: {line}"))
}

/// `loads.<field>` reads the first row the action's `loads` read answers (beyond10x/ess#723,
/// C17): before it answers the toggle is drawn disabled (`·` for its key, which runs nothing) and
/// the field is absent (off, and an action whose `visible` reads it is not drawn); after, `bind`,
/// `checked` and `visible` read the field; a command the action sends reads it again. React holds
/// the same moments (`ess-ui-react`, `tests/toggle_state.rs`).
#[test]
fn an_action_reads_the_row_it_loads_once_it_answers_and_again_after_its_command() {
    let actions = "        actions:\n          - {name: paused, as: toggle, label: Paused, does: flags.SetPaused, loads: {view: flags.Settings}, bind: {paused: not loads.paused}}\n          - {name: resume, label: Resume, does: flags.Resume, loads: {view: flags.Settings}, visible: loads.paused, bind: {since: loads.since}}";
    let yaml = document("[title]", actions);
    let document = ess_ui::load_str(&yaml).unwrap_or_else(|error| panic!("{error}\n{yaml}"));
    let shared = Rc::new(RefCell::new(Loaded {
        settings: ess_yaml::from_str("{paused: true, since: '09:00'}").expect("the row parses"),
        sent: Vec::new(),
    }));
    let adapter = Box::new(Settings {
        rows: ess_yaml::from_str(ROWS).expect("the rows parse"),
        shared: Rc::clone(&shared),
    });
    let mut options = Options::new(dir("loads").join("state"));
    options
        .view_latency
        .insert("flags.Settings".to_owned(), Duration::from_secs(1));
    let mut app = App::with_adapter(document, adapter, Vec::new(), options)
        .unwrap_or_else(|error| panic!("{error}"));
    app.focus_section("list");
    let paused = |screen: &str| {
        screen
            .lines()
            .find(|line| line.contains("Paused"))
            .unwrap_or_else(|| panic!("the toggle is drawn:\n{screen}"))
            .to_owned()
    };

    let before = app.render_text(120, 24);
    assert!(
        paused(&before).contains("· [ ] Paused"),
        "drawn disabled and off until the row answers:\n{before}"
    );
    assert!(
        !before.contains("Resume"),
        "a visible reading the absent field hides its action:\n{before}"
    );
    app.advance(Duration::ZERO);
    app.advance(Duration::from_secs(1));

    let after = app.render_text(120, 24);
    let line = paused(&after);
    assert!(
        line.contains("[x] Paused") && !line.contains("· [x]"),
        "checked reads loads.paused once the row answers:\n{after}"
    );
    let resume = key_before(&line, "Resume");
    shared.borrow_mut().settings =
        ess_yaml::from_str("{paused: false, since: '10:00'}").expect("the row parses");
    app.keys(&resume.to_string());
    let sent = shared.borrow().sent.clone();
    assert_eq!(sent.len(), 1, "{sent:?}");
    assert_eq!(sent[0].0, "flags.Resume");
    assert_eq!(
        sent[0].1.get("since"),
        Some(&Value::String("09:00".to_owned())),
        "bind reads loads.since"
    );
    app.advance(Duration::ZERO);
    app.advance(Duration::from_secs(1));

    let again = app.render_text(120, 24);
    assert!(
        paused(&again).contains("[ ] Paused"),
        "the command reads the loaded row again:\n{again}"
    );
    assert!(
        !paused(&again).contains("Resume"),
        "visible reads the row read again:\n{again}"
    );
}
