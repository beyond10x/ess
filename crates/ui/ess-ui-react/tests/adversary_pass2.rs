//! Adversary pass 2 on the React generator: what correction 1 introduced. Actor-scoped storage
//! keys and their reload on an actor change, the `stale_after` timer, resume after a reconnect,
//! per-session channel keying, and runtime `data-ui-path` addressing of collection rows.
//!
//! Runtime cases compile the generated runtime to `CommonJS` with `tsc` and run it under `node`
//! against a small hook-level stand-in for `react` (written into the scratch project, never into
//! the generated output): state slots, effects flushed on demand, contexts, and a static tree walk.

use std::path::{Path, PathBuf};
use std::process::Command;

use ess_ui::Document;
use ess_ui_react::GeneratedFiles;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

fn example_file() -> PathBuf {
    root().join("examples/partner-portal/ui.yaml")
}

fn scratch(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("ess-ui-react-adv2")
        .join(name);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("the old scratch project is removed");
    }
    dir
}

fn load(text: &str) -> Document {
    ess_ui::load_str(text).unwrap_or_else(|error| panic!("the document loads: {error}"))
}

fn generate(name: &str, text: &str) -> (PathBuf, Result<GeneratedFiles, String>) {
    let document = load(text);
    let out = scratch(name);
    let result = ess_ui_react::generate(&document, &root(), &out).map_err(|e| e.to_string());
    (out, result)
}

fn example(name: &str) -> PathBuf {
    let out = scratch(name);
    ess_ui_react::generate_path(&example_file(), &out).unwrap_or_else(|error| panic!("{error}"));
    out
}

fn file<'a>(files: &'a GeneratedFiles, path: &str) -> &'a str {
    files
        .files
        .get(path)
        .unwrap_or_else(|| panic!("`{path}` is generated; have {:?}", files.files.keys()))
}

// ── a hook-level React stand-in and a CommonJS build of the runtime ──────────────────────────

const REACT_STUB: &str = r#""use strict";
let current = null;
const contexts = [];
function slot(init) {
  if (!current) throw new Error("hook outside a component");
  const i = current.index++;
  if (current.slots.length <= i) current.slots.push(init());
  return current.slots[i];
}
function changed(a, b) {
  return !a || !b || a.length !== b.length || a.some((v, i) => !Object.is(v, b[i]));
}
exports.useState = function (initial) {
  const s = slot(() => ({ value: typeof initial === "function" ? initial() : initial }));
  if (!s.set) s.set = (next) => { s.value = typeof next === "function" ? next(s.value) : next; };
  return [s.value, s.set];
};
exports.useRef = (initial) => slot(() => ({ current: initial }));
exports.useMemo = (factory, deps) => {
  const s = slot(() => ({}));
  if (!s.has || changed(s.deps, deps)) { s.value = factory(); s.deps = deps; s.has = true; }
  return s.value;
};
exports.useCallback = (callback, deps) => exports.useMemo(() => callback, deps);
exports.useEffect = (effect, deps) => {
  const s = slot(() => ({ ran: false }));
  if (!s.ran || deps === undefined || changed(s.deps, deps)) {
    s.ran = true;
    s.deps = deps;
    current.pending.push(() => {
      if (s.cleanup) s.cleanup();
      const c = effect();
      s.cleanup = typeof c === "function" ? c : undefined;
    });
  }
};
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
exports.__contexts = contexts;
exports.__mount = (component, ctx) => {
  const inst = { slots: [], index: 0, pending: [], ctx: ctx || new Map() };
  const api = {
    render(props) {
      const previous = current;
      current = inst;
      inst.index = 0;
      try { return (api.result = component(props || {})); } finally { current = previous; }
    },
    flush() {
      while (inst.pending.length) inst.pending.splice(0).forEach((f) => f());
    },
    unmount() {
      for (const s of inst.slots) if (s && s.cleanup) s.cleanup();
    },
  };
  return api;
};
function walk(node, ctx) {
  if (node === null || node === undefined || typeof node === "boolean") return [];
  if (typeof node === "string" || typeof node === "number") return [String(node)];
  if (Array.isArray(node)) return node.flatMap((n) => walk(n, ctx));
  if (typeof node.type === "function") {
    if (node.type._ctx) {
      const next = new Map(ctx);
      next.set(node.type._ctx, node.props.value);
      return walk(node.props.children, next);
    }
    return walk(exports.__mount(node.type, ctx).render(node.props), ctx);
  }
  if (node.type === undefined && typeof node[Symbol.iterator] === "function") return walk([...node], ctx);
  return [{ tag: node.type, props: node.props, children: walk(node.props.children, ctx) }];
}
exports.__renderTree = (node) => walk(node, new Map());
"#;

const JSX_STUB: &str = r#""use strict";
const React = require("./index.js");
function jsx(type, props, key) { return { type, props: props || {}, key: key === undefined ? null : key }; }
module.exports = { jsx, jsxs: jsx, Fragment: React.Fragment };
"#;

/// Shared by every runtime script: fake timers and clock, Web Storage areas, the scope context.
const PRELUDE: &str = r#""use strict";
const assert = require("assert");
const React = require("react");
const { jsx } = require("react/jsx-runtime");
const rt = (p) => require("./adv-out/src/runtime/" + p);
function area() {
  const m = new Map();
  return { getItem: (k) => (m.has(k) ? m.get(k) : null), setItem: (k, v) => void m.set(k, String(v)), removeItem: (k) => void m.delete(k) };
}
let now = 1000000;
const queue = [];
let seq = 0;
Date.now = () => now;
globalThis.setTimeout = (fn, ms, ...args) => { const t = { at: now + (ms || 0), fn: () => fn(...args), id: ++seq }; queue.push(t); return t; };
globalThis.clearTimeout = (t) => { const i = queue.indexOf(t); if (i >= 0) queue.splice(i, 1); };
function step() {
  if (!queue.length) return false;
  queue.sort((a, b) => a.at - b.at || a.id - b.id);
  const t = queue.shift();
  now = Math.max(now, t.at);
  t.fn();
  return true;
}
function scopeContext() {
  rt("core");
  const c = React.__contexts.find((x) => x._default && typeof x._default.set === "function" && "values" in x._default);
  assert(c, "the scope context exists");
  return c;
}
"#;

fn write(path: &Path, text: &str) {
    std::fs::create_dir_all(path.parent().expect("a parent")).expect("the directory is created");
    std::fs::write(path, text).expect("the file is written");
}

/// Compiles `modules` (paths under `src/`) and what they import to `CommonJS` under `adv-out/`.
fn build_runtime(project: &Path, modules: &[&str]) {
    write(
        &project.join("node_modules/react/package.json"),
        r#"{"name":"react","main":"index.js","type":"commonjs"}"#,
    );
    write(&project.join("node_modules/react/index.js"), REACT_STUB);
    write(&project.join("node_modules/react/jsx-runtime.js"), JSX_STUB);
    let files: Vec<String> = modules.iter().map(|m| format!("\"src/{m}\"")).collect();
    write(
        &project.join("tsconfig.adv.json"),
        &format!(
            r#"{{
  "extends": "./tsconfig.offline.json",
  "compilerOptions": {{
    "noEmit": false, "outDir": "adv-out", "rootDir": ".", "module": "commonjs",
    "moduleResolution": "node10", "ignoreDeprecations": "6.0", "noEmitOnError": false
  }},
  "include": [],
  "files": [{}]
}}"#,
            files.join(", ")
        ),
    );
    let output = Command::new("tsc")
        .args(["-p", "tsconfig.adv.json"])
        .current_dir(project)
        .output()
        .expect("tsc is on PATH");
    write(
        &project.join("adv-out/package.json"),
        r#"{"type":"commonjs"}"#,
    );
    for module in modules {
        let js = Path::new(module).with_extension("js");
        assert!(
            project.join("adv-out/src").join(&js).exists(),
            "tsc emitted no {}: {}{}",
            js.display(),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

/// Runs `script` (after the prelude) under node in `project`; panics with its output on failure.
fn node(project: &Path, name: &str, script: &str) {
    let path = project.join(format!("{name}.cjs"));
    write(&path, &format!("{PRELUDE}\n{script}"));
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

// ── actor-scoped browser storage (Store.meaning.*_storage.constraints.key) ─────────────────

/// The key is `[origin, actor.user_id, actor.account_id]`; joined with `|`, two actors whose ids
/// contain `|` share one key and read each other's drafts and preferences.
#[test]
fn two_actors_never_share_a_storage_key() {
    let project = example("storage-key");
    build_runtime(&project, &["runtime/state/storage.ts"]);
    node(
        &project,
        "storage-key",
        r#"
const { storageKey } = rt("state/storage");
const a = storageKey("portal:pages/tickets.detail/sections/reply/draft", { user_id: "u|1", account_id: "acme" });
const b = storageKey("portal:pages/tickets.detail/sections/reply/draft", { user_id: "u", account_id: "1|acme" });
assert.notStrictEqual(a, b, `actor {u|1, acme} and actor {u, 1|acme} share the storage key ${a}`);
"#,
    );
}

/// On an actor change the hook must not hand the new actor the previous actor's value: the
/// render with the new actor happens before any effect runs, and React paints it.
#[test]
fn an_actor_change_never_renders_the_previous_actors_stored_value() {
    let project = example("actor-flash");
    build_runtime(
        &project,
        &[
            "runtime/core.tsx",
            "runtime/state/storage.ts",
            "runtime/state/session_storage.ts",
        ],
    );
    node(
        &project,
        "actor-flash",
        r#"
globalThis.sessionStorage = area();
const ctx = scopeContext();
const { useSessionStorageState } = rt("state/session_storage");
const { storageKey } = rt("state/storage");
const scopeFor = (actor) => ({ ...ctx._default, values: { actor } });
const A = { signed_in: true, user_id: "ua", account_id: "acc" };
const B = { signed_in: true, user_id: "ub", account_id: "acc" };
const key = "portal:pages/tickets.detail/sections/reply/draft";
sessionStorage.setItem(storageKey(key, A), JSON.stringify("A's private reply"));
const contexts = new Map([[ctx, scopeFor(A)]]);
const hook = React.__mount(() => useSessionStorageState(key, ""), contexts);
assert.strictEqual(hook.render()[0], "A's private reply", "setup: actor A reads its draft");
hook.flush();
contexts.set(ctx, scopeFor(B));
const [seen] = hook.render();
assert.strictEqual(seen, "", `the first render for actor B returns actor A's value ${JSON.stringify(seen)}`);
"#,
    );
}

/// `store: server` with a `local_storage` fallback keys by actor too; after an actor change to
/// one with nothing stored, the hook must not keep showing the previous actor's value.
#[test]
fn a_server_state_on_its_local_fallback_does_not_keep_the_previous_actors_value() {
    let project = example("fallback-actor");
    build_runtime(&project, &["runtime/core.tsx", "runtime/state/server.ts"]);
    node(
        &project,
        "fallback-actor",
        r#"
globalThis.localStorage = area();
const ctx = scopeContext();
const { useServerState } = rt("state/server");
const { storageKey } = rt("state/storage");
const scopeFor = (actor) => ({ ...ctx._default, values: { actor } });
const A = { signed_in: true, user_id: "ua", account_id: "acc" };
const B = { signed_in: true, user_id: "ub", account_id: "acc" };
const key = "portal:pages/overview/state/layout";
localStorage.setItem(storageKey(key, A), JSON.stringify("A's layout"));
const contexts = new Map([[ctx, scopeFor(A)]]);
const hook = React.__mount(
  () => useServerState(key, "default", { fallback: { when: "actor.signed_in", store: "local_storage" } }),
  contexts,
);
hook.render();
hook.flush();
assert.strictEqual(hook.render()[0], "A's layout", "setup: actor A reads its layout from the fallback");
contexts.set(ctx, scopeFor(B));
hook.render();
hook.flush();
const [seen] = hook.render();
assert.strictEqual(seen, "default", `after the effects ran, actor B still holds actor A's value ${JSON.stringify(seen)}`);
"#,
    );
}

// ── stale_after and resume (Channel.stale_after, Channel.resume) ────────────────────────────

/// `activity` (`stale_after` 60s, backoff 1s..30s): once the down time passes `stale_after` the
/// channel is stale until it is live again; a retry that fails again is not "less stale".
#[test]
fn a_stale_channel_stays_stale_across_failed_retries() {
    let project = example("stale-retries");
    build_runtime(&project, &["runtime/live.ts"]);
    node(
        &project,
        "stale-retries",
        r#"
const sources = [];
globalThis.EventSource = class { constructor(url) { this.url = url; sources.push(this); } close() { this.closed = true; } };
const live = rt("live");
live.useNetworkChannels("http://h");
const hook = React.__mount(() => live.useChannels(["activity"]));
hook.render();
hook.flush();
const life = () => hook.render().activity.lifecycle;
sources[0].onopen();
assert.strictEqual(life(), "live", "setup: the channel goes live");
sources[0].onerror();
const seen = [life()];
let failed = 1;
const start = now;
while (now - start < 200000 && step()) {
  seen.push(life());
  while (sources.length > failed) {
    sources[failed++].onerror();
    seen.push(life());
  }
}
const first = seen.indexOf("stale");
assert(first >= 0, `never stale: ${seen.join(" ")}`);
const back = seen.slice(first).indexOf("reconnecting");
assert.strictEqual(back, -1, `stale flips back to reconnecting on a retry: ${seen.join(" ")}`);
"#,
    );
}

/// Live again clears the stale timer: a channel that recovered before `stale_after` is not
/// turned stale by the timer of the outage it recovered from.
#[test]
fn live_again_clears_the_stale_timer() {
    let project = example("stale-cleared");
    build_runtime(&project, &["runtime/live.ts"]);
    node(
        &project,
        "stale-cleared",
        r#"
const sources = [];
globalThis.EventSource = class { constructor(url) { this.url = url; sources.push(this); } close() {} };
const live = rt("live");
live.useNetworkChannels("http://h");
const hook = React.__mount(() => live.useChannels(["activity"]));
hook.render();
hook.flush();
const life = () => hook.render().activity.lifecycle;
sources[0].onopen();
sources[0].onerror();
while (sources.length < 2 && step()) {}
sources[1].onopen();
assert.strictEqual(life(), "live");
const start = now;
while (now - start < 120000 && step()) {}
now = start + 120000;
assert.strictEqual(life(), "live", "a recovered channel went stale later");
"#,
    );
}

/// `resume: refetch` re-reads once per down→live transition, not on the first connect and not
/// once per retry.
#[test]
fn a_reconnect_refetches_once_per_transition() {
    let project = example("refetch-once");
    build_runtime(&project, &["runtime/live.ts"]);
    node(
        &project,
        "refetch-once",
        r#"
const sources = [];
globalThis.EventSource = class { constructor(url) { this.url = url; sources.push(this); } close() {} };
const live = rt("live");
live.useNetworkChannels("http://h");
let refetches = 0;
const read = { status: "ready", rows: [], params: {}, refetch: () => { refetches += 1; } };
const hook = React.__mount(() => live.useLive(read, { channel: "tickets", effect: "insert_top" }));
hook.render();
hook.flush();
sources[0].onopen();
assert.strictEqual(refetches, 0, "the first connect refetched");
for (let flap = 1; flap <= 3; flap += 1) {
  const before = sources.length;
  sources[sources.length - 1].onerror();
  while (sources.length === before && step()) {}
  sources[sources.length - 1].onerror();
  while (sources.length === before + 1 && step()) {}
  sources[sources.length - 1].onopen();
  assert.strictEqual(refetches, flap, `after ${flap} outages with two retries each: ${refetches} refetches`);
}
"#,
    );
}

/// `ticket_chat` (both-way, so WebSocket) is `resume: from_last_seen`: the reconnect must carry
/// the last seen position, or the replies posted while it was down never arrive.
#[test]
fn a_websocket_from_last_seen_reconnect_carries_the_last_seen_position() {
    let project = example("ws-resume");
    build_runtime(&project, &["runtime/live.ts"]);
    node(
        &project,
        "ws-resume",
        r#"
const sockets = [];
globalThis.WebSocket = class { constructor(url) { this.url = url; this.sent = []; sockets.push(this); } send(frame) { this.sent.push(frame); } close() {} };
const live = rt("live");
live.useNetworkChannels("http://h");
const hook = React.__mount(() => live.useChannels(["ticket_chat"], { ticket_chat: "tk-01" }));
hook.render();
hook.flush();
sockets[0].onopen();
sockets[0].onmessage({ data: JSON.stringify({ event: "tickets.ReplyPosted", id: "m-7", payload: { id: "m-7", ticket_id: "tk-01" } }) });
sockets[0].onclose();
while (sockets.length < 2 && step()) {}
assert.strictEqual(sockets.length, 2, "setup: the socket reconnects");
sockets[1].onopen();
const carried = sockets[1].url.includes("m-7") || sockets[1].sent.some((frame) => frame.includes("m-7"));
assert(carried, `the reconnect resumes from nothing: url ${sockets[1].url}, frames ${JSON.stringify(sockets[1].sent)}`);
"#,
    );
}

// ── per-session channels (Channel.session) ──────────────────────────────────────────────────

const CHAT: &str = r"
format: ess-ui/1
app: chat
model: chat.system
placement_profile: fat
shells:
  app: {regions: {main: {kind: page_outlet}}}
navigation:
  home: tickets.detail
  sections: [{name: all, pages: [tickets.detail]}]
channels:
  chat:
    carries: {events: [chat.Posted]}
    direction: server_to_client
    delivery: every_event
    resume: from_last_seen
    scope: param
    session: {per: ticket_id}
pages:
  tickets.detail:
    kind: detail_page
    title: Ticket
    params: {id: string}
    sections:
      - name: messages
        component: collection
        reads: {view: chat.Messages, params: {ticket_id: params.id}}
        columns: [body]
        live: {channel: chat, effect: insert_top}
";

/// A one-way per-session channel with `from_last_seen`: the reconnect URL must keep the session
/// param and add the cursor as a second query param.
#[test]
fn an_sse_session_reconnect_keeps_the_session_and_the_cursor_apart() {
    let (project, result) = generate("sse-session", CHAT);
    result.unwrap_or_else(|error| panic!("{error}"));
    build_runtime(&project, &["runtime/live.ts"]);
    node(
        &project,
        "sse-session",
        r#"
const sources = [];
globalThis.EventSource = class { constructor(url) { this.url = url; sources.push(this); } close() {} };
const live = rt("live");
live.useNetworkChannels("http://h");
const hook = React.__mount(() => live.useChannels(["chat"], { chat: "tk-1" }));
hook.render();
hook.flush();
sources[0].onopen();
sources[0].onmessage({ data: JSON.stringify({ event: "chat.Posted", payload: { id: "m-7" } }), lastEventId: "m-7" });
sources[0].onerror();
while (sources.length < 2 && step()) {}
const url = new URL(sources[1].url);
assert.strictEqual(url.searchParams.get("ticket_id"), "tk-1", `reconnect URL ${sources[1].url}`);
assert.strictEqual(url.searchParams.get("last_event_id"), "m-7", `reconnect URL ${sources[1].url}`);
"#,
    );
}

const CHAT_NO_SESSION_VALUE: &str = r"
format: ess-ui/1
app: chat
model: chat.system
placement_profile: fat
shells:
  app: {regions: {main: {kind: page_outlet}}}
navigation:
  home: tickets.overview
  sections: [{name: all, pages: [tickets.overview]}]
channels:
  chat:
    carries: {events: [chat.Posted]}
    direction: server_to_client
    delivery: every_event
    resume: refetch
    scope: param
    session: {per: ticket_id}
pages:
  tickets.overview:
    kind: detail_page
    title: Overview
    header: {live: [chat]}
    sections:
      - name: recent
        component: collection
        reads: {view: chat.Recent}
        columns: [body]
        live: {channel: chat, effect: insert_top}
";

/// `session: {per: ticket_id}` is one instance per value. A page with no `ticket_id` anywhere
/// (no page param, no read param) has no value: generation must refuse rather than subscribe
/// every such page to one shared `undefined` session.
#[test]
fn a_session_channel_on_a_page_without_its_per_value_is_refused() {
    let (_, result) = generate("session-absent", CHAT_NO_SESSION_VALUE);
    match result {
        Err(error) => assert!(
            error.contains("chat") && error.contains("ticket_id"),
            "the refusal names the channel and its per param: {error}"
        ),
        Ok(files) => {
            let page = file(&files, "src/pages/TicketsOverview.tsx");
            let line = page
                .lines()
                .find(|line| line.contains("useChannels("))
                .unwrap_or("<none>");
            panic!(
                "generated without a ticket_id to key the session on; the page subscribes with:\n{line}"
            );
        }
    }
}

const CHAT_BY_SELECTION: &str = r#"
format: ess-ui/1
app: chat
model: chat.system
placement_profile: fat
shells:
  app: {regions: {main: {kind: page_outlet}}}
navigation:
  home: tickets.board
  sections: [{name: all, pages: [tickets.board]}]
channels:
  chat:
    carries: {events: [chat.Posted]}
    direction: server_to_client
    delivery: every_event
    resume: refetch
    scope: param
    session: {per: ticket_id}
pages:
  tickets.board:
    kind: detail_page
    title: Board
    header: {live: [chat]}
    state:
      current: {type: string, class: selection, default: "tk-1"}
    sections:
      - name: messages
        component: collection
        reads: {view: chat.Messages, params: {ticket_id: state.current}}
        columns: [body]
        live: {channel: chat, effect: insert_top}
"#;

/// The page-level `useChannels` and the section's `useLive` must key the same session instance:
/// both evaluate the section read's `ticket_id` expression, so both need the same scope. The
/// page evaluates it against `{ params }` only, so `state.current` is undefined there and the
/// header shows a second, sessionless instance.
#[test]
fn the_page_and_its_section_key_a_session_channel_on_the_same_value() {
    let (project, result) = generate("session-by-state", CHAT_BY_SELECTION);
    result.unwrap_or_else(|error| panic!("{error}"));
    build_runtime(&project, &["runtime/expr.ts"]);
    let files = std::fs::read_to_string(project.join("src/pages/TicketsBoard.tsx"))
        .expect("the page is generated");
    let line = files
        .lines()
        .find(|line| line.contains("useChannels("))
        .unwrap_or_else(|| panic!("no useChannels in:\n{files}"));
    let scope = line
        .split("evaluate(\"state.current\", ")
        .nth(1)
        .unwrap_or_else(|| panic!("the page keys chat on something else: {line}"));
    let scope = scope.split(')').next().unwrap_or_default().to_owned();
    // What that scope object gives `state.current`, evaluated by the generated runtime. The page
    // reads its state from its own hook (`currentValue`), so the harness supplies that local with
    // the value the section is given (coordinator, correction 2 verification).
    let values = scope
        .replace("__params", "{ id: \"x\" }")
        .replace("currentValue", "\"tk-1\"");
    node(
        &project,
        "session-by-state",
        &format!(
            r#"
const {{ evaluate }} = rt("expr");
const page = evaluate("state.current", {values});
const section = evaluate("state.current", {{ params: {{}}, state: {{ current: "tk-1" }} }});
assert.strictEqual(page, section, `the page keys chat on ${{JSON.stringify(page)}} (scope {scope}), the section on ${{JSON.stringify(section)}}`);
"#
        ),
    );
}

// ── runtime rows (NodePath.syntax.containers: rows by row key) ──────────────────────────────

/// Renders a collection section with two rows (ids r-1, r-2) as the generator frames it and
/// returns every `data-ui-path` the tree carries, as JSON on stdout.
const ROWS_SCRIPT: &str = r#"
const { SectionFrame } = rt("core");
const { Collection } = rt("composites/collection");
const data = { status: "ready", rows: [{ id: "r-1", title: "One" }, { id: "r-2", title: "Two" }], params: {}, total: 2, refetch() {} };
const at = "pages/rows.board/sections/patched";
const tree = jsx(SectionFrame, {
  "data-ui-path": at,
  name: "patched",
  data,
  children: jsx(Collection, {
    columns: [{ "data-ui-path": `${at}/columns/title`, field: "title", name: "title" }],
    rowActions: [{ "data-ui-path": `${at}/row_actions/open`, name: "open", label: "Open", navigate: { to: "rows.board", params: {} } }],
  }),
});
const paths = [];
const visit = (nodes) => nodes.forEach((n) => { if (typeof n === "object") { if (n.props["data-ui-path"] !== undefined) paths.push(n.props["data-ui-path"]); visit(n.children); } });
visit(React.__renderTree(tree));
"#;

/// Every rendered node's path is unique at runtime: a column or row action repeated per row is
/// addressed under its row (`…/rows/<row key>/…`), not by one path shared by every row.
#[test]
fn runtime_paths_are_unique_across_the_rows_of_a_collection() {
    let project = example("rows-unique");
    build_runtime(
        &project,
        &["runtime/core.tsx", "runtime/composites/collection.tsx"],
    );
    node(
        &project,
        "rows-unique",
        &format!(
            r"{ROWS_SCRIPT}
const counts = {{}};
paths.forEach((p) => {{ counts[p] = (counts[p] || 0) + 1; }});
const repeated = Object.entries(counts).filter(([, n]) => n > 1);
assert.deepStrictEqual(repeated, [], `paths rendered more than once: ${{JSON.stringify(repeated)}}`);
"
        ),
    );
}

/// Rows are addressed by their row key under `rows/`, the key being `live.match` first, then
/// `id` (never a list index).
#[test]
fn collection_rows_render_their_row_key_path() {
    let project = example("rows-keyed");
    build_runtime(
        &project,
        &["runtime/core.tsx", "runtime/composites/collection.tsx"],
    );
    node(
        &project,
        "rows-keyed",
        &format!(
            r#"{ROWS_SCRIPT}
for (const key of ["r-1", "r-2"]) {{
  assert(paths.includes(`${{at}}/rows/${{key}}`), `no ${{at}}/rows/${{key}} among ${{JSON.stringify(paths)}}`);
}}
"#
        ),
    );
}

/// The section generation must hand the collection what it needs to key rows by `live.match`.
#[test]
fn a_live_match_reaches_the_collection_that_keys_its_rows() {
    const EFFECTS: &str = r"
format: ess-ui/1
app: rows
model: rows.system
placement_profile: fat
shells:
  app: {regions: {main: {kind: page_outlet}}}
navigation:
  home: rows.board
  sections: [{name: all, pages: [rows.board]}]
channels:
  rows:
    carries: {events: [rows.Changed]}
    direction: server_to_client
    delivery: every_event
    resume: refetch
pages:
  rows.board:
    kind: detail_page
    title: Rows
    sections:
      - name: patched
        component: collection
        reads: {view: rows.All}
        columns: [title]
        live: {channel: rows, on: [rows.Changed], effect: patch_row, match: key}
";
    let (_, result) = generate("rows-match", EFFECTS);
    let files = result.unwrap_or_else(|error| panic!("{error}"));
    let page = file(&files, "src/pages/RowsBoard.tsx");
    let start = page.find("<Collection").expect("a collection is rendered");
    let element = &page[start
        ..page[start..]
            .find("/>")
            .map_or(page.len(), |end| start + end)];
    assert!(
        element.contains("\"key\""),
        "the collection never learns that rows are keyed by `key`; it keys by `row.id`, else the index:\n{element}"
    );
}
