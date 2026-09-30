//! The reference rendered from `schemas/ui/ess-ui.schema.yaml`: one self-contained HTML page and
//! the committed Markdown page of the documentation site.

use std::fs;
use std::path::{Path, PathBuf};

use serde_yaml::Value;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

fn schema_path() -> PathBuf {
    root().join("schemas/ui/ess-ui.schema.yaml")
}

fn schema() -> Value {
    serde_yaml::from_str(&fs::read_to_string(schema_path()).expect("the schema reads"))
        .expect("the schema is YAML")
}

fn html() -> String {
    ess_ui_docs::render_html(&schema_path()).expect("the schema renders as HTML")
}

fn markdown() -> String {
    ess_ui_docs::render_markdown(&schema_path()).expect("the schema renders as Markdown")
}

/// The part of the HTML page documenting one construct.
fn section(html: &str, construct: &str) -> String {
    let start = html
        .find(&format!(
            r#"<section class="construct" id="{}""#,
            construct.to_lowercase()
        ))
        .unwrap_or_else(|| panic!("{construct} has a section"));
    let end = html[start..]
        .find("</section>")
        .expect("the section closes")
        + start;
    html[start..end].to_owned()
}

fn construct_names(schema: &Value) -> Vec<String> {
    schema["constructs"]
        .as_mapping()
        .expect("constructs is a map")
        .keys()
        .map(|name| name.as_str().expect("a construct name").to_owned())
        .collect()
}

#[test]
fn the_committed_markdown_is_a_fresh_render() {
    let page = root().join("website/docs/reference/ess-ui.md");
    let committed = fs::read_to_string(&page).unwrap_or_default();
    assert!(
        committed == markdown(),
        "{} is not the render of the schema; regenerate it with \
         `cargo run -p ess-ui-docs --example render -- --format md --out {}`",
        page.display(),
        page.display()
    );
}

#[test]
fn the_page_is_self_contained() {
    let html = html();
    assert!(html.starts_with("<!DOCTYPE html>"));
    assert!(html.contains("<style>") && html.contains("prefers-color-scheme: dark"));
    for external in [
        "<link",
        "src=\"http",
        "src='http",
        "@import",
        "url(http",
        "<script src",
    ] {
        assert!(!html.contains(external), "the page loads {external}");
    }
}

#[test]
fn chapters_follow_the_groups_in_order_and_every_construct_has_a_section() {
    let schema = schema();
    let html = html();
    let mut groups: Vec<(u64, String)> = schema["groups"]
        .as_sequence()
        .expect("groups is a list")
        .iter()
        .map(|group| {
            (
                group["order"].as_u64().expect("an order"),
                group["title"].as_str().expect("a title").to_owned(),
            )
        })
        .collect();
    groups.sort();
    let mut last = 0;
    for (_, title) in &groups {
        let at = html
            .find(&format!("<h2>{title}</h2>"))
            .unwrap_or_else(|| panic!("chapter {title} is rendered"));
        assert!(at > last, "chapter {title} is out of order");
        last = at;
    }
    let quick = html
        .find("<h2>Quick reference</h2>")
        .expect("a quick reference");
    assert!(quick > last, "the quick reference comes after the chapters");
    for name in construct_names(&schema) {
        let body = section(&html, &name);
        assert!(
            body.contains("<table class=\"fields\">"),
            "{name} has a properties table"
        );
        assert!(body.contains("class=\"example\""), "{name} has an example");
        assert!(
            html[quick..].contains(&format!(
                r##"<a href="#{}">{name}</a>"##,
                name.to_lowercase()
            )),
            "{name} is in the quick reference"
        );
    }
}

#[test]
fn types_read_as_words_and_link_to_the_construct_they_name() {
    let html = html();
    let document = section(&html, "Document");
    assert!(
        document.contains("one of: <code>thin</code> | <code>fat</code> | <code>hybrid</code>"),
        "{document}"
    );
    assert!(
        document.contains(r##"map of <code>name</code> → <a href="#shell">Shell</a>"##),
        "{document}"
    );
    let record = section(&html, "record");
    assert!(
        record.contains(r##"list of <a href="#field">Field</a>"##),
        "{record}"
    );
    let page = section(&html, "Page");
    assert!(
        page.contains(r##"name of a <a href="#pagekind">PageKind</a>"##),
        "{page}"
    );
    let reads = section(&html, "Reads");
    assert!(
        reads.contains("name of an ESS <code>view</code>"),
        "{reads}"
    );
    let navigation = section(&html, "Navigation");
    assert!(
        navigation.contains(r##"list of name of a <a href="#page">Page</a>"##),
        "{navigation}"
    );
}

#[test]
fn shorthands_examples_and_checks_are_shown_on_their_construct() {
    let html = html();
    let field = section(&html, "Field");
    assert!(field.contains("You may also write"), "{field}");
    assert!(field.contains("{field: $value, name: $value}"), "{field}");
    let document = section(&html, "Document");
    assert!(
        document.contains(r#"<span class="k">format</span>"#),
        "the example is highlighted: {document}"
    );
    let page = section(&html, "Page");
    assert!(page.contains("page_reachable"), "{page}");
    let action = section(&html, "Action");
    assert!(action.contains("opens_resolves"), "{action}");
}

/// The Markdown outside front matter, fenced blocks and code spans.
fn prose(markdown: &str) -> String {
    let body = markdown
        .strip_prefix("---\n")
        .and_then(|rest| rest.split_once("\n---\n"))
        .map(|(_, body)| body)
        .expect("the page opens with front matter");
    let mut out = String::new();
    let mut fenced = false;
    for line in body.lines() {
        if line.starts_with("```") {
            fenced = !fenced;
            continue;
        }
        if fenced {
            continue;
        }
        let mut in_code = false;
        let mut chars = line.chars().peekable();
        while let Some(c) = chars.next() {
            match c {
                '\\' if !in_code => {
                    chars.next();
                }
                '`' => in_code = !in_code,
                _ if !in_code => out.push(c),
                _ => {}
            }
        }
        out.push('\n');
    }
    out
}

#[test]
fn the_markdown_is_safe_for_the_sites_mdx_parser() {
    let markdown = markdown();
    assert!(markdown.starts_with("---\ntitle: "), "front matter first");
    assert!(!markdown.contains("<!--"), "no HTML comments");
    let prose = prose(&markdown);
    for c in ['{', '}', '<', '>'] {
        assert!(
            !prose.contains(c),
            "an unescaped `{c}` outside code: {}",
            prose
                .lines()
                .find(|line| line.contains(c))
                .unwrap_or_default()
        );
    }
    for name in construct_names(&schema()) {
        assert!(
            markdown.contains(&format!("\n### {name}\n")),
            "{name} has a heading"
        );
    }
}
