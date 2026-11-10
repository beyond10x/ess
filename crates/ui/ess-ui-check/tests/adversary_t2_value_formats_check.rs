//! Adversary cases for story:a-declared-field-format-renders-or-is-refused: the schema says a
//! field's `scale` is "only on such a field" (`as: percent`) and a metric's `unit` / `scale` are
//! "only with that format", the same promise `unit` on a field makes and `check_unit` keeps.

use std::path::PathBuf;

use ess_ui_check::{check_source, Finding, Options, Report, Severity};

fn document(metric: &str, column: &str) -> String {
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
      - {metric}
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

fn errors(report: &Report) -> Vec<&Finding> {
    report
        .findings
        .iter()
        .filter(|finding| finding.severity == Severity::Error)
        .collect()
}

const METRIC: &str = "{name: used, component: metric, reads: t.Usage, format: bytes}";

#[test]
fn a_scale_on_a_field_that_is_no_percent_is_refused() {
    let report = report(&document(
        METRIC,
        "{field: size, as: duration, scale: hundredths}",
    ));
    assert!(
        errors(&report)
            .iter()
            .any(|finding| finding.message.contains("scale")),
        "`scale` is only on an `as: percent` field: {:#?}",
        report.findings
    );
}

#[test]
fn a_metric_unit_without_format_duration_is_refused() {
    let report = report(&document(
        "{name: used, component: metric, reads: t.Usage, format: bytes, unit: m}",
        "{field: size}",
    ));
    assert!(
        errors(&report)
            .iter()
            .any(|finding| finding.message.contains("unit")),
        "a metric `unit` is only with `format: duration`: {:#?}",
        report.findings
    );
}
