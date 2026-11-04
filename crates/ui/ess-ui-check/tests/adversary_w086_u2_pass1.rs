//! Adversary pass 1 on story:a-toggle-shows-the-row-state-it-flips, check side: the Acceptance
//! says a `does:` toggle with no `checked:` and no `not` bind entry warns `toggle_state_unknown`.
//! Neither renderer reads an action's `loads` for a toggle's state (the React `ActionSpec` carries
//! no `loads`; the terminal's `toggle_on` reads `ess_ui::toggle_state` only), so a toggle with
//! `loads` renders off as well and is no exception.

use std::path::Path;

use ess_ui::{NodeRef, ToggleState};
use ess_ui_check::{check_source, Finding, Options, Report};

/// A page whose header carries `actions`.
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
    header:
      actions:
{actions}
    sections:
      - name: list
        component: collection
        reads: {{view: flags.All}}
        columns: [title]
"
    )
}

fn report(text: &str) -> Report {
    check_source(
        text,
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
fn a_command_toggle_with_loads_and_no_known_state_warns() {
    let text = document(
        "        - {name: live, as: toggle, label: Live, does: flags.SetLive, bind: {on: params.on}, loads: {view: flags.Live}}",
    );
    let document = ess_ui::load_str(&text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let state: Option<ToggleState> = document
        .nodes()
        .into_iter()
        .find_map(|located| match located.node {
            NodeRef::Action(action) if action.name == "live" => Some(ess_ui::toggle_state(action)),
            _ => None,
        })
        .expect("the header toggle is located");
    assert_eq!(
        state, None,
        "no expression names the toggle's state, so both renderers draw it off"
    );
    let report = report(&text);
    assert!(
        tripped(&report, "document_loads").is_empty(),
        "{:#?}",
        report.findings
    );
    let found = tripped(&report, "toggle_state_unknown");
    assert_eq!(
        found.len(),
        1,
        "a `does:` toggle that every renderer draws off warns, `loads` or not: {:#?}",
        report.findings
    );
    assert_eq!(found[0].path, "pages/flags.list/header/actions/live");
}
