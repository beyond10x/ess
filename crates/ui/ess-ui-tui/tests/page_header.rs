//! The terminal draws a header's badge after the title, its live status as `● <lifecycle>`, and
//! marks the current switch entry
//! (story:a-page-shows-its-title-total-badge-and-live-status-and-marks-the-current-page).

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
channels:
  orders:
    carries: {events: [shop.OrderPlaced]}
    direction: server_to_client
    delivery: every_event
    resume: refetch
pages:
  orders.list:
    kind: static_page
    title: Orders
    header:
      title: Orders
      badge: {text: Beta, tone: info}
      live: [orders]
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
        .join("ess-ui-tui-page-header")
        .join(test);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("a stale state dir is removed");
    }
    dir
}

fn screen() -> String {
    let mut app = App::from_text(DOCUMENT, Path::new("."), Options::new(state_dir("screen")))
        .unwrap_or_else(|error| panic!("{error}"));
    app.open_page("orders.list", &[]);
    app.render_text(120, 20)
}

#[test]
fn the_badge_follows_the_title() {
    let screen = screen();
    let line = screen
        .lines()
        .find(|line| line.contains("[Beta]"))
        .unwrap_or_else(|| panic!("no badge:\n{screen}"));
    assert!(
        line.find("Orders").expect("the title is drawn") < line.find("[Beta]").unwrap(),
        "the badge is before the title:\n{screen}"
    );
}

#[test]
fn the_live_status_reads_as_a_dot_and_its_lifecycle() {
    let screen = screen();
    assert!(
        screen
            .lines()
            .any(|line| line.contains("[Beta]") && line.contains("● connecting")),
        "the header has no `● <lifecycle>`:\n{screen}"
    );
}

#[test]
fn the_current_switch_entry_is_marked() {
    let screen = screen();
    assert!(
        screen.contains("▸ orders.list") && screen.contains("orders.cards"),
        "the switch does not mark the current page:\n{screen}"
    );
    assert!(
        !screen.contains("▸ orders.cards"),
        "the sibling is marked:\n{screen}"
    );
}
