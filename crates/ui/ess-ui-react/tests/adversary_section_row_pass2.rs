//! Adversary pass 2 on `section.<name>.row`, after the fix for pass 1: the same documents as
//! `ess-ui-tui/tests/adversary_section_row_pass2.rs`, which holds the terminal's value for each
//! case. The reference: "the first row of a section that reads; absent while the section loads,
//! when it is empty, and for a section without a read".
//!
//! Runtime cases compile the generated project to `CommonJS` with `tsc` and run it under `node`
//! against the stand-ins in `support`.

mod support;

use std::path::{Path, PathBuf};

use support::{compile, node};

/// `pick` chooses the order `order` reads; `customer` reads by `section.order.row.customer_id`;
/// the header's title is the `id` of the record `order` holds.
const SELECTION: &str = r"
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
    header:
      title: Order
      title_from: {section: order, field: id}
    sections:
      - {name: pick, component: collection, selection: single, reads: {view: orders.All, key: id}, columns: [id]}
      - {name: order, component: record, reads: {view: orders.ById, key: id, params: {id: section.pick.selection.id}}, fields: [id]}
      - {name: customer, component: record, reads: {view: customers.ById, key: id, params: {id: section.order.row.customer_id}}, fields: [email]}
";

/// The overlay `invoice` is the shell's, not the page's; it reads by
/// `section.order.row.invoice_id`, and a row action of the page opens it.
const SHELL_OVERLAY: &str = r"
format: ess-ui/1
app: shop
model: shop.system
placement_profile: fat
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

fn project(name: &str, text: &str) -> PathBuf {
    let out = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("ess-ui-react-adversary-section-row-pass2")
        .join(name);
    if out.exists() {
        std::fs::remove_dir_all(&out).expect("the old scratch project is removed");
    }
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let document = ess_ui::load_str(text).unwrap_or_else(|error| panic!("{error}"));
    ess_ui_react::generate(&document, &root, &out).unwrap_or_else(|error| panic!("{error}"));
    compile(&out, &["App.tsx"]);
    out
}

/// A read of a view named in `hold` never answers: the read it starts stays under way.
const HARNESS: &str = r#"
const b = browser("/order");
const { App } = out("App");
const { setDataAdapter, invalidate } = out("runtime/data");
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
const hold = new Set();
setDataAdapter({
  read: async (request) => {
    requests.push({ view: request.view, params: { ...request.params } });
    if (hold.has(request.view)) return new Promise(() => {});
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
const heading = () => root.find((node) => node.tag === "h1").map(textOf).join("|");
const lastRead = (view) => requests.filter((request) => request.view === view).at(-1);
const count = (view) => requests.filter((request) => request.view === view).length;
"#;

fn run(name: &str, text: &str, body: &str) {
    let out = project(name, text);
    node(
        &out,
        name,
        &format!(
            "{HARNESS}\n(async () => {{\n{body}\n}})().catch((error) => {{ console.error(error); process.exitCode = 1; }});\n"
        ),
    );
}

/// The shell's overlay reads by `section.order.row.invoice_id`. The terminal resolves the page's
/// sections in every overlay it opens, its shell's too, and reads invoice `i-3`; the React shell
/// renders its overlays outside the page, where no section is reported, and reads with no id.
#[test]
fn a_shell_overlays_read_param_bound_to_a_sections_row_resolves() {
    run(
        "shell-overlay",
        SHELL_OVERLAY,
        r#"
  await settle();
  const open = root.find((n) => n.tag === "button" && textOf(n) === "Open invoice");
  assert.strictEqual(open.length, 1, root.text());
  open[0].props.onClick();
  await settle();
  const invoice = lastRead("invoices.ById");
  assert.ok(invoice, "the shell's overlay reads: " + JSON.stringify(requests));
  assert.deepStrictEqual(invoice.params, { id: "i-3" }, "the overlay's param is section.order.row.invoice_id: " + JSON.stringify(requests));
"#,
    );
}

/// Acceptance 5: `header.title_from` keeps its behaviour in both renderers, and the changelog
/// says it is unchanged in both. At the base the React header kept the record `order` held while
/// `order` read for another pick (`row: data?.rows[0]` for the title's section, rows kept while
/// refreshing); the title now reads the row the fix gates on `current`, so it falls back to the
/// literal title while the section reloads.
#[test]
fn the_header_title_keeps_its_record_while_its_section_reloads_for_new_params() {
    run(
        "title-reload",
        SELECTION,
        r#"
  await settle();
  assert.ok(heading().includes("o-1"), "the title is the id of the first order: " + heading());
  hold.add("orders.ById");
  const second = root.find((n) => n.tag === "tr" && textOf(n) === "o-2");
  assert.strictEqual(second.length, 1, root.text());
  second[0].props.onClick();
  await settle();
  assert.strictEqual(lastRead("orders.ById").params.id, "o-2", "order reloads for o-2: " + JSON.stringify(requests));
  assert.ok(heading().includes("o-1"), "title_from keeps the record it showed while order reloads, as at the base: " + heading());
"#,
    );
}

/// The React half of the terminal case of the same name: `pick` reads again with the same params
/// while `o-2` is picked. The selection is held while `pick` refreshes, `order` keeps `o-2`, and
/// `customer` keeps reading `c-2`.
#[test]
fn a_row_downstream_of_a_selection_is_held_while_the_collection_reads_again() {
    run(
        "selection-refresh",
        SELECTION,
        r#"
  await settle();
  const second = root.find((n) => n.tag === "tr" && textOf(n) === "o-2");
  assert.strictEqual(second.length, 1, root.text());
  second[0].props.onClick();
  await settle();
  assert.strictEqual(lastRead("customers.ById").params.id, "c-2", JSON.stringify(requests));
  const before = count("orders.All");
  hold.add("orders.All");
  invalidate();
  await settle();
  assert.ok(count("orders.All") > before, "pick reads again: " + JSON.stringify(requests));
  assert.strictEqual(lastRead("orders.ById").params.id, "o-2", "order keeps the pick: " + JSON.stringify(requests));
  assert.strictEqual(lastRead("customers.ById").params.id, "c-2", "section.order.row is o-2 while pick reads again: " + JSON.stringify(requests));
"#,
    );
}
