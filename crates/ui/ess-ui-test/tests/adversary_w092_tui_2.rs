//! Adversary, unit w092-tui: host slots at the scopes the unit's own fixture does not declare —
//! a record's field, a record tab's `form` and a collection's column — read and driven through
//! `ess-ui-test/1` against the terminal target.

use std::path::{Path, PathBuf};

use ess_ui_test::{Outcome, Status};

const DOCUMENT: &str = "format: ess-ui/1
app: t
model: t.system
placement_profile: fat
fixtures: {views: {t.ById: byid.yaml, t.Routes: routes.yaml}}
shells:
  app:
    regions:
      main: {kind: page_outlet}
navigation: {home: p, sections: [{name: all, pages: [p]}]}
pages:
  p:
    kind: detail_page
    title: P
    sections:
      - name: summary
        component: record
        reads: {view: t.ById}
        fields: [name, {field: notes, host: NotesEditor, reason: needs spell checking, stands_in_for: rich_text}]
        tabs: [{name: history, form: {host: AuditTrail, reason: draws a timeline, stands_in_for: collection}}]
      - name: routes
        component: collection
        reads: {view: t.Routes}
        columns: [name, {field: route, host: RouteCell, reason: a vendor cell, stands_in_for: text}]
";

const BYID: &str = "views:\n  t.ById:\n    rows:\n      - {id: k-1, name: Anvil, notes: hi}\n";
const ROUTES: &str = "views:\n  t.Routes:\n    rows:\n      - {id: r-1, name: North, route: A12}\n";

fn document(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("ess-ui-test-adversary-w092-tui-2")
        .join(name);
    std::fs::create_dir_all(&dir).expect("the dir is made");
    std::fs::write(dir.join("ui.yaml"), DOCUMENT).expect("the document is written");
    std::fs::write(dir.join("byid.yaml"), BYID).expect("the fixture is written");
    std::fs::write(dir.join("routes.yaml"), ROUTES).expect("the fixture is written");
    dir.join("ui.yaml")
}

fn run(name: &str, steps: &[&str]) -> Outcome {
    let mut text = format!(
        "format: ess-ui-test/1\ndocument: {}\ntests:\n- name: slot\n  steps:\n",
        document(name).display()
    );
    for step in steps {
        text.push_str("  - ");
        text.push_str(step);
        text.push('\n');
    }
    let file = ess_ui_test::parse_str(&text, Path::new("adversary.yaml"))
        .unwrap_or_else(|error| panic!("{error}\n{text}"));
    let report =
        ess_ui_test::execute(&document(name), &[file]).unwrap_or_else(|error| panic!("{error}"));
    report.tests.into_iter().next().expect("one outcome")
}

#[test]
fn the_document_declares_the_three_slots_at_these_paths() {
    let document = ess_ui::load_str(DOCUMENT).unwrap_or_else(|error| panic!("{error}"));
    let paths: Vec<String> = document
        .host_slots()
        .into_iter()
        .map(|(path, _)| path.to_string())
        .collect();
    assert_eq!(
        paths,
        vec![
            "pages/p/sections/summary/fields/notes",
            "pages/p/sections/summary/tabs/history/form",
            "pages/p/sections/routes/columns/route",
        ],
        "{paths:?}"
    );
}

/// Acceptance: "An `ess-ui-test/1` `text` check of `<host>: <reason>` at the slot's path passes
/// against the terminal target" — a field slot of a record.
#[test]
fn a_text_check_at_a_record_field_slot_passes() {
    let outcome = run(
        "record_field",
        &[
            "open: p",
            "expect: {at: pages/p/sections/summary/fields/notes, text: \"NotesEditor: needs spell checking\"}",
        ],
    );
    assert_eq!(outcome.status, Status::Passed, "{outcome:?}");
}

/// The same line, for a slot at a record tab's `form` (the first and only tab, shown).
#[test]
fn a_text_check_at_a_tab_form_slot_passes() {
    let outcome = run(
        "tab_form",
        &[
            "open: p",
            "expect: {at: pages/p/sections/summary/tabs/history/form, text: \"AuditTrail: draws a timeline\"}",
        ],
    );
    assert_eq!(outcome.status, Status::Passed, "{outcome:?}");
}

/// Acceptance: a step "addressing the slot's path or a path below it — is refused on every
/// target by one rule in `parity.rs`, with the message ..." A column slot is addressed in a row:
/// the row-scoped path resolves to the slot's node.
#[test]
fn act_at_a_column_slot_in_a_row_is_refused_by_name() {
    let at = "pages/p/sections/routes/rows/r-1/columns/route";
    let outcome = run("column_act", &["open: p", &format!("act: {at}")]);
    assert_eq!(outcome.status, Status::Failed, "{outcome:?}");
    assert_eq!(outcome.step, Some(2), "{outcome:?}");
    let wanted = format!(
        "`{at}` is host slot `RouteCell`: the host draws it, so a test cannot act inside it"
    );
    let message = outcome.message.as_deref().unwrap_or_default();
    assert!(
        message.ends_with(&format!(": {wanted}")),
        "expected the refusal\n  {wanted}\nfound\n  {message}"
    );
}
