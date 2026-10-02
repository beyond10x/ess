//! Adversary pass 1: live updates against the schema's `Live` and `Channel` constructs, on the
//! virtual clock.

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
        .join("ess-ui-tui-adv1")
        .join(test);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("a stale state dir is removed");
    }
    dir
}

/// Answers from the example's fixtures and records every view read.
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

/// The example with its own scripts replaced by `scripts` (channel → script YAML), or kept when
/// `scripts` is `None`.
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

fn priority_of(app: &App, id: &str) -> String {
    app.rows("list")
        .iter()
        .find(|row| field(row, "id") == id)
        .map(|row| field(row, "priority").to_owned())
        .unwrap_or_default()
}

/// `coalesce: 500ms` on tickets.list: "batch bursts into one render". Two events 300ms apart are
/// one burst, so no render may show the first applied and the second not.
#[test]
fn adv1_coalesce_batches_a_burst_into_one_render() {
    let script = "{channel: tickets, events: [\
        {at: 1000ms, event: tickets.TicketUpdated, payload: {id: tk-04, priority: high}},\
        {at: 1300ms, event: tickets.TicketUpdated, payload: {id: tk-02, priority: low}}]}";
    let (mut app, _) = app_with(
        "coalesce",
        &example_text(),
        Some(&[("tickets", script)]),
        None,
    );
    app.open_page("tickets.list", &[]);
    assert_eq!(priority_of(&app, "tk-04"), "urgent");
    app.advance(Duration::from_millis(1100));
    let first = priority_of(&app, "tk-04") == "high";
    let second = priority_of(&app, "tk-02") == "low";
    assert_eq!(
        first, second,
        "at 1100ms one render shows tk-04 patched ({first}) and tk-02 patched ({second}): \
         the 500ms coalesce window did not batch the burst"
    );
    app.advance(Duration::from_millis(700));
    assert_eq!(priority_of(&app, "tk-04"), "high");
    assert_eq!(priority_of(&app, "tk-02"), "low");
}

/// `only_if: matches(params)` "drops live rows outside the section's filters". With the search
/// filter set to `portal`, the list holds only tk-04; a `TicketOpened` for "Export shows wrong
/// totals" is outside that filter.
#[test]
fn adv1_only_if_matches_params_drops_a_row_outside_the_search_filter() {
    let (mut app, _) = app_with("only-if-search", &example_text(), None, None);
    app.open_page("tickets.list", &[]);
    app.focus_section("filters");
    app.keys("/portal<enter>");
    let ids: Vec<String> = app
        .rows("list")
        .iter()
        .map(|row| field(row, "id").to_owned())
        .collect();
    assert_eq!(ids, ["tk-04"], "the search narrows the read");
    // tickets.yaml at 0s: TicketOpened tk-06 "Export shows wrong totals".
    app.advance(Duration::ZERO);
    let ids: Vec<String> = app
        .rows("list")
        .iter()
        .map(|row| field(row, "id").to_owned())
        .collect();
    assert_eq!(
        ids,
        ["tk-04"],
        "a live row that does not match q=portal was inserted"
    );
}

/// `when_paged_away: count_new` shows "N new" *instead of shifting rows*.
#[test]
fn adv1_count_new_while_paged_away_does_not_shift_the_page() {
    let mut options = Options::new(state_dir("paged-away"));
    options.page_size = 2;
    let (mut app, _) = app_with("paged-away", &example_text(), None, Some(options));
    app.open_page("tickets.list", &[]);
    app.focus_section("list");
    app.keys("n");
    let before = app.render_text(140, 48);
    assert!(before.contains("page 2/3"), "{before}");
    assert!(before.contains("Portal down for EU users"), "{before}");
    assert!(before.contains("API rate limit question"), "{before}");
    // 0s: TicketOpened tk-06 arrives while the reader is on page 2.
    app.advance(Duration::ZERO);
    let after = app.render_text(140, 48);
    assert!(after.contains("+1 new"), "{after}");
    assert!(
        after.contains("Portal down for EU users") && after.contains("API rate limit question"),
        "page 2 shifted under the reader instead of counting the new row:\n{after}"
    );
}

/// `resume: refetch` on the tickets channel: after a reconnect the fed section re-reads
/// (schema: `stale → refreshing` on reconnect; "memory: kept; resume refetch re-reads").
#[test]
fn adv1_resume_refetch_rereads_the_fed_section_after_a_reconnect() {
    let script = "{channel: tickets, events: [\
        {at: 1s, lifecycle: reconnecting},\
        {at: 2s, lifecycle: live}]}";
    let (mut app, reads) = app_with(
        "resume",
        &example_text(),
        Some(&[("tickets", script)]),
        None,
    );
    app.open_page("tickets.list", &[]);
    app.advance(Duration::ZERO);
    let before = count(&reads, "tickets.Page");
    assert!(before >= 1);
    app.advance(Duration::from_secs(1));
    app.advance(Duration::from_secs(1));
    assert!(
        count(&reads, "tickets.Page") > before,
        "the tickets channel reconnected with `resume: refetch` and tickets.Page was not re-read"
    );
}

/// `view_cache` is placed in `memory`, which the schema says is "lost on unmount"; on return
/// "the section re-enters loading". Leaving tickets.list and coming back must re-read it.
#[test]
fn adv1_returning_to_a_page_rereads_its_memory_view_cache() {
    let (mut app, reads) = app_with("return", &example_text(), Some(&[]), None);
    app.open_page("tickets.list", &[]);
    let before = count(&reads, "tickets.Page");
    assert!(before >= 1);
    app.open_page("overview", &[]);
    app.keys("<bs>");
    assert_eq!(app.page(), "tickets.list");
    assert!(
        count(&reads, "tickets.Page") > before,
        "tickets.list was shown again from a cache the schema places in memory (lost on unmount)"
    );
}

/// `ticket_chat` is `scope: param`, `session: {per: ticket_id}`, and its fixture script is the
/// session of tk-01. Ticket tk-02's conversation must not receive tk-01's replies.
#[test]
fn adv1_a_param_scoped_channel_does_not_leak_into_another_tickets_conversation() {
    let (mut app, _) = app_with("chat-scope", &example_text(), None, None);
    app.open_page("tickets.detail", &[("id", "tk-02")]);
    assert!(app.rows("conversation").is_empty(), "tk-02 has no messages");
    // ticket_chat.yaml at 3s: ReplyPosted m-06 for ticket tk-01.
    app.advance(Duration::from_secs(4));
    let leaked: Vec<String> = app
        .rows("conversation")
        .iter()
        .map(|row| field(row, "body").to_owned())
        .collect();
    assert!(
        leaked.is_empty(),
        "tk-02's conversation shows tk-01's replies: {leaked:?}"
    );
}

/// `stale_after: 30s` on metrics alone marks the metric stale: reconnecting at 15s, stale from
/// 45s, before the script's own `lifecycle: stale` at 50s.
#[test]
fn adv1_stale_after_alone_marks_the_metric_stale() {
    let (mut app, _) = app_with("stale-after", &example_text(), None, None);
    app.advance(Duration::from_secs(46));
    let text = app.render_text(120, 48);
    let revenue = text
        .lines()
        .find(|line| line.contains("Revenue this month"))
        .unwrap_or_else(|| panic!("{text}"));
    assert!(revenue.contains("stale"), "{text}");
}

/// A section fed by a channel past `stale_after` is marked stale (activity.feed list,
/// `stale: {mark: both}`).
#[test]
fn adv1_a_collection_fed_by_a_stale_channel_is_marked() {
    let script = "{channel: activity, events: [{at: 1s, lifecycle: reconnecting}]}";
    let (mut app, _) = app_with(
        "stale-section",
        &example_text(),
        Some(&[("activity", script)]),
        None,
    );
    app.open_page("activity.feed", &[]);
    app.advance(Duration::from_secs(62));
    let text = app.render_text(120, 48);
    let title = text
        .lines()
        .find(|line| line.contains("┌ list"))
        .unwrap_or_else(|| panic!("{text}"));
    assert!(title.contains("stale"), "{text}");
}

/// `remove_row`, `patch_row` and `refetch` each have their effect.
#[test]
fn adv1_remove_row_patch_row_and_refetch_have_their_effects() {
    let live = "live: {channel: tickets, effect: insert_or_patch, only_if: matches(params), when_paged_away: count_new, coalesce: 500ms}";
    let text = example_text();
    assert!(text.contains(live));

    let removing = text.replace(
        live,
        "live: {channel: tickets, on: [tickets.TicketClosed], effect: remove_row}",
    );
    let (mut app, _) = app_with("remove-row", &removing, None, None);
    app.open_page("tickets.list", &[]);
    app.advance(Duration::from_secs(13));
    let ids: Vec<String> = app
        .rows("list")
        .iter()
        .map(|row| field(row, "id").to_owned())
        .collect();
    assert!(!ids.contains(&"tk-01".to_owned()), "{ids:?}");
    assert!(!ids.contains(&"tk-06".to_owned()), "{ids:?}");

    let patching = text.replace(live, "live: {channel: tickets, effect: patch_row}");
    let (mut app, _) = app_with("patch-row", &patching, None, None);
    app.open_page("tickets.list", &[]);
    app.advance(Duration::from_secs(7));
    assert_eq!(priority_of(&app, "tk-04"), "high");
    assert!(
        !app.rows("list")
            .iter()
            .any(|row| field(row, "id") == "tk-06"),
        "patch_row does not insert"
    );

    let refetching = text.replace(live, "live: {channel: tickets, effect: refetch}");
    let (mut app, reads) = app_with("refetch", &refetching, None, None);
    app.open_page("tickets.list", &[]);
    let before = count(&reads, "tickets.Page");
    app.advance(Duration::from_secs(1));
    assert!(count(&reads, "tickets.Page") > before);
}
