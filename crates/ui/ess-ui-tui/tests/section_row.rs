//! `section.<name>.row` in the terminal: the first row of a section that reads, beside
//! `section.<name>.selection`. A read parameter and a navigate parameter bound to it resolve on a
//! section that is not the one the header's title reads its record from, to the values the React
//! project resolves for the same document (`ess-ui-react`, `tests/section_row.rs`).

use std::path::{Path, PathBuf};
use std::time::Duration;

use ess_ui_tui::live::Script;
use ess_ui_tui::{App, FixtureAdapter, Lifecycle, Options};

const DOCUMENT: &str = r"
format: ess-ui/1
app: shop
model: shop.system
placement_profile: fat
fixtures:
  views:
    orders.ById: orders.yaml
    customers.ById: customers.yaml
    lines.All: lines.yaml
    invoices.ById: invoices.yaml
shells:
  app: {regions: {main: {kind: page_outlet}}}
navigation:
  home: order
  sections: [{name: all, pages: [order, invoice]}]
pages:
  order:
    kind: detail_page
    title: Order
    header:
      title: Order
      title_from: {section: customer, field: name}
    sections:
      - {name: order, component: record, reads: {view: orders.ById, key: id}, fields: [id]}
      - {name: customer, component: record, reads: {view: customers.ById, key: id, params: {id: section.order.row.customer_id}}, fields: [email]}
      - name: lines
        component: collection
        reads: {view: lines.All, key: sku}
        columns: [sku]
        row_actions:
          - {name: open_invoice, label: Open invoice, navigate: {to: invoice, params: {invoice_id: section.order.row.invoice_id}}}
  invoice:
    kind: detail_page
    title: Invoice
    params: {invoice_id: string}
    sections:
      - {name: summary, component: record, reads: {view: invoices.ById, key: id, params: {id: params.invoice_id}}, fields: [id]}
";

const FIXTURES: &[(&str, &str)] = &[
    (
        "orders.yaml",
        "views:\n  orders.ById:\n    rows:\n      - {id: o-1, customer_id: c-9, invoice_id: i-3}\n",
    ),
    (
        "customers.yaml",
        "views:\n  customers.ById:\n    rows:\n      - {id: c-1, name: Ada, email: ada@shop.test}\n      \
         - {id: c-9, name: Grace, email: grace@shop.test}\n",
    ),
    (
        "lines.yaml",
        "views:\n  lines.All:\n    rows:\n      - {sku: bolt}\n",
    ),
    (
        "invoices.yaml",
        "views:\n  invoices.ById:\n    rows:\n      - {id: i-3}\n",
    ),
];

fn open(name: &str, text: &str) -> App {
    let dir: PathBuf = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("ess-ui-tui-section-row")
        .join(name);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("a stale dir is removed");
    }
    std::fs::create_dir_all(&dir).expect("the dir is created");
    for (name, body) in FIXTURES {
        std::fs::write(dir.join(name), body).expect("the fixture is written");
    }
    let mut app = App::from_text(text, &dir, Options::new(dir.join("state")))
        .unwrap_or_else(|error| panic!("{error}"));
    app.open_page("order", &[]);
    app
}

fn ids(app: &App, section: &str) -> Vec<String> {
    app.rows(section)
        .iter()
        .map(|row| row["id"].as_str().unwrap_or_default().to_owned())
        .collect()
}

#[test]
fn a_read_param_bound_to_a_sections_row_resolves_on_a_section_the_title_does_not_read() {
    let app = open("read-param", DOCUMENT);
    assert_eq!(ids(&app, "customer"), ["c-9"]);
    let screen = app.render_text(120, 40);
    assert!(screen.contains("grace@shop.test"), "{screen}");
    assert!(!screen.contains("ada@shop.test"), "{screen}");
}

#[test]
fn a_navigate_param_bound_to_a_sections_row_resolves_on_a_section_the_title_does_not_read() {
    let mut app = open("navigate-param", DOCUMENT);
    app.focus_section("lines");
    app.keys("<enter>");
    assert_eq!(app.page(), "invoice");
    assert_eq!(app.location(), "/invoice?invoice_id=i-3");
}

#[test]
fn the_header_title_still_reads_its_sections_record() {
    let app = open("title", DOCUMENT);
    let screen = app.render_text(120, 40);
    let title = screen
        .lines()
        .find(|line| line.contains("Grace"))
        .unwrap_or_else(|| panic!("the title is the record field:\n{screen}"));
    assert!(!title.contains("Order"), "{screen}");
}

/// A section that is `load: on_demand` and has not been loaded holds no row, though another
/// section already cached the same read: `customer` reads with no id (every customer) until
/// `order` is loaded, as the React project's section, whose read waits for its activation.
#[test]
fn an_on_demand_section_that_is_not_loaded_holds_no_row() {
    let text = DOCUMENT
        .replace(
            "      - {name: order, component: record, reads: {view: orders.ById, key: id}, fields: [id]}\n",
            "      - {name: peer, component: record, reads: {view: orders.ById, key: id}, fields: [id]}\n      \
             - {name: order, component: record, load: on_demand, reads: {view: orders.ById, key: id}, fields: [id]}\n",
        );
    let mut app = open("on-demand", &text);
    assert_eq!(ids(&app, "peer"), ["o-1"]);
    assert_eq!(ids(&app, "customer"), ["c-1", "c-9"]);
    app.focus_section("order");
    app.keys("R");
    assert_eq!(ids(&app, "customer"), ["c-9"]);
}

/// A read started again with the same params (a live refetch; a poll or a command do the same)
/// does not blank the row: the rows it held stay while it runs, as the React project's section
/// does (`refreshing`). The reference says the row is absent while the section loads, and a
/// section that has answered for its current params is not loading them.
#[test]
fn a_row_is_held_while_its_read_runs_again_for_the_same_params() {
    let text = DOCUMENT
        .replace(
            "pages:\n",
            "channels:\n  feed:\n    carries: {events: [orders.Changed]}\n    direction: server_to_client\n    \
             delivery: every_event\n    resume: refetch\npages:\n",
        )
        .replace(
            "reads: {view: orders.ById, key: id}, fields: [id]}",
            "reads: {view: orders.ById, key: id}, fields: [id], live: {channel: feed, effect: refetch}}",
        );
    let dir: PathBuf = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("ess-ui-tui-section-row")
        .join("same-params");
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("a stale dir is removed");
    }
    std::fs::create_dir_all(&dir).expect("the dir is created");
    for (name, body) in FIXTURES {
        std::fs::write(dir.join(name), body).expect("the fixture is written");
    }
    let document = ess_ui::load_str(&text).unwrap_or_else(|error| panic!("{error}"));
    let (adapter, _) = FixtureAdapter::load(&document, &dir, None).expect("the fixtures load");
    let script = Script::parse(
        "feed",
        &ess_yaml::from_str(
            "{channel: feed, events: [{at: 2s, event: orders.Changed, payload: {id: o-1}}]}",
        )
        .expect("the script is YAML"),
    )
    .expect("the script parses");
    let mut options = Options::new(dir.join("state"));
    options
        .view_latency
        .insert("orders.ById".to_owned(), Duration::from_secs(1));
    let mut app = App::with_adapter(document, Box::new(adapter), vec![script], options)
        .unwrap_or_else(|error| panic!("{error}"));
    app.open_page("order", &[]);
    app.advance(Duration::from_secs(1));
    assert_eq!(ids(&app, "customer"), ["c-9"]);
    app.advance(Duration::from_secs(1));
    assert_eq!(app.section_state("order"), Lifecycle::Loading);
    assert_eq!(ids(&app, "customer"), ["c-9"]);
    app.advance(Duration::from_secs(1));
    assert_eq!(app.section_state("order"), Lifecycle::Ready);
    assert_eq!(ids(&app, "customer"), ["c-9"]);
}

/// `pick` holds the order `o-2` under its cursor and reads again for the same request (`R`): the
/// selection `order` reads by is the held row while it runs, `customer` keeps reading `c-2`, and
/// the cursor is on the same row when the answer arrives.
#[test]
fn a_selection_is_held_while_its_collection_reads_again_and_the_cursor_stays() {
    let text = "format: ess-ui/1\napp: shop\nmodel: shop.system\nplacement_profile: fat\n\
        fixtures:\n  views:\n    orders.All: orders.yaml\n    orders.ById: orders.yaml\n    \
        customers.ById: customers.yaml\n\
        shells:\n  app: {regions: {main: {kind: page_outlet}}}\n\
        navigation:\n  home: order\n  sections: [{name: all, pages: [order]}]\n\
        pages:\n  order:\n    kind: detail_page\n    title: Order\n    sections:\n      \
        - {name: pick, component: collection, selection: single, reads: {view: orders.All, key: id}, columns: [id]}\n      \
        - {name: order, component: record, reads: {view: orders.ById, key: id, params: {id: section.pick.selection.id}}, fields: [id]}\n      \
        - {name: customer, component: record, reads: {view: customers.ById, key: id, params: {id: section.order.row.customer_id}}, fields: [email]}\n";
    let dir: PathBuf = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("ess-ui-tui-section-row")
        .join("selection-held");
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("a stale dir is removed");
    }
    std::fs::create_dir_all(&dir).expect("the dir is created");
    for (name, body) in [
        (
            "orders.yaml",
            "views:\n  orders.All:\n    rows:\n      - {id: o-1}\n      - {id: o-2}\n  \
             orders.ById:\n    rows:\n      - {id: o-1, customer_id: c-1}\n      \
             - {id: o-2, customer_id: c-2}\n",
        ),
        (
            "customers.yaml",
            "views:\n  customers.ById:\n    rows:\n      - {id: c-1, email: ada@shop.test}\n      \
             - {id: c-2, email: grace@shop.test}\n",
        ),
    ] {
        std::fs::write(dir.join(name), body).expect("the fixture is written");
    }
    let mut options = Options::new(dir.join("state"));
    options
        .view_latency
        .insert("orders.All".to_owned(), Duration::from_secs(1));
    let mut app = App::from_text(text, &dir, options).unwrap_or_else(|error| panic!("{error}"));
    app.open_page("order", &[]);
    app.advance(Duration::from_secs(1));
    app.focus_section("pick");
    app.keys("j");
    assert_eq!(ids(&app, "order"), ["o-2"]);
    app.keys("R");
    assert_eq!(app.section_state("pick"), Lifecycle::Loading);
    assert_eq!(ids(&app, "order"), ["o-2"]);
    assert_eq!(ids(&app, "customer"), ["c-2"]);
    app.advance(Duration::from_secs(1));
    assert_eq!(app.section_state("pick"), Lifecycle::Ready);
    assert_eq!(ids(&app, "order"), ["o-2"]);
    assert_eq!(ids(&app, "customer"), ["c-2"]);
}
