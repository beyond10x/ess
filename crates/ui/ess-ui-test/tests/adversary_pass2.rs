//! Adversary pass 2: the same test file must give the same verdict in the terminal and in the
//! generated Playwright spec. Each case here runs a file headless and compares what the terminal
//! resolved with what the generated React project renders at the same `data-ui-path`.

use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::{Path, PathBuf};

use ess_ui_test::{Outcome, Status};

fn example_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../examples/partner-portal")
        .canonicalize()
        .expect("the example exists")
}

fn document() -> PathBuf {
    example_dir().join("ui.yaml")
}

fn file_for(document: &Path, tests: &str) -> ess_ui_test::TestFile {
    let text = format!(
        "format: ess-ui-test/1\ndocument: {}\ntests:\n{tests}",
        document.display()
    );
    ess_ui_test::parse_str(&text, Path::new("adversary2.yaml"))
        .unwrap_or_else(|error| panic!("{error}\n{text}"))
}

fn run_on(document: &Path, tests: &str) -> Outcome {
    let report = ess_ui_test::execute(document, &[file_for(document, tests)])
        .unwrap_or_else(|error| panic!("{error}"));
    report.tests.into_iter().next().expect("one outcome")
}

fn run(tests: &str) -> Outcome {
    run_on(&document(), tests)
}

/// One test of `steps`, each a one-line YAML step, with optional `fixtures` text (already
/// indented under the test).
fn test(fixtures: &str, steps: &[&str]) -> String {
    let mut text = String::from("- name: adversary2\n");
    text.push_str(fixtures);
    text.push_str("  steps:\n");
    for step in steps {
        text.push_str("  - ");
        text.push_str(step);
        text.push('\n');
    }
    text
}

fn scratch(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("ess-ui-test-adv2")
        .join(name);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("the old scratch dir is removed");
    }
    std::fs::create_dir_all(&dir).expect("the scratch dir is made");
    dir
}

fn react(document: &Path) -> ess_ui_react::GeneratedFiles {
    let loaded = ess_ui::load_path(document).expect("the document loads");
    ess_ui_react::render(&loaded, document.parent().expect("a directory"))
        .unwrap_or_else(|error| panic!("{error}"))
}

fn generated<'a>(files: &'a ess_ui_react::GeneratedFiles, path: &str) -> &'a str {
    files
        .files
        .get(path)
        .unwrap_or_else(|| panic!("`{path}` is generated; have {:?}", files.files.keys()))
}

/// The page file that renders `path` as a `data-ui-path`.
fn page_rendering<'a>(files: &'a ess_ui_react::GeneratedFiles, path: &str) -> &'a str {
    files
        .files
        .iter()
        .find(|(name, text)| name.starts_with("src/pages/") && text.contains(path))
        .map_or_else(
            || panic!("no generated page mentions {path}"),
            |(_, text)| text.as_str(),
        )
}

/// partners.list is a cards collection with an `item` (the partner card). The terminal draws
/// the card line from the columns and records a region per cell, so `expect` at
/// `rows/pt-003/columns/tier` passes there. The generated collection renders a card's column
/// cells only when the collection has no `item` (`props.item ? null : <dl>…`), so the same
/// path names no element in the browser and the Playwright step waits for an element that never
/// appears. The same file: green in the terminal, red in the browser.
#[test]
fn a_cell_the_browser_does_not_render_does_not_pass_in_the_terminal() {
    let at = "pages/partners.list/sections/list/rows/pt-003/columns/tier";
    let outcome = run(&test(
        "",
        &[
            "open: partners.list",
            &format!("expect: {{at: {at}, text: silver}}"),
        ],
    ));
    let files = react(&document());
    let page = page_rendering(&files, "pages/partners.list/sections/list");
    let runtime = generated(&files, "src/runtime/composites/collection.tsx");
    let item_on_list = page.contains("item={");
    let cells_dropped_with_item = runtime.contains("{props.item ? null : (");
    let browser_has_cell = !(item_on_list && cells_dropped_with_item);
    let spec = ess_ui_test::playwright(
        &[file_for(
            &document(),
            &test(
                "",
                &[
                    "open: partners.list",
                    &format!("expect: {{at: {at}, text: silver}}"),
                ],
            ),
        )],
        &ess_ui::load_path(&document()).expect("the document loads"),
    );
    let spec_waits_for_it = spec.contains(&format!(
        "await expect(page.locator('[data-ui-path=\"{at}\"]')).toContainText(\"silver\");"
    ));
    assert!(
        outcome.status == Status::Failed || browser_has_cell || !spec_waits_for_it,
        "the terminal passed `expect text` at {at} ({outcome:?}), but the generated collection \
         renders no column cell for a cards collection with an `item`, and the Playwright spec \
         waits for [data-ui-path=\"{at}\"]: the same file passes in the terminal and fails in \
         the browser"
    );
}

/// A collection with selectable columns (`columns: {binds, all}`). The canonical path of a
/// column is `<collection>/columns/all/<name>`; the terminal records the header there. The
/// React header added by the coordinator renders `<collection>/columns/<name>` (no `all`), so
/// `expect` at the canonical header path passes in the terminal and finds no element in the
/// browser, while the non-canonical path the browser renders is refused by the terminal as
/// naming no node.
#[test]
fn a_selectable_column_header_has_the_same_path_in_both_renderers() {
    let dir = scratch("selectable");
    std::fs::write(
        dir.join("things.yaml"),
        "views:\n  things.Page:\n    rows:\n      - {id: th-1, name: Anvil, tier: gold}\n      \
         - {id: th-2, name: Bolt, tier: silver}\n",
    )
    .expect("the fixture is written");
    std::fs::write(
        dir.join("ui.yaml"),
        r"format: ess-ui/1
app: probe
model: probe.system
placement_profile: fat
fixtures: {views: {things.Page: things.yaml}}
shells:
  app:
    regions:
      nav:     {kind: navigation}
      main:    {kind: page_outlet}
      overlay: {kind: overlay_outlet}
navigation:
  home: things
  sections: [{name: all, pages: [things]}]
pages:
  things:
    kind: list_page
    title: Things
    state:
      cols: {type: {list: string}, class: page_state, store: url}
    sections:
      - name: list
        component: collection
        reads: {view: things.Page}
        columns: {binds: state.cols, all: [{field: name, label: Title}, tier]}
",
    )
    .expect("the document is written");
    let doc_path = dir.join("ui.yaml");
    let loaded = ess_ui::load_path(&doc_path).unwrap_or_else(|error| panic!("{error}"));
    let canonical = "pages/things/sections/list/columns/all/name";
    assert!(
        loaded
            .nodes()
            .iter()
            .any(|located| located.path.to_string() == canonical),
        "the canonical path of the column is {canonical}"
    );
    let terminal = run_on(
        &doc_path,
        &test(
            "",
            &[
                "open: things",
                &format!("expect: {{at: {canonical}, text: Title}}"),
            ],
        ),
    );
    assert_eq!(
        terminal.status,
        Status::Passed,
        "the terminal records the header at its canonical path: {terminal:?}"
    );
    let files = react(&doc_path);
    let runtime = generated(&files, "src/runtime/composites/collection.tsx");
    let header = runtime
        .lines()
        .find(|line| line.contains("<th key={column.name}"))
        .unwrap_or_else(|| panic!("the collection renders a header cell per column:\n{runtime}"));
    let container = "pages/things/sections/list";
    let rendered = if header.contains("data-ui-path={columnPath(column.name)}")
        && runtime.contains("`${scopedContainer}/columns/${column}`")
    {
        format!("{container}/columns/name")
    } else if header.contains("column[\"data-ui-path\"]") {
        canonical.to_owned()
    } else {
        panic!("unrecognised header path in `{header}`; update this case");
    };
    assert_eq!(
        rendered, canonical,
        "the terminal records the header of a selectable column at {canonical}; the browser \
         renders it at {rendered}, so one test file cannot address it in both"
    );
}

/// A text cell longer than the terminal's column width (28 cells) is cut with `…`. The browser
/// shows the whole value, so `expect text` of the whole subject at the cell passes in the
/// browser and fails in the terminal.
#[test]
fn a_long_cell_value_is_read_whole_in_the_terminal_as_in_the_browser() {
    let subject = "Quarterly reconciliation export fails for multi-currency invoices";
    let fixtures = format!(
        "  fixtures:\n    views:\n      tickets.Page:\n        total: 1\n        rows:\n\
         \x20         - {{id: tk-90, subject: {subject}, partner: Alder Resale, priority: low, \
         updated_at: \"2026-09-30T10:00:00Z\", opened_at: \"2026-09-30T10:00:00Z\"}}\n"
    );
    let at = "pages/tickets.list/sections/list/rows/tk-90/columns/subject";
    let outcome = run(&test(
        &fixtures,
        &[
            "open: tickets.list",
            // Control: the row is there and shows the start of the subject.
            "expect: {at: pages/tickets.list/sections/list, rows: [tk-90]}",
            &format!("expect: {{at: {at}, text: \"{subject}\"}}"),
        ],
    ));
    assert_eq!(
        outcome.status,
        Status::Passed,
        "the browser shows the whole subject at {at}; the terminal cut it: {outcome:?}"
    );
}

/// `text` at a row action reads its label, as the browser's button does. The terminal offers
/// the action on the row (its hint) but records no region for it, so the step is refused.
#[test]
fn text_at_a_row_action_reads_its_label() {
    // Control: a header action's label is read from its own cells.
    let header = run(&test(
        "",
        &[
            "open: tickets.list",
            "expect: {at: pages/tickets.list/header/actions/create, text: New ticket}",
        ],
    ));
    assert_eq!(header.status, Status::Passed, "{header:?}");
    let outcome = run(&test(
        "",
        &[
            "open: tickets.list",
            "expect: {at: pages/tickets.list/sections/list/rows/tk-01/row_actions/open, text: Open}",
        ],
    ));
    assert_eq!(
        outcome.status,
        Status::Passed,
        "the browser's button at the row action shows `Open`; the terminal refused: {outcome:?}"
    );
}

/// The terminal refuses a move that plays more than `MAX_CYCLES_PER_ADVANCE` cycles of a
/// looping script; the Playwright spec of the same file runs the page clock for it, so the
/// browser passes a test the terminal fails. The spec must mark such a step, as it marks every
/// other step the browser cannot mean the same way.
#[test]
fn a_move_the_terminal_refuses_is_not_a_plain_step_in_the_browser() {
    // ticket_chat loops every 18s: 180018s is 10001 cycles of it.
    let tests = test("", &["open: overview", "advance: 180018s"]);
    let outcome = run(&tests);
    assert_eq!(outcome.status, Status::Failed, "{outcome:?}");
    let spec = ess_ui_test::playwright(
        &[file_for(&document(), &tests)],
        &ess_ui::load_path(&document()).expect("the document loads"),
    );
    assert!(
        spec.contains("test.fixme("),
        "the terminal fails `advance: 180018s` ({:?}); the spec runs the page clock for it:\n{spec}",
        outcome.message
    );
}

/// At exactly the limit the move is allowed: 10000 cycles of the 18s `ticket_chat` loop.
#[test]
fn a_move_of_exactly_the_limit_is_played() {
    let outcome = run(&test("", &["open: overview", "advance: 180000s"]));
    assert_eq!(outcome.status, Status::Passed, "{outcome:?}");
}

/// Moving the clock past what a `Duration` holds fails the terminal run (pass 1); generating the
/// Playwright spec for the same file must not panic.
#[test]
fn the_playwright_spec_of_a_move_past_the_end_of_time_is_not_a_panic() {
    let tests = test(
        "",
        &[
            "open: overview",
            "advance: 18446744073709551615s",
            "advance: 18446744073709551615s",
        ],
    );
    let file = file_for(&document(), &tests);
    let document = ess_ui::load_path(&document()).expect("the document loads");
    let spec = catch_unwind(AssertUnwindSafe(|| {
        ess_ui_test::playwright(&[file], &document)
    }));
    assert!(
        spec.is_ok(),
        "generating the Playwright spec panicked on a move past the end of time"
    );
}
