//! `section.<name>.row` in the terminal: the first row of a section that reads, beside
//! `section.<name>.selection`. A read parameter and a navigate parameter bound to it resolve on a
//! section that is not the one the header's title reads its record from, to the values the React
//! project resolves for the same document (`ess-ui-react`, `tests/section_row.rs`).

use std::path::{Path, PathBuf};

use ess_ui_tui::{App, Options};

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
