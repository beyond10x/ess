//! Adversary pass 1 on `section.<name>.row`: the same document as
//! `ess-ui-tui/tests/adversary_section_row_pass1.rs`, which holds the terminal's value for each
//! case. The reference: "the first row of a section that reads; absent while the section loads,
//! when it is empty, and for a section without a read".
//!
//! Runtime cases compile the generated project to `CommonJS` with `tsc` and run it under `node`
//! against the stand-ins in `support`.

mod support;

use std::path::{Path, PathBuf};

use support::{compile, node};

/// `pick` chooses the order `order` reads; `customer` reads by `section.order.row.customer_id`;
/// the page overlay `invoice` reads by `section.order.row.invoice_id`.
const DOCUMENT: &str = r"
format: ess-ui/1
app: shop
model: shop.system
placement_profile: fat
shells:
  app: {regions: {main: {kind: page_outlet}}}
navigation:
  home: order
  sections: [{name: all, pages: [order]}]
pages:
  order:
    kind: detail_page
    title: Order
    sections:
      - {name: pick, component: collection, selection: single, reads: {view: orders.All, key: id}, columns: [id]}
      - {name: order, component: record, reads: {view: orders.ById, key: id, params: {id: section.pick.selection.id}}, fields: [id]}
      - {name: customer, component: record, reads: {view: customers.ById, key: id, params: {id: section.order.row.customer_id}}, fields: [email]}
      - name: lines
        component: collection
        reads: {view: lines.All, key: sku}
        columns: [sku]
        row_actions:
          - {name: open_invoice, label: Open invoice, opens: invoice}
    overlays:
      invoice:
        kind: drawer
        component: record
        title: Invoice
        reads: {view: invoices.ById, key: id, params: {id: section.order.row.invoice_id}}
        fields: [id]
";

fn project(name: &str) -> PathBuf {
    let out = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("ess-ui-react-adversary-section-row-pass1")
        .join(name);
    if out.exists() {
        std::fs::remove_dir_all(&out).expect("the old scratch project is removed");
    }
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let document = ess_ui::load_str(DOCUMENT).unwrap_or_else(|error| panic!("{error}"));
    ess_ui_react::generate(&document, &root, &out).unwrap_or_else(|error| panic!("{error}"));
    compile(&out, &["App.tsx"]);
    out
}

/// The read of `orders.ById` for `o-2` never answers while `held` is set: the reload it starts
/// stays under way.
const HARNESS: &str = r#"
const b = browser("/order");
const { App } = out("App");
const { setDataAdapter } = out("runtime/data");
const requests = [];
const tables = {
  "orders.All": [{ id: "o-1" }, { id: "o-2" }],
  "orders.ById": [
    { id: "o-1", customer_id: "c-1", invoice_id: "i-3" },
    { id: "o-2", customer_id: "c-2", invoice_id: "i-4" },
  ],
  "customers.ById": [
    { id: "c-1", email: "ada@shop.test" },
    { id: "c-2", email: "grace@shop.test" },
  ],
  "lines.All": [{ sku: "bolt" }],
  "invoices.ById": [{ id: "i-1" }, { id: "i-3" }, { id: "i-4" }],
};
let held = false;
setDataAdapter({
  read: async (request) => {
    requests.push({ view: request.view, params: { ...request.params } });
    if (held && request.view === "orders.ById" && request.params.id === "o-2") return new Promise(() => {});
    const rows = (tables[request.view] || []).filter((row) =>
      Object.entries(request.params).every(([name, value]) => value === undefined || row[name] === value));
    return { rows, total: rows.length };
  },
  send: async () => ({ accepted: true }),
  loadState: async () => undefined,
  saveState: async () => undefined,
});
const root = React.__root(jsx(App, {}));
const tick = () => new Promise((resolve) => setTimeout(resolve, 5));
async function settle() { for (let i = 0; i < 6; i++) { root.settle(); await tick(); } root.settle(); }
function textOf(node) {
  if (typeof node === "string") return node;
  return (node.children || []).map(textOf).join("");
}
const lastRead = (view) => requests.filter((request) => request.view === view).at(-1);
"#;

fn run(name: &str, body: &str) {
    let out = project(name);
    node(
        &out,
        name,
        &format!(
            "{HARNESS}\n(async () => {{\n{body}\n}})().catch((error) => {{ console.error(error); process.exitCode = 1; }});\n"
        ),
    );
}

/// A page overlay's read param bound to `section.order.row.invoice_id`: the terminal reads the
/// invoice `i-3` (the first row of `order` is `o-1`); the React overlay renders outside the page
/// frame's registry and reads with no id.
#[test]
fn a_page_overlays_read_param_bound_to_a_sections_row_resolves() {
    run(
        "overlay",
        r#"
  await settle();
  assert.strictEqual(lastRead("customers.ById").params.id, "c-1", "on the page the row of order resolves: " + JSON.stringify(requests));
  const open = root.find((n) => n.tag === "button" && textOf(n) === "Open invoice");
  assert.strictEqual(open.length, 1, root.text());
  open[0].props.onClick();
  await settle();
  const invoice = lastRead("invoices.ById");
  assert.ok(invoice, "the overlay reads: " + JSON.stringify(requests));
  assert.deepStrictEqual(invoice.params, { id: "i-3" }, "the overlay's param is section.order.row.invoice_id: " + JSON.stringify(requests));
"#,
    );
}

/// `order` reloads for another pick: while its read is under way the reference says its row is
/// absent, and the terminal reads `customer` with no id (every customer). React keeps reporting
/// the previous order's row, so `customer` still reads `c-1`.
#[test]
fn a_sections_row_is_absent_while_it_reloads_for_new_params() {
    run(
        "reload",
        r#"
  await settle();
  assert.strictEqual(lastRead("orders.ById").params.id, undefined, JSON.stringify(requests));
  assert.strictEqual(lastRead("customers.ById").params.id, "c-1", "the first row of order is o-1: " + JSON.stringify(requests));
  held = true;
  const second = root.find((n) => n.tag === "tr" && textOf(n) === "o-2");
  assert.strictEqual(second.length, 1, root.text());
  second[0].props.onClick();
  await settle();
  assert.strictEqual(lastRead("orders.ById").params.id, "o-2", "order reloads for o-2: " + JSON.stringify(requests));
  assert.strictEqual(
    lastRead("customers.ById").params.id,
    undefined,
    "section.order.row is absent while order loads, so customer reads with no id: " + JSON.stringify(requests),
  );
"#,
    );
}
