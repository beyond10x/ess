//! A server-paged collection pages and sizes on the state paths its read binds
//! (story:a-server-paged-collection-has-a-page-size-control).
//!
//! Each case generates a project from a small document, reads the `pager` prop the page hands
//! its collection, and renders the generated collection runtime with that prop under node.
mod support;

use std::path::{Path, PathBuf};

/// A page whose `list` section reads `t.All` as written in `reads`.
fn document(reads: &str) -> String {
    format!(
        "format: ess-ui/1
app: pager
model: t
placement_profile: fat
shells: {{app: {{regions: {{main: {{kind: page_outlet}}}}}}}}
navigation: {{home: p, sections: [{{name: all, pages: [p]}}]}}
pages:
  p:
    kind: detail_page
    title: Paged
    state:
      p: {{type: integer, class: page_state, store: url, default: 1}}
      n: {{type: integer, class: page_state, store: url, default: 25}}
    sections:
      - {{name: list, component: collection, reads: {reads}, columns: [id]}}
"
    )
}

/// Generates the project for `reads` and compiles the collection runtime.
fn project(name: &str, reads: &str) -> PathBuf {
    let out = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("ess-ui-react-page-size")
        .join(name);
    if out.exists() {
        std::fs::remove_dir_all(&out).unwrap();
    }
    let document = ess_ui::load_str(&document(reads))
        .unwrap_or_else(|error| panic!("the document loads: {error}"));
    ess_ui_react::generate(&document, Path::new("."), &out).unwrap();
    support::compile(
        &out,
        &["runtime/core.tsx", "runtime/composites/collection.tsx"],
    );
    out
}

/// The `pager` prop the generated page hands its collection, as a JavaScript literal, or
/// `undefined` when it hands none.
fn pager_prop(out: &Path) -> String {
    let page = std::fs::read_to_string(out.join("src/pages/P.tsx")).unwrap();
    let start = page
        .find("<Collection")
        .unwrap_or_else(|| panic!("no collection in:\n{page}"));
    let element = &page[start..start + page[start..].find("/>").unwrap()];
    let Some(at) = element.find("pager={") else {
        return "undefined".to_owned();
    };
    let literal = &element[at + "pager={".len()..];
    let mut depth = 0usize;
    for (index, character) in literal.char_indices() {
        match character {
            '{' | '[' => depth += 1,
            '}' | ']' if depth == 0 => return literal[..index].to_owned(),
            '}' | ']' => depth -= 1,
            _ => {}
        }
    }
    panic!("an unterminated pager prop in {element}");
}

/// The harness every case runs: `mount(data)` renders the collection with the emitted `pager`
/// over `data`, under a scope whose `state` is `state` and records every change to it.
fn script(out: &Path, case: &str) -> String {
    format!(
        r#"
const {{ ScopeLayer, DataScope }} = out("runtime/core");
const {{ Collection }} = out("runtime/composites/collection");
const pager = {pager};
let state = {{ p: 1, n: 25 }};
// Each declared state has its own setter, as the generated page registers them; a write to any
// other state path lands in `stray`.
const stray = [];
let data;
function Harness() {{
  return jsx(ScopeLayer, {{
    values: {{ state }},
    setters: {{
      "state.p": (value) => {{ state = {{ ...state, p: value }}; }},
      "state.n": (value) => {{ state = {{ ...state, n: value }}; }},
      state: (next) => {{ stray.push(next); }},
    }},
    children: jsx(DataScope, {{
      data,
      children: jsx(Collection, {{
        "data-ui-path": "pages/p/sections/list",
        columns: [{{ "data-ui-path": "pages/p/sections/list/columns/id", field: "id", name: "id" }}],
        pager,
      }}),
    }}),
  }});
}}
const rows = (count) => Array.from({{ length: count }}, (_, index) => ({{ id: `r-${{index}}` }}));
const textOf = (node) => (typeof node === "string" ? node : (node.children || []).map(textOf).join(""));
let root;
function mount(next, at) {{
  data = {{ status: "ready", refetch() {{}}, params: {{}}, ...next }};
  if (at) state = {{ ...state, ...at }};
  root = React.__root(jsx(Harness, {{}})).settle();
  return root;
}}
const footer = () => root.find((node) => node.tag === "div" && node.props.className === "ui-pager")[0];
const button = (label) => {{
  const found = root.find((node) => node.tag === "button" && textOf(node) === label);
  assert.equal(found.length, 1, `one ${{label}} button: ${{JSON.stringify(root.tree)}}`);
  return found[0];
}};
const select = () => root.find((node) => node.tag === "select")[0];
{case}
"#,
        pager = pager_prop(out),
    )
}

const DECLARED: &str =
    "{view: t.All, params: {page: state.p, size: state.n}, paging: server, pager: {page: page, size: size, sizes: [10, 25, 50]}}";

#[test]
fn the_pager_sets_the_state_path_its_read_binds() {
    for (name, reads) in [
        (
            "declared",
            "{view: t.All, params: {page: state.p}, paging: server, pager: {page: page}}",
        ),
        ("sized", DECLARED),
    ] {
        let out = project(&format!("binds-{name}"), reads);
        support::node(
            &out,
            "binds",
            &script(
                &out,
                r#"
mount({ rows: rows(25), total: 60 }, { p: 2 });
assert.ok(footer(), "a declared pager renders its footer");
button("Next").props.onClick();
root.settle();
assert.equal(state.p, 3, "Next sets the state path the page param binds");
button("Previous").props.onClick();
root.settle();
assert.equal(state.p, 2, "Previous sets the state path the page param binds");
assert.deepEqual(stray, [], "no write to any other state path, such as state.page");
"#,
            ),
        );
    }
}

#[test]
fn a_pager_with_a_size_renders_a_per_page_select() {
    let out = project("select", DECLARED);
    support::node(
        &out,
        "select",
        &script(
            &out,
            r#"
mount({ rows: rows(25), total: 60 });
const labels = root.find((node) => node.tag === "label" && node.children.some((child) => typeof child === "object" && child.tag === "select"));
assert.equal(labels.length, 1, "the per-page select sits in a label");
assert.match(textOf(labels[0]), /Per page/, "the select is labelled");
const options = select().children.filter((child) => typeof child === "object" && child.tag === "option");
assert.deepEqual(options.map((option) => option.props.value), ["10", "25", "50"], "the select offers the pager's sizes");
assert.deepEqual(options.map(textOf), ["10", "25", "50"]);
assert.equal(select().props.value, "25", "the select shows the current size");
mount({ rows: rows(50), total: 60 }, { n: 50 });
assert.equal(select().props.value, "50", "the select follows the size's state path");
"#,
        ),
    );
    // Without `size`, the footer has no select.
    let out = project(
        "no-select",
        "{view: t.All, params: {page: state.p}, paging: server, pager: {page: page}}",
    );
    support::node(
        &out,
        "no-select",
        &script(
            &out,
            r#"
mount({ rows: rows(25), total: 60 });
assert.ok(footer(), "the footer renders");
assert.equal(select(), undefined, "a pager without a size has no per-page select");
"#,
        ),
    );
}

#[test]
fn choosing_a_size_sets_the_size_and_returns_to_page_one() {
    let out = project("choose", DECLARED);
    support::node(
        &out,
        "choose",
        &script(
            &out,
            r#"
mount({ rows: rows(25), total: 60 }, { p: 3 });
select().props.onChange({ target: { value: "50" } });
root.settle();
assert.equal(state.n, 50, "the select sets the size param's state path");
assert.equal(state.p, 1, "choosing a size returns to page one");
assert.deepEqual(stray, [], "no write to a hardcoded state.size or state.page");
"#,
        ),
    );
}

#[test]
fn next_is_disabled_on_the_last_page_when_the_total_is_known() {
    let out = project("last", DECLARED);
    support::node(
        &out,
        "last",
        &script(
            &out,
            r#"
mount({ rows: rows(10), total: 60 }, { p: 3, n: 25 });
assert.equal(button("Next").props.disabled, true, "page 3 of 60 rows at 25 is the last");
assert.match(textOf(footer()), /Page 3 of 3/, "the footer says where the reader is");
mount({ rows: rows(25), total: 60 }, { p: 1 });
assert.equal(button("Next").props.disabled, false, "page 1 is not the last");
assert.equal(button("Previous").props.disabled, true, "page 1 has no previous");
assert.match(textOf(footer()), /Page 1 of 3/);
mount({ rows: rows(25) }, { p: 1 });
assert.equal(button("Next").props.disabled, false, "without a total Next stays enabled");
assert.doesNotMatch(textOf(footer()), / of /, "without a total the footer names no page count");
"#,
        ),
    );
}

#[test]
fn a_declared_pager_always_shows_its_footer() {
    let out = project("always", DECLARED);
    support::node(
        &out,
        "always",
        &script(
            &out,
            r#"
mount({ rows: rows(5), total: 5 }, { p: 1 });
assert.ok(footer(), "the footer renders when the total fits one page");
assert.match(textOf(footer()), /Page 1 of 1/);
assert.equal(button("Next").props.disabled, true);
mount({ rows: [], total: 0 }, { p: 1 });
assert.ok(footer(), "the footer renders on an empty read");
mount({ rows: rows(5) }, { p: 1 });
assert.ok(footer(), "the footer renders without a total");
"#,
        ),
    );
}

#[test]
fn a_server_read_without_a_pager_keeps_todays_footer() {
    let out = project(
        "legacy",
        "{view: t.All, params: {page: state.p}, paging: server}",
    );
    assert_ne!(
        pager_prop(&out),
        "undefined",
        "a server read with a page param hands its collection the page's state path"
    );
    support::node(
        &out,
        "legacy",
        &script(
            &out,
            r#"
mount({ rows: rows(5), total: 5 }, { p: 1 });
assert.equal(footer(), undefined, "as before, no footer when the total fits the rows");
mount({ rows: rows(25) }, { p: 1 });
assert.equal(footer(), undefined, "as before, no footer without a total");
mount({ rows: rows(25), total: 60 }, { p: 2 });
assert.ok(footer(), "as before, a footer when the total exceeds the rows");
assert.equal(select(), undefined, "no per-page select without a pager");
assert.equal(textOf(footer()).replace(/\s+/g, " "), "PreviousPage 2Next", "today's footer: Previous, the page, Next");
button("Next").props.onClick();
root.settle();
assert.equal(state.p, 3, "Next sets the state path the page param binds");
button("Previous").props.onClick();
root.settle();
assert.equal(state.p, 2, "Previous sets the state path the page param binds");
assert.deepEqual(stray, [], "no write to any other state path, such as state.page");
"#,
        ),
    );
    // A read without a `page` param has no footer, as before.
    let out = project(
        "legacy-unpaged",
        "{view: t.All, params: {at: state.p}, paging: server}",
    );
    assert_eq!(
        pager_prop(&out),
        "undefined",
        "no page param, no pager prop"
    );
}
