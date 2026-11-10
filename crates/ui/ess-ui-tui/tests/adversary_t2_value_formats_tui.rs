//! Adversary cases for story:a-declared-field-format-renders-or-is-refused, terminal side.

use std::path::{Path, PathBuf};

use ess_ui_tui::{App, Options};

const DOC: &str = r"
format: ess-ui/1
app: t
model: t.system
placement_profile: fat
fixtures: {dir: data, views: {t.Stats: stats.yaml}}
shells:
  app: {regions: {main: {kind: page_outlet}}}
navigation:
  home: stats
  sections: [{name: all, pages: [stats]}]
pages:
  stats:
    kind: list_page
    title: Stats
    sections:
      - name: rows
        component: collection
        reads: {view: t.Stats}
        columns:
          - {field: title}
          - {field: at, as: timestamp}
          - {field: at, name: clock, as: time}
          - {field: ratio, as: percent}
";

/// 1767323045000 ms is 2026-01-02T03:04:05Z; 0.123 is 12.3% in React
/// (`Intl.NumberFormat` percent, `maximumFractionDigits: 1`).
const STATS: &str = r"
views:
  t.Stats:
    rows:
      - {id: s-1, title: first, at: 1767323045000, ratio: 0.123}
";

fn screen(test: &str) -> String {
    let base: PathBuf = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("ess-ui-tui-adversary-t2-value-formats")
        .join(test);
    if base.exists() {
        std::fs::remove_dir_all(&base).expect("a stale dir is removed");
    }
    std::fs::create_dir_all(base.join("data")).expect("the dir is created");
    std::fs::write(base.join("data/stats.yaml"), STATS).expect("the fixture is written");
    let mut app = App::from_text(DOC, &base, Options::new(base.join("state")))
        .unwrap_or_else(|error| panic!("{error}"));
    app.open_page("stats", &[]);
    app.render_text(200, 48)
}

fn row<'a>(screen: &'a str, title: &str) -> &'a str {
    screen
        .lines()
        .find(|line| line.contains(title))
        .unwrap_or_else(|| panic!("no row {title}:\n{screen}"))
}

/// React reads an epoch-millisecond number as an instant (`formatValue`, and its own test pins
/// it for `as: timestamp`); the terminal shows the raw number.
#[test]
fn an_epoch_ms_timestamp_shows_date_and_time() {
    let screen = screen("epoch-timestamp");
    let first = row(&screen, "first");
    assert!(
        first.contains("2026-01-02 03:04"),
        "an epoch-ms timestamp in ISO order:\n{screen}"
    );
}

#[test]
fn an_epoch_ms_time_shows_the_time_of_day() {
    let screen = screen("epoch-time");
    let first = row(&screen, "first");
    assert!(
        !first.contains("1767323045000"),
        "an epoch-ms time is no raw number:\n{screen}"
    );
}

/// React shows 0.123 as `12.3%`; the terminal must show the same percent.
#[test]
fn a_percent_shows_the_digits_react_shows() {
    let screen = screen("percent-digits");
    let first = row(&screen, "first");
    assert!(
        first.contains("12.3%"),
        "0.123 is 12.3% as in React:\n{screen}"
    );
}
