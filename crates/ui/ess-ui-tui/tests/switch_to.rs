//! `switch_to` moves between sibling views of one record and keeps the page params the target
//! declares (beyond10x/ess#355).

use std::path::{Path, PathBuf};

use ess_ui_tui::{App, Options};

fn example_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../examples/partner-portal")
}

fn state_dir(test: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("ess-ui-tui-switch")
        .join(test);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("a stale state dir is removed");
    }
    dir
}

/// The example with `partners.detail` offering a sibling view of the same partner,
/// `partners.activity`, which declares `id` and nothing else.
fn document() -> String {
    let text = std::fs::read_to_string(example_dir().join("ui.yaml")).expect("the example reads");
    let detail = "  partners.detail:\n    kind: detail_page\n    title: Partner\n";
    assert!(
        text.contains(detail),
        "the example still has partners.detail"
    );
    let sibling = "  partners.activity:\n    kind: detail_page\n    title: Partner activity\n    \
                   params: {id: PartnerId}\n    switch_to: [partners.detail]\n    sections:\n      \
                   - name: summary\n        component: record\n        \
                   reads: {view: partners.ById, params: {id: params.id}}\n        \
                   fields: [name]\n\n";
    text.replace(
        detail,
        &format!("{sibling}{detail}    switch_to: [partners.activity]\n"),
    )
}

#[test]
fn switching_to_a_sibling_page_keeps_the_params_it_declares() {
    let mut app = App::from_text(
        &document(),
        &example_dir(),
        Options::new(state_dir("keeps")),
    )
    .unwrap_or_else(|error| panic!("{error}"));
    app.open_page("partners.detail", &[("id", "pt-003")]);
    assert_eq!(app.location(), "/partners.detail?id=pt-003");
    app.keys(":partner activity<enter>");
    assert_eq!(app.page(), "partners.activity");
    assert_eq!(app.location(), "/partners.activity?id=pt-003");
    let screen = app.render_text(120, 40);
    assert!(
        screen.contains("Cedar Partners"),
        "the sibling reads the same partner:\n{screen}"
    );
    app.keys(":partner<enter>");
    assert_eq!(app.location(), "/partners.detail?id=pt-003");
}
