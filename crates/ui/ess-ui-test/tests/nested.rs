//! Record tabs, header actions and nested collections in the terminal: a tab's nested node is
//! drawn, a tab's action and a header action are driven, the collection of a record's `item` is
//! driven in both renderers, and another nested collection's rows are refused in both renderers
//! with one reason.

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use ess_ui_test::{Outcome, Status};

const DOCUMENT: &str = r"format: ess-ui/1
app: probe
model: probe.system
placement_profile: fat
fixtures: {views: {things.Page: things.yaml, parts.Page: things.yaml}}
shells:
  app:
    regions:
      nav:     {kind: navigation}
      main:    {kind: page_outlet}
      overlay: {kind: overlay_outlet}
navigation:
  home: things.list
  sections: [{name: all, pages: [things.list, things.detail]}]
pages:
  things.list:
    kind: list_page
    title: Things
    nav: {label: Things, synonyms: [Refresh]}
    sections:
      - name: list
        component: collection
        reads: {view: things.Page}
        columns: [name]
  things.detail:
    kind: detail_page
    title: Thing
    header:
      actions: [{name: refresh, does: things.Refresh, label: Refresh}]
    sections:
      - name: summary
        component: record
        reads: {view: things.Page}
        fields: [name]
        item:
          - name: parts
            component: collection
            reads: {view: parts.Page}
            columns: [label]
            row_actions: [{name: pick, label: Pick, does: things.Pick, bind: {id: row.id}}]
        tabs:
          - {name: info, label: Info, fields: [id]}
          - {name: notes, label: Notes, form: {name: archive, does: things.Archive, label: Archive}}
          - name: history
            label: History
            form: {component: collection, reads: {view: things.Page}, columns: [name]}
";

/// The document, written once: the tests run in parallel and would otherwise truncate the files
/// another test is reading.
fn document() -> PathBuf {
    static WRITTEN: OnceLock<PathBuf> = OnceLock::new();
    WRITTEN
        .get_or_init(|| {
            let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("ess-ui-test-nested");
            std::fs::create_dir_all(&dir).expect("the dir is made");
            std::fs::write(
                dir.join("things.yaml"),
                "views:\n  things.Page:\n    rows:\n      - {id: th-1, name: Anvil}\n      - {id: th-2, name: Bolt}\n  parts.Page:\n    rows:\n      - {id: pa-1, label: Cog}\n      - {id: pa-2, label: Gear}\n",
            )
            .expect("the fixture is written");
            std::fs::write(dir.join("ui.yaml"), DOCUMENT).expect("the document is written");
            dir.join("ui.yaml")
        })
        .clone()
}

fn file(steps: &[&str]) -> ess_ui_test::TestFile {
    let mut text = format!(
        "format: ess-ui-test/1\ndocument: {}\ntests:\n- name: nested\n  steps:\n",
        document().display()
    );
    for step in steps {
        text.push_str("  - ");
        text.push_str(step);
        text.push('\n');
    }
    ess_ui_test::parse_str(&text, Path::new("nested.yaml"))
        .unwrap_or_else(|error| panic!("{error}\n{text}"))
}

fn run(steps: &[&str]) -> Outcome {
    let report =
        ess_ui_test::execute(&document(), &[file(steps)]).unwrap_or_else(|error| panic!("{error}"));
    report.tests.into_iter().next().expect("one outcome")
}

fn spec(steps: &[&str]) -> String {
    ess_ui_test::playwright(
        &[file(steps)],
        &ess_ui::load_path(&document()).expect("the document loads"),
    )
}

fn passed(outcome: &Outcome) {
    assert_eq!(outcome.status, Status::Passed, "{outcome:?}");
}

#[test]
fn a_header_action_runs_though_a_page_shares_its_label() {
    passed(&run(&[
        "open: things.detail",
        "act: pages/things.detail/header/actions/refresh",
        "expect_command: things.Refresh",
    ]));
}

#[test]
fn a_tabs_action_is_driven_after_its_tab_is_shown() {
    passed(&run(&[
        "open: things.detail",
        "act: pages/things.detail/sections/summary/tabs/notes/form",
        "expect_command: things.Archive",
    ]));
}

#[test]
fn a_tabs_nested_node_is_drawn_when_its_tab_is_selected() {
    passed(&run(&[
        "open: things.detail",
        "expect: {at: pages/things.detail/sections/summary, not_text: Bolt}",
        "select: pages/things.detail/sections/summary/tabs/history",
        "expect: {at: pages/things.detail/sections/summary, text: Bolt}",
    ]));
}

#[test]
fn a_nested_collections_rows_are_refused_with_one_reason_in_both_renderers() {
    let steps = [
        "open: things.detail",
        "select: pages/things.detail/sections/summary/tabs/history/form/rows/th-2",
    ];
    let outcome = run(&steps);
    assert_eq!(outcome.step, Some(2), "{outcome:?}");
    let message = outcome.message.unwrap_or_default();
    assert!(message.contains("nested inside a record"), "{message}");
    let reason = message
        .split_once(": ")
        .map_or(message.as_str(), |(_, reason)| reason);
    let spec = spec(&steps);
    assert!(spec.contains("test.fixme("), "{spec}");
    assert!(
        spec.contains(&serde_json::to_string(reason).expect("a string")),
        "{reason}\n\n{spec}"
    );
}

/// A collection in a record's `item` has a cursor and row regions at the path React draws it at
/// (`…/sections/<s>/item/<name>/rows/<key>`): a journey expects its rows, selects one and acts on
/// its row action in the terminal, and the Playwright spec runs the same steps instead of marking
/// them `test.fixme` (beyond10x/ess#722, the third list).
#[test]
fn a_collection_in_a_records_item_is_driven_in_both_renderers() {
    let steps = [
        "open: things.detail",
        "expect: {at: pages/things.detail/sections/summary/item/parts, rows: 2}",
        "select: pages/things.detail/sections/summary/item/parts/rows/pa-2",
        "act: pages/things.detail/sections/summary/item/parts/rows/pa-2/row_actions/pick",
        "expect_command: {command: things.Pick, input: {id: pa-2}}",
    ];
    passed(&run(&steps));
    let spec = spec(&steps);
    assert!(!spec.contains("test.fixme("), "{spec}");
    assert!(
        spec.contains("sections/summary/item/parts/rows/pa-2/row_actions/pick"),
        "{spec}"
    );
}

/// The terminal moves through the rows of a record's `item` and runs their actions, but marks
/// none of them: a `mark` there is refused with one reason in both renderers.
#[test]
fn a_mark_in_a_records_item_collection_is_refused_with_one_reason_in_both_renderers() {
    let steps = [
        "open: things.detail",
        "mark: pages/things.detail/sections/summary/item/parts/rows/pa-2",
    ];
    let outcome = run(&steps);
    assert_eq!(outcome.step, Some(2), "{outcome:?}");
    let message = outcome.message.unwrap_or_default();
    assert!(message.contains("`select` and `act` only"), "{message}");
    let reason = message
        .split_once(": ")
        .map_or(message.as_str(), |(_, reason)| reason);
    let spec = spec(&steps);
    assert!(spec.contains("test.fixme("), "{spec}");
    assert!(
        spec.contains(&serde_json::to_string(reason).expect("a string")),
        "{reason}\n\n{spec}"
    );
}
