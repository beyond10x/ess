//! `field: .` names the row itself: a read answering plain strings draws one row per string,
//! sorts by them and keys each row by its string (story:a-column-may-show-a-plain-string-row).
mod support;

use std::path::Path;

fn document(sections: &str) -> String {
    format!(
        "format: ess-ui/1
app: plain
model: t
placement_profile: fat
shells: {{app: {{regions: {{main: {{kind: page_outlet}}}}}}}}
navigation: {{home: p, sections: [{{name: all, pages: [p]}}]}}
pages:
  p:
    kind: detail_page
    title: Plain
    sections:
{sections}
"
    )
}

fn run(name: &str, sections: &str, body: &str) {
    let out = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("ess-ui-react-plain-row")
        .join(name);
    if out.exists() {
        std::fs::remove_dir_all(&out).unwrap();
    }
    let document = document(sections);
    let loaded = ess_ui::load_str(&document)
        .unwrap_or_else(|error| panic!("the document loads: {error}\n{document}"));
    ess_ui_react::generate(&loaded, Path::new("."), &out).unwrap();
    support::compile(&out, &["pages/P.tsx"]);
    support::node(
        &out,
        name,
        &format!(
            r#"
const nav = browser("/");
const {{ setDataAdapter }} = out("runtime/data");
const {{ PPage }} = out("pages/P");
const answers = {{ "t.Names": ["beta", "alpha"], "t.Item": {{ id: "i-1", label: "An object" }} }};
setDataAdapter({{
  read: async (request) => {{
    const answer = answers[request.view];
    return {{ rows: Array.isArray(answer) ? answer : [answer] }};
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
const rows = (section) => root.find((node) => node.tag === "tr" && typeof node.props["data-ui-path"] === "string" && node.props["data-ui-path"].startsWith(`pages/p/sections/${{section}}/rows/`));
(async () => {{
{body}
}})().catch((error) => {{ console.error(error); process.exitCode = 1; }});
"#
        ),
    );
}

#[test]
fn a_column_of_the_row_itself_draws_sorts_and_keys_plain_strings() {
    run(
        "names",
        "      - {name: list, component: collection, reads: t.Names, columns: [{field: ., name: name, label: Name, sortable: true}]}",
        r#"
  await settle();
  assert.deepEqual(rows("list").map(textOf), ["beta", "alpha"], "one row per string, as read");
  assert.deepEqual(
    rows("list").map((row) => row.props["data-ui-path"]),
    ["pages/p/sections/list/rows/beta", "pages/p/sections/list/rows/alpha"],
    "a plain row is keyed by itself",
  );
  const sort = root.find((node) => node.tag === "button" && node.props.className === "ui-sort")[0];
  assert.ok(sort, "the column sorts");
  sort.props.onClick({});
  await settle();
  assert.deepEqual(rows("list").map(textOf), ["alpha", "beta"], "sorting by the row orders the strings");
"#,
    );
}

#[test]
fn the_row_itself_is_absent_on_an_object_row() {
    run(
        "object",
        "      - {name: list, component: collection, reads: t.Item, columns: [label, {field: ., name: whole}]}",
        r#"
  await settle();
  const cells = rows("list")[0].children.filter((child) => typeof child === "object" && child.tag === "td").map(textOf);
  assert.equal(cells[0], "An object");
  assert.ok(!cells[1].includes("An object") && !cells[1].includes("object Object") && !cells[1].includes("i-1"), `an object row reads as absent: ${JSON.stringify(cells)}`);
"#,
    );
}
