//! Security pass, ESS W1 unit UI1: integrity conformance checks of the unit's own contract.
//!
//! - `story:a-generated-ui-calls-the-routes-its-http-binding-serves`: each path variable a route
//!   carries is filled from data (a row, a form) and must stay one segment of the route the binding
//!   declares, so a request never lands on another route. The cases drive the generated runtimes
//!   (the TUI's `HttpAdapter`, React's routed `httpAdapter` under `node` with the real `fetch`)
//!   against a recording listener on `127.0.0.1:0` and compare the request target the listener
//!   received with the route's template, as the generated server's matcher does (one segment per
//!   template segment, each variable percent-decoding to the value sent).
//! - `story:account-menu-entries-keep-sets-and-navigate`, acceptance 2: `ess ui check` refuses any
//!   declared key a renderer drops. The cases declare `sets` on an account menu entry that the
//!   React menu then does not apply, and require that either the check refuses the entry or the
//!   React menu applies the state.

#[path = "../../../ui/ess-ui-react/tests/support/mod.rs"]
mod support;

use std::collections::BTreeMap;
use std::io::{Read as _, Write as _};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::{Arc, Mutex};

use ess_ui::binding::Binding;

const ROUTES: &str = "story:a-generated-ui-calls-the-routes-its-http-binding-serves";
const MENU: &str = "story:account-menu-entries-keep-sets-and-navigate";

fn ess(arguments: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ess"))
        .args(arguments)
        .env_remove("ESS_UI_AUTHORIZATION")
        .stdin(Stdio::null())
        .output()
        .expect("the ess binary runs")
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

fn utf8(path: &Path) -> &str {
    path.to_str().expect("the scratch path is UTF-8")
}

fn scratch(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("adversary-w1-ui1-security")
        .join(name);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("a stale scratch directory is removed");
    }
    std::fs::create_dir_all(&dir).expect("the scratch directory is created");
    dir
}

fn fixture(path: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/ui-http")
        .join(path)
}

// ── a recording listener ────────────────────────────────────────────────────────────────────

/// Accepts every connection on `127.0.0.1:0`, records each request line and answers `202`.
struct Recorder {
    base: String,
    lines: Arc<Mutex<Vec<String>>>,
}

impl Recorder {
    fn start() -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("a local port");
        let base = format!("http://{}", listener.local_addr().expect("an address"));
        let lines = Arc::new(Mutex::new(Vec::new()));
        let seen = Arc::clone(&lines);
        std::thread::spawn(move || {
            for stream in listener.incoming() {
                let Ok(mut stream) = stream else { continue };
                let mut read = Vec::new();
                let mut buffer = [0_u8; 4096];
                let head_end = loop {
                    let Ok(n) = stream.read(&mut buffer) else {
                        break None;
                    };
                    if n == 0 {
                        break None;
                    }
                    read.extend_from_slice(&buffer[..n]);
                    if let Some(at) = read.windows(4).position(|w| w == b"\r\n\r\n") {
                        break Some(at + 4);
                    }
                };
                let Some(head_end) = head_end else { continue };
                let head = String::from_utf8_lossy(&read[..head_end]).into_owned();
                let length = head
                    .lines()
                    .find_map(|line| {
                        let (name, value) = line.split_once(':')?;
                        name.eq_ignore_ascii_case("content-length")
                            .then(|| value.trim().parse::<usize>().ok())?
                    })
                    .unwrap_or(0);
                while read.len() < head_end + length {
                    match stream.read(&mut buffer) {
                        Ok(0) | Err(_) => break,
                        Ok(n) => read.extend_from_slice(&buffer[..n]),
                    }
                }
                seen.lock()
                    .expect("the record")
                    .push(head.lines().next().unwrap_or_default().to_owned());
                let body = r#"{"outcome":"removed"}"#;
                let _ = write!(
                    stream,
                    "HTTP/1.1 202 Accepted\r\nContent-Type: application/json\r\nContent-Length: \
                     {}\r\nConnection: close\r\n\r\n{body}",
                    body.len()
                );
                let _ = stream.flush();
            }
        });
        Self { base, lines }
    }

    fn lines(&self) -> Vec<String> {
        self.lines.lock().expect("the record").clone()
    }
}

/// The route every path case sends: a command removing one item of one list.
const TEMPLATE: &str = "/lists/{list}/items/{item}";

/// A route table holding only [`TEMPLATE`], bound to `DELETE`, both variables carried.
fn table() -> serde_json::Value {
    serde_json::json!({
        "system": "probe",
        "names": {},
        "components": {
            "probe": {
                "views": {},
                "commands": {
                    "probe.lists.RemoveItem": {
                        "path": TEMPLATE,
                        "method": "DELETE",
                        "path_params": {"list": "list", "item": "item"},
                        "body_required": false,
                        "errors": {}
                    }
                }
            }
        }
    })
}

/// `text` with every `%XX` decoded, as the generated server's path matcher decodes a segment.
fn decoded(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut out = Vec::new();
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' && index + 3 <= bytes.len() {
            let hex = std::str::from_utf8(&bytes[index + 1..index + 3]).unwrap_or("zz");
            if let Ok(byte) = u8::from_str_radix(hex, 16) {
                out.push(byte);
                index += 3;
                continue;
            }
        }
        out.push(bytes[index]);
        index += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// Where the request line `line` sent for `(list, item)` left the route: `None` when its target
/// has [`TEMPLATE`]'s four segments, the literals in place and each variable decoding to the value
/// sent; else why not.
fn off_route(line: &str, list: &str, item: &str) -> Option<String> {
    let mut parts = line.split(' ');
    let method = parts.next().unwrap_or_default();
    let target = parts.next().unwrap_or_default();
    let path = target.split('?').next().unwrap_or_default();
    let segments: Vec<&str> = path.strip_prefix('/').unwrap_or(path).split('/').collect();
    let expected = ["lists", list, "items", item];
    if method != "DELETE" {
        return Some(format!("method `{method}` (the binding says DELETE)"));
    }
    if segments.len() != expected.len() {
        return Some(format!(
            "target `{target}` has {} segments, the route {TEMPLATE} has 4",
            segments.len()
        ));
    }
    let got: Vec<String> = segments.iter().map(|segment| decoded(segment)).collect();
    (got != expected).then(|| format!("target `{target}` decodes to {got:?}, sent {expected:?}"))
}

/// Values a row or a form may hold that carry URL syntax: each must stay one segment.
const RESERVED: &[&str] = &["a/b", "a?b=c", "a#b", "100%", "{list}", "x y", "a\r\nX: y"];

/// Values that are a dot segment: a URL resolver removes them (RFC 3986 5.2.4, and the WHATWG URL
/// parser `fetch` uses, including their `%2e` spellings), so the request lands on another route.
const DOTS: &[&str] = &["..", "."];

// ── the terminal's adapter ──────────────────────────────────────────────────────────────────

#[test]
fn the_terminal_fills_each_path_variable_as_one_segment_of_its_route() {
    let recorder = Recorder::start();
    let binding: Binding = serde_json::from_value(table()).expect("a route table");
    let bases = BTreeMap::from([("probe".to_owned(), recorder.base.clone())]);
    let adapter =
        ess_ui_tui::HttpAdapter::new(binding, &bases, None).expect("the adapter is built");
    let mut cases = Vec::new();
    for value in RESERVED.iter().chain(DOTS) {
        for (list, item) in [("L1", *value), (*value, "I1")] {
            let input = BTreeMap::from([
                ("list".to_owned(), ess_yaml::Value::String(list.to_owned())),
                ("item".to_owned(), ess_yaml::Value::String(item.to_owned())),
            ]);
            let before = recorder.lines().len();
            let sent = adapter.post("probe.lists.RemoveItem", &input);
            let lines = recorder.lines();
            match (sent, lines.get(before)) {
                (Ok(_), Some(line)) => {
                    if let Some(why) = off_route(line, list, item) {
                        cases.push(format!("({list:?}, {item:?}): {why}"));
                    }
                }
                // A refusal before sending keeps the request on no route at all.
                (Err(_), None) => {}
                (sent, line) => cases.push(format!("({list:?}, {item:?}): {sent:?} {line:?}")),
            }
        }
    }
    assert!(
        cases.is_empty(),
        "{ROUTES}: the terminal sent a request off the route {TEMPLATE}:\n{}",
        cases.join("\n")
    );
}

// ── React's routed adapter ──────────────────────────────────────────────────────────────────

/// The model's source digest, as its generated schemas' provenance carries it.
fn digest(dir: &Path, model: &Path) -> String {
    let out = dir.join("schema");
    let generated = ess(&[
        "generate",
        "--path",
        utf8(model),
        "--kind",
        "schema",
        "--out",
        utf8(&out),
    ]);
    assert!(generated.status.success(), "{}", text(&generated.stderr));
    let mut pending = vec![out];
    let mut files = Vec::new();
    while let Some(dir) = pending.pop() {
        for entry in std::fs::read_dir(&dir).expect("the schema directory") {
            let path = entry.expect("an entry").path();
            if path.is_dir() {
                pending.push(path);
            } else {
                files.push(path);
            }
        }
    }
    files.sort();
    for path in files {
        let Ok(bytes) = std::fs::read(&path) else {
            continue;
        };
        let Ok(schema) = serde_json::from_slice::<serde_json::Value>(&bytes) else {
            continue;
        };
        if let Some(digest) = schema["x-ess-provenance"]["source_digest"].as_str() {
            return digest.trim_start_matches("sha256:").to_owned();
        }
    }
    panic!("no generated schema carries a source digest")
}

/// The React app `ess generate ui --target react --http` writes for the `items` toy, whose
/// binding moves a method and carries a path parameter, so its runtime is the routed one.
fn routed_react(dir: &Path) -> PathBuf {
    let model = fixture("items/system.yaml");
    let routes = std::fs::read_to_string(fixture("items/routes.yaml")).expect("the routes read");
    let http = dir.join("items-http.yaml");
    std::fs::write(
        &http,
        format!(
            "type: ess-http/1\nspecification:\n  system: items\n  version: v1\n  source_digest: \
             sha256:{}\n{routes}",
            digest(dir, &model)
        ),
    )
    .expect("the binding is written");
    let out = dir.join("react");
    let generated = ess(&[
        "generate",
        "ui",
        "--target",
        "react",
        "--path",
        utf8(&fixture("items/ui.yaml")),
        "--model",
        utf8(&model),
        "--http",
        utf8(&http),
        "--out",
        utf8(&out),
    ]);
    assert_eq!(
        generated.status.code(),
        Some(0),
        "{}",
        text(&generated.stderr)
    );
    let data = std::fs::read_to_string(out.join("src/runtime/data.ts")).expect("data.ts");
    assert!(data.contains("function routePath"), "the routed runtime");
    support::compile(&out, &["runtime/data.ts"]);
    out
}

/// Sends `probe.lists.RemoveItem` through React's routed `httpAdapter` and the real `fetch` for
/// each `(list, item)`; returns, per case, whether the adapter refused before sending.
fn drive_react(out: &Path, base: &str, cases: &[(&str, &str)]) -> Vec<bool> {
    let script = format!(
        r#"
const {{ httpAdapter }} = out("runtime/data");
const adapter = httpAdapter({table}, () => {base:?});
const sent = [];
(async () => {{
  for (const [list, item] of {cases}) {{
    let refused = false;
    const real = globalThis.fetch;
    let fetched = false;
    globalThis.fetch = (url, init) => {{ fetched = true; return real(url, init); }};
    try {{ await adapter.send("probe.lists.RemoveItem", {{ list, item }}); }} catch (error) {{}}
    globalThis.fetch = real;
    refused = !fetched;
    sent.push(refused);
  }}
  console.log(JSON.stringify(sent));
}})().catch((error) => {{ console.error(String(error)); process.exit(1); }});
"#,
        table = table(),
        cases = serde_json::to_string(cases).expect("JSON"),
    );
    let printed = support::node_stdout(out, "dots", &script);
    serde_json::from_str(printed.lines().last().unwrap_or_default())
        .unwrap_or_else(|error| panic!("{error}: {printed}"))
}

/// The request lines `recorder` received for `cases`, one per case React did not refuse, compared
/// with the route.
fn react_off_route(values: &[&str]) -> Vec<String> {
    let dir = scratch(&format!("react-{}", values.len()));
    let out = routed_react(&dir);
    let recorder = Recorder::start();
    let cases: Vec<(&str, &str)> = values
        .iter()
        .flat_map(|value| [("L1", *value), (*value, "I1")])
        .collect();
    let refused = drive_react(&out, &recorder.base, &cases);
    let lines = recorder.lines();
    let mut sent = lines.iter();
    let mut off = Vec::new();
    for ((list, item), refused) in cases.iter().zip(&refused) {
        if *refused {
            continue;
        }
        match sent.next() {
            Some(line) => {
                if let Some(why) = off_route(line, list, item) {
                    off.push(format!("({list:?}, {item:?}): {why}"));
                }
            }
            None => off.push(format!("({list:?}, {item:?}): sent, nothing received")),
        }
    }
    off
}

#[test]
fn react_fills_a_path_variable_holding_url_syntax_as_one_segment_of_its_route() {
    let off = react_off_route(RESERVED);
    assert!(
        off.is_empty(),
        "{ROUTES}: React sent a request off the route {TEMPLATE}:\n{}",
        off.join("\n")
    );
}

#[test]
fn react_never_sends_a_dot_segment_value_to_another_route() {
    let off = react_off_route(DOTS);
    assert!(
        off.is_empty(),
        "{ROUTES}: a path variable holding a dot segment reached another route than {TEMPLATE} \
         (the binding's DELETE then acts on the parent resource); refuse such a value before \
         sending, as an empty one is:\n{}",
        off.join("\n")
    );
}

// ── account menu entries: a dropped `sets` is refused or applied ────────────────────────────

/// A document whose account menu holds `entry` and a region shown while the theme is dark.
fn menu_document(entry: &str) -> String {
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
            - {entry}
navigation:
  home: things.list
  sections: [{{name: all, pages: [things.list]}}]
pages:
  things.list:
    kind: detail_page
    title: Things
    sections:
      - name: summary
        reads: {{view: things.All}}
        fields: [title]
"#
    )
}

const MENU_HARNESS: &str = r#"
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
const dark = () => root.find((n) => n.props && n.props["data-ui-path"] === "shells/app/regions/dark").length;
(async () => {
  await settle();
  const menu = root.find((n) => n.tag === "button" && n.props.className === "ui-button" && textOf(n) === "Account");
  menu[0].props.onClick();
  await settle();
  const entry = root.find((n) => n.tag === "button" && n.props.className === "ui-menu-item" && textOf(n) === "Dark theme");
  assert.strictEqual(entry.length, 1, `one entry: ${root.text()}`);
  entry[0].props.onClick();
  await settle();
  console.log(JSON.stringify({ dark: dark() }));
})().catch((error) => { console.error(error); process.exitCode = 1; });
"#;

/// For a document whose only account menu entry is `entry`, labelled `Dark theme` and setting
/// `shell.theme` to dark: the `account_menu` findings `ess ui check` reports, and whether the React
/// menu's click turned the theme dark.
fn menu_case(name: &str, entry: &str) -> (Vec<String>, bool) {
    let dir = scratch(&format!("menu-{name}"));
    let source = menu_document(entry);
    let path = dir.join("ui.yaml");
    std::fs::write(&path, &source).expect("the document is written");
    let checked = ess(&["ui", "check", "--path", utf8(&path), "--format", "json"]);
    let report: serde_json::Value = serde_json::from_slice(&checked.stdout)
        .unwrap_or_else(|error| panic!("{error}: {}", text(&checked.stderr)));
    let findings: Vec<String> = report["findings"]
        .as_array()
        .expect("a findings list")
        .iter()
        .filter(|finding| finding["severity"] == "error")
        .filter(|finding| {
            finding["path"]
                .as_str()
                .unwrap_or_default()
                .starts_with("shells/app/regions/account")
        })
        .map(ToString::to_string)
        .collect();
    let document = ess_ui::load_str(&source).unwrap_or_else(|error| panic!("{error}"));
    let out = dir.join("react");
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    // story:account-menu-entries-keep-sets-and-navigate: the emitter refuses an entry that does not
    // read as an action. That refusal is loud, so it counts as refused, but only where the check
    // refuses the entry too: a generate refusal of an entry the check admits stays a failure.
    if let Err(error) = ess_ui_react::generate(&document, &root, &out) {
        assert!(
            !findings.is_empty(),
            "story:account-menu-entries-keep-sets-and-navigate: the React emitter refused an entry \
             `ess ui check` admits: {error}"
        );
        return (findings, false);
    }
    support::compile(&out, &["App.tsx"]);
    let printed = support::node_stdout(&out, name, MENU_HARNESS);
    let shown: serde_json::Value = serde_json::from_str(printed.lines().last().unwrap_or_default())
        .unwrap_or_else(|error| panic!("{error}: {printed}"));
    (findings, shown["dark"] == 1)
}

#[test]
fn an_account_menu_entry_whose_sets_react_does_not_apply_beside_does_is_refused_by_the_check() {
    let (findings, applied) = menu_case(
        "does-and-sets",
        r#"{name: dark, label: Dark theme, does: things.SignOut, sets: {shell.theme: "'dark'"}}"#,
    );
    assert!(
        applied || !findings.is_empty(),
        "{MENU}: the entry declares `sets` beside `does`; the React menu ran only `does` and left \
         the theme light, and `ess ui check` refused nothing at the entry"
    );
}

#[test]
fn an_account_menu_entry_whose_navigate_is_not_an_action_navigate_keeps_its_sets_or_is_refused() {
    let (findings, applied) = menu_case(
        "navigate-shorthand",
        r#"{name: dark, label: Dark theme, sets: {shell.theme: "'dark'"}, navigate: things.list}"#,
    );
    assert!(
        applied || !findings.is_empty(),
        "{MENU}: the entry declares `sets` and a `navigate` written as a page name; the React \
         emitter reads the entry as an `Action`, that read fails on `navigate`, and both `sets` and \
         `navigate` are dropped, while `ess ui check` refused nothing at the entry"
    );
}
