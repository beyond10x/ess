//! A bulk export reads its view with params evaluated in the action's scope, so
//! `params: {ids: selection.id}` reads the selected rows' ids
//! (story:a-refused-node-is-reported-at-its-own-path, beyond10x/ess#723).
//!
//! The case generates a small document, compiles its page to `CommonJS` with `tsc` and runs it
//! under `node` against the stand-ins in `support`: two rows are selected, the bulk action is
//! clicked, and the read it makes is taken off the data adapter.

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
        selection: multiple
        reads: {view: desk.Files}
        columns: [name]
        bulk_actions:
          - {name: download, label: Download, export: {reads: desk.Bundle, as: zip, params: {ids: selection.id, kind: archive}}}
";

fn project(name: &str) -> PathBuf {
    let out = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("ess-ui-react-export-params")
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
const reads = [];
setDataAdapter({
  read: async (request) => {
    reads.push(JSON.parse(JSON.stringify(request)));
    return request.view === "desk.Files"
      ? { rows: [{ id: "f-1", name: "One" }, { id: "f-2", name: "Two" }, { id: "f-3", name: "Three" }], total: 3 }
      : { rows: [] };
  },
  send: async () => ({ accepted: true }),
  loadState: async () => undefined,
  saveState: async () => undefined,
});
globalThis.document = { createElement: () => ({ click() {} }) };
URL.createObjectURL = () => "blob:export";
URL.revokeObjectURL = () => undefined;
const tick = () => new Promise((resolve) => setTimeout(resolve, 5));
const root = React.__root(jsx(FilesPage, {}));
async function settle() { for (let i = 0; i < 4; i++) { root.settle(); await tick(); } root.settle(); }
const textOf = (node) => (typeof node === "string" ? node : (node.children || []).map(textOf).join(""));
(async () => {
  await settle();
  const boxes = root.find((node) => node.tag === "input" && node.props.type === "checkbox" && node.props["aria-label"] === "Select row");
  assert.strictEqual(boxes.length, 3, `three rows to select: ${root.text()}`);
  boxes[0].props.onChange({ target: { checked: true } });
  await settle();
  root.find((node) => node.tag === "input" && node.props.type === "checkbox" && node.props["aria-label"] === "Select row")[2].props.onChange({ target: { checked: true } });
  await settle();
  const button = root.find((node) => node.tag === "button" && textOf(node) === "Download");
  assert.strictEqual(button.length, 1, `the bulk action renders once: ${root.text()}`);
  button[0].props.onClick({ preventDefault() {}, stopPropagation() {} });
  await settle();
  const exported = reads.filter((request) => request.view === "desk.Bundle");
  assert.deepStrictEqual(
    exported.map((request) => request.params),
    [{ ids: ["f-1", "f-3"], kind: "archive" }],
    `the export reads: ${JSON.stringify(reads)}`,
  );
})().catch((error) => { console.error(error); process.exitCode = 1; });
"#;

#[test]
fn a_bulk_export_sends_its_selection_params() {
    let out = project("bulk");
    support::node(&out, "bulk", SCRIPT);
}
