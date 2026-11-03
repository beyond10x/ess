//! The two rules on value labels (story:a-value-shows-its-declared-label-beside-its-tone):
//! `label_map_refs` refuses a `labels:` naming no entry of `label_maps`, and `label_map_unused`
//! warns of an entry no field or badge names.

use std::path::PathBuf;

use ess_ui_check::{check_source, Finding, Options, Report, Severity};

fn document(label_maps: &str, column: &str, badge: &str) -> String {
    format!(
        "format: ess-ui/1
app: t
model: t.system
placement_profile: fat
shells: {{app: {{regions: {{main: {{kind: page_outlet}}}}}}}}
navigation: {{home: p, sections: [{{name: all, pages: [p, q]}}]}}
{label_maps}pages:
  p:
    kind: list_page
    title: Runs
    sections:
      - name: runs
        component: collection
        reads: t.Runs
        columns:
          - {column}
  q:
    kind: detail_page
    title: Run
    sections:
      - name: summary
        reads: t.ById
        children:
          - {badge}
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

fn findings<'a>(report: &'a Report, id: &str) -> Vec<&'a Finding> {
    report
        .findings
        .iter()
        .filter(|finding| finding.check == id)
        .collect()
}

#[test]
fn an_unknown_label_map_name_is_an_error() {
    let text = document(
        "label_maps:\n  run_state: {in_progress: In progress}\n",
        "{field: state, as: badge, labels: nope}",
        "{name: state, primitive: badge, field: state, labels: run_state}",
    );
    let report = report(&text);
    let found = findings(&report, "label_map_refs");
    let finding = found
        .iter()
        .find(|finding| finding.path == "pages/p/sections/runs/columns/state/labels")
        .unwrap_or_else(|| panic!("no `label_map_refs` at the field: {:#?}", report.findings));
    assert_eq!(finding.severity, Severity::Error, "{finding:?}");
    assert!(finding.message.contains("nope"), "{finding:?}");
    assert!(report.has_errors());

    let badge = document(
        "label_maps:\n  run_state: {in_progress: In progress}\n",
        "{field: state, as: badge, labels: run_state}",
        "{name: state, primitive: badge, field: state, labels: nope}",
    );
    let report = self::report(&badge);
    assert!(
        findings(&report, "label_map_refs")
            .iter()
            .any(|finding| finding.path == "pages/q/sections/summary/children/state/labels"),
        "a badge naming no label map is refused at the badge: {:#?}",
        report.findings
    );

    let none = document(
        "",
        "{field: state, as: badge, labels: nope}",
        "{name: state, primitive: badge, field: state}",
    );
    let report = self::report(&none);
    let found = findings(&report, "label_map_refs");
    assert!(
        found.iter().any(
            |finding| finding.path == "pages/p/sections/runs/columns/state/labels"
                && finding.message.contains("declares no `label_maps`")
        ),
        "with no `label_maps` at all: {:#?}",
        report.findings
    );

    let fine = document(
        "label_maps:\n  run_state: {in_progress: In progress}\n",
        "{field: state, as: badge, labels: run_state}",
        "{name: state, primitive: badge, field: state, labels: {in_progress: Running}}",
    );
    let report = self::report(&fine);
    assert!(!report.has_errors(), "{:#?}", report.findings);
}

#[test]
fn an_unused_label_map_is_a_warning() {
    let text = document(
        "label_maps:\n  run_state: {in_progress: In progress}\n  \
         by_column: {done: Done}\n  by_badge: {failed: Failed}\n",
        "{field: state, as: badge, labels: by_column}",
        "{name: state, primitive: badge, field: state, labels: by_badge}",
    );
    let report = report(&text);
    let unused = findings(&report, "label_map_unused");
    let paths: Vec<&str> = unused.iter().map(|finding| finding.path.as_str()).collect();
    assert_eq!(
        paths,
        ["label_maps/run_state"],
        "only the map no field or badge names warns: {:#?}",
        report.findings
    );
    assert_eq!(unused[0].severity, Severity::Warning);
    assert!(unused[0].message.contains("run_state"), "{:?}", unused[0]);
    assert!(!report.has_errors(), "{:#?}", report.findings);
}
