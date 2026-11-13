//! The declared themes render differently (story:light-and-dark-themes-render-differently,
//! `docs/design/ui-style-tokens.md` § Renderers): `tokens.css` holds one `[data-theme="<name>"]`
//! block per theme other than the default, with only what it overrides, and the shell sets
//! `data-theme` on the document element from the `chosen_by` state, through that state's own
//! store module, and to `theme.default` before the state has a value or when nothing chooses.
//! A document without `themes:` generates no `data-theme` at all.
//!
//! Runtime cases compile the generated project to `CommonJS` with `tsc` and run it under `node`
//! against the stand-ins in `support`.

mod support;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use support::{compile, node};

const PAGES: &str = r"
navigation:
  home: things.list
  sections: [{name: all, pages: [things.list]}]
pages:
  things.list:
    kind: detail_page
    title: Things
    sections:
      - name: summary
        reads: {view: things.All}
        fields: [title]
";

/// Light and dark, `light` the default, the choice kept in local storage, and an account menu
/// whose entries set it.
fn chosen() -> String {
    format!(
        r##"format: ess-ui/1
app: probe
model: probe.system
placement_profile: fat
themes:
  light: {{}}
  dark:
    color: {{surface: "#101010", text: "#eeeeee"}}
theme: {{default: light, chosen_by: shell.theme}}
shells:
  app:
    state:
      theme: {{type: {{enum: [light, dark]}}, class: preference, store: local_storage, default: light}}
    regions:
      main: {{kind: page_outlet}}
      account:
        kind: account_menu
        props:
          actions:
            - {{name: dark, label: Dark theme, sets: {{shell.theme: "'dark'"}}}}
            - {{name: light, label: Light theme, sets: {{shell.theme: "'light'"}}}}
{PAGES}"##
    )
}

/// Dark the default and nothing chooses.
fn unchosen() -> String {
    format!(
        r##"format: ess-ui/1
app: probe
model: probe.system
placement_profile: fat
themes:
  light: {{}}
  dark:
    color: {{surface: "#101010"}}
theme: {{default: dark}}
shells:
  app:
    regions:
      main: {{kind: page_outlet}}
{PAGES}"##
    )
}

/// No `themes:` and no `theme:`.
fn plain() -> String {
    format!(
        r"format: ess-ui/1
app: probe
model: probe.system
placement_profile: fat
shells:
  app:
    regions:
      main: {{kind: page_outlet}}
{PAGES}"
    )
}

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

fn render(text: &str) -> BTreeMap<String, String> {
    let document = ess_ui::load_str(text).unwrap_or_else(|error| panic!("{error}"));
    ess_ui_react::render(&document, &root())
        .unwrap_or_else(|error| panic!("the document generates: {error}"))
        .files
}

fn project(name: &str, text: &str) -> PathBuf {
    let out = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("ess-ui-react-themes")
        .join(name);
    if out.exists() {
        std::fs::remove_dir_all(&out).expect("the old scratch project is removed");
    }
    let document = ess_ui::load_str(text).unwrap_or_else(|error| panic!("{error}"));
    ess_ui_react::generate(&document, &root(), &out).unwrap_or_else(|error| panic!("{error}"));
    let storage = out.join("src/runtime/state/storage.ts").exists();
    let modules: &[&str] = if storage {
        &["App.tsx", "runtime/state/storage.ts"]
    } else {
        &["App.tsx"]
    };
    compile(&out, modules);
    out
}

/// Renders the app at `/things/list` with a stand-in document element that records every
/// `data-theme` it is given, and a stand-in storage area.
const HARNESS: &str = r#"
const b = browser("/things/list");
const { App } = out("App");
const { setDataAdapter } = out("runtime/data");
let storageKey = () => undefined;
try { ({ storageKey } = out("runtime/state/storage")); } catch { /* a document without a stored state has no storage module */ }
setDataAdapter({
  read: async () => ({ rows: [], total: 0 }),
  send: async () => ({ accepted: true }),
  loadState: async () => undefined,
  saveState: async () => undefined,
});
const attributes = {};
const writes = [];
globalThis.document = {
  documentElement: {
    setAttribute(name, value) { attributes[name] = String(value); writes.push([name, String(value)]); },
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
const stateKey = storageKey("probe:shells/app/state/theme", undefined);
const theme = () => document.documentElement.getAttribute("data-theme");
const tick = () => new Promise((resolve) => setTimeout(resolve, 5));
function mount() {
  const root = React.__root(jsx(App, {}));
  return root;
}
async function settle(root) { for (let i = 0; i < 4; i++) { root.settle(); await tick(); } root.settle(); }
function textOf(node) {
  if (typeof node === "string") return node;
  return (node.children || []).map(textOf).join("");
}
async function pick(root, label) {
  const menu = root.find((n) => n.tag === "button" && n.props.className === "ui-button" && textOf(n) === "Account");
  assert.strictEqual(menu.length, 1, `one account button: ${root.text()}`);
  menu[0].props.onClick();
  await settle(root);
  const entry = root.find((n) => n.tag === "button" && n.props.className === "ui-menu-item" && textOf(n) === label);
  assert.strictEqual(entry.length, 1, `one entry labelled ${label}: ${root.text()}`);
  entry[0].props.onClick();
  await settle(root);
}
"#;

fn run(name: &str, text: &str, body: &str) {
    let out = project(name, text);
    node(
        &out,
        name,
        &format!(
            "{HARNESS}\n(async () => {{\n{body}\n}})().catch((error) => {{ console.error(error); process.exitCode = 1; }});\n"
        ),
    );
}

/// The declarations of the block opened by `selector`, one `name: value` per entry.
fn block(css: &str, selector: &str) -> Option<Vec<String>> {
    let start = css.find(&format!("{selector} {{\n"))?;
    let rest = &css[start + selector.len() + 3..];
    let end = rest.find('}')?;
    Some(
        rest[..end]
            .lines()
            .map(|line| line.trim().trim_end_matches(';').to_owned())
            .filter(|line| !line.is_empty())
            .collect(),
    )
}

#[test]
fn tokens_css_holds_one_block_per_theme_other_than_the_default_with_only_its_overrides() {
    let files = render(&chosen());
    let css = &files["src/runtime/tokens.css"];
    assert!(
        css.matches("[data-theme=").count() == 1,
        "story:light-and-dark-themes-render-differently: one block, for the one theme other than the default:\n{css}"
    );
    assert_eq!(
        block(css, "[data-theme=\"dark\"]").unwrap_or_else(|| panic!(
            "story:light-and-dark-themes-render-differently: tokens.css has no dark block:\n{css}"
        )),
        ["--ui-color-surface: #101010", "--ui-color-text: #eeeeee"],
        "story:light-and-dark-themes-render-differently: the dark block holds what dark overrides, and nothing else"
    );
    let root_block = block(css, ":root").expect("a :root block");
    assert!(
        root_block.contains(&"--ui-color-surface: #ffffff".to_owned()),
        "the default theme's values stay in :root: {root_block:?}"
    );
    assert!(
        css.find(":root {").unwrap_or(usize::MAX) < css.find("[data-theme=").unwrap_or(0),
        "a theme block follows :root, which it equals in specificity:\n{css}"
    );
}

#[test]
fn a_dark_default_puts_its_values_in_root_and_the_other_theme_restores_the_built_in_ones() {
    let files = render(&unchosen());
    let css = &files["src/runtime/tokens.css"];
    let root_block = block(css, ":root").expect("a :root block");
    assert!(
        root_block.contains(&"--ui-color-surface: #101010".to_owned()),
        "story:light-and-dark-themes-render-differently: :root holds the default theme's values: {root_block:?}"
    );
    assert_eq!(
        block(css, "[data-theme=\"light\"]").unwrap_or_else(|| panic!(
            "story:light-and-dark-themes-render-differently: tokens.css has no light block:\n{css}"
        )),
        ["--ui-color-surface: #ffffff"],
        "story:light-and-dark-themes-render-differently: the other theme holds what differs from :root"
    );
    assert!(
        !css.contains("[data-theme=\"dark\"]"),
        "the default theme has no block of its own:\n{css}"
    );
}

#[test]
fn the_shell_sets_data_theme_from_the_chosen_state_through_its_own_store() {
    run(
        "chosen",
        &chosen(),
        r#"
  let root = mount();
  await settle(root);
  assert.strictEqual(theme(), "light", "story:light-and-dark-themes-render-differently: the state holds light");
  await pick(root, "Dark theme");
  assert.strictEqual(theme(), "dark", "story:light-and-dark-themes-render-differently: the state was set to dark");
  assert.strictEqual(store.get(stateKey), JSON.stringify("dark"), "the state's own store module holds the choice");
  await pick(root, "Light theme");
  assert.strictEqual(theme(), "light", "story:light-and-dark-themes-render-differently: back to light");
  // A reload: the store holds dark before the first render.
  store.set(stateKey, JSON.stringify("dark"));
  root = mount();
  await settle(root);
  assert.strictEqual(theme(), "dark", "story:light-and-dark-themes-render-differently: the stored value is read through the state's store");
  // A stored value that names no theme shows the default.
  store.set(stateKey, JSON.stringify("sepia"));
  root = mount();
  await settle(root);
  assert.strictEqual(theme(), "light", "story:light-and-dark-themes-render-differently: a value naming no theme shows the default");
"#,
    );
}

#[test]
fn without_chosen_by_the_shell_sets_the_default_theme() {
    run(
        "unchosen",
        &unchosen(),
        r#"
  const root = mount();
  await settle(root);
  assert.strictEqual(theme(), "dark", "story:light-and-dark-themes-render-differently: nothing chooses, so theme.default");
"#,
    );
}

#[test]
fn a_document_without_themes_generates_no_data_theme() {
    let naming = |text: &str| -> Vec<String> {
        render(text)
            .into_iter()
            .filter(|(_, content)| content.contains("data-theme"))
            .map(|(path, _)| path)
            .collect()
    };
    let found = naming(&plain());
    assert!(
        found.is_empty(),
        "story:light-and-dark-themes-render-differently: a document without themes: has no data-theme: {found:?}"
    );
    let found = naming(&chosen());
    assert!(
        found.contains(&"src/runtime/tokens.css".to_owned())
            && found.contains(&"src/runtime/theme.ts".to_owned()),
        "a document with themes: names data-theme in tokens.css and the runtime that sets it: {found:?}"
    );
}
