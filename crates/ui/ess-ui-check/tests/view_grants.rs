//! `section_readable` learns view grants (beyond10x/ess#286).
//!
//! From source `ess/22` an actor's `may:` may name a view, and the actors naming it may read it.
//! A section reading such a view is readable, whether or not any actor may invoke a command of the
//! context that owns it; a view no grant names keeps the approximation it had.

use std::path::{Path, PathBuf};

use ess_ui_check::{check_source, model_from_sources, Model, Options, Report};

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn fixture(file: &str) -> String {
    std::fs::read_to_string(crate_dir().join("tests/fixtures/model").join(file))
        .unwrap_or_else(|error| panic!("{file}: {error}"))
}

/// The fixture model in source `ess/22`, with `actors` appended to the audit domain. Nobody may
/// invoke a command of `shop.audit`.
fn model(actors: &str) -> Model {
    // From ess/4 an emitted payload names every field's source: the minted identities are generated.
    let mut audit = fixture("domains/audit.yaml").replace(
        "            note: input.note\n",
        "            note: input.note\n            entry_id: {generated: true}\n",
    );
    if !actors.is_empty() {
        audit.push_str("\nactors:\n");
        audit.push_str(actors);
    }
    let sources: Vec<(String, String)> = [
        (
            "system.yaml",
            fixture("system.yaml").replace("format: ess/1", "format: ess/22"),
        ),
        ("components.yaml", fixture("components.yaml")),
        (
            "domains/stock.yaml",
            fixture("domains/stock.yaml").replace(
                "            label: input.label\n",
                "            label: input.label\n            item_id: {generated: true}\n",
            ),
        ),
        ("domains/audit.yaml", audit),
    ]
    .into_iter()
    .map(|(label, text)| (label.to_owned(), text))
    .collect();
    model_from_sources(&sources, Path::new("shop")).unwrap_or_else(|error| panic!("{error}"))
}

const DOCUMENT: &str = "format: ess-ui/1\napp: t\nmodel: shop\nplacement_profile: fat\n\
     shells: {app: {regions: {main: {kind: page_outlet}}}}\n\
     navigation: {home: p, sections: [{name: all, pages: [p]}]}\n\
     pages: {p: {kind: detail_page, title: P, sections: [{name: audit, component: collection, \
     reads: audit.Entries}]}}\n";

fn unreadable(report: &Report) -> usize {
    report
        .findings
        .iter()
        .filter(|finding| finding.check == "section_readable")
        .count()
}

fn check(model: &Model) -> Report {
    check_source(
        DOCUMENT,
        "ui.yaml",
        &crate_dir(),
        Some(model),
        &Options::default(),
    )
}

#[test]
fn a_section_reading_a_view_an_actor_is_granted_is_readable() {
    let granted = model("  - name: shop.audit.Reader\n    may:\n      - shop.audit.Entries\n");
    let report = check(&granted);
    assert_eq!(unreadable(&report), 0, "{:#?}", report.findings);
}

#[test]
fn a_section_reading_a_view_no_grant_names_keeps_the_approximation() {
    let report = check(&model(""));
    assert_eq!(unreadable(&report), 1, "{:#?}", report.findings);
}
