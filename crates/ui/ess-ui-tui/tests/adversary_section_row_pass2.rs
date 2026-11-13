//! Adversary pass 2 on `section.<name>.row`, after the fix for pass 1: the terminal's value for
//! the documents of `ess-ui-react/tests/adversary_section_row_pass2.rs` (the same documents, with
//! fixtures). The reference: "the first row of a section that reads; absent while the section
//! loads, when it is empty, and for a section without a read".

use std::path::{Path, PathBuf};
use std::time::Duration;

use ess_ui_tui::{App, Lifecycle, Options};

const FIXTURES_HEAD: &str = r"
format: ess-ui/1
app: shop
model: shop.system
placement_profile: fat
fixtures:
  views:
    orders.All: orders.yaml
    orders.ById: orders.yaml
    customers.ById: customers.yaml
    lines.All: lines.yaml
    invoices.ById: invoices.yaml
";

/// `pick` chooses the order `order` reads; `customer` reads by `section.order.row.customer_id`.
const SELECTION: &str = r"
shells:
  app: {regions: {main: {kind: page_outlet}}}
navigation:
  home: order
  sections: [{name: all, pages: [order]}]
pages:
  order:
    kind: detail_page
    title: Order
    header:
      title: Order
      title_from: {section: order, field: id}
    sections:
      - {name: pick, component: collection, selection: single, reads: {view: orders.All, key: id}, columns: [id]}
      - {name: order, component: record, reads: {view: orders.ById, key: id, params: {id: section.pick.selection.id}}, fields: [id]}
      - {name: customer, component: record, reads: {view: customers.ById, key: id, params: {id: section.order.row.customer_id}}, fields: [email]}
";

/// The overlay `invoice` is the shell's, not the page's; it reads by
/// `section.order.row.invoice_id`.
const SHELL_OVERLAY: &str = r"
shells:
  app:
    regions: {main: {kind: page_outlet}}
    overlays:
      invoice:
        kind: drawer
        component: record
        title: Invoice
        reads: {view: invoices.ById, key: id, params: {id: section.order.row.invoice_id}}
        fields: [id]
navigation:
  home: order
  sections: [{name: all, pages: [order]}]
pages:
  order:
    kind: detail_page
    title: Order
    sections:
      - {name: order, component: record, reads: {view: orders.ById, key: id}, fields: [id]}
      - name: lines
        component: collection
        reads: {view: lines.All, key: sku}
        columns: [sku]
        row_actions:
          - {name: open_invoice, label: Open invoice, opens: invoice}
";

const FIXTURES: &[(&str, &str)] = &[
    (
        "orders.yaml",
        "views:\n  orders.All:\n    rows:\n      - {id: o-1}\n      - {id: o-2}\n  \
         orders.ById:\n    rows:\n      - {id: o-1, customer_id: c-1, invoice_id: i-3}\n      \
         - {id: o-2, customer_id: c-2, invoice_id: i-4}\n",
    ),
    (
        "customers.yaml",
        "views:\n  customers.ById:\n    rows:\n      - {id: c-1, email: ada@shop.test}\n      \
         - {id: c-2, email: grace@shop.test}\n",
    ),
    (
        "lines.yaml",
        "views:\n  lines.All:\n    rows:\n      - {sku: bolt}\n",
    ),
    (
        "invoices.yaml",
        "views:\n  invoices.ById:\n    rows:\n      - {id: i-1}\n      - {id: i-3}\n      - {id: i-4}\n",
    ),
];

fn open(name: &str, body: &str, slow: Option<&str>) -> App {
    let dir: PathBuf = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("ess-ui-tui-adversary-section-row-pass2")
        .join(name);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("a stale dir is removed");
    }
    std::fs::create_dir_all(&dir).expect("the dir is created");
    for (name, body) in FIXTURES {
        std::fs::write(dir.join(name), body).expect("the fixture is written");
    }
    let mut options = Options::new(dir.join("state"));
    if let Some(view) = slow {
        options
            .view_latency
            .insert(view.to_owned(), Duration::from_secs(1));
    }
    let text = format!("{FIXTURES_HEAD}{body}");
    let mut app = App::from_text(&text, &dir, options).unwrap_or_else(|error| panic!("{error}"));
    app.open_page("order", &[]);
    app.advance(Duration::from_secs(1));
    app
}

fn ids(app: &App, section: &str) -> Vec<String> {
    app.rows(section)
        .iter()
        .map(|row| row["id"].as_str().unwrap_or_default().to_owned())
        .collect()
}

/// The terminal half of the React case of the same name: the shell's overlay reads invoice `i-3`.
#[test]
fn a_shell_overlays_read_param_bound_to_a_sections_row_resolves() {
    let mut app = open("shell-overlay", SHELL_OVERLAY, None);
    assert_eq!(ids(&app, "order"), ["o-1", "o-2"]);
    app.open_overlay("invoice");
    let screen = app.render_text(120, 40);
    assert!(screen.contains("i-3"), "{screen}");
    assert!(!screen.contains("i-1"), "{screen}");
}

/// `o-2` is picked and `pick` reads again with the same params (`R`, which the fix holds rows
/// for). The terminal's selection is the row under the cursor of the rows shown, and a read
/// under way shows none: the selection goes, `order` reads with no id, its first row is `o-1`,
/// and `customer` reads `c-1`, another order's customer. React holds the selection and the row
/// (`o-2`, `c-2`) while `pick` refreshes.
#[test]
fn a_row_downstream_of_a_selection_is_held_while_the_collection_reads_again() {
    let mut app = open("selection-refresh", SELECTION, Some("orders.All"));
    assert_eq!(ids(&app, "pick"), ["o-1", "o-2"]);
    app.focus_section("pick");
    app.keys("j");
    assert_eq!(ids(&app, "order"), ["o-2"]);
    assert_eq!(ids(&app, "customer"), ["c-2"]);
    app.keys("R");
    assert_eq!(app.section_state("pick"), Lifecycle::Loading);
    assert_eq!(
        ids(&app, "customer"),
        ["c-2"],
        "section.order.row is o-2 while pick reads again for the same params"
    );
}
