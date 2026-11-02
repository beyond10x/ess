//! `ess generate ui --target react|tui --http <binding>` and `ess ui run --http <binding>`: a
//! generated UI calls the routes an `ess-http/1` binding serves (beyond10x/ess#713).
//!
//! Two toy specifications under `tests/fixtures/ui-http/`: `shelf/` is the one #713 reports, its
//! binding moving every route off the default surface; `items/` adds a command bound to `put`
//! with a path parameter. Each binding's `routes.yaml` is written below a header carrying the
//! model's own source digest. The served cases follow `generate_ui_tui.rs`: the Rust server
//! `ess generate synthesize --layout crate --http` writes for the same binding is built into a
//! target directory of its own under `CARGO_TARGET_TMPDIR`, started on `127.0.0.1:0`, and driven
//! through the generated route tables: the terminal's by `ess_ui_tui::HttpAdapter`, React's by
//! its `httpAdapter` under `node`.

#[path = "../../../ui/ess-ui-react/tests/support/mod.rs"]
mod support;

use std::collections::BTreeMap;
use std::io::BufRead as _;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Output, Stdio};

use ess_ui::binding::Binding;
use ess_yaml::Value;
use serde_json::json;

const STORY: &str = "story:a-generated-ui-calls-the-routes-its-http-binding-serves";

fn fixture(path: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/ui-http")
        .join(path)
}

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
        .join("generate-ui-http")
        .join(name);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("a stale scratch directory is removed");
    }
    std::fs::create_dir_all(&dir).expect("the scratch directory is created");
    dir
}

/// One toy specification: its fixture directory, its model path and its system.
#[derive(Clone, Copy)]
struct Toy {
    dir: &'static str,
    model: &'static str,
    system: &'static str,
}

const SHELF: Toy = Toy {
    dir: "shelf",
    model: "shelf/spec",
    system: "shelf",
};

const ITEMS: Toy = Toy {
    dir: "items",
    model: "items/system.yaml",
    system: "items",
};

impl Toy {
    fn model(self) -> PathBuf {
        fixture(self.model)
    }

    fn document(self) -> PathBuf {
        fixture(&format!("{}/ui.yaml", self.dir))
    }

    fn routes(self) -> String {
        std::fs::read_to_string(fixture(&format!("{}/routes.yaml", self.dir)))
            .expect("the routes read")
    }

    /// The model's source digest, as its generated schemas' provenance carries it.
    fn digest(self, dir: &Path) -> String {
        let out = dir.join(format!("{}-schema", self.dir));
        let model = self.model();
        let generated = ess(&[
            "generate",
            "--path",
            utf8(&model),
            "--kind",
            "schema",
            "--out",
            utf8(&out),
        ]);
        assert!(generated.status.success(), "{}", text(&generated.stderr));
        for text in files(&out).into_values() {
            let Ok(schema) = serde_json::from_slice::<serde_json::Value>(&text) else {
                continue;
            };
            if let Some(digest) = schema["x-ess-provenance"]["source_digest"].as_str() {
                return digest.trim_start_matches("sha256:").to_owned();
            }
        }
        panic!(
            "no generated schema of {} carries a source digest",
            self.dir
        )
    }

    /// Writes `routes` as an `ess-http/1` binding of this model, named `name`, under `dir`.
    fn binding(self, dir: &Path, name: &str, routes: &str) -> PathBuf {
        let path = dir.join(name);
        let digest = self.digest(dir);
        std::fs::write(
            &path,
            format!(
                "type: ess-http/1\nspecification:\n  system: {}\n  version: v1\n  source_digest: \
                 sha256:{digest}\n{routes}",
                self.system
            ),
        )
        .expect("the binding is written");
        path
    }
}

/// `ess generate ui --target <target>` of `document` into `dir/<target>`, with `--http` when
/// given; returns the run and its output directory.
fn generate(
    dir: &Path,
    target: &str,
    toy: Toy,
    document: &Path,
    http: Option<&Path>,
) -> (Output, PathBuf) {
    let out = dir.join(target);
    let model = toy.model();
    let mut arguments = vec![
        "generate",
        "ui",
        "--target",
        target,
        "--path",
        utf8(document),
        "--model",
        utf8(&model),
        "--out",
        utf8(&out),
    ];
    if let Some(http) = http {
        arguments.extend(["--http", utf8(http)]);
    }
    (ess(&arguments), out)
}

/// Every file under `dir`, by its path relative to `dir`.
fn files(dir: &Path) -> BTreeMap<String, Vec<u8>> {
    fn walk(root: &Path, dir: &Path, into: &mut BTreeMap<String, Vec<u8>>) {
        let mut entries: Vec<_> = std::fs::read_dir(dir)
            .unwrap_or_else(|error| panic!("{}: {error}", dir.display()))
            .map(|entry| entry.expect("a directory entry").path())
            .collect();
        entries.sort();
        for path in entries {
            if path.is_dir() {
                walk(root, &path, into);
            } else {
                let relative = path
                    .strip_prefix(root)
                    .expect("under the root")
                    .to_string_lossy()
                    .replace('\\', "/");
                into.insert(relative, std::fs::read(&path).expect("readable"));
            }
        }
    }
    let mut into = BTreeMap::new();
    walk(dir, dir, &mut into);
    into
}

/// A Rust string literal's text, as `ess-ui-tui`'s generator escapes it.
fn unescape(literal: &str) -> String {
    let mut out = String::new();
    let mut chars = literal.chars();
    while let Some(character) = chars.next() {
        if character != '\\' {
            out.push(character);
            continue;
        }
        match chars.next() {
            Some('u') => {
                let code: String = chars
                    .by_ref()
                    .skip(1)
                    .take_while(|digit| *digit != '}')
                    .collect();
                let code = u32::from_str_radix(&code, 16).expect("a hex escape");
                out.push(char::from_u32(code).expect("a scalar value"));
            }
            Some(other) => out.push(other),
            None => {}
        }
    }
    out
}

/// The route table a generated app carries, as JSON: the literal in the terminal app's
/// `src/binding.rs` or the React app's `src/binding.ts`.
fn table(target: &str, out: &Path) -> serde_json::Value {
    let file = if target == "tui" {
        "src/binding.rs"
    } else {
        "src/binding.ts"
    };
    let source = std::fs::read_to_string(out.join(file)).expect("the route table reads");
    let literal = if target == "tui" {
        let start = source.find("= \"").expect("a string literal") + 3;
        let end = source.rfind("\";").expect("a string literal");
        unescape(&source[start..end])
    } else {
        let start = source.find("= {").expect("a JSON object") + 2;
        let end = source.rfind('}').expect("a JSON object");
        source[start..=end].to_owned()
    };
    serde_json::from_str(&literal).unwrap_or_else(|error| panic!("{error}:\n{source}"))
}

#[test]
fn with_http_both_route_tables_carry_the_bindings_paths_methods_and_parameters() {
    let dir = scratch("tables");
    for toy in [SHELF, ITEMS] {
        let http = toy.binding(&dir, &format!("{}.yaml", toy.dir), &toy.routes());
        let expected = if toy.dir == "shelf" {
            json!({"shelf-service": {
                "views": {
                    "shelf.items.AllItems": {"path": "/items"},
                    "shelf.items.AllStamps": {"path": "/stamps"},
                },
                "commands": {
                    "shelf.items.AddItem": {"path": "/items"},
                    "shelf.items.AddStamp": {"path": "/stamps"},
                },
            }})
        } else {
            json!({"items": {
                "views": {"items.catalog.AllItems": {"path": "/items"}},
                "commands": {
                    "items.catalog.AddItem": {"path": "/items"},
                    "items.catalog.RenameItem": {
                        "path": "/items/{item_id}",
                        "method": "PUT",
                        "path_params": {"item_id": "item_id"},
                    },
                },
            }})
        };
        for target in ["tui", "react"] {
            let (output, out) = generate(
                &dir.join(toy.dir),
                target,
                toy,
                &toy.document(),
                Some(&http),
            );
            assert_eq!(
                output.status.code(),
                Some(0),
                "{STORY}: {} {target}: {}",
                toy.dir,
                text(&output.stderr)
            );
            // Each route's path, method and path parameters as the binding declares them; the
            // method is absent at its default, `GET` for a view and `POST` for a command.
            let table = table(target, &out);
            let mut seen = json!({});
            for (component, served) in table["components"].as_object().expect("components") {
                for kind in ["views", "commands"] {
                    for (name, route) in served[kind].as_object().expect("routes") {
                        let mut kept = json!({"path": route["path"]});
                        for member in ["method", "path_params"] {
                            if let Some(value) = route.get(member) {
                                kept[member] = value.clone();
                            }
                        }
                        seen[component][kind][name] = kept;
                    }
                }
            }
            assert_eq!(seen, expected, "{STORY}: {} {target}", toy.dir);
        }
    }
}

/// The routes ESS derives for the shelf, restated as a binding.
const SHELF_DERIVED: &str = "routes:\n  \
    - {view: shelf.items.AllItems, method: get, path: /items/views/AllItems}\n  \
    - {view: shelf.items.AllStamps, method: get, path: /items/views/AllStamps}\n  \
    - {command: shelf.items.AddItem, method: post, path: /items/commands/add-item}\n  \
    - {command: shelf.items.AddStamp, method: post, path: /items/commands/add-stamp}\n";

#[test]
fn without_http_the_generated_bytes_are_unchanged() {
    let dir = scratch("unchanged");
    let restated = SHELF.binding(&dir, "derived.yaml", SHELF_DERIVED);
    for target in ["tui", "react"] {
        let (unbound, unbound_out) =
            generate(&dir.join("unbound"), target, SHELF, &SHELF.document(), None);
        assert_eq!(unbound.status.code(), Some(0), "{}", text(&unbound.stderr));
        let (bound, bound_out) = generate(
            &dir.join("restated"),
            target,
            SHELF,
            &SHELF.document(),
            Some(&restated),
        );
        assert_eq!(
            bound.status.code(),
            Some(0),
            "{STORY}: {}",
            text(&bound.stderr)
        );
        // A binding that restates every derived route generates the bytes no binding does.
        let unbound_files = files(&unbound_out);
        assert_eq!(unbound_files, files(&bound_out), "{STORY}: {target}");
        let table = table(target, &unbound_out).to_string();
        for member in ["\"method\"", "\"path_params\""] {
            assert!(
                !table.contains(member),
                "{STORY}: {target}: a route table without --http carries no {member}: {table}"
            );
        }
    }
}

#[test]
fn a_view_or_command_the_binding_does_not_route_is_refused_naming_it_and_the_binding() {
    let dir = scratch("unrouted");
    let without_view = SHELF.binding(
        &dir,
        "no-stamp-view.yaml",
        "routes:\n  - {view: shelf.items.AllItems, method: get, path: /items}\n  \
         - {command: shelf.items.AddItem, method: post, path: /items}\n  \
         - {command: shelf.items.AddStamp, method: post, path: /stamps}\n",
    );
    let without_command = SHELF.binding(
        &dir,
        "no-stamp-command.yaml",
        "routes:\n  - {view: shelf.items.AllItems, method: get, path: /items}\n  \
         - {view: shelf.items.AllStamps, method: get, path: /stamps}\n  \
         - {command: shelf.items.AddItem, method: post, path: /items}\n",
    );
    for (http, unrouted) in [
        (&without_view, "shelf.items.AllStamps"),
        (&without_command, "shelf.items.AddStamp"),
    ] {
        let file = http
            .file_name()
            .and_then(|name| name.to_str())
            .expect("a name");
        for target in ["tui", "react"] {
            let (output, out) = generate(
                &dir.join(file),
                target,
                SHELF,
                &SHELF.document(),
                Some(http),
            );
            let stderr = text(&output.stderr);
            assert_eq!(output.status.code(), Some(1), "{STORY}: {target}: {stderr}");
            assert!(
                stderr.contains(unrouted) && stderr.contains(file),
                "{STORY}: {target}: the refusal names {unrouted} and {file}: {stderr}"
            );
            assert!(!out.exists(), "{STORY}: {target}: nothing is written");
        }
        // `ess ui run --http` reads the same binding and refuses the same way, before it draws.
        let model = SHELF.model();
        let document = SHELF.document();
        let output = ess(&[
            "ui",
            "run",
            "--tui",
            "--path",
            utf8(&document),
            "--model",
            utf8(&model),
            "--http",
            utf8(http),
            "--base-url",
            "http://127.0.0.1:9",
        ]);
        let stderr = text(&output.stderr);
        assert_eq!(output.status.code(), Some(1), "{STORY}: ui run: {stderr}");
        assert!(
            stderr.contains(unrouted) && stderr.contains(file),
            "{STORY}: ui run: the refusal names {unrouted} and {file}: {stderr}"
        );
    }
}

#[test]
fn http_needs_model() {
    let document = SHELF.document();
    let out = scratch("needs-model");
    let output = ess(&[
        "generate",
        "ui",
        "--target",
        "react",
        "--path",
        utf8(&document),
        "--out",
        utf8(&out.join("react")),
        "--http",
        utf8(&fixture("shelf/routes.yaml")),
    ]);
    let stderr = text(&output.stderr);
    assert_eq!(output.status.code(), Some(2), "{STORY}: {stderr}");
    assert!(stderr.contains("--model"), "{STORY}: {stderr}");
}

// ── the served surface ────────────────────────────────────────────────────────────────────────

/// Kills the server when the case ends, pass or fail.
struct Served(Child);

impl Drop for Served {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

const CLERK: &str = "Actor Clerk";

/// The Rust server `ess generate synthesize --layout crate --http` writes for `toy` bound by
/// `http`, built into a target directory both toys share; returns the binary.
fn server(dir: &Path, toy: Toy, component: &str, http: &Path) -> PathBuf {
    let root = dir.join("server");
    let model = toy.model();
    let synthesized = ess(&[
        "generate",
        "synthesize",
        "--target",
        "rust",
        "--layout",
        "crate",
        "--path",
        utf8(&model),
        "--http",
        utf8(http),
        "--out",
        utf8(&root),
    ]);
    assert!(
        synthesized.status.success(),
        "{}{}",
        text(&synthesized.stdout),
        text(&synthesized.stderr)
    );
    let name = format!("{component}-server");
    let target = Path::new(env!("CARGO_TARGET_TMPDIR")).join("generate-ui-http-target");
    let built = Command::new(std::env::var_os("CARGO").expect("Cargo supplies its executable"))
        .args([
            "build",
            "--offline",
            "--quiet",
            "--features",
            "server",
            "--bin",
            &name,
            "--target-dir",
        ])
        .arg(&target)
        .current_dir(&root)
        .env_remove("CARGO_TARGET_DIR")
        .env_remove("CARGO_ENCODED_RUSTFLAGS")
        .env_remove("RUSTC_WRAPPER")
        .env("CARGO_INCREMENTAL", "0")
        .output()
        .expect("cargo runs");
    assert!(
        built.status.success(),
        "the synthesized server builds: {}",
        text(&built.stderr)
    );
    target.join("debug").join(name)
}

/// Starts `binary` on an ephemeral port of `127.0.0.1`, callers named by their `Authorization`
/// header; returns it and its base URL.
fn serve(binary: &Path) -> (Served, String) {
    let mut child = Command::new(binary)
        .args(["--listen", "127.0.0.1:0", "--callers", "actor-header"])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("the server starts");
    let stdout = child.stdout.take().expect("piped");
    let served = Served(child);
    let mut lines = std::io::BufReader::new(stdout).lines();
    let address = loop {
        let line = lines
            .next()
            .expect("a ready record before the server stops")
            .expect("text");
        let Ok(record) = serde_json::from_str::<serde_json::Value>(&line) else {
            continue;
        };
        if record["event"] == "system.ready" {
            break record["runtime"]["address"]
                .as_str()
                .unwrap_or_else(|| panic!("the ready record names the address: {line}"))
                .to_owned();
        }
    };
    std::thread::spawn(move || lines.for_each(drop));
    (served, format!("http://{address}"))
}

/// Drives the terminal app's route table: each command with its input (YAML), then each view;
/// returns every command's status and the rows each view read.
fn drive_terminal(
    out: &Path,
    component: &str,
    base: &str,
    sends: &[(&str, &str)],
    reads: &[&str],
) -> (Vec<u16>, Vec<usize>) {
    let binding: Binding = serde_json::from_value(table("tui", out)).expect("a binding");
    let bases = BTreeMap::from([(component.to_owned(), base.to_owned())]);
    let adapter = ess_ui_tui::HttpAdapter::new(binding, &bases, Some(CLERK.to_owned()))
        .expect("the adapter is built");
    let mut statuses = Vec::new();
    for (command, input) in sends {
        let input: BTreeMap<String, Value> = ess_yaml::from_str(input).expect("the input parses");
        let (status, body) = adapter
            .post(command, &input)
            .unwrap_or_else(|error| panic!("{STORY}: terminal {command}: {error}"));
        assert_eq!(
            ess_ui::binding::classify(status, &body),
            ess_ui::binding::Answer::Accepted,
            "{STORY}: terminal {command}: {status} {body}"
        );
        statuses.push(status);
    }
    let mut rows = Vec::new();
    for view in reads {
        let read = ess_ui_tui::DataAdapter::read(
            &adapter,
            &ess_ui_tui::ReadRequest {
                view: (*view).to_owned(),
                fixture: None,
                params: BTreeMap::new(),
            },
        )
        .unwrap_or_else(|error| panic!("{STORY}: terminal {view}: {error}"));
        rows.push(read.rows.len());
    }
    (statuses, rows)
}

/// Drives the React app's route table under `node` with the real `fetch`: each command with its
/// input, then each view; returns every call's method, path and status, and the rows each view
/// read.
fn drive_react(
    out: &Path,
    base: &str,
    sends: &[(&str, serde_json::Value)],
    reads: &[&str],
) -> (Vec<(String, String, u16)>, Vec<usize>) {
    support::compile(out, &["binding.ts", "runtime/data.ts"]);
    let sends = serde_json::to_string(sends).expect("JSON");
    let reads = serde_json::to_string(reads).expect("JSON");
    let script = format!(
        r#"
const {{ binding }} = out("binding");
const {{ httpAdapter, setAuthorization }} = out("runtime/data");
const calls = [];
const real = globalThis.fetch;
globalThis.fetch = async (url, init) => {{
  const response = await real(url, init);
  calls.push([(init && init.method) || "GET", new URL(url).pathname, response.status]);
  return response;
}};
setAuthorization({CLERK:?});
const adapter = httpAdapter(binding, () => {base:?});
(async () => {{
  for (const [command, input] of {sends}) await adapter.send(command, input);
  const rows = [];
  for (const view of {reads}) rows.push((await adapter.read({{ view, params: {{}} }})).rows.length);
  console.log(JSON.stringify({{ calls, rows }}));
}})().catch((error) => {{ console.error(String(error), JSON.stringify(calls)); process.exit(1); }});
"#
    );
    let printed = support::node_stdout(out, "served", &script);
    let value: serde_json::Value = serde_json::from_str(printed.lines().last().unwrap_or_default())
        .unwrap_or_else(|error| panic!("{error}: {printed}"));
    (
        serde_json::from_value(value["calls"].clone()).expect("the calls"),
        serde_json::from_value(value["rows"].clone()).expect("the rows"),
    )
}

/// The terminal and the React app of `toy`, generated with `--http <http>` under `dir`.
fn generated(dir: &Path, toy: Toy, http: &Path) -> (PathBuf, PathBuf) {
    let mut outs = Vec::new();
    for target in ["tui", "react"] {
        let (output, out) = generate(dir, target, toy, &toy.document(), Some(http));
        assert_eq!(
            output.status.code(),
            Some(0),
            "{STORY}: {target}: {}",
            text(&output.stderr)
        );
        outs.push(out);
    }
    let react = outs.pop().expect("the React app");
    (outs.pop().expect("the terminal app"), react)
}

#[test]
fn both_generated_uis_reach_the_server_generated_with_the_same_binding() {
    let dir = scratch("served-shelf");
    let http = SHELF.binding(&dir, "shelf-http.yaml", &SHELF.routes());
    let (tui, react) = generated(&dir, SHELF, &http);
    let binary = server(&dir, SHELF, "shelf-service", &http);
    let (_server, base) = serve(&binary);

    let (statuses, rows) = drive_terminal(
        &tui,
        "shelf-service",
        &base,
        &[
            (
                "shelf.items.AddItem",
                "{created_at: '2026-11-02T10:00:00Z', labels: [{key: colour, value: red}]}",
            ),
            (
                "shelf.items.AddStamp",
                "{stamped_at: '2026-11-02T10:00:00Z'}",
            ),
        ],
        &["shelf.items.AllItems", "shelf.items.AllStamps"],
    );
    for status in &statuses {
        assert!(matches!(status, 200 | 202), "{STORY}: {statuses:?}");
    }
    assert_eq!(
        rows,
        [1, 1],
        "{STORY}: the terminal reads back what it sent"
    );

    let (calls, rows) = drive_react(
        &react,
        &base,
        &[
            (
                "shelf.items.AddItem",
                json!({"created_at": "2026-11-02T11:00:00Z", "labels": []}),
            ),
            (
                "shelf.items.AddStamp",
                json!({"stamped_at": "2026-11-02T11:00:00Z"}),
            ),
        ],
        &["shelf.items.AllItems", "shelf.items.AllStamps"],
    );
    let routes: Vec<(&str, &str)> = calls
        .iter()
        .map(|(method, path, _)| (method.as_str(), path.as_str()))
        .collect();
    assert_eq!(
        routes,
        [
            ("POST", "/items"),
            ("POST", "/stamps"),
            ("GET", "/items"),
            ("GET", "/stamps"),
        ],
        "{STORY}: {calls:?}"
    );
    for (method, path, status) in &calls {
        assert!(
            matches!(status, 200 | 202),
            "{STORY}: {method} {path}: {status}"
        );
    }
    assert_eq!(rows, [2, 2], "{STORY}: React reads back both apps' sends");
}

#[test]
fn a_command_bound_to_put_with_a_path_parameter_reaches_its_route() {
    let dir = scratch("served-items");
    let http = ITEMS.binding(&dir, "items-http.yaml", &ITEMS.routes());
    let (tui, react) = generated(&dir, ITEMS, &http);
    let binary = server(&dir, ITEMS, "items", &http);
    let (_server, base) = serve(&binary);

    let (statuses, rows) = drive_terminal(
        &tui,
        "items",
        &base,
        &[
            ("items.catalog.AddItem", "{item_id: 1, title: Lamp}"),
            ("items.catalog.RenameItem", "{item_id: 1, title: Desk lamp}"),
        ],
        &["items.catalog.AllItems"],
    );
    for status in &statuses {
        assert!(matches!(status, 200 | 202), "{STORY}: {statuses:?}");
    }
    assert_eq!(rows, [1], "{STORY}");

    let (calls, _) = drive_react(
        &react,
        &base,
        &[
            (
                "items.catalog.AddItem",
                json!({"item_id": 2, "title": "Chair"}),
            ),
            (
                "items.catalog.RenameItem",
                json!({"item_id": 2, "title": "Arm chair"}),
            ),
        ],
        &["items.catalog.AllItems"],
    );
    let routes: Vec<(&str, &str)> = calls
        .iter()
        .map(|(method, path, _)| (method.as_str(), path.as_str()))
        .collect();
    assert_eq!(
        routes,
        [("POST", "/items"), ("PUT", "/items/2"), ("GET", "/items")],
        "{STORY}: {calls:?}"
    );
    for (method, path, status) in &calls {
        assert!(
            matches!(status, 200 | 202),
            "{STORY}: {method} {path}: {status}"
        );
    }
}
