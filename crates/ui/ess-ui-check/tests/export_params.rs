//! An export's params read `selection.<field>` only on a bulk action, where the section's
//! selection is in scope (story:a-refused-node-is-reported-at-its-own-path, beyond10x/ess#723).

use std::path::Path;

use ess_ui_check::{check_source, Finding, Options, Severity};

fn document(row_export: &str, bulk_export: &str) -> String {
    format!(
        "format: ess-ui/1
app: desk
model: desk.system
placement_profile: fat
shells:
  app: {{regions: {{main: {{kind: page_outlet}}}}}}
navigation:
  home: files
  sections: [{{name: all, pages: [files]}}]
pages:
  files:
    kind: list_page
    title: Files
    sections:
      - name: list
        component: collection
        selection: multiple
        reads: {{view: desk.Files}}
        columns: [name]
        row_actions:
          - {{name: fetch, label: Fetch, export: {row_export}}}
        actions:
          - {{name: everything, label: Everything, export: {{reads: desk.Bundle, as: zip, params: selection.id}}}}
        bulk_actions:
          - {{name: download, label: Download, export: {bulk_export}}}
"
    )
}

fn findings(text: &str) -> Vec<Finding> {
    check_source(
        text,
        "test.yaml",
        Path::new(env!("CARGO_MANIFEST_DIR")),
        None,
        &Options::default(),
    )
    .findings
    .into_iter()
    .filter(|finding| finding.check == "export_selection")
    .collect()
}

#[test]
fn a_selection_param_outside_a_bulk_action_is_an_error() {
    let text = document(
        "{reads: desk.Bundle, as: zip, params: {ids: selection.id, kind: row.kind}}",
        "{reads: desk.Bundle, as: zip, params: {ids: selection.id, all: section.list.selection}}",
    );
    let found = findings(&text);
    let at: Vec<&str> = found.iter().map(|finding| finding.path.as_str()).collect();
    assert_eq!(
        at,
        [
            "pages/files/sections/list/actions/everything/export/params",
            "pages/files/sections/list/row_actions/fetch/export/params/ids",
        ],
        "{found:#?}"
    );
    for finding in &found {
        assert_eq!(finding.severity, Severity::Error, "{finding:?}");
        assert!(
            finding.message.contains("selection.id") && finding.message.contains("bulk"),
            "{finding:?}"
        );
    }
}

#[test]
fn a_selection_param_on_a_bulk_action_is_fine() {
    let text = document(
        "{reads: desk.Bundle, as: zip, params: {kind: row.kind}}",
        "{reads: desk.Bundle, as: zip, params: {ids: selection.id}}",
    )
    .replace(
        "params: selection.id}}",
        "params: {all: section.list.selection}}}",
    );
    let found = findings(&text);
    assert!(found.is_empty(), "{found:#?}");
}

/// The runtime puts the selection in scope only around `bulk_actions`, so `bulk: true` on an
/// action elsewhere does not make it a bulk action.
#[test]
fn a_bulk_flag_outside_bulk_actions_is_no_bulk_action() {
    let text = document(
        "{reads: desk.Bundle, as: zip, params: {kind: row.kind}}",
        "{reads: desk.Bundle, as: zip, params: {ids: selection.id}}",
    )
    .replace(
        "{name: everything, label: Everything, export:",
        "{name: everything, label: Everything, bulk: true, export:",
    );
    assert!(text.contains("bulk: true"), "{text}");
    let found = findings(&text);
    let at: Vec<&str> = found.iter().map(|finding| finding.path.as_str()).collect();
    assert_eq!(
        at,
        ["pages/files/sections/list/actions/everything/export/params"],
        "story:a-refused-node-is-reported-at-its-own-path: `bulk: true` outside `bulk_actions` \
         has no selection in scope: {found:#?}"
    );
    assert!(
        found[0].message.contains("`bulk_actions`") && !found[0].message.contains("bulk: true"),
        "the message names the one place the selection is in scope: {:?}",
        found[0]
    );
}
