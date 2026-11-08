//! Adversary, unit w091-u1-host, finding 1 (story:host-covered-rows-are-their-own-count).
//!
//! A field slot is a field that carries `host`: "the host draws the input" (`Field.host`). The
//! loader keeps the field's other keys beside the slot, so a field slot may still write
//! `suggests: {reads: …}` (the values its tags input offers) or `label_from` (the related view
//! it shows labels from). The host component draws that input, so the view it reads there is
//! reached only by a host slot. `names` yields those reads with `host: false`, so `coverage`
//! counts the view as **covered**, and the covered count moves when the slot is removed —
//! against the acceptance "The covered count is the same as for the same document with the slot
//! removed" and design `ui-host-slots` ("never counts it as covered by the specification").

use std::path::PathBuf;

use ess_ui_check::{check_source, Options, Report};

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn shop() -> ess_ui_check::Model {
    ess_ui_check::load_model(&crate_dir().join("tests/fixtures/model"))
        .unwrap_or_else(|error| panic!("{error}"))
}

fn shop_document(p: &str) -> String {
    format!(
        "format: ess-ui/1\napp: t\nmodel: shop\nplacement_profile: fat\n\
         shells: {{app: {{regions: {{main: {{kind: page_outlet}}}}}}}}\n\
         navigation: {{home: p, sections: [{{name: all, pages: [p]}}]}}\n\
         pages:\n  p: {p}\n"
    )
}

fn checked(text: &str, model: &ess_ui_check::Model) -> Report {
    check_source(
        text,
        "test.yaml",
        &crate_dir(),
        Some(model),
        &Options::default(),
    )
}

fn counts(report: &Report) -> (usize, usize, usize) {
    let coverage = report.coverage.unwrap_or_else(|| {
        panic!("a loaded document checked with a model counts coverage: {report:#?}")
    });
    (coverage.covered, coverage.host_covered, coverage.uncovered)
}

#[test]
fn a_field_slots_suggestions_read_is_not_covered() {
    let model = shop();
    // The form sends `stock.AddItem`; its one input is drawn by the host, and that input's
    // suggestions read `audit.Entries`.
    let with = checked(
        &shop_document(
            "{kind: detail_page, title: P, sections: [{name: add, component: form, \
              does: stock.AddItem, fields: [{field: label, as: tags, host: TagPicker, \
              reason: a vendor tag picker, stands_in_for: rich_text, \
              suggests: {reads: audit.Entries, value: note}}]}]}",
        ),
        &model,
    );
    // The same document with the slot removed.
    let without = checked(
        &shop_document(
            "{kind: detail_page, title: P, sections: [{name: add, component: form, \
              does: stock.AddItem}]}",
        ),
        &model,
    );
    let (with_covered, with_host, _) = counts(&with);
    let (without_covered, _, _) = counts(&without);
    assert_eq!(
        with_covered,
        without_covered,
        "the field slot's own `suggests` read made `audit.Entries` covered: with the slot \
         {:?}, without it {:?}; findings {:#?}",
        counts(&with),
        counts(&without),
        with.findings
    );
    assert_eq!(
        with_host,
        1,
        "`audit.Entries` is reached only by the slot: {:?}",
        counts(&with)
    );
}

#[test]
fn a_field_slots_label_from_read_is_not_covered() {
    let model = shop();
    let with = checked(
        &shop_document(
            "{kind: detail_page, title: P, sections: [{name: summary, reads: stock.Items, \
              fields: [{field: label, host: LabelPad, reason: a vendor label, \
              stands_in_for: rich_text, \
              label_from: {view: audit.Entries, field: note, key: entry_id}}]}]}",
        ),
        &model,
    );
    let without = checked(
        &shop_document(
            "{kind: detail_page, title: P, sections: [{name: summary, reads: stock.Items, \
              fields: [item_id]}]}",
        ),
        &model,
    );
    let (with_covered, _, _) = counts(&with);
    let (without_covered, _, _) = counts(&without);
    assert_eq!(
        with_covered,
        without_covered,
        "the field slot's own `label_from` read made `audit.Entries` covered: with the slot \
         {:?}, without it {:?}; findings {:#?}",
        counts(&with),
        counts(&without),
        with.findings
    );
}
