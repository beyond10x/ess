//! `section.<name>.row` in the generated project: every section that reads reports its first row
//! to the page, so a read parameter and a navigate parameter bound to it resolve on a section
//! that is not the one the header's title reads its record from. The terminal renderer holds the
//! same document and the same expected values (`ess-ui-tui`, `tests/section_row.rs`).
//!
//! Runtime cases compile the generated project to `CommonJS` with `tsc` and run it under `node`
//! against the stand-ins in `support`.

mod support;

use std::path::{Path, PathBuf};

use support::{compile, node};

const DOCUMENT: &str = r"
format: ess-ui/1
app: shop
model: shop.system
placement_profile: fat
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

fn project(name: &str, text: &str) -> PathBuf {
    let out = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("ess-ui-react-section-row")
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

const HARNESS: &str = r#"
const b = browser("/order");
const { App } = out("App");
const { setDataAdapter } = out("runtime/data");
const requests = [];
const tables = {
  "orders.ById": [{ id: "o-1", customer_id: "c-9", invoice_id: "i-3" }],
  "customers.ById": [
    { id: "c-1", name: "Ada", email: "ada@shop.test" },
    { id: "c-9", name: "Grace", email: "grace@shop.test" },
  ],
  "lines.All": [{ sku: "bolt" }],
  "invoices.ById": [{ id: "i-3" }],
};
let hold = false;
setDataAdapter({
  read: async (request) => {
    requests.push({ view: request.view, params: { ...request.params } });
    if (hold && request.view === "orders.ById") return new Promise(() => {});
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

#[test]
fn a_read_param_bound_to_a_sections_row_resolves_on_a_section_the_title_does_not_read() {
    run(
        "read-param",
        DOCUMENT,
        r#"
  await settle();
  assert.deepStrictEqual(lastRead("customers.ById").params, { id: "c-9" }, JSON.stringify(requests));
  assert.ok(root.text().includes("grace@shop.test"), root.text());
  assert.ok(!root.text().includes("ada@shop.test"), root.text());
"#,
    );
}

#[test]
fn a_navigate_param_bound_to_a_sections_row_resolves_on_a_section_the_title_does_not_read() {
    run(
        "navigate-param",
        DOCUMENT,
        r#"
  await settle();
  const open = root.find((n) => n.tag === "button" && textOf(n) === "Open invoice");
  assert.strictEqual(open.length, 1, root.text());
  open[0].props.onClick();
  await settle();
  assert.ok(
    b.calls.some(([kind, href]) => kind === "push" && href === "/invoice/i-3"),
    JSON.stringify(b.calls),
  );
"#,
    );
}

#[test]
fn the_header_title_still_reads_its_sections_record() {
    run(
        "title",
        DOCUMENT,
        r#"
  await settle();
  assert.strictEqual(heading(), "Grace", "the title is the record field");
"#,
    );
}

/// A read started again with the same params (a command invalidates every read; a poll or a live
/// refresh does the same) does not blank the row: `customer` still reads `c-9` while the read of
/// `order` is under way. The terminal holds the row the same way
/// (`ess-ui-tui`, `a_row_is_held_while_its_read_runs_again_for_the_same_params`).
#[test]
fn a_row_is_held_while_its_read_runs_again_for_the_same_params() {
    run(
        "same-params",
        DOCUMENT,
        r#"
  const { invalidate } = out("runtime/data");
  await settle();
  assert.deepStrictEqual(lastRead("customers.ById").params, { id: "c-9" }, JSON.stringify(requests));
  const before = requests.filter((request) => request.view === "orders.ById").length;
  hold = true;
  invalidate();
  await settle();
  const after = requests.filter((request) => request.view === "orders.ById").length;
  assert.ok(after > before, "order reads again: " + JSON.stringify(requests));
  assert.deepStrictEqual(lastRead("customers.ById").params, { id: "c-9" }, "the row is held while order reads again: " + JSON.stringify(requests));
  assert.ok(root.text().includes("grace@shop.test"), root.text());
"#,
    );
}

#[test]
fn every_section_with_a_read_reports_its_row_and_none_is_marked() {
    let files = {
        let document = ess_ui::load_str(DOCUMENT).unwrap_or_else(|error| panic!("{error}"));
        ess_ui_react::render(&document, Path::new("."))
            .unwrap_or_else(|error| panic!("the project renders: {error}"))
    };
    let page = &files.files["src/pages/Order.tsx"];
    assert!(!page.contains("reportsRow"), "{page}");
    let core = &files.files["src/runtime/core.tsx"];
    assert!(!core.contains("reportsRow"), "{core}");
}
