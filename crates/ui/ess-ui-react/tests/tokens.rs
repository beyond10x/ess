//! The React renderer draws the declared tokens
//! (story:the-react-renderer-draws-the-declared-tokens, `docs/design/ui-style-tokens.md`
//! § Renderers): the generator writes `src/runtime/tokens.css` from the document's merged token
//! table, `styles.css` reads `var(--…)` for every value a token holds, and a document without
//! `tokens:` computes the values of the stylesheet that shipped before tokens (the pinned copy
//! `tests/fixtures/styles-0.92.css`).

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

const BASE: &str = r"format: ess-ui/1
app: t
model: t.system
placement_profile: fat
shells:
  app: {regions: {main: {kind: page_outlet}}}
navigation: {home: runs, sections: [{name: all, pages: [runs]}]}
pages:
  runs:
    kind: list_page
    title: Runs
    sections:
      - name: runs
        component: collection
        reads: t.Runs
        columns: [{field: title}]
";

const OLD_STYLES: &str = include_str!("fixtures/styles-0.92.css");

fn generate(extra: &str) -> BTreeMap<String, String> {
    let text = format!("{BASE}{extra}");
    let document = ess_ui::load_str(&text).unwrap_or_else(|error| panic!("{error}"));
    ess_ui_react::render(&document, Path::new(env!("CARGO_MANIFEST_DIR")))
        .unwrap_or_else(|error| panic!("the document generates: {error}"))
        .files
}

fn file<'a>(files: &'a BTreeMap<String, String>, path: &str) -> &'a str {
    files
        .get(path)
        .unwrap_or_else(|| panic!("the project has no {path}: {:?}", files.keys()))
}

/// The `--name: value;` declarations of a stylesheet, in order.
fn custom_properties(css: &str) -> Vec<(String, String)> {
    declarations(css)
        .into_iter()
        .filter(|(_, property, _)| property.starts_with("--"))
        .map(|(_, property, value)| (property, value))
        .collect()
}

/// Every declaration of a stylesheet as (selector, property, value), one per selector of a
/// comma list; a nested rule's selector is its parents' joined by a space.
fn declarations(css: &str) -> Vec<(String, String, String)> {
    let mut found = Vec::new();
    let mut stack: Vec<String> = Vec::new();
    let mut buffer = String::new();
    let mut flush = |buffer: &mut String, stack: &[String]| {
        if let Some((property, value)) = buffer.split_once(':') {
            let selector = stack.join(" ");
            for part in selector.split(',') {
                found.push((
                    part.trim().to_owned(),
                    property.trim().to_owned(),
                    value.trim().to_owned(),
                ));
            }
        }
        buffer.clear();
    };
    for character in css.chars() {
        match character {
            '{' => {
                stack.push(buffer.trim().to_owned());
                buffer.clear();
            }
            ';' => flush(&mut buffer, &stack),
            '}' => {
                flush(&mut buffer, &stack);
                stack.pop();
            }
            other => buffer.push(other),
        }
    }
    found
}

/// A value with every `var(--name)` replaced by what `table` holds, to a fixed point.
fn resolve(value: &str, table: &BTreeMap<String, String>) -> String {
    let mut value = value.to_owned();
    while let Some(start) = value.find("var(") {
        let rest = &value[start + 4..];
        let mut depth = 1;
        let mut end = rest.len();
        for (at, character) in rest.char_indices() {
            match character {
                '(' => depth += 1,
                ')' => {
                    depth -= 1;
                    if depth == 0 {
                        end = at;
                        break;
                    }
                }
                _ => {}
            }
        }
        let name = rest[..end].split(',').next().unwrap_or("").trim();
        let held = table
            .get(name)
            .unwrap_or_else(|| panic!("`{name}` is held by no custom property"));
        value = format!(
            "{}{}{}",
            &value[..start],
            held,
            &rest[(end + 1).min(rest.len())..]
        );
    }
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn table(properties: Vec<(String, String)>) -> BTreeMap<String, String> {
    properties.into_iter().collect()
}

#[test]
fn the_project_writes_tokens_css_and_imports_it() {
    let files = generate("");
    let tokens = file(&files, "src/runtime/tokens.css");
    assert_eq!(tokens.matches(":root {").count(), 1, "{tokens}");
    for line in [
        "--ui-color-accent: #2f5bd3;",
        "--ui-color-on_accent: #fff;",
        "--ui-space-md: 1rem;",
        "--ui-radius-pill: 999px;",
        "--ui-type-heading-size: 1.05rem;",
        "--ui-type-heading-weight: 600;",
        "--ui-type-mono-family: ui-monospace, monospace;",
        "--ui-tone-danger-text: var(--ui-color-danger);",
        "--ui-tone-danger-fill: var(--ui-color-danger_fill);",
        "--ui-tone-neutral-fill: var(--ui-color-line);",
    ] {
        assert!(
            tokens.contains(&format!("  {line}\n")),
            "no `{line}` in\n{tokens}"
        );
    }
    let main = file(&files, "src/main.tsx");
    let tokens_at = main
        .find("import \"./runtime/tokens.css\";")
        .expect("main imports tokens.css");
    let styles_at = main
        .find("import \"./runtime/styles.css\";")
        .expect("main imports styles.css");
    assert!(tokens_at < styles_at, "tokens.css loads before styles.css");
}

#[test]
fn every_token_of_the_merged_table_is_a_custom_property() {
    let document = ess_ui::load_str(BASE).expect("loads");
    let merged = document.base_tokens();
    let files = generate("");
    let mut written = table(custom_properties(file(&files, "src/runtime/tokens.css")));
    // The renderer-fixed layout lengths (`--ui-size-*`) are no document token: no group holds
    // them and a document cannot change them.
    let sizes: Vec<String> = written
        .keys()
        .filter(|name| name.starts_with("--ui-size-"))
        .cloned()
        .collect();
    assert!(sizes.len() > 10, "{sizes:?}");
    for name in &sizes {
        written.remove(name);
    }
    let with_tokens = generate("tokens:\n  space: {md: 2rem}\n");
    let after = table(custom_properties(file(
        &with_tokens,
        "src/runtime/tokens.css",
    )));
    for name in &sizes {
        assert_eq!(
            after[name],
            table(custom_properties(file(&files, "src/runtime/tokens.css")))[name]
        );
    }
    let mut expected = BTreeSet::new();
    for name in merged.color.keys() {
        expected.insert(format!("--ui-color-{name}"));
    }
    for name in merged.space.keys() {
        expected.insert(format!("--ui-space-{name}"));
    }
    for name in merged.radius.keys() {
        expected.insert(format!("--ui-radius-{name}"));
    }
    for (style, entry) in &merged.typography {
        for (part, held) in [
            ("family", entry.family.is_some()),
            ("size", entry.size.is_some()),
            ("weight", entry.weight.is_some()),
        ] {
            if held {
                expected.insert(format!("--ui-type-{style}-{part}"));
            }
        }
    }
    for tone in merged.tone.keys() {
        expected.insert(format!("--ui-tone-{tone}-text"));
        expected.insert(format!("--ui-tone-{tone}-fill"));
    }
    assert_eq!(written.keys().cloned().collect::<BTreeSet<_>>(), expected);
}

#[test]
fn changing_one_token_changes_exactly_that_custom_property() {
    let before = generate("");
    let after =
        generate("tokens:\n  color: {accent: '#123456', brand: '#abcdef'}\n  space: {md: 2rem}\n");
    let one = generate("tokens:\n  color: {accent: '#123456'}\n");

    let was = custom_properties(file(&before, "src/runtime/tokens.css"));
    let now = custom_properties(file(&one, "src/runtime/tokens.css"));
    assert_eq!(was.len(), now.len());
    let changed: Vec<_> = was.iter().zip(&now).filter(|(a, b)| a != b).collect();
    assert_eq!(changed.len(), 1, "{changed:?}");
    assert_eq!(changed[0].0 .0, "--ui-color-accent");
    assert_eq!(changed[0].1 .1, "#123456");

    let many = table(custom_properties(file(&after, "src/runtime/tokens.css")));
    assert_eq!(many["--ui-color-accent"], "#123456");
    assert_eq!(many["--ui-color-brand"], "#abcdef");
    assert_eq!(many["--ui-space-md"], "2rem");

    for files in [&after, &one] {
        assert_eq!(
            file(files, "src/runtime/styles.css"),
            file(&before, "src/runtime/styles.css"),
            "the stylesheet reads the tokens; it does not carry them"
        );
    }
}

#[test]
fn the_stylesheet_keeps_no_value_a_token_holds() {
    let files = generate("");
    let css = file(&files, "src/runtime/styles.css");
    assert!(!css.contains("rgb("), "a color literal");
    assert!(
        !css.contains("system-ui") && !css.contains("monospace"),
        "a font literal"
    );
    let document = ess_ui::load_str(BASE).expect("loads");
    let merged = document.base_tokens();
    let mut held: BTreeSet<String> = BTreeSet::new();
    held.extend(merged.space.values().map(|value| value.0.clone()));
    held.extend(merged.radius.values().map(|value| value.0.clone()));
    for entry in merged.typography.values() {
        held.extend(entry.size.iter().map(|value| value.0.clone()));
        held.extend(entry.weight.iter().map(|value| value.0.clone()));
    }
    for (selector, property, value) in declarations(css) {
        assert!(
            !value.split_whitespace().any(|part| part.starts_with('#')),
            "{selector} {{ {property}: {value} }} is a color literal"
        );
        let bytes = value.as_bytes();
        for at in 0..bytes.len() {
            let unit = [b"px".as_slice(), b"rem", b"em"]
                .iter()
                .any(|unit| bytes[at..].starts_with(unit));
            let after = at
                + if bytes[at..].starts_with(b"rem") {
                    3
                } else {
                    2
                };
            let ends = after >= bytes.len() || !bytes[after].is_ascii_alphanumeric();
            assert!(
                !(unit
                    && at > 0
                    && (bytes[at - 1].is_ascii_digit() || bytes[at - 1] == b'.')
                    && ends),
                "{selector} {{ {property}: {value} }} is a length literal"
            );
        }
        // A font weight is a token or a renderer constant, never a keyword or a number.
        assert!(
            property != "font-weight" || value.starts_with("var("),
            "{selector} {{ {property}: {value} }} is a weight literal"
        );
        // Viewport lengths are renderer constants. A percentage is exempt where it is a share of
        // the parent (a width, a height, a gradient stop), not a space the document could set.
        let viewport = ["vh", "vw", "vmin", "vmax"].iter().any(|unit| {
            value
                .split(|c: char| !c.is_ascii_alphanumeric() && c != '.')
                .any(|part| {
                    part.strip_suffix(unit).is_some_and(|digits| {
                        !digits.is_empty() && digits.chars().all(|c| c.is_ascii_digit() || c == '.')
                    })
                })
        });
        assert!(
            !viewport,
            "{selector} {{ {property}: {value} }} is a viewport length literal"
        );
        let spacing = [
            "padding",
            "margin",
            "gap",
            "font-size",
            "top",
            "right",
            "bottom",
            "left",
            "inset",
            "min-height",
            "min-width",
            "max-height",
            "line-height",
        ];
        assert!(
            !(spacing
                .iter()
                .any(|name| property == *name || property.starts_with(&format!("{name}-")))
                && value.contains('%')),
            "{selector} {{ {property}: {value} }} is a percentage length literal"
        );
        if property == "z-index" {
            continue;
        }
        for part in value.split_whitespace() {
            assert!(
                !held.contains(part),
                "{selector} {{ {property}: {value} }} writes `{part}`, which a token holds"
            );
        }
    }
}

#[test]
fn a_document_without_tokens_computes_the_values_of_the_old_stylesheet() {
    let files = generate("");
    let new = table(custom_properties(file(&files, "src/runtime/tokens.css")));
    let old = table(custom_properties(OLD_STYLES));
    let drawn: BTreeSet<(String, String, String)> =
        declarations(file(&files, "src/runtime/styles.css"))
            .into_iter()
            .filter(|(_, property, _)| !property.starts_with("--"))
            .map(|(selector, property, value)| (selector, property, resolve(&value, &new)))
            .collect();
    let mut missing = Vec::new();
    let mut compared = 0;
    for (selector, property, value) in declarations(OLD_STYLES) {
        if property.starts_with("--") {
            continue;
        }
        // Button emphasis moved to its own class prefix.
        let selector = match selector.as_str() {
            ".ui-tone-primary" => ".ui-emphasis-primary".to_owned(),
            ".ui-tone-danger" => ".ui-emphasis-danger".to_owned(),
            ".ui-tone-ghost" => ".ui-emphasis-ghost".to_owned(),
            _ => selector,
        };
        // The one color no token holds, black at 0.2 for the table's scroll shadow, is now the
        // backdrop token (black at 0.3) mixed down to 2/3: the same pixels, not the same text.
        if value.contains("rgb(0 0 0 / 0.2)") {
            assert!(
                drawn.iter().any(|(s, p, v)| s == &selector
                    && p == &property
                    && v.contains("color-mix(in srgb, rgb(0 0 0 / 0.3) 66.667%, transparent)")),
                "{selector} {property}: the shadow is not the backdrop token mixed"
            );
            continue;
        }
        compared += 1;
        let entry = (selector, property, resolve(&value, &old));
        if !drawn.contains(&entry) {
            missing.push(entry);
        }
    }
    assert!(compared > 250, "{compared} declarations compared");
    assert!(
        missing.is_empty(),
        "{} declarations differ:\n{missing:#?}",
        missing.len()
    );
}
