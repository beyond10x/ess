//! Adversary pass 1 on story:a-refused-node-is-reported-at-its-own-path (wave 0.86.0, unit U4):
//! `export_selection` at every non-bulk action position the walker reaches, and not on a bulk
//! action's nested names.

use std::path::Path;

use ess_ui_check::{check_source, Options};

const SELECTED: &str = "export: {reads: desk.Bundle, as: zip, params: {ids: selection.id}}";

fn document() -> String {
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
    header:
      actions: [{{name: top, label: Top, {SELECTED}}}]
    sections:
      - name: list
        component: collection
        selection: multiple
        reads: {{view: desk.Files}}
        columns: [name]
        states: {{empty: {{message: Nothing, action: {{name: blank, label: Blank, {SELECTED}}}}}}}
        bulk_actions:
          - {{name: download, label: Download, {SELECTED}}}
      - name: one
        component: record
        reads: {{view: desk.File}}
        fields: [name]
        actions: [{{name: single, label: Single, {SELECTED}}}]
"
    )
}

#[test]
fn every_non_bulk_position_is_checked() {
    let text = document();
    let report = check_source(
        &text,
        "test.yaml",
        Path::new(env!("CARGO_MANIFEST_DIR")),
        None,
        &Options::default(),
    );
    let mut at: Vec<String> = report
        .findings
        .iter()
        .filter(|finding| finding.check == "export_selection")
        .map(|finding| finding.path.clone())
        .collect();
    at.sort();
    assert_eq!(
        at,
        [
            "pages/files/header/actions/top/export/params/ids",
            "pages/files/sections/list/states/empty/action/export/params/ids",
            "pages/files/sections/one/actions/single/export/params/ids",
        ],
        "{:#?}",
        report.findings
    );
}
