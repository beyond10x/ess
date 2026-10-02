//! One authored test remains meaningful in both renderers after client row filtering.
use std::path::Path;

#[test]
fn filtered_rows_empty_state_and_total_have_no_parity_refusal() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("ui.yaml");
    let document = r"format: ess-ui/1
app: filtered
model: t
placement_profile: fat
shells: {app: {regions: {main: {kind: page_outlet}, nav: {kind: navigation}}}}
navigation: {home: p, sections: [{name: items, pages: {from_view: t.All, page: p, param: id, label: row.label, filter: row.group == one}}]}
pages:
  p:
    kind: detail_page
    title: Filtered
    params: {id: string}
    state: {pick: {type: string, class: component_state, default: one}}
    header: {total: list, actions: [{name: empty, label: Empty, sets: {state.pick: missing}}]}
    sections:
      - {name: list, component: collection, reads: {view: t.All, filter: row.group == state.pick}, columns: [label], states: {empty: {message: Nobody}}}
";
    std::fs::write(&path, document).unwrap();
    let text = format!(
        r"format: ess-ui-test/1
document: {}
tests:
  - name: bounded filters
    fixtures:
      views: {{t.All: {{rows: [{{id: a, label: Alpha, group: one}}, {{id: b, label: Beta, group: two}}], total: 99}}}}
    steps:
      - open: p
      - expect: {{at: pages/p/sections/list, rows: [a]}}
      - expect: {{at: pages/p/header, not_text: '99'}}
      - act: pages/p/header/actions/empty
      - expect: {{at: pages/p/sections/list, rows: 0, state: empty, text: Nobody}}
",
        path.display()
    );
    let file = ess_ui_test::parse_str(&text, Path::new("filter.test.yaml")).unwrap();
    let report = ess_ui_test::execute(&path, std::slice::from_ref(&file)).unwrap();
    assert_eq!(
        report.tests[0].status,
        ess_ui_test::Status::Passed,
        "{report:?}"
    );
    let playwright = ess_ui_test::playwright(&[file], &ess_ui::load_str(document).unwrap());
    assert!(!playwright.contains("test.fixme"), "{playwright}");
    assert!(playwright.contains("Nobody"));
}
