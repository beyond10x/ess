//! Adversary pass 1: keyboard reach, degrades, small terminals, unicode, lifecycle edges.

use std::collections::BTreeMap;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::{Path, PathBuf};
use std::time::Duration;

use ess_ui_tui::{
    App, DataAdapter, FixtureAdapter, Lifecycle, Options, ReadRequest, ReadResult, TuiError,
};
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

fn open_text(test: &str, document: &str, options: Option<Options>) -> Result<App, TuiError> {
    App::from_text(
        document,
        &example_dir(),
        options.unwrap_or_else(|| Options::new(state_dir(test))),
    )
}

fn open(test: &str) -> App {
    open_text(test, &example_text(), None).unwrap_or_else(|error| panic!("{error}"))
}

/// The shell's `account` region (`kind: account_menu`) offers "Switch organization" (opens the
/// shell overlay `switch_org`) and "Sign out". A keyboard user must be able to reach them.
#[test]
fn adv1_the_account_menu_is_drawn_and_reachable_by_keyboard() {
    let mut app = open("account-menu");
    let home = app.render_text(160, 48);
    app.keys(":switch<enter>");
    let after = app.render_text(160, 48);
    assert!(
        home.contains("Sign out") || after.contains("Switch organization"),
        "the account menu region renders nothing and its actions have no key:\n{after}"
    );
}

/// partners.list declares `selection: multiple` and a bulk action "Add tag"
/// (partners.TagPartners). After selecting a row it must be shown and runnable.
#[test]
fn adv1_bulk_actions_are_offered_once_rows_are_selected() {
    let mut app = open("bulk");
    app.keys("gs");
    app.focus_section("list");
    app.keys("<space>");
    let text = app.render_text(160, 60);
    assert!(text.contains('✓'), "the row is selected:\n{text}");
    assert!(
        text.contains("Add tag"),
        "the bulk action is dropped silently:\n{text}"
    );
}

/// The schema's `Degrades.rule`: `use: [construct.degrades.$capability,
/// "capabilities.$capability.fallbacks[0]"]`, `if_none: refuse`. A graph editor without its own
/// `degrades` takes the table's first fallback for `no_graph_editor` (`collection`).
#[test]
fn adv1_a_missing_degrades_entry_takes_the_capability_tables_first_fallback() {
    let text = example_text().replace("        degrades: {no_graph_editor: collection}\n", "");
    let mut app = open_text("first-fallback", &text, None).unwrap_or_else(|error| {
        panic!("refused although the schema's capability table supplies `collection`: {error}")
    });
    app.open_page("workflows.editor", &[]);
    assert!(app.render_text(120, 48).contains("Sales lead approval"));
}

/// A capability degraded to `refuse` on a node nested inside a board widget: the refusal names
/// that node, not its section.
#[test]
fn adv1_a_refusal_inside_a_board_widget_names_the_widget() {
    let widget = "          win_rate:     {component: metric, reads: {view: deals.WinRate}, format: percent, label: Win rate}\n";
    let text = example_text();
    assert!(text.contains(widget));
    let text = text.replace(
        widget,
        &format!(
            "{widget}          trend:        {{component: chart, chart: pie, reads: {{view: deals.StageTotals}}, x: stage, series: [count], degrades: {{no_charts: refuse}}}}\n"
        ),
    );
    let error = open_text("board-refuse", &text, None)
        .err()
        .expect("refused");
    let TuiError::Refused(refusal) = &error else {
        panic!("{error}")
    };
    let path = refusal.path.to_string();
    assert!(
        path.starts_with("pages/overview/sections/board") && path.contains("trend"),
        "{error}"
    );
}

/// Rendering never panics on a tiny terminal, with overlays, the palette and unicode labels.
#[test]
fn adv1_tiny_terminals_and_resizes_never_panic() {
    let long = "Übersicht 📊 — ein sehr langer Titel, der niemals in einen schmalen Bereich passt 東京都渋谷区";
    let text = example_text()
        .replace(
            "nav: {label: Overview, synonyms: [home, kpi]}",
            &format!("nav: {{label: \"{long}\", synonyms: [home, kpi]}}"),
        )
        .replace("    title: Overview\n", &format!("    title: \"{long}\"\n"));
    let sizes = [
        (20, 5),
        (1, 1),
        (2, 2),
        (20, 2),
        (20, 3),
        (10, 4),
        (3, 30),
        (200, 5),
        (27, 6),
    ];
    let mut app = open_text("tiny", &text, None).unwrap_or_else(|error| panic!("{error}"));
    let mut failures = Vec::new();
    for page in app.page_names() {
        app.open_page(&page, &[("id", "pt-001")]);
        let mut states: Vec<String> = vec![String::from("page")];
        states.extend(
            app.overlay_names()
                .into_iter()
                .map(|name| format!("overlay {name}")),
        );
        states.push("palette".into());
        for state in states {
            if let Some(name) = state.strip_prefix("overlay ") {
                app.open_overlay(name);
            } else if state == "palette" {
                app.keys(":東京");
            }
            for (width, height) in sizes {
                let outcome = catch_unwind(AssertUnwindSafe(|| app.render_text(width, height)));
                if outcome.is_err() {
                    failures.push(format!("{page} / {state} at {width}x{height}"));
                }
            }
            app.keys("<esc>");
        }
    }
    assert!(failures.is_empty(), "panicked: {failures:#?}");
}

/// Replaces the first user's name with a wide-character name.
struct Wide(FixtureAdapter);

impl DataAdapter for Wide {
    fn read(&self, request: &ReadRequest) -> Result<ReadResult, String> {
        let mut result = self.0.read(request)?;
        if request.view == "users.All" {
            for row in &mut result.rows {
                if row["id"].as_str() == Some("us-01") {
                    row["name"] = Value::String("東京都渋谷区パートナー株式会社".into());
                }
            }
        }
        Ok(result)
    }
    fn run(&mut self, command: &str, input: &BTreeMap<String, Value>) -> Result<String, String> {
        self.0.run(command, input)
    }
    fn load_state(&self, path: &str) -> Option<Value> {
        self.0.load_state(path)
    }
    fn store_state(&mut self, path: &str, value: Value) {
        self.0.store_state(path, value);
    }
}

fn cell_of(line: &str, needle: &str) -> usize {
    let byte = line
        .find(needle)
        .unwrap_or_else(|| panic!("{needle} in {line}"));
    line[..byte].chars().count()
}

/// A table column stays aligned when a cell holds wide (two-cell) characters.
#[test]
fn adv1_table_columns_stay_aligned_with_wide_characters() {
    let document = ess_ui::load_str(&example_text()).expect("loads");
    let (inner, scripts) = FixtureAdapter::load(&document, &example_dir(), None).expect("fixtures");
    let mut app = App::with_adapter(
        document,
        Box::new(Wide(inner)),
        scripts,
        Options::new(state_dir("wide")),
    )
    .unwrap_or_else(|error| panic!("{error}"));
    app.open_page("users.list", &[]);
    let text = app.render_text(180, 60);
    let header = text
        .lines()
        .find(|line| line.to_lowercase().contains("email") && !line.contains('@'))
        .unwrap_or_else(|| panic!("{text}"));
    let wide = text
        .lines()
        .find(|line| line.contains("dana@example.com"))
        .unwrap_or_else(|| panic!("{text}"));
    let narrow = text
        .lines()
        .find(|line| line.contains("lee@example.com"))
        .unwrap_or_else(|| panic!("{text}"));
    let column = cell_of(&header.to_lowercase(), "email");
    assert_eq!(cell_of(narrow, "lee@example.com"), column, "{text}");
    assert_eq!(
        cell_of(wide, "dana@example.com"),
        column,
        "the email column of the wide-name row is shifted:\n{text}"
    );
}

/// A failing placeholder fixture read shows `failed` with retry; `R` retries it (failed →
/// loading), and the section fails again.
#[test]
fn adv1_a_failing_fixture_read_shows_failed_and_retries() {
    let text = example_text().replace(
        "fixture: fixtures/forecast.yaml",
        "fixture: fixtures/missing.yaml",
    );
    let mut options = Options::new(state_dir("failed-retry"));
    options.read_latency = Duration::from_secs(1);
    let mut app =
        open_text("failed-retry", &text, Some(options)).unwrap_or_else(|error| panic!("{error}"));
    app.open_page("forecast.quarterly", &[]);
    app.advance(Duration::from_secs(1));
    assert_eq!(app.section_state("chart"), Lifecycle::Failed);
    let screen = app.render_text(120, 48);
    assert!(
        screen.contains("failed") && screen.contains("R retries"),
        "{screen}"
    );
    app.focus_section("chart");
    app.keys("R");
    assert_eq!(app.section_state("chart"), Lifecycle::Loading);
    app.advance(Duration::from_secs(1));
    assert_eq!(app.section_state("chart"), Lifecycle::Failed);
}

/// A filter-bar search with no match shows the section's empty state; the last page stays last.
#[test]
fn adv1_empty_result_and_last_page_edges() {
    let mut options = Options::new(state_dir("edges"));
    options.page_size = 4;
    let mut app = open_text("edges", &example_text(), Some(options))
        .unwrap_or_else(|error| panic!("{error}"));
    app.keys("gs");
    app.focus_section("list");
    app.keys("nnnnn");
    let last = app.render_text(120, 60);
    assert!(last.contains("page 2/2"), "{last}");
    app.focus_section("filters");
    app.keys("/zzzz-no-such-partner<enter>");
    assert_eq!(app.section_state("list"), Lifecycle::Empty);
    let empty = app.render_text(120, 60);
    assert!(empty.contains("No partners yet"), "{empty}");
}
