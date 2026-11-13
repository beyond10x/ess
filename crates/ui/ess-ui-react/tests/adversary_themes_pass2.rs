//! Pass 2 on story:light-and-dark-themes-render-differently, against the fix that starts
//! a `chosen_by` state without `default:` at `theme.default`.
//!
//! `theme_choice` in `ess-ui-check` asks that every variant of the `chosen_by` enum names a theme
//! and that `theme.default` names a theme. It does not ask that `theme.default` is a variant of
//! that enum. The fix emits `theme.default` as the state's initial value, typed as the enum, so a
//! document the checker accepts can generate a hook call `tsc` refuses.

mod support;

use std::path::{Path, PathBuf};

use support::{compile, node};

/// Contrast the default theme, the user choosing between dark and light only, no `default:` on
/// the state.
fn default_outside_the_enum() -> String {
    r##"format: ess-ui/1
app: probe
model: probe.system
placement_profile: fat
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

/// Light the default, the enum listing dark first, no `default:` on the theme state; a second
/// shell state and a page state of the same enum, neither of them the `chosen_by` state.
fn neighbours() -> String {
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
      other: {type: {enum: [dark, light]}, class: preference, store: local_storage}
    regions:
      main: {kind: page_outlet}
navigation:
  home: things.list
  sections: [{name: all, pages: [things.list]}]
pages:
  things.list:
    kind: detail_page
    title: Things
    state:
      theme: {type: {enum: [dark, light]}, class: preference, store: local_storage}
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
        .join("ess-ui-react-adversary-themes-pass2")
        .join(name);
    if out.exists() {
        std::fs::remove_dir_all(&out).expect("the old scratch project is removed");
    }
    let document = ess_ui::load_str(text).unwrap_or_else(|error| panic!("{error}"));
    ess_ui_react::generate(&document, &root(), &out).unwrap_or_else(|error| panic!("{error}"));
    compile(&out, &["App.tsx", "runtime/state/storage.ts"]);
    out
}

/// The `theme_choice` findings `ess ui check` gives for the document.
fn theme_choice_findings(text: &str) -> Vec<String> {
    let report = ess_ui_check::check_source(
        text,
        "t.ui.yaml",
        Path::new(env!("CARGO_MANIFEST_DIR")),
        None,
        &ess_ui_check::Options::default(),
    );
    report
        .findings
        .iter()
        .filter(|finding| finding.check == "theme_choice")
        .map(|finding| format!("{}: {}", finding.path, finding.message))
        .collect()
}

const HARNESS: &str = r#"
const b = browser("/things/list");
const { App } = out("App");
const { setDataAdapter } = out("runtime/data");
const { storageKey } = out("runtime/state/storage");
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
const stateKey = storageKey("probe:shells/app/state/theme", undefined);
const theme = () => document.documentElement.getAttribute("data-theme");
const tick = () => new Promise((resolve) => setTimeout(resolve, 5));
async function settle(root) { for (let i = 0; i < 4; i++) { root.settle(); await tick(); } root.settle(); }
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

#[test]
fn a_theme_default_outside_the_chosen_by_enum_is_accepted_and_still_compiles() {
    let text = default_outside_the_enum();
    assert_eq!(
        theme_choice_findings(&text),
        Vec::<String>::new(),
        "the checker accepts a theme.default that names a theme the chosen_by enum leaves out"
    );
    run(
        "default-outside-the-enum",
        &text,
        r#"
  const root = React.__root(jsx(App, {}));
  await settle(root);
  assert.strictEqual(theme(), "contrast", "story:light-and-dark-themes-render-differently: before the state holds a value the shell shows theme.default");
"#,
    );
}

#[test]
fn a_stored_choice_wins_over_theme_default_on_reload_when_the_state_has_no_default() {
    run(
        "stored-no-default",
        &neighbours(),
        r#"
  store.set(stateKey, JSON.stringify("dark"));
  const root = React.__root(jsx(App, {}));
  await settle(root);
  assert.strictEqual(theme(), "dark", "story:light-and-dark-themes-render-differently: the stored choice is shown on reload, not theme.default");
"#,
    );
}

#[test]
fn only_the_chosen_by_shell_state_starts_at_theme_default() {
    let document = ess_ui::load_str(&neighbours()).unwrap_or_else(|error| panic!("{error}"));
    let files = ess_ui_react::render(&document, &root())
        .unwrap_or_else(|error| panic!("{error}"))
        .files;
    let hook = |key: &str| -> Vec<String> {
        let needle = format!("(\"probe:{key}\", ");
        files
            .values()
            .flat_map(|content| content.lines())
            .filter_map(|line| line.split_once(&needle).map(|(_, rest)| rest.to_owned()))
            .collect()
    };
    assert_eq!(
        hook("shells/app/state/theme"),
        vec!["\"light\");".to_owned()],
        "the chosen_by state starts at theme.default"
    );
    assert_eq!(
        hook("shells/app/state/other"),
        vec!["\"dark\");".to_owned()],
        "another shell state of the same enum keeps the enum's first variant"
    );
    let page: Vec<String> = files
        .values()
        .flat_map(|content| content.lines())
        .filter(|line| line.contains("probe:pages/") && line.contains("/state/theme\""))
        .map(str::to_owned)
        .collect();
    assert_eq!(page.len(), 1, "one page hook for the page state: {page:?}");
    assert!(
        page[0].ends_with(", \"dark\");"),
        "a page state named like the chosen_by state keeps the enum's first variant: {}",
        page[0]
    );
}
