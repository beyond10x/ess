//! A page header's help is a node, `pages/<page>/header/help`: a test acts on it to open it and
//! expects its text there, in the terminal and in the Playwright spec. A downstream app that
//! draws the help closed until it is opened (a disclosure beside the title) can then be driven by
//! the same test, which before could only expect the text on the whole page.

use std::path::{Path, PathBuf};

use ess_ui_test::{Outcome, Status};

const DOCUMENT: &str = r"format: ess-ui/1
app: probe
model: probe.system
placement_profile: fat
fixtures: {views: {things.Page: things.yaml}}
shells:
  app:
    regions:
      nav:     {kind: navigation}
      main:    {kind: page_outlet}
navigation:
  home: things.list
  sections: [{name: all, pages: [things.list, things.bare]}]
pages:
  things.list:
    kind: list_page
    title: Things
    header:
      help: {text: Things are kept for a year, link: 'https://example.com/help/things'}
    sections:
      - name: list
        component: collection
        reads: {view: things.Page}
        columns: [name]
  things.bare:
    kind: list_page
    title: Bare
    sections:
      - name: list
        component: collection
        reads: {view: things.Page}
        columns: [name]
";

fn document(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("ess-ui-test-header-help")
        .join(name);
    std::fs::create_dir_all(&dir).expect("the dir is made");
    std::fs::write(
        dir.join("things.yaml"),
        "views:\n  things.Page:\n    rows:\n      - {id: th-1, name: Anvil}\n",
    )
    .expect("the fixture is written");
    std::fs::write(dir.join("ui.yaml"), DOCUMENT).expect("the document is written");
    dir.join("ui.yaml")
}

fn file(name: &str, steps: &[&str]) -> ess_ui_test::TestFile {
    let mut text = format!(
        "format: ess-ui-test/1\ndocument: {}\ntests:\n- name: help\n  steps:\n",
        document(name).display()
    );
    for step in steps {
        text.push_str("  - ");
        text.push_str(step);
        text.push('\n');
    }
    ess_ui_test::parse_str(&text, Path::new("header_help.yaml"))
        .unwrap_or_else(|error| panic!("{error}\n{text}"))
}

fn run(name: &str, steps: &[&str]) -> Outcome {
    let report = ess_ui_test::execute(&document(name), &[file(name, steps)])
        .unwrap_or_else(|error| panic!("{error}"));
    report.tests.into_iter().next().expect("one outcome")
}

fn spec(name: &str, steps: &[&str]) -> String {
    ess_ui_test::playwright(
        &[file(name, steps)],
        &ess_ui::load_path(&document(name)).expect("the document loads"),
    )
}

const HELP: &str = "pages/things.list/header/help";

#[test]
fn a_page_headers_help_is_opened_and_read_at_its_own_path() {
    let steps = [
        "open: things.list",
        &format!("act: {HELP}"),
        &format!("expect: {{at: {HELP}, text: Things are kept for a year}}"),
        &format!("expect: {{at: {HELP}, not_text: Anvil}}"),
    ];
    let outcome = run("open", &steps);
    assert_eq!(outcome.status, Status::Passed, "{outcome:?}");
    let spec = spec("open", &steps);
    let at = format!("[data-ui-path=\"{HELP}\"]");
    for wanted in [
        format!("await page.locator('{at}').click();"),
        format!(
            "await expect(page.locator('{at}')).toContainText(\"Things are kept for a year\");"
        ),
    ] {
        assert!(spec.contains(&wanted), "{wanted}\n\n{spec}");
    }
    assert!(!spec.contains("test.fixme"), "{spec}");
}

#[test]
fn a_header_without_help_has_no_help_node() {
    let outcome = run(
        "none",
        &["open: things.bare", "act: pages/things.bare/header/help"],
    );
    assert_eq!(outcome.step, Some(2), "{outcome:?}");
    assert!(
        outcome
            .message
            .unwrap_or_default()
            .contains("no node at pages/things.bare/header/help"),
        "a page whose header declares no help has no help node"
    );
}
