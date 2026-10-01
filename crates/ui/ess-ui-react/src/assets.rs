//! The fixed files of a generated project: its configuration, the page it is served from, the
//! offline type declarations and the runtime component set (the router among them). Runtime
//! modules are emitted only when something imports them.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

use ess_ui::binding::Binding;

/// A runtime module: its id (path under `src/` without extension), output file and text.
struct Module {
    id: &'static str,
    file: &'static str,
    text: &'static str,
}

macro_rules! module {
    ($id:literal, $ext:literal) => {
        Module {
            id: $id,
            file: concat!("src/", $id, ".", $ext),
            text: include_str!(concat!("../templates/", $id, ".", $ext, ".tmpl")),
        }
    };
}

const RUNTIME: &[Module] = &[
    module!("runtime/json", "ts"),
    module!("runtime/expr", "ts"),
    module!("runtime/data", "ts"),
    module!("runtime/answer", "ts"),
    module!("runtime/core", "tsx"),
    module!("runtime/router", "tsx"),
    module!("runtime/actions", "tsx"),
    module!("runtime/fields", "tsx"),
    module!("runtime/overlays", "tsx"),
    module!("runtime/live", "ts"),
    module!("runtime/shell", "tsx"),
    module!("runtime/navigation", "tsx"),
    module!("runtime/widget", "tsx"),
    module!("runtime/composites/collection", "tsx"),
    module!("runtime/composites/record", "tsx"),
    module!("runtime/composites/form", "tsx"),
    module!("runtime/composites/choice", "tsx"),
    module!("runtime/composites/filter_bar", "tsx"),
    module!("runtime/composites/confirm", "tsx"),
    module!("runtime/composites/metric", "tsx"),
    module!("runtime/composites/chart", "tsx"),
    module!("runtime/composites/board", "tsx"),
    module!("runtime/composites/graph_editor", "tsx"),
    module!("runtime/composites/rich_text", "tsx"),
    module!("runtime/composites/references", "tsx"),
    module!("runtime/primitives/text", "tsx"),
    module!("runtime/primitives/badge", "tsx"),
    module!("runtime/primitives/icon", "tsx"),
    module!("runtime/primitives/button", "tsx"),
    module!("runtime/primitives/link", "tsx"),
    module!("runtime/primitives/input", "tsx"),
    module!("runtime/primitives/toggle", "tsx"),
    module!("runtime/primitives/image", "tsx"),
    module!("runtime/primitives/divider", "tsx"),
    module!("runtime/state/memory", "ts"),
    module!("runtime/state/url", "ts"),
    module!("runtime/state/storage", "ts"),
    module!("runtime/state/session_storage", "ts"),
    module!("runtime/state/local_storage", "ts"),
    module!("runtime/state/remote", "ts"),
    module!("runtime/state/server", "ts"),
    module!("runtime/state/server_session", "ts"),
];

/// Modules every project has: the scope, data adapter and expression evaluator.
const ALWAYS: &[&str] = &[
    "runtime/json",
    "runtime/expr",
    "runtime/data",
    "runtime/core",
];

/// Every relative import of a module, resolved to a module id.
fn imports(id: &str, text: &str) -> Vec<String> {
    let mut found = Vec::new();
    for line in text.lines() {
        let Some(start) = line.find(" from \".") else {
            continue;
        };
        let rest = &line[start + " from \"".len()..];
        let Some(end) = rest.find('"') else {
            continue;
        };
        let mut segments: Vec<&str> = id.split('/').collect();
        segments.pop();
        for part in rest[..end].split('/') {
            match part {
                "." => {}
                ".." => {
                    segments.pop();
                }
                other => segments.push(other),
            }
        }
        found.push(segments.join("/"));
    }
    found
}

/// A template's text for a bound or a plain project.
///
/// A line reading `//+bound` opens lines only a project bound to a served surface carries, and
/// `//-bound` closes them; `//+plain` … `//-plain` hold the lines they replace. The marker lines
/// themselves are never emitted, so a plain project is the template with every bound block
/// removed — byte for byte what it was before the blocks were written.
pub(crate) fn select(text: &str, bound: bool) -> String {
    let mut out = String::with_capacity(text.len());
    let mut keep = true;
    for line in text.split_inclusive('\n') {
        match line.trim() {
            "//+bound" => keep = bound,
            "//+plain" => keep = !bound,
            "//-bound" | "//-plain" => keep = true,
            _ if keep => out.push_str(line),
            _ => {}
        }
    }
    out
}

/// The runtime files for the modules used, closed over their imports.
pub(crate) fn runtime(used: &BTreeSet<String>, bound: bool) -> BTreeMap<String, String> {
    let table: BTreeMap<&str, (&Module, String)> = RUNTIME
        .iter()
        .map(|module| (module.id, (module, select(module.text, bound))))
        .collect();
    let mut wanted: BTreeSet<String> = used.clone();
    wanted.extend(ALWAYS.iter().map(|id| (*id).to_owned()));
    let mut queue: Vec<String> = wanted.iter().cloned().collect();
    while let Some(id) = queue.pop() {
        if let Some((module, text)) = table.get(id.as_str()) {
            for dependency in imports(module.id, text) {
                if dependency.starts_with("runtime/") && wanted.insert(dependency.clone()) {
                    queue.push(dependency);
                }
            }
        }
    }
    let mut files = BTreeMap::new();
    for id in wanted {
        if let Some((module, text)) = table.get(id.as_str()) {
            files.insert(module.file.to_owned(), text.clone());
        }
    }
    files.insert(
        "src/runtime/styles.css".to_owned(),
        include_str!("../templates/runtime/styles.css.tmpl").to_owned(),
    );
    files
}

/// Configuration, entry point and offline declarations, with the app's name and title filled in;
/// bound to the served surface when `binding` is given, with one base-URL `<meta>` per served
/// component in `www/index.html`.
pub(crate) fn project(
    app: &str,
    title: &str,
    binding: Option<&Binding>,
) -> BTreeMap<String, String> {
    let mut metas = String::new();
    for component in binding.iter().flat_map(|binding| binding.components.keys()) {
        let _ = writeln!(
            metas,
            "    <meta name=\"ess-base-url:{}\" content=\"\" />",
            html_attribute(component)
        );
    }
    let fill = |text: &str| {
        select(text, binding.is_some())
            .replace("{{app}}", app)
            .replace("{{title}}", title)
            .replace("    {{base_url_metas}}\n", &metas)
    };
    let mut files = BTreeMap::new();
    for (file, text) in [
        (
            "package.json",
            include_str!("../templates/project/package.json.tmpl"),
        ),
        (
            "tsconfig.json",
            include_str!("../templates/project/tsconfig.json.tmpl"),
        ),
        (
            "tsconfig.offline.json",
            include_str!("../templates/project/tsconfig.offline.json.tmpl"),
        ),
        (
            "www/index.html",
            include_str!("../templates/project/index.html.tmpl"),
        ),
        (
            "README.md",
            include_str!("../templates/project/README.md.tmpl"),
        ),
        (
            "src/main.tsx",
            include_str!("../templates/project/main.tsx.tmpl"),
        ),
        (
            "src/assets.d.ts",
            include_str!("../templates/project/assets.d.ts.tmpl"),
        ),
        (
            "types/react.d.ts",
            include_str!("../templates/types/react.d.ts.tmpl"),
        ),
        (
            "types/react-jsx-runtime.d.ts",
            include_str!("../templates/types/react-jsx-runtime.d.ts.tmpl"),
        ),
        (
            "types/react-dom-client.d.ts",
            include_str!("../templates/types/react-dom-client.d.ts.tmpl"),
        ),
    ] {
        files.insert(file.to_owned(), fill(text));
    }
    files
}

/// `src/binding.ts`: the binding as data, for the `httpAdapter` `main.tsx` switches on. The
/// literal is the binding's JSON, so what the adapter reads is what `ess_ui_check::binding`
/// computed, member for member.
pub(crate) fn binding(binding: &Binding) -> String {
    let json = serde_json::to_string_pretty(binding).unwrap_or_else(|_| "{}".to_owned());
    format!(
        "// Generated by ess-ui-react from the served surface of model `{}` (`ess generate ui\n\
         // --model`). Do not edit: generate again when the model changes.\n\
         import type {{ Binding }} from \"./runtime/data\";\n\n\
         export const binding: Binding = {json};\n",
        binding.system.replace('\n', " ")
    )
}

/// `text` safe inside a double-quoted HTML attribute.
fn html_attribute(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}
