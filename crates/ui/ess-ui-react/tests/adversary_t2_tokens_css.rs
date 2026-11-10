//! Adversary cases for story:the-react-renderer-draws-the-declared-tokens: what a document can
//! write into `src/runtime/tokens.css`, and whether every `var(--…)` `styles.css` reads is
//! declared for a check-clean document.

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

fn generate(extra: &str) -> BTreeMap<String, String> {
    let text = format!("{BASE}{extra}");
    let document = ess_ui::load_str(&text).unwrap_or_else(|error| panic!("{error}"));
    ess_ui_react::render(&document, Path::new(env!("CARGO_MANIFEST_DIR")))
        .unwrap_or_else(|error| panic!("the document generates: {error}"))
        .files
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

/// The error `generate` gives for a document it refuses.
fn refusal(extra: &str) -> String {
    let text = format!("{BASE}{extra}");
    let document = ess_ui::load_str(&text).unwrap_or_else(|error| panic!("{error}"));
    match ess_ui_react::render(&document, Path::new(env!("CARGO_MANIFEST_DIR"))) {
        Ok(_) => panic!("the generator wrote a token it should refuse"),
        Err(error) => error.to_string(),
    }
}

/// `(name, value)` of every `--name: value;` in a block.
fn custom_properties(css: &str) -> Vec<(String, String)> {
    css.split(['{', '}', ';'])
        .filter_map(|part| part.split_once(':'))
        .map(|(name, value)| (name.trim().to_owned(), value.trim().to_owned()))
        .filter(|(name, _)| name.starts_with("--"))
        .collect()
}

#[test]
fn a_type_family_cannot_write_a_second_declaration() {
    let extra = "tokens:\n  type: {body: {family: 'system-ui; --ui-size-nav-width: 0px'}}\n";
    // Expected outcome adjusted to the refusal: `ess ui check` files the value at its key path,
    // and `generate` (which does not run the check) refuses it, naming the token.
    let findings = token_findings(extra);
    assert!(
        findings
            .iter()
            .any(|finding| finding.contains("tokens/type/body/family")),
        "{findings:?}"
    );
    let error = refusal(extra);
    assert!(error.contains("tokens/type/body/family"), "{error}");
}

#[test]
fn a_token_name_cannot_close_the_root_block() {
    let extra =
        "tokens:\n  color: {'x: #fff; } body { display: none } :root { --ui-color-y': '#fff'}\n";
    // Expected outcome adjusted to the refusal, at both layers.
    let findings = token_findings(extra);
    assert!(
        findings
            .iter()
            .any(|finding| finding.contains("not a token name")),
        "{findings:?}"
    );
    let error = refusal(extra);
    assert!(error.contains("not a token name"), "{error}");
}

// Backlog (filed by the coordinator): `a type style overriding one field keeps the built-in
// others`. A `type` entry replaces the built-in entry whole, so naming only a family drops the
// built-in size and weight; that rule is deliberately unchanged in this story.
#[test]
#[ignore = "backlog: a type style overriding one field keeps the built-in others"]
fn every_property_styles_css_reads_is_declared_when_one_type_field_changes() {
    let extra = "tokens:\n  type: {heading: {family: 'Georgia, serif'}}\n";
    let findings = token_findings(extra);
    assert!(
        findings.is_empty(),
        "the document is check-clean: {findings:?}"
    );
    let files = generate(extra);
    let declared: BTreeSet<String> = custom_properties(&files["src/runtime/tokens.css"])
        .into_iter()
        .map(|(name, _)| name)
        .collect();
    let styles = &files["src/runtime/styles.css"];
    let mut undeclared = BTreeSet::new();
    for (at, _) in styles.match_indices("var(") {
        let rest = &styles[at + 4..];
        let end = rest.find([')', ',']).unwrap_or(rest.len());
        let name = rest[..end].trim();
        let has_fallback = rest[end..].starts_with(',');
        if !has_fallback && !declared.contains(name) {
            undeclared.insert(name.to_owned());
        }
    }
    assert!(
        undeclared.is_empty(),
        "styles.css reads {undeclared:?}, which tokens.css does not declare once \
         `type.heading` names only its family: those declarations compute `unset`"
    );
}

#[test]
fn the_stylesheet_keeps_no_type_or_viewport_literal() {
    let files = generate("");
    let styles = &files["src/runtime/styles.css"];
    let mut literals = Vec::new();
    for rule in styles.split('}') {
        for declaration in rule.split(['{', ';']) {
            let Some((property, value)) = declaration.split_once(':') else {
                continue;
            };
            let (property, value) = (property.trim(), value.trim());
            if property == "font-weight" && !value.starts_with("var(") {
                literals.push(format!("{property}: {value}"));
            }
            for part in value.split_whitespace() {
                let digits = part.trim_end_matches(char::is_alphabetic);
                let unit = &part[digits.len()..];
                if !digits.is_empty()
                    && digits.chars().all(|c| c.is_ascii_digit() || c == '.')
                    && ["vh", "vw", "vmin", "vmax"].contains(&unit)
                {
                    literals.push(format!("{property}: {value}"));
                }
            }
        }
    }
    assert!(
        literals.is_empty(),
        "styles.css keeps type or length literals: {literals:#?}"
    );
}
