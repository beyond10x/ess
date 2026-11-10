//! A field takes a tone (story:a-badge-field-takes-a-tone): `tone_by` naming no tone map is
//! `tone_map_refs` at the field, a map a field names is no longer `tone_map_unused`, and a band
//! map whose `from` values do not strictly ascend is refused (`tone_bands`).

use std::path::PathBuf;

use ess_ui_check::{check_source, Finding, Options, Report, Severity};

fn document(tone_maps: &str, column: &str) -> String {
    format!(
        "format: ess-ui/1
app: t
model: t.system
placement_profile: fat
shells: {{app: {{regions: {{main: {{kind: page_outlet}}}}}}}}
navigation: {{home: p, sections: [{{name: all, pages: [p]}}]}}
{tone_maps}pages:
  p:
    kind: list_page
    title: Runs
    sections:
      - name: runs
        component: collection
        reads: t.Runs
        columns:
          - {column}
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
fn a_tone_by_naming_no_map_is_an_error_at_the_field() {
    let text = document(
        "tone_maps:\n  run_state: {done: success}\n",
        "{field: state, as: badge, tone_by: nope}",
    );
    let report = report(&text);
    let found = findings(&report, "tone_map_refs");
    let finding = found
        .iter()
        .find(|finding| finding.path == "pages/p/sections/runs/columns/state/tone_by")
        .unwrap_or_else(|| panic!("no `tone_map_refs` at the field: {:#?}", report.findings));
    assert_eq!(finding.severity, Severity::Error, "{finding:?}");
    assert!(finding.message.contains("nope"), "{finding:?}");
    assert!(report.has_errors());
}

#[test]
fn a_map_a_field_names_is_not_unused() {
    let text = document(
        "tone_maps:\n  run_state: {done: success}\n",
        "{field: state, as: badge, tone: info, tone_by: run_state}",
    );
    let report = report(&text);
    assert!(
        findings(&report, "tone_map_unused").is_empty(),
        "{:#?}",
        report.findings
    );
    assert!(!report.has_errors(), "{:#?}", report.findings);
}

#[test]
fn a_map_no_field_names_is_still_unused() {
    let text = document(
        "tone_maps:\n  run_state: {done: success}\n",
        "{field: state, as: badge}",
    );
    let report = report(&text);
    assert_eq!(findings(&report, "tone_map_unused").len(), 1);
}

#[test]
fn bands_that_do_not_ascend_are_refused() {
    let text = document(
        "tone_maps:\n  load: {bands: [{from: 0, tone: success}, {from: 80, tone: danger}, {from: 80, tone: warning}]}\n",
        "{field: load, tone_by: load}",
    );
    let report = report(&text);
    let found = findings(&report, "tone_bands");
    let finding = found
        .iter()
        .find(|finding| finding.path == "tone_maps/load")
        .unwrap_or_else(|| panic!("no `tone_bands` at the map: {:#?}", report.findings));
    assert_eq!(finding.severity, Severity::Error, "{finding:?}");
    assert!(report.has_errors());
}

#[test]
fn ascending_bands_are_clean() {
    let text = document(
        "tone_maps:\n  load: {bands: [{from: 0, tone: success}, {from: 50, tone: warning}, {from: 80, tone: danger}]}\n",
        "{field: load, tone_by: load}",
    );
    let report = report(&text);
    assert!(
        findings(&report, "tone_bands").is_empty(),
        "{:#?}",
        report.findings
    );
    assert!(!report.has_errors(), "{:#?}", report.findings);
}
