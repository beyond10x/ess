//! `as: bytes` on a field passes `enum_values` (story:a-value-renders-in-the-form-its-field-declares),
//! as `format: bytes` on a metric does; a word the schema does not list is still refused.

use std::path::PathBuf;

use ess_ui_check::{check_source, Finding, Options, Report, Severity};

fn document(column: &str) -> String {
    format!(
        "format: ess-ui/1
app: t
model: t.system
placement_profile: fat
shells: {{app: {{regions: {{main: {{kind: page_outlet}}}}}}}}
navigation: {{home: p, sections: [{{name: all, pages: [p]}}]}}
pages:
  p:
    kind: list_page
    title: Files
    sections:
      - {{name: used, component: metric, reads: t.Usage, format: bytes}}
      - name: files
        component: collection
        reads: t.Files
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

fn enum_findings(report: &Report) -> Vec<&Finding> {
    report
        .findings
        .iter()
        .filter(|finding| finding.check == "enum_values")
        .collect()
}

#[test]
fn a_bytes_field_passes_the_enum_check() {
    let report = report(&document("{field: size, as: bytes}"));
    assert!(
        enum_findings(&report).is_empty(),
        "`as: bytes` is a value of `as`: {:#?}",
        report.findings
    );
}

#[test]
fn a_word_the_schema_does_not_list_is_still_refused() {
    let report = report(&document("{field: size, as: kilobytes}"));
    let found = enum_findings(&report);
    let finding = found
        .iter()
        .find(|finding| finding.path == "pages/p/sections/files/columns/size")
        .unwrap_or_else(|| panic!("no `enum_values` at the column: {:#?}", report.findings));
    assert_eq!(finding.severity, Severity::Error, "{finding:?}");
    assert!(
        finding.message.contains("bytes"),
        "the message lists bytes: {finding:?}"
    );
}
