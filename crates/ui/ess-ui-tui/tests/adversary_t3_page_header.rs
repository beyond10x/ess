//! Adversary, turn 3 (story:a-page-shows-its-title-total-badge-and-live-status-and-marks-the-current-page):
//! the schema says a header's `switch` holds "pages from `switch_to`", so a page names the same
//! sibling in both. React offers it once (`emit.rs` skips a `switch_to` page already in the
//! switch); the terminal chains the two lists and offers it twice.

use std::path::{Path, PathBuf};

use ess_ui_tui::{App, Options};

const DOCUMENT: &str = r"
format: ess-ui/1
app: shop
model: shop.system
placement_profile: fat
shells:
  app: {regions: {main: {kind: page_outlet}}}
navigation:
  home: orders.list
  sections: [{name: all, pages: [orders.list, orders.cards]}]
pages:
  orders.list:
    kind: static_page
    title: Orders
    switch_to: [orders.cards]
    header:
      title: Orders
      switch: [orders.cards]
    sections:
      - {name: note, component: collection, reads: {view: shop.Orders}, columns: [id]}
  orders.cards:
    kind: static_page
    title: Orders as cards
    sections:
      - {name: note, component: collection, reads: {view: shop.Orders}, columns: [id]}
";

fn state_dir(test: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("ess-ui-tui-adversary-t3-page-header")
        .join(test);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("a stale state dir is removed");
    }
    dir
}

#[test]
#[ignore = "pre-existing: the terminal chains switch and switch_to without removing duplicates; backlog"]
fn a_page_in_both_switch_and_switch_to_is_offered_once() {
    let mut app = App::from_text(DOCUMENT, Path::new("."), Options::new(state_dir("once")))
        .unwrap_or_else(|error| panic!("{error}"));
    app.open_page("orders.list", &[]);
    let screen = app.render_text(160, 20);
    let line = screen
        .lines()
        .find(|line| line.contains('⇄'))
        .unwrap_or_else(|| panic!("no switch:\n{screen}"));
    let switch = &line[line.find('⇄').expect("found above")..];
    assert_eq!(
        switch.matches("orders.cards").count(),
        1,
        "the sibling is offered more than once (React offers it once):\n{screen}"
    );
    assert_eq!(switch.matches("▸ orders.list").count(), 1, "{screen}");
}
