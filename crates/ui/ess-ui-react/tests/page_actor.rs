//! A page's `actor` (beyond10x/ess#284) is a statement `ess ui check --model` holds the page's
//! commands to. It is not rendered: the generated project is byte for byte the one the same
//! document without it gives.

use std::path::{Path, PathBuf};

fn example_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../examples/partner-portal")
}

fn example_text() -> String {
    std::fs::read_to_string(example_dir().join("ui.yaml")).expect("the example reads")
}

#[test]
fn a_page_actor_changes_no_generated_file() {
    let text = example_text();
    let detail = "  partners.detail:\n    kind: detail_page\n";
    assert!(
        text.contains(detail),
        "the example still has partners.detail"
    );
    let with_actor = text.replace(
        detail,
        &format!("{detail}    actor: partners.PartnerManager\n"),
    );
    let without = ess_ui::load_str(&text).unwrap_or_else(|error| panic!("{error}"));
    let with = ess_ui::load_str(&with_actor).unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(
        with.pages["partners.detail"].actor.as_deref(),
        Some("partners.PartnerManager")
    );
    let before = ess_ui_react::render(&without, &example_dir()).expect("the example renders");
    let after = ess_ui_react::render(&with, &example_dir()).expect("the example renders");
    assert_eq!(before, after);
}
