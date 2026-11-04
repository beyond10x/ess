//! An empty state's `title` is shown as a line before its message
//! (story:a-refused-node-is-reported-at-its-own-path, beyond10x/ess#723).

use std::path::{Path, PathBuf};

use ess_ui_tui::{App, Options};

const DOCUMENT: &str = "format: ess-ui/1
app: desk
model: desk.system
placement_profile: fat
fixtures: {views: {desk.Files: files.yaml}}
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
      - name: list
        component: collection
        reads: {view: desk.Files}
        columns: [name]
        states: {empty: {title: No files, message: Upload a file to see it here.}}
";

fn scratch(test: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("ess-ui-tui-empty-title")
        .join(test);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("a stale dir is removed");
    }
    std::fs::create_dir_all(&dir).expect("the dir is made");
    std::fs::write(
        dir.join("files.yaml"),
        "views:\n  desk.Files:\n    rows: []\n",
    )
    .expect("the fixture is written");
    dir
}

#[test]
fn an_empty_state_shows_its_title() {
    let dir = scratch("titled");
    let app = App::from_text(DOCUMENT, &dir, Options::new(dir.join("state")))
        .unwrap_or_else(|error| panic!("{error}"));
    let screen = app.render_text(100, 30);
    let line_of = |text: &str| {
        screen
            .lines()
            .position(|line| line.contains(text))
            .unwrap_or_else(|| panic!("`{text}` is not shown:\n{screen}"))
    };
    let title = line_of("No files");
    let message = line_of("Upload a file to see it here.");
    assert!(
        title < message,
        "the title is not shown before the message:\n{screen}"
    );
}

#[test]
fn an_empty_title_is_no_title() {
    let dir = scratch("untitled");
    let screen = |document: &str| {
        App::from_text(document, &dir, Options::new(dir.join("state")))
            .unwrap_or_else(|error| panic!("{error}"))
            .render_text(100, 30)
    };
    let blank = screen(&DOCUMENT.replace("title: No files, ", "title: \"\", "));
    let none = screen(&DOCUMENT.replace("title: No files, ", ""));
    assert!(
        blank.contains("Upload a file to see it here."),
        "the message is shown:\n{blank}"
    );
    assert_eq!(
        blank, none,
        "story:a-refused-node-is-reported-at-its-own-path: `title: \"\"` draws what no title \
         draws, as React does"
    );
}
