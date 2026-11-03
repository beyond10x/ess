//! The two rules on a toggle's state: `checked:` belongs to a toggle only, and a toggle that runs
//! a command with no state to show (no `checked:`, no `bind` entry written `not <expr>`) is
//! reported, while the document still loads.

use std::path::Path;

use ess_ui_check::{check_source, Finding, Options, Report, Severity};

const ROW_ACTIONS: &str = "pages/flags.list/sections/list/row_actions";

/// A list of flags whose row actions are written as `actions`.
fn document(actions: &str) -> String {
    format!(
        r"
format: ess-ui/1
app: flags
model: flags.system
placement_profile: fat
shells:
  app: {{regions: {{main: {{kind: page_outlet}}}}}}
navigation:
  home: flags.list
  sections: [{{name: all, pages: [flags.list]}}]
pages:
  flags.list:
    kind: detail_page
    title: Flags
    sections:
      - name: list
        component: collection
        reads: {{view: flags.All}}
        columns: [title]
        row_actions:
{actions}
"
    )
}

fn report(actions: &str) -> Report {
    let text = document(actions);
    check_source(
        &text,
        "flags.yaml",
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

#[test]
fn checked_on_an_action_that_is_not_a_toggle_is_an_error() {
    let report = report(concat!(
        "          - {name: archive, does: flags.Archive, bind: {id: row.id}, checked: row.archived}\n",
        "          - {name: enabled, as: toggle, does: flags.SetFlag, bind: {id: row.id, on: row.wanted}, checked: row.enabled}",
    ));
    assert!(
        tripped(&report, "document_loads").is_empty(),
        "the document loads: {:#?}",
        report.findings
    );
    let found = tripped(&report, "checked_needs_toggle");
    assert_eq!(found.len(), 1, "{:#?}", report.findings);
    assert_eq!(found[0].path, format!("{ROW_ACTIONS}/archive"));
    assert_eq!(found[0].severity, Severity::Error);
    assert!(
        found[0].message.contains("as: toggle"),
        "the message names what `checked` needs: {}",
        found[0].message
    );
    assert!(report.has_errors());
}

#[test]
fn a_command_toggle_with_no_known_state_warns() {
    let report = report(concat!(
        "          - {name: enabled, as: toggle, does: flags.SetFlag, bind: {id: row.id}}\n",
        "          - {name: flipped, as: toggle, does: flags.SetFlag, bind: {id: row.id, on: not row.on}}\n",
        "          - {name: checked, as: toggle, does: flags.SetFlag, bind: {id: row.id}, checked: row.on}\n",
        "          - {name: pause, as: toggle, sets: {state.paused: toggle}}",
    ));
    assert!(
        tripped(&report, "document_loads").is_empty(),
        "the document still loads: {:#?}",
        report.findings
    );
    let found = tripped(&report, "toggle_state_unknown");
    assert_eq!(
        found.len(),
        1,
        "only the toggle with neither `checked` nor a `not` bind entry: {:#?}",
        report.findings
    );
    assert_eq!(found[0].path, format!("{ROW_ACTIONS}/enabled"));
    assert_eq!(found[0].severity, Severity::Warning);
    assert!(
        found[0].message.contains("checked"),
        "the message names the way out: {}",
        found[0].message
    );
    assert!(
        tripped(&report, "checked_needs_toggle").is_empty(),
        "{:#?}",
        report.findings
    );
}
