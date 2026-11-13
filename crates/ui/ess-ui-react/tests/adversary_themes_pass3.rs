//! Pass 3 on story:light-and-dark-themes-render-differently, against the fix that widens the
//! generated type of a `chosen_by` shell state whose enum leaves out `theme.default`.
//!
//! The document reaches that state through a named enum type, binds a segmented choice on a page
//! to it, writes it from an account menu entry and shows a region on a condition comparing it.
//! `ess ui check` accepts it; the generated app must compile and every one of those must work.

mod support;

use std::path::{Path, PathBuf};

use support::{compile, node};

const DOCUMENT: &str = r##"format: ess-ui/1
app: probe
model: probe.system
placement_profile: fat
types:
  Mode: {enum: [dark, light]}
themes:
  light: {}
  dark:
    color: {surface: "#101010"}
  contrast:
    color: {surface: "#000000"}
theme: {default: contrast, chosen_by: shell.theme}
shells:
  app:
    state:
      theme: {type: Mode, class: preference, store: local_storage}
    regions:
      main: {kind: page_outlet}
      flag: {kind: notifications, visible: "shell.theme == 'contrast'"}
      account:
        kind: account_menu
        props:
          actions:
            - {name: light, label: Light theme, sets: {shell.theme: "'light'"}}
navigation:
  home: things.list
  sections: [{name: all, pages: [things.list]}]
pages:
  things.list:
    kind: detail_page
    title: Things
    sections:
      - {name: mode, component: choice, options: [dark, light], style: segmented, binds: shell.theme}
      - name: summary
        reads: {view: things.All}
        fields: [title]
"##;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

#[test]
fn a_named_enum_chosen_by_state_bound_set_and_compared_compiles_and_works() {
    let errors: Vec<String> = ess_ui_check::check_source(
        DOCUMENT,
        "t.ui.yaml",
        Path::new(env!("CARGO_MANIFEST_DIR")),
        None,
        &ess_ui_check::Options::default(),
    )
    .findings
    .iter()
    .filter(|finding| finding.severity == ess_ui_check::Severity::Error)
    .map(|finding| format!("{} {}: {}", finding.check, finding.path, finding.message))
    .collect();
    assert_eq!(
        errors,
        Vec::<String>::new(),
        "ess ui check accepts the document"
    );
    let out = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("ess-ui-react-adversary-themes-pass3")
        .join("named");
    if out.exists() {
        std::fs::remove_dir_all(&out).expect("the old scratch project is removed");
    }
    let document = ess_ui::load_str(DOCUMENT).unwrap_or_else(|error| panic!("{error}"));
    ess_ui_react::generate(&document, &root(), &out).unwrap_or_else(|error| panic!("{error}"));
    compile(&out, &["App.tsx", "runtime/state/storage.ts"]);
    node(
        &out,
        "named",
        r#"
browser("/things/list");
const { App } = out("App");
const { setDataAdapter } = out("runtime/data");
setDataAdapter({
  read: async () => ({ rows: [], total: 0 }),
  send: async () => ({ accepted: true }),
  loadState: async () => undefined,
  saveState: async () => undefined,
});
const attributes = {};
globalThis.document = {
  documentElement: {
    setAttribute(name, value) { attributes[name] = String(value); },
    getAttribute(name) { return name in attributes ? attributes[name] : null; },
    removeAttribute(name) { delete attributes[name]; },
  },
};
const store = new Map();
globalThis.localStorage = {
  getItem: (key) => (store.has(key) ? store.get(key) : null),
  setItem: (key, value) => { store.set(key, String(value)); },
  removeItem: (key) => { store.delete(key); },
};
const theme = () => document.documentElement.getAttribute("data-theme");
const tick = () => new Promise((resolve) => setTimeout(resolve, 5));
function textOf(node) {
  if (typeof node === "string") return node;
  return (node.children || []).map(textOf).join("");
}
(async () => {
  const root = React.__root(jsx(App, {}));
  async function settle() { for (let i = 0; i < 4; i++) { root.settle(); await tick(); } root.settle(); }
  const chips = () => root.find((n) => n.tag === "button" && typeof n.props.className === "string" && n.props.className.startsWith("ui-chip"));
  const pressed = () => chips().filter((n) => n.props["aria-pressed"]).map((n) => textOf(n).toLowerCase());
  const flag = () => root.find((n) => n.props && n.props["data-ui-path"] === "shells/app/regions/flag").length;
  await settle();
  assert.strictEqual(theme(), "contrast", "before the state holds a value the shell shows theme.default");
  assert.strictEqual(chips().length, 2, `the segmented choice offers dark and light: ${root.text()}`);
  assert.deepStrictEqual(pressed(), [], "no chip is on while the state holds a theme the choice does not offer");
  assert.strictEqual(flag(), 1, "the condition comparing the state with theme.default holds");
  chips().find((n) => textOf(n).toLowerCase() === "dark").props.onClick();
  await settle();
  assert.strictEqual(theme(), "dark", "picking dark on the segmented choice shows dark");
  assert.deepStrictEqual(pressed(), ["dark"], "the dark chip is on");
  assert.strictEqual(flag(), 0, "the condition no longer holds");
  const menu = root.find((n) => n.tag === "button" && n.props.className === "ui-button" && textOf(n) === "Account");
  menu[0].props.onClick();
  await settle();
  root.find((n) => n.tag === "button" && n.props.className === "ui-menu-item" && textOf(n) === "Light theme")[0].props.onClick();
  await settle();
  assert.strictEqual(theme(), "light", "the account menu entry set the state to light");
  assert.deepStrictEqual(pressed(), ["light"], "the light chip is on");
})().catch((error) => { console.error(error); process.exitCode = 1; });
"#,
    );
}
