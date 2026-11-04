//! Values in the form their field declares, in the terminal
//! (story:a-value-renders-in-the-form-its-field-declares): a byte count, on an `as: bytes` column
//! and on a metric with `format: bytes`, is scaled to B, KB, MB or GB in 1024 steps; a placeholder
//! read takes its fixture from the declared `fixtures.dir`, as the React target does.

use std::path::{Path, PathBuf};

use ess_ui_tui::{App, Options};

const DOC: &str = r"
format: ess-ui/1
app: t
model: t.system
placement_profile: fat
fixtures: {dir: data, views: {t.Files: files.yaml, t.Usage: files.yaml}}
shells:
  app: {regions: {main: {kind: page_outlet}}}
navigation:
  home: files
  sections: [{name: all, pages: [files]}]
pages:
  files:
    kind: list_page
    title: Files
    sections:
      - name: used
        component: metric
        reads: {view: t.Usage}
        match: used
        format: bytes
        label: Used
      - name: files
        component: collection
        reads: {view: t.Files}
        columns:
          - {field: title}
          - {field: size, as: bytes}
      - name: quarters
        component: collection
        reads: {placeholder: t.Quarters, fixture: quarters.yaml}
        columns: [quarter]
";

const FILES: &str = r"
views:
  t.Files:
    rows:
      - {id: f-1, title: tiny, size: 512}
      - {id: f-2, title: small, size: 1536}
      - {id: f-3, title: large, size: 5368709120}
      - {id: f-4, title: edge, size: 1048575}
  t.Usage:
    rows:
      - {id: u, used: 3221225472}
";

const QUARTERS: &str =
    "views:\n  t.Quarters:\n    rows:\n      - {id: q-1, quarter: from-the-fixture-dir}\n";

fn dir(test: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("ess-ui-tui-value-formats")
        .join(test);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("a stale dir is removed");
    }
    std::fs::create_dir_all(dir.join("data")).expect("the dir is created");
    std::fs::write(dir.join("data/files.yaml"), FILES).expect("the fixture is written");
    std::fs::write(dir.join("data/quarters.yaml"), QUARTERS).expect("the fixture is written");
    dir
}

fn screen(test: &str) -> String {
    let base = dir(test);
    let mut app = App::from_text(DOC, &base, Options::new(base.join("state")))
        .unwrap_or_else(|error| panic!("{error}"));
    app.open_page("files", &[]);
    app.render_text(140, 48)
}

/// The row of `screen` holding `title`.
fn row<'a>(screen: &'a str, title: &str) -> &'a str {
    screen
        .lines()
        .find(|line| line.contains(title))
        .unwrap_or_else(|| panic!("no row {title}:\n{screen}"))
}

#[test]
fn a_bytes_column_scales_in_1024_steps() {
    let screen = screen("column");
    for (title, shown) in [
        ("tiny", "512 B"),
        ("small", "1.5 KB"),
        ("large", "5 GB"),
        ("edge", "1 MB"),
    ] {
        assert!(
            row(&screen, title).contains(shown),
            "{title} shows {shown}:\n{screen}"
        );
    }
    assert!(!screen.contains("1536"), "{screen}");
}

#[test]
fn a_bytes_metric_scales_in_1024_steps() {
    let screen = screen("metric");
    assert!(row(&screen, "Used").contains("3 GB"), "{screen}");
    assert!(!screen.contains("3221225472"), "{screen}");
}

#[test]
fn a_placeholder_read_takes_its_fixture_from_fixtures_dir() {
    let screen = screen("placeholder");
    assert!(screen.contains("from-the-fixture-dir"), "{screen}");
}
