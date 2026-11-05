//! A short text column is not truncated while the table has room
//! (story:a-short-text-column-is-not-truncated-while-the-table-has-room): a table's columns take
//! the width of their widest cell while the line fits the box, and only a value that does not fit
//! the room left is cut, ending in `…`, as before.

use std::path::{Path, PathBuf};

use ess_ui_tui::{App, Options};

const DOC: &str = r"
format: ess-ui/1
app: t
model: t.system
placement_profile: fat
fixtures: {dir: data, views: {t.Items: items.yaml}}
shells:
  app: {regions: {main: {kind: page_outlet}}}
navigation:
  home: items
  sections: [{name: all, pages: [items]}]
pages:
  items:
    kind: list_page
    title: Items
    sections:
      - name: items
        component: collection
        reads: {view: t.Items}
        columns: [code, name, note]
";

/// Thirty-four cells: a short text, wider than the old fixed column cap of 28.
const NAME: &str = "Quarterly shelf inventory recount";

fn screen(test: &str, note: &str, width: u16) -> String {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("ess-ui-tui-column-widths")
        .join(test);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("a stale dir is removed");
    }
    std::fs::create_dir_all(dir.join("data")).expect("the dir is created");
    let items = format!(
        "views:\n  t.Items:\n    rows:\n      - {{id: i-1, code: A-1, name: {NAME}, note: {note}}}\n"
    );
    std::fs::write(dir.join("data/items.yaml"), items).expect("the fixture is written");
    let base: PathBuf = dir;
    let mut app = App::from_text(DOC, &base, Options::new(base.join("state")))
        .unwrap_or_else(|error| panic!("{error}"));
    app.open_page("items", &[]);
    app.render_text(width, 30)
}

fn row_line(screen: &str) -> &str {
    screen
        .lines()
        .find(|line| line.contains("A-1"))
        .unwrap_or_else(|| panic!("the row is drawn:\n{screen}"))
}

#[test]
fn a_short_text_column_shows_its_whole_value_while_the_table_has_room() {
    let screen = screen("room", "ok", 140);
    let row = row_line(&screen);
    assert!(
        row.contains(NAME),
        "a {}-cell name in a 140-cell table is shown whole:\n{screen}",
        NAME.len()
    );
    assert!(!row.contains('…'), "nothing on the row is cut:\n{screen}");
}

#[test]
fn a_long_value_still_truncates_and_the_short_column_beside_it_stays_whole() {
    let long = "word ".repeat(60);
    let long = long.trim_end();
    let screen = screen("long", long, 140);
    let row = row_line(&screen);
    assert!(
        row.contains(NAME),
        "the short column keeps its whole value beside a long one:\n{screen}"
    );
    assert!(
        !row.contains(long),
        "a 300-cell note does not fit a 140-cell table:\n{screen}"
    );
    assert!(
        row.contains("word word…") || row.contains("word …"),
        "the long note is cut with an ellipsis:\n{screen}"
    );
    for line in screen.lines() {
        assert!(
            line.chars().count() <= 140,
            "no line overflows the screen:\n{screen}"
        );
    }
}

#[test]
fn without_room_a_column_keeps_its_old_cap_and_the_line_is_cut_at_the_box() {
    let long = "word ".repeat(60);
    let long = long.trim_end();
    let screen = screen("narrow", long, 40);
    let row = row_line(&screen);
    assert!(
        !row.contains(NAME),
        "a 34-cell name does not fit a 40-cell screen beside its columns:\n{screen}"
    );
    assert!(
        row.contains('…'),
        "the row is cut with an ellipsis:\n{screen}"
    );
}
