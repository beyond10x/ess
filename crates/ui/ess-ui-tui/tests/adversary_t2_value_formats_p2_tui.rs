//! Adversary cases, pass 2, for story:a-declared-field-format-renders-or-is-refused, terminal
//! side: the hand-written epoch/ISO-to-UTC conversion and the percent digits, against what the
//! React target shows for the same value (`Date` and `Intl` in UTC, measured with node 22).

use std::path::{Path, PathBuf};

use ess_ui_tui::{App, Options};

fn doc(columns: &str) -> String {
    format!(
        "
format: ess-ui/1
app: t
model: t.system
placement_profile: fat
fixtures: {{dir: data, views: {{t.Stats: stats.yaml}}}}
shells:
  app: {{regions: {{main: {{kind: page_outlet}}}}}}
navigation:
  home: stats
  sections: [{{name: all, pages: [stats]}}]
pages:
  stats:
    kind: list_page
    title: Stats
    sections:
      - name: rows
        component: collection
        reads: {{view: t.Stats}}
        columns:
          - {{field: title}}
{columns}"
    )
}

fn screen(test: &str, columns: &str, rows: &str) -> String {
    let base: PathBuf = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("ess-ui-tui-adversary-t2-value-formats-p2")
        .join(test);
    if base.exists() {
        std::fs::remove_dir_all(&base).expect("a stale dir is removed");
    }
    std::fs::create_dir_all(base.join("data")).expect("the dir is created");
    std::fs::write(
        base.join("data/stats.yaml"),
        format!("views:\n  t.Stats:\n    rows:\n{rows}"),
    )
    .expect("the fixture is written");
    let mut app = App::from_text(&doc(columns), &base, Options::new(base.join("state")))
        .unwrap_or_else(|error| panic!("{error}"));
    app.open_page("stats", &[]);
    app.render_text(220, 48)
}

fn row<'a>(screen: &'a str, title: &str) -> &'a str {
    screen
        .lines()
        .find(|line| line.contains(title))
        .unwrap_or_else(|| panic!("no row {title}:\n{screen}"))
}

/// React: `Intl.NumberFormat(undefined, {style: "percent", maximumFractionDigits: 1})` rounds
/// half away from zero on the decimal value: 0.1225 is `12.3%`, 0.0025 is `0.3%`. The terminal's
/// `format!("{:.1}")` rounds the binary value half to even: `12.2%` and `0.2%`.
#[test]
fn a_percent_on_a_tie_rounds_as_react_does() {
    let screen = screen(
        "percent-tie",
        "          - {field: ratio, as: percent}\n",
        "      - {id: a, title: rowa, ratio: 0.1225}\n      - {id: b, title: rowb, ratio: 0.0025}\n",
    );
    assert!(
        row(&screen, "rowa").contains("12.3%"),
        "0.1225 shows 12.3% as React does:\n{screen}"
    );
    assert!(
        row(&screen, "rowb").contains("0.3%"),
        "0.0025 shows 0.3% as React does:\n{screen}"
    );
}

/// React reads a date-only text as UTC midnight: `as: timestamp` of `2026-01-02` shows
/// `01/02/2026, 00:00`. The terminal shows the bare date, no time.
#[test]
fn a_date_only_timestamp_shows_midnight() {
    let screen = screen(
        "date-only",
        "          - {field: at, as: timestamp}\n",
        "      - {id: a, title: rowa, at: '2026-01-02'}\n",
    );
    assert!(
        row(&screen, "rowa").contains("2026-01-02 00:00"),
        "a date-only timestamp shows its date and 00:00 as React does:\n{screen}"
    );
}

/// An HTTP date (`Fri, 02 Jan 2026 03:04:05 GMT`) is an instant to JavaScript's `Date`, and React
/// shows `03:04`. The terminal's fallback splits it at the `T` of `GMT` and shows nothing.
#[test]
#[ignore = "backlog: the terminal reads an HTTP-style date as an instant"]
fn an_http_date_time_shows_the_time_of_day() {
    let screen = screen(
        "http-date",
        "          - {field: at, as: time}\n",
        "      - {id: a, title: rowa, at: 'Fri, 02 Jan 2026 03:04:05 GMT'}\n",
    );
    assert!(
        row(&screen, "rowa").contains("03:04"),
        "an HTTP date shows the time of day as React does:\n{screen}"
    );
}

/// Pins the conversion against `new Date(v).toISOString()`: a negative epoch, the 1900 and 2024
/// February, an offset of +05:45 with fractional seconds, `-00:00`, a compact `+0545`, no offset
/// (UTC in the terminal), the last instant JavaScript holds, and the day of a date column.
#[test]
fn instants_convert_as_javascript_does() {
    let screen = screen(
        "instants",
        "          - {field: at, as: timestamp}\n          - {field: at, name: day, as: date}\n",
        "      - {id: a, title: neg, at: -1}\n\
         \x20     - {id: b, title: oldcentury, at: -2208988800001}\n\
         \x20     - {id: c, title: noleap, at: '1900-03-01T00:30:00+01:00'}\n\
         \x20     - {id: d, title: leapday, at: '2024-02-29T23:59:59.999Z'}\n\
         \x20     - {id: e, title: kathmandu, at: '2026-01-02T03:04:05.123456+05:45'}\n\
         \x20     - {id: f, title: minuszero, at: '2026-01-02T03:04:05-00:00'}\n\
         \x20     - {id: g, title: compact, at: '2026-01-02T03:04:05+0545'}\n\
         \x20     - {id: h, title: naive, at: '2026-01-02T03:04:05'}\n\
         \x20     - {id: i, title: farfuture, at: 8640000000000000}\n\
         \x20     - {id: j, title: dayedge, at: 1767310200000}\n",
    );
    for (title, expected) in [
        ("neg", "1969-12-31 23:59"),
        ("oldcentury", "1899-12-31 23:59"),
        ("noleap", "1900-02-28 23:30"),
        ("leapday", "2024-02-29 23:59"),
        ("kathmandu", "2026-01-01 21:19"),
        ("minuszero", "2026-01-02 03:04"),
        ("compact", "2026-01-01 21:19"),
        ("naive", "2026-01-02 03:04"),
        ("farfuture", "275760-09-13 00:00"),
        ("dayedge", "2026-01-01 23:30"),
    ] {
        assert!(
            row(&screen, title).contains(expected),
            "{title} shows {expected}:\n{screen}"
        );
    }
}
