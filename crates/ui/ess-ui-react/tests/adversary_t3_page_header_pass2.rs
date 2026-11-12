//! Adversary, turn 3, pass 2 (story:a-page-shows-its-title-total-badge-and-live-status-and-marks-the-current-page):
//! the fixes of pass 1, driven at runtime under `node`.
//!
//! 1. The count is named by the title its section reports to the page registry (the fix of
//!    pass 1's F2). The unit's `a_total_reads_as_a_count` greps the `HeaderFrame` text for `.title`,
//!    which `props.title` already holds, and pass 1's runtime case passes `totalLabel` itself, so
//!    dropping the registry lookup leaves the suite green. This case drives `PageFrame`, two
//!    titled sections with totals and a header naming one of them, changes that section's title
//!    at runtime, and runs the same check against a copy of the compiled runtime with the lookup
//!    removed, which must fail.
//! 2. Navigation (the fix of pass 1's F1) with two links to one page that differ by their path
//!    param, a page reached by two aliases, a param that needs encoding, and a param link whose
//!    href is another page's path.

mod support;

use std::path::{Path, PathBuf};

use support::{compile, node};

fn scratch(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("ess-ui-react-adversary-t3-page-header-pass2")
        .join(name);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("the old scratch dir is removed");
    }
    std::fs::create_dir_all(&dir).expect("the scratch dir is created");
    dir
}

fn generate(name: &str, text: &str) -> PathBuf {
    let dir = scratch(name);
    let document = ess_ui::load_str(text).unwrap_or_else(|error| panic!("{error}"));
    let project = dir.join("app");
    ess_ui_react::generate(&document, &dir, &project)
        .unwrap_or_else(|error| panic!("the document generates: {error}"));
    project
}

const TEXT_OF: &str = r#"
function textOf(node) {
  if (typeof node === "string") return node;
  return (node.children || []).map(textOf).join("");
}
"#;

const HEADER: &str = r"
format: ess-ui/1
app: shop
model: shop.system
placement_profile: fat
shells:
  app: {regions: {main: {kind: page_outlet}}}
navigation:
  home: orders
  sections: [{name: all, pages: [orders]}]
pages:
  orders:
    kind: static_page
    title: Orders
    header: {title: Orders, total: open}
    sections:
      - {name: done, component: collection, title: Done orders, reads: {view: shop.Orders}, columns: [id]}
      - {name: open, component: collection, title: Open orders, reads: {view: shop.Orders}, columns: [id]}
";

#[test]
fn the_count_is_named_by_the_title_its_section_reports() {
    let project = generate("named", HEADER);
    compile(&project, &["runtime/core.tsx"]);
    node(
        &project,
        "named",
        &format!(
            r#"{TEXT_OF}
const fs = require("fs");
const path = require("path");
const compiled = path.join(process.cwd(), "test-out/src/runtime/core.js");
const lookup = "(props.total ? sections[props.total]?.title : undefined) ?? ";
const text = fs.readFileSync(compiled, "utf8");
assert.strictEqual(text.split(lookup).length, 2, "the compiled HeaderFrame names the count by the reported section title");
fs.writeFileSync(compiled.replace(/core\.js$/, "core_mutant.js"), text.replace(lookup, ""));
const read = (total) => ({{ status: "ready", rows: [{{ id: 1 }}], total, params: {{}}, refetch() {{}} }});
function run(core) {{
  const {{ PageFrame, HeaderFrame, SectionFrame }} = core;
  let setTitle;
  function Open() {{
    const [title, set] = React.useState("Open orders");
    setTitle = set;
    return jsx(SectionFrame, {{ "data-ui-path": "pages/orders/sections/open", name: "open", title, data: read(1234567) }});
  }}
  const root = React.__root(jsx(PageFrame, {{ "data-ui-path": "pages/orders", children: [
    jsx(HeaderFrame, {{ "data-ui-path": "pages/orders/header", title: "Orders", total: "open" }}, "h"),
    jsx(SectionFrame, {{ "data-ui-path": "pages/orders/sections/done", name: "done", title: "Done orders", data: read(3) }}, "d"),
    jsx(Open, {{}}, "o"),
  ] }}));
  root.settle();
  const count = () => {{
    const found = root.find((n) => n.tag === "span" && /\bui-count\b/.test(String(n.props.className)));
    assert.strictEqual(found.length, 1, "one count mark");
    return {{ role: found[0].props.role, name: found[0].props["aria-label"], text: textOf(found[0]) }};
  }};
  const first = count();
  setTitle("Open and held orders");
  root.settle();
  return [first, count()];
}}
const grouped = new Intl.NumberFormat().format(1234567);
const want = [
  {{ role: "img", name: "1234567 Open orders", text: grouped }},
  {{ role: "img", name: "1234567 Open and held orders", text: grouped }},
];
assert.deepStrictEqual(run(out("runtime/core")), want, "the count is named by the title its own section reports, and follows it");
assert.throws(
  () => assert.deepStrictEqual(run(out("runtime/core_mutant")), want),
  assert.AssertionError,
  "this case catches a HeaderFrame that no longer reads the reported title",
);
"#
        ),
    );
}

const NAV: &str = r"
format: ess-ui/1
app: shop
model: shop.system
placement_profile: fat
shells:
  app: {regions: {navigation: {kind: navigation}, main: {kind: page_outlet}}}
navigation:
  home: orders.list
  sections: [{name: all, pages: [orders.list]}]
pages:
  orders:
    kind: detail_page
    title: Order
    params: {id: string}
    aliases: [legacy.order, o]
    sections:
      - name: summary
        reads: {view: orders.ById}
        fields: [title]
  orders.list:
    kind: detail_page
    title: Orders
    sections:
      - name: summary
        reads: {view: orders.All}
        fields: [title]
";

#[test]
fn navigation_marks_the_one_link_whose_page_and_params_are_shown() {
    let project = generate("nav", NAV);
    compile(&project, &["runtime/navigation.tsx", "runtime/router.tsx"]);
    node(
        &project,
        "nav",
        &format!(
            r#"{TEXT_OF}
const {{ Router }} = out("runtime/router");
const {{ Navigation, NavSection }} = out("runtime/navigation");
const {{ hrefFor, matchPage }} = out("routes");
const links = [
  {{ page: "orders.list", label: "List", synonyms: [] }},
  {{ page: "orders", label: "One", synonyms: [], params: {{ id: 1 }} }},
  {{ page: "orders", label: "Spaced", synonyms: [], params: {{ id: "a b/c" }} }},
  {{ page: "orders", label: "Id list", synonyms: [], params: {{ id: "list" }} }},
];
assert.strictEqual(hrefFor("orders", {{ id: "list" }}), "/orders/list", "the param link's href is the list page's path");
function marked(at) {{
  browser(at);
  const root = React.__root(jsx(Router, {{ children: jsx(Navigation, {{ restrictions: [], children: jsx(NavSection, {{ "data-ui-path": "navigation/sections/all", name: "all", links }}) }}) }}));
  root.settle();
  return root.find((n) => n.tag === "a" && n.props["aria-current"] === "page").map(textOf);
}}
const want = {{
  "/orders/1": ["One"],
  "/Orders/1/": ["One"],
  "/legacy/order/1": ["One"],
  "/o/1": ["One"],
  "/orders/a%20b%2Fc": ["Spaced"],
  "/O/a%20b%2Fc/": ["Spaced"],
  "/orders/3": [],
  "/orders/list": ["List"],
  "/orders/list?page=2": ["List"],
}};
const got = {{}};
for (const at of Object.keys(want)) got[at] = marked(at);
assert.deepStrictEqual(got, want, `matchPage: ${{JSON.stringify(Object.keys(want).map((at) => [at, matchPage(at.split("?")[0])]))}}`);
"#
        ),
    );
}
