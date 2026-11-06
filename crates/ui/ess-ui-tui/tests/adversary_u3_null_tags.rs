//! Adversary, wave 0.90.0 unit 3 (story:a-null-tag-draws-no-chip), terminal side: the cases the
//! unit's own test does not name, each with the chips the React runtime draws for the same value.

use std::path::{Path, PathBuf};

use ess_ui_tui::{App, Options};

const DOC: &str = r"
format: ess-ui/1
app: t
model: t.system
placement_profile: fat
fixtures: {dir: data, views: {t.Calls: tags.yaml}}
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
";

/// React draws: `["null"]` one chip "null"; `[~, null, Null]` nothing; `[false, 0]` two chips;
/// `[a, ~, a, ~, a]` three chips; `[]` nothing.
const ROWS: &str = r#"
views:
  t.Calls:
    rows:
      - {id: c-1, title: quoted, labels: ["null"], after: end-1}
      - {id: c-2, title: spelled, labels: [~, null, Null, NULL], after: end-2}
      - {id: c-3, title: falsy, labels: [false, 0], after: end-3}
      - {id: c-4, title: repeated, labels: [a, ~, a, ~, a], after: end-4}
      - {id: c-5, title: empty, labels: [], after: end-5}
"#;

fn screen(test: &str) -> String {
    let base: PathBuf = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("ess-ui-tui-adversary-u3-null-tags")
        .join(test);
    if base.exists() {
        std::fs::remove_dir_all(&base).expect("a stale dir is removed");
    }
    std::fs::create_dir_all(base.join("data")).expect("the dir is created");
    std::fs::write(base.join("data/tags.yaml"), ROWS).expect("the fixture is written");
    let mut app = App::from_text(DOC, &base, Options::new(base.join("state")))
        .unwrap_or_else(|error| panic!("{error}"));
    app.open_page("calls", &[]);
    app.render_text(140, 48)
}

fn row<'a>(screen: &'a str, title: &str) -> &'a str {
    screen
        .lines()
        .find(|line| line.contains(title))
        .unwrap_or_else(|| panic!("no row {title}:\n{screen}"))
}

/// The cell between the title and the next column.
fn cell<'a>(screen: &'a str, title: &str, after: &str) -> &'a str {
    let line = row(screen, title);
    let start = line.find(title).expect("the title") + title.len();
    let end = line.find(after).expect("the next column");
    line[start..end].trim()
}

#[test]
fn the_terminal_draws_the_chips_react_draws() {
    let screen = screen("parity");
    assert_eq!(cell(&screen, "quoted", "end-1"), "#null", "the text null is a tag:\n{screen}");
    assert_eq!(cell(&screen, "spelled", "end-2"), "", "every YAML null draws nothing:\n{screen}");
    assert_eq!(cell(&screen, "falsy", "end-3"), "#false #0", "{screen}");
    assert_eq!(cell(&screen, "repeated", "end-4"), "#a #a #a", "{screen}");
    assert_eq!(cell(&screen, "empty", "end-5"), "", "{screen}");
}
