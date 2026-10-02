//! The partner-portal example driven headless through ratatui's `TestBackend`.

use std::cell::Cell;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::Duration;

use ess_ui_tui::live::Script;
use ess_ui_tui::{
    App, DataAdapter, FixtureAdapter, Lifecycle, Options, ReadRequest, ReadResult, TuiError,
};
use serde_yaml::Value;

const WIDTH: u16 = 120;
const HEIGHT: u16 = 48;

fn example_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../examples/partner-portal")
}

fn example_text() -> String {
    std::fs::read_to_string(example_dir().join("ui.yaml")).expect("the example reads")
}

fn state_dir(test: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("ess-ui-tui")
        .join(test);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("a stale state dir is removed");
    }
    dir
}

fn open(test: &str) -> App {
    App::from_path(
        &example_dir().join("ui.yaml"),
        Options::new(state_dir(test)),
    )
    .unwrap_or_else(|error| panic!("{error}"))
}

fn open_text(test: &str, document: &str) -> Result<App, TuiError> {
    App::from_text(document, &example_dir(), Options::new(state_dir(test)))
}

fn screen(app: &mut App) -> String {
    app.render_text(WIDTH, HEIGHT)
}

fn scratch(name: &str, text: &str) {
    if let Some(dir) = std::env::var_os("ESS_UI_TUI_SCREENS") {
        let dir = PathBuf::from(dir);
        std::fs::create_dir_all(&dir).expect("the screen dir exists");
        std::fs::write(dir.join(name), text).expect("the screen is written");
    }
}

fn field<'a>(row: &'a serde_yaml::Value, key: &str) -> &'a str {
    row[key].as_str().unwrap_or_default()
}

#[test]
fn the_example_opens_navigates_filters_opens_an_overlay_and_plays_a_live_event() {
    let mut app = open("walkthrough");
    let home = screen(&mut app);
    assert_eq!(app.page(), "overview");
    assert!(home.contains("Partner portal"), "{home}");
    assert!(
        home.contains("Sales"),
        "the navigation pane lists sections:\n{home}"
    );
    assert!(home.contains("Revenue this month"), "{home}");
    scratch("overview.txt", &home);

    // g + letter opens the first entry of the navigation section with that initial.
    app.keys("gs");
    assert_eq!(app.page(), "partners.list");
    let list = screen(&mut app);
    assert!(list.contains("Cedar Partners"), "{list}");
    assert!(list.contains("Birch Channel"), "{list}");

    // Focus the collection and filter it.
    app.focus_section("list");
    app.keys("/cedar<enter>");
    let filtered = screen(&mut app);
    assert!(filtered.contains("Cedar Partners"), "{filtered}");
    assert!(!filtered.contains("Birch Channel"), "{filtered}");
    scratch("partners-filtered.txt", &filtered);

    // `e` is the edit row action: it opens the edit drawer as a full-screen pane.
    app.keys("e");
    let overlay = screen(&mut app);
    assert!(overlay.contains("Edit partner"), "{overlay}");
    assert!(
        overlay.contains("esc"),
        "the pane says how to close it:\n{overlay}"
    );
    assert!(
        overlay.contains("Cedar Partners"),
        "the drawer loads its row by the opener's id:\n{overlay}"
    );
    scratch("partner-edit-overlay.txt", &overlay);
    app.keys("<esc>");
    assert!(!screen(&mut app).contains("Edit partner"));

    // The fuzzy palette opens a page by its label.
    app.keys(":tickt<enter>");
    assert_eq!(app.page(), "tickets.list");
    assert_eq!(app.section_state("list"), Lifecycle::Ready);
    let before = app.rows("list");
    let tk04 = before
        .iter()
        .find(|row| field(row, "id") == "tk-04")
        .unwrap();
    assert_eq!(field(tk04, "priority"), "urgent");

    // tickets.yaml: at 6s TicketUpdated patches tk-04 to high; the list coalesces bursts over
    // 500ms, so the patch is drawn when that window closes.
    app.advance(Duration::from_millis(6500));
    let after = app.rows("list");
    let tk04 = after
        .iter()
        .find(|row| field(row, "id") == "tk-04")
        .unwrap();
    assert_eq!(field(tk04, "priority"), "high", "{after:?}");
    assert!(
        after.iter().any(|row| field(row, "id") == "tk-06"),
        "the 0s TicketOpened was inserted: {after:?}"
    );
    let live = screen(&mut app);
    let patched = live
        .lines()
        .find(|line| line.contains("Portal down for EU users"))
        .unwrap_or_else(|| panic!("{live}"));
    assert!(patched.contains("high"), "{patched}");
    assert!(live.contains("Export shows wrong totals"), "{live}");
    assert!(
        live.contains("tickets"),
        "the status segment names the channel:\n{live}"
    );
}

#[test]
fn a_stale_channel_marks_its_metric() {
    let mut app = open("stale");
    app.advance(Duration::from_secs(40));
    let reconnecting = screen(&mut app);
    let revenue = |text: &str| {
        text.lines()
            .find(|line| line.contains("Revenue this month"))
            .map_or_else(|| panic!("{text}"), str::to_owned)
    };
    assert!(!revenue(&reconnecting).contains("stale"), "{reconnecting}");
    assert!(reconnecting.contains("reconnecting"), "{reconnecting}");
    app.advance(Duration::from_secs(10));
    let stale = screen(&mut app);
    assert!(revenue(&stale).contains("stale"), "{stale}");
    scratch("overview-stale.txt", &stale);
}

#[test]
fn a_missing_degrade_takes_the_schema_fallback_and_a_refusing_one_names_its_node() {
    // Degrades.rule: without its own entry a node takes the capability table's first fallback.
    let text = example_text().replace("        chart: line\n", "        chart: pie\n");
    let mut app = open_text("fallback-chart", &text).unwrap_or_else(|error| panic!("{error}"));
    app.open_page("forecast.quarterly", &[]);
    let chart = screen(&mut app);
    let header = chart
        .lines()
        .find(|line| line.contains("month") && line.contains("committed"))
        .unwrap_or_else(|| panic!("no_charts falls back to `table`:\n{chart}"));
    assert!(header.contains("best_case"), "{chart}");
    assert!(
        chart.contains("2026-10") && chart.contains("310000"),
        "{chart}"
    );
    assert!(
        !chart.chars().any(|c| ('▁'..='█').contains(&c)),
        "a pie is not drawn as a sparkline:\n{chart}"
    );

    // A degrade declared as `refuse` refuses, naming the node.
    let text = example_text().replace("no_graph_editor: collection", "no_graph_editor: refuse");
    let error = open_text("refused-graph-refuse", &text)
        .err()
        .expect("refused");
    let TuiError::Refused(refusal) = &error else {
        panic!("{error}")
    };
    assert_eq!(
        refusal.path.to_string(),
        "pages/workflows.editor/sections/editor"
    );
    assert!(refusal.message.contains("no_graph_editor"), "{error}");

    // A fallback the schema does not list for the capability refuses, naming the node.
    let text = example_text().replace(
        "        chart: line\n",
        "        chart: pie\n        degrades: {no_charts: hologram}\n",
    );
    let error = open_text("refused-chart", &text).err().expect("refused");
    assert!(
        error
            .to_string()
            .starts_with("pages/forecast.quarterly/sections/chart"),
        "{error}"
    );
    assert!(error.to_string().contains("no_charts: hologram"), "{error}");
}

#[test]
fn stale_after_alone_marks_a_reconnecting_channels_metric() {
    // No `lifecycle: stale` beat: the channel reconnects at 1s and `stale_after: 30s` decides.
    let document = ess_ui::load_str(&example_text()).expect("the example loads");
    let (adapter, _) =
        FixtureAdapter::load(&document, &example_dir(), None).expect("the fixtures load");
    let script = Script::parse(
        "metrics",
        &serde_yaml::from_str(
            "{channel: metrics, events: [\
             {at: 0s, event: metrics.Live, payload: {revenue_mtd: 1, open_deals: 2, overdue_invoices: 3}},\
             {at: 1s, lifecycle: reconnecting}]}",
        )
        .expect("script yaml"),
    )
    .expect("the script parses");
    let mut app = App::with_adapter(
        document,
        Box::new(adapter),
        vec![script],
        Options::new(state_dir("stale-after")),
    )
    .unwrap_or_else(|error| panic!("{error}"));
    let revenue = |app: &mut App| {
        let text = screen(app);
        text.lines()
            .find(|line| line.contains("Revenue this month"))
            .map_or_else(|| panic!("{text}"), str::to_owned)
    };
    app.advance(Duration::from_secs(30));
    assert!(
        !revenue(&mut app).contains("stale"),
        "29s down is not stale"
    );
    app.advance(Duration::from_secs(1));
    assert!(revenue(&mut app).contains("stale"), "30s down is stale");
}

#[test]
fn columns_and_areas_degrade_to_a_stack_in_section_order() {
    // `order: section_declaration` is the order of the loaded page, its kind's sections merged
    // in: for the overview that puts the dashboard kind's `board` first.
    let document = ess_ui::load_path(&example_dir().join("ui.yaml")).expect("the example loads");
    for (page, layout) in [("overview", "areas"), ("partners.detail", "columns")] {
        let declared: Vec<String> = document.pages[page]
            .sections
            .iter()
            .map(|section| format!("┌ {} ", section.name))
            .collect();
        assert!(
            declared.len() > 2,
            "{page} places several sections in {layout}"
        );
        let mut app = App::from_path(
            &example_dir().join("ui.yaml"),
            Options::new(state_dir("layout")),
        )
        .unwrap_or_else(|error| panic!("{error}"));
        app.open_page(page, &[("id", "pt-001")]);
        let text = app.render_text(WIDTH, 120);
        let at: Vec<usize> = declared
            .iter()
            .map(|title| {
                text.find(title.as_str())
                    .unwrap_or_else(|| panic!("{title}:\n{text}"))
            })
            .collect();
        assert!(
            at.windows(2).all(|pair| pair[0] < pair[1]),
            "{page}: {declared:?}\n{text}"
        );
    }
}

#[test]
fn state_placement_follows_the_document() {
    let dir = state_dir("placement");
    let mut app = App::from_path(&example_dir().join("ui.yaml"), Options::new(dir.clone()))
        .unwrap_or_else(|error| panic!("{error}"));
    app.keys("gs");
    // url state is the TUI's own location.
    app.focus_section("filters");
    app.keys("/cedar<enter>");
    assert_eq!(app.location(), "/partners.list?search=cedar");

    // A session_storage draft goes to a file under the state dir, keyed by app, the actor's
    // user id and account id (session.Me: us-01 in org-01).
    app.open_page("tickets.detail", &[("id", "tk-01")]);
    app.focus_section("reply");
    app.keys("<enter>hello<esc>");
    let stored = std::fs::read_to_string(
        dir.join("portal")
            .join("us-01")
            .join("org-01")
            .join("session_storage.yaml"),
    )
    .expect("the draft is stored");
    assert!(stored.contains("hello"), "{stored}");

    // A sensitive draft never reaches a file.
    app.open_page("auth.sign_in", &[]);
    app.focus_section("form");
    app.keys("<enter>me@example.com<tab>hunter2<esc>");
    for file in walk(&dir) {
        let text = std::fs::read_to_string(&file).unwrap_or_default();
        assert!(
            !text.contains("hunter2"),
            "{} holds a secret",
            file.display()
        );
    }
    assert!(screen(&mut app).contains("•••••••"));
}

fn walk(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    for entry in std::fs::read_dir(dir).into_iter().flatten().flatten() {
        let path = entry.path();
        if path.is_dir() {
            files.extend(walk(&path));
        } else {
            files.push(path);
        }
    }
    files
}

#[test]
fn sections_show_loading_empty_and_failed() {
    let mut options = Options::new(state_dir("lifecycle"));
    options.read_latency = Duration::from_secs(1);
    let mut app = App::from_path(&example_dir().join("ui.yaml"), options)
        .unwrap_or_else(|error| panic!("{error}"));
    app.open_page("tickets.list", &[]);
    assert_eq!(app.section_state("list"), Lifecycle::Loading);
    assert!(screen(&mut app).contains("loading"));
    app.advance(Duration::from_secs(1));
    assert_eq!(app.section_state("list"), Lifecycle::Ready);

    let mut app = open("empty");
    app.open_page("partners.detail", &[("id", "pt-003")]);
    assert_eq!(app.section_state("contacts"), Lifecycle::Empty);

    let text = example_text().replace("{view: tickets.Page, params", "{view: tickets.Gone, params");
    let mut app = open_text("failed", &text).unwrap_or_else(|error| panic!("{error}"));
    app.open_page("tickets.list", &[]);
    assert_eq!(app.section_state("list"), Lifecycle::Failed);
    assert!(screen(&mut app).contains("tickets.Gone"));
}

#[test]
fn every_page_and_overlay_of_the_example_renders() {
    let document = ess_ui::load_path(&example_dir().join("ui.yaml")).expect("the example loads");
    let mut app = open("every-page");
    for page in app.page_names() {
        app.open_page(&page, &[("id", "pt-001")]);
        let def = &document.pages[&page];
        let text = app.render_text(WIDTH, 200);
        let title = def.title.clone().unwrap_or_else(|| page.clone());
        assert!(text.contains(&title), "{page}: title {title}\n{text}");
        // Every section is drawn with something inside its box.
        for section in &def.sections {
            let heading = format!("┌ {} ", section.name);
            let lines: Vec<&str> = text.lines().collect();
            let at = lines
                .iter()
                .position(|line| line.contains(&heading))
                .unwrap_or_else(|| panic!("{page}: section {}\n{text}", section.name));
            let column = lines[at][..lines[at].find(&heading).unwrap()]
                .chars()
                .count();
            let inside: String = lines[at + 1].chars().skip(column + 1).take(60).collect();
            assert!(
                inside
                    .chars()
                    .any(|c| !c.is_whitespace() && c != '│' && c != '└'),
                "{page}: section {} draws an empty box\n{text}",
                section.name
            );
        }
        for overlay in app.overlay_names() {
            let def_overlay = def
                .overlays
                .get(&overlay)
                .unwrap_or_else(|| &document.shells[&def.shell].overlays[&overlay]);
            app.open_overlay(&overlay);
            let text = screen(&mut app);
            let title = def_overlay.title.clone().unwrap_or_else(|| overlay.clone());
            assert!(text.contains(&title), "{page}/{overlay}: title\n{text}");
            let body = overlay_body_marker(&def_overlay.body);
            assert!(
                text.contains(&body),
                "{page}/{overlay}: body marker `{body}`\n{text}"
            );
            app.keys("<esc>");
        }
    }
}

/// Text only an overlay's body draws: its first field's label, or its confirm button.
fn overlay_body_marker(body: &ess_ui::Body) -> String {
    let label = |field: &ess_ui::Field| {
        field
            .label
            .clone()
            .unwrap_or_else(|| field.field.clone())
            .chars()
            .take(12)
            .collect::<String>()
    };
    match body {
        ess_ui::Body::Composite(ess_ui::Composite::Form(form)) => {
            let first = form
                .fields
                .first()
                .or_else(|| form.groups.first().and_then(|group| group.fields.first()))
                .or_else(|| {
                    form.tabs.first().and_then(|tab| match &tab.fields {
                        Some(ess_ui::TabFields::Fields(fields)) => fields.first(),
                        _ => None,
                    })
                })
                .expect("a form overlay has a field");
            label(first)
        }
        ess_ui::Body::Composite(ess_ui::Composite::Record(record)) => {
            label(record.fields.first().expect("a record overlay has a field"))
        }
        ess_ui::Body::Composite(ess_ui::Composite::Confirm(confirm)) => confirm
            .confirm_label
            .clone()
            .unwrap_or_else(|| "Confirm".into()),
        other => panic!("no marker for {other:?}"),
    }
}

#[test]
fn the_graph_editor_renders_as_its_declared_collection_and_the_chart_as_a_sparkline() {
    let mut app = open("degraded");
    app.open_page("workflows.editor", &[]);
    let text = screen(&mut app);
    assert!(text.contains("Sales lead approval"), "{text}");
    app.open_page("forecast.quarterly", &[]);
    let text = screen(&mut app);
    assert!(text.contains("committed"), "{text}");
    assert!(text.chars().any(|c| ('▁'..='█').contains(&c)), "{text}");
}

#[test]
fn collections_page_sort_and_run_row_actions_through_confirms() {
    let mut options = Options::new(state_dir("paging"));
    options.page_size = 4;
    let mut app = App::from_path(&example_dir().join("ui.yaml"), options)
        .unwrap_or_else(|error| panic!("{error}"));
    app.keys("gs");
    app.focus_section("list");
    let first = screen(&mut app);
    assert!(
        first.contains("Alder Resale") && !first.contains("Fir Cloud"),
        "{first}"
    );
    assert!(first.contains("page 1/2"), "{first}");
    app.keys("n");
    let second = screen(&mut app);
    assert!(
        second.contains("Fir Cloud") && second.contains("page 2/2"),
        "{second}"
    );
    app.keys("p");

    // `s` moves to the next allowed sort field (revenue_ytd, ascending), `S` flips it.
    app.keys("s");
    let ascending = screen(&mut app);
    let line_of = |text: &str, name: &str| text.lines().position(|line| line.contains(name));
    assert!(ascending.contains("sort revenue_ytd ↑"), "{ascending}");
    assert!(
        line_of(&ascending, "Dogwood Systems") < line_of(&ascending, "Cedar Partners"),
        "{ascending}"
    );
    app.keys("S");
    let descending = screen(&mut app);
    assert!(
        descending.contains("Birch Channel") && !descending.contains("Dogwood Systems"),
        "{descending}"
    );

    // A row action whose `visible` fails is not offered; an inline confirm runs its command.
    app.keys(":invoices<enter>");
    app.focus_section("list");
    assert!(!screen(&mut app).contains("r remind"));
    app.keys("jj");
    assert!(screen(&mut app).contains("r remind"));
    app.keys("r");
    let confirm = screen(&mut app);
    assert!(confirm.contains("Send payment reminder"), "{confirm}");
    app.keys("y");
    let done = screen(&mut app);
    assert!(
        done.contains("invoices.SendReminder accepted (fixture): invoice_id=in-03"),
        "{done}"
    );
}

#[test]
fn a_confirm_shows_its_references() {
    let mut app = open("confirm");
    app.keys("gs");
    app.focus_section("list");
    app.keys("d");
    let text = screen(&mut app);
    assert!(text.contains("Delete partner"), "{text}");
    assert!(
        text.contains("Cloud rollout"),
        "the used-by view is shown first:\n{text}"
    );
    assert!(text.contains("house account"), "{text}");
}

/// The example's fixtures, with an actor whose organization becomes org-02 once
/// `session.SwitchOrganization` has run.
struct Switching {
    inner: FixtureAdapter,
    switched: Cell<bool>,
}

impl DataAdapter for Switching {
    fn read(&self, request: &ReadRequest) -> Result<ReadResult, String> {
        let mut result = self.inner.read(request)?;
        if request.view == "session.Me" && self.switched.get() {
            for row in &mut result.rows {
                row["organization"]["id"] = Value::String("org-02".into());
            }
        }
        Ok(result)
    }
    fn run(&mut self, command: &str, input: &BTreeMap<String, Value>) -> ess_ui::binding::Answer {
        if command == "session.SwitchOrganization" {
            self.switched.set(true);
        }
        self.inner.run(command, input)
    }
    fn load_state(&self, path: &str) -> Option<Value> {
        self.inner.load_state(path)
    }
    fn store_state(&mut self, path: &str, value: Value) {
        self.inner.store_state(path, value);
    }
}

#[test]
fn storage_is_rekeyed_when_the_actor_switches_organization() {
    let dir = state_dir("rekey");
    let document = ess_ui::load_str(&example_text()).expect("the example loads");
    let (inner, scripts) =
        FixtureAdapter::load(&document, &example_dir(), None).expect("the fixtures load");
    let adapter = Switching {
        inner,
        switched: Cell::new(false),
    };
    let mut app = App::with_adapter(
        document,
        Box::new(adapter),
        scripts,
        Options::new(dir.clone()),
    )
    .unwrap_or_else(|error| panic!("{error}"));
    let file = |account: &str| {
        dir.join("portal")
            .join("us-01")
            .join(account)
            .join("session_storage.yaml")
    };
    let reply = |app: &mut App, text: &str| {
        app.open_page("tickets.detail", &[("id", "tk-01")]);
        app.focus_section("reply");
        app.keys(&format!("<enter>{text}<esc>"));
    };
    reply(&mut app, "before-switch");
    let first = std::fs::read_to_string(file("org-01")).expect("org-01 holds the draft");
    assert!(first.contains("before-switch"), "{first}");

    app.keys(":switch<enter>");
    assert!(screen(&mut app).contains("Switch organization"));
    app.keys("<c-s>");
    reply(&mut app, "after-switch");
    let second = std::fs::read_to_string(file("org-02")).expect("org-02 holds the draft");
    assert!(second.contains("after-switch"), "{second}");
    assert!(
        !second.contains("before-switch"),
        "the new account starts its own session storage: {second}"
    );
    let first = std::fs::read_to_string(file("org-01")).unwrap_or_default();
    assert!(
        !first.contains("after-switch"),
        "the old account's file is not written after the switch: {first}"
    );
    assert!(!screen(&mut app).contains("before-switch"));
}

/// A record page that narrows its read by a single-value page param shows the row it names,
/// as the React output does (`matchesParams`), not the first row of the view.
#[test]
fn a_record_page_shows_the_row_its_single_value_param_names() {
    let dir = state_dir("record-by-param").join("doc");
    std::fs::create_dir_all(&dir).expect("the document dir exists");
    std::fs::write(
        dir.join("things.yaml"),
        "views:\n  things.Page:\n    rows:\n      - {id: th-1, name: Anvil}\n      \
         - {id: th-2, name: Bolt}\n",
    )
    .expect("the fixture is written");
    std::fs::write(
        dir.join("ui.yaml"),
        r"format: ess-ui/1
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
          - {name: open, navigate: {to: things.record, params: {id: row.id}}, label: Open}
  things.record:
    kind: detail_page
    title: Thing
    params: {id: string}
    sections:
      - name: summary
        component: record
        reads: {view: things.Page, params: {id: params.id}}
        fields: [id, name]
",
    )
    .expect("the document is written");
    let mut app = App::from_path(
        &dir.join("ui.yaml"),
        Options::new(state_dir("record-by-param-state")),
    )
    .unwrap_or_else(|error| panic!("{error}"));
    app.open_page("things.list", &[]);
    assert_eq!(app.rows("list").len(), 2, "the list shows both rows");
    app.focus_section("list");
    app.keys("j<enter>");
    assert_eq!(app.page(), "things.record");
    let record = screen(&mut app);
    assert!(
        record.contains("Bolt"),
        "the record of th-2 shows th-2:\n{record}"
    );
    assert!(
        !record.contains("Anvil"),
        "the record of th-2 does not show th-1:\n{record}"
    );
    let rows = app.rows("summary");
    assert_eq!(
        rows.iter().map(|row| field(row, "id")).collect::<Vec<_>>(),
        ["th-2"],
        "the read is narrowed to the row its param names"
    );
}
