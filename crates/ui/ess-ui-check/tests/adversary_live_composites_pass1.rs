//! Adversary pass 1 for beyond10x/ess#354: `header.title_from` held to the model like every other
//! row field the document names.
//!
//! The unit's schema note: `title_from` "shows a field of the record a section of the page holds
//! ... in place of `title`, which stays the text shown until the record holds the field". A field
//! the rows never carry therefore leaves the literal title on screen for good, with nothing said.
//! `ess ui check --model` holds every other row-field name a document writes (a `group_by`, an
//! aggregate's `field`, a `label_from`, a choice's `value` and `label`) to the view under
//! `row_fields`; `title_from.field` is not held to it.

use std::path::{Path, PathBuf};

use ess_ui_check::{check_source, model_from_sources, Finding, Model, Options, Report};

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(file: &str) -> String {
    std::fs::read_to_string(crate_dir().join("tests/fixtures/model").join(file))
        .unwrap_or_else(|error| panic!("{file}: {error}"))
}

fn model(stock: String) -> Model {
    let sources = [
        ("system.yaml", read("system.yaml")),
        ("components.yaml", read("components.yaml")),
        ("domains/stock.yaml", stock),
        ("domains/audit.yaml", read("domains/audit.yaml")),
    ]
    .map(|(label, text)| (label.to_owned(), text));
    model_from_sources(&sources, Path::new("shop")).unwrap_or_else(|error| panic!("{error}"))
}

/// A page whose header title reads `field` of the first row of `summary`, which reads
/// `stock.Items`.
fn document(field: &str) -> String {
    format!(
        "format: ess-ui/1\napp: t\nmodel: shop\nplacement_profile: fat\n\
         shells: {{app: {{regions: {{main: {{kind: page_outlet}}}}}}}}\n\
         navigation: {{home: p, sections: [{{name: all, pages: [p]}}]}}\n\
         pages: {{p: {{kind: detail_page, title: P, \
         header: {{title: Item, title_from: {{section: summary, field: {field}}}}}, \
         sections: [{{name: summary, component: record, reads: stock.Items, fields: [label]}}]}}}}\n"
    )
}

fn title_from_findings(report: &Report) -> Vec<&Finding> {
    report
        .findings
        .iter()
        .filter(|finding| {
            finding.check == "row_fields" && finding.path.starts_with("pages/p/header/title_from")
        })
        .collect()
}

#[test]
fn a_title_from_field_the_view_does_not_have_is_flagged() {
    let model = model(read("domains/stock.yaml"));
    let report = check_source(
        &document("labell"),
        "test.yaml",
        &crate_dir(),
        Some(&model),
        &Options::default(),
    );
    assert!(
        report
            .findings
            .iter()
            .all(|finding| finding.check != "document_loads"),
        "{:#?}",
        report.findings
    );
    assert!(
        !title_from_findings(&report).is_empty(),
        "`labell` is no row field of `stock.Items`, so the header shows `Item` for good: {:#?}",
        report.findings
    );
}

#[test]
fn a_title_from_field_renamed_on_the_wire_is_flagged() {
    let stock = read("domains/stock.yaml").replace(
        "      - name: label\n        type: String\n    naming:\n      wire: items\n",
        "      - name: label\n        type: String\n        wire: title\n    \
         naming:\n      wire: items\n",
    );
    assert!(stock.contains("wire: title"), "the view field is renamed");
    let model = model(stock);
    let report = check_source(
        &document("label"),
        "test.yaml",
        &crate_dir(),
        Some(&model),
        &Options::default(),
    );
    assert!(
        !title_from_findings(&report).is_empty(),
        "served rows carry `label` as `title`, so no record ever holds `label` and the header \
         shows `Item` for good: {:#?}",
        report.findings
    );
}
