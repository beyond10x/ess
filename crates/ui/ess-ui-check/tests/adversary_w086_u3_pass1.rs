//! Adversary pass 1 on story:a-server-paged-collection-has-a-page-size-control: pagers the
//! check accepts that no renderer shows.

use std::path::PathBuf;

use ess_ui_check::{check_source, Options, Report};

fn document(section: &str) -> String {
    format!(
        "format: ess-ui/1
app: t
model: t.system
placement_profile: fat
shells: {{app: {{regions: {{main: {{kind: page_outlet}}}}}}}}
navigation: {{home: p, sections: [{{name: all, pages: [p]}}]}}
pages:
  p:
    kind: detail_page
    title: P
    state:
      at: {{type: integer, class: page_state, store: url, default: 1}}
    sections:
      - {section}
"
    )
}

fn report(text: &str) -> Report {
    check_source(
        text,
        "test.yaml",
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")),
        None,
        &Options::default(),
    )
}

/// "A declared pager always shows its footer": only the collection renders one
/// (`ess-ui-react/src/emit.rs`, the single `.opt("pager", ...)`), so a pager on another
/// composite's read is a key nothing honours and the check says so.
#[test]
fn adversary_a_pager_no_renderer_shows_is_reported() {
    let report = report(&document(
        "{name: one, component: record, reads: {view: t.ById, params: {p: state.at}, paging: server, pager: {page: p}}, fields: [label]}",
    ));
    let named: Vec<_> = report
        .findings
        .iter()
        .filter(|finding| finding.check.starts_with("pager") || finding.message.contains("pager"))
        .collect();
    assert!(
        !named.is_empty(),
        "a pager on a record's read is reported; findings: {:#?}",
        report.findings
    );
}
