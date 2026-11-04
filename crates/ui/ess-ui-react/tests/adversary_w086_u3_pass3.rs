//! Adversary pass 3 on story:a-server-paged-collection-has-a-page-size-control: fix round 2's
//! held-value entry of the pager, driven end to end. Each case generates a whole page, renders
//! it under node with a data adapter that records every read it is asked, and works the footer
//! the way a reader would: the states are the page's real stores (URL query parameters).
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
        .join("ess-ui-react-adversary-w086-u3-pass3")
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
const entries = () => options().map((option) => [option.props.value, textOf(option)]);
const query = (key) => new URL(nav.path, "http://app.test").searchParams.get(key);
"#
    )
}

/// An integer page state and an integer size state, both in the URL, sizes 25 and 50.
fn url_states() -> String {
    document(
        "      p: {type: integer, class: page_state, store: url, default: 1}\n      n: {type: integer, class: page_state, store: url, default: 25}",
        "{view: t.All, params: {page: state.p, size: state.n}, paging: server, pager: {page: page, size: size, sizes: [25, 50]}}",
        "",
    )
}

/// A held 0: its own entry, no "of N", Next enabled; choosing it writes nothing; Next keeps it;
/// choosing 50 sends 50 from page 1 and drops the entry; Back brings both the 0 and the page back.
#[test]
fn adversary_a_held_zero_through_next_a_real_size_and_back() {
    run(
        "zero-round-trip",
        &url_states(),
        "/?n=0",
        r#"
  await settle();
  assert.deepEqual(last(), { page: 1, size: 0 }, JSON.stringify(reads));
  assert.equal(select().props.value, "0");
  assert.deepEqual(entries(), [["0", "0"], ["25", "25"], ["50", "50"]]);
  assert.equal(textOf(footer()).includes(" of "), false, textOf(footer()));
  assert.equal(button("Next").props.disabled, false, "Next stays enabled while the size counts no pages");
  const calls = nav.calls.length;
  select().props.onChange({ target: { value: "0" } });
  await settle();
  assert.equal(nav.calls.length, calls, `choosing the held 0 navigates: ${JSON.stringify(nav.calls)}`);
  assert.deepEqual(last(), { page: 1, size: 0 }, JSON.stringify(reads));
  button("Next").props.onClick({});
  await settle();
  assert.deepEqual(last(), { page: 2, size: 0 }, `Next keeps the held 0: ${JSON.stringify(reads)}; at ${nav.path}`);
  assert.equal(select().props.value, "0");
  assert.equal(textOf(footer()).includes("Page 2"), true, textOf(footer()));
  select().props.onChange({ target: { value: "50" } });
  await settle();
  assert.deepEqual(last(), { page: 1, size: 50 }, `after choosing 50: ${JSON.stringify(reads)}; at ${nav.path}`);
  assert.deepEqual(entries(), [["25", "25"], ["50", "50"]], "the held entry is gone once a size is held");
  assert.equal(textOf(footer()).includes("Page 1 of 3"), true, textOf(footer()));
  assert.equal(query("n"), "50", nav.path);
  nav.pop("/?n=0&p=2");
  await settle();
  assert.equal(select().props.value, "0", `Back shows the held 0 again: ${textOf(footer())}`);
  assert.deepEqual(entries(), [["0", "0"], ["25", "25"], ["50", "50"]]);
  assert.deepEqual(last(), { page: 2, size: 0 }, JSON.stringify(reads));
"#,
    );
}

/// A held negative: its own entry; choosing the state's default size (25) clears the query
/// parameter and the select shows 25 from page 1.
#[test]
fn adversary_a_held_negative_then_the_default_size() {
    run(
        "negative",
        &url_states(),
        "/?n=-5&p=4",
        r#"
  await settle();
  assert.deepEqual(last(), { page: 4, size: -5 }, JSON.stringify(reads));
  assert.equal(select().props.value, "-5");
  assert.deepEqual(entries(), [["-5", "-5"], ["25", "25"], ["50", "50"]]);
  select().props.onChange({ target: { value: "25" } });
  await settle();
  assert.deepEqual(last(), { page: 1, size: 25 }, `after choosing 25: ${JSON.stringify(reads)}; at ${nav.path}`);
  assert.equal(query("n"), null, `the default size leaves the query: ${nav.path}`);
  assert.equal(select().props.value, "25");
  assert.deepEqual(entries(), [["25", "25"], ["50", "50"]]);
  assert.equal(textOf(footer()).includes("Page 1 of 5"), true, textOf(footer()));
"#,
    );
}

/// Text in an integer state through the URL: its own entry; choosing it writes nothing; Next
/// keeps the text in the URL and in the read.
#[test]
fn adversary_held_text_survives_next() {
    run(
        "text",
        &url_states(),
        "/?n=abc&p=3",
        r#"
  await settle();
  assert.deepEqual(last(), { page: 3, size: "abc" }, JSON.stringify(reads));
  assert.equal(select().props.value, "abc");
  assert.deepEqual(entries(), [["abc", "abc"], ["25", "25"], ["50", "50"]]);
  const calls = nav.calls.length;
  select().props.onChange({ target: { value: "abc" } });
  await settle();
  assert.equal(nav.calls.length, calls, JSON.stringify(nav.calls));
  button("Next").props.onClick({});
  await settle();
  assert.deepEqual(last(), { page: 4, size: "abc" }, `${JSON.stringify(reads)}; at ${nav.path}`);
  assert.equal(query("n"), "abc", nav.path);
"#,
    );
}

/// JSON the URL decodes to something other than a number: an array and a boolean get an entry
/// of their own text; `null` and an empty parameter hold nothing and show "Default".
#[test]
fn adversary_decoded_non_numbers_and_nothing() {
    for (name, at, sent, shown) in [
        ("array", "/?n=%5B25%5D", "[25]", "[25]"),
        ("boolean", "/?n=true", "true", "true"),
        ("null", "/?n=null", "", ""),
        ("empty", "/?n=", "", ""),
    ] {
        let label = if shown.is_empty() { "Default" } else { shown };
        let sent = if sent.is_empty() {
            String::new()
        } else {
            format!("assert.deepEqual(last().size, {sent}, JSON.stringify(reads));")
        };
        run(
            name,
            &url_states(),
            at,
            &format!(
                r#"
  await settle();
  {sent}
  assert.equal(select().props.value, {shown:?});
  assert.deepEqual(entries(), [[{shown:?}, {label:?}], ["25", "25"], ["50", "50"]]);
  assert.equal(textOf(footer()).includes(" of "), false, textOf(footer()));
  select().props.onChange({{ target: {{ value: "50" }} }});
  await settle();
  assert.deepEqual(last(), {{ page: 1, size: 50 }}, JSON.stringify(reads));
  assert.deepEqual(entries(), [["25", "25"], ["50", "50"]]);
"#
            ),
        );
    }
}
