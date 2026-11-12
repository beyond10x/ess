//! The terminal draws what the document declares for a row and for a read that has not started
//! (story:each-row-and-each-waiting-read-draws-what-the-document-declares, beyond10x/ess#723):
//! a column or field `visible:` that reads `row.` is judged per row, a read waiting on another
//! section's selection draws `states.empty`, and a board row without a widget names why.

use std::path::{Path, PathBuf};

use ess_ui_tui::{App, Options};

const DOCUMENT: &str = "format: ess-ui/1
app: desk
model: desk.system
placement_profile: fat
fixtures:
  views:
    desk.Orders: orders.yaml
    desk.Quiet: quiet.yaml
    desk.Nobody: nobody.yaml
    desk.Detail: detail.yaml
    desk.Widgets: widgets.yaml
shells:
  app: {regions: {main: {kind: page_outlet}}}
navigation:
  home: orders
  sections: [{name: all, pages: [orders]}]
pages:
  orders:
    kind: list_page
    title: Orders
    sections:
      - name: table
        component: collection
        reads: {view: desk.Orders}
        columns:
          - id
          - {field: note, visible: row.has_note}
          - {field: secret, visible: params.reveal}
      - name: cards
        component: collection
        style: cards
        reads: {view: desk.Orders}
        columns:
          - id
          - {field: note, visible: row.has_note}
          - {field: secret, visible: params.reveal}
      - name: noted
        component: record
        reads: {view: desk.Orders}
        fields:
          - id
          - {field: note, visible: row.has_note}
      - name: quiet
        component: record
        reads: {view: desk.Quiet}
        fields:
          - id
          - {field: note, visible: row.has_note}
      - name: pick
        component: collection
        selection: single
        reads: {view: desk.Nobody}
        columns: [id]
      - name: detail
        component: record
        depends_on: pick
        reads: {view: desk.Detail, params: {id: section.pick.selection.id}}
        fields: [id]
        states:
          empty:
            title: No order
            message: Pick an order
            action: {name: reload, label: Reload, navigate: {to: orders}}
      - name: bare
        component: record
        depends_on: pick
        reads: {view: desk.Detail, params: {id: section.pick.selection.id}}
        fields: [id]
      - name: follows
        component: record
        depends_on: table
        reads: {view: desk.Orders, params: {id: section.table.selection.id}}
        fields: [id]
      - name: board
        component: board
        reads: {view: desk.Widgets}
        widget_by: kind
        widgets:
          gauge: {primitive: text, text: row.id}
";

fn scratch(test: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("ess-ui-tui-row-states")
        .join(test);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("a stale dir is removed");
    }
    std::fs::create_dir_all(&dir).expect("the dir is made");
    for (file, text) in [
        (
            "orders.yaml",
            "views:\n  desk.Orders:\n    rows:\n      - {id: o1, note: fragile, has_note: true, secret: sx}\n      - {id: o2, note: stale, has_note: false, secret: sy}\n",
        ),
        (
            "quiet.yaml",
            "views:\n  desk.Quiet:\n    rows:\n      - {id: q1, note: unseen, has_note: false}\n",
        ),
        ("detail.yaml", "views:\n  desk.Detail:\n    rows: []\n"),
        ("nobody.yaml", "views:\n  desk.Nobody:\n    rows: []\n"),
        (
            "widgets.yaml",
            "views:\n  desk.Widgets:\n    rows:\n      - {widget: w1, id: w1, kind: gauge, value: 1}\n      - {widget: w2, id: w2, value: 7}\n      - {widget: w3, id: w3, kind: ghost, value: 3}\n",
        ),
    ] {
        std::fs::write(dir.join(file), text).expect("the fixture is written");
    }
    dir
}

fn screen(test: &str, document: &str) -> String {
    let dir = scratch(test);
    App::from_text(document, &dir, Options::new(dir.join("state")))
        .unwrap_or_else(|error| panic!("{error}"))
        .render_text(110, 200)
}

/// The lines of `screen` from the one holding `from` up to the next section border.
fn section<'a>(screen: &'a str, name: &str) -> Vec<&'a str> {
    let heading = format!(" {name} ");
    let start = screen
        .lines()
        .position(|line| line.contains(&heading))
        .unwrap_or_else(|| panic!("section {name} is not shown:\n{screen}"));
    let mut lines: Vec<&str> = screen.lines().skip(start + 1).collect();
    if let Some(end) = lines.iter().position(|line| line.contains('┌')) {
        lines.truncate(end);
    }
    lines
}

#[test]
fn a_column_condition_reading_the_row_is_judged_per_row() {
    let screen = screen("per_row", DOCUMENT);
    for name in ["table", "cards"] {
        let lines = section(&screen, name).join("\n");
        assert!(
            lines.contains("fragile"),
            "{name} draws the first row's note:\n{screen}"
        );
        assert!(
            !lines.contains("stale"),
            "{name} leaves the second row's note out:\n{screen}"
        );
        assert!(
            !lines.contains("sx") && !lines.contains("sy"),
            "a column that reads no row is hidden whole in {name}:\n{screen}"
        );
    }
    let table = section(&screen, "table");
    assert!(
        table
            .iter()
            .any(|line| line.to_lowercase().contains("note")),
        "the note header stays:\n{screen}"
    );
    let noted = section(&screen, "noted").join("\n");
    assert!(
        noted.to_lowercase().contains("note"),
        "a record whose row has a note draws it:\n{screen}"
    );
    let quiet = section(&screen, "quiet").join("\n");
    assert!(
        !quiet.contains("unseen"),
        "a record whose row has none omits it:\n{screen}"
    );
}

#[test]
fn a_read_that_has_not_started_shows_its_empty_state() {
    let screen = screen("idle", DOCUMENT);
    let detail = section(&screen, "detail").join("\n");
    assert!(!detail.contains("loading"), "no skeleton:\n{screen}");
    assert!(detail.contains("No order"), "the empty title:\n{screen}");
    assert!(
        detail.contains("Pick an order"),
        "the empty message:\n{screen}"
    );
    assert!(detail.contains("[Reload]"), "the empty action:\n{screen}");
    let bare = section(&screen, "bare").join("\n");
    assert!(!bare.contains("loading"), "no skeleton:\n{screen}");
    assert!(
        bare.contains("Nothing to show yet"),
        "the neutral line:\n{screen}"
    );
    let follows = section(&screen, "follows").join("\n");
    assert!(
        follows.contains("o1"),
        "a read whose section has a row to follow is read:\n{screen}"
    );
}

#[test]
fn a_board_row_without_a_widget_names_its_kind() {
    let screen = screen("board", DOCUMENT);
    let board = section(&screen, "board").join("\n");
    assert!(board.contains("w1"), "the row with a widget:\n{screen}");
    assert!(
        board.contains("no kind"),
        "a row without a kind names the field:\n{screen}"
    );
    assert!(
        board.contains("No widget for ghost"),
        "a kind no widget names is named:\n{screen}"
    );
}
