//! The style checks: `tokens:` (`token_values`, `token_names`, `token_refs`), `themes:`
//! (`theme_tokens`), `theme:` (`theme_choice`) and `tone_maps:` (`tone_map_unused`).
//! `tone_map_refs` is the loader's refusal of a `tones` naming no map, filed by `classify`.
//!
//! The split is by subject: a value written under `tokens:` breaks a `token_*` check, and the
//! same fault written in a theme's overrides breaks `theme_tokens`.

use std::collections::{BTreeMap, BTreeSet};

use ess_ui::{
    Body, Document, NodePath, Primitive, StateClass, TokenValue, Tokens, TypeExpr, TypeToken,
};

use crate::schema;
use crate::walk::body_of;
use crate::Sink;

pub(crate) fn run(document: &Document, sink: &mut Sink) {
    let root = NodePath::root();
    let base = Tokens::builtin().merged(&document.tokens);
    tokens(&document.tokens, &base, &root.child("tokens"), sink);
    for (name, overrides) in &document.themes {
        let full = base.merged(overrides);
        theme(
            overrides,
            &base,
            &full,
            &root.child("themes").child(name),
            sink,
        );
    }
    theme_choice(document, sink);
    tone_map_unused(document, sink);
}

// ── grammars ─────────────────────────────────────────────────────────────────────────────────

/// Why `value` is not a color of `Tokens.grammar.color`, or `None` when it is one.
fn color_problem(value: &TokenValue) -> Option<String> {
    let text = value.0.as_str();
    if let Some(digits) = text.strip_prefix('#') {
        let hex = digits.chars().all(|c| c.is_ascii_hexdigit());
        return (!(hex && [3, 6, 8].contains(&digits.len()))).then(|| {
            format!("`{text}` is not a color: a hex color is `#rgb`, `#rrggbb` or `#rrggbbaa`")
        });
    }
    if let Some(inner) = text
        .strip_prefix("rgb(")
        .and_then(|rest| rest.strip_suffix(')'))
    {
        let parts: Vec<&str> = inner.split_whitespace().collect();
        let channel =
            |part: &str| part.parse::<u8>().is_ok() && part.chars().all(|c| c.is_ascii_digit());
        let alpha = |part: &str| {
            is_decimal(part)
                && part
                    .parse::<f64>()
                    .is_ok_and(|alpha| (0.0..=1.0).contains(&alpha))
        };
        let fits = matches!(parts.as_slice(), [r, g, b, "/", a]
            if channel(r) && channel(g) && channel(b) && alpha(a));
        return (!fits).then(|| {
            format!(
                "`{text}` is not a color: `rgb(r g b / a)` takes r, g and b from 0 to 255 and a \
                 from 0 to 1"
            )
        });
    }
    let shown = if text.is_empty() {
        "an empty value (an unquoted `#` starts a YAML comment; quote the color)".to_owned()
    } else {
        format!("`{text}`")
    };
    Some(format!(
        "{shown} is not a color: a color is `#rgb`, `#rrggbb`, `#rrggbbaa` or `rgb(r g b / a)`, \
         never `hsl()` or a color name"
    ))
}

/// A decimal: digits, optionally with a fraction (`1`, `0.25`, `.5`).
fn is_decimal(text: &str) -> bool {
    let (whole, fraction) = text.split_once('.').unwrap_or((text, ""));
    let digits = |part: &str| part.chars().all(|c| c.is_ascii_digit());
    let has_fraction = text.contains('.');
    digits(whole)
        && digits(fraction)
        && (!whole.is_empty() || !fraction.is_empty())
        && (!has_fraction || !fraction.is_empty())
}

/// Why `value` is not a length of `Tokens.grammar.length`, or `None` when it is one.
fn length_problem(value: &TokenValue) -> Option<String> {
    let text = value.0.as_str();
    if text == "0" {
        return None;
    }
    let fits = ["rem", "px", "em"]
        .iter()
        .any(|unit| text.strip_suffix(unit).is_some_and(is_decimal));
    (!fits).then(|| {
        format!("`{text}` is not a length: a length is `0`, or a decimal with `px`, `rem` or `em`")
    })
}

/// Why `value` is not a weight of `Tokens.grammar.weight`, or `None` when it is one.
fn weight_problem(value: &TokenValue) -> Option<String> {
    let text = value.0.as_str();
    let fits = text.chars().all(|c| c.is_ascii_digit())
        && text
            .parse::<u32>()
            .is_ok_and(|weight| (100..=900).contains(&weight) && weight % 100 == 0);
    (!fits).then(|| format!("`{text}` is not a weight: a weight is 100 to 900 in steps of 100"))
}

/// Every value of `tokens` outside its group's grammar, by path.
fn value_problems(tokens: &Tokens, at: &NodePath) -> Vec<(NodePath, String)> {
    let mut problems = Vec::new();
    for (name, value) in &tokens.color {
        problems
            .extend(color_problem(value).map(|problem| (at.child("color").child(name), problem)));
    }
    for (group, entries) in [("space", &tokens.space), ("radius", &tokens.radius)] {
        for (name, value) in entries {
            problems.extend(
                length_problem(value).map(|problem| (at.child(group).child(name), problem)),
            );
        }
    }
    for (style, entry) in &tokens.typography {
        let here = at.child("type").child(style);
        let TypeToken { size, weight, .. } = entry;
        if let Some(size) = size {
            problems.extend(length_problem(size).map(|problem| (here.child("size"), problem)));
        }
        if let Some(weight) = weight {
            problems.extend(weight_problem(weight).map(|problem| (here.child("weight"), problem)));
        }
    }
    problems
}

/// Every `tone` entry of `tokens` naming a color `colors` does not declare, by path.
fn tone_problems(
    tokens: &Tokens,
    colors: &BTreeMap<String, TokenValue>,
    at: &NodePath,
) -> Vec<(NodePath, String)> {
    let mut problems = Vec::new();
    for (tone, entry) in &tokens.tone {
        for (role, color) in [("text", &entry.text), ("fill", &entry.fill)] {
            if !colors.contains_key(color) {
                problems.push((
                    at.child("tone").child(tone).child(role),
                    format!("`{color}` names no color of the built-in table or `tokens.color`"),
                ));
            }
        }
    }
    problems
}

// ── tokens ───────────────────────────────────────────────────────────────────────────────────

fn tokens(tokens: &Tokens, base: &Tokens, at: &NodePath, sink: &mut Sink) {
    for (path, problem) in value_problems(tokens, at) {
        sink.push("token_values", &path, problem);
    }
    for (group, names) in [
        ("type", tokens.typography.keys().collect::<Vec<_>>()),
        ("tone", tokens.tone.keys().collect()),
    ] {
        let known = schema::token_names(group);
        for name in names {
            if !known.contains(name) {
                sink.push(
                    "token_names",
                    &at.child(group).child(name),
                    format!(
                        "`{name}` is not a {}; the names are {}",
                        if group == "type" {
                            "text style"
                        } else {
                            "tone"
                        },
                        known.join(", ")
                    ),
                );
            }
        }
    }
    for (path, problem) in tone_problems(tokens, &base.color, at) {
        sink.push("token_refs", &path, problem);
    }
}

// ── themes ───────────────────────────────────────────────────────────────────────────────────

fn theme(overrides: &Tokens, base: &Tokens, full: &Tokens, at: &NodePath, sink: &mut Sink) {
    let groups: [(&str, Vec<&String>, Vec<&String>); 5] = [
        (
            "color",
            overrides.color.keys().collect(),
            base.color.keys().collect(),
        ),
        (
            "space",
            overrides.space.keys().collect(),
            base.space.keys().collect(),
        ),
        (
            "radius",
            overrides.radius.keys().collect(),
            base.radius.keys().collect(),
        ),
        (
            "type",
            overrides.typography.keys().collect(),
            base.typography.keys().collect(),
        ),
        (
            "tone",
            overrides.tone.keys().collect(),
            base.tone.keys().collect(),
        ),
    ];
    for (group, names, declared) in groups {
        for name in names {
            if !declared.contains(&name) {
                sink.push(
                    "theme_tokens",
                    &at.child(group).child(name),
                    format!(
                        "`{name}` names no {group} token: a theme overrides only a token the \
                         built-in table or `tokens:` declares"
                    ),
                );
            }
        }
    }
    for (path, problem) in value_problems(overrides, at) {
        sink.push("theme_tokens", &path, problem);
    }
    for (path, problem) in tone_problems(overrides, &full.color, at) {
        sink.push("theme_tokens", &path, problem);
    }
}

// ── theme_choice ─────────────────────────────────────────────────────────────────────────────

fn theme_choice(document: &Document, sink: &mut Sink) {
    let Some(choice) = &document.theme else {
        return;
    };
    let at = NodePath::root().child("theme");
    if document.themes.is_empty() {
        sink.push(
            "theme_choice",
            &at,
            "`theme:` chooses among `themes:`, and the document declares none",
        );
        return;
    }
    let themes: Vec<&str> = document.themes.keys().map(String::as_str).collect();
    let listed = themes.join(", ");
    if !document.themes.contains_key(&choice.default) {
        sink.push(
            "theme_choice",
            &at.child("default"),
            format!(
                "`{}` names no theme; the themes are {listed}",
                choice.default
            ),
        );
    }
    let Some(chosen_by) = &choice.chosen_by else {
        return;
    };
    let here = at.child("chosen_by");
    let Some(name) = chosen_by.0.strip_prefix("shell.").filter(|name| {
        !name.is_empty() && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
    }) else {
        sink.push(
            "theme_choice",
            &here,
            format!(
                "`{}` is not shell state: `chosen_by` is `shell.<name>`",
                chosen_by.0
            ),
        );
        return;
    };
    let declaring: Vec<(&String, &ess_ui::State)> = document
        .shells
        .iter()
        .filter_map(|(shell, declared)| declared.state.get(name).map(|state| (shell, state)))
        .collect();
    if declaring.is_empty() {
        sink.push(
            "theme_choice",
            &here,
            format!("`shell.{name}` names a state no shell declares"),
        );
        return;
    }
    for (shell, state) in declaring {
        let path = NodePath::root()
            .child("shells")
            .child(shell)
            .child("state")
            .child(name);
        if state.class != StateClass::Preference {
            sink.push(
                "theme_choice",
                &path,
                format!("`shell.{name}` chooses the theme, so it is class `preference`"),
            );
        }
        match enum_values(document, &state.ty, 0) {
            None => sink.push(
                "theme_choice",
                &path.child("type"),
                format!("`shell.{name}` chooses the theme, so its type is an enum of theme names"),
            ),
            Some(variants) => {
                for variant in variants.iter().filter(|v| !themes.contains(&v.as_str())) {
                    sink.push(
                        "theme_choice",
                        &path.child("type"),
                        format!("the variant `{variant}` names no theme; the themes are {listed}"),
                    );
                }
            }
        }
        let default = state.default.as_ref().and_then(serde_yaml::Value::as_str);
        if let Some(default) = default.filter(|default| !themes.contains(default)) {
            sink.push(
                "theme_choice",
                &path.child("default"),
                format!("the default `{default}` names no theme; the themes are {listed}"),
            );
        }
    }
}

/// The variants of an enum type, following the document's named types; `None` for any other
/// type.
fn enum_values(document: &Document, ty: &TypeExpr, depth: usize) -> Option<Vec<String>> {
    match ty {
        TypeExpr::Enum(values) => Some(values.values.clone()),
        TypeExpr::Named(name) if depth < 32 => document
            .types
            .get(name)
            .and_then(|declared| enum_values(document, declared, depth + 1)),
        _ => None,
    }
}

// ── tone_map_unused ──────────────────────────────────────────────────────────────────────────

fn tone_map_unused(document: &Document, sink: &mut Sink) {
    let mut named = BTreeSet::new();
    for located in document.nodes() {
        let tone_by = match body_of(located.node) {
            Some(Body::Primitive(Primitive::Badge(badge))) => badge.tone_by.as_ref(),
            Some(Body::Primitive(Primitive::Icon(icon))) => icon.tone_by.as_ref(),
            _ => None,
        };
        named.extend(tone_by.and_then(|tone_by| tone_by.tones.clone()));
    }
    for name in document.tone_maps.keys() {
        if !named.contains(name) {
            sink.push(
                "tone_map_unused",
                &NodePath::root().child("tone_maps").child(name),
                format!("no `tone_by` names the tone map `{name}`"),
            );
        }
    }
}
