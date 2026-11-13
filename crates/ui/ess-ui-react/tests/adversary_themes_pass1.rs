//! Adversary pass 1 on story:light-and-dark-themes-render-differently.
//!
//! The acceptance says the shell sets `data-theme` "to `theme.default` before the state has a
//! value", and the design (`docs/design/ui-style-tokens.md` § Themes) says a renderer shows
//! `default` where the state "holds no value". A `chosen_by` state may leave out `default:`
//! (`theme_choice` in `ess-ui-check` only checks a default that is given). Such a state starts at
//! the generator's type initial, which for an enum is its first variant, so before the user
//! picks anything the shell shows that variant instead of `theme.default`.

mod support;

use std::path::{Path, PathBuf};

use support::{compile, node};

/// Light the default, the choice an enum listing dark first, no `default:` on the state, and
/// nothing stored.
fn undefaulted() -> String {
    r##"format: ess-ui/1
app: probe
model: probe.system
placement_profile: fat
themes:
  light: {}
  dark:
    color: {surface: "#101010"}
theme: {default: light, chosen_by: shell.theme}
shells:
  app:
    state:
      theme: {type: {enum: [dark, light]}, class: preference, store: local_storage}
    regions:
      main: {kind: page_outlet}
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
"##
    .to_owned()
}

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

fn project(name: &str, text: &str) -> PathBuf {
    let out = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("ess-ui-react-adversary-themes-pass1")
        .join(name);
    if out.exists() {
        std::fs::remove_dir_all(&out).expect("the old scratch project is removed");
    }
    let document = ess_ui::load_str(text).unwrap_or_else(|error| panic!("{error}"));
    ess_ui_react::generate(&document, &root(), &out).unwrap_or_else(|error| panic!("{error}"));
    compile(&out, &["App.tsx", "runtime/state/storage.ts"]);
    out
}

const HARNESS: &str = r#"
const b = browser("/things/list");
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
async function settle(root) { for (let i = 0; i < 4; i++) { root.settle(); await tick(); } root.settle(); }
"#;

#[test]
fn a_chosen_by_state_without_a_default_shows_theme_default_before_it_holds_a_value() {
    let out = project("undefaulted", &undefaulted());
    node(
        &out,
        "undefaulted",
        &format!(
            "{HARNESS}\n(async () => {{\n{}\n}})().catch((error) => {{ console.error(error); process.exitCode = 1; }});\n",
            r#"
  const root = React.__root(jsx(App, {}));
  await settle(root);
  assert.strictEqual(store.size, 0, "nothing was stored, so the state holds no value");
  assert.strictEqual(theme(), "light", "story:light-and-dark-themes-render-differently: before the chosen_by state holds a value the shell shows theme.default (light), not the enum's first variant");
"#
        ),
    );
}
