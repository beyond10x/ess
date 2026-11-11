//! Adversary, turn 3 (story:a-page-shows-its-title-total-badge-and-live-status-and-marks-the-current-page):
//! the header and navigation the unit emits, driven at runtime under `node` where the unit's own
//! cases only read the template text, plus two cases on the emitted page files.
//!
//! 1. Navigation marks the current page by comparing the link's href with `location.pathname`
//!    byte for byte, while `matchPage` (`routes.ts`) answers case-insensitively, with a trailing
//!    slash optional and on every alias. On such a path the page is shown and no link is marked.
//! 2. `loading` is a channel lifecycle (`live.ts` `Lifecycle`, the schema's lifecycle enum); the
//!    header's word table has no entry for it, so the mark reads the raw `loading`.
//! 3. A total of 0 and a large total, grouped and named (pins the runtime the suite only greps).
//! 4. The acceptance: "A generated page file changes only where its page has a header `switch` or
//!    `switch_to` ... or a `badge`". A header `total` over a titled section now adds `totalLabel`.
//! 5. A header without a `title` on a page with one: the terminal draws the page title, React
//!    draws no heading at all.

mod support;

use std::path::{Path, PathBuf};

use support::{compile, node};

fn scratch(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("ess-ui-react-adversary-t3-page-header")
        .join(name);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("the old scratch dir is removed");
    }
    std::fs::create_dir_all(&dir).expect("the scratch dir is created");
    dir
}

fn generate(name: &str, text: &str) -> (PathBuf, ess_ui_react::GeneratedFiles) {
    let dir = scratch(name);
    let document = ess_ui::load_str(text).unwrap_or_else(|error| panic!("{error}"));
    let project = dir.join("app");
    let files = ess_ui_react::generate(&document, &dir, &project)
        .unwrap_or_else(|error| panic!("the document generates: {error}"));
    (project, files)
}

const NAV: &str = r"
format: ess-ui/1
app: shop
model: shop.system
placement_profile: fat
shells:
  app: {regions: {navigation: {kind: navigation}, main: {kind: page_outlet}}}
navigation:
  home: things.list
  sections: [{name: all, pages: [things.list, things.new]}]
pages:
  things.list:
    kind: static_page
    title: Things
    aliases: [old.list]
    sections:
      - {name: list, component: collection, reads: {view: shop.Things}, columns: [id]}
  things.new:
    kind: static_page
    title: New thing
    sections:
      - {name: list, component: collection, reads: {view: shop.Things}, columns: [id]}
";

const TEXT_OF: &str = r#"
function textOf(node) {
  if (typeof node === "string") return node;
  return (node.children || []).map(textOf).join("");
}
"#;

#[test]
fn navigation_marks_the_current_page_on_every_path_the_router_shows_it_on() {
    let (project, _) = generate("nav", NAV);
    compile(&project, &["runtime/navigation.tsx", "runtime/router.tsx"]);
    node(
        &project,
        "nav",
        &format!(
            r#"{TEXT_OF}
const {{ Router }} = out("runtime/router");
const {{ Navigation, NavSection }} = out("runtime/navigation");
const {{ matchPage }} = out("routes");
const links = [
  {{ page: "things.list", label: "Things", synonyms: [] }},
  {{ page: "things.new", label: "New thing", synonyms: [] }},
];
function marked(path) {{
  browser(path);
  const root = React.__root(jsx(Router, {{ children: jsx(Navigation, {{ restrictions: [], children: jsx(NavSection, {{ "data-ui-path": "navigation/sections/all", name: "all", links }}) }}) }}));
  root.settle();
  return root.find((n) => n.tag === "a" && n.props["aria-current"] === "page").map(textOf);
}}
assert.deepStrictEqual(marked("/things/list"), ["Things"], "the control: the page's own path marks its link");
assert.deepStrictEqual(marked("/things/new"), ["New thing"], "a sibling under the same prefix marks only itself");
const missed = [];
for (const path of ["/things/list/", "/Things/List", "/old/list"]) {{
  assert.strictEqual(matchPage(path).page, "things.list", `the router shows things.list at ${{path}}`);
  const got = marked(path);
  if (JSON.stringify(got) !== JSON.stringify(["Things"])) missed.push(`${{path}} -> ${{JSON.stringify(got)}}`);
}}
assert.deepStrictEqual(missed, [], "the router shows things.list on these paths, and navigation marks no link there");
"#
        ),
    );
}

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
channels:
  orders:
    carries: {events: [shop.OrderPlaced]}
    direction: server_to_client
    delivery: every_event
    resume: refetch
pages:
  orders:
    kind: static_page
    title: Orders
    header: {title: Orders, total: list, live: [orders]}
    sections:
      - {name: list, component: collection, title: Open orders, reads: {view: shop.Orders}, columns: [id]}
";

fn header_script(project: &Path, name: &str, script: &str) {
    compile(project, &["runtime/core.tsx"]);
    node(
        project,
        name,
        &format!(
            r#"{TEXT_OF}
const {{ ScopeLayer, HeaderFrame }} = out("runtime/core");
function header(values, props) {{
  const root = React.__root(jsx(ScopeLayer, {{ values, children: jsx(HeaderFrame, {{ "data-ui-path": "pages/orders/header", ...props }}) }}));
  return root.settle();
}}
{script}
"#
        ),
    );
}

#[test]
fn a_loading_channel_reads_as_a_word() {
    let (project, _) = generate("loading", HEADER);
    header_script(
        &project,
        "loading",
        r#"
const mark = (lifecycle) => {
  const root = header({ channel: { orders: { lifecycle } }, section: {} }, { live: ["orders"] });
  const found = root.find((n) => n.tag === "span" && n.props.className === "ui-live");
  assert.strictEqual(found.length, 1);
  return { text: textOf(found[0]), lifecycle: found[0].props["data-lifecycle"], title: found[0].props.title };
};
assert.deepStrictEqual(mark("live"), { text: "Live", lifecycle: "live", title: "orders" }, "the control");
assert.deepStrictEqual(mark("loading"), { text: "Loading", lifecycle: "loading", title: "orders" }, "`loading` is a channel lifecycle and reads as a word");
"#,
    );
}

#[test]
fn a_total_of_zero_and_a_large_total_render_as_named_grouped_counts() {
    let (project, _) = generate("total", HEADER);
    header_script(
        &project,
        "total",
        r#"
const count = (total, props) => {
  const root = header({ section: { list: { total } } }, { total: "list", ...props });
  const found = root.find((n) => n.tag === "span" && /\bui-count\b/.test(String(n.props.className)));
  assert.strictEqual(found.length, 1, `one count mark for ${total}`);
  return { text: textOf(found[0]), name: found[0].props["aria-label"] };
};
assert.deepStrictEqual(count(0, {}), { text: "0", name: "0 rows" }, "a total of 0 is drawn, named by rows");
const grouped = new Intl.NumberFormat().format(1234567);
assert.notStrictEqual(grouped, "1234567", "the locale groups");
assert.deepStrictEqual(count(1234567, { totalLabel: "Open orders" }), { text: grouped, name: "1234567 Open orders" });
const none = header({ section: {} }, { total: "list" });
assert.strictEqual(none.find((n) => n.tag === "span" && /\bui-count\b/.test(String(n.props.className))).length, 0, "no total yet, no mark");
"#,
    );
}

#[test]
fn a_page_with_a_total_and_no_badge_or_switch_keeps_its_page_file() {
    let (_, files) = generate("bytes", HEADER);
    let page = &files.files["src/pages/Orders.tsx"];
    assert!(
        !page.contains("totalLabel"),
        "the acceptance: a page file changes only where its page has a header switch, switch_to \
         or badge; this page has none and gains `totalLabel`:\n{page}"
    );
}

#[test]
#[ignore = "pre-existing: a header without a title draws no page title in React, the terminal falls back to it; backlog"]
fn a_header_without_a_title_still_shows_the_page_title() {
    const UNTITLED: &str = r"
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
    header: {total: list}
    sections:
      - {name: list, component: collection, reads: {view: shop.Orders}, columns: [id]}
";
    let (_, files) = generate("untitled", UNTITLED);
    let page = &files.files["src/pages/Orders.tsx"];
    assert!(
        page.contains("title=\"Orders\"") || page.contains("title={\"Orders\"}"),
        "a page titled `Orders` whose header has no title is given no title to draw (the terminal \
         draws `Orders`):\n{page}"
    );
}
