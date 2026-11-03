//! An account menu entry runs as an action: one with `sets` changes the named state, one with
//! `navigate` changes the route. The toy document switches the theme from the account menu.
//!
//! Runtime cases compile the generated project to `CommonJS` with `tsc` and run it under `node`
//! against the stand-ins in `support`.

mod support;

use std::path::{Path, PathBuf};

use support::{compile, node};

const DOCUMENT: &str = r#"
format: ess-ui/1
app: probe
model: probe.system
placement_profile: fat
shells:
  app:
    state:
      theme: {type: {enum: [light, dark]}, class: preference, store: memory, default: light}
    regions:
      main: {kind: page_outlet}
      dark: {kind: notifications, visible: "shell.theme == 'dark'"}
      account:
        kind: account_menu
        props:
          actions:
            - {name: dark, label: Dark theme, sets: {shell.theme: "'dark'"}}
            - {name: other, label: Other page, navigate: {to: things.other}}
navigation:
  home: things.list
  sections: [{name: all, pages: [things.list, things.other]}]
pages:
  things.list:
    kind: detail_page
    title: Things
    sections:
      - name: summary
        reads: {view: things.All}
        fields: [title]
  things.other:
    kind: detail_page
    title: Other
    sections:
      - name: summary
        reads: {view: things.All}
        fields: [title]
"#;

fn project(name: &str) -> PathBuf {
    let out = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("ess-ui-react-account-menu")
        .join(name);
    if out.exists() {
        std::fs::remove_dir_all(&out).expect("the old scratch project is removed");
    }
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let document = ess_ui::load_str(DOCUMENT).unwrap_or_else(|error| panic!("{error}"));
    ess_ui_react::generate(&document, &root, &out).unwrap_or_else(|error| panic!("{error}"));
    compile(&out, &["App.tsx"]);
    out
}

/// Renders the app at `/things/list` and picks the account menu entry labelled `label`.
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
const root = React.__root(jsx(App, {}));
const tick = () => new Promise((resolve) => setTimeout(resolve, 5));
async function settle() { for (let i = 0; i < 4; i++) { root.settle(); await tick(); } root.settle(); }
function textOf(node) {
  if (typeof node === "string") return node;
  return (node.children || []).map(textOf).join("");
}
async function pick(label) {
  const menu = root.find((n) => n.tag === "button" && n.props.className === "ui-button" && textOf(n) === "Account");
  assert.strictEqual(menu.length, 1, `one account button: ${root.text()}`);
  menu[0].props.onClick();
  await settle();
  const entry = root.find((n) => n.tag === "button" && n.props.className === "ui-menu-item" && textOf(n) === label);
  assert.strictEqual(entry.length, 1, `one entry labelled ${label}: ${root.text()}`);
  entry[0].props.onClick();
  await settle();
}
const dark = () => root.find((n) => n.props && n.props["data-ui-path"] === "shells/app/regions/dark").length;
"#;

fn run(name: &str, body: &str) {
    let out = project(name);
    node(
        &out,
        name,
        &format!(
            "{HARNESS}\n(async () => {{\n{body}\n}})().catch((error) => {{ console.error(error); process.exitCode = 1; }});\n"
        ),
    );
}

#[test]
fn an_account_menu_entry_with_sets_changes_the_named_state() {
    run(
        "sets",
        r#"
  await settle();
  assert.strictEqual(dark(), 0, "the theme starts light");
  await pick("Dark theme");
  assert.strictEqual(dark(), 1, "story:account-menu-entries-keep-sets-and-navigate: the entry set shell.theme to dark");
"#,
    );
}

#[test]
fn an_account_menu_entry_with_navigate_changes_the_route() {
    run(
        "navigate",
        r#"
  await settle();
  await pick("Other page");
  assert.ok(
    b.calls.some(([kind, href]) => kind === "push" && href === "/things/other"),
    `story:account-menu-entries-keep-sets-and-navigate: the entry navigated: ${JSON.stringify(b.calls)}`,
  );
"#,
    );
}

/// An entry that does not read as an action is refused, naming it, not drawn without its `sets`
/// or `navigate`: `navigate` written as a page name is one.
#[test]
fn an_account_menu_entry_that_is_not_an_action_is_refused() {
    let text = DOCUMENT.replace("navigate: {to: things.other}", "navigate: things.other");
    let document = ess_ui::load_str(&text).unwrap_or_else(|error| panic!("{error}"));
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let error = ess_ui_react::render(&document, &root)
        .expect_err("story:account-menu-entries-keep-sets-and-navigate: the entry is refused");
    assert!(
        error
            .to_string()
            .starts_with("shells/app/regions/account/props/actions/other: this account menu entry does not read as an action"),
        "story:account-menu-entries-keep-sets-and-navigate: {error}"
    );
}
