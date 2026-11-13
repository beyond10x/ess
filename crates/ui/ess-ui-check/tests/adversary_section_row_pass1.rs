//! Adversary pass 1 on `section_row`: sites and cycles beyond the unit's own cases.

use std::path::PathBuf;

use ess_ui_check::{check_source, Finding, Options, Report};

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn page(extra: &str, sections: &str) -> String {
    format!(
        "format: ess-ui/1\napp: t\nmodel: shop\nplacement_profile: fat\n\
         shells: {{app: {{regions: {{main: {{kind: page_outlet}}}}}}}}\n\
         navigation: {{home: p, sections: [{{name: all, pages: [p]}}]}}\n\
         pages:\n  p:\n    kind: detail_page\n    title: P\n{extra}    sections:\n{sections}\n"
    )
}

fn report(text: &str) -> Report {
    check_source(text, "test.yaml", &crate_dir(), None, &Options::default())
}

fn row_findings(report: &Report) -> Vec<&Finding> {
    report
        .findings
        .iter()
        .filter(|finding| finding.check == "section_row")
        .collect()
}

#[test]
fn a_cycle_of_three_sections_is_refused_naming_all_three() {
    let text = page(
        "",
        "      - {name: a, component: record, reads: {view: t.ById, params: {id: section.c.row.id}}, fields: [id]}\n      \
         - {name: b, component: record, reads: {view: t.ById, params: {id: section.a.row.id}}, fields: [id]}\n      \
         - {name: c, component: record, reads: {view: t.ById, params: {id: section.b.row.id}}, fields: [id]}",
    );
    let report = report(&text);
    let found = row_findings(&report);
    assert_eq!(found.len(), 3, "{:#?}", report.findings);
    for finding in &found {
        for name in ["`a`", "`b`", "`c`"] {
            assert!(finding.message.contains(name), "{finding:?}");
        }
    }
    assert!(report.has_errors());
}

#[test]
fn a_row_of_a_section_that_reads_nothing_is_refused_in_visible_and_the_header() {
    let text = page(
        "    header: {title: P, badge: {text: section.note.row.state}}\n",
        "      - {name: note, component: record, fields: [id]}\n      \
         - {name: lines, component: collection, reads: t.Lines, columns: [sku], visible: section.note.row.id}",
    );
    let report = report(&text);
    let paths: Vec<&str> = row_findings(&report)
        .iter()
        .map(|finding| finding.path.as_str())
        .collect();
    assert!(
        paths.iter().any(|path| path.starts_with("pages/p/header")),
        "{:#?}",
        report.findings
    );
    assert!(
        paths.contains(&"pages/p/sections/lines"),
        "{:#?}",
        report.findings
    );
}

/// The document of the renderer cases: a page overlay reads by `section.order.row`. The check
/// admits it.
#[test]
fn a_page_overlay_bound_to_a_sections_row_is_admitted() {
    let text = page(
        "    overlays:\n      invoice: {kind: drawer, component: record, title: Invoice, \
         reads: {view: t.Invoice, params: {id: section.order.row.invoice_id}}, fields: [id]}\n",
        "      - {name: order, component: record, reads: t.ById, fields: [id]}\n      \
         - {name: lines, component: collection, reads: t.Lines, columns: [sku], \
         row_actions: [{name: open, label: Open, opens: invoice}]}",
    );
    let report = report(&text);
    assert!(
        !report.has_errors() && row_findings(&report).is_empty(),
        "{:#?}",
        report.findings
    );
}
