//! A live `insert_top` of a key the section already holds shows one row: a row path names one
//! row. The held row takes the event's values and moves to the top of the section.

use std::path::{Path, PathBuf};

use ess_ui_test::{Outcome, Status};

const FIXTURE: &str = "views:\n  shop.Lines:\n    rows:\n      \
                       - {line_id: 1, label: Tea}\n      \
                       - {line_id: 2, label: Milk}\n      \
                       - {line_id: 3, label: Rice}\n";

const SCRIPT: &str = "channel: orders\nloop: true\nevents:\n  \
                      - {at: 1s, event: shop.LineAdded, payload: {line_id: 4, label: Salt}}\n  \
                      - {at: 2s, event: shop.LineAdded, payload: {line_id: 5, label: Oil}}\n  \
                      - {at: 3s, event: shop.LineAdded, payload: {line_id: 6, label: Flour}}\n  \
                      - {at: 4s, event: shop.LineAdded, payload: {line_id: 2, label: Cream}}\n";

fn document(name: &str, effect: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("ess-ui-test-live-insert")
        .join(name);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("the old scratch dir is removed");
    }
    std::fs::create_dir_all(&dir).expect("the scratch dir is made");
    std::fs::write(dir.join("lines.yaml"), FIXTURE).expect("the fixture is written");
    std::fs::write(dir.join("script.yaml"), SCRIPT).expect("the script is written");
    std::fs::write(
        dir.join("ui.yaml"),
        format!(
            "format: ess-ui/1
app: shop
model: shop.system
placement_profile: fat
fixtures: {{views: {{shop.Lines: lines.yaml}}, scripts: {{orders: script.yaml}}}}
shells:
  app:
    regions:
      main: {{kind: page_outlet}}
navigation:
  home: lines
  sections: [{{name: all, pages: [lines]}}]
channels:
  orders:
    carries: {{events: [shop.LineAdded]}}
    direction: server_to_client
    delivery: every_event
    resume: refetch
pages:
  lines:
    kind: detail_page
    title: Lines
    sections:
      - name: lines
        component: collection
        reads: {{view: shop.Lines, key: line_id}}
        columns: [label]
        live: {{channel: orders, effect: {effect}{max_rows}}}
",
            max_rows = if name.ends_with("-capped") {
                ", max_rows: 5"
            } else {
                ""
            }
        ),
    )
    .expect("the document is written");
    dir.join("ui.yaml")
}

fn run(document: &Path, steps: &[&str]) -> Outcome {
    let mut text = format!(
        "format: ess-ui-test/1\ndocument: {}\ntests:\n- name: live insert\n  steps:\n",
        document.display()
    );
    for step in steps {
        text.push_str("  - ");
        text.push_str(step);
        text.push('\n');
    }
    let file = ess_ui_test::parse_str(&text, Path::new("live_insert.yaml"))
        .unwrap_or_else(|error| panic!("{error}\n{text}"));
    let report = ess_ui_test::execute(document, &[file]).unwrap_or_else(|error| panic!("{error}"));
    report.tests.into_iter().next().expect("one outcome")
}

const SECTION: &str = "pages/lines/sections/lines";

fn rows(keys: &str) -> String {
    format!("expect: {{at: {SECTION}, rows: [{keys}]}}")
}

#[test]
fn a_looping_script_that_inserts_a_held_key_again_moves_its_row_to_the_top() {
    let document = document("looping", "insert_top");
    let outcome = run(
        &document,
        &[
            "open: lines",
            &rows("1, 2, 3"),
            "advance: 3s",
            &rows("6, 5, 4, 1, 2, 3"),
            // The loop's first pass ends with a fixture row's key, the second begins with
            // events 4, 5 and 6 again: every one is a held key.
            "advance: 1s",
            &rows("2, 6, 5, 4, 1, 3"),
            "advance: 1s",
            &rows("4, 2, 6, 5, 1, 3"),
            "advance: 3s",
            &rows("2, 6, 5, 4, 1, 3"),
            "advance: 32s",
            &rows("2, 6, 5, 4, 1, 3"),
        ],
    );
    assert_eq!(outcome.status, Status::Passed, "{outcome:?}");
}

#[test]
fn an_insert_of_a_fixture_row_s_key_moves_that_row_to_the_top_with_the_event_s_values() {
    let document = document("fixture-key", "insert_top");
    let outcome = run(
        &document,
        &[
            "open: lines",
            "advance: 4s",
            &rows("2, 6, 5, 4, 1, 3"),
            "expect: {at: pages/lines/sections/lines/rows/2, text: Cream}",
            "expect: {at: pages/lines/sections/lines/rows/2, not_text: Milk}",
        ],
    );
    assert_eq!(outcome.status, Status::Passed, "{outcome:?}");
}

#[test]
fn the_cap_holds_after_a_held_key_moves() {
    let document = document("looping-capped", "insert_top");
    let outcome = run(
        &document,
        &[
            "open: lines",
            "advance: 4s",
            &rows("2, 6, 5, 4, 1"),
            "advance: 1s",
            &rows("4, 2, 6, 5, 1"),
        ],
    );
    assert_eq!(outcome.status, Status::Passed, "{outcome:?}");
}

#[test]
fn insert_or_patch_still_patches_a_held_row_in_place() {
    let document = document("patch", "insert_or_patch");
    let outcome = run(
        &document,
        &["open: lines", "advance: 4s", &rows("6, 5, 4, 1, 2, 3")],
    );
    assert_eq!(outcome.status, Status::Passed, "{outcome:?}");
}
