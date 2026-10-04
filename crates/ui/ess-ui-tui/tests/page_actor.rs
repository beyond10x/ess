//! A page's `actor` (beyond10x/ess#284) is a statement `ess ui check --model` holds the page's
//! commands to. The terminal renderer shows the page exactly as it shows it without one.

use std::path::{Path, PathBuf};

use ess_ui_tui::{App, Options};

fn example_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../examples/partner-portal")
}

fn state_dir(test: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("ess-ui-tui-page-actor")
        .join(test);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("a stale state dir is removed");
    }
    dir
}

fn screen(text: &str, scratch: &str) -> String {
    let mut app = App::from_text(text, &example_dir(), Options::new(state_dir(scratch)))
        .unwrap_or_else(|error| panic!("{error}"));
    app.open_page("partners.detail", &[("id", "pt-003")]);
    app.render_text(120, 40)
}

#[test]
fn a_page_actor_changes_nothing_the_terminal_shows() {
    let text = std::fs::read_to_string(example_dir().join("ui.yaml")).expect("the example reads");
    let detail = "  partners.detail:\n    kind: detail_page\n";
    assert!(
        text.contains(detail),
        "the example still has partners.detail"
    );
    let with_actor = text.replace(
        detail,
        &format!("{detail}    actor: partners.PartnerManager\n"),
    );
    let without = screen(&text, "without");
    assert!(without.contains("Cedar Partners"), "{without}");
    assert_eq!(screen(&with_actor, "with"), without);
}
