//! Runs generated modules under `node`: a small stand-in for `react` that renders a tree from
//! its root, keeps each component's hooks by its position, runs effects after a render and
//! renders again when state changes; a stand-in browser (`window.location`, `window.history`,
//! `popstate`); and a `CommonJS` build of the modules a case needs, made with `tsc`. Everything
//! is written into the scratch project, never into the generated output.

#![allow(dead_code)]

use std::path::Path;
use std::process::Command;

const REACT_STUB: &str = r#""use strict";
let current = null;
let scheduled = false;
const ids = new WeakMap();
let nextId = 0;
const idOf = (fn) => {
  if (!ids.has(fn)) ids.set(fn, ++nextId);
  return ids.get(fn);
};
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
  if (!s.set) {
    s.set = (next) => {
      const value = typeof next === "function" ? next(s.value) : next;
      if (!Object.is(value, s.value)) {
        s.value = value;
        scheduled = true;
      }
    };
  }
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
  return ctx;
};
exports.Fragment = (props) => props.children;
exports.StrictMode = (props) => props.children;
// Renders `element` from the root. Components `expand` refuses are left as leaves
// `{ component, props, at }`; `at` is the position a component keeps its hooks by.
exports.__root = (element, options) => {
  const expand = (options && options.expand) || (() => true);
  const instances = new Map();
  const pending = [];
  const root = { tree: [], leaves: [], rendered: [] };
  function walk(node, ctx, at) {
    if (node === null || node === undefined || typeof node === "boolean") return [];
    if (typeof node === "string" || typeof node === "number") return [String(node)];
    if (Array.isArray(node)) {
      return node.flatMap((n, i) => walk(n, ctx, `${at}[${n && n.key != null ? n.key : i}]`));
    }
    if (typeof node.type === "function") {
      if (node.type._ctx) {
        const next = new Map(ctx);
        next.set(node.type._ctx, node.props.value);
        return walk(node.props.children, next, `${at}/P${idOf(node.type)}`);
      }
      const here = `${at}/${idOf(node.type)}${node.key != null ? `:${node.key}` : ""}`;
      if (!expand(node.type)) {
        root.leaves.push({ component: node.type, props: node.props, at: here });
        return [{ component: node.type, props: node.props }];
      }
      let inst = instances.get(here);
      if (!inst) {
        inst = { slots: [], index: 0, pending: [] };
        instances.set(here, inst);
      }
      inst.seen = true;
      inst.ctx = ctx;
      inst.index = 0;
      const previous = current;
      current = inst;
      let out;
      try {
        out = node.type(node.props);
      } finally {
        current = previous;
      }
      pending.push(...inst.pending.splice(0));
      root.rendered.push({ component: node.type, props: node.props, at: here });
      return walk(out, ctx, here);
    }
    if (node.type === undefined && typeof node[Symbol.iterator] === "function") return walk([...node], ctx, at);
    return [{ tag: node.type, props: node.props, children: walk(node.props.children, ctx, `${at}/${node.type}`) }];
  }
  root.render = () => {
    for (const inst of instances.values()) inst.seen = false;
    root.leaves = [];
    root.rendered = [];
    scheduled = false;
    root.tree = walk(element, new Map(), "");
    for (const [key, inst] of instances) {
      if (!inst.seen) {
        for (const s of inst.slots) if (s && s.cleanup) s.cleanup();
        instances.delete(key);
      }
    }
    pending.splice(0).forEach((effect) => effect());
    return root;
  };
  root.settle = () => {
    let passes = 0;
    do {
      if (++passes > 20) throw new Error("the tree never settles");
      root.render();
    } while (scheduled);
    return root;
  };
  root.find = (predicate) => {
    const found = [];
    const visit = (nodes) => nodes.forEach((n) => {
      if (typeof n !== "object") return;
      if (predicate(n)) found.push(n);
      if (n.children) visit(n.children);
    });
    visit(root.tree);
    return found;
  };
  root.text = () => {
    const parts = [];
    const visit = (nodes) => nodes.forEach((n) => (typeof n === "string" ? parts.push(n) : n.children && visit(n.children)));
    visit(root.tree);
    return parts.join("");
  };
  return root;
};
"#;

const JSX_STUB: &str = r#""use strict";
const React = require("./index.js");
function jsx(type, props, key) { return { type, props: props || {}, key: key === undefined ? null : key }; }
module.exports = { jsx, jsxs: jsx, Fragment: React.Fragment };
"#;

/// Shared by every script: `assert`, the stand-ins, `out(<module under src/>)`, and
/// `browser(<url path>)`, which installs `window` at that path and records every history call.
const PRELUDE: &str = r#""use strict";
const assert = require("assert");
const React = require("react");
const { jsx } = require("react/jsx-runtime");
const out = (p) => require("./test-out/src/" + p);
function browser(path) {
  let url = new URL(path, "http://app.test");
  const calls = [];
  const listeners = new Map();
  globalThis.window = {
    location: {
      get pathname() { return url.pathname; },
      get search() { return url.search; },
      get hash() { return url.hash; },
      get href() { return url.href; },
    },
    history: {
      pushState(_state, _title, href) { calls.push(["push", String(href)]); url = new URL(href, url); },
      replaceState(_state, _title, href) { calls.push(["replace", String(href)]); url = new URL(href, url); },
    },
    addEventListener(type, listener) {
      if (!listeners.has(type)) listeners.set(type, new Set());
      listeners.get(type).add(listener);
    },
    removeEventListener(type, listener) {
      if (listeners.has(type)) listeners.get(type).delete(listener);
    },
  };
  return {
    calls,
    get path() { return url.pathname + url.search + url.hash; },
    // The browser's back or forward button: the address changes, then `popstate` fires.
    pop(next) {
      url = new URL(next, url);
      for (const listener of listeners.get("popstate") || []) listener({ type: "popstate" });
    },
    listening(type) { return (listeners.get(type) || new Set()).size; },
  };
}
// A click on an element: `init` overrides the unmodified primary-button defaults.
function click(element, init) {
  const event = {
    button: 0, metaKey: false, altKey: false, ctrlKey: false, shiftKey: false,
    defaultPrevented: false,
    preventDefault() { this.defaultPrevented = true; },
    stopPropagation() {},
    ...init,
  };
  element.props.onClick(event);
  return event;
}
"#;

pub fn write(path: &Path, text: &str) {
    std::fs::create_dir_all(path.parent().expect("a parent")).expect("the directory is created");
    std::fs::write(path, text).expect("the file is written");
}

/// Compiles `modules` (paths under `src/`) and what they import to `CommonJS` under `test-out/`.
pub fn compile(project: &Path, modules: &[&str]) {
    write(
        &project.join("node_modules/react/package.json"),
        r#"{"name":"react","main":"index.js","type":"commonjs"}"#,
    );
    write(&project.join("node_modules/react/index.js"), REACT_STUB);
    write(&project.join("node_modules/react/jsx-runtime.js"), JSX_STUB);
    let files: Vec<String> = modules.iter().map(|m| format!("\"src/{m}\"")).collect();
    write(
        &project.join("tsconfig.test.json"),
        &format!(
            r#"{{
  "extends": "./tsconfig.offline.json",
  "compilerOptions": {{
    "noEmit": false, "outDir": "test-out", "rootDir": ".", "module": "commonjs",
    "moduleResolution": "node10", "ignoreDeprecations": "6.0"
  }},
  "include": [],
  "files": [{}]
}}"#,
            files.join(", ")
        ),
    );
    let output = Command::new("tsc")
        .args(["-p", "tsconfig.test.json"])
        .current_dir(project)
        .output()
        .expect("tsc is on PATH");
    assert!(
        output.status.success(),
        "tsc failed in {}:\n{}{}",
        project.display(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    write(
        &project.join("test-out/package.json"),
        r#"{"type":"commonjs"}"#,
    );
}

/// Runs `script` (after the prelude) under node in `project`; panics with its output on failure.
pub fn node(project: &Path, name: &str, script: &str) {
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
