//! A page shows its title, a counted total, a badge and its live status, and marks the current page
//! (story:a-page-shows-its-title-total-badge-and-live-status-and-marks-the-current-page,
//! beyond10x/ess#723).

use std::path::Path;

const DOCUMENT: &str = "format: ess-ui/1
app: shop
model: shop.system
placement_profile: fat
shells:
  app: {regions: {navigation: {kind: navigation}, main: {kind: page_outlet}}}
navigation:
  home: orders.list
  sections: [{name: all, pages: [orders.list, orders.cards, plain, bare]}]
channels:
  orders:
    carries: {events: [shop.OrderPlaced]}
    direction: server_to_client
    delivery: every_event
    resume: refetch
pages:
  orders.list:
    kind: list_page
    title: Orders
    header:
      title: Orders
      total: list
      badge: {text: Beta, tone: info}
      live: [orders]
      switch: [orders.cards]
    sections:
      - {name: list, component: collection, title: Open orders, reads: {view: shop.Orders}, columns: [id]}
  orders.cards:
    kind: list_page
    title: Orders as cards
    switch_to: [orders.list]
    header: {title: Orders as cards}
    sections:
      - {name: cards, component: collection, reads: {view: shop.Orders}, columns: [id]}
  plain:
    kind: list_page
    title: Plain
    header: {title: Plain, total: rows}
    sections:
      - {name: rows, component: collection, reads: {view: shop.Orders}, columns: [id]}
  bare:
    kind: static_page
    title: Bare page
    sections:
      - {name: list, component: collection, reads: {view: shop.Orders}, columns: [id]}
";

fn files() -> ess_ui_react::GeneratedFiles {
    let document =
        ess_ui::load_str(DOCUMENT).unwrap_or_else(|error| panic!("the document loads: {error}"));
    ess_ui_react::render(&document, Path::new("."))
        .unwrap_or_else(|error| panic!("the project renders: {error}"))
}

/// The body of `export function <name>` in the runtime core.
fn component(name: &str) -> String {
    let files = files();
    let core = &files.files["src/runtime/core.tsx"];
    let start = core
        .find(&format!("export function {name}"))
        .unwrap_or_else(|| panic!("the runtime defines {name}"));
    let body = &core[start..];
    body[..body.find("\n}\n").unwrap_or(body.len())].to_owned()
}

#[test]
fn a_total_reads_as_a_count() {
    let header = component("HeaderFrame");
    assert!(
        header.contains("ui-count") && !header.contains("className=\"ui-total\""),
        "the total is not a count mark:\n{header}"
    );
    assert!(
        header.contains("Intl.NumberFormat") || header.contains("toLocaleString"),
        "the total is not grouped by the locale:\n{header}"
    );
    assert!(
        header.contains("role=\"img\"")
            && header.contains("aria-label")
            && header.contains(".title")
            && header.contains("\"rows\""),
        "the count has no accessible name `<total> <section title, else rows>`:\n{header}"
    );
    let files = files();
    let frame = {
        let core = &files.files["src/runtime/core.tsx"];
        let start = core
            .find("export function SectionFrame")
            .expect("SectionFrame");
        core[start..].to_owned()
    };
    assert!(
        frame.contains("title: props.title"),
        "the section does not report its title to the header:\n{frame}"
    );
    for (name, page) in &files.files {
        assert!(
            !name.starts_with("src/pages/") || !page.contains("totalLabel"),
            "{name}: a page file carries the section title the runtime reads:\n{page}"
        );
    }
}

#[test]
fn live_status_is_a_labelled_mark() {
    let header = component("HeaderFrame");
    for word in ["Live", "Connecting", "Reconnecting", "Stale", "Closed"] {
        assert!(
            header.contains(&format!("\"{word}\"")),
            "the status mark has no word `{word}`:\n{header}"
        );
    }
    assert!(
        header.contains("data-lifecycle") && header.contains("title={channel}"),
        "the status mark keeps `data-lifecycle` and names the channel in its title:\n{header}"
    );
    assert!(
        !header.contains("{channel}: {"),
        "the raw `<channel>: <lifecycle>` text remains:\n{header}"
    );
}

#[test]
fn the_switch_offers_and_marks_the_current_page() {
    let files = files();
    let list = &files.files["src/pages/OrdersList.tsx"];
    let current = list
        .find("current: true")
        .unwrap_or_else(|| panic!("the current entry is not marked:\n{list}"));
    let other = list
        .find("\"orders.cards\"")
        .unwrap_or_else(|| panic!("the sibling is not offered:\n{list}"));
    assert!(current < other, "the declared order is lost:\n{list}");
    assert!(
        list.matches("current: true").count() == 1,
        "exactly one entry is current:\n{list}"
    );
    let cards = &files.files["src/pages/OrdersCards.tsx"];
    assert!(
        cards.contains("current: true") && cards.contains("\"orders.list\""),
        "a `switch_to` page offers itself and its sibling:\n{cards}"
    );
    let header = component("HeaderFrame");
    assert!(
        header.contains("aria-current=\"page\"") && header.contains("ui-segmented"),
        "the switch is not a segmented control marking the current entry:\n{header}"
    );
    let at = header.find("aria-current").expect("marked above");
    let start = header[..at].rfind('<').expect("the mark is an attribute");
    let end = at + header[at..].find('>').expect("the element opens");
    assert!(
        !header[start..end].contains("href="),
        "the current entry navigates:\n{}",
        &header[start..end]
    );
}

#[test]
fn navigation_marks_the_current_page() {
    let files = files();
    let navigation = &files.files["src/runtime/navigation.tsx"];
    assert!(
        navigation.contains("aria-current") && navigation.contains("useLocation"),
        "a navigation link does not mark the current page:\n{navigation}"
    );
}

#[test]
fn a_header_badge_draws_its_text_and_tone() {
    let files = files();
    let page = &files.files["src/pages/OrdersList.tsx"];
    assert!(
        page.contains("badge=") && page.contains("Beta") && page.contains("info"),
        "the header is not given its badge:\n{page}"
    );
    let header = component("HeaderFrame");
    let title = header.find("<h1>").expect("the header draws its title");
    let badge = header
        .find("ui-badge ui-tone-")
        .unwrap_or_else(|| panic!("the header draws no badge:\n{header}"));
    assert!(badge > title, "the badge is not after the title:\n{header}");
    let plain = &files.files["src/pages/OrdersCards.tsx"];
    assert!(
        !plain.contains("badge="),
        "a page without a badge:\n{plain}"
    );
}

#[test]
fn a_page_without_a_header_shows_its_title() {
    let files = files();
    let page = &files.files["src/pages/Bare.tsx"];
    assert!(
        page.contains("title=\"Bare page\"") || page.contains("title={\"Bare page\"}"),
        "the frame is not given the page title:\n{page}"
    );
    let frame = component("PageFrame");
    assert!(
        frame.contains("<h1>{props.title}</h1>") || frame.contains("<h1 "),
        "the page frame draws no heading from its title:\n{frame}"
    );
}
