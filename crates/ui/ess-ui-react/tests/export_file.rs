//! An export saves the file the server answers: a zip or an image is bytes, not the read rows
//! written as JSON text. An adapter that fetches files (`download`) hands the answer over as a
//! Blob, and that Blob is what the user saves; an answer in JSON still exports the rows.
//!
//! The case generates a small document, compiles its page to `CommonJS` with `tsc` and runs it
//! under `node` against the stand-ins in `support`.

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
          - {name: download, label: Download, export: {reads: desk.Bundle, as: zip, params: {ids: selection.id}}}
          - {name: sheet, label: Sheet, export: {reads: desk.Sheet, as: csv, params: {ids: selection.id}}}
";

fn project(name: &str) -> PathBuf {
    let out = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("ess-ui-react-export-file")
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
const ZIP = new Uint8Array([0x50, 0x4b, 0x03, 0x04, 0x14, 0x00, 0xff, 0xfe, 0x00, 0x80]);
const reads = [];
const downloads = [];
setDataAdapter({
  read: async (request) => {
    reads.push(request.view);
    return request.view === "desk.Files"
      ? { rows: [{ id: "f-1", name: "One" }, { id: "f-2", name: "Two" }], total: 2 }
      : { rows: [{ id: "r-1", name: "Row" }] };
  },
  download: async (request) => {
    downloads.push(JSON.parse(JSON.stringify(request)));
    return request.view === "desk.Bundle"
      ? new Blob([ZIP], { type: "application/zip" })
      : new Blob([JSON.stringify({ items: [] })], { type: "application/json" });
  },
  send: async () => ({ accepted: true }),
  loadState: async () => undefined,
  saveState: async () => undefined,
});
const saved = [];
globalThis.document = { createElement: () => ({ click() {} }) };
URL.createObjectURL = (blob) => { saved.push(blob); return "blob:export"; };
URL.revokeObjectURL = () => undefined;
const tick = () => new Promise((resolve) => setTimeout(resolve, 5));
const root = React.__root(jsx(FilesPage, {}));
async function settle() { for (let i = 0; i < 4; i++) { root.settle(); await tick(); } root.settle(); }
const textOf = (node) => (typeof node === "string" ? node : (node.children || []).map(textOf).join(""));
const press = async (label) => {
  const button = root.find((node) => node.tag === "button" && textOf(node) === label);
  assert.strictEqual(button.length, 1, `${label} renders once: ${root.text()}`);
  button[0].props.onClick({ preventDefault() {}, stopPropagation() {} });
  await settle();
};
(async () => {
  await settle();
  // The first box of a multiple-selection collection is its select-all (0.88.0); the row box selects one row.
  root.find((node) => node.tag === "input" && node.props.type === "checkbox" && node.props["aria-label"] === "Select row")[0].props.onChange({ target: { checked: true } });
  await settle();

  await press("Download");
  assert.deepStrictEqual(downloads, [{ view: "desk.Bundle", params: { ids: ["f-1"] } }], "the zip is fetched as a file with its params");
  assert.ok(!reads.includes("desk.Bundle"), `the zip is not read as rows: ${JSON.stringify(reads)}`);
  assert.strictEqual(saved.length, 1, "one file is saved");
  const bytes = new Uint8Array(await saved[0].arrayBuffer());
  assert.deepStrictEqual(Array.from(bytes), Array.from(ZIP), "the saved file is the bytes the server answered");

  await press("Sheet");
  assert.strictEqual(downloads.length, 2, "the CSV is fetched as a file too");
  assert.ok(reads.includes("desk.Sheet"), "an answer in JSON exports the read rows instead");
  assert.strictEqual(saved.length, 2, "one more file is saved");
  const text = await saved[1].text();
  assert.ok(text.includes("Row"), `the CSV holds the rows: ${text}`);
})().catch((error) => { console.error(error); process.exitCode = 1; });
"#;

#[test]
fn an_export_saves_the_file_the_server_answers() {
    let out = project("bytes");
    support::node(&out, "bytes", SCRIPT);
}
