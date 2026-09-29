//! The web target represents `Json` (beyond10x/ess#224).
//!
//! The web target is two halves, and `Json` crosses both:
//!
//! - the **bridge crate** (Rust, built for `wasm32-unknown-unknown`) decodes and encodes through
//!   the Rust target's wire module, so a `Json` value there is the Rust types crate's own
//!   `json::Value`, carried unchanged: members in their order, numbers in their spelling;
//! - the **page** (JavaScript, typed with `JSDoc` so `tsc` checks it) holds a `Json` value as
//!   `JsonValue` — what `JSON.parse` answers — and edits one as JSON text. The page reads every
//!   JSON number as a double already (the `Integer` weakening); a `Json` value is read the same
//!   way, and `TARGET.md` says so.
//!
//! The fixture uses `Json` at every position the target types: a newtype, a struct member (bare,
//! in a list, in a map, optional), a union variant, an entity field, a command input and response,
//! an event payload, an error payload, a view row and a binding-copied field.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use ess_compiler::{resolve::compile, source::SourceMap, EssIr};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_synth::{synthesize_for, Synthesis, Target};

/// `Json` at every position, served by one component so the bridge dispatches its commands.
const EVERY_POSITION: &str = "format: ess/15
system: demo
version: v1
domain: demo.docs
types:
  - {name: demo.docs.Body, kind: newtype, of: Json}
  - name: demo.docs.Envelope
    kind: struct
    fields:
      - {name: body, type: Json}
      - {name: items, type: List<Json>}
      - {name: labels, type: \"Map<String, Json>\"}
      - {name: note, type: Optional<Json>}
  - name: demo.docs.Either
    kind: union
    tag: kind
    variants:
      raw: Json
      text: String
entities:
  - name: demo.docs.Doc
    identity: {name: id, type: String}
    fields:
      - {name: body, type: Json}
    lifecycle:
      initial: Active
      states: [Active]
      terminal: [Active]
events:
  - name: demo.docs.Stored
    fields:
      - {name: id, type: String}
      - {name: body, type: Json}
      - {name: wrapped, type: demo.docs.Body}
  - name: demo.docs.Noted
    fields:
      - {name: id, type: String}
      - {name: body, type: Json}
commands:
  - name: demo.docs.Store
    input:
      - {name: id, type: String}
      - {name: body, type: Json}
      - {name: wrapped, type: demo.docs.Body}
      - {name: envelope, type: demo.docs.Envelope}
      - {name: either, type: demo.docs.Either}
      - {name: items, type: List<Json>}
      - {name: labels, type: \"Map<String, Json>\"}
    response:
      - {name: echo, type: Json}
    outcomes:
      - name: stored
        emits: [demo.docs.Stored]
        payload:
          demo.docs.Stored: {id: input.id, body: input.body, wrapped: input.wrapped}
      - name: refused
        external: the store refuses the document
        error: demo.docs.Refused
  - name: demo.docs.Note
    input:
      - {name: id, type: String}
      - {name: body, type: Json}
    outcomes:
      - name: noted
        emits: [demo.docs.Noted]
        payload:
          demo.docs.Noted: {id: input.id, body: input.body}
errors:
  - name: demo.docs.Refused
    fields:
      - {name: detail, type: Json}
views:
  - name: demo.docs.Docs
    source: demo.docs.Doc
    consistency: eventual
    fields:
      - {name: id, type: String}
      - {name: body, type: Json}
components:
  - component: docs-service
    owns: {domains: [demo.docs]}
    accepts: {commands: [demo.docs.Store, demo.docs.Note]}
    publishes: {events: [demo.docs.Stored, demo.docs.Noted]}
    reached_by: network
bindings:
  - id: note-on-stored
    when: {event: demo.docs.Stored}
    invoke: {command: demo.docs.Note}
    mapping:
      id: event.id
      body: event.body
    delivery: at_least_once
    on_failure: retry
";

/// The same system with every `Json` replaced by `String`: what a model without `Json` emits.
fn without_json() -> String {
    EVERY_POSITION.replace("Json", "String")
}

/// One command input carrying all six JSON kinds at every input position. Compact, and spelled
/// the way the generated writer spells it, so the module must hand it back byte for byte; a
/// `Map<String, Json>` is ordered by key, a `Json` object keeps the order its members arrived in.
const INPUT: &str = r#"{"id":"d1","body":{"object":{"k":"v"},"array":[1,"two",[]],"number":-12.50e3,"string":"say \"hi\"","bool":true,"null":null},"wrapped":[{},[],0,"",false,null],"envelope":{"body":null,"items":[{"a":1},[2],3.0,"four",true,null],"labels":{"a":[],"b":false,"n":1E2,"o":{},"s":"x","z":null},"note":{"deep":[{"deeper":null}]}},"either":{"kind":"raw","value":{"x":[true]}},"items":[null,{"m":"n"}],"labels":{"only":[1,{"two":2}]}}"#;

/// A host crate that installs a realization echoing every `Json` it is given: `Store` publishes
/// its input's `body` and `wrapped`, answers `echo` with its `envelope`'s body, the binding copies
/// `body` into `Note`, and the view projects one row per stored document.
const HOST: &str = r#"//! A realization for the Json round trip: every behaviour copies what it is given.

use std::cell::RefCell;
use std::rc::Rc;

use demo_types::docs::obligations::{DocsQuery, NoteBehavior, StoreBehavior};
use demo_types::docs::{Docs, Note, NoteOutcome, Noted, Refused, Store, StoreOutcome, Stored};
use demo_types::obligation::UnmetObligation;

#[derive(Clone, Default)]
struct Echo {
    rows: Rc<RefCell<Vec<Docs>>>,
}

impl StoreBehavior for Echo {
    fn store(&mut self, input: Store) -> Result<StoreOutcome, UnmetObligation> {
        if input.id == "refuse" {
            return Ok(StoreOutcome::Refused { error: Refused { detail: input.body } });
        }
        self.rows.borrow_mut().push(Docs { id: input.id.clone(), body: input.body.clone() });
        Ok(StoreOutcome::Stored {
            stored: Stored { id: input.id, body: input.body, wrapped: input.wrapped },
        })
    }
}

impl NoteBehavior for Echo {
    fn note(&mut self, input: Note) -> Result<NoteOutcome, UnmetObligation> {
        Ok(NoteOutcome::Noted { noted: Noted { id: input.id, body: input.body } })
    }
}

impl DocsQuery for Echo {
    fn docs(&self) -> Result<Vec<Docs>, UnmetObligation> {
        Ok(self.rows.borrow().clone())
    }
}

/// Installs the echoing realization.
#[no_mangle]
pub extern "C" fn ess_realize() {
    let echo = Echo::default();
    let system = demo_system::System::new(docs_service::DocsService::new(echo));
    demo_web::install(Box::new(system));
}
"#;

/// Drives the realized module through the page's own glue, then through the raw exports.
///
/// Through the glue, a `Json` value is what `JSON.parse` answers, so the six kinds are compared by
/// value. Through the raw exports, the response is text, and the input's `Json` members must be in
/// it exactly as they were sent: member order and number spelling kept by the module.
const DRIVER: &str = r#"import { readFileSync } from "node:fs";
import { deepStrictEqual, ok, strictEqual } from "node:assert/strict";

const [glue, wasm, input] = process.argv.slice(2);
const { open } = await import(glue);
const bytes = readFileSync(wasm);

const system = await open(bytes);
strictEqual(system.realized, true, "the host installs its realization");
const sent = JSON.parse(input);
const answer = system.request({ request: "command", command: "demo.docs.Store", input: sent });
ok(answer.ok, JSON.stringify(answer));
strictEqual(answer.outcome.outcome, "stored", JSON.stringify(answer));

const stored = answer.log.find((entry) => entry.event === "demo.docs.Stored");
ok(stored, JSON.stringify(answer.log));
deepStrictEqual(stored.payload.body, sent.body, "the event carries the Json body");
deepStrictEqual(stored.payload.wrapped, sent.wrapped, "the event carries the Json newtype");
const noted = answer.log.find((entry) => entry.event === "demo.docs.Noted");
ok(noted, JSON.stringify(answer.log));
deepStrictEqual(noted.payload.body, sent.body, "the binding copies the Json field");
const invoked = answer.invocations.find((entry) => entry.binding === "note-on-stored");
ok(invoked, JSON.stringify(answer.invocations));
deepStrictEqual(invoked.input.body, sent.body, "the invocation's input carries it");
deepStrictEqual(answer.views["demo.docs.Docs"].rows, [{ id: "d1", body: sent.body }],
  "the view row carries its Json column");
const kinds = new Set(Object.values(sent.body).map((value) =>
  value === null ? "null" : Array.isArray(value) ? "array" : typeof value));
deepStrictEqual([...kinds].sort(), ["array", "boolean", "null", "number", "object", "string"]);

// The declared error's Json payload, through the refusal the page renders.
const refusal = system.request({ request: "command", command: "demo.docs.Store",
  input: { ...sent, id: "refuse" } });
ok(refusal.ok, JSON.stringify(refusal));
strictEqual(refusal.outcome.outcome, "refused", JSON.stringify(refusal));
deepStrictEqual(refusal.outcome.refusal.payload.detail, sent.body, "the error carries the Json detail");

// The raw text, through the exports alone, on a fresh module.
const fresh = await open(bytes);
const request = `{"request":"command","command":"demo.docs.Store","input":${input}}`;
const encoded = new TextEncoder().encode(request);
const address = fresh.exports.ess_input_reserve(encoded.length);
new Uint8Array(fresh.exports.memory.buffer, address, encoded.length).set(encoded);
const at = fresh.exports.ess_dispatch();
const length = fresh.exports.ess_output_len();
const text = new TextDecoder().decode(new Uint8Array(fresh.exports.memory.buffer, at, length));
const body = input.slice(input.indexOf(`"body":`) + 7, input.indexOf(`,"wrapped":`));
const wrapped = input.slice(input.indexOf(`"wrapped":`) + 10, input.indexOf(`,"envelope":`));
ok(text.includes(`"body":${body}`), `the module keeps the body's spelling:\n${text}`);
ok(text.includes(`"wrapped":${wrapped}`), `the module keeps the newtype's spelling:\n${text}`);
ok(text.includes(`"body":{"object":{"k":"v"},"array":[1,"two",[]],"number":-12.50e3,`), text);

console.log("json round trip: six kinds through the page glue; spelling kept by the module");
"#;

fn ir(source: &str) -> EssIr {
    let spec = Specification::assemble([(
        Source::new("spec.yaml"),
        RawSpecFile::parse(source).expect("well formed"),
    )])
    .unwrap_or_else(|errors| panic!("validates: {errors}"));
    let mut sources = SourceMap::new();
    sources.insert("spec.yaml", source);
    compile(&spec, &sources).unwrap_or_else(|errors| panic!("compiles: {errors}"))
}

fn emit(source: &str, target: Target) -> Synthesis {
    synthesize_for(&ir(source), target)
        .unwrap_or_else(|failure| panic!("the {target:?} target represents Json: {failure}"))
}

fn artifact<'a>(synthesis: &'a Synthesis, path: &str) -> &'a str {
    &synthesis
        .artifacts
        .get(path)
        .unwrap_or_else(|| {
            panic!(
                "`{path}` was not emitted; emitted: {:?}",
                synthesis.artifacts.keys().collect::<Vec<_>>()
            )
        })
        .contents
}

fn scratch(label: &str) -> PathBuf {
    Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("json-web-{label}"))
}

fn write(synthesis: &Synthesis, directory: &Path) {
    let _ = std::fs::remove_dir_all(directory);
    for (relative, artifact) in &synthesis.artifacts {
        let destination = directory.join(relative);
        std::fs::create_dir_all(destination.parent().unwrap()).unwrap();
        std::fs::write(&destination, &artifact.contents).unwrap();
    }
}

/// The Rust and web trees, in the layout the web manifest names the Rust crates by.
fn trees(root: &Path) -> PathBuf {
    write(
        &emit(EVERY_POSITION, Target::Rust),
        &root.join("generated/rust/demo"),
    );
    let web = root.join("generated/web/demo");
    write(&emit(EVERY_POSITION, Target::Web), &web);
    web
}

fn run(mut command: Command, what: &str) -> Output {
    let output = command
        .output()
        .unwrap_or_else(|error| panic!("{what}: {error}"));
    eprintln!(
        "{what}: {command:?}\n{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    output
}

/// Cargo for `wasm32-unknown-unknown`, offline, with every warning an error.
fn cargo_wasm(directory: &Path, verb: &str, target: &Path) -> Output {
    let lock = {
        let mut command =
            Command::new(std::env::var_os("CARGO").expect("Cargo supplies its executable"));
        command
            .args(["generate-lockfile", "--offline"])
            .current_dir(directory);
        run(command, "lock")
    };
    assert!(lock.status.success(), "the lockfile resolves offline");
    let mut command =
        Command::new(std::env::var_os("CARGO").expect("Cargo supplies its executable"));
    command
        .args([
            verb,
            "--locked",
            "--offline",
            "--target",
            "wasm32-unknown-unknown",
            "--target-dir",
        ])
        .arg(target)
        .current_dir(directory)
        .env_remove("CARGO_TARGET_DIR")
        .env_remove("CARGO_ENCODED_RUSTFLAGS")
        .env("RUSTFLAGS", "-D warnings");
    run(command, verb)
}

/// The page's module script, exactly as `index.html` carries it.
fn page_script(page: &str) -> &str {
    let open = "<script type=\"module\">\n";
    let start = page.find(open).expect("the page has a module script") + open.len();
    let end = page[start..]
        .find("\n</script>")
        .expect("the script closes")
        + start;
    &page[start..end]
}

#[test]
fn issue_224_web_names_json_at_every_position() {
    let synthesis = emit(EVERY_POSITION, Target::Web);

    // The bridge's JSON module is the types crate's, so a decoded `Json` is the value a semantic
    // type holds.
    let json = artifact(&synthesis, "crates/demo-web/src/json.rs");
    assert!(json.contains("pub use demo_types::json::*;"), "{json}");
    assert!(!json.contains("pub enum Value"), "{json}");
    let wire = artifact(&synthesis, "crates/demo-web/src/wire.rs");
    assert!(wire.contains("json::push_value(out, "), "{wire}");
    assert!(wire.contains("json::value_at("), "{wire}");

    // The catalogue names `json` at every position the page builds a control for.
    let catalog = artifact(&synthesis, "catalog.json");
    let positions = catalog.matches(r#""name": "json""#).count();
    assert!(
        positions >= 10,
        "every Json position is in the catalogue; found {positions}:\n{catalog}"
    );

    // The page types a Json value and builds a control for it.
    let page = artifact(&synthesis, "index.html");
    let script = page_script(page);
    assert!(
        script.contains(
            "@typedef {null | boolean | number | string | JsonArray | JsonObject} JsonValue"
        ),
        "{script}"
    );
    assert!(
        script.contains("if (name === \"json\") return jsonControl();"),
        "{script}"
    );

    // The limit is stated where the other browser limits are.
    let target = artifact(&synthesis, "TARGET.md");
    assert!(target.contains("a `Json` value"), "{target}");
}

#[test]
fn issue_224_the_web_bridge_builds_for_wasm_with_warnings_as_errors() {
    let root = scratch("build");
    let web = trees(&root);
    let build = cargo_wasm(&web, "check", &scratch("target"));
    assert!(
        build.status.success(),
        "the generated web bridge does not compile for wasm32"
    );
}

#[test]
fn issue_224_the_page_type_checks_with_tsc() {
    // The six kinds are each a `JsonValue`; a function inside an object is not. The second run is
    // what shows `JsonValue` is a type `tsc` enforces rather than an `any` it waves through.
    let six_kinds = "/** @type {JsonValue[]} */\nexport const sixKinds = [{ k: \"v\" }, [1, \
                     \"two\", []], -12.5e3, \"say\", true, null];\n";
    let checked = tsc("tsc", six_kinds);
    assert!(
        checked.status.success(),
        "the generated page does not type-check:\n{}",
        String::from_utf8_lossy(&checked.stdout)
    );
    let refused = tsc(
        "tsc-refuses",
        "/** @type {JsonValue} */\nexport const notJson = { f: () => 1 };\n",
    );
    let stdout = String::from_utf8_lossy(&refused.stdout);
    assert!(!refused.status.success(), "tsc accepted a function as JSON");
    assert!(
        stdout.contains("is not assignable to type 'JsonValue'"),
        "{stdout}"
    );
}

/// `tsc --noEmit` over the page's own script with one probe appended, and the glue beside it as
/// the page imports it. `// @ts-check` and the probe are the harness's, so the generated bytes stay
/// exactly what synthesis emitted.
fn tsc(label: &str, probe: &str) -> Output {
    let synthesis = emit(EVERY_POSITION, Target::Web);
    let directory = scratch(label);
    let _ = std::fs::remove_dir_all(&directory);
    std::fs::create_dir_all(&directory).unwrap();
    std::fs::write(
        directory.join("page.js"),
        format!(
            "// @ts-check\n{}\n{probe}",
            page_script(artifact(&synthesis, "index.html"))
        ),
    )
    .unwrap();
    std::fs::write(
        directory.join("bridge.js"),
        artifact(&synthesis, "bridge.js"),
    )
    .unwrap();
    let mut tsc = Command::new("tsc");
    tsc.current_dir(&directory).args([
        "--noEmit",
        "--allowJs",
        "--strict",
        "false",
        "--target",
        "es2022",
        "--module",
        "es2022",
        "--moduleResolution",
        "bundler",
        "--lib",
        "es2022,dom",
        "page.js",
    ]);
    run(tsc, label)
}

#[test]
fn issue_224_six_kinds_round_trip_through_the_page_glue_and_the_module() {
    let root = scratch("round-trip");
    let web = trees(&root);
    let host = root.join("host");
    std::fs::create_dir_all(host.join("src")).unwrap();
    std::fs::write(
        host.join("Cargo.toml"),
        "[package]\nname = \"json-host\"\nversion = \"0.0.0\"\nedition = \"2021\"\n\n\
         [lib]\ncrate-type = [\"cdylib\"]\n\n[dependencies]\n\
         demo-web = { path = \"../generated/web/demo/crates/demo-web\" }\n\
         demo-types = { path = \"../generated/rust/demo/crates/demo-types\" }\n\
         demo-system = { path = \"../generated/rust/demo/crates/demo-system\" }\n\
         docs-service = { path = \"../generated/rust/demo/crates/docs-service\" }\n\n[workspace]\n",
    )
    .unwrap();
    std::fs::write(host.join("src/lib.rs"), HOST).unwrap();
    let target = scratch("target");
    let build = cargo_wasm(&host, "build", &target);
    assert!(build.status.success(), "the realized host does not build");
    let wasm = target.join("wasm32-unknown-unknown/debug/json_host.wasm");
    std::fs::write(root.join("driver.mjs"), DRIVER).unwrap();
    let glue = format!("file://{}", web.join("bridge.js").display());
    let mut node = Command::new("node");
    node.current_dir(&root)
        .arg("driver.mjs")
        .arg(&glue)
        .arg(&wasm)
        .arg(INPUT);
    let driven = run(node, "node");
    assert!(
        driven.status.success(),
        "the six JSON kinds do not round-trip through the page glue and the module"
    );
    assert_eq!(
        String::from_utf8_lossy(&driven.stdout),
        "json round trip: six kinds through the page glue; spelling kept by the module\n"
    );
}

#[test]
fn issue_224_a_model_without_json_keeps_the_web_bytes() {
    let synthesis = emit(&without_json(), Target::Web);
    let json = artifact(&synthesis, "crates/demo-web/src/json.rs");
    assert!(json.contains("pub enum Value {"), "{json}");
    assert!(!json.contains("pub use"), "{json}");
    let script = page_script(artifact(&synthesis, "index.html"));
    assert!(!script.contains("JsonValue"), "{script}");
    assert!(!script.contains("jsonControl"), "{script}");
    let target = artifact(&synthesis, "TARGET.md");
    assert!(!target.contains("a `Json` value"), "{target}");
}
