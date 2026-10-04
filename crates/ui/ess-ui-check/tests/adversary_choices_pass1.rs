//! Adversary pass 1 for beyond10x/ess#328 and #330: the unit's own schema notes, driven against
//! the code the unit wrote.

use std::path::{Path, PathBuf};

use ess_ui_check::{check_source, model_from_sources, Finding, Model, Options, Report};

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(file: &str) -> String {
    std::fs::read_to_string(crate_dir().join("tests/fixtures/model").join(file))
        .unwrap_or_else(|error| panic!("{file}: {error}"))
}

fn model(stock: String, audit: String) -> Model {
    let sources = [
        ("system.yaml", read("system.yaml")),
        ("components.yaml", read("components.yaml")),
        ("domains/stock.yaml", stock),
        ("domains/audit.yaml", audit),
    ]
    .map(|(label, text)| (label.to_owned(), text));
    model_from_sources(&sources, Path::new("shop")).unwrap_or_else(|error| panic!("{error}"))
}

/// A document of model `shop` whose page reads `stock.Items` and holds `sections` besides.
fn document(sections: &str) -> String {
    format!(
        "format: ess-ui/1\napp: t\nmodel: shop\nplacement_profile: fat\n\
         shells: {{app: {{regions: {{main: {{kind: page_outlet}}}}}}}}\n\
         navigation: {{home: p, sections: [{{name: all, pages: [p]}}]}}\n\
         pages: {{p: {{kind: detail_page, title: P, sections: [{{name: summary, reads: stock.Items}}, {sections}]}}}}\n"
    )
}

fn findings<'a>(report: &'a Report, id: &str) -> Vec<&'a Finding> {
    report
        .findings
        .iter()
        .filter(|finding| finding.check == id)
        .collect()
}

/// `schemas/ui/ess-ui.schema.yaml` (choice `options` note, written by this unit): a name below
/// the system resolves "by ... the last segments only one enum ends with". `Shelf` is the last
/// segment of exactly one enum, `shop.stock.Shelf`; `shop.audit.Shelf` is a newtype, not an enum.
#[test]
fn a_last_segment_only_one_enum_ends_with_resolves_to_that_enum() {
    let stock = read("domains/stock.yaml").replace(
        "    of: Uuid\n",
        "    of: Uuid\n  - name: shop.stock.Shelf\n    kind: enum\n    variants: [Top, Bottom]\n",
    );
    let audit = read("domains/audit.yaml").replace(
        "    of: Uuid\n",
        "    of: Uuid\n  - name: shop.audit.Shelf\n    kind: newtype\n    of: String\n",
    );
    let model = model(stock, audit);
    let text = document("{name: pick, component: choice, options: Shelf}");
    let loaded = ess_ui::load_str_with(&text, &model).unwrap_or_else(|error| {
        panic!("only one enum ends with `Shelf`, as the schema note says resolves: {error}")
    });
    assert_eq!(loaded.model_enums["Shelf"], "shop.stock.Shelf");
}

/// The unit's binding carries a view's identity by its wire name (`identity_of`), because the rows
/// a served view answers carry wire names. A choice's `value` naming the field by its model name,
/// when the view renames it on the wire, names a field no row has: every row then "offers no
/// option" (`Choice::row_option`), and the schema note says `value` is "the row field of `reads`".
/// `ess ui check --model` holds `value` to the view's row fields under `row_fields`, so it must
/// say so; it admits the model name instead.
#[test]
fn a_choice_value_naming_a_field_the_rows_carry_under_another_wire_name_is_flagged() {
    let stock = read("domains/stock.yaml").replace(
        "      - name: label\n        type: String\n    naming:\n      wire: items\n",
        "      - name: label\n        type: String\n        wire: title\n    \
         naming:\n      wire: items\n",
    );
    assert!(stock.contains("wire: title"), "the view field is renamed");
    let model = model(stock, read("domains/audit.yaml"));
    let text = document("{name: pick, component: choice, reads: stock.Items, value: label}");
    let report = check_source(
        &text,
        "test.yaml",
        &crate_dir(),
        Some(&model),
        &Options::default(),
    );
    assert!(
        findings(&report, "row_fields")
            .iter()
            .any(|finding| finding.path == "pages/p/sections/pick/value"),
        "`label` travels as `title`, so no row carries `label` and the choice offers nothing: \
         {:#?}",
        report.findings
    );
}
