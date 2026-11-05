//! Tag suggestions in the generated React project: an `as: tags` input with `suggests` offers the
//! values its read answers, each one the draft does not hold yet, and still keeps values typed into
//! it, so a tags field can suggest existing tags without a create command.
//!
//! Each case reads the field spec the generated page passes, then renders the generated runtime's
//! `FieldInput` under `node` over a data adapter that answers the suggestions read.

mod support;

use std::path::{Path, PathBuf};
use std::process::Command;

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
          - {field: name, label: Name}
          - {field: tags, label: Tags, as: tags, suggests: {reads: {view: t.Tags, params: {category: '"run"'}}, value: name}}
          - {field: labels, label: Labels, as: tags}
"#;

fn scratch(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("ess-ui-react-tags-suggest")
        .join(name);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("the old scratch dir is removed");
    }
    std::fs::create_dir_all(&dir).expect("the scratch dir is created");
    dir
}

/// Generates the document into `<scratch>/app` and returns the project and its page sources.
fn generate(name: &str) -> (PathBuf, String) {
    let dir = scratch(name);
    let document = ess_ui::load_str(DOCUMENT).unwrap_or_else(|error| panic!("{error}"));
    let project = dir.join("app");
    let files = ess_ui_react::generate(&document, &dir, &project)
        .unwrap_or_else(|error| panic!("the document generates: {error}"));
    let pages: String = files
        .files
        .iter()
        .filter(|(path, _)| !path.starts_with("src/runtime/"))
        .map(|(_, text)| text.as_str())
        .collect();
    (project, pages)
}

/// The spec the page passes the form field `name`, up to the next node path.
fn field_spec(pages: &str, name: &str) -> String {
    let marker = format!("\"data-ui-path\": \"pages/runs/sections/add/fields/{name}\"");
    let start = pages
        .find(&marker)
        .unwrap_or_else(|| panic!("the page passes the field `{name}`:\n{pages}"));
    let spec = &pages[start + marker.len()..];
    spec[..spec.find("\"data-ui-path\": ").unwrap_or(spec.len())].to_owned()
}

/// `host(field, draft)` renders `FieldInput` over a draft it holds; `draft()` returns it. The
/// adapter answers `t.Tags` with `rows` and records every read.
const RENDER: &str = r#"
const { FieldInput } = out("runtime/fields");
const { ScopeLayer } = out("runtime/core");
const { setDataAdapter } = out("runtime/data");
let rows = [];
const reads = [];
setDataAdapter({
  read: async (request) => { reads.push(request); return { rows: request.view === "t.Tags" ? rows : [], total: rows.length }; },
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

#[test]
fn the_page_passes_the_suggestions_read() {
    let (project, pages) = generate("spec");
    let spec = field_spec(&pages, "tags");
    assert!(
        spec.contains("suggests: {") && spec.contains(r#"view: "t.Tags""#),
        "the page passes the field its suggestions read:\n{spec}"
    );
    assert!(
        spec.contains(r#"value: "name""#),
        "the page passes the row field each suggestion takes:\n{spec}"
    );
    assert!(
        !field_spec(&pages, "labels").contains("suggests"),
        "a tags field without `suggests` passes none"
    );
    let output = Command::new("tsc")
        .args(["-p", "tsconfig.offline.json", "--noEmit"])
        .current_dir(&project)
        .output()
        .expect("tsc is on PATH");
    assert!(
        output.status.success(),
        "the generated project type-checks:\n{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn a_suggestion_not_yet_held_is_offered_and_added() {
    let (project, _) = generate("offer");
    render(
        &project,
        "offer",
        r#"
rows = [{ name: "vip" }, { name: "sales" }, { name: "support" }];
const field = { "data-ui-path": "pages/runs/sections/add/fields/tags", field: "tags", name: "tags", as: "tags", suggests: { reads: { view: "t.Tags", params: { category: '"run"' } }, value: "name" } };
let root = await host(field, { tags: ["vip"] });
assert.deepStrictEqual(reads.map((r) => [r.view, r.params]), [["t.Tags", { category: "run" }]], "the suggestions read is sent with its params");
assert.deepStrictEqual(suggestions(root).map(textOf), ["sales", "support"], "a tag the draft holds is not suggested");
suggestions(root)[0].props.onClick();
root = root.settle();
assert.deepStrictEqual(draft().tags, ["vip", "sales"], "a picked suggestion is added");
assert.deepStrictEqual(suggestions(root).map(textOf), ["support"]);
"#,
    );
}

#[test]
fn a_typed_value_is_kept() {
    let (project, _) = generate("typed");
    render(
        &project,
        "typed",
        r#"
rows = ["vip", "sales"];
const field = { "data-ui-path": "pages/runs/sections/add/fields/tags", field: "tags", name: "tags", as: "tags", suggests: { reads: { view: "t.Tags" } } };
let root = await host(field, { tags: [] });
assert.deepStrictEqual(suggestions(root).map(textOf), ["vip", "sales"], "a plain-string row is the suggestion itself");
input(root).props.onChange({ target: { value: "brand-new, vip" } });
root = root.settle();
assert.deepStrictEqual(draft().tags, ["brand-new", "vip"], "a typed value no read answers is kept");
assert.deepStrictEqual(suggestions(root).map(textOf), ["sales"]);
"#,
    );
}

#[test]
fn a_tags_field_without_suggests_reads_nothing() {
    let (project, _) = generate("plain");
    render(
        &project,
        "plain",
        r#"
rows = ["vip"];
const field = { "data-ui-path": "pages/runs/sections/add/fields/labels", field: "labels", name: "labels", as: "tags" };
const root = await host(field, { labels: ["a"] });
assert.deepStrictEqual(reads, [], "no read without suggests");
assert.deepStrictEqual(suggestions(root), []);
assert.strictEqual(input(root).props.value, "a");
"#,
    );
}
