//! Adversary pass 1 on story:a-server-paged-collection-has-a-page-size-control: documents the
//! pager check accepts, rendered through the generated collection runtime.
mod support;

use std::path::{Path, PathBuf};

/// A page whose `list` section reads `t.All` as written in `reads`, with the page's `state`
/// entries as written in `state`.
fn document(state: &str, reads: &str) -> String {
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
{state}
    sections:
      - {{name: list, component: collection, reads: {reads}, columns: [id]}}
"
    )
}

fn project(name: &str, state: &str, reads: &str) -> PathBuf {
    let out = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("ess-ui-react-adversary-w086-u3")
        .join(name);
    if out.exists() {
        std::fs::remove_dir_all(&out).unwrap();
    }
    let document = ess_ui::load_str(&document(state, reads))
        .unwrap_or_else(|error| panic!("the document loads: {error}"));
    ess_ui_react::generate(&document, Path::new("."), &out).unwrap();
    support::compile(
        &out,
        &["runtime/core.tsx", "runtime/composites/collection.tsx"],
    );
    out
}

/// The `pager` prop the generated page hands its collection, as a JavaScript literal.
fn pager_prop(out: &Path) -> String {
    let page = std::fs::read_to_string(out.join("src/pages/P.tsx")).unwrap();
    let start = page
        .find("<Collection")
        .unwrap_or_else(|| panic!("no collection in:\n{page}"));
    let element = &page[start..start + page[start..].find("/>").unwrap()];
    let at = element
        .find("pager={")
        .unwrap_or_else(|| panic!("no pager prop in {element}"));
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

/// Mounts the collection with the emitted pager under a scope holding `initial` as `state`,
/// with one setter per top-level state named in `roots`, as a generated page registers them.
fn script(out: &Path, initial: &str, roots: &[&str], case: &str) -> String {
    let setters: String = roots
        .iter()
        .map(|root| {
            format!("      \"state.{root}\": (value) => {{ state = {{ ...state, {root}: value }}; }},\n")
        })
        .collect::<Vec<_>>()
        .concat();
    format!(
        r#"
const {{ ScopeLayer, DataScope }} = out("runtime/core");
const {{ Collection }} = out("runtime/composites/collection");
const pager = {pager};
let state = {initial};
const stray = [];
let data;
function Harness() {{
  return jsx(ScopeLayer, {{
    values: {{ state }},
    setters: {{
{setters}      state: (next) => {{ stray.push(next); }},
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
function mount(next) {{
  data = {{ status: "ready", refetch() {{}}, params: {{}}, ...next }};
  root = React.__root(jsx(Harness, {{}})).settle();
  return root;
}}
const footer = () => root.find((node) => node.tag === "div" && node.props.className === "ui-pager")[0];
const button = (label) => root.find((node) => node.tag === "button" && textOf(node) === label)[0];
const select = () => root.find((node) => node.tag === "select")[0];
{case}
"#,
        pager = pager_prop(out),
    )
}

/// The page and the size bound to two fields of one record state, as the example documents
/// bind `state.window.from` and `state.window.to`. The check accepts it (`state_path` takes
/// dotted paths); choosing a size must leave both writes standing.
#[test]
fn adversary_choosing_a_size_keeps_the_size_when_both_paths_share_one_state() {
    let out = project(
        "record",
        "      q: {type: {record: {page: integer, size: integer}}, class: page_state, store: url, default: {page: 1, size: 25}}",
        "{view: t.All, params: {page: state.q.page, size: state.q.size}, paging: server, pager: {page: page, size: size, sizes: [25, 50]}}",
    );
    support::node(
        &out,
        "record",
        &script(
            &out,
            "{ q: { page: 3, size: 25 } }",
            &["q"],
            r#"
mount({ rows: rows(25), total: 120 });
select().props.onChange({ target: { value: "50" } });
root.settle();
assert.deepEqual(state.q, { page: 1, size: 50 }, "choosing 50 sets state.q.size and returns state.q.page to 1");
"#,
        ),
    );
}

/// The size state's default is not among `sizes` (here the schema default `[25, 50, 100]`).
/// A controlled select whose value matches no option shows its first option in a browser, so
/// the reader sees 25 while the read sends 20, and picking 25 fires no change.
#[test]
fn adversary_the_select_offers_the_size_the_read_sends() {
    let out = project(
        "off-list",
        "      p: {type: integer, class: page_state, store: url, default: 1}\n      n: {type: integer, class: page_state, store: url, default: 20}",
        "{view: t.All, params: {page: state.p, size: state.n}, paging: server, pager: {page: page, size: size}}",
    );
    support::node(
        &out,
        "off-list",
        &script(
            &out,
            "{ p: 1, n: 20 }",
            &["p", "n"],
            r#"
mount({ rows: rows(20), total: 60 });
const options = select().children.filter((child) => typeof child === "object" && child.tag === "option").map((option) => option.props.value);
assert.ok(options.includes(select().props.value), `the select's value ${select().props.value} is one of its options ${JSON.stringify(options)}, so it shows the current size`);
"#,
        ),
    );
}

/// The size state has no default, so the read sends no size and the server pages by its own.
/// The footer must not count pages with a size the read never sent.
#[test]
fn adversary_an_unset_size_does_not_disable_next() {
    let out = project(
        "unset",
        "      p: {type: integer, class: page_state, store: url, default: 1}\n      n: {type: integer, class: page_state, store: url}",
        "{view: t.All, params: {page: state.p, size: state.n}, paging: server, pager: {page: page, size: size}}",
    );
    support::node(
        &out,
        "unset",
        &script(
            &out,
            "{ p: 3, n: null }",
            &["p", "n"],
            r#"
// The server answered page 3 with its own 10 rows of 60: pages 4 to 6 remain.
mount({ rows: rows(10), total: 60 });
assert.equal(button("Next").props.disabled, false, `Next stays enabled when the size the read sent is unknown; footer: ${textOf(footer())}`);
"#,
        ),
    );
}
