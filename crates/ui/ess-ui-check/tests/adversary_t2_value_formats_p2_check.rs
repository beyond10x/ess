//! Adversary cases, pass 2, for story:a-declared-field-format-renders-or-is-refused: the load
//! refusals of a misplaced `scale` / `unit` (ess-ui `path.rs` `check_scale`,
//! `check_metric_format`) in a form, a header metric and under an `UNMAPPED:` marker.

use std::path::PathBuf;

use ess_ui_check::{check_source, Finding, Options, Report, Severity};

fn document(header: &str, sections: &str) -> String {
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
    title: Files
    header: {{title: Files, metrics: [{header}]}}
    sections:
{sections}"
    )
}

const PLAIN_HEADER: &str = "{name: total, component: metric, reads: t.Usage, format: bytes}";

fn form(field: &str) -> String {
    format!(
        "      - name: edit
        component: form
        does: t.Configure
        groups:
          - name: profile
            fields:
              - {field}
"
    )
}

fn column(field: &str) -> String {
    format!(
        "      - name: files
        component: collection
        reads: t.Files
        columns:
          - {field}
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

fn errors_at<'a>(report: &'a Report, tail: &str) -> Vec<&'a Finding> {
    report
        .findings
        .iter()
        .filter(|finding| finding.severity == Severity::Error && finding.path.ends_with(tail))
        .collect()
}

/// Every error that refuses a `scale` or a `unit`, wherever it stands.
fn refusals(report: &Report) -> Vec<&Finding> {
    report
        .findings
        .iter()
        .filter(|finding| {
            finding.severity == Severity::Error
                && (finding.message.contains("`scale: ") || finding.message.contains("`unit: "))
        })
        .collect()
}

#[test]
fn a_form_field_scale_is_refused_unless_it_is_a_percent() {
    let refused = report(&document(
        PLAIN_HEADER,
        &form("{field: rate, as: number, scale: hundredths}"),
    ));
    assert!(
        !errors_at(&refused, "groups/profile/fields/rate/scale").is_empty(),
        "a form field `as: number` refuses `scale` at its key: {:#?}",
        refused.findings
    );
    let kept = report(&document(
        PLAIN_HEADER,
        &form("{field: rate, as: percent, scale: hundredths}"),
    ));
    assert!(
        refusals(&kept).is_empty(),
        "a form field `as: percent` keeps `scale`: {:#?}",
        kept.findings
    );
}

#[test]
fn an_unmapped_as_or_format_keeps_its_scale_and_unit() {
    let field = report(&document(
        PLAIN_HEADER,
        &column("{field: rate, as: 'UNMAPPED: not read yet', scale: hundredths, unit: s}"),
    ));
    assert!(
        refusals(&field).is_empty(),
        "an `UNMAPPED:` `as` keeps `scale` and `unit`: {:#?}",
        field.findings
    );
    let metric = report(&document(
        "{name: total, component: metric, reads: t.Usage, format: 'UNMAPPED: not read yet', \
         unit: s, scale: hundredths}",
        &column("{field: size}"),
    ));
    assert!(
        refusals(&metric).is_empty(),
        "an `UNMAPPED:` metric format keeps `unit` and `scale`: {:#?}",
        metric.findings
    );
}

#[test]
fn a_header_metric_refuses_a_unit_or_scale_its_format_does_not_take() {
    for (metric, key) in [
        (
            "{name: total, component: metric, reads: t.Usage, format: percent, unit: s}",
            "header/metrics/total/unit",
        ),
        (
            "{name: total, component: metric, reads: t.Usage, format: duration, scale: hundredths}",
            "header/metrics/total/scale",
        ),
        (
            "{name: total, component: metric, reads: t.Usage, unit: m}",
            "header/metrics/total/unit",
        ),
    ] {
        let refused = report(&document(metric, &column("{field: size}")));
        assert!(
            !errors_at(&refused, key).is_empty(),
            "{metric} is refused at {key}: {:#?}",
            refused.findings
        );
    }
}
