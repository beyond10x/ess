//! An empty state's `title` renders as a heading above its message
//! (story:a-refused-node-is-reported-at-its-own-path, beyond10x/ess#723).
//!
//! The case generates a small document whose section reads no rows, compiles its page to
//! `CommonJS` with `tsc` and renders it under `node` against the stand-ins in `support`.

mod support;

use std::path::{Path, PathBuf};

const DOCUMENT: &str = "format: ess-ui/1
app: desk
model: desk.system
placement_profile: fat
shells:
  app: {regions: {main: {kind: page_outlet}}}
navigation:
  home: files
  sections: [{name: all, pages: [files]}]
pages:
  files:
    kind: list_page
    title: Files
    sections:
      - name: list
        component: collection
        reads: {view: desk.Files}
        columns: [name]
        states: {empty: {title: No files, message: Upload a file to see it here.}}
      - name: plain
        component: collection
        reads: {view: desk.Files}
        columns: [name]
        states: {empty: {message: Nothing else either.}}
";

fn project(name: &str) -> PathBuf {
    let out = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("ess-ui-react-empty-title")
        .join(name);
    if out.exists() {
        std::fs::remove_dir_all(&out).expect("the old scratch project is removed");
    }
    let document =
        ess_ui::load_str(DOCUMENT).unwrap_or_else(|error| panic!("the document loads: {error}"));
    ess_ui_react::generate(&document, Path::new("."), &out)
        .unwrap_or_else(|error| panic!("the project generates: {error}"));
    support::compile(&out, &["pages/Files.tsx"]);
    out
}

const SCRIPT: &str = r#"
browser("/files");
const { setDataAdapter } = out("runtime/data");
const { FilesPage } = out("pages/Files");
setDataAdapter({
  read: async () => ({ rows: [], total: 0 }),
  send: async () => ({ accepted: true }),
  loadState: async () => undefined,
  saveState: async () => undefined,
});
const tick = () => new Promise((resolve) => setTimeout(resolve, 5));
const root = React.__root(jsx(FilesPage, {}));
async function settle() { for (let i = 0; i < 4; i++) { root.settle(); await tick(); } root.settle(); }
const textOf = (node) => (typeof node === "string" ? node : (node.children || []).map(textOf).join(""));
(async () => {
  await settle();
  const states = root.find((node) => String(node.props && node.props.className || "").includes("ui-state-empty"));
  assert.strictEqual(states.length, 2, `both sections are empty: ${root.text()}`);
  const [titled, plain] = states;
  const kids = titled.children.filter((child) => typeof child === "object");
  assert.ok(kids.length >= 2, `the titled empty state has a heading and a message: ${JSON.stringify(titled)}`);
  assert.ok(/^h[1-6]$/.test(kids[0].tag), `the first child is a heading: ${JSON.stringify(kids[0])}`);
  assert.strictEqual(textOf(kids[0]), "No files");
  assert.strictEqual(kids[1].tag, "p");
  assert.strictEqual(textOf(kids[1]), "Upload a file to see it here.");
  const headings = plain.children.filter((child) => typeof child === "object" && /^h[1-6]$/.test(child.tag));
  assert.deepStrictEqual(headings, [], "an empty state without a title has no heading");
  assert.strictEqual(textOf(plain), "Nothing else either.");
})().catch((error) => { console.error(error); process.exitCode = 1; });
"#;

#[test]
fn an_empty_state_renders_its_title_above_its_message() {
    let out = project("titled");
    support::node(&out, "titled", SCRIPT);
}
