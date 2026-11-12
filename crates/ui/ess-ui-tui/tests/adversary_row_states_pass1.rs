//! Adversary pass 1 on story:each-row-and-each-waiting-read-draws-what-the-document-declares:
//! the terminal half of the cases where React and the terminal must draw the same thing for the
//! same document (the React half is `ess-ui-react/tests/adversary_row_states_pass1.rs`), and a
//! section that follows a list holding a row, which must still read.

use std::path::{Path, PathBuf};

use ess_ui_tui::{App, Options};

const DOCUMENT: &str = r#"format: ess-ui/1
app: desk
model: desk.system
placement_profile: fat
fixtures:
  views:
    desk.Orders: orders.yaml
    desk.Lines: lines.yaml
    desk.Line: line.yaml
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
          - {field: note, visible: row.has_note or params.reveal}
          - {field: secret, visible: "params.mode == 'row.x'"}
          - {field: code, visible: params.show-row.on}
      - name: lines
        component: collection
        depends_on: table
        reads: {view: desk.Lines, params: {order: section.table.selection.id}}
        columns: [id]
      - name: item
        component: record
        depends_on: lines
        reads: {view: desk.Line, params: {id: section.lines.selection.id}}
        fields: [id, sku]
"#;

fn scratch(test: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("ess-ui-tui-adversary-row-states-pass1")
        .join(test);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("a stale dir is removed");
    }
    std::fs::create_dir_all(&dir).expect("the dir is made");
    for (file, text) in [
        (
            "orders.yaml",
            "views:\n  desk.Orders:\n    rows:\n      - {id: o1, note: fragile, has_note: true, secret: sx, code: c1}\n      - {id: o2, note: stale, has_note: false, secret: sy, code: c2}\n",
        ),
        (
            "lines.yaml",
            "views:\n  desk.Lines:\n    rows:\n      - {id: l1}\n      - {id: l2}\n",
        ),
        (
            "line.yaml",
            "views:\n  desk.Line:\n    rows:\n      - {id: l1, sku: SKU-ONE}\n",
        ),
    ] {
        std::fs::write(dir.join(file), text).expect("the fixture is written");
    }
    dir
}

fn screen(test: &str) -> String {
    let dir = scratch(test);
    App::from_text(DOCUMENT, &dir, Options::new(dir.join("state")))
        .unwrap_or_else(|error| panic!("{error}"))
        .render_text(110, 200)
}

/// The lines of `screen` from the one holding the section's name up to the next section border.
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

/// `row.has_note or params.reveal` mixes a row path with a param path. React judges it per row
/// and draws `fragile` in the first row (`a_condition_mixing_row_and_params_is_judged_per_row`
/// in ess-ui-react). The terminal's evaluator has no `or`: it walks `row` by the segment
/// `has_note or params`, gets null, and empties the cell of every row.
#[test]
fn a_condition_mixing_row_and_params_is_judged_per_row() {
    let screen = screen("mixed");
    let table = section(&screen, "table").join("\n");
    assert!(
        table.contains("fragile"),
        "the first row's note is drawn, as React draws it:\n{screen}"
    );
    assert!(
        !table.contains("stale"),
        "the second row's note is left out:\n{screen}"
    );
}

/// The terminal half of the two React cases: a condition whose `row.` is inside quotes or is the
/// tail of a hyphenated name reads no row, so the column is hidden whole, header and all.
#[test]
fn a_condition_reading_no_row_path_hides_the_whole_column() {
    let screen = screen("whole");
    let table = section(&screen, "table").join("\n").to_lowercase();
    assert!(table.contains("o1"), "the table is drawn:\n{screen}");
    assert!(
        !table.contains("secret") && !table.contains("sx"),
        "a quoted `row.` reads no row:\n{screen}"
    );
    assert!(
        !table.contains("code") && !table.contains("c1"),
        "a name ending in `-row` reads no row:\n{screen}"
    );
}

/// `item` depends on `lines`, which holds rows and a cursor, the terminal's selection. It must
/// read. `lines` reads a `section.` param, so `section.lines.selection` resolves to null in the
/// terminal (app.rs `reads_self`), and the new `App::waiting` treats `item` as never started:
/// it draws `Nothing to show yet` forever and its read is never issued. At the base it read.
#[test]
fn a_section_following_a_list_that_holds_a_row_still_reads() {
    let screen = screen("chain");
    let lines = section(&screen, "lines").join("\n");
    assert!(
        lines.contains("l1"),
        "the list item follows has rows:\n{screen}"
    );
    let item = section(&screen, "item").join("\n");
    assert!(
        item.contains("SKU-ONE"),
        "a section following a list that holds a row reads:\n{screen}"
    );
}
