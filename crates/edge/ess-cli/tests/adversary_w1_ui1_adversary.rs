//! Adversary pass, ESS W1 unit UI1. Each case asserts a story's Acceptance directly.
//!
//! - story:account-menu-entries-keep-sets-and-navigate, acceptance 2: "`ess ui check` refuses any
//!   declared key that a renderer drops". The cases build account menu entries whose every key is
//!   in the unit's accepted set, and that a renderer still drops.
//! - story:a-generated-ui-calls-the-routes-its-http-binding-serves, acceptance 1: "both runtimes
//!   use [the method]". A command an `ess-http/1` binding serves on `get` (admitted by
//!   `ESS-HTTP-006` and `ESS-HTTP-014` when the path carries every input field).

#[path = "../../../ui/ess-ui-react/tests/support/mod.rs"]
mod support;

use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

use ess_ui_check::{check_source, Options, Report, Severity};

const MENU: &str = "story:account-menu-entries-keep-sets-and-navigate";
const ROUTES: &str = "story:a-generated-ui-calls-the-routes-its-http-binding-serves";

fn scratch(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("adversary-w1-ui1")
        .join(name);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("a stale scratch directory is removed");
    }
    std::fs::create_dir_all(&dir).expect("the scratch directory is created");
    dir
}

/// A document whose shell has an account menu with `entries` (YAML flow items).
fn menu_document(entries: &str) -> String {
    format!(
        r#"format: ess-ui/1
app: probe
model: probe.system
placement_profile: fat
shells:
  app:
    state:
      theme: {{type: {{enum: [light, dark]}}, class: preference, store: memory, default: light}}
    regions:
      main: {{kind: page_outlet}}
      dark: {{kind: notifications, visible: "shell.theme == 'dark'"}}
      account:
        kind: account_menu
        props:
          actions:
{entries}
navigation:
  home: things.list
  sections: [{{name: all, pages: [things.list, things.other]}}]
pages:
  things.list:
    kind: detail_page
    title: Things
    sections:
      - name: summary
        reads: {{view: things.All}}
        fields: [title]
  things.other:
    kind: detail_page
    title: Other
    sections:
      - name: summary
        reads: {{view: things.All}}
        fields: [title]
"#
    )
}

fn check(text: &str) -> Report {
    check_source(
        text,
        "ui.yaml",
        Path::new(env!("CARGO_MANIFEST_DIR")),
        None,
        &Options::default(),
    )
}

/// Every error finding at `prefix` or under it.
fn errors_under<'a>(report: &'a Report, prefix: &str) -> Vec<&'a ess_ui_check::Finding> {
    report
        .findings
        .iter()
        .filter(|finding| finding.severity == Severity::Error && finding.path.starts_with(prefix))
        .collect()
}

const ACTIONS: &str = "shells/app/regions/account/props/actions";

/// An entry whose keys are all in the accepted set (`name`, `label`, `opens`, `does`, `sets`,
/// `navigate`) but which does not parse as an action is dropped whole by the terminal:
/// `App::account_actions` (`crates/ui/ess-ui-tui/src/app.rs:3470`) keeps only the entries
/// `ess_yaml::from_value::<Action>` admits, and the React emitter (`emit.rs:2471`) drops `sets`
/// and `navigate` the same way. The check passes each of them in silence.
#[test]
fn an_account_menu_entry_the_terminal_cannot_read_as_an_action_is_refused() {
    let mut unrefused = Vec::new();
    for (what, entry) in [
        (
            "an entry without `name`",
            r#"{label: Dark theme, sets: {shell.theme: "'dark'"}}"#,
        ),
        (
            "`navigate` written as a page name",
            "{name: away, label: Away, navigate: things.other}",
        ),
        (
            "`sets` written as a scalar",
            r#"{name: dark, label: Dark, sets: "'dark'"}"#,
        ),
    ] {
        let value: ess_yaml::Value = ess_yaml::from_str(entry).expect("the entry is YAML");
        // The terminal's own reading of the entry: it is dropped.
        assert!(
            ess_yaml::from_value::<ess_ui::Action>(value).is_err(),
            "{what}: the terminal reads it as an action"
        );
        let text = menu_document(&format!("            - {entry}"));
        ess_ui::load_str(&text).unwrap_or_else(|error| panic!("{what}: {error}"));
        let report = check(&text);
        if errors_under(&report, ACTIONS).is_empty() {
            unrefused.push(format!("{what}: {entry}"));
        }
    }
    assert!(
        unrefused.is_empty(),
        "{MENU}: dropped by the terminal, and `ess ui check` refuses nothing under {ACTIONS}:\n{}",
        unrefused.join("\n")
    );
}

/// The React shell's menu runs an entry's `does`, else `runAction` (`shell.tsx.tmpl:170-180`),
/// and `runAction` runs `navigate` before, and instead of, `sets` (`actions.tsx.tmpl:178-205`).
/// An entry that declares `sets` beside `does` or beside `navigate` therefore loses `sets` in
/// React, while every key it declares is in the accepted set. Either React keeps the `sets` or the
/// check refuses the entry; this case fails when neither holds.
#[test]
fn sets_beside_does_or_navigate_on_an_account_menu_entry_is_kept_or_refused() {
    let entries = [
        (
            "out_dark",
            "Sign out dark",
            r#"{name: out_dark, label: Sign out dark, does: probe.SignOut, sets: {shell.theme: "'dark'"}}"#,
        ),
        (
            "away_dark",
            "Away dark",
            r#"{name: away_dark, label: Away dark, navigate: {to: things.other}, sets: {shell.theme: "'dark'"}}"#,
        ),
    ];
    let text = menu_document(
        &entries
            .iter()
            .map(|(_, _, entry)| format!("            - {entry}"))
            .collect::<Vec<_>>()
            .join("\n"),
    );
    let document = ess_ui::load_str(&text).unwrap_or_else(|error| panic!("{error}"));
    let out = scratch("menu-sets").join("app");
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    ess_ui_react::generate(&document, &root, &out).unwrap_or_else(|error| panic!("{error}"));
    support::compile(&out, &["App.tsx"]);
    let report = check(&text);
    let mut failures = Vec::new();
    for (name, label, entry) in entries {
        let script = format!(
            r#"
const b = browser("/things/list");
const {{ App }} = out("App");
const {{ setDataAdapter }} = out("runtime/data");
setDataAdapter({{
  read: async () => ({{ rows: [], total: 0 }}),
  send: async () => ({{ accepted: true }}),
  loadState: async () => undefined,
  saveState: async () => undefined,
}});
const root = React.__root(jsx(App, {{}}));
const tick = () => new Promise((resolve) => setTimeout(resolve, 5));
async function settle() {{ for (let i = 0; i < 4; i++) {{ root.settle(); await tick(); }} root.settle(); }}
function textOf(node) {{
  if (typeof node === "string") return node;
  return (node.children || []).map(textOf).join("");
}}
const dark = () => root.find((n) => n.props && n.props["data-ui-path"] === "shells/app/regions/dark").length;
(async () => {{
  await settle();
  const before = dark();
  const menu = root.find((n) => n.tag === "button" && n.props.className === "ui-button" && textOf(n) === "Account");
  menu[0].props.onClick();
  await settle();
  const entry = root.find((n) => n.tag === "button" && n.props.className === "ui-menu-item" && textOf(n) === {label:?});
  if (entry.length !== 1) throw new Error("no entry " + {label:?} + ": " + root.text());
  entry[0].props.onClick();
  await settle();
  console.log(JSON.stringify({{ before, after: dark() }}));
}})().catch((error) => {{ console.error(error); process.exit(1); }});
"#
        );
        let printed = support::node_stdout(&out, name, &script);
        let value: serde_json::Value =
            serde_json::from_str(printed.lines().last().unwrap_or_default())
                .unwrap_or_else(|error| panic!("{error}: {printed}"));
        assert_eq!(value["before"], 0, "{name}: the theme starts light");
        let kept = value["after"] == 1;
        let refused = !errors_under(&report, &format!("{ACTIONS}/{name}")).is_empty();
        if !kept && !refused {
            failures.push(format!(
                "{entry}: React did not set shell.theme, and `ess ui check` refuses nothing \
                 under {ACTIONS}/{name}"
            ));
        }
    }
    assert!(
        failures.is_empty(),
        "{MENU}: a declared key a renderer drops goes unrefused:\n{}\nfindings: {:#?}",
        failures.join("\n"),
        report.findings
    );
}

// ── a command the binding serves on `get` ───────────────────────────────────────────────────

fn ess(arguments: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ess"))
        .args(arguments)
        .env_remove("ESS_UI_AUTHORIZATION")
        .stdin(Stdio::null())
        .output()
        .expect("the ess binary runs")
}

fn utf8(path: &Path) -> &str {
    path.to_str().expect("the scratch path is UTF-8")
}

fn items(path: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/ui-http/items")
        .join(path)
}

/// The source digest of the items toy, as the generation of its schemas records it.
fn digest(dir: &Path) -> String {
    let out = dir.join("schema");
    let model = items("system.yaml");
    let generated = ess(&[
        "generate",
        "--path",
        utf8(&model),
        "--kind",
        "schema",
        "--out",
        utf8(&out),
    ]);
    assert!(
        generated.status.success(),
        "{}",
        String::from_utf8_lossy(&generated.stderr)
    );
    // A generated file stamps only its own slice's digest; the run records the model's.
    let state: serde_json::Value =
        serde_json::from_slice(&std::fs::read(out.join(".ess-output/state.json")).unwrap())
            .unwrap();
    state["payload"]["checkpoint"]["ledger"]["owners"][0]["model"]
        .as_str()
        .unwrap_or_else(|| panic!("the generation recorded no model digest: {state}"))
        .to_owned()
}

/// `RenameItem` served on `GET /items/{item_id}/titles/{title}`: the path carries both input
/// fields, so the binding needs no `request` (`ESS-HTTP-014`), and its compiled request is the
/// default, which `ess_ui_check::binding_over` admits as a `json` body. The generated React
/// runtime then sends `GET` with a body, which `fetch` refuses before any byte reaches a server
/// (the Fetch standard's `Request` constructor: "Request with GET/HEAD method cannot have body").
#[test]
fn a_command_bound_to_get_is_sent_by_the_react_runtime() {
    let dir = scratch("get-command");
    let http = dir.join("items-get.yaml");
    std::fs::write(
        &http,
        format!(
            "type: ess-http/1\nspecification:\n  system: items\n  version: v1\n  source_digest: \
             sha256:{}\nroutes:\n  \
             - {{view: items.catalog.AllItems, method: get, path: /items}}\n  \
             - {{command: items.catalog.AddItem, method: post, path: /items}}\n  \
             - command: items.catalog.RenameItem\n    method: get\n    \
             path: /items/{{item_id}}/titles/{{title}}\n    \
             parameters: {{item_id: input.item_id, title: input.title}}\n",
            digest(&dir)
        ),
    )
    .expect("the binding is written");
    let out = dir.join("react");
    let model = items("system.yaml");
    let document = items("ui.yaml");
    let generated = ess(&[
        "generate",
        "ui",
        "--target",
        "react",
        "--path",
        utf8(&document),
        "--model",
        utf8(&model),
        "--out",
        utf8(&out),
        "--http",
        utf8(&http),
    ]);
    assert_eq!(
        generated.status.code(),
        Some(0),
        "{ROUTES}: the binding is admitted: {}",
        String::from_utf8_lossy(&generated.stderr)
    );
    support::compile(&out, &["binding.ts", "runtime/data.ts"]);
    let script = r#"
const { binding } = out("binding");
const { httpAdapter } = out("runtime/data");
const calls = [];
const refused = [];
globalThis.fetch = async (url, init) => {
  // What the platform's fetch does first: build the request, which refuses a GET with a body.
  try {
    new Request(url, init);
  } catch (error) {
    refused.push(String(error));
    throw error;
  }
  calls.push([(init && init.method) || "GET", new URL(url).pathname]);
  return new Response("{}", { status: 202, headers: { "content-type": "application/json" } });
};
const adapter = httpAdapter(binding, () => "http://127.0.0.1:9");
(async () => {
  let answer;
  try { answer = await adapter.send("items.catalog.RenameItem", { item_id: 1, title: "Lamp" }); }
  catch (error) { answer = String(error); }
  console.log(JSON.stringify({ calls, refused, answer }));
})();
"#;
    let printed = support::node_stdout(&out, "get-command", script);
    let value: serde_json::Value = serde_json::from_str(printed.lines().last().unwrap_or_default())
        .unwrap_or_else(|error| panic!("{error}: {printed}"));
    assert_eq!(
        value["calls"],
        serde_json::json!([["GET", "/items/1/titles/Lamp"]]),
        "{ROUTES}: the React runtime sends a command the binding serves on `get`: {value}"
    );
}
