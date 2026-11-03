//! Adversary pass 1 on story:a-refused-node-is-reported-at-its-own-path (wave 0.86.0, unit U4).
//!
//! The new check `export_selection` admits `selection.<field>` in an export's params on a bulk
//! action, which it defines as one of `bulk_actions` *or an action with `bulk: true`*, and its
//! message tells the author either is fine. The generated runtime puts `selection` in scope only
//! around `bulk_actions` (`collection.tsx.tmpl`, the `ScopeLayer` around `props.bulkActions`);
//! `bulk: true` is emitted into the spec and read by nothing. So an action in `actions:` with
//! `bulk: true` and `params: {ids: selection.id}` passes the check and exports without the ids.
//! The case holds the two together: either the check refuses that document, or the runtime reads
//! the view with the selected ids.

mod support;

use std::path::{Path, PathBuf};

use ess_ui_check::{check_source, Options, Severity};

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
        actions:
          - {name: everything, label: Everything, bulk: true, export: {reads: desk.Bundle, as: zip, params: {ids: selection.id}}}
";

fn project(name: &str) -> PathBuf {
    let out = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("ess-ui-react-adversary-w086-u4-pass1")
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
const boxes = () => root.find((node) => node.tag === "input" && node.props.type === "checkbox");
(async () => {
  await settle();
  assert.strictEqual(boxes().length, 3, `three rows to select: ${root.text()}`);
  boxes()[0].props.onChange({ target: { checked: true } });
  await settle();
  boxes()[2].props.onChange({ target: { checked: true } });
  await settle();
  const button = root.find((node) => node.tag === "button" && textOf(node) === "Everything");
  assert.strictEqual(button.length, 1, `the bulk: true action renders once: ${root.text()}`);
  button[0].props.onClick({ preventDefault() {}, stopPropagation() {} });
  await settle();
  const exported = reads.filter((request) => request.view === "desk.Bundle");
  assert.deepStrictEqual(
    exported.map((request) => request.params),
    [{ ids: ["f-1", "f-3"] }],
    `story:a-refused-node-is-reported-at-its-own-path: export_selection admits selection.id on a bulk: true action, so the export must read the selected ids: ${JSON.stringify(exported)}`,
  );
})().catch((error) => { console.error(error); process.exitCode = 1; });
"#;

#[test]
fn a_selection_param_the_check_admits_reaches_the_export() {
    let refused = check_source(
        DOCUMENT,
        "test.yaml",
        Path::new(env!("CARGO_MANIFEST_DIR")),
        None,
        &Options::default(),
    )
    .findings
    .into_iter()
    .any(|finding| finding.check == "export_selection" && finding.severity == Severity::Error);
    if refused {
        return;
    }
    let out = project("bulk-flag");
    support::node(&out, "bulk-flag", SCRIPT);
}
