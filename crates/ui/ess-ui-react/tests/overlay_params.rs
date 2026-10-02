//! Overlay params are "values passed by the opener" (`Overlay.params`): a row action that opens a
//! page overlay evaluates the overlay's `params` against the row it was run on, as the terminal
//! renderer does. The cases generate a small document, compile its page and the runtime to
//! `CommonJS` with `tsc` and run them under `node` against a hook-level stand-in for `react`
//! (written into the scratch project, never into the generated output): the row action is
//! clicked, the overlay opens, and the command it sends is read off the data adapter.

use std::path::{Path, PathBuf};
use std::process::Command;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

fn scratch(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("ess-ui-react-overlay-params")
        .join(name);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("the old scratch project is removed");
    }
    dir
}

fn generate(name: &str, text: &str) -> PathBuf {
    let document =
        ess_ui::load_str(text).unwrap_or_else(|error| panic!("the document loads: {error}"));
    let out = scratch(name);
    ess_ui_react::generate(&document, &root(), &out).unwrap_or_else(|error| panic!("{error}"));
    out
}

const REACT_STUB: &str = r#""use strict";
let current = null;
const contexts = [];
function slot(init) {
  if (!current) throw new Error("hook outside a component");
  const i = current.index++;
  if (current.slots.length <= i) current.slots.push(init());
  return current.slots[i];
}
exports.useState = function (initial) {
  const s = slot(() => ({ value: typeof initial === "function" ? initial() : initial }));
  if (!s.set) s.set = (next) => { s.value = typeof next === "function" ? next(s.value) : next; };
  return [s.value, s.set];
};
exports.useRef = (initial) => slot(() => ({ current: initial }));
exports.useMemo = (factory) => slot(() => ({ value: factory() })).value;
exports.useCallback = (callback) => callback;
exports.useEffect = () => undefined;
exports.useLayoutEffect = exports.useEffect;
exports.useContext = (ctx) => (current && current.ctx.has(ctx) ? current.ctx.get(ctx) : ctx._default);
exports.createContext = (value) => {
  const ctx = { _default: value };
  const Provider = (props) => props.children;
  Provider._ctx = ctx;
  ctx.Provider = Provider;
  contexts.push(ctx);
  return ctx;
};
exports.Fragment = (props) => props.children;
exports.StrictMode = (props) => props.children;
exports.__mount = (component, ctx) => {
  const inst = { slots: [], index: 0, ctx: ctx || new Map() };
  return {
    render(props) {
      const previous = current;
      current = inst;
      inst.index = 0;
      try { return component(props || {}); } finally { current = previous; }
    },
  };
};
const mounted = new Map();
function walk(node, ctx, key) {
  if (node === null || node === undefined || typeof node === "boolean") return [];
  if (typeof node === "string" || typeof node === "number") return [String(node)];
  if (Array.isArray(node)) return node.flatMap((n, i) => walk(n, ctx, `${key}.${i}`));
  if (typeof node.type === "function") {
    if (node.type._ctx) {
      const next = new Map(ctx);
      next.set(node.type._ctx, node.props.value);
      return walk(node.props.children, next, `${key}>`);
    }
    // A component keeps its hook slots across walks while it stays at the same place.
    const at = `${key}/${node.type.name}`;
    let inst = mounted.get(at);
    if (!inst || inst.type !== node.type) {
      inst = { type: node.type, api: exports.__mount(node.type, ctx) };
      mounted.set(at, inst);
    }
    return walk(inst.api.render(node.props), ctx, at);
  }
  return [{ tag: node.type, props: node.props, children: walk(node.props.children, ctx, `${key}<${node.type}`) }];
}
exports.__renderTree = (node) => walk(node, new Map(), "");
"#;

const JSX_STUB: &str = r#""use strict";
const React = require("./index.js");
function jsx(type, props, key) { return { type, props: props || {}, key: key === undefined ? null : key }; }
module.exports = { jsx, jsxs: jsx, Fragment: React.Fragment };
"#;

const ROUTER_STUB: &str = r#""use strict";
exports.useSearchParams = () => [new URLSearchParams(), () => undefined];
exports.useParams = () => ({});
exports.useNavigate = () => () => undefined;
exports.useLocation = () => ({ pathname: "/", search: "", hash: "" });
for (const name of ["BrowserRouter", "Routes", "Route", "Navigate", "Link", "Outlet"]) exports[name] = () => null;
"#;

fn write(path: &Path, text: &str) {
    std::fs::create_dir_all(path.parent().expect("a parent")).expect("the directory is created");
    std::fs::write(path, text).expect("the file is written");
}

/// Compiles `modules` (paths under `src/`) and what they import to `CommonJS` under `cjs-out/`.
fn build(project: &Path, modules: &[&str]) {
    write(
        &project.join("node_modules/react/package.json"),
        r#"{"name":"react","main":"index.js","type":"commonjs"}"#,
    );
    write(&project.join("node_modules/react/index.js"), REACT_STUB);
    write(&project.join("node_modules/react/jsx-runtime.js"), JSX_STUB);
    write(
        &project.join("node_modules/react-router/package.json"),
        r#"{"name":"react-router","main":"index.js","type":"commonjs"}"#,
    );
    write(
        &project.join("node_modules/react-router/index.js"),
        ROUTER_STUB,
    );
    let files: Vec<String> = modules.iter().map(|m| format!("\"src/{m}\"")).collect();
    write(
        &project.join("tsconfig.cjs.json"),
        &format!(
            r#"{{
  "extends": "./tsconfig.offline.json",
  "compilerOptions": {{
    "noEmit": false, "outDir": "cjs-out", "rootDir": ".", "module": "commonjs",
    "moduleResolution": "node10", "ignoreDeprecations": "6.0", "noEmitOnError": false
  }},
  "include": [],
  "files": [{}]
}}"#,
            files.join(", ")
        ),
    );
    let output = Command::new("tsc")
        .args(["-p", "tsconfig.cjs.json"])
        .current_dir(project)
        .output()
        .expect("tsc is on PATH");
    write(
        &project.join("cjs-out/package.json"),
        r#"{"type":"commonjs"}"#,
    );
    for module in modules {
        let js = Path::new(module).with_extension("js");
        assert!(
            project.join("cjs-out/src").join(&js).exists(),
            "tsc emitted no {}: {}{}",
            js.display(),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

/// Runs `script` under node in `project`; panics with its output on failure.
fn node(project: &Path, name: &str, script: &str) {
    let path = project.join(format!("{name}.cjs"));
    write(&path, script);
    let output = Command::new("node")
        .arg(&path)
        .current_dir(project)
        .output()
        .expect("node is on PATH");
    assert!(
        output.status.success(),
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

const OBJECTIVES: &str = r"
format: ess-ui/1
app: objectives
model: objectives.system
placement_profile: fat
shells:
  app: {regions: {main: {kind: page_outlet}}}
navigation:
  home: objectives.list
  sections: [{name: all, pages: [objectives.list]}]
pages:
  objectives.list:
    kind: detail_page
    title: Objectives
    sections:
      - name: list
        component: collection
        reads: {view: objectives.Open}
        columns: [goal]
        row_actions:
          - {name: abandon, opens: abandon, label: Abandon}
          - {name: resolve, opens: resolve, label: Resolve}
    overlays:
      abandon:
        kind: dialog
        component: confirm
        title: Abandon objective
        does: objectives.AbandonObjective
        params: {objective_id: row.objective_id}
      resolve:
        kind: drawer
        component: form
        title: Resolve ambiguity
        does: objectives.ResolveAmbiguity
        params: {ambiguity_id: row.ambiguity_id}
        fields: [resolution]
";

/// Renders the generated page once to collect the overlay table and the row actions it hands to
/// the runtime, then mounts the runtime's `OverlayHost` with that table around those row actions,
/// run on one row. `open(name)` clicks a row action and re-renders; `send()` submits what opened.
const HARNESS: &str = r#""use strict";
const assert = require("assert");
const React = require("react");
const jsxRuntime = require("react/jsx-runtime");
const rt = (p) => require("./cjs-out/src/runtime/" + p);
const { OverlayHost } = rt("overlays");
const { ScopeLayer } = rt("core");
const { ActionControl } = rt("actions");
const { setDataAdapter } = rt("data");
const { Collection } = rt("composites/collection");
const sent = [];
setDataAdapter({
  read: async () => ({ rows: [] }),
  send: async (command, input) => { sent.push({ command, input: JSON.parse(JSON.stringify(input)) }); return {}; },
  loadState: async () => undefined,
  saveState: async () => undefined,
});
const created = [];
const jsx = jsxRuntime.jsx;
jsxRuntime.jsx = jsxRuntime.jsxs = (type, props, key) => { created.push({ type, props }); return jsx(type, props, key); };
const page = require("./cjs-out/src/pages/ObjectivesList");
const Page = Object.values(page).find((value) => typeof value === "function");
React.__renderTree(jsx(Page, {}));
const host = created.find((element) => element.type === OverlayHost);
assert(host, "the page renders an OverlayHost");
const collection = created.find((element) => element.type === Collection);
assert(collection, "the page renders its collection");
const row = { id: "row-1", objective_id: "obj-1", ambiguity_id: "amb-1", goal: "Ship the portal" };
const tree = jsx(OverlayHost, {
  overlays: host.props.overlays,
  children: jsx(ScopeLayer, {
    values: { row },
    children: collection.props.rowActions.map((action) => jsx(ActionControl, { action }, action.name)),
  }),
});
const find = (nodes, test) => {
  for (const node of nodes) {
    if (typeof node !== "object") continue;
    if (test(node)) return node;
    const inner = find(node.children, test);
    if (inner) return inner;
  }
  return undefined;
};
const text = (node) => node.children.map((child) => (typeof child === "object" ? text(child) : child)).join("");
const open = (label) => {
  const button = find(React.__renderTree(tree), (n) => n.tag === "button" && text(n) === label);
  assert(button, `the row action ${label} renders`);
  button.props.onClick();
  return React.__renderTree(tree);
};
const settle = () => new Promise((resolve) => setTimeout(resolve, 0));
"#;

/// A confirm overlay opened from a row sends the row's id under the overlay's param name.
#[test]
fn a_confirm_overlay_opened_from_a_row_sends_the_rows_value_as_its_param() {
    let project = generate("confirm", OBJECTIVES);
    build(&project, &["pages/ObjectivesList.tsx"]);
    node(
        &project,
        "confirm",
        &format!(
            r#"{HARNESS}
(async () => {{
  const opened = open("Abandon");
  const confirm = find(opened, (n) => n.tag === "button" && text(n) === "Confirm");
  assert(confirm, "the abandon overlay opens with its confirm button");
  confirm.props.onClick();
  await settle();
  assert.strictEqual(sent.length, 1, `one command is sent: ${{JSON.stringify(sent)}}`);
  assert.strictEqual(sent[0].command, "objectives.AbandonObjective");
  assert.strictEqual(sent[0].input.objective_id, "obj-1", `the confirm sends ${{JSON.stringify(sent[0].input)}}`);
}})().catch((error) => {{ console.error(error); process.exit(1); }});
"#
        ),
    );
}

/// A form overlay opened from a row submits the row's id under the overlay's param name.
#[test]
fn a_form_overlay_opened_from_a_row_sends_the_rows_value_as_its_param() {
    let project = generate("form", OBJECTIVES);
    build(&project, &["pages/ObjectivesList.tsx"]);
    node(
        &project,
        "form",
        &format!(
            r#"{HARNESS}
(async () => {{
  const opened = open("Resolve");
  const form = find(opened, (n) => n.tag === "form");
  assert(form, "the resolve overlay opens with its form");
  form.props.onSubmit({{ preventDefault() {{}} }});
  await settle();
  assert.strictEqual(sent.length, 1, `one command is sent: ${{JSON.stringify(sent)}}`);
  assert.strictEqual(sent[0].command, "objectives.ResolveAmbiguity");
  assert.strictEqual(sent[0].input.ambiguity_id, "amb-1", `the form sends ${{JSON.stringify(sent[0].input)}}`);
}})().catch((error) => {{ console.error(error); process.exit(1); }});
"#
        ),
    );
}
