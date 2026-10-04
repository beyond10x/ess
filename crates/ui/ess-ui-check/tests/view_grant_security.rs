//! Security review of beyond10x/ess#286: ui-check describes the read grants the served surface
//! enforces.
//!
//! The fixture's `shop-service` is `reached_by: network`, so its generated server answers
//! `shop.audit.Entries`, read-granted to `shop.audit.Reader` only, with the standard `403` to a
//! request authenticated as no actor and to any other actor (`docs/design/view-grants.md`). A UI
//! document that reads it where the server will refuse it should not check clean.

use std::path::{Path, PathBuf};

use ess_ui_check::{check_source, model_from_sources, Finding, Model, Options, Report};

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn fixture(file: &str) -> String {
    std::fs::read_to_string(crate_dir().join("tests/fixtures/model").join(file))
        .unwrap_or_else(|error| panic!("{file}: {error}"))
}

/// The fixture model in `ess/22`: `shop.audit.Reader` may read `shop.audit.Entries`, and
/// `shop.audit.Recorder` may record an entry and read nothing granted.
fn model() -> Model {
    let mut audit = fixture("domains/audit.yaml").replace(
        "            note: input.note\n",
        "            note: input.note\n            entry_id: {generated: true}\n",
    );
    audit.push_str(
        "\nactors:\n  - name: shop.audit.Reader\n    may:\n      - shop.audit.Entries\n  - name: \
         shop.audit.Recorder\n    may:\n      - shop.audit.RecordEntry\n",
    );
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

/// One page `p` reading `audit.Entries`, the page carrying `page_actor` when given, and `extra`
/// top-level keys.
fn document(page_actor: Option<&str>, extra: &str) -> String {
    let actor = page_actor.map_or(String::new(), |actor| format!("actor: {actor}, "));
    format!(
        "format: ess-ui/1\napp: t\nmodel: shop\nplacement_profile: fat\n{extra}\
         shells: {{app: {{regions: {{main: {{kind: page_outlet}}}}}}}}\n\
         navigation: {{home: p, sections: [{{name: all, pages: [p]}}]}}\n\
         pages: {{p: {{kind: detail_page, title: P, {actor}sections: [{{name: audit, component: \
         collection, reads: audit.Entries}}]}}}}\n"
    )
}

fn check(text: &str) -> Report {
    check_source(
        text,
        "ui.yaml",
        &crate_dir(),
        Some(&model()),
        &Options::default(),
    )
}

/// The findings that name the read-granted view.
fn about_entries(report: &Report) -> Vec<&Finding> {
    report
        .findings
        .iter()
        .filter(|finding| finding.message.contains("shop.audit.Entries"))
        .collect()
}

/// An `actor: anonymous` document is read by nobody signed in. The server refuses every request
/// authenticated as no actor a read of `shop.audit.Entries`, so a section reading it renders a
/// refusal for every visitor.
#[test]
fn an_anonymous_document_reading_a_read_granted_view_is_reported() {
    let signed_in = check(&document(None, ""));
    assert_eq!(
        about_entries(&signed_in).len(),
        0,
        "control: a signed-in document may be read by the Reader: {:#?}",
        signed_in.findings
    );
    let anonymous = check(&document(None, "actor: anonymous\n"));
    assert_ne!(
        about_entries(&anonymous).len(),
        0,
        "an anonymous document reads `shop.audit.Entries`, which the server refuses to every \
         request authenticated as no actor, and ui-check reports nothing: {:#?}",
        anonymous.findings
    );
}

/// A page bound to `shop.audit.Recorder` reads `shop.audit.Entries`, which the Recorder's grant
/// does not name: the server answers that page's read `403` naming the Recorder. ui-check already
/// reports a page actor sending a command its grant does not name (`page_actor_grants`); the read
/// side is the same question.
#[test]
fn a_page_whose_actor_may_not_read_a_read_granted_view_is_reported() {
    let reader = check(&document(Some("shop.audit.Reader"), ""));
    assert_eq!(
        about_entries(&reader).len(),
        0,
        "control: the Reader's grant names the view: {:#?}",
        reader.findings
    );
    let recorder = check(&document(Some("shop.audit.Recorder"), ""));
    assert_ne!(
        about_entries(&recorder).len(),
        0,
        "page `p`'s actor `shop.audit.Recorder` may not read `shop.audit.Entries`, which the \
         server refuses it, and ui-check reports nothing: {:#?}",
        recorder.findings
    );
}
