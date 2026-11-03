//! Adversary pass 2 on story:a-toggle-shows-the-row-state-it-flips, check side, against fix
//! round 1 (the `loads` exemption removed, the state read off the parse tree).

use std::path::Path;

use ess_ui_check::{check_source, Finding, Options, Report};

fn document(row_actions: &str) -> String {
    format!(
        r"
format: ess-ui/1
app: posts
model: posts.system
placement_profile: fat
shells:
  app: {{regions: {{main: {{kind: page_outlet}}}}}}
navigation:
  home: posts.list
  sections: [{{name: all, pages: [posts.list]}}]
pages:
  posts.list:
    kind: detail_page
    title: Posts
    sections:
      - name: list
        component: collection
        reads: {{view: posts.All}}
        columns: [title]
        row_actions:
{row_actions}
"
    )
}

fn report(text: &str, name: &str) -> Report {
    check_source(
        text,
        name,
        Path::new(env!("CARGO_MANIFEST_DIR")),
        None,
        &Options::default(),
    )
}

fn tripped<'a>(report: &'a Report, id: &str) -> Vec<&'a Finding> {
    report
        .findings
        .iter()
        .filter(|finding| finding.check == id)
        .collect()
}

/// `not item.enabled` reads no root (`item` is not one), so both renderers evaluate the entry and
/// the state the library names from it as literal text: the toggle is drawn on for every row.
/// The rule's subject is a `does:` toggle whose state no expression names; literal text names
/// none.
#[test]
fn a_not_over_literal_dotted_text_warns() {
    let text = document(
        "          - {name: enabled, as: toggle, label: Enabled, does: posts.SetEnabled, bind: {id: row.id, enabled: not item.enabled}}",
    );
    let report = report(&text, "posts.yaml");
    assert!(
        tripped(&report, "document_loads").is_empty(),
        "{:#?}",
        report.findings
    );
    assert_eq!(
        tripped(&report, "toggle_state_unknown").len(),
        1,
        "`not item.enabled` is literal text, so no state is known: {:#?}",
        report.findings
    );
}

/// The `loads` removal adds no `toggle_state_unknown` to a shipped example document.
#[test]
fn no_example_document_gains_a_toggle_state_warning() {
    let examples = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../examples");
    let mut checked = 0;
    for entry in std::fs::read_dir(&examples).expect("the examples directory is read") {
        let ui = entry.expect("an entry").path().join("ui.yaml");
        if !ui.is_file() {
            continue;
        }
        let text = std::fs::read_to_string(&ui).expect("the document is read");
        let report = report(&text, &ui.display().to_string());
        let found = tripped(&report, "toggle_state_unknown");
        assert!(found.is_empty(), "{}: {found:#?}", ui.display());
        checked += 1;
    }
    assert!(
        checked > 0,
        "no example ui.yaml was found under {}",
        examples.display()
    );
}
