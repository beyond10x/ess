//! Adversary pass 3 on `section.<name>.row`, after the fix for pass 2: the React half of
//! `ess-ui-tui/tests/adversary_section_row_pass3.rs`. `o-2` is picked and `pick` reads again;
//! its second answer is in another order, or fails. Each case asserts the value the React
//! project binds; the terminal case of the same name asserts the same value.
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
  sections: [{name: all, pages: [order]}]
pages:
  order:
    kind: detail_page
    title: Order
    sections:
      - {name: pick, component: collection, selection: single, reads: {view: orders.All, key: id}, columns: [id]}
      - {name: order, component: record, reads: {view: orders.ById, key: id, params: {id: section.pick.selection.id}}, fields: [id]}
      - {name: customer, component: record, reads: {view: customers.ById, key: id, params: {id: section.order.row.customer_id}}, fields: [email]}
";

fn project(name: &str) -> PathBuf {
    let out = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("ess-ui-react-adversary-section-row-pass3")
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

/// A read of a view named in `fail` is refused.
const HARNESS: &str = r#"
const b = browser("/order");
const { App } = out("App");
const { setDataAdapter, invalidate } = out("runtime/data");
const requests = [];
const tables = {
  "orders.All": [{ id: "o-1" }, { id: "o-2" }],
  "orders.ById": [
    { id: "o-1", customer_id: "c-1" },
    { id: "o-2", customer_id: "c-2" },
    { id: "o-3", customer_id: "c-3" },
  ],
  "customers.ById": [
    { id: "c-1", email: "ada@shop.test" },
    { id: "c-2", email: "grace@shop.test" },
    { id: "c-3", email: "alan@shop.test" },
  ],
};
const fail = new Set();
setDataAdapter({
  read: async (request) => {
    requests.push({ view: request.view, params: { ...request.params } });
    if (fail.has(request.view)) throw new Error("orders are down");
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
const count = (view) => requests.filter((request) => request.view === view).length;
async function pickSecond() {
  await settle();
  const second = root.find((n) => n.tag === "tr" && textOf(n) === "o-2");
  assert.strictEqual(second.length, 1, root.text());
  second[0].props.onClick();
  await settle();
  assert.strictEqual(lastRead("orders.ById").params.id, "o-2", JSON.stringify(requests));
  assert.strictEqual(lastRead("customers.ById").params.id, "c-2", JSON.stringify(requests));
}
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

#[test]
fn a_selection_is_kept_on_its_record_when_the_reread_answers_in_another_order() {
    run(
        "new-first",
        r#"
  await pickSecond();
  const before = count("orders.All");
  tables["orders.All"] = [{ id: "o-3" }, { id: "o-1" }, { id: "o-2" }];
  invalidate();
  await settle();
  assert.ok(count("orders.All") > before, "pick reads again: " + JSON.stringify(requests));
  assert.ok(root.text().includes("o-3"), root.text());
  assert.strictEqual(lastRead("orders.ById").params.id, "o-2", "the selection stays on o-2: " + JSON.stringify(requests));
  assert.strictEqual(lastRead("customers.ById").params.id, "c-2", JSON.stringify(requests));
"#,
    );
}

#[test]
fn a_selection_after_its_collections_reread_fails() {
    run(
        "fails",
        r#"
  await pickSecond();
  const before = count("orders.All");
  fail.add("orders.All");
  invalidate();
  await settle();
  assert.ok(count("orders.All") > before, "pick reads again: " + JSON.stringify(requests));
  assert.strictEqual(lastRead("orders.ById").params.id, "o-2", "the selection stays on o-2: " + JSON.stringify(requests));
  assert.strictEqual(lastRead("customers.ById").params.id, "c-2", JSON.stringify(requests));
"#,
    );
}
