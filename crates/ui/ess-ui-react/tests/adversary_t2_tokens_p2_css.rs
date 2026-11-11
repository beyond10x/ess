//! Adversary cases, pass 2, for story:the-react-renderer-draws-the-declared-tokens: the correction
//! (163d934990) refuses `; { } < > \` and comments in a token value, but an open string or an open
//! bracket also carries a declaration past its own `;`: the CSS tokenizer reads to the end of the
//! line inside a string (a bad-string token) and to the matching `)` or `]`, or the end of the file,
//! inside a bracket. Either way the next declaration (or every later one, the closing `}` of
//! `:root` included) becomes part of the token's value and is dropped with it.

use std::collections::BTreeMap;
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

/// The files `generate` writes, or its error.
fn generated(extra: &str) -> Result<BTreeMap<String, String>, String> {
    let text = format!("{BASE}{extra}");
    let document = ess_ui::load_str(&text).unwrap_or_else(|error| panic!("{error}"));
    ess_ui_react::render(&document, Path::new(env!("CARGO_MANIFEST_DIR")))
        .map(|rendered| rendered.files)
        .map_err(|error| error.to_string())
}

/// The findings `ess ui check` files under `tokens`, as `check path: message`.
fn token_findings(extra: &str) -> Vec<String> {
    let text = format!("{BASE}{extra}");
    let report = ess_ui_check::check_source(
        &text,
        "t.ui.yaml",
        Path::new(env!("CARGO_MANIFEST_DIR")),
        None,
        &ess_ui_check::Options::default(),
    );
    report
        .findings
        .iter()
        .filter(|finding| finding.path.contains("tokens") || finding.check.starts_with("token"))
        .map(|finding| format!("{} {}: {}", finding.check, finding.path, finding.message))
        .collect()
}

/// Every line of a `:root` block whose declaration leaves a string or a bracket open at its end,
/// so that the CSS tokenizer carries it past the `;` into the following declarations.
fn open_declarations(css: &str) -> Vec<String> {
    let mut open = Vec::new();
    for line in css.lines() {
        let mut quote: Option<char> = None;
        let mut depth: i32 = 0;
        for c in line.chars() {
            match (quote, c) {
                (Some(q), c) if c == q => quote = None,
                (None, '"' | '\'') => quote = Some(c),
                (None, '(' | '[') => depth += 1,
                (None, ')' | ']') => depth -= 1,
                _ => {}
            }
        }
        if quote.is_some() || depth != 0 {
            open.push(line.to_owned());
        }
    }
    open
}

/// `generate` either refuses the document or writes a `tokens.css` whose every declaration closes
/// on its own line.
fn assert_generate_contains(extra: &str) {
    match generated(extra) {
        Err(_) => {}
        Ok(files) => {
            let css = &files["src/runtime/tokens.css"];
            let open = open_declarations(css);
            assert!(
                open.is_empty(),
                "generate wrote a declaration that runs past its `;` into the next ones: \
                 {open:#?}\n--- tokens.css ---\n{css}"
            );
        }
    }
}

const OPEN_QUOTE: &str = "tokens:\n  type: {body: {family: '\"Segoe UI, system-ui'}}\n";
const OPEN_PAREN: &str = "tokens:\n  type: {mono: {family: 'local(Menlo, monospace'}}\n";

#[test]
fn check_refuses_a_family_with_an_open_quote() {
    let findings = token_findings(OPEN_QUOTE);
    assert!(
        findings
            .iter()
            .any(|finding| finding.contains("tokens/type/body/family")),
        "`ess ui check` passes a family whose open string swallows the next declaration: \
         {findings:#?}"
    );
}

#[test]
fn generate_refuses_a_family_with_an_open_quote() {
    assert_generate_contains(OPEN_QUOTE);
}

#[test]
fn check_refuses_a_family_with_an_open_paren() {
    let findings = token_findings(OPEN_PAREN);
    assert!(
        findings
            .iter()
            .any(|finding| finding.contains("tokens/type/mono/family")),
        "`ess ui check` passes a family whose open bracket swallows every later declaration and \
         the closing brace of :root: {findings:#?}"
    );
}

#[test]
fn generate_refuses_a_family_with_an_open_paren() {
    assert_generate_contains(OPEN_PAREN);
}

/// `ess ui check` refuses this color by its grammar, but `generate` does not run the check and
/// applies only its own character refusal.
#[test]
fn generate_refuses_a_color_with_an_open_paren() {
    assert_generate_contains("tokens:\n  color: {backdrop: 'rgb(0 0 0 / 0.3'}\n");
}

/// Pins the other side of a fix: a quoted family name is an ordinary CSS font family, and both
/// layers must keep accepting it.
#[test]
fn a_quoted_family_stays_legal() {
    let extra = "tokens:\n  type: {body: {family: '\"Segoe UI\", system-ui'}}\n";
    let findings = token_findings(extra);
    assert!(
        !findings
            .iter()
            .any(|finding| finding.contains("tokens/type/body/family")),
        "{findings:#?}"
    );
    let files = generated(extra).unwrap_or_else(|error| panic!("{error}"));
    assert!(files["src/runtime/tokens.css"]
        .contains("  --ui-type-body-family: \"Segoe UI\", system-ui;\n"));
}

/// The acceptance: `styles.css.tmpl` keeps no color, length or type literal. The scanner in
/// tokens.rs looks for px/rem/em, vh/vw, `%` on spacing properties, font-weight and the values a
/// token holds; these declarations are none of those and still literals.
#[test]
fn the_stylesheet_keeps_no_literal_the_scanner_misses() {
    let files = generated("").unwrap_or_else(|error| panic!("{error}"));
    let css = &files["src/runtime/styles.css"];
    let mut literals = Vec::new();
    for rule in css.split('}') {
        let Some((selector, body)) = rule.split_once('{') else {
            continue;
        };
        for declaration in body.split(';') {
            let Some((property, value)) = declaration.split_once(':') else {
                continue;
            };
            let (property, value) = (property.trim(), value.trim());
            let length = matches!(
                property,
                "stroke-width" | "stroke-dasharray" | "line-height" | "border-radius"
            ) && !value.starts_with("var(");
            let color = value
                .split([' ', ',', '(', ')'])
                .any(|part| part == "transparent");
            if length || color {
                literals.push(format!("{} {{ {property}: {value} }}", selector.trim()));
            }
        }
    }
    assert!(literals.is_empty(), "{literals:#?}");
}
