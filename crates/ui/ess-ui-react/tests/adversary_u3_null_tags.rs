//! Adversary, wave 0.90.0 unit 3 (story:a-null-tag-draws-no-chip): a `null` entry of an
//! `as: tags` value carries no tag. The display drops it; these cases drive the same value through
//! the tags INPUT, which an edit form fills from the record it loads (`{...row, ...draft}` in
//! `form.tsx`), and pin the display cases the unit's own test does not name.
//!
//! Each case renders the generated runtime's `FieldValue` or `FieldInput` under `node`.

mod support;

use std::path::{Path, PathBuf};

use support::{compile, node};

const DOCUMENT: &str = r#"
format: ess-ui/1
app: t
model: t.system
placement_profile: fat
shells:
  app: {regions: {main: {kind: page_outlet}}}
navigation:
  home: runs
  sections: [{name: all, pages: [runs]}]
pages:
  runs:
    kind: list_page
    title: Runs
    sections:
      - name: add
        component: form
        does: t.AddRun
        fields:
          - {field: tags, label: Tags, as: tags, suggests: {reads: {view: t.Tags}, value: name}}
"#;

fn scratch(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("ess-ui-react-adversary-u3-null-tags")
        .join(name);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("the old scratch dir is removed");
    }
    std::fs::create_dir_all(&dir).expect("the scratch dir is created");
    dir
}

fn generate(name: &str) -> PathBuf {
    let dir = scratch(name);
    let document = ess_ui::load_str(DOCUMENT).unwrap_or_else(|error| panic!("{error}"));
    let project = dir.join("app");
    ess_ui_react::generate(&document, &dir, &project)
        .unwrap_or_else(|error| panic!("the document generates: {error}"));
    project
}

const RENDER: &str = r#"
const { FieldInput, FieldValue } = out("runtime/fields");
const { ScopeLayer } = out("runtime/core");
const { setDataAdapter } = out("runtime/data");
let rows = [];
setDataAdapter({
  read: async (request) => ({ rows: request.view === "t.Tags" ? rows : [], total: rows.length }),
  send: async () => ({ accepted: true }),
  loadState: async () => undefined,
  saveState: async () => undefined,
});
let held;
function Host(props) {
  const [draft, setDraft] = React.useState(props.draft);
  held = draft;
  return jsx(ScopeLayer, { values: { draft }, setters: { draft: setDraft }, children: jsx(FieldInput, { field: props.field }) });
}
const tick = () => new Promise((resolve) => setTimeout(resolve, 5));
async function host(field, draft) {
  const root = React.__root(jsx(Host, { field, draft }));
  for (let i = 0; i < 4; i++) { root.settle(); await tick(); }
  return root.settle();
}
const draft = () => held;
function textOf(node) {
  if (typeof node === "string") return node;
  return (node.children || []).map(textOf).join("");
}
const suggestions = (root) => root.find((n) => n.tag === "button" && /\bui-tag-suggestion\b/.test(String(n.props.className)));
const input = (root) => root.find((n) => n.tag === "input")[0];
const field = { "data-ui-path": "pages/runs/sections/add/fields/tags", field: "tags", name: "tags", as: "tags", suggests: { reads: { view: "t.Tags" }, value: "name" } };
const shown = (f, row) => React.__root(jsx(FieldValue, { field: f, row })).settle();
const chips = (root) => root.find((n) => n.tag === "span" && n.props.className === "ui-tag").map(textOf);
"#;

fn render(project: &Path, name: &str, script: &str) {
    compile(project, &["runtime/fields.tsx"]);
    node(
        project,
        name,
        &format!(
            "{RENDER}\n(async () => {{\n{script}\n}})().catch((error) => {{ console.error(error); process.exitCode = 1; }});\n"
        ),
    );
}

/// An edit form loads a record whose tags hold `null` and the user picks a suggestion: the input
/// writes `[...held, suggestion]` with `held = value.map(String)`, so the null becomes the text
/// "null", which is sent and then drawn as a chip reading "null" - the chip the story removes.
#[test]
fn a_suggestion_picked_beside_a_held_null_tag_writes_no_text_null() {
    let project = generate("pick");
    render(
        &project,
        "pick",
        r#"
rows = [{ name: "x" }];
let root = await host(field, { tags: ["a", null, "b"] });
assert.deepStrictEqual(suggestions(root).map(textOf), ["x"]);
suggestions(root)[0].props.onClick();
root = root.settle();
assert.ok(!draft().tags.includes("null"), "a held null tag is written back as the text \"null\": " + JSON.stringify(draft().tags));
assert.deepStrictEqual(chips(shown(field, { tags: draft().tags })), ["a", "b", "x"]);
"#,
    );
}

/// The same record in the tags input shows an empty slot for the null entry ("a, , b"); the
/// display beside it shows two chips.
#[test]
fn a_held_null_tag_shows_no_empty_slot_in_the_tags_input() {
    let project = generate("slot");
    render(
        &project,
        "slot",
        r#"
const root = await host(field, { tags: ["a", null, "b"] });
assert.strictEqual(input(root).props.value, "a, b", "the input shows an empty entry for null");
"#,
    );
}

/// A tag literally named "null" is a tag; a held `null` entry hides it from the suggestions
/// because the input compares `String(null)` with it.
#[test]
fn a_tag_named_null_is_still_suggested_beside_a_held_null() {
    let project = generate("named");
    render(
        &project,
        "named",
        r#"
rows = [{ name: "null" }, { name: "x" }];
const root = await host(field, { tags: [null] });
assert.deepStrictEqual(suggestions(root).map(textOf), ["null", "x"], "a held null hides the suggestion named \"null\"");
"#,
    );
}

/// Display guards: the string "null" is a tag and still draws; several nulls draw nothing; a
/// dotted path into a list yields `undefined` and `null` entries, which draw no chip; React keys
/// stay unique when the dropped entries shift the index.
#[test]
fn the_display_drops_only_absent_entries() {
    let project = generate("display");
    render(
        &project,
        "display",
        r#"
const tags = { "data-ui-path": "pages/runs/sections/add/fields/tags", field: "labels", name: "labels", as: "tags" };
assert.deepStrictEqual(chips(shown(tags, { labels: ["null"] })), ["null"], "the text null is a tag");
assert.deepStrictEqual(chips(shown(tags, { labels: [null, null, null] })), []);
assert.strictEqual(shown(tags, { labels: [null, null] }).text(), "");
assert.deepStrictEqual(chips(shown(tags, { labels: [false, 0, ""] })), ["false", "0", ""], "falsy values are tags");
const dotted = { ...tags, field: "owners.name", name: "owners" };
assert.deepStrictEqual(chips(shown(dotted, { owners: [{ name: "a" }, { name: null }, {}, { name: "b" }] })), ["a", "b"]);
const runtime = require("react/jsx-runtime");
const keys = [];
for (const name of ["jsx", "jsxs"]) {
  const original = runtime[name];
  runtime[name] = (type, props, key) => {
    if (type === "span" && props && props.className === "ui-tag") keys.push(key);
    return original(type, props, key);
  };
}
const root = shown(tags, { labels: ["a", null, "a", null, "a"] });
assert.deepStrictEqual(chips(root), ["a", "a", "a"]);
assert.strictEqual(keys.length, 3, "every chip is keyed: " + JSON.stringify(keys));
assert.strictEqual(new Set(keys).size, keys.length, "keys are unique: " + JSON.stringify(keys));
"#,
    );
}
