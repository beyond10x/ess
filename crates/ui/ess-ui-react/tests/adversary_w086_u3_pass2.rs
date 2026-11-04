//! Adversary pass 2 on story:a-server-paged-collection-has-a-page-size-control: the fix round's
//! pager, driven end to end. Each case generates a whole page, renders it under node with a data
//! adapter that records every read it is asked, and works the footer the way a reader would: the
//! states are the page's real stores (URL query parameters here), not a hand-built scope.
mod support;

use std::path::Path;

/// A page `p` whose `list` section reads `t.All` as written in `reads`, with the page's `state`
/// entries as written in `state` (each line indented six spaces) and `widgets` written at the
/// document root when given.
fn document(state: &str, reads: &str, widgets: &str) -> String {
    format!(
        "format: ess-ui/1
app: pager
model: t
placement_profile: fat
shells: {{app: {{regions: {{main: {{kind: page_outlet}}}}}}}}
navigation: {{home: p, sections: [{{name: all, pages: [p]}}]}}
{widgets}
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

/// Generates and compiles the page, then runs `body` (inside an async function) after the
/// harness below, with the browser at `at`.
fn run(name: &str, document: &str, at: &str, body: &str) {
    let out = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("ess-ui-react-adversary-w086-u3-pass2")
        .join(name);
    if out.exists() {
        std::fs::remove_dir_all(&out).unwrap();
    }
    let loaded = ess_ui::load_str(document)
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

/// The browser at `at`, an adapter that answers every read with `answer.rows` rows of
/// `answer.total` and records each request in `reads`, the page, and footer helpers.
fn harness(at: &str) -> String {
    format!(
        r#"
const nav = browser({at:?});
const {{ setDataAdapter }} = out("runtime/data");
const {{ PPage }} = out("pages/P");
const reads = [];
const answer = {{ rows: 25, total: 120 }};
setDataAdapter({{
  read: async (request) => {{
    reads.push(JSON.parse(JSON.stringify(request.params)));
    return {{ rows: Array.from({{ length: answer.rows }}, (_, index) => ({{ id: `r-${{index}}` }})), total: answer.total }};
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
const button = (label) => root.find((node) => node.tag === "button" && textOf(node) === label)[0];
const select = () => root.find((node) => node.tag === "select")[0];
const options = () => select().children.filter((child) => typeof child === "object" && child.tag === "option");
const last = () => reads[reads.length - 1];
"#
    )
}

/// Page and size in two URL states: the select writes both through two query updates, and the
/// second must not drop the first.
#[test]
fn adversary_two_url_states_keep_both_writes_when_a_size_is_chosen() {
    let text = document(
        "      p: {type: integer, class: page_state, store: url, default: 1}\n      n: {type: integer, class: page_state, store: url, default: 25}",
        "{view: t.All, params: {page: state.p, size: state.n}, paging: server, pager: {page: page, size: size, sizes: [25, 50]}}",
        "",
    );
    run(
        "two-url",
        &text,
        "/?p=3",
        r#"
  await settle();
  assert.deepEqual(last(), { page: 3, size: 25 }, `the first read: ${JSON.stringify(reads)}`);
  select().props.onChange({ target: { value: "50" } });
  await settle();
  assert.deepEqual(last(), { page: 1, size: 50 }, `after choosing 50: ${JSON.stringify(reads)}; at ${nav.path}`);
  assert.equal(select().props.value, "50", "the select shows 50");
  assert.equal(new URL(nav.path, "http://app.test").searchParams.get("n"), "50", `the URL holds the size: ${nav.path}`);
"#,
    );
}

/// Page and size at different depths of one record state held in the URL:
/// `state.q.page` and `state.q.inner.size`.
#[test]
fn adversary_page_and_size_at_different_depths_of_one_url_state() {
    let text = document(
        "      q: {type: {record: {page: integer, inner: {record: {size: integer}}}}, class: page_state, store: url, default: {page: 3, inner: {size: 25}}}",
        "{view: t.All, params: {page: state.q.page, size: state.q.inner.size}, paging: server, pager: {page: page, size: size, sizes: [25, 50]}}",
        "",
    );
    run(
        "depths",
        &text,
        "/",
        r#"
  await settle();
  assert.deepEqual(last(), { page: 3, size: 25 }, `the first read: ${JSON.stringify(reads)}`);
  select().props.onChange({ target: { value: "50" } });
  await settle();
  assert.deepEqual(last(), { page: 1, size: 50 }, `after choosing 50: ${JSON.stringify(reads)}; at ${nav.path}`);
  button("Next").props.onClick({});
  await settle();
  assert.deepEqual(last(), { page: 2, size: 50 }, `Next keeps the size: ${JSON.stringify(reads)}; at ${nav.path}`);
"#,
    );
}

/// The size state holds nothing: "Default" is shown and the read sends no size. Choosing a
/// size sends it and drops "Default"; choosing "Default" while it is shown writes nothing; the
/// browser's Back returns to the unset size and "Default" comes back.
#[test]
fn adversary_the_default_option_round_trip() {
    let text = document(
        "      p: {type: integer, class: page_state, store: url, default: 1}\n      n: {type: {optional: integer}, class: page_state, store: url}",
        "{view: t.All, params: {page: state.p, size: state.n}, paging: server, pager: {page: page, size: size, sizes: [25, 50]}}",
        "",
    );
    run(
        "default",
        &text,
        "/?p=2",
        r#"
  await settle();
  assert.equal(last().size, undefined, `no size is sent: ${JSON.stringify(reads)}`);
  assert.equal(select().props.value, "", "the select shows the default");
  assert.deepEqual(options().map((option) => [option.props.value, textOf(option)]), [["", "Default"], ["25", "25"], ["50", "50"]]);
  const calls = nav.calls.length;
  select().props.onChange({ target: { value: "" } });
  await settle();
  assert.equal(nav.calls.length, calls, `choosing Default navigates nowhere: ${JSON.stringify(nav.calls)}`);
  assert.equal(last().page, 2, "choosing Default keeps the page");
  select().props.onChange({ target: { value: "50" } });
  await settle();
  assert.deepEqual(last(), { page: 1, size: 50 }, `after choosing 50: ${JSON.stringify(reads)}`);
  assert.deepEqual(options().map((option) => option.props.value), ["25", "50"], "Default is gone once a size is held");
  nav.pop("/?p=2");
  await settle();
  assert.equal(select().props.value, "", `Back to no size shows Default again; footer: ${textOf(footer())}`);
  assert.equal(last().size, undefined, `and the read sends no size: ${JSON.stringify(reads)}`);
"#,
    );
}

/// A shared link holds `n=0`: the read sends size 0, so the select must not call it "Default"
/// (the template's doc comment: "so it always shows what the read sends").
#[test]
fn adversary_a_zero_size_in_the_url_is_not_shown_as_the_default() {
    let text = document(
        "      p: {type: integer, class: page_state, store: url, default: 1}\n      n: {type: integer, class: page_state, store: url, default: 25}",
        "{view: t.All, params: {page: state.p, size: state.n}, paging: server, pager: {page: page, size: size, sizes: [25, 50]}}",
        "",
    );
    run(
        "zero",
        &text,
        "/?n=0",
        r#"
  await settle();
  assert.equal(last().size, 0, `the read sends size 0: ${JSON.stringify(reads)}`);
  const shown = options().find((option) => option.props.value === select().props.value);
  assert.notEqual(textOf(shown), "Default", `the read sends size 0 and the select calls it ${JSON.stringify(textOf(shown))}; options ${JSON.stringify(options().map(textOf))}`);
"#,
    );
}

/// Numeric strings and fractions reach the size state through the URL: `1.5` into an integer
/// state, ` 25` and `50` into a string state. The select shows the number the read sends and
/// lists each size once.
#[test]
fn adversary_numeric_strings_show_one_entry_per_size() {
    let fractional = document(
        "      p: {type: integer, class: page_state, store: url, default: 1}\n      n: {type: integer, class: page_state, store: url, default: 25}",
        "{view: t.All, params: {page: state.p, size: state.n}, paging: server, pager: {page: page, size: size, sizes: [25, 50]}}",
        "",
    );
    run(
        "fraction",
        &fractional,
        "/?n=1.5",
        r#"
  await settle();
  assert.equal(last().size, 1.5, JSON.stringify(reads));
  assert.equal(select().props.value, "1.5");
  assert.deepEqual(options().map((option) => option.props.value), ["1.5", "25", "50"]);
"#,
    );
    let text = document(
        "      p: {type: integer, class: page_state, store: url, default: 1}\n      n: {type: string, class: page_state, store: url, default: '50'}",
        "{view: t.All, params: {page: state.p, size: state.n}, paging: server, pager: {page: page, size: size, sizes: [25, 50]}}",
        "",
    );
    run(
        "string-fifty",
        &text,
        "/",
        r#"
  await settle();
  assert.equal(last().size, "50", JSON.stringify(reads));
  assert.equal(select().props.value, "50");
  assert.deepEqual(options().map((option) => option.props.value), ["25", "50"], "50 held as a string is listed once");
  assert.equal(textOf(footer()).includes("of 3"), true, textOf(footer()));
"#,
    );
    run(
        "string-padded",
        &text,
        "/?n=%2025",
        r#"
  await settle();
  assert.equal(last().size, " 25", JSON.stringify(reads));
  assert.equal(select().props.value, "25");
  assert.deepEqual(options().map((option) => option.props.value), ["25", "50"], "' 25' is listed once, as 25");
"#,
    );
}

/// A server read with a `page` param bound to a nested path and no pager keeps today's footer,
/// writing that nested path and leaving the record's other member alone.
#[test]
fn adversary_the_legacy_footer_writes_a_nested_page_path() {
    let text = document(
        "      q: {type: {record: {page: integer, size: integer}}, class: page_state, store: url, default: {page: 1, size: 25}}",
        "{view: t.All, params: {page: state.q.page, size: state.q.size}, paging: server}",
        "",
    );
    run(
        "legacy",
        &text,
        "/",
        r#"
  answer.total = 60;
  await settle();
  assert.ok(footer(), `the legacy footer shows: ${root.text()}`);
  assert.equal(select(), undefined, "no size select without a pager");
  button("Next").props.onClick({});
  await settle();
  assert.deepEqual(last(), { page: 2, size: 25 }, `Next writes state.q.page: ${JSON.stringify(reads)}; at ${nav.path}`);
  answer.total = 20;
  answer.rows = 20;
  button("Previous").props.onClick({});
  await settle();
  assert.equal(footer(), undefined, `the legacy footer hides when the total fits: ${root.text()}`);
"#,
    );
}

/// A collection with a pager declared inside a widget, its params bound through the widget's
/// args to the page's states: the expanded body pages on the use site's state.
#[test]
fn adversary_a_pager_in_a_widget_body_pages_the_use_sites_state() {
    let widgets = "widgets:
  paged:
    summary: A paged list.
    params:
      page: {type: integer, required: true, note: the page}
      size: {type: integer, required: true, note: the size}
    body:
      - {name: rows, component: collection, reads: {view: t.All, params: {page: args.page, size: args.size}, paging: server, pager: {page: page, size: size, sizes: [25, 50]}}, columns: [id]}";
    let text = format!(
        "format: ess-ui/1
app: pager
model: t
placement_profile: fat
shells: {{app: {{regions: {{main: {{kind: page_outlet}}}}}}}}
navigation: {{home: p, sections: [{{name: all, pages: [p]}}]}}
{widgets}
pages:
  p:
    kind: detail_page
    title: Paged
    state:
      p: {{type: integer, class: page_state, store: url, default: 1}}
      n: {{type: integer, class: page_state, store: url, default: 25}}
    sections:
      - {{name: summary, reads: t.ById, children: [{{name: use, component: paged, args: {{page: state.p, size: state.n}}}}]}}
"
    );
    run(
        "widget",
        &text,
        "/?p=2",
        r#"
  await settle();
  assert.ok(footer(), `the widget's collection shows its pager: ${root.text()}`);
  assert.deepEqual(last(), { page: 2, size: 25 }, JSON.stringify(reads));
  select().props.onChange({ target: { value: "50" } });
  await settle();
  assert.deepEqual(last(), { page: 1, size: 50 }, `${JSON.stringify(reads)}; at ${nav.path}`);
"#,
    );
}
