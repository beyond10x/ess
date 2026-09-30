//! Adversary pass 2: bulk actions at zero selected rows.

use std::path::{Path, PathBuf};

use ess_ui_tui::{App, Options};

fn example_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../examples/partner-portal")
}

fn open(test: &str) -> App {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("ess-ui-tui-adv2")
        .join(test);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("a stale state dir is removed");
    }
    App::from_path(&example_dir().join("ui.yaml"), Options::new(dir))
        .unwrap_or_else(|error| panic!("{error}"))
}

/// The key the bulk bar offers for "Add tag", read from the screen.
fn bulk_key(text: &str) -> char {
    let line = text
        .lines()
        .find(|line| line.contains("Add tag"))
        .unwrap_or_else(|| panic!("no bulk bar:\n{text}"));
    let at = line.find(" Add tag").expect("the label");
    line[..at].chars().last().expect("a key before the label")
}

/// With nothing selected — never, or after selecting and deselecting a row — the bulk key runs
/// no command.
#[test]
fn adv2_a_bulk_key_at_zero_selected_rows_runs_nothing() {
    let mut app = open("bulk-zero");
    app.open_page("partners.list", &[]);
    app.focus_section("list");
    let key = bulk_key(&app.render_text(160, 48));
    app.keys(&key.to_string());
    assert!(
        !app.render_text(160, 48).contains("TagPartners"),
        "the bulk action ran with no row selected"
    );
    app.keys("<space><space>");
    app.keys(&key.to_string());
    let text = app.render_text(160, 48);
    assert!(
        !text.contains("TagPartners"),
        "the bulk action ran after the only selected row was deselected:\n{text}"
    );
    app.keys("<space>");
    app.keys(&key.to_string());
    assert!(
        app.render_text(160, 48).contains("TagPartners"),
        "the bulk action runs once a row is selected"
    );
}
