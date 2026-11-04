//! A document using the three presentation keys of
//! story:choices-filter-bars-and-forms-render-what-they-declare (`placeholder` on a choice,
//! `reset: {label}` on a filter bar, `help` on a form group, a field and a column) checks clean.

use std::path::PathBuf;

use ess_ui_check::{check_source, Options, Severity};

const DOCUMENT: &str = r"format: ess-ui/1
app: shelf
model: shelf.system
placement_profile: fat
shells:
  app: {regions: {main: {kind: page_outlet}}}
navigation: {home: p, sections: [{name: all, pages: [p]}]}
pages:
  p:
    kind: detail_page
    title: Items
    state:
      stage: {type: string, class: page_state, store: url}
    sections:
      - name: filters
        component: filter_bar
        binds: [state.stage]
        choices:
          - {name: stage, component: choice, options: [open, won], placeholder: Any stage}
        reset: {label: Clear filters}
      - name: list
        component: collection
        reads: {view: shelf.Items, params: {stage: state.stage}}
        columns:
          - {field: title, help: {text: The name shown on the shelf}}
      - name: form
        component: form
        does: shelf.Configure
        groups:
          - name: profile
            label: Profile
            help: {text: Who sees the shelf, link: 'https://example.test/profile'}
            fields:
              - {field: owner, help: {text: The person who answers for it}}
";

#[test]
fn the_three_keys_check_clean() {
    let report = check_source(
        DOCUMENT,
        "shelf.yaml",
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")),
        None,
        &Options::default(),
    );
    let errors: Vec<_> = report
        .findings
        .iter()
        .filter(|finding| finding.severity == Severity::Error)
        .collect();
    assert!(errors.is_empty(), "{errors:#?}");
    assert!(!report.has_errors(), "{:#?}", report.findings);
}
