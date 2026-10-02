//! The generated router: `routes.ts` matches a path to its page over pages and aliases, a static
//! segment before a param, and refuses two pages on one path; `runtime/router.tsx` follows the
//! History API — links push, URL state replaces, `popstate` re-renders, and an unknown path
//! redirects home without leaving a history entry behind.
//!
//! Runtime cases compile the generated project to `CommonJS` with `tsc` and run it under `node`
//! against the stand-ins in `support`.

mod support;

use std::path::{Path, PathBuf};

use ess_ui::Document;
use support::{compile, node};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

fn scratch(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("ess-ui-react-router")
        .join(name);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("the old scratch project is removed");
    }
    dir
}

fn load(text: &str) -> Document {
    ess_ui::load_str(text).unwrap_or_else(|error| panic!("the document loads: {error}"))
}

/// `things` routes `/things/:id` and is declared before `things.list` and `things.new`, whose
/// static segments must still win; it keeps an old path as an alias.
const SHOP: &str = r"
format: ess-ui/1
app: shop
model: shop.system
placement_profile: fat
shells:
  app: {regions: {main: {kind: page_outlet}}}
navigation:
  home: things.list
  sections: [{name: all, pages: [things.list]}]
pages:
  things:
    kind: detail_page
    title: Thing
    params: {id: string}
    aliases: [legacy.thing]
    sections:
      - name: summary
        reads: {view: things.ById}
        fields: [title]
  things.list:
    kind: detail_page
    title: Things
    state:
      q: {type: string, class: page_state, store: url}
    sections:
      - name: summary
        reads: {view: things.All}
        fields: [title]
  things.new:
    kind: detail_page
    title: New thing
    sections:
      - name: summary
        reads: {view: things.All}
        fields: [title]
";

/// The shop project, compiled: the app with every page, shell and runtime module it imports.
fn shop(name: &str) -> PathBuf {
    let out = scratch(name);
    ess_ui_react::generate(&load(SHOP), &root(), &out).unwrap_or_else(|error| panic!("{error}"));
    compile(&out, &["App.tsx", "runtime/state/url.ts"]);
    out
}

#[test]
fn a_page_path_matches_its_page_and_decodes_its_params() {
    let project = shop("match");
    node(
        &project,
        "match",
        r#"
const { matchPage, pageAt } = out("routes");
assert.deepStrictEqual(matchPage("/things/a%20b%2Fc"), { page: "things", params: { id: "a b/c" } });
assert.deepStrictEqual(matchPage("/Things/7/"), { page: "things", params: { id: "7" } }, "case-insensitive, trailing slash optional");
assert.strictEqual(pageAt("/things/7"), "things");
assert.strictEqual(matchPage("/things"), undefined, "a missing param matches nothing");
assert.strictEqual(matchPage("/things/7/more"), undefined, "an extra segment matches nothing");
"#,
    );
}

#[test]
fn a_static_segment_outranks_a_param_segment() {
    let project = shop("rank");
    node(
        &project,
        "rank",
        r#"
const { matchPage } = out("routes");
assert.deepStrictEqual(matchPage("/things/new"), { page: "things.new", params: {} });
assert.deepStrictEqual(matchPage("/things/list"), { page: "things.list", params: {} });
assert.deepStrictEqual(matchPage("/things/old"), { page: "things", params: { id: "old" } });
"#,
    );
}

#[test]
fn an_alias_path_reaches_its_page() {
    let project = shop("alias");
    node(
        &project,
        "alias",
        r#"
const { matchPage, pageAt } = out("routes");
assert.deepStrictEqual(matchPage("/legacy/thing/7"), { page: "things", params: { id: "7" } });
assert.strictEqual(pageAt("/legacy/thing/7"), "things", "pageAt agrees with the router on aliases");
"#,
    );
}

#[test]
fn two_pages_routing_the_same_path_are_refused() {
    const SAME: &str = r"
format: ess-ui/1
app: twice
model: twice.system
placement_profile: fat
shells:
  app: {regions: {main: {kind: page_outlet}}}
navigation:
  home: a.view
  sections: [{name: all, pages: [a.view]}]
pages:
  a.view:
    kind: detail_page
    title: A
    params: {id: string}
    sections:
      - name: summary
        reads: {view: things.ById}
        fields: [title]
  b.view:
    kind: detail_page
    title: B
    params: {key: string}
    aliases: [a.view]
    sections:
      - name: summary
        reads: {view: things.ById}
        fields: [title]
";
    let error = ess_ui_react::render(&load(SAME), &root())
        .map(|_| ())
        .expect_err("two pages on one path are refused")
        .to_string();
    assert!(
        error.contains("pages/a.view") && error.contains("pages/b.view"),
        "the refusal names both pages: {error}"
    );
    assert!(
        error.contains("/a/view/:"),
        "the refusal names the path: {error}"
    );
}

#[test]
fn an_unknown_path_redirects_home_with_replace() {
    let project = shop("unknown");
    node(
        &project,
        "unknown",
        r#"
const { Pages } = out("App");
const router = out("runtime/router");
const expand = (t) => t === router.Router || t === Pages || t === router.Matched || t === router.Redirect;
for (const start of ["/nowhere?x=1", "/"]) {
  const b = browser(start);
  const root = React.__root(jsx(router.Router, { children: jsx(Pages, {}) }), { expand });
  root.render();
  assert.deepStrictEqual(b.calls, [["replace", "/things/list"]], `${start} redirects home, replacing its entry`);
  root.settle();
  assert.deepStrictEqual(b.calls, [["replace", "/things/list"]], "the redirect fires once");
  assert.strictEqual(root.leaves.length, 1);
  assert.strictEqual(root.leaves[0].component, out("shells/AppShell").AppShell, "home renders under its shell");
}
"#,
    );
}

#[test]
fn a_link_click_pushes_history_and_a_modified_click_does_not() {
    let project = shop("link");
    node(
        &project,
        "link",
        r#"
const b = browser("/things/list");
const { Router, Link, useLocation } = out("runtime/router");
function Where() { return useLocation().pathname; }
const root = React.__root(jsx(Router, { children: [jsx(Link, { to: "/things/7", children: "seven" }, "a"), jsx(Where, {}, "w")] }));
root.settle();
const anchor = () => root.find((n) => n.tag === "a")[0];
assert.strictEqual(anchor().props.href, "/things/7", "a link is an <a href>, so it opens in a new tab too");
for (const modifier of [{ metaKey: true }, { ctrlKey: true }, { shiftKey: true }, { altKey: true }, { button: 1 }]) {
  const event = click(anchor(), modifier);
  assert.strictEqual(event.defaultPrevented, false, `${JSON.stringify(modifier)} is left to the browser`);
}
click(anchor(), { defaultPrevented: true });
assert.deepStrictEqual(b.calls, [], "no modified or already handled click navigates");
const event = click(anchor());
assert.strictEqual(event.defaultPrevented, true, "the router handles a plain click");
assert.deepStrictEqual(b.calls, [["push", "/things/7"]]);
root.settle();
assert.strictEqual(root.text(), "seven/things/7", "the router renders the new location");
"#,
    );
}

#[test]
fn popstate_rerenders_the_matched_page() {
    let project = shop("popstate");
    node(
        &project,
        "popstate",
        r#"
const b = browser("/things/list");
const { Pages } = out("App");
const router = out("runtime/router");
const expand = (t) => t === router.Router || t === Pages || t === router.Matched;
const root = React.__root(jsx(router.Router, { children: jsx(Pages, {}) }), { expand });
root.settle();
const matched = () => root.rendered.find((r) => r.component === router.Matched).props;
assert.strictEqual(matched().outlet.type, out("pages/ThingsList").ThingsListPage);
assert.strictEqual(b.listening("popstate"), 1, "the router listens for popstate");
b.pop("/things/9");
root.settle();
assert.strictEqual(matched().outlet.type, out("pages/Things").ThingsPage);
assert.deepStrictEqual({ ...matched().params }, { id: "9" });
assert.deepStrictEqual(b.calls, [], "popstate itself writes no history");
"#,
    );
}

#[test]
fn url_state_replaces_history_and_keeps_other_query_keys() {
    let project = shop("url-state");
    node(
        &project,
        "url-state",
        r#"
const b = browser("/things/list?sort=name#top");
const { Router } = out("runtime/router");
const { useUrlState } = out("runtime/state/url");
let setter;
function Probe() {
  const [value, set] = useUrlState("q", "");
  setter = set;
  return `q=${value}`;
}
const root = React.__root(jsx(Router, { children: jsx(Probe, {}) }));
root.settle();
assert.strictEqual(root.text(), "q=");
setter("bolt");
root.settle();
assert.deepStrictEqual(b.calls, [["replace", "/things/list?sort=name&q=bolt#top"]]);
assert.strictEqual(root.text(), "q=bolt");
setter("");
root.settle();
assert.deepStrictEqual(b.calls, [["replace", "/things/list?sort=name&q=bolt#top"], ["replace", "/things/list?sort=name#top"]]);
assert.strictEqual(root.text(), "q=");
"#,
    );
}
