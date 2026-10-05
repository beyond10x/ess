//! Adversary pass 1 for beyond10x/ess#354 in the terminal: the unit's own schema note, driven
//! against the code the unit wrote.
//!
//! `schemas/ui/ess-ui.schema.yaml` (`CompositeNode` doc, written by this unit): "A nested composite
//! with its own `reads` ... takes `live` as a section does: the channel's events change its own
//! rows while it is shown. A node that is not shown (an inactive tab, a collapsed `expand`, a node
//! whose `visible` is false) applies no event and reads its view again when it is shown."

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::Duration;

use ess_ui::binding::Answer;
use ess_ui_tui::live::Script;
use ess_ui_tui::{App, DataAdapter, FixtureAdapter, Options, ReadRequest, ReadResult};
use serde_yaml::Value;

const DATA: &str = r"
views:
  objectives.All:
    rows:
      - {objective_id: o-1, goal: First goal}
      - {objective_id: o-2, goal: Second goal}
  objectives.Evidence:
    rows:
      - {evidence_id: e-1, note: First benchmark}
      - {evidence_id: e-2, note: Second benchmark}
      - {evidence_id: e-3, note: Third benchmark}
";

fn dir(test: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("ess-ui-tui-adversary-live-composites-pass1")
        .join(test);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("a stale dir is removed");
    }
    std::fs::create_dir_all(&dir).expect("the dir is created");
    std::fs::write(dir.join("data.yaml"), DATA).expect("the fixture is written");
    dir
}

type Reads = Rc<RefCell<BTreeMap<String, usize>>>;

/// Counts reads per request key, answering from the fixtures.
struct Counting {
    inner: FixtureAdapter,
    reads: Reads,
}

impl DataAdapter for Counting {
    fn read(&self, request: &ReadRequest) -> Result<ReadResult, String> {
        *self.reads.borrow_mut().entry(request.key()).or_default() += 1;
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

fn app(
    test: &str,
    document_text: &str,
    script: Option<&str>,
    page_size: Option<usize>,
) -> (App, Reads) {
    let base = dir(test);
    let document = ess_ui::load_str(document_text).unwrap_or_else(|error| panic!("{error}"));
    let (inner, _) = FixtureAdapter::load(&document, &base, None).expect("the fixtures load");
    let scripts = script
        .map(|script| {
            vec![Script::parse(
                "progress",
                &serde_yaml::from_str(script).expect("the script is YAML"),
            )
            .expect("the script parses")]
        })
        .unwrap_or_default();
    let reads = Rc::new(RefCell::new(BTreeMap::new()));
    let adapter = Counting {
        inner,
        reads: Rc::clone(&reads),
    };
    let mut options = Options::new(base.join("state"));
    if let Some(size) = page_size {
        options.page_size = size;
    }
    let app = App::with_adapter(document, Box::new(adapter), scripts, options)
        .unwrap_or_else(|error| panic!("{error}"));
    (app, reads)
}

fn count(reads: &Reads, key: &str) -> usize {
    reads.borrow().get(key).copied().unwrap_or(0)
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

// ── a collapsed `expand` shown again ─────────────────────────────────────────────────────────

const EXPAND: &str = r"
format: ess-ui/1
app: console
model: console.system
placement_profile: fat
fixtures:
  views: {objectives.All: data.yaml, objectives.Evidence: data.yaml}
shells:
  app: {regions: {main: {kind: page_outlet}}}
navigation:
  home: objectives
  sections: [{name: all, pages: [objectives]}]
channels:
  progress:
    carries: {events: [objectives.EvidenceAdded]}
    direction: server_to_client
    delivery: every_event
    resume: refetch
pages:
  objectives:
    kind: list_page
    title: Objectives
    sections:
      - name: list
        component: collection
        reads: {view: objectives.All, key: objective_id}
        columns: [goal]
        expand:
          name: detail
          component: metric
          reads: {view: objectives.Evidence, params: {objective: row.objective_id}}
          aggregate: count
          label: Evidence
          live: {channel: progress, effect: insert_top}
";

/// The expand of row o-1 is shown, then hidden while the cursor shows o-2's, then shown again.
/// While hidden it applied no event (it is no target), so by the schema note it reads its view
/// again when shown. The React project unmounts the first row's expand when another row is
/// expanded and mounts it fresh (`useRead(…, fresh = true)`) when it is expanded again. The
/// terminal keys "newly shown" by node path (`App::shown_live`), and every row's expand has the
/// same path `…/list/expand`, so o-1's cached read, which missed every event played while o-2
/// was shown, is shown again as it was.
#[test]
fn an_expand_shown_again_for_its_row_reads_again() {
    let (mut app, reads) = app("expand-again", EXPAND, None, None);
    app.open_page("objectives", &[]);
    run_for(&mut app, Duration::from_millis(300));
    app.focus_section("list");
    app.keys("x");
    run_for(&mut app, Duration::from_millis(300));
    let first = "objectives.Evidence?objective=o-1";
    let second = "objectives.Evidence?objective=o-2";
    assert_eq!(
        count(&reads, first),
        1,
        "o-1's expand is shown and reads: {reads:?}"
    );
    app.keys("j");
    run_for(&mut app, Duration::from_millis(300));
    assert_eq!(
        count(&reads, second),
        1,
        "o-2's expand is shown and reads: {reads:?}"
    );
    app.keys("k");
    run_for(&mut app, Duration::from_millis(300));
    assert_eq!(
        count(&reads, first),
        2,
        "o-1's expand was not shown while the cursor was on o-2, so it reads again when shown \
         again: {reads:?}\n{}",
        screen(&app)
    );
}

// ── a section and a nested node sharing one read ─────────────────────────────────────────────

fn shared(section_live: &str) -> String {
    format!(
        r"
format: ess-ui/1
app: console
model: console.system
placement_profile: fat
fixtures:
  views: {{objectives.Evidence: data.yaml}}
shells:
  app: {{regions: {{main: {{kind: page_outlet}}}}}}
navigation:
  home: board
  sections: [{{name: all, pages: [board]}}]
channels:
  progress:
    carries: {{events: [objectives.EvidenceAdded]}}
    direction: server_to_client
    delivery: every_event
    resume: refetch
pages:
  board:
    kind: list_page
    title: Board
    header:
      title: Board
      metrics:
        - name: evidence_count
          component: metric
          reads: {{view: objectives.Evidence}}
          aggregate: count
          label: Evidence count
          live: {{channel: progress, on: [objectives.EvidenceAdded], effect: insert_top}}
    sections:
      - name: evidence
        component: collection
        reads: {{view: objectives.Evidence, key: evidence_id}}
        columns: [note]
        live: {section_live}
"
    )
}

const ADDED: &str = "{channel: progress, events: [
  {at: 1000ms, event: objectives.EvidenceAdded, payload: {evidence_id: e-4, note: Fourth benchmark}}]}";

fn metric_line(screen: &str) -> String {
    screen
        .lines()
        .find(|line| line.contains("Evidence count"))
        .unwrap_or_else(|| panic!("no metric line:\n{screen}"))
        .to_owned()
}

/// The section and the header metric read the same view, so the terminal holds one cache entry
/// and takes the event into it once (`App::nested_targets`): the section is the target and the
/// metric is dropped. Paged away, the section holds the new row as "1 new" and changes no rows,
/// so the metric, which declares its own `live: {effect: insert_top}` and is shown, never counts
/// it. In the React project the metric applies its own `live` to its own rows (`useLive`), so it
/// counts 4 at once.
#[test]
fn a_header_metric_sharing_a_paged_away_sections_read_counts_the_new_row() {
    let (mut app, _) = app(
        "shared-paged-away",
        &shared("{channel: progress, on: [objectives.EvidenceAdded], effect: insert_top}"),
        Some(ADDED),
        Some(2),
    );
    app.open_page("board", &[]);
    run_for(&mut app, Duration::from_millis(300));
    assert!(
        metric_line(&screen(&app)).contains("Evidence count: 3"),
        "{}",
        screen(&app)
    );
    app.focus_section("evidence");
    app.keys("n");
    run_for(&mut app, Duration::from_millis(1000));
    let shown = screen(&app);
    assert!(
        metric_line(&shown).contains("Evidence count: 4"),
        "the header metric's own `live` inserts the row while it is shown:\n{shown}"
    );
}

/// The same sharing with the section patching rows only: the section takes the event and finds no
/// row to patch, and the metric's own `effect: insert_top` is never applied. Each node's `live`
/// is documented as its own ("same fields/effects as a section"); here the metric's is ignored.
#[test]
fn a_header_metric_sharing_a_patching_sections_read_applies_its_own_effect() {
    let (mut app, _) = app(
        "shared-patch",
        &shared("{channel: progress, on: [objectives.EvidenceAdded], effect: patch_row}"),
        Some(ADDED),
        None,
    );
    app.open_page("board", &[]);
    run_for(&mut app, Duration::from_millis(300));
    assert!(
        metric_line(&screen(&app)).contains("Evidence count: 3"),
        "{}",
        screen(&app)
    );
    run_for(&mut app, Duration::from_millis(1000));
    let shown = screen(&app);
    assert!(
        metric_line(&shown).contains("Evidence count: 4"),
        "the header metric's `effect: insert_top` inserts the new row:\n{shown}"
    );
}

/// Control for the two cases above: the same page with a section that takes no `live`. The
/// metric is then the only target and counts the new row, so what the two cases above measure
/// is the sharing, not the metric.
#[test]
fn control_a_header_metric_alone_on_the_read_counts_the_new_row() {
    let text = shared("NONE").replace("        live: NONE\n", "");
    assert!(!text.contains("NONE"), "{text}");
    let (mut app, _) = app("shared-control", &text, Some(ADDED), None);
    app.open_page("board", &[]);
    run_for(&mut app, Duration::from_millis(300));
    assert!(
        metric_line(&screen(&app)).contains("Evidence count: 3"),
        "{}",
        screen(&app)
    );
    run_for(&mut app, Duration::from_millis(1000));
    let shown = screen(&app);
    assert!(
        metric_line(&shown).contains("Evidence count: 4"),
        "the header metric inserts the new row:\n{shown}"
    );
}
