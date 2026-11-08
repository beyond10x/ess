//! Host slots in the generated React project (design `ui-host-slots`, section Generators): the
//! project exports `HostComponents`, a registry type with exactly one entry per declared `host`
//! name, and the app entry requires a value of it. Each entry is a component that receives
//! `{ adapter, path, page, scope }`: a data adapter narrowed to the slot's declared reads and
//! commands, the slot's document path, the page's route parameters and the scope values in force
//! where the slot stands.
//!
//! The type cases write the host's `src/host-components.tsx` into a generated project and run
//! `tsc -p tsconfig.offline.json` over it: a slot left unregistered, a name no slot declares and
//! an access the slot did not declare each fail the type check. The render case compiles the app
//! and runs it under `node` against the stand-ins in `support`, with a probe registered for each
//! slot.

mod support;

use std::path::{Path, PathBuf};
use std::process::Command;

use support::{compile, node, write};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

fn scratch(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("ess-ui-react-host-slots")
        .join(name);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("the old scratch project is removed");
    }
    dir
}

/// A slot at each of the five scopes: the page `editor`, the section `map`, the overlay `sign`,
/// the tab form `history` and the field `notes`.
const FIVE_SCOPES: &str = "format: ess-ui/1
app: t
model: t.system
placement_profile: fat
shells: {app: {regions: {main: {kind: page_outlet}}}}
navigation: {home: p, sections: [{name: all, pages: [p, editor]}]}
pages:
  p:
    kind: detail_page
    title: P
    sections:
      - name: summary
        component: record
        reads: t.ById
        fields:
          - name
          - {field: notes, host: NotesEditor, reason: needs spell checking, stands_in_for: rich_text}
        tabs:
          - name: history
            form: {host: AuditTrail, reason: draws a timeline, stands_in_for: collection, reads: t.History}
      - {name: map, host: RouteMap, reason: a map widget, stands_in_for: chart, reads: t.Routes}
    overlays:
      sign: {kind: dialog, host: SignaturePad, reason: captures a pen signature, stands_in_for: form, commands: [t.Sign]}
  editor:
    kind: detail_page
    title: Editor
    host: FlowCanvas
    reason: a vendor canvas
    stands_in_for: graph_editor
    reads: t.Flow
    commands: [t.SaveFlow]
";

fn generate(name: &str, text: &str) -> (PathBuf, ess_ui_react::GeneratedFiles) {
    let document = ess_ui::load_str(text).unwrap_or_else(|error| panic!("{error}"));
    let out = scratch(name);
    let files = ess_ui_react::generate(&document, &root(), &out)
        .unwrap_or_else(|error| panic!("a document with host slots generates: {error}"));
    (out, files)
}

/// Each slot's component, written as a host writes it: typed by its registry entry, using
/// exactly the access its slot declares.
const COMPONENTS: &str = r#"import type { ReactNode } from "react";
import type { HostComponents } from "./hosts";

const FlowCanvas: HostComponents["FlowCanvas"] = ({ adapter, path, page, scope }) => {
  void adapter.read({ view: "t.Flow", params: { id: page.id, zoom: scope.zoom } });
  void adapter.send("t.SaveFlow", { path });
  return null;
};
const NotesEditor: HostComponents["NotesEditor"] = ({ path }): ReactNode => path;
const AuditTrail: HostComponents["AuditTrail"] = ({ adapter }) => {
  void adapter.read({ view: "t.History", params: {} });
  return null;
};
const RouteMap: HostComponents["RouteMap"] = ({ adapter }) => {
  void adapter.read({ view: "t.Routes", params: {} });
  return null;
};
const SignaturePad: HostComponents["SignaturePad"] = ({ adapter }) => {
  void adapter.send("t.Sign", {});
  return null;
};
"#;

/// Writes `src/host-components.tsx` from `COMPONENTS` with `edit` applied, exporting `hosts`
/// (unannotated, as `main.tsx` imports it), and type-checks the project offline.
fn type_check(project: &Path, edit: impl Fn(&str) -> String, hosts: &str) -> (bool, String) {
    write(
        &project.join("src/host-components.tsx"),
        &format!("{}\nexport const hosts = {{ {hosts} }};\n", edit(COMPONENTS)),
    );
    let output = Command::new("tsc")
        .args(["-p", "tsconfig.offline.json", "--noEmit"])
        .current_dir(project)
        .output()
        .expect("tsc is on PATH");
    (
        output.status.success(),
        format!(
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ),
    )
}

const ALL: &str = "FlowCanvas, NotesEditor, AuditTrail, RouteMap, SignaturePad";

#[test]
fn a_document_with_host_slots_generates_one_registry_entry_per_host_name() {
    let (_, files) = generate("registry", FIVE_SCOPES);
    let hosts = files
        .files
        .get("src/hosts.ts")
        .unwrap_or_else(|| panic!("no src/hosts.ts among {:?}", files.files.keys()));
    let entries: Vec<&str> = hosts
        .lines()
        .map(str::trim)
        .filter(|line| line.contains(": HostComponent<"))
        .collect();
    assert_eq!(
        entries,
        vec![
            r#"readonly FlowCanvas: HostComponent<"t.Flow", "t.SaveFlow">;"#,
            "readonly NotesEditor: HostComponent<never, never>;",
            r#"readonly AuditTrail: HostComponent<"t.History", never>;"#,
            r#"readonly RouteMap: HostComponent<"t.Routes", never>;"#,
            r#"readonly SignaturePad: HostComponent<never, "t.Sign">;"#,
        ],
        "{hosts}"
    );
    let app = &files.files["src/App.tsx"];
    assert!(
        app.contains("export function App<H extends HostComponents>(props: { hosts: Registered<H> }): ReactNode {"),
        "the app requires a registry:\n{app}"
    );
    let main = &files.files["src/main.tsx"];
    assert!(
        main.contains("import { hosts } from \"./host-components\";")
            && main.contains("<App hosts={hosts} />"),
        "the entry passes the host's registry:\n{main}"
    );
    assert!(
        !files.files.contains_key("src/host-components.tsx"),
        "the host's registry is the host's file, never generated"
    );
}

#[test]
fn a_document_without_host_slots_keeps_its_entry_and_app() {
    let text = FIVE_SCOPES
        .split("      - {name: map")
        .next()
        .map(|head| {
            head.replace(
                "          - {field: notes, host: NotesEditor, reason: needs spell checking, stands_in_for: rich_text}\n",
                "",
            )
            .replace(
                "        tabs:\n          - name: history\n            form: {host: AuditTrail, reason: draws a timeline, stands_in_for: collection, reads: t.History}\n",
                "",
            )
            .replace("[p, editor]", "[p]")
        })
        .expect("the head of the document");
    let (_, files) = generate("plain", &text);
    assert!(!files.files.contains_key("src/hosts.ts"));
    assert!(!files.files.contains_key("src/runtime/hosts.tsx"));
    assert!(files.files["src/main.tsx"].contains("      <App />\n"));
    assert!(files.files["src/App.tsx"].contains("export function App(): ReactNode {"));
}

#[test]
fn the_project_type_checks_with_every_slot_registered() {
    let (project, _) = generate("complete", FIVE_SCOPES);
    let (ok, output) = type_check(&project, str::to_owned, ALL);
    assert!(ok, "every slot registered with its declared access:\n{output}");
}

#[test]
fn a_slot_left_unregistered_fails_the_type_check() {
    let (project, _) = generate("unregistered", FIVE_SCOPES);
    let (ok, output) = type_check(
        &project,
        str::to_owned,
        "FlowCanvas, NotesEditor, AuditTrail, RouteMap",
    );
    assert!(
        !ok && output.contains("src/main.tsx") && output.contains("SignaturePad"),
        "a missing registration is a type error naming it:\n{output}"
    );
}

#[test]
fn a_name_no_slot_declares_fails_the_type_check() {
    let (project, _) = generate("extra", FIVE_SCOPES);
    let (ok, output) = type_check(
        &project,
        |text| format!("{text}const WeatherBox = (): null => null;\n"),
        &format!("{ALL}, WeatherBox"),
    );
    assert!(
        !ok && output.contains("src/main.tsx") && output.contains("WeatherBox"),
        "an extra registration is a type error naming it:\n{output}"
    );
}

#[test]
fn a_slot_reading_a_view_it_did_not_declare_fails_the_type_check() {
    let (project, _) = generate("undeclared-read", FIVE_SCOPES);
    let (ok, output) = type_check(
        &project,
        |text| text.replace(r#"view: "t.Routes""#, r#"view: "t.Flow""#),
        ALL,
    );
    assert!(
        !ok && output.contains("src/host-components.tsx") && output.contains("\"t.Flow\""),
        "a read of an undeclared view is a type error:\n{output}"
    );
}

#[test]
fn a_slot_sending_a_command_it_did_not_declare_fails_the_type_check() {
    let (project, _) = generate("undeclared-command", FIVE_SCOPES);
    let (ok, output) = type_check(
        &project,
        |text| text.replace(r#"adapter.send("t.SaveFlow""#, r#"adapter.send("t.Sign""#),
        ALL,
    );
    assert!(
        !ok && output.contains("src/host-components.tsx") && output.contains("\"t.Sign\""),
        "a command the slot did not declare is a type error:\n{output}"
    );
    // A slot that declares no command has no `send` at all.
    let (ok, output) = type_check(
        &project,
        |text| {
            text.replace(
                r#"void adapter.read({ view: "t.Routes", params: {} });"#,
                r#"void adapter.send("t.Sign", {});"#,
            )
        },
        ALL,
    );
    assert!(
        !ok && output.contains("src/host-components.tsx") && output.contains("'send'"),
        "`adapter.send` outside the narrowed type is a type error:\n{output}"
    );
}

#[test]
fn a_component_taking_the_whole_adapter_fails_the_type_check() {
    let (project, _) = generate("wide-adapter", FIVE_SCOPES);
    let (ok, output) = type_check(
        &project,
        |text| {
            text.replace(
                r#"const RouteMap: HostComponents["RouteMap"] = ({ adapter }) => {"#,
                "const RouteMap = ({ adapter }: { adapter: DataAdapter; path: string; \
                 page: Record<string, string | undefined>; scope: Record<string, unknown> }) => {\n  \
                 void adapter.loadState(\"server\", \"k\");",
            )
            .replace(
                "import type { HostComponents } from \"./hosts\";",
                "import type { HostComponents } from \"./hosts\";\nimport type { DataAdapter } from \"./runtime/data\";",
            )
        },
        ALL,
    );
    assert!(
        !ok && output.contains("src/main.tsx") && output.contains("RouteMap"),
        "a component that needs more than its slot declares cannot be registered for it:\n{output}"
    );
}

/// A page with route params and page state: the section `map` is a slot between two generated
/// sections, and `pin` a slot between two nodes of `notes`, whose record adds `row` to the scope.
const RENDERED: &str = "format: ess-ui/1
app: t
model: t.system
placement_profile: fat
shells: {app: {regions: {main: {kind: page_outlet}}}}
navigation: {home: things.list, sections: [{name: all, pages: [things.list]}]}
pages:
  things.list:
    kind: detail_page
    title: Things
    sections:
      - {name: all, component: record, fields: [title]}
  things:
    kind: detail_page
    title: Thing
    params: {id: string}
    state:
      zoom: {type: integer, class: page_state, store: url, default: 3}
    sections:
      - {name: summary, component: record, fields: [title]}
      - {name: map, host: RouteMap, reason: a map widget, stands_in_for: chart, reads: things.Routes, commands: [things.Move]}
      - name: notes
        component: record
        fields: [title]
        children:
          - {name: hint, primitive: text, text: row.title}
          - {name: pin, host: PinBoard, reason: a pin board, stands_in_for: none}
          - {name: tail, primitive: text, text: row.title}
";

#[test]
fn a_registered_slot_renders_at_its_path_with_its_narrowed_adapter_page_and_scope() {
    let (project, _) = generate("render", RENDERED);
    compile(&project, &["App.tsx"]);
    node(
        &project,
        "render",
        r#"
(async () => {
browser("/things/7");
const data = out("runtime/data");
const calls = [];
data.setDataAdapter({
  read(request) { calls.push(["read", request.view, request.params]); return Promise.resolve({ rows: [] }); },
  send(command, input) { calls.push(["send", command, input]); return Promise.resolve({ accepted: true }); },
  loadState() { return Promise.resolve(undefined); },
  saveState() { return Promise.resolve(); },
});
const seen = {};
const probe = (name) => (props) => {
  seen[name] = props;
  return jsx("i", { "data-probe": name });
};
const { App } = out("App");
const root = React.__root(jsx(App, { hosts: { RouteMap: probe("RouteMap"), PinBoard: probe("PinBoard") } })).settle();

// Each slot renders inside one element carrying its path and its host, at its place among its
// siblings, with the registered component inside it.
for (const [host, path] of [["RouteMap", "pages/things/sections/map"], ["PinBoard", "pages/things/sections/notes/children/pin"]]) {
  const regions = root.find((n) => n.props && n.props["data-ui-host"] === host);
  assert.strictEqual(regions.length, 1, `${host} renders once`);
  assert.strictEqual(regions[0].props["data-ui-path"], path, `${host} carries its path`);
  assert.strictEqual(root.find((n) => n.props && n.props["data-ui-path"] === path).length, 1, `${path} renders once`);
  const inside = [];
  const visit = (nodes) => nodes.forEach((n) => { if (typeof n === "object") { if (n.props && n.props["data-probe"]) inside.push(n.props["data-probe"]); if (n.children) visit(n.children); } });
  visit(regions[0].children);
  assert.deepStrictEqual(inside, [host], `${host}'s component renders inside its region`);
}
const order = root
  .find((n) => n.props && typeof n.props["data-ui-path"] === "string")
  .map((n) => n.props["data-ui-path"])
  .filter((path) => /^pages\/things\/sections\/[a-z]+(\/children\/[a-z]+)?$/.test(path));
assert.deepStrictEqual(order, [
  "pages/things/sections/summary",
  "pages/things/sections/map",
  "pages/things/sections/notes",
  "pages/things/sections/notes/children/hint",
  "pages/things/sections/notes/children/pin",
  "pages/things/sections/notes/children/tail",
], "each slot stands at its position among its siblings");

// The four props, and nothing else.
for (const host of ["RouteMap", "PinBoard"]) {
  assert.deepStrictEqual(Object.keys(seen[host]).sort(), ["adapter", "page", "path", "scope"], host);
}
assert.strictEqual(seen.RouteMap.path, "pages/things/sections/map");
assert.strictEqual(seen.PinBoard.path, "pages/things/sections/notes/children/pin");
for (const host of ["RouteMap", "PinBoard"]) {
  assert.deepStrictEqual({ ...seen[host].page }, { id: "7" }, `${host} gets the page's route params`);
  const scope = seen[host].scope;
  assert.strictEqual(scope.params.id, "7", `${host}'s scope holds the page params`);
  assert.strictEqual(scope.state.zoom, 3, `${host}'s scope holds the page state`);
  assert.ok("actor" in scope && "section" in scope, `${host}'s scope holds the app's and the page's layers`);
  assert.ok(!("states" in scope), `${host} gets no section states`);
}
assert.ok("row" in seen.PinBoard.scope, "a slot inside a record gets the record's layer");
assert.ok(!("row" in seen.RouteMap.scope), "a section slot gets no row");

// The adapter is narrowed to what the slot declares.
assert.deepStrictEqual(Object.keys(seen.RouteMap.adapter).sort(), ["read", "send"]);
assert.deepStrictEqual(Object.keys(seen.PinBoard.adapter), [], "a slot that declares nothing gets no access");
const routes = await seen.RouteMap.adapter.read({ view: "things.Routes", params: { near: 1 } });
assert.deepStrictEqual(routes, { rows: [] });
await seen.RouteMap.adapter.send("things.Move", { to: 2 });
await assert.rejects(seen.RouteMap.adapter.read({ view: "things.Other", params: {} }), (error) => error.status === 403);
await assert.rejects(seen.RouteMap.adapter.send("things.Delete", {}), (error) => error.status === 403);
assert.deepStrictEqual(calls, [
  ["read", "things.Routes", { near: 1 }],
  ["send", "things.Move", { to: 2 }],
], "only the declared read and command reach the data adapter");
})().catch((error) => { console.error(error); process.exit(1); });
"#,
    );
}
