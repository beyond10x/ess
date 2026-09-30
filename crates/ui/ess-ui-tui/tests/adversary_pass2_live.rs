//! Adversary pass 2: what correction 1 introduced in live delivery — rows held by
//! `when_paged_away: count_new`, coalescing against `resume: refetch`, and session scoping.

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

fn example_text() -> String {
    std::fs::read_to_string(example_dir().join("ui.yaml")).expect("the example reads")
}

fn state_dir(test: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("ess-ui-tui-adv2")
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
    fn run(&mut self, command: &str, input: &BTreeMap<String, Value>) -> Result<String, String> {
        self.inner.run(command, input)
    }
    fn load_state(&self, path: &str) -> Option<Value> {
        self.inner.load_state(path)
    }
    fn store_state(&mut self, path: &str, value: Value) {
        self.inner.store_state(path, value);
    }
}

fn app_with(
    test: &str,
    document_text: &str,
    scripts: Option<&[(&str, &str)]>,
    options: Option<Options>,
) -> (App, Rc<RefCell<Vec<String>>>) {
    let document = ess_ui::load_str(document_text).expect("the document loads");
    let (inner, original) =
        FixtureAdapter::load(&document, &example_dir(), None).expect("the fixtures load");
    let scripts = match scripts {
        None => original,
        Some(scripts) => scripts
            .iter()
            .map(|(channel, yaml)| {
                Script::parse(channel, &serde_yaml::from_str(yaml).expect("script yaml"))
                    .expect("the script parses")
            })
            .collect(),
    };
    let reads = Rc::new(RefCell::new(Vec::new()));
    let adapter = Counting {
        inner,
        reads: Rc::clone(&reads),
    };
    let options = options.unwrap_or_else(|| Options::new(state_dir(test)));
    let app = App::with_adapter(document, Box::new(adapter), scripts, options)
        .unwrap_or_else(|error| panic!("{error}"));
    (app, reads)
}

fn count(reads: &Rc<RefCell<Vec<String>>>, view: &str) -> usize {
    reads.borrow().iter().filter(|read| *read == view).count()
}

fn field<'a>(row: &'a Value, key: &str) -> &'a str {
    row[key].as_str().unwrap_or_default()
}

fn row_of(app: &App, section: &str, id: &str) -> Option<Value> {
    app.rows(section)
        .into_iter()
        .find(|row| field(row, "id") == id)
}

fn paged_list(test: &str) -> App {
    let mut options = Options::new(state_dir(test));
    options.page_size = 2;
    let (mut app, _) = app_with(test, &example_text(), None, Some(options));
    app.open_page("tickets.list", &[]);
    app.focus_section("list");
    app.keys("n");
    assert!(app.render_text(140, 48).contains("page 2/3"));
    app
}

/// tickets.yaml: `TicketOpened` tk-06 at 0s, `TicketUpdated` tk-06 (priority urgent) at 18s. Read on
/// page 2 through both, one row is new: tk-06. The count is of rows, so it reads "+1 new".
#[test]
fn adv2_a_held_row_patched_while_held_is_counted_once() {
    let mut app = paged_list("held-count");
    app.advance(Duration::from_secs(19));
    let text = app.render_text(140, 48);
    assert!(
        text.contains("+1 new") && !text.contains("+2 new"),
        "one new row (tk-06) was opened and then patched while held; the footer counts events:\n{}",
        text.lines()
            .filter(|line| line.contains(" new"))
            .collect::<Vec<_>>()
            .join("\n")
    );
}

/// The same run, then back to page 1 with `p`: the released row carries its latest state,
/// priority `urgent` from the 18s patch, and its subject from the 0s open.
#[test]
fn adv2_a_held_row_is_released_with_its_latest_patch() {
    let mut app = paged_list("held-latest");
    app.advance(Duration::from_secs(19));
    app.keys("p");
    let row = row_of(&app, "list", "tk-06").expect("tk-06 is inserted on return to page 1");
    assert_eq!(
        field(&row, "subject"),
        "Export shows wrong totals",
        "{row:?}"
    );
    assert_eq!(
        field(&row, "priority"),
        "urgent",
        "the held rows were replayed newest first, so the 0s open overwrote the 18s patch: {row:?}"
    );
}

/// A search from page 2 puts the list back on page 1. On page 1 there is nothing "paged away"
/// any more: the held row is shown if it matches, and no "+N new" count remains.
#[test]
fn adv2_a_search_back_to_page_one_does_not_strand_the_held_row() {
    let mut app = paged_list("held-search");
    app.advance(Duration::ZERO);
    assert!(app.render_text(140, 48).contains("+1 new"));
    app.focus_section("filters");
    // q=e matches "Export shows wrong totals" (tk-06) and most fixture rows.
    app.keys("/e<enter>");
    let text = app.render_text(140, 48);
    assert!(text.contains("page 1/"), "{text}");
    let shown = row_of(&app, "list", "tk-06").is_some();
    assert!(
        shown || !text.contains("+1 new"),
        "on page 1 the held row tk-06 is neither shown nor released, and \"+1 new\" stays:\n{}",
        text.lines()
            .filter(|line| line.contains("page "))
            .collect::<Vec<_>>()
            .join("\n")
    );
}

/// `resume: refetch` sets the fed section loading on reconnect. An event of the same instant as
/// the reconnect (the example's own scripts do this: activity at 20s, metrics at 55s) arrives
/// while that read is outstanding; it must not be lost. Without `coalesce`, it is dropped.
#[test]
fn adv2_an_event_at_the_reconnect_instant_is_not_lost_to_the_refetch() {
    let live = "live: {channel: tickets, effect: insert_or_patch, only_if: matches(params), when_paged_away: count_new, coalesce: 500ms}";
    let text = example_text();
    assert!(text.contains(live));
    let uncoalesced = text.replace(
        live,
        "live: {channel: tickets, effect: insert_or_patch, only_if: matches(params), when_paged_away: count_new}",
    );
    let script = "{channel: tickets, events: [\
        {at: 1s, lifecycle: reconnecting},\
        {at: 2s, lifecycle: live},\
        {at: 2s, event: tickets.TicketUpdated, payload: {id: tk-04, priority: high}}]}";
    let (mut app, reads) = app_with(
        "refetch-instant",
        &uncoalesced,
        Some(&[("tickets", script)]),
        None,
    );
    app.open_page("tickets.list", &[]);
    let before = count(&reads, "tickets.Page");
    app.advance(Duration::from_secs(1));
    app.advance(Duration::from_secs(1));
    assert!(count(&reads, "tickets.Page") > before, "the refetch ran");
    let row = row_of(&app, "list", "tk-04").expect("tk-04 is listed");
    assert_eq!(
        field(&row, "priority"),
        "high",
        "TicketUpdated tk-04 arrived with the reconnect and was dropped because the section was \
         loading: {row:?}"
    );
}

/// The coalesced form of the same interleaving: a batch opened before the reconnect is flushed
/// while the refetch is outstanding, when one clock step spans both.
#[test]
fn adv2_a_coalesced_batch_is_not_lost_to_a_refetch_in_the_same_step() {
    let script = "{channel: tickets, events: [\
        {at: 1000ms, event: tickets.TicketUpdated, payload: {id: tk-04, priority: high}},\
        {at: 1100ms, lifecycle: reconnecting},\
        {at: 1200ms, lifecycle: live}]}";
    let (mut app, _) = app_with(
        "refetch-batch",
        &example_text(),
        Some(&[("tickets", script)]),
        None,
    );
    app.open_page("tickets.list", &[]);
    app.advance(Duration::from_secs(2));
    let row = row_of(&app, "list", "tk-04").expect("tk-04 is listed");
    assert_eq!(
        field(&row, "priority"),
        "high",
        "the coalesced TicketUpdated was flushed into a loading section and dropped: {row:?}"
    );
}

/// A reconnect that ends with an event rather than a `live` beat is still a reconnect: under
/// `resume: refetch` the fed section re-reads.
#[test]
fn adv2_an_event_that_ends_a_reconnect_refetches() {
    let script = "{channel: tickets, events: [\
        {at: 1s, lifecycle: reconnecting},\
        {at: 3s, event: tickets.TicketUpdated, payload: {id: tk-04, priority: high}}]}";
    let (mut app, reads) = app_with(
        "event-ends-reconnect",
        &example_text(),
        Some(&[("tickets", script)]),
        None,
    );
    app.open_page("tickets.list", &[]);
    app.advance(Duration::from_secs(2));
    let before = count(&reads, "tickets.Page");
    app.advance(Duration::from_secs(2));
    assert!(
        count(&reads, "tickets.Page") > before,
        "the channel went reconnecting → live (by an event) and tickets.Page was not re-read"
    );
}

/// `ticket_chat` is `session: {per: ticket_id}` and its script is tk-01's session. The header of
/// tk-02 must not report tk-01's connection: at 16s tk-01's session is reconnecting.
#[test]
fn adv2_another_tickets_session_status_does_not_show_in_the_header() {
    let (mut app, _) = app_with("session-status", &example_text(), None, None);
    app.open_page("tickets.detail", &[("id", "tk-02")]);
    app.advance(Duration::from_secs(16));
    let text = app.render_text(160, 48);
    let top = text.lines().next().unwrap_or_default().to_owned();
    assert!(
        !top.contains("ticket_chat reconnecting"),
        "tk-02's header shows tk-01's session lifecycle: {top}"
    );
}

/// On tk-01's own page, `channel.ticket_chat.typing` ("true while a `TypingStarted` event is
/// younger than 5s") shows the typing icon at 1s, after `TypingStarted` at 0s.
#[test]
fn adv2_the_typing_indicator_shows_in_its_own_session() {
    let (mut app, _) = app_with("typing", &example_text(), None, None);
    app.open_page("tickets.detail", &[("id", "tk-01")]);
    app.advance(Duration::from_secs(1));
    let text = app.render_text(140, 60);
    assert!(
        text.contains("Someone is typing"),
        "TypingStarted for tk-01 played at 0s and the typing icon is not drawn on tk-01:\n{text}"
    );
}

/// A script whose `session` does not name the channel's `per` param is not scoped to any
/// ticket; delivering it to every ticket fails open.
#[test]
fn adv2_a_script_session_without_the_per_param_does_not_reach_every_ticket() {
    let script = "{channel: ticket_chat, session: {ticket: tk-01}, events: [\
        {at: 1s, event: tickets.ReplyPosted, payload: {id: m-99, ticket_id: tk-01, body: stray}}]}";
    let (mut app, _) = app_with(
        "session-missing-per",
        &example_text(),
        Some(&[("ticket_chat", script)]),
        None,
    );
    app.open_page("tickets.detail", &[("id", "tk-02")]);
    app.advance(Duration::from_secs(2));
    let bodies: Vec<String> = app
        .rows("conversation")
        .iter()
        .map(|row| field(row, "body").to_owned())
        .collect();
    assert!(
        bodies.is_empty(),
        "a session without ticket_id was delivered to tk-02: {bodies:?}"
    );
}
