//! `App::regions`: where the last frame drew each node, by canonical path, so a headless reader
//! can tell rows apart by key and read a node's own cells.

use std::path::{Path, PathBuf};

use ess_ui_tui::{App, Options, Region};

fn example_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../examples/partner-portal")
}

fn open(test: &str) -> App {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("ess-ui-tui-regions")
        .join(test);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("a stale state dir is removed");
    }
    App::from_path(&example_dir().join("ui.yaml"), Options::new(dir))
        .unwrap_or_else(|error| panic!("{error}"))
}

/// The text inside `region` of a `width`-wide screen rendered as text.
fn text_in(screen: &str, region: &Region) -> String {
    screen
        .lines()
        .skip(usize::from(region.area.y))
        .take(usize::from(region.area.height))
        .map(|line| {
            line.chars()
                .skip(usize::from(region.area.x))
                .take(usize::from(region.area.width))
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn rows_are_recorded_by_key_and_cells_by_column() {
    let mut app = open("rows");
    app.open_page("partners.list", &[]);
    let screen = app.render_text(160, 120);
    let regions = app.regions();
    let list = "pages/partners.list/sections/list";
    let rows: Vec<&Region> = regions
        .iter()
        .filter(|region| {
            region
                .row
                .as_ref()
                .is_some_and(|key| region.path == format!("{list}/rows/{key}"))
        })
        .collect();
    assert!(!rows.is_empty(), "{regions:#?}");
    assert!(
        rows.windows(2).all(|pair| pair[0].area.y < pair[1].area.y),
        "rows are recorded top to bottom: {rows:#?}"
    );
    let row = regions
        .iter()
        .find(|region| region.path == format!("{list}/rows/pt-003"))
        .unwrap_or_else(|| panic!("row pt-003 is recorded: {regions:#?}"));
    assert!(text_in(&screen, row).contains("Cedar Partners"), "{screen}");
    // partners.list is a cards collection with an `item`: the generated React project renders
    // the item, not the column cells, so no cell is recorded.
    assert!(
        regions.iter().all(|region| !region
            .path
            .starts_with(&format!("{list}/rows/pt-003/columns/"))),
        "{regions:#?}"
    );
    // A table's cells are recorded by column.
    app.open_page("tickets.list", &[]);
    let tickets_screen = app.render_text(160, 120);
    let tickets_regions = app.regions();
    let tickets = "pages/tickets.list/sections/list";
    let priority = tickets_regions
        .iter()
        .find(|region| region.path == format!("{tickets}/rows/tk-01/columns/priority"))
        .unwrap_or_else(|| panic!("the priority cell of tk-01 is recorded: {tickets_regions:#?}"));
    assert_eq!(priority.row.as_deref(), Some("tk-01"));
    let cell = text_in(&tickets_screen, priority);
    assert!(
        !cell.contains("SSO login fails"),
        "the cell is its own: {cell:?}"
    );
    assert!(
        cell.contains("high"),
        "the cell shows the priority: {cell:?}"
    );
    let subject = tickets_regions
        .iter()
        .find(|region| region.path == format!("{tickets}/rows/tk-01/columns/subject"))
        .unwrap_or_else(|| panic!("the subject cell of tk-01 is recorded: {tickets_regions:#?}"));
    assert_eq!(text_in(&tickets_screen, subject).trim(), "SSO login fails");
    assert_eq!(subject.text.as_deref(), Some("SSO login fails"));
    // A cards collection draws no column header, so none is recorded: a reader cannot scope text
    // to it.
    assert!(
        regions
            .iter()
            .all(|region| region.path != format!("{list}/columns/tier")),
        "{regions:#?}"
    );
    assert!(
        regions
            .iter()
            .any(|region| region.path == format!("{list}/children/hint")),
        "the section's child is recorded: {regions:#?}"
    );
    assert!(
        regions
            .iter()
            .any(|region| region.path == "pages/partners.list/header/actions/create"),
        "the header action is recorded: {regions:#?}"
    );
}

#[test]
fn a_new_frame_replaces_the_regions_of_the_last() {
    let mut app = open("replace");
    app.open_page("partners.list", &[]);
    app.render_text(160, 120);
    assert!(app
        .regions()
        .iter()
        .any(|region| region.path.starts_with("pages/partners.list/")));
    app.open_page("overview", &[]);
    app.render_text(160, 120);
    assert!(
        app.regions()
            .iter()
            .all(|region| !region.path.starts_with("pages/partners.list/")),
        "{:#?}",
        app.regions()
    );
}
