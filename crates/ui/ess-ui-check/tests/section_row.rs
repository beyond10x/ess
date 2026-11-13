//! `section.<name>.row` is a listed binding: the first row of a section that reads. `ess ui check`
//! admits it beside `section.<name>.selection` and holds it to what the language says of it: the
//! section reads, with `--model` its rows carry the field, and no read waits on its own row.

use std::path::{Path, PathBuf};

use ess_ui_check::{check_source, model_from_sources, Finding, Model, Options, Report};

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn page(sections: &str) -> String {
    format!(
        "format: ess-ui/1\napp: t\nmodel: shop\nplacement_profile: fat\n\
         shells: {{app: {{regions: {{main: {{kind: page_outlet}}}}}}}}\n\
         navigation: {{home: p, sections: [{{name: all, pages: [p, q]}}]}}\n\
         pages:\n  p:\n    kind: detail_page\n    title: P\n    sections:\n{sections}\n  \
         q:\n    kind: detail_page\n    title: Q\n    params: {{id: string}}\n    sections:\n      \
         - {{name: only, component: record, reads: t.ById, fields: [id]}}\n"
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
fn a_row_binding_is_admitted_beside_selection() {
    let text = page(
        "      - {name: order, component: record, reads: t.ById, fields: [id]}\n      \
         - {name: customer, component: record, reads: {view: t.Customer, params: {id: section.order.row.customer_id}}, fields: [email]}\n      \
         - {name: lines, component: collection, reads: t.Lines, columns: [sku], visible: section.order.row, row_actions: [{name: open, label: Open, navigate: {to: q, params: {id: section.order.row.invoice_id}}}]}\n      \
         - {name: picked, component: record, reads: {view: t.Picked, params: {id: section.lines.selection.sku}}, fields: [sku]}",
    );
    let report = report(&text);
    assert!(
        !report.has_errors() && row_findings(&report).is_empty(),
        "{:#?}",
        report.findings
    );
}

#[test]
fn a_row_of_a_section_that_reads_nothing_is_refused() {
    let text = page(
        "      - {name: note, component: record, fields: [id]}\n      \
         - {name: lines, component: collection, reads: t.Lines, columns: [sku], row_actions: [{name: open, label: Open, navigate: {to: q, params: {id: section.note.row.id}}}]}",
    );
    let report = report(&text);
    let found = row_findings(&report);
    assert_eq!(found.len(), 1, "{:#?}", report.findings);
    assert_eq!(found[0].path, "pages/p/sections/lines/row_actions/open");
    assert!(
        found[0].message.contains("`note`") && found[0].message.contains("reads nothing"),
        "{found:#?}"
    );
    assert!(report.has_errors());
}

#[test]
fn a_row_of_a_section_that_reads_nothing_is_refused_in_a_read_param() {
    let text = page(
        "      - {name: note, component: record, fields: [id]}\n      \
         - {name: customer, component: record, reads: {view: t.Customer, params: {id: section.note.row.id}}, fields: [email]}",
    );
    let report = report(&text);
    let found = row_findings(&report);
    assert_eq!(found.len(), 1, "{:#?}", report.findings);
    assert_eq!(found[0].path, "pages/p/sections/customer");
}

#[test]
fn a_read_whose_params_read_its_own_row_is_refused() {
    let text = page(
        "      - {name: order, component: record, reads: {view: t.ById, params: {id: section.order.row.id}}, fields: [id]}",
    );
    let report = report(&text);
    let found = row_findings(&report);
    assert_eq!(found.len(), 1, "{:#?}", report.findings);
    assert_eq!(found[0].path, "pages/p/sections/order/reads");
    assert!(found[0].message.contains("`order`"), "{found:#?}");
    assert!(report.has_errors());
}

#[test]
fn a_cycle_of_sections_reading_each_others_row_is_refused_naming_both() {
    let text = page(
        "      - {name: left, component: record, reads: {view: t.ById, params: {id: section.right.row.id}}, fields: [id]}\n      \
         - {name: right, component: record, reads: {view: t.ById, params: {id: section.left.row.id}}, fields: [id]}\n      \
         - {name: free, component: record, reads: {view: t.ById, params: {id: section.left.row.id}}, fields: [id]}",
    );
    let report = report(&text);
    let found = row_findings(&report);
    let paths: Vec<&str> = found.iter().map(|finding| finding.path.as_str()).collect();
    assert_eq!(
        paths,
        [
            "pages/p/sections/left/reads",
            "pages/p/sections/right/reads"
        ],
        "{:#?}",
        report.findings
    );
    for finding in &found {
        assert!(
            finding.message.contains("`left`") && finding.message.contains("`right`"),
            "{finding:?}"
        );
        assert!(!finding.message.contains("`free`"), "{finding:?}");
    }
}

#[test]
fn a_chain_of_rows_that_settles_is_admitted() {
    let text = page(
        "      - {name: a, component: record, reads: t.ById, fields: [id]}\n      \
         - {name: b, component: record, reads: {view: t.ById, params: {id: section.a.row.id}}, fields: [id]}\n      \
         - {name: c, component: record, reads: {view: t.ById, params: {id: section.b.row.id}}, fields: [id]}",
    );
    let report = report(&text);
    assert!(row_findings(&report).is_empty(), "{:#?}", report.findings);
}

fn read(file: &str) -> String {
    std::fs::read_to_string(crate_dir().join("tests/fixtures/model").join(file))
        .unwrap_or_else(|error| panic!("{file}: {error}"))
}

fn model() -> Model {
    let sources = [
        ("system.yaml", read("system.yaml")),
        ("components.yaml", read("components.yaml")),
        ("domains/stock.yaml", read("domains/stock.yaml")),
        ("domains/audit.yaml", read("domains/audit.yaml")),
    ]
    .map(|(label, text)| (label.to_owned(), text));
    model_from_sources(&sources, Path::new("shop")).unwrap_or_else(|error| panic!("{error}"))
}

fn with_field(field: &str) -> String {
    page(&format!(
        "      - {{name: items, component: record, reads: stock.Items, fields: [label]}}\n      \
         - {{name: other, component: record, reads: stock.Items, fields: [label], visible: \"section.items.row.{field} == 'x'\"}}"
    ))
}

fn row_field_findings(report: &Report) -> Vec<&Finding> {
    report
        .findings
        .iter()
        .filter(|finding| finding.check == "row_fields")
        .collect()
}

#[test]
fn with_a_model_a_row_field_the_view_lacks_is_refused() {
    let model = model();
    let report = check_source(
        &with_field("labell"),
        "test.yaml",
        &crate_dir(),
        Some(&model),
        &Options::default(),
    );
    let found = row_field_findings(&report);
    assert_eq!(found.len(), 1, "{:#?}", report.findings);
    assert_eq!(found[0].path, "pages/p/sections/other");
    assert!(
        found[0].message.contains("`labell`") && found[0].message.contains("stock.Items"),
        "{found:#?}"
    );
}

#[test]
fn with_a_model_a_row_field_the_view_carries_is_admitted() {
    let model = model();
    let report = check_source(
        &with_field("label"),
        "test.yaml",
        &crate_dir(),
        Some(&model),
        &Options::default(),
    );
    assert!(
        row_field_findings(&report).is_empty() && row_findings(&report).is_empty(),
        "{:#?}",
        report.findings
    );
}
