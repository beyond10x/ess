//! The fixed files of a generated project: its configuration, the page it is served from, the
//! offline type declarations and the runtime component set (the router among them). Runtime
//! modules are emitted only when something imports them.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

use ess_ui::binding::Binding;
use ess_ui::token_grammar;

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
/// blocks were written. `//+effects` … `//-effects` hold the lines of a fixture project generated
/// with the effects of its model, and `//+noeffects` … `//-noeffects` those of one without.
pub(crate) fn select(text: &str, bound: bool, routed: bool, effects: bool) -> String {
    let mut out = String::with_capacity(text.len());
    // Whether each open block is kept; a line is emitted when every one is.
    let mut open: Vec<bool> = Vec::new();
    for line in text.split_inclusive('\n') {
        match line.trim() {
            "//+bound" => open.push(bound),
            "//+plain" => open.push(!bound),
            "//+routed" => open.push(routed),
            "//+unrouted" => open.push(!routed),
            "//+effects" => open.push(effects),
            "//+noeffects" => open.push(!effects),
            "//-bound" | "//-plain" | "//-routed" | "//-unrouted" | "//-effects"
            | "//-noeffects" => {
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
    effects: bool,
) -> BTreeMap<String, String> {
    let table: BTreeMap<&str, (&Module, String)> = RUNTIME
        .iter()
        .map(|module| {
            (
                module.id,
                (module, select(module.text, bound, routed, effects)),
            )
        })
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
            false,
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

/// Fixed non-length constants the stylesheet needs (a stroke width, a dash pattern, a unitless
/// line height, a round radius, the transparent color): renderer-fixed, not document-declared.
/// Each is written into `tokens.css` as `--ui-fixed-<name>`, so `styles.css` holds no literal.
const FIXED_CONSTANTS: &[(&str, &str)] = &[
    ("chart-dash", "2 1"),
    ("chart-stroke", "0.6"),
    ("line-height-tight", "1"),
    ("radius-round", "50%"),
    ("transparent", "transparent"),
];

/// One declaration of `tokens.css`, refused when its name or value is not safe to write. `ess ui
/// check` refuses the same names and values at their key paths (the grammars are
/// [`ess_ui::token_grammar`]'s), but `generate` does not run the check. `problem` is the grammar
/// of the value's group.
fn declare(
    out: &mut String,
    path: &str,
    name: &str,
    property: &str,
    value: &str,
    problem: fn(&str) -> Option<String>,
) -> Result<(), GenerateError> {
    if let Some(why) = token_grammar::name_problem(name) {
        return Err(GenerateError::new(format!("{path}/{name}: {why}")));
    }
    if let Some(why) = problem(value) {
        return Err(GenerateError::new(format!("{path}/{name}: {why}")));
    }
    let _ = writeln!(out, "  {property}: {value};");
    Ok(())
}

/// `src/runtime/tokens.css`: one `:root` block holding every token of the document's merged
/// table (the built-in table, then `tokens:`, `type` entries filled), as the custom properties
/// `styles.css` reads: `--ui-color-<name>`, `--ui-space-<name>`, `--ui-radius-<name>`,
/// `--ui-type-<style>-family|size|weight` and `--ui-tone-<tone>-text|fill`. The renderer-fixed
/// `--ui-size-<name>` lengths ([`FIXED_SIZES`]) and `--ui-fixed-<name>` constants
/// ([`FIXED_CONSTANTS`]) follow the radii. A tone's colors are written as a reference to the color
/// token they name, so a color override reaches its tones.
///
/// A token name or value outside its grammar, or one that could write outside its own custom
/// property, is refused with an error naming the token.
pub(crate) fn tokens_css(document: &ess_ui::Document) -> Result<String, GenerateError> {
    let tokens = document.base_tokens();
    let mut out = String::from(":root {\n");
    for (group, entries, problem) in [
        (
            "color",
            &tokens.color,
            token_grammar::color_value_problem as fn(&str) -> Option<String>,
        ),
        ("space", &tokens.space, token_grammar::length_value_problem),
        (
            "radius",
            &tokens.radius,
            token_grammar::length_value_problem,
        ),
    ] {
        for (name, value) in entries {
            declare(
                &mut out,
                &format!("tokens/{group}"),
                name,
                &format!("--ui-{group}-{name}"),
                &value.0,
                problem,
            )?;
        }
    }
    for (name, value) in FIXED_SIZES {
        let _ = writeln!(out, "  --ui-size-{name}: {value};");
    }
    for (name, value) in FIXED_CONSTANTS {
        let _ = writeln!(out, "  --ui-fixed-{name}: {value};");
    }
    for (style, entry) in &tokens.typography {
        if let Some(why) = token_grammar::name_problem(style) {
            return Err(GenerateError::new(format!("tokens/type/{style}: {why}")));
        }
        for (part, value, problem) in [
            (
                "family",
                entry.family.as_deref(),
                token_grammar::css_unsafe as fn(&str) -> Option<String>,
            ),
            (
                "size",
                entry.size.as_ref().map(|value| value.0.as_str()),
                token_grammar::size_value_problem,
            ),
            (
                "weight",
                entry.weight.as_ref().map(|value| value.0.as_str()),
                token_grammar::weight_value_problem,
            ),
        ] {
            if let Some(value) = value {
                declare(
                    &mut out,
                    &format!("tokens/type/{style}"),
                    part,
                    &format!("--ui-type-{style}-{part}"),
                    value,
                    problem,
                )?;
            }
        }
    }
    for (tone, entry) in &tokens.tone {
        for (role, color) in [("text", &entry.text), ("fill", &entry.fill)] {
            if let Some(why) = token_grammar::name_problem(color) {
                return Err(GenerateError::new(format!(
                    "tokens/tone/{tone}/{role}: {why}"
                )));
            }
            declare(
                &mut out,
                &format!("tokens/tone/{tone}"),
                role,
                &format!("--ui-tone-{tone}-{role}"),
                &format!("var(--ui-color-{color})"),
                token_grammar::css_unsafe,
            )?;
        }
    }
    out.push_str("}\n");
    Ok(out)
}
