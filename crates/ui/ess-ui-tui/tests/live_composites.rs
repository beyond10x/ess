//! beyond10x/ess#354 in the terminal: a reading node inside a tab or a header takes its own `live`
//! block and applies the channel's events to its own read while it is shown; a tab shown again
//! reads again; and `header.title_from` shows a field of the record a named section holds, with
//! that section's live changes applied.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::Duration;

use ess_ui::binding::{Answer, Binding};
use ess_ui_tui::live::Script;
use ess_ui_tui::{App, DataAdapter, FixtureAdapter, Options, ReadRequest, ReadResult};
use serde_yaml::Value;

const DOCUMENT: &str = r"
format: ess-ui/1
app: console
model: console.system
placement_profile: fat
fixtures:
  views: {objectives.ById: data.yaml, objectives.Evidence: data.yaml, objectives.Cost: data.yaml}
shells:
  app: {regions: {main: {kind: page_outlet}}}
navigation:
  home: objective
  sections: [{name: all, pages: [objective]}]
channels:
  progress:
    carries: {events: [objectives.EvidenceAdded, objectives.GoalChanged]}
    direction: server_to_client
    delivery: every_event
    resume: refetch
pages:
  objective:
    kind: detail_page
    title: Objective
    header:
      title: Objective
      title_from: {section: objective, field: goal}
      metrics:
        - name: evidence_count
          component: metric
          reads: {view: objectives.Evidence}
          aggregate: count
          label: Evidence count
          live: {channel: progress, on: [objectives.EvidenceAdded], effect: insert_top}
    sections:
      - name: objective
        component: record
        reads: {view: objectives.ById, key: objective_id}
        live: {channel: progress, on: [objectives.GoalChanged], effect: patch_row}
        fields: [goal]
        tabs:
          - name: evidence
            label: Evidence
            form:
              name: evidence
              component: collection
              reads: {view: objectives.Evidence, key: evidence_id}
              columns: [note]
              live: {channel: progress, on: [objectives.EvidenceAdded], effect: insert_top}
          - name: cost
            label: Cost
            form:
              name: cost
              component: metric
              reads: {view: objectives.Cost}
              aggregate: sum
              field: amount
              label: Cost
              live: {channel: progress, effect: refetch}
";

const DATA: &str = r"
views:
  objectives.ById:
    rows:
      - {objective_id: o-1, goal: Ship the importer}
  objectives.Evidence:
    rows:
      - {evidence_id: e-1, note: First benchmark}
  objectives.Cost:
    rows:
      - {id: c-1, amount: 10}
      - {id: c-2, amount: 5}
";

/// Evidence at 1 s, a goal change at 2 s, more evidence at 3 s.
const SCRIPT: &str = "{channel: progress, events: [
  {at: 1000ms, event: objectives.EvidenceAdded, payload: {evidence_id: e-2, note: Second benchmark}},
  {at: 2000ms, event: objectives.GoalChanged, payload: {objective_id: o-1, goal: Ship the importer twice}},
  {at: 3000ms, event: objectives.EvidenceAdded, payload: {evidence_id: e-3, note: Third benchmark}}]}";

fn dir(test: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("ess-ui-tui-live-composites")
        .join(test);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("a stale dir is removed");
    }
    std::fs::create_dir_all(&dir).expect("the dir is created");
    std::fs::write(dir.join("data.yaml"), DATA).expect("the fixture is written");
    dir
}

/// Counts reads per view, answering from the fixtures.
struct Counting {
    inner: FixtureAdapter,
    reads: Rc<RefCell<BTreeMap<String, usize>>>,
}

impl DataAdapter for Counting {
    fn read(&self, request: &ReadRequest) -> Result<ReadResult, String> {
        *self
            .reads
            .borrow_mut()
            .entry(request.view.clone())
            .or_default() += 1;
        self.inner.read(request)
    }
    fn run(&mut self, command: &str, input: &BTreeMap<String, Value>) -> Answer {
        self.inner.run(command, input)
    }
    fn load_state(&self, path: &str) -> Option<Value> {
        self.inner.load_state(path)
    }
    fn store_state(&mut self, path: &str, value: Value) {
        self.inner.store_state(path, value);
    }
}

type Reads = Rc<RefCell<BTreeMap<String, usize>>>;

fn app(test: &str) -> (App, Reads) {
    let base = dir(test);
    let document = ess_ui::load_str(DOCUMENT).unwrap_or_else(|error| panic!("{error}"));
    let (inner, _) = FixtureAdapter::load(&document, &base, None).expect("the fixtures load");
    let script = Script::parse(
        "progress",
        &serde_yaml::from_str(SCRIPT).expect("the script is YAML"),
    )
    .expect("the script parses");
    let reads = Rc::new(RefCell::new(BTreeMap::new()));
    let adapter = Counting {
        inner,
        reads: Rc::clone(&reads),
    };
    let mut app = App::with_adapter(
        document,
        Box::new(adapter),
        vec![script],
        Options::new(base.join("state")),
    )
    .unwrap_or_else(|error| panic!("{error}"));
    app.open_page("objective", &[]);
    (app, reads)
}

fn count(reads: &Reads, view: &str) -> usize {
    reads.borrow().get(view).copied().unwrap_or(0)
}

fn run_for(app: &mut App, total: Duration) {
    let step = Duration::from_millis(100);
    let mut gone = Duration::ZERO;
    while gone < total {
        app.advance(step);
        gone += step;
    }
}

fn screen(app: &App) -> String {
    app.render_text(120, 40)
}

/// The first line of the screen holding `needle`.
fn line_with(screen: &str, needle: &str) -> String {
    screen
        .lines()
        .find(|line| line.contains(needle))
        .unwrap_or_else(|| panic!("no line holds `{needle}`:\n{screen}"))
        .to_owned()
}

#[test]
fn a_live_collection_in_the_shown_tab_takes_the_channels_events() {
    let (mut app, _) = app("tab-collection");
    run_for(&mut app, Duration::from_millis(500));
    let before = screen(&app);
    assert!(before.contains("First benchmark"), "{before}");
    assert!(!before.contains("Second benchmark"), "{before}");
    run_for(&mut app, Duration::from_millis(1000));
    let after = screen(&app);
    assert!(
        after.contains("Second benchmark"),
        "the tab's collection took the event:\n{after}"
    );
    assert!(after.contains("First benchmark"), "{after}");
}

#[test]
fn a_live_header_metric_takes_the_channels_events() {
    let (mut app, _) = app("header-metric");
    run_for(&mut app, Duration::from_millis(500));
    let before = screen(&app);
    assert!(
        line_with(&before, "Evidence count").contains("Evidence count: 1"),
        "{before}"
    );
    run_for(&mut app, Duration::from_millis(1000));
    let after = screen(&app);
    assert!(
        line_with(&after, "Evidence count").contains("Evidence count: 2"),
        "the header metric counts the inserted row:\n{after}"
    );
}

#[test]
fn the_header_title_reads_the_named_sections_record_and_its_live_changes() {
    let (mut app, _) = app("header-title");
    run_for(&mut app, Duration::from_millis(500));
    let before = screen(&app);
    let header = line_with(&before, "Evidence count");
    let title_line = before
        .lines()
        .take_while(|line| *line != header)
        .last()
        .unwrap_or_default()
        .to_owned();
    assert!(
        title_line.contains("Ship the importer"),
        "the header title is the record's goal:\n{before}"
    );
    run_for(&mut app, Duration::from_millis(2000));
    let after = screen(&app);
    assert!(
        after.contains("Ship the importer twice"),
        "the title follows the section patched live:\n{after}"
    );
}

#[test]
fn the_header_title_is_the_literal_title_until_the_record_holds_the_field() {
    let base = dir("header-title-fallback");
    std::fs::write(
        base.join("data.yaml"),
        DATA.replace(
            "{objective_id: o-1, goal: Ship the importer}",
            "{objective_id: o-1}",
        ),
    )
    .expect("the fixture is written");
    let mut app = App::from_text(DOCUMENT, &base, Options::new(base.join("state")))
        .unwrap_or_else(|error| panic!("{error}"));
    app.open_page("objective", &[]);
    let shown = screen(&app);
    let header = line_with(&shown, "Evidence count");
    let title_line = shown
        .lines()
        .take_while(|line| *line != header)
        .last()
        .unwrap_or_default()
        .to_owned();
    assert!(title_line.contains("Objective"), "{shown}");
}

#[test]
fn an_inactive_tab_holds_no_subscription_and_reads_again_when_shown() {
    let (mut app, reads) = app("inactive-tab");
    run_for(&mut app, Duration::from_millis(500));
    assert_eq!(
        count(&reads, "objectives.Cost"),
        0,
        "the hidden cost tab reads nothing"
    );
    run_for(&mut app, Duration::from_millis(1000));
    assert_eq!(
        count(&reads, "objectives.Cost"),
        0,
        "a refetch event does not reach the hidden tab"
    );
    app.focus_section("objective");
    app.keys("]");
    run_for(&mut app, Duration::from_millis(100));
    assert_eq!(count(&reads, "objectives.Cost"), 1, "shown, the tab reads");
    assert!(screen(&app).contains("Cost: 15"), "{}", screen(&app));
    run_for(&mut app, Duration::from_millis(800));
    assert_eq!(
        count(&reads, "objectives.Cost"),
        2,
        "the shown tab refetches on the 2 s event"
    );
    let evidence = count(&reads, "objectives.Evidence");
    app.keys("[");
    run_for(&mut app, Duration::from_millis(100));
    assert!(
        count(&reads, "objectives.Evidence") > evidence,
        "the evidence tab reads again when shown"
    );
}

#[test]
fn a_live_node_in_a_record_item_follows_its_row_context() {
    let text = DOCUMENT.replace(
        "        fields: [goal]\n",
        "        fields: [goal]\n        item:\n          - name: inline\n            component: metric\n            reads: {view: objectives.Evidence, params: {objective: row.objective_id}}\n            aggregate: count\n            label: Inline count\n            live: {channel: progress, on: [objectives.EvidenceAdded], effect: insert_top}\n",
    );
    let base = dir("record-item");
    let document = ess_ui::load_str(&text).unwrap_or_else(|error| panic!("{error}"));
    let (adapter, _) = FixtureAdapter::load(&document, &base, None).expect("the fixtures load");
    let script = Script::parse(
        "progress",
        &serde_yaml::from_str(SCRIPT).expect("the script is YAML"),
    )
    .expect("the script parses");
    let mut app = App::with_adapter(
        document,
        Box::new(adapter),
        vec![script],
        Options::new(base.join("state")),
    )
    .unwrap_or_else(|error| panic!("{error}"));
    app.open_page("objective", &[]);
    run_for(&mut app, Duration::from_millis(500));
    assert!(screen(&app).contains("Inline count: 1"), "{}", screen(&app));
    run_for(&mut app, Duration::from_millis(1000));
    assert!(screen(&app).contains("Inline count: 2"), "{}", screen(&app));
}

/// A form field's choice with its own `reads` takes `live`: a `refetch` event reads its options
/// again while the form is shown.
#[test]
fn a_live_form_field_choice_reads_its_options_again_on_an_event() {
    let text = format!(
        "{}\n      - name: assign\n        component: form\n        does: objectives.Assign\n        fields:\n          - field: owner\n            as: choice\n            choice: {{component: choice, reads: {{view: people.All, key: person_id}}, live: {{channel: progress, on: [objectives.GoalChanged], effect: refetch}}}}\n",
        DOCUMENT
            .trim_end()
            .replace("objectives.Cost: data.yaml}", "objectives.Cost: data.yaml, people.All: data.yaml}"),
    );
    let base = dir("form-choice");
    std::fs::write(
        base.join("data.yaml"),
        format!("{DATA}  people.All:\n    rows:\n      - {{person_id: p-1, name: Ada}}\n"),
    )
    .expect("the fixture is written");
    let document = ess_ui::load_str(&text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let (inner, _) = FixtureAdapter::load(&document, &base, None).expect("the fixtures load");
    let script = Script::parse(
        "progress",
        &serde_yaml::from_str(SCRIPT).expect("the script is YAML"),
    )
    .expect("the script parses");
    let reads = Rc::new(RefCell::new(BTreeMap::new()));
    let adapter = Counting {
        inner,
        reads: Rc::clone(&reads),
    };
    let mut app = App::with_adapter(
        document,
        Box::new(adapter),
        vec![script],
        Options::new(base.join("state")),
    )
    .unwrap_or_else(|error| panic!("{error}"));
    app.open_page("objective", &[]);
    run_for(&mut app, Duration::from_millis(1500));
    let before = count(&reads, "people.All");
    assert!(
        before >= 1,
        "the choice reads its options: {:?}",
        reads.borrow()
    );
    run_for(&mut app, Duration::from_millis(1000));
    assert_eq!(
        count(&reads, "people.All"),
        before + 1,
        "the goal change at 2 s reads the options again: {:?}",
        reads.borrow()
    );
}

// ── bound to a served surface ────────────────────────────────────────────────────────────────

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

fn gatepass() -> Vec<(String, String)> {
    let root = root().join("examples/gatepass");
    ["system.yaml", "components.yaml", "domains/visit.yaml"]
        .iter()
        .map(|file| {
            let text = std::fs::read_to_string(root.join(file))
                .unwrap_or_else(|error| panic!("{file}: {error}"));
            ((*file).to_owned(), text)
        })
        .collect()
}

fn bound_desk(extra: &str) -> String {
    format!(
        "format: ess-ui/1
app: desk
model: gatepass
placement_profile: fat
shells: {{app: {{regions: {{main: {{kind: page_outlet}}}}}}}}
navigation: {{home: desk, sections: [{{name: all, pages: [desk]}}]}}
channels:
  visits:
    carries: {{events: [visit.VisitRegistered]}}
    direction: server_to_client
    delivery: every_event
    resume: refetch
pages:
  desk:
    kind: detail_page
    title: Desk
    sections:
      - name: today
        component: record
        reads: visit.ExpectedVisits
        item:
          - name: waiting
            component: metric
            reads: {{view: visit.ExpectedVisits, params: {{building: North}}{extra}}}
            aggregate: count
            label: Waiting
            live: {{channel: visits, effect: insert_top}}
"
    )
}

/// Answers every read with one row, counting reads per request.
struct Served {
    reads: Rc<RefCell<BTreeMap<String, usize>>>,
}

impl DataAdapter for Served {
    fn read(&self, request: &ReadRequest) -> Result<ReadResult, String> {
        *self.reads.borrow_mut().entry(request.key()).or_default() += 1;
        Ok(ReadResult {
            rows: vec![serde_yaml::from_str("{visit_id: v-1, visitor: Ada}").expect("a row")],
            total: Some(1),
        })
    }
    fn run(&mut self, _command: &str, _input: &BTreeMap<String, Value>) -> Answer {
        panic!("no command is sent")
    }
    fn load_state(&self, _path: &str) -> Option<Value> {
        None
    }
    fn store_state(&mut self, _path: &str, _value: Value) {}
}

fn bound(test: &str, document_text: &str) -> Result<(App, Reads), String> {
    let document =
        ess_ui::load_str(document_text).unwrap_or_else(|error| panic!("{error}\n{document_text}"));
    let binding: Binding = ess_ui_check::binding(&document, &gatepass())
        .unwrap_or_else(|error| panic!("the desk binds: {error}"));
    let reads = Rc::new(RefCell::new(BTreeMap::new()));
    let adapter = Served {
        reads: Rc::clone(&reads),
    };
    let base = dir(test);
    App::bound(
        document,
        Box::new(adapter),
        binding,
        Options::new(base.join("state")),
    )
    .map(|app| (app, reads))
    .map_err(|error| error.to_string())
}

#[test]
fn a_bound_nested_live_node_polls_its_read() {
    let (mut app, reads) =
        bound("bound-polls", &bound_desk(", refresh: 2s")).expect("the desk opens");
    app.open_page("desk", &[]);
    run_for(&mut app, Duration::from_millis(100));
    let nested = |reads: &Reads| {
        reads
            .borrow()
            .iter()
            .filter(|(key, _)| key.contains("North"))
            .map(|(_, count)| *count)
            .sum::<usize>()
    };
    let first = nested(&reads);
    assert!(first >= 1, "the nested metric read: {:?}", reads.borrow());
    run_for(&mut app, Duration::from_millis(1000));
    assert_eq!(
        nested(&reads),
        first,
        "no read before the refresh interval: {:?}",
        reads.borrow()
    );
    run_for(&mut app, Duration::from_millis(1100));
    assert!(
        nested(&reads) > first,
        "the nested live metric polls at its refresh: {:?}",
        reads.borrow()
    );
}

#[test]
fn a_bound_nested_live_node_refusing_to_poll_is_refused_at_its_path() {
    let text = bound_desk("").replace(
        "            aggregate: count\n",
        "            aggregate: count\n            degrades: {no_live: refuse}\n",
    );
    let Err(error) = bound("bound-refused", &text) else {
        panic!("refused");
    };
    assert!(
        error.contains("pages/desk/sections/today/item/waiting/live"),
        "{error}"
    );
}
