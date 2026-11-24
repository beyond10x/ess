//! Confirming an action runs that action once, whether or not its confirm declares `does`, and an
//! inline confirm is drawn at its canonical node path, scoped under the row that opened it.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use ess_ui_tui::{App, DataAdapter, FixtureAdapter, Options, ReadRequest, ReadResult};
use ess_yaml::Value;

const DOCUMENT: &str = r"format: ess-ui/1
app: probe
model: probe.system
placement_profile: fat
fixtures: {views: {things.Page: things.yaml}}
shells:
  app:
    regions:
      nav:     {kind: navigation}
      main:    {kind: page_outlet}
      overlay: {kind: overlay_outlet}
navigation:
  home: things.list
  sections: [{name: all, pages: [things.list]}]
pages:
  things.list:
    kind: list_page
    title: Things
    sections:
      - name: list
        component: collection
        reads: {view: things.Page}
        columns: [name]
        row_actions:
          - {name: remove, does: things.Remove, bind: {thing: row.id}, label: Remove, confirm: {title: Remove thing}}
          - {name: drop, does: things.Drop, label: Drop, confirm: ask}
    overlays:
      ask: {kind: dialog, component: confirm, title: Drop thing}
";

/// The fixture adapter, shared so a test reads its command log.
struct Shared(Rc<RefCell<FixtureAdapter>>);

impl DataAdapter for Shared {
    fn read(&self, request: &ReadRequest) -> Result<ReadResult, String> {
        self.0.borrow().read(request)
    }
    fn run(&mut self, command: &str, input: &BTreeMap<String, Value>) -> ess_ui::binding::Answer {
        self.0.borrow_mut().run(command, input)
    }
    fn load_state(&self, path: &str) -> Option<Value> {
        self.0.borrow().load_state(path)
    }
    fn store_state(&mut self, path: &str, value: Value) {
        self.0.borrow_mut().store_state(path, value);
    }
}

fn scratch(test: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("ess-ui-tui-confirm")
        .join(test);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("a stale dir is removed");
    }
    std::fs::create_dir_all(&dir).expect("the dir is made");
    std::fs::write(
        dir.join("things.yaml"),
        "views:\n  things.Page:\n    rows:\n      - {id: th-1, name: Anvil}\n      - {id: th-2, name: Bolt}\n",
    )
    .expect("the fixture is written");
    dir
}

fn open(test: &str) -> (App, Rc<RefCell<FixtureAdapter>>) {
    open_with(test, DOCUMENT)
}

fn open_with(test: &str, yaml: &str) -> (App, Rc<RefCell<FixtureAdapter>>) {
    let dir = scratch(test);
    let document = ess_ui::load_str(yaml).expect("the document loads");
    let (adapter, scripts) = FixtureAdapter::load(&document, &dir, None).expect("fixtures load");
    let adapter = Rc::new(RefCell::new(adapter));
    let mut app = App::with_adapter(
        document,
        Box::new(Shared(Rc::clone(&adapter))),
        scripts,
        Options::new(dir.join("state")),
    )
    .unwrap_or_else(|error| panic!("{error}"));
    app.open_page("things.list", &[]);
    app.focus_section("list");
    (app, adapter)
}

fn sent(adapter: &Rc<RefCell<FixtureAdapter>>) -> Vec<(String, BTreeMap<String, Value>)> {
    adapter.borrow().commands.clone()
}

fn input(pairs: &[(&str, &str)]) -> BTreeMap<String, Value> {
    pairs
        .iter()
        .map(|(name, value)| ((*name).to_owned(), Value::String((*value).to_owned())))
        .collect()
}

#[test]
fn an_inline_confirm_is_drawn_under_its_row_and_runs_the_action_once() {
    let (mut app, adapter) = open("inline");
    // Second row, then the first row action's key (`r` for Remove).
    app.keys("j");
    let text = app.render_text(100, 30);
    assert!(text.contains("Remove"), "{text}");
    app.keys("r");
    let text = app.render_text(100, 30);
    assert!(text.contains("Remove thing"), "{text}");
    let paths: Vec<String> = app
        .regions()
        .into_iter()
        .map(|region| region.path)
        .collect();
    assert!(
        paths.contains(
            &"pages/things.list/sections/list/rows/th-2/row_actions/remove/confirm/overlay"
                .to_owned()
        ),
        "{paths:?}"
    );
    app.keys("y");
    assert_eq!(
        sent(&adapter),
        vec![("things.Remove".to_owned(), input(&[("thing", "th-2")]))],
        "the action runs once, with its own bind"
    );
}

#[test]
fn a_confirm_overlay_without_does_runs_the_action_that_opened_it() {
    let (mut app, adapter) = open("opens");
    app.keys("j");
    app.render_text(100, 30);
    app.keys("d");
    let text = app.render_text(100, 30);
    assert!(text.contains("Drop thing"), "{text}");
    app.keys("y");
    assert_eq!(
        sent(&adapter),
        vec![("things.Drop".to_owned(), BTreeMap::new())],
        "the opener's command runs on confirm"
    );
}

/// A `bind` value written as a list sends one list input of the values its expressions read
/// (story:both-renderers-read-one-binding-expression-grammar, beyond10x/ess#723).
#[test]
fn a_listed_bind_value_is_sent_as_one_list() {
    let text = DOCUMENT.replace(
        "          - {name: drop, does: things.Drop, label: Drop, confirm: ask}\n",
        "          - {name: drop, does: things.Drop, label: Drop, confirm: ask}\n          \
         - {name: tag, does: things.Tag, bind: {things: [row.id, row.name]}, label: Tag}\n",
    );
    assert_ne!(text, DOCUMENT, "the document changed");
    let (mut app, adapter) = open_with("listed", &text);
    app.keys("jt");
    let listed = Value::Sequence(vec![
        Value::String("th-2".to_owned()),
        Value::String("Bolt".to_owned()),
    ]);
    assert_eq!(
        sent(&adapter),
        vec![(
            "things.Tag".to_owned(),
            BTreeMap::from([("things".to_owned(), listed)])
        )],
        "{}",
        app.render_text(100, 30)
    );
}

/// A header action whose `visible:` is false is not drawn, a row action guarded with `and` is
/// offered on the row it holds for, and a bulk bind `selection.<field>` sends the list of that
/// field over the selected rows (story:the-terminal-evaluates-every-expression-form-the-schema-defines,
/// beyond10x/ess#722).
#[test]
fn the_terminal_hides_offers_and_sends_as_every_expression_form_reads() {
    let text = DOCUMENT
        .replace(
            "    title: Things\n    sections:\n",
            "    title: Things\n    header:\n      actions:\n        \
             - {name: hidden, does: things.Never, label: Never drawn, visible: \"params.never == yes\"}\n        \
             - {name: shown, does: things.Shown, label: Drawn action}\n    sections:\n",
        )
        .replace(
            "        columns: [name]\n",
            "        columns: [name]\n        selection: multiple\n        \
             bulk_actions: [{name: archive, does: things.Archive, bulk: true, bind: {ids: selection.id}}]\n",
        )
        .replace(
            "          - {name: drop, does: things.Drop, label: Drop, confirm: ask}\n",
            "          - {name: drop, does: things.Drop, label: Drop, confirm: ask}\n          \
             - {name: reopen, does: things.Reopen, bind: {thing: row.id}, label: Reopen, \
             visible: \"row.name == Bolt and row.id != nobody\"}\n",
        );
    let (mut app, adapter) = open_with("forms", &text);
    let screen = app.render_text(120, 30);
    assert!(screen.contains("Drawn action"), "{screen}");
    assert!(!screen.contains("Never drawn"), "{screen}");
    let hint = |screen: &str| {
        screen
            .lines()
            .find(|line| line.contains("Remove"))
            .map(str::to_owned)
            .unwrap_or_default()
    };
    assert!(!hint(&screen).contains("Reopen"), "Anvil:\n{screen}");
    app.keys("j");
    let screen = app.render_text(120, 30);
    assert!(hint(&screen).contains("Reopen"), "Bolt:\n{screen}");
    app.keys(" A");
    let ids = Value::Sequence(vec![Value::String("th-2".to_owned())]);
    assert_eq!(
        sent(&adapter),
        vec![(
            "things.Archive".to_owned(),
            BTreeMap::from([("ids".to_owned(), ids)])
        )],
        "{}",
        app.render_text(120, 30)
    );
}
