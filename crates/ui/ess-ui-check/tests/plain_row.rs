//! `field: .` names the row itself, in a collection column and a record field: accepted on a
//! read whose rows are plain strings, refused by name on a read whose rows are entities
//! (story:a-column-may-show-a-plain-string-row).

use std::path::PathBuf;

use ess_ui_check::{check_source, load_model, Finding, Model, Options, Report};

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn model() -> Model {
    load_model(&crate_dir().join("tests/fixtures/model")).unwrap_or_else(|error| panic!("{error}"))
}

/// A `shop` document whose page `p` has `sections`.
fn document(sections: &str) -> String {
    format!(
        "format: ess-ui/1
app: t
model: shop
placement_profile: fat
shells: {{app: {{regions: {{main: {{kind: page_outlet}}}}}}}}
navigation: {{home: p, sections: [{{name: all, pages: [p]}}]}}
pages:
  p:
    kind: detail_page
    title: P
    sections: {sections}
"
    )
}

fn report(sections: &str) -> Report {
    check_source(
        &document(sections),
        "test.yaml",
        &crate_dir(),
        Some(&model()),
        &Options::default(),
    )
}

fn field_in_model(report: &Report) -> Vec<&Finding> {
    report
        .findings
        .iter()
        .filter(|finding| finding.check == "field_in_model")
        .collect()
}

#[test]
fn the_row_itself_is_accepted_on_a_read_of_plain_strings() {
    let report = report(
        "[{name: names, component: collection, reads: {placeholder: t.Names, fixture: tests/fixtures/plain-names.yaml}, columns: [{field: ., name: name, label: Name}]}, \
          {name: one, component: record, reads: {placeholder: t.Name, fixture: tests/fixtures/plain-names.yaml}, fields: [{field: ., name: name}]}]",
    );
    assert!(field_in_model(&report).is_empty(), "{:#?}", report.findings);
    assert!(
        report
            .findings
            .iter()
            .all(|finding| finding.severity != ess_ui_check::Severity::Error),
        "{:#?}",
        report.findings
    );
}

#[test]
fn the_row_itself_is_refused_by_name_on_a_read_of_entities() {
    let report = report(
        "[{name: items, component: collection, reads: stock.Items, columns: [label, {field: ., name: whole}]}, \
          {name: one, component: record, reads: stock.Items, fields: [{field: ., name: whole}]}]",
    );
    let found = field_in_model(&report);
    let paths: Vec<&str> = found.iter().map(|finding| finding.path.as_str()).collect();
    assert_eq!(
        paths,
        [
            "pages/p/sections/items/columns/whole",
            "pages/p/sections/one/fields/whole"
        ],
        "{found:#?}"
    );
    for finding in found {
        assert!(
            finding.message.contains("`field: .`")
                && finding.message.contains("shop.stock.Items")
                && finding.message.contains("`label`"),
            "the refusal names `field: .`, the view and its fields: {:?}",
            finding.message
        );
    }
}
