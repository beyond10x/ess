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

/// `as: duration` columns, one per unit (story:a-duration-renders-as-hours-minutes-seconds), on a
/// collection and on a record.
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
          - {field: raw}
      - name: last
        component: record
        reads: {view: t.Calls}
        fields:
          - {field: title, label: Record}
          - {field: talk, label: Talked, as: duration}
          - {field: hold, label: Held, as: duration, unit: s}
";

const CALLS: &str = r"
views:
  t.Calls:
    rows:
      - {id: c-1, title: short, talk: 73000, hold: 73, raw: 73000}
      - {id: c-2, title: long, talk: 93600000, hold: 93600, raw: 93600}
      - {id: c-3, title: rounded, talk: 59999, hold: 3599, raw: 1}
";

fn duration_screen(test: &str) -> String {
    let base = dir(test);
    std::fs::write(base.join("data/calls.yaml"), CALLS).expect("the fixture is written");
    let mut app = App::from_text(DURATIONS, &base, Options::new(base.join("state")))
        .unwrap_or_else(|error| panic!("{error}"));
    app.open_page("calls", &[]);
    app.render_text(140, 48)
}

#[test]
fn a_duration_column_shows_hours_minutes_and_seconds_from_ms_or_s() {
    let screen = duration_screen("duration-column");
    let short = row(&screen, "short");
    assert!(short.contains("0:01:13"), "73000 ms and 73 s:\n{screen}");
    assert_eq!(short.matches("0:01:13").count(), 2, "both units:\n{screen}");
    assert_eq!(
        short.matches("73000").count(),
        1,
        "only the plain `raw` column shows the number:\n{screen}"
    );
    let long = row(&screen, "long");
    assert_eq!(
        long.matches("26:00:00").count(),
        2,
        "26 hours stay hours:\n{screen}"
    );
    assert!(
        !long.contains("1:02:00:00") && !long.contains("1d"),
        "{screen}"
    );
    let rounded = row(&screen, "rounded");
    assert!(rounded.contains("0:00:59"), "whole seconds:\n{screen}");
    assert!(rounded.contains("0:59:59"), "{screen}");
    assert!(
        long.contains("93600"),
        "a field without `as: duration` shows its number:\n{screen}"
    );
}

#[test]
fn a_duration_record_field_shows_hours_minutes_and_seconds() {
    let screen = duration_screen("duration-record");
    assert!(row(&screen, "Talked").contains("0:01:13"), "{screen}");
    assert!(row(&screen, "Held").contains("0:01:13"), "{screen}");
}

#[test]
fn a_placeholder_read_takes_its_fixture_from_fixtures_dir() {
    let screen = screen("placeholder");
    assert!(screen.contains("from-the-fixture-dir"), "{screen}");
}

/// `as: tags` on a collection and on a record (story:a-null-tag-draws-no-chip): a `null` entry
/// carries no tag and draws no chip; a list of only `null` draws nothing.
const TAGS: &str = r"
format: ess-ui/1
app: t
model: t.system
placement_profile: fat
fixtures: {dir: data, views: {t.Calls: tags.yaml, t.Last: last.yaml}}
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
          - {field: labels, as: tags}
          - {field: after}
      - name: last
        component: record
        reads: {view: t.Last}
        fields:
          - {field: labels, label: Labelled, as: tags}
";

const TAG_ROWS: &str = r"
views:
  t.Calls:
    rows:
      - {id: c-1, title: mixed, labels: [a, null, b], after: end-1}
      - {id: c-2, title: only, labels: [null], after: end-2}
      - {id: c-3, title: scalars, labels: [3, true, x], after: end-3}
";

const LAST_ROWS: &str = "views:\n  t.Last:\n    rows:\n      - {id: l-1, labels: [p, null, q]}\n";

fn tags_screen(test: &str) -> String {
    let base = dir(test);
    std::fs::write(base.join("data/tags.yaml"), TAG_ROWS).expect("the fixture is written");
    std::fs::write(base.join("data/last.yaml"), LAST_ROWS).expect("the fixture is written");
    let mut app = App::from_text(TAGS, &base, Options::new(base.join("state")))
        .unwrap_or_else(|error| panic!("{error}"));
    app.open_page("calls", &[]);
    app.render_text(140, 48)
}

#[test]
fn a_null_tag_draws_no_chip_in_a_column_or_a_record() {
    let screen = tags_screen("null-tags");
    let mixed = row(&screen, "mixed");
    assert!(mixed.contains("#a #b"), "two chips, a and b:\n{screen}");
    assert_eq!(mixed.matches('#').count(), 2, "no chip for null:\n{screen}");
    let only = row(&screen, "only");
    assert!(
        !only.contains('#'),
        "a list of only null draws no chip:\n{screen}"
    );
    let after = only.find("end-2").expect("the next column is drawn");
    assert!(
        only[only.find("only").expect("the title") + "only".len()..after]
            .trim()
            .is_empty(),
        "and nothing else:\n{screen}"
    );
    assert!(
        row(&screen, "scalars").contains("#3 #true #x"),
        "other values still draw their text:\n{screen}"
    );
    let record = row(&screen, "Labelled");
    assert!(record.contains("#p #q"), "{screen}");
    assert_eq!(
        record.matches('#').count(),
        2,
        "no chip for null:\n{screen}"
    );
}

/// An edit form loading a record whose tags hold `null`: editing the tags input starts from the
/// held tags without an empty slot for the null, as the React input shows them.
#[test]
fn a_held_null_tag_leaves_no_empty_slot_in_the_tags_input() {
    let base = dir("null-tags-input");
    std::fs::write(base.join("data/last.yaml"), LAST_ROWS).expect("the fixture is written");
    let doc = TAGS.replace(
        "      - name: last\n        component: record\n        reads: {view: t.Last}\n        fields:\n          - {field: labels, label: Labelled, as: tags}",
        "      - name: edit\n        component: form\n        does: t.Save\n        loads: {view: t.Last}\n        fields:\n          - {field: labels, label: Edited, as: tags}",
    );
    assert!(doc.contains("label: Edited"), "the edit applied");
    std::fs::write(base.join("data/tags.yaml"), TAG_ROWS).expect("the fixture is written");
    let mut app = App::from_text(&doc, &base, Options::new(base.join("state")))
        .unwrap_or_else(|error| panic!("{error}"));
    app.open_page("calls", &[]);
    app.focus_section("edit");
    app.keys("<enter>x");
    let screen = app.render_text(140, 48);
    let edited = row(&screen, "Edited");
    assert!(
        edited.contains("[p, qx"),
        "the held tags, then the typed text:\n{screen}"
    );
    assert!(!edited.contains(", ,"), "no empty slot for null:\n{screen}");
}
