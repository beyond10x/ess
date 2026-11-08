//! A client-paged collection offers page sizes, shows a from-to range and returns to its first
//! page when its search changes; every pager with a known size shows the range
//! (story:a-client-paged-collection-offers-sizes-and-a-range).
//!
//! Each case generates a whole page and renders it under node with a data adapter that answers
//! every read with `answer.rows` rows, `keep-<n>` for the first `answer.kept` and `other-<n>`
//! after them, and records each request it is asked.
mod support;

use std::path::Path;

/// A page `p` whose `list` section reads `t.All` as written in `reads`, with a URL search state
/// `q` and integer URL states `p` and `n`.
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
      q: {{type: string, class: page_state, store: url}}
      p: {{type: integer, class: page_state, store: url, default: 1}}
      n: {{type: integer, class: page_state, store: url, default: 10}}
    sections:
      - {{name: list, component: collection, reads: {reads}, columns: [name]}}
"
    )
}

/// Generates and compiles the page, then runs `body` (inside an async function) after the
/// harness, with the browser at `at`.
fn run(name: &str, reads: &str, at: &str, body: &str) {
    let out = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("ess-ui-react-client-pager")
        .join(name);
    if out.exists() {
        std::fs::remove_dir_all(&out).unwrap();
    }
    let document = document(reads);
    let loaded = ess_ui::load_str(&document)
        .unwrap_or_else(|error| panic!("the document loads: {error}\n{document}"));
    ess_ui_react::generate(&loaded, Path::new("."), &out).unwrap();
    support::compile(&out, &["pages/P.tsx"]);
    support::node(
        &out,
        name,
        &format!(
            "{}\n(async () => {{\n{body}\n}})().catch((error) => {{ console.error(error); process.exitCode = 1; }});\n",
            harness(at)
        ),
    );
}

fn harness(at: &str) -> String {
    format!(
        r#"
const nav = browser({at:?});
const {{ setDataAdapter }} = out("runtime/data");
const {{ PPage }} = out("pages/P");
const reads = [];
const answer = {{ rows: 30, kept: 4, total: undefined }};
setDataAdapter({{
  read: async (request) => {{
    reads.push(JSON.parse(JSON.stringify(request.params)));
    const rows = Array.from({{ length: answer.rows }}, (_, index) => ({{ id: `r-${{index}}`, name: index < answer.kept ? `keep-${{index}}` : `other-${{index}}` }}));
    return answer.total === undefined ? {{ rows }} : {{ rows, total: answer.total }};
  }},
  send: async () => ({{ accepted: true }}),
  loadState: async () => undefined,
  saveState: async () => undefined,
}});
const {{ Router }} = out("runtime/router");
const root = React.__root(jsx(Router, {{ children: jsx(PPage, {{}}) }}));
const tick = () => new Promise((resolve) => setTimeout(resolve, 5));
async function settle() {{ for (let i = 0; i < 6; i++) {{ root.settle(); await tick(); }} root.settle(); }}
const textOf = (node) => (typeof node === "string" ? node : (node.children || []).map(textOf).join(""));
const footer = () => root.find((node) => node.tag === "div" && node.props.className === "ui-pager")[0];
const range = () => {{
  const found = root.find((node) => node.props && node.props.className === "ui-pager-range");
  return found.length === 0 ? undefined : textOf(found[0]);
}};
const button = (label) => root.find((node) => node.tag === "button" && textOf(node) === label)[0];
const select = () => root.find((node) => node.tag === "select")[0];
const options = () => select().children.filter((child) => typeof child === "object" && child.tag === "option");
const shown = () => root.find((node) => node.tag === "tr" && node.props["data-ui-path"] !== undefined && /\/rows\//.test(node.props["data-ui-path"])).map(textOf);
"#
    )
}

const CLIENT: &str =
    "{view: t.All, filter: row.name contains state.q, paging: client, pager: {sizes: [10, 25]}}";

#[test]
fn a_client_pager_draws_a_page_of_the_rows_and_its_range() {
    run(
        "sizes",
        CLIENT,
        "/",
        r#"
  await settle();
  assert.ok(footer(), "a client pager renders its footer");
  assert.equal(shown().length, 10, `the first size pages the rows: ${JSON.stringify(shown())}`);
  assert.equal(range(), "1-10 of 30", textOf(footer()));
  assert.deepEqual(options().map((option) => option.props.value), ["10", "25"], "the select offers the pager's sizes");
  assert.equal(select().props.value, "10", "the first size stands selected");
  select().props.onChange({ target: { value: "25" } });
  await settle();
  assert.equal(shown().length, 25, `after choosing 25: ${JSON.stringify(shown())}`);
  assert.equal(range(), "1-25 of 30", textOf(footer()));
  assert.equal(select().props.value, "25");
  button("Next").props.onClick({});
  await settle();
  assert.equal(shown().length, 5, "the last page holds the rest");
  assert.equal(range(), "26-30 of 30", textOf(footer()));
  assert.equal(button("Next").props.disabled, true, "Next is disabled on the last page");
  assert.equal(reads.length, 1, `paging on the client reads once: ${JSON.stringify(reads)}`);
  assert.equal(new URL(nav.path, "http://app.test").searchParams.get("p"), null, "a client pager writes no page state");
"#,
    );
}

#[test]
fn a_search_returns_a_client_paged_collection_to_its_first_page() {
    run(
        "reset",
        CLIENT,
        "/",
        r#"
  await settle();
  button("Next").props.onClick({});
  await settle();
  button("Next").props.onClick({});
  await settle();
  assert.equal(range(), "21-30 of 30", textOf(footer()));
  assert.match(textOf(footer()), /Page 3 of 3/);
  nav.pop("/?q=keep");
  await settle();
  assert.deepEqual(shown(), ["keep-0", "keep-1", "keep-2", "keep-3"], "the search keeps four rows, on page 1");
  assert.match(textOf(footer()), /Page 1 of 1/);
  assert.equal(range(), "1-4 of 4", textOf(footer()));
  nav.pop("/");
  await settle();
  assert.equal(range(), "1-10 of 30", "clearing the search starts at page 1 again");
"#,
    );
}

#[test]
fn a_server_pager_with_a_known_size_shows_its_range() {
    const SERVER: &str = "{view: t.All, params: {page: state.p, size: state.n}, paging: server, pager: {page: page, size: size, sizes: [10, 25]}}";
    run(
        "server",
        SERVER,
        "/?p=4",
        r#"
  answer.rows = 4;
  answer.total = 34;
  await settle();
  assert.equal(range(), "31-34 of 34", textOf(footer()));
  assert.match(textOf(footer()), /Page 4 of 4/);
"#,
    );
    run(
        "server-unsized",
        "{view: t.All, params: {page: state.p}, paging: server, pager: {page: page}}",
        "/?p=2",
        r#"
  answer.rows = 10;
  answer.total = 34;
  await settle();
  assert.ok(footer(), "a declared pager shows its footer");
  assert.equal(range(), undefined, "without a size the range is unknown");
"#,
    );
}
