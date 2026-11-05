//! Adversary pass 1 over the UI work of the 0.87.0 wave: terminal behaviour checked and found to
//! hold, pinned where the wave's own tests would let a mutant through.
//!
//! - The 28-cell floor of `column_widths` (story:a-short-text-column-is-not-truncated-while-the-
//!   table-has-room): `column_widths.rs` stays green when the floor is removed, because its narrow
//!   case only asserts that the name is cut. Pinned here by the exact cut.
//! - The boundary: a table whose line needs exactly the box shows every column whole, one cell
//!   less cuts; widening never cuts again; no line is wider than the screen, wide letters included.
//! - Negative durations in the terminal (story:a-duration-renders-as-hours-minutes-seconds): the
//!   wave tests them only in the React runtime.

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

/// Thirty-four cells.
const NAME: &str = "Quarterly shelf inventory recount";
/// Seventeen wide letters: thirty-four cells.
const WIDE: &str = "棚卸再集計表棚卸再集計表棚卸再集計";

fn base(test: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("ess-ui-tui-adversary-w087-ui-pins-pass1")
        .join(test);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("a stale dir is removed");
    }
    std::fs::create_dir_all(dir.join("data")).expect("the dir is created");
    dir
}

fn app(test: &str, name: &str, note: &str) -> App {
    let dir = base(test);
    let items = format!(
        "views:\n  t.Items:\n    rows:\n      - {{id: i-1, code: A-1, name: '{name}', note: '{note}'}}\n"
    );
    std::fs::write(dir.join("data/items.yaml"), items).expect("the fixture is written");
    let mut app = App::from_text(DOC, &dir, Options::new(dir.join("state")))
        .unwrap_or_else(|error| panic!("{error}"));
    app.open_page("items", &[]);
    app
}

fn row_line(screen: &str) -> String {
    screen
        .lines()
        .find(|line| line.contains("A-1"))
        .unwrap_or_else(|| panic!("the row is drawn:\n{screen}"))
        .to_owned()
}

/// Terminal cells of a line of `render_text`, which writes one character per cell: a wide letter
/// is followed by the filler of its second cell.
fn cells(line: &str) -> usize {
    line.chars().count()
}

/// The letters a line of `render_text` shows, without the filler after each wide letter.
fn letters(line: &str) -> String {
    let mut shown = String::new();
    let mut filler = false;
    for letter in line.chars() {
        if filler && letter == ' ' {
            filler = false;
            continue;
        }
        filler = ('\u{3000}'..='\u{9fff}').contains(&letter);
        shown.push(letter);
    }
    shown
}

#[test]
fn pin_without_room_a_column_is_cut_at_exactly_28_cells() {
    let mut app = app("floor", NAME, &"word ".repeat(60));
    let screen = app.render_text(40, 30);
    let row = row_line(&screen);
    // 27 letters and the ellipsis: the floor. Without it the name would shrink to fit 40 cells.
    assert!(
        row.contains("Quarterly shelf inventory r…"),
        "a cut column keeps 28 cells:\n{screen}"
    );
    assert!(
        !row.contains("Quarterly shelf inventory re"),
        "and no more than 28:\n{screen}"
    );
}

#[test]
fn pin_a_table_that_just_fits_shows_every_column_whole_and_widening_never_cuts() {
    let mut first_whole = None;
    for width in 30u16..=140 {
        let mut app = app(&format!("fit-{width}"), NAME, "ok");
        let screen = app.render_text(width, 30);
        for line in screen.lines() {
            assert!(
                cells(line) <= usize::from(width),
                "no line is wider than {width}:\n{screen}"
            );
        }
        let row = row_line(&screen);
        let whole = row.contains(NAME) && row.contains("ok") && !row.contains('…');
        match first_whole {
            None if whole => first_whole = Some(width),
            Some(at) => assert!(whole, "whole at {at}, cut again at {width}:\n{screen}"),
            None => {}
        }
    }
    let at = first_whole.expect("the table fits at some width up to 140");
    // The line needs 2 (mark) + 4 + 2 + 34 + 2 + 4 = 48 cells; the box adds at most its border.
    assert!(
        (48..=50).contains(&at),
        "the line of 48 cells is whole from 48 to 50 screen cells, not from {at}"
    );
}

#[test]
fn pin_a_wide_letter_column_is_measured_in_cells() {
    let mut roomy = app("wide-room", WIDE, "ok");
    let screen = roomy.render_text(140, 30);
    let row = row_line(&screen);
    assert!(
        letters(&row).contains(WIDE),
        "34 cells of wide letters fit a 140-cell table:\n{screen}"
    );
    // The next column starts where its heading does: the wide column is padded in cells.
    let heading = screen
        .lines()
        .find(|line| line.contains("code") && line.contains("note"))
        .unwrap_or_else(|| panic!("the heading is drawn:\n{screen}"));
    let at = |line: &str, word: &str| {
        let byte = line
            .find(word)
            .unwrap_or_else(|| panic!("{word} in {line}"));
        line[..byte].chars().count()
    };
    assert_eq!(at(heading, "note"), at(&row, "ok"), "{screen}");
    let mut narrow = app("wide-narrow", WIDE, &"word ".repeat(60));
    let screen = narrow.render_text(60, 30);
    for line in screen.lines() {
        assert!(cells(line) <= 60, "no line is wider than 60:\n{screen}");
    }
    let row = row_line(&screen);
    assert!(!letters(&row).contains(WIDE), "{screen}");
    assert!(
        row.contains('…'),
        "the wide column is cut with an ellipsis:\n{screen}"
    );
}

const DURATIONS: &str = r"
format: ess-ui/1
app: t
model: t.system
placement_profile: fat
fixtures: {dir: data, views: {t.Calls: calls.yaml}}
shells:
  app: {regions: {main: {kind: page_outlet}}}
navigation:
  home: calls
  sections: [{name: all, pages: [calls]}]
pages:
  calls:
    kind: list_page
    title: Calls
    sections:
      - name: calls
        component: collection
        reads: {view: t.Calls}
        columns:
          - {field: title}
          - {field: talk, as: duration}
          - {field: hold, as: duration, unit: s}
";

#[test]
fn pin_negative_and_fractional_durations_in_the_terminal() {
    let dir = base("durations");
    std::fs::write(
        dir.join("data/calls.yaml"),
        "views:\n  t.Calls:\n    rows:\n      \
         - {id: c-1, title: back, talk: -500, hold: -73}\n      \
         - {id: c-2, title: part, talk: 1999.9, hold: 73.9}\n      \
         - {id: c-3, title: hundred, talk: 360000000, hold: 360061}\n",
    )
    .expect("the fixture is written");
    let mut app = App::from_text(DURATIONS, &dir, Options::new(dir.join("state")))
        .unwrap_or_else(|error| panic!("{error}"));
    app.open_page("calls", &[]);
    let screen = app.render_text(140, 30);
    let row = |title: &str| {
        screen
            .lines()
            .find(|line| line.contains(title))
            .unwrap_or_else(|| panic!("no row {title}:\n{screen}"))
            .to_owned()
    };
    let back = row("back");
    assert!(back.contains("-0:01:13"), "-73 s:\n{screen}");
    assert!(
        back.contains(" 0:00:00"),
        "-500 ms is under a second, unsigned:\n{screen}"
    );
    assert!(!back.contains("-0:00:00"), "{screen}");
    let part = row("part");
    assert!(
        part.contains("0:00:01"),
        "1999.9 ms drops its fraction:\n{screen}"
    );
    assert!(
        part.contains("0:01:13"),
        "73.9 s drops its fraction:\n{screen}"
    );
    let hundred = row("hundred");
    assert!(hundred.contains("100:00:00"), "{screen}");
    assert!(hundred.contains("100:01:01"), "{screen}");
}
