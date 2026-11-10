//! The fixed files of a generated project: its configuration, the page it is served from, the
//! offline type declarations and the runtime component set (the router among them). Runtime
//! modules are emitted only when something imports them.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

use ess_ui::binding::Binding;

use crate::GenerateError;

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
    module!("runtime/hosts", "tsx"),
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

/// A template's text for a bound or a plain project, and for a bound one whose route table moves
/// a method or carries a path parameter ([`Binding::routed`]).
///
/// A line reading `//+bound` opens lines only a project bound to a served surface carries, and
/// `//-bound` closes them; `//+plain` … `//-plain` hold the lines they replace. Inside a bound
/// block, `//+routed` … `//-routed` hold lines only a routed project carries and `//+unrouted` …
/// `//-unrouted` the lines they replace (beyond10x/ess#713). The marker lines themselves are never
/// emitted, so a plain project is the template with every bound block removed, and an unrouted one
/// the template with every routed block removed — each byte for byte what it was before the
/// blocks were written.
pub(crate) fn select(text: &str, bound: bool, routed: bool) -> String {
    let mut out = String::with_capacity(text.len());
    // Whether each open block is kept; a line is emitted when every one is.
    let mut open: Vec<bool> = Vec::new();
    for line in text.split_inclusive('\n') {
        match line.trim() {
            "//+bound" => open.push(bound),
            "//+plain" => open.push(!bound),
            "//+routed" => open.push(routed),
            "//+unrouted" => open.push(!routed),
            "//-bound" | "//-plain" | "//-routed" | "//-unrouted" => {
                open.pop();
            }
            _ if open.iter().all(|kept| *kept) => out.push_str(line),
            _ => {}
        }
    }
    out
}

/// The runtime files for the modules used, closed over their imports.
pub(crate) fn runtime(
    used: &BTreeSet<String>,
    bound: bool,
    routed: bool,
) -> BTreeMap<String, String> {
    let table: BTreeMap<&str, (&Module, String)> = RUNTIME
        .iter()
        .map(|module| (module.id, (module, select(module.text, bound, routed))))
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
        select(
            text,
            binding.is_some(),
            binding.is_some_and(Binding::routed),
        )
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

/// Layout lengths the stylesheet needs that no token group holds: renderer-fixed constants, not
/// document-declared. Each is written into `tokens.css` as `--ui-size-<name>`, named by what it
/// sizes, so `styles.css` reads `var(--…)` for every length. A document cannot change them.
const FIXED_SIZES: &[(&str, &str)] = &[
    ("text-weight-regular", "normal"),
    ("viewport-height", "100vh"),
    ("card-min-width", "14rem"),
    ("chart-height", "10rem"),
    ("chart-label-room", "1.2rem"),
    ("dialog-width", "28rem"),
    ("drawer-width", "32rem"),
    ("fade-depth", "0.9rem"),
    ("fade-width", "2.5rem"),
    ("gap-tight", "0.35rem"),
    ("gap-wide", "0.75rem"),
    ("hairline", "1px"),
    ("hairline-thick", "2px"),
    ("image-max-height", "3rem"),
    ("nav-width", "15rem"),
    ("new-count-offset", "4rem"),
    ("pad-badge", "0.45rem"),
    ("pad-cell", "0.4rem"),
    ("pad-hair", "0.1rem"),
    ("pad-snug", "0.3rem"),
    ("row-body-basis", "12rem"),
    ("spinner-stroke", "3px"),
    ("swatch", "0.7rem"),
    ("text-metric", "1.6rem"),
    ("text-small", "0.75rem"),
    ("text-subtitle", "0.95rem"),
    ("text-tiny", "0.7rem"),
    ("text-title", "1.4rem"),
];

/// Why a token name cannot be written into a custom property name: `^[a-z][a-z0-9_-]*$`.
fn token_name_ok(name: &str) -> bool {
    let mut chars = name.chars();
    chars.next().is_some_and(|c| c.is_ascii_lowercase())
        && chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '-')
}

/// Whether a token value can be written into one CSS declaration without ending it or its rule:
/// none of `; { } < > \`, no comment and no control character.
fn token_value_ok(value: &str) -> bool {
    !value
        .chars()
        .any(|c| c.is_control() || [';', '{', '}', '<', '>', '\\'].contains(&c))
        && !value.contains("/*")
        && !value.contains("*/")
}

/// One declaration of `tokens.css`, refused when its name or value is not safe to write. `ess ui
/// check` refuses the same names and values at their key paths, but `generate` does not run it.
fn declare(
    out: &mut String,
    group: &str,
    name: &str,
    property: &str,
    value: &str,
) -> Result<(), GenerateError> {
    if !token_name_ok(name) {
        return Err(GenerateError::new(format!(
            "tokens/{group}/{name}: not a token name (a name is `^[a-z][a-z0-9_-]*$`)"
        )));
    }
    if !token_value_ok(value) {
        return Err(GenerateError::new(format!(
            "tokens/{group}/{name}: `{}` cannot be written into a CSS declaration (it holds one of \
             `; {{ }} < > \\`, a comment or a control character)",
            value.escape_debug()
        )));
    }
    let _ = writeln!(out, "  {property}: {value};");
    Ok(())
}

/// `src/runtime/tokens.css`: one `:root` block holding every token of the document's merged
/// table (the built-in table, then `tokens:`, `type` entries filled), as the custom properties
/// `styles.css` reads: `--ui-color-<name>`, `--ui-space-<name>`, `--ui-radius-<name>`,
/// `--ui-type-<style>-family|size|weight` and `--ui-tone-<tone>-text|fill`. The renderer-fixed
/// `--ui-size-<name>` lengths ([`FIXED_SIZES`]) follow the radii. A tone's colors are written as
/// a reference to the color token they name, so a color override reaches its tones.
///
/// A token name or value that could write outside its own custom property is refused with an
/// error naming the token.
pub(crate) fn tokens_css(document: &ess_ui::Document) -> Result<String, GenerateError> {
    let tokens = document.base_tokens();
    let mut out = String::from(":root {\n");
    for (group, entries) in [
        ("color", &tokens.color),
        ("space", &tokens.space),
        ("radius", &tokens.radius),
    ] {
        for (name, value) in entries {
            declare(
                &mut out,
                group,
                name,
                &format!("--ui-{group}-{name}"),
                &value.0,
            )?;
        }
    }
    for (name, value) in FIXED_SIZES {
        let _ = writeln!(out, "  --ui-size-{name}: {value};");
    }
    for (style, entry) in &tokens.typography {
        for (part, value) in [
            ("family", entry.family.as_deref()),
            ("size", entry.size.as_ref().map(|value| value.0.as_str())),
            (
                "weight",
                entry.weight.as_ref().map(|value| value.0.as_str()),
            ),
        ] {
            if let Some(value) = value {
                let property = format!("--ui-type-{style}-{part}");
                declare(&mut out, &format!("type/{style}"), part, &property, value)?;
                if !token_name_ok(style) {
                    return Err(GenerateError::new(format!(
                        "tokens/type/{style}: not a token name (a name is `^[a-z][a-z0-9_-]*$`)"
                    )));
                }
            }
        }
    }
    for (tone, entry) in &tokens.tone {
        for (role, color) in [("text", &entry.text), ("fill", &entry.fill)] {
            if !token_name_ok(color) {
                return Err(GenerateError::new(format!(
                    "tokens/tone/{tone}/{role}: `{color}` is not a token name"
                )));
            }
            let property = format!("--ui-tone-{tone}-{role}");
            declare(
                &mut out,
                "tone",
                tone,
                &property,
                &format!("var(--ui-color-{color})"),
            )?;
        }
    }
    out.push_str("}\n");
    Ok(out)
}
