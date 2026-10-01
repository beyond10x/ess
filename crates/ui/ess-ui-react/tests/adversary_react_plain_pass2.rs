//! Adversary pass 2 on the plain React output: page `aliases`, which the schema calls "legacy
//! route paths" (a free `string`, not a node name), as such a path is naturally written — with a
//! leading slash, or with a segment the address bar percent-encodes. react-router, which the
//! generated router replaces, matched both: it collapses `//` when it joins route paths and
//! decodes the pathname before matching (react-router 7, `matchRoutes`, measured).

mod support;

use std::path::{Path, PathBuf};

use support::{compile, node};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

fn scratch(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("ess-ui-react-adversary-plain-pass2")
        .join(name);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("the old scratch project is removed");
    }
    dir
}

/// `things` keeps three legacy route paths: one written as a path, one with a non-ASCII
/// segment, one with a space.
const LEGACY: &str = r"
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
    aliases: [/legacy/thing, über/uns, old reports]
    sections:
      - name: summary
        reads: {view: things.ById}
        fields: [title]
  things.list:
    kind: detail_page
    title: Things
    sections:
      - name: summary
        reads: {view: things.All}
        fields: [title]
";

fn legacy(name: &str) -> PathBuf {
    let out = scratch(name);
    let doc = ess_ui::load_str(LEGACY).unwrap_or_else(|error| panic!("the document loads: {error}"));
    ess_ui_react::generate(&doc, &root(), &out).unwrap_or_else(|error| panic!("{error}"));
    compile(&out, &["routes.ts"]);
    out
}

/// A legacy route path is written `/legacy/thing`; the pattern becomes `//legacy/thing/:id`,
/// whose empty first segment no address-bar path has, so the old link lands on home.
#[test]
fn an_alias_written_with_a_leading_slash_reaches_its_page() {
    let project = legacy("leading-slash");
    node(
        &project,
        "leading-slash",
        r#"
const { matchPage } = out("routes");
assert.deepStrictEqual(
  matchPage("/legacy/thing/7"),
  { page: "things", params: { id: "7" } },
  "the legacy path /legacy/thing/7 does not reach its page (react-router matched it)",
);
"#,
    );
}

/// `window.location.pathname` is percent-encoded, and `matchPage` compares static segments
/// without decoding them, so `über/uns` and `old reports` never match the address bar.
#[test]
fn an_alias_with_a_segment_the_address_bar_encodes_reaches_its_page() {
    let project = legacy("encoded-static");
    node(
        &project,
        "encoded-static",
        r#"
const { matchPage } = out("routes");
const pathname = (p) => new URL(p, "http://app.test").pathname;
assert.strictEqual(pathname("/über/uns/7"), "/%C3%BCber/uns/7", "the browser encodes the path");
assert.deepStrictEqual(
  matchPage(pathname("/über/uns/7")),
  { page: "things", params: { id: "7" } },
  "the legacy path /über/uns/7 does not reach its page (react-router matched it)",
);
assert.deepStrictEqual(
  matchPage(pathname("/old reports/7")),
  { page: "things", params: { id: "7" } },
  "the legacy path /old reports/7 does not reach its page (react-router matched it)",
);
"#,
    );
}
