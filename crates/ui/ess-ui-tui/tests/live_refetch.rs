//! A burst of `effect: refetch` events re-reads once, not once per event (beyond10x/ess#325):
//! a coalesced batch is one re-read, and a refetch arriving while a read is outstanding is
//! answered by that read.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::Duration;

use ess_ui_tui::live::Script;
use ess_ui_tui::{App, DataAdapter, FixtureAdapter, Options, ReadRequest, ReadResult};
use serde_yaml::Value;

fn example_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../examples/partner-portal")
}

fn state_dir(test: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("ess-ui-tui-refetch")
        .join(test);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("a stale state dir is removed");
    }
    dir
}

struct Counting {
    inner: FixtureAdapter,
    reads: Rc<RefCell<Vec<String>>>,
}

impl DataAdapter for Counting {
    fn read(&self, request: &ReadRequest) -> Result<ReadResult, String> {
        self.reads.borrow_mut().push(request.view.clone());
        self.inner.read(request)
    }
    fn run(&mut self, command: &str, input: &BTreeMap<String, Value>) -> ess_ui::binding::Answer {
        self.inner.run(command, input)
    }
    fn load_state(&self, path: &str) -> Option<Value> {
        self.inner.load_state(path)
    }
    fn store_state(&mut self, path: &str, value: Value) {
        self.inner.store_state(path, value);
    }
}

const ORIGINAL_LIVE: &str = "live: {channel: tickets, effect: insert_or_patch, only_if: matches(params), when_paged_away: count_new, coalesce: 500ms}";

/// Five `TicketUpdated` events within 200ms, two seconds in.
const BURST: &str = "{channel: tickets, events: [
  {at: 2000ms, event: tickets.TicketUpdated, payload: {id: tk-01, priority: high}},
  {at: 2050ms, event: tickets.TicketUpdated, payload: {id: tk-02, priority: high}},
  {at: 2100ms, event: tickets.TicketUpdated, payload: {id: tk-03, priority: high}},
  {at: 2150ms, event: tickets.TicketUpdated, payload: {id: tk-04, priority: high}},
  {at: 2200ms, event: tickets.TicketUpdated, payload: {id: tk-05, priority: high}}]}";

/// The tickets list with `effect` as its `live`, under a 300ms read latency; returns the app on
/// the list and the reads it has made.
fn tickets_list(test: &str, effect: &str) -> (App, Rc<RefCell<Vec<String>>>) {
    let example =
        std::fs::read_to_string(example_dir().join("ui.yaml")).expect("the example reads");
    assert!(
        example.contains(ORIGINAL_LIVE),
        "the example's tickets list live"
    );
    let document = ess_ui::load_str(&example.replace(ORIGINAL_LIVE, effect)).expect("the document");
    let (inner, _) =
        FixtureAdapter::load(&document, &example_dir(), None).expect("the fixtures load");
    let script = Script::parse(
        "tickets",
        &serde_yaml::from_str(BURST).expect("the script is YAML"),
    )
    .expect("the script parses");
    let reads = Rc::new(RefCell::new(Vec::new()));
    let adapter = Counting {
        inner,
        reads: Rc::clone(&reads),
    };
    let mut options = Options::new(state_dir(test));
    options.read_latency = Duration::from_millis(300);
    let mut app = App::with_adapter(document, Box::new(adapter), vec![script], options)
        .unwrap_or_else(|error| panic!("{error}"));
    app.open_page("tickets.list", &[]);
    (app, reads)
}

fn count(reads: &Rc<RefCell<Vec<String>>>) -> usize {
    reads
        .borrow()
        .iter()
        .filter(|view| *view == "tickets.Page")
        .count()
}

/// Steps the clock 50ms at a time, so every event and every read lands on its own step.
fn run_for(app: &mut App, total: Duration) {
    let step = Duration::from_millis(50);
    let mut gone = Duration::ZERO;
    while gone < total {
        app.advance(step);
        gone += step;
    }
}

#[test]
fn a_coalesced_burst_of_refetch_events_reads_once() {
    let (mut app, reads) = tickets_list(
        "coalesced",
        "live: {channel: tickets, effect: refetch, only_if: matches(params), coalesce: 500ms}",
    );
    run_for(&mut app, Duration::from_millis(1500));
    let before = count(&reads);
    assert!(before >= 1, "the list has read");
    run_for(&mut app, Duration::from_secs(3));
    assert_eq!(
        count(&reads) - before,
        1,
        "five events in one 500ms window are one re-read"
    );
}

#[test]
fn refetch_events_while_a_read_is_outstanding_collapse_into_it() {
    let (mut app, reads) = tickets_list(
        "outstanding",
        "live: {channel: tickets, effect: refetch, only_if: matches(params)}",
    );
    run_for(&mut app, Duration::from_millis(1500));
    let before = count(&reads);
    run_for(&mut app, Duration::from_secs(3));
    // tk-01 at 2000ms starts a read answered at 2300ms; the four events up to 2200ms arrive
    // while it is outstanding and are answered by it.
    assert_eq!(
        count(&reads) - before,
        1,
        "the events after the first are answered by the read they arrived during"
    );
}
