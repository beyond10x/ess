//! Adversary pass 1: the reference read against the schema it claims to document.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use serde_yaml::Value;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

fn schema_text() -> String {
    fs::read_to_string(root().join("schemas/ui/ess-ui.schema.yaml")).expect("the schema reads")
}

fn schema() -> Value {
    serde_yaml::from_str(&schema_text()).expect("the schema is YAML")
}

fn html() -> String {
    ess_ui_docs::render_html_str(&schema_text()).expect("the schema renders as HTML")
}

fn markdown() -> String {
    ess_ui_docs::render_markdown_str(&schema_text()).expect("the schema renders as Markdown")
}

fn unescape_html(text: &str) -> String {
    text.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&amp;", "&")
}

/// The visible text of an HTML fragment: tags dropped, entities decoded.
fn html_text(html: &str) -> String {
    let mut out = String::new();
    let mut in_tag = false;
    for c in html.chars() {
        match c {
            '<' => in_tag = true,
            '>' if in_tag => in_tag = false,
            c if !in_tag => out.push(c),
            _ => {}
        }
    }
    unescape_html(&out)
}

/// The visible text of the Markdown page: escapes and backticks dropped.
fn markdown_text(markdown: &str) -> String {
    let mut out = String::new();
    let mut chars = markdown.chars();
    while let Some(c) = chars.next() {
        match c {
            '\\' => {
                if let Some(next) = chars.next() {
                    out.push(next);
                }
            }
            '`' => {}
            c => out.push(c),
        }
    }
    out
}

fn squash(text: &str) -> String {
    text.replace('`', "")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Keys that name the schema's own structure rather than content a reader needs to see.
const STRUCTURAL: &[&str] = &[
    "format",
    "type_rule",
    "summary",
    "doc",
    "primitives",
    "constructors",
    "note",
    "form",
    "forms",
    "example",
    "expressions",
    "unmapped_marker",
    "shorthands",
    "placeholders",
    "operators",
    "index",
    "inheritance",
    "groups",
    "name",
    "order",
    "title",
    "layers",
    "contains",
    "constructs",
    "group",
    "fields",
    "type",
    "required",
    "default",
    "shorthand",
    "at",
    "accepts",
    "expands_to",
    "construct",
    "lowering",
    "lowers_to",
    "traceability",
    "checks",
    "report_by",
    "list",
    "id",
    "subject",
    "severity",
];

/// Every string leaf, and every non-structural key, with the path it sits at.
fn leaves(value: &Value, at: &str, out: &mut Vec<(String, String)>) {
    match value {
        Value::String(s) => out.push((at.to_owned(), s.clone())),
        Value::Mapping(map) => {
            for (key, inner) in map {
                let key = key.as_str().unwrap_or_default();
                let path = format!("{at}.{key}");
                if !STRUCTURAL.contains(&key) {
                    out.push((format!("{path} (key)"), key.to_owned()));
                }
                // A construct's `group` is shown as the chapter it sits in, and a group's
                // `name` only joins the two.
                if (at.starts_with(".constructs.") && key == "group")
                    || (at.starts_with(".groups[") && key == "name")
                {
                    continue;
                }
                leaves(inner, &path, out);
            }
        }
        Value::Sequence(items) => {
            for (i, item) in items.iter().enumerate() {
                leaves(item, &format!("{at}[{i}]"), out);
            }
        }
        _ => {}
    }
}

fn missing_from(page_text: &str) -> Vec<String> {
    let page = squash(page_text);
    let lower = page.to_lowercase();
    let mut out = Vec::new();
    leaves(&schema(), "", &mut out);
    out.into_iter()
        .filter(|(path, text)| {
            let plain = squash(text);
            let quoted = squash(&text.replace('\\', "\\\\").replace('"', "\\\""));
            // A key may be shown as a label: `exactly_one_of` as "Exactly one of".
            let label = path.ends_with("(key)") && lower.contains(&text.replace('_', " "));
            // `absent_placeholder` is shown as the sentence it stands for.
            let sentence = path == ".shorthands.absent_placeholder (key)"
                && page.contains("A placeholder that resolves to nothing");
            !page.contains(&plain) && !page.contains(&quoted) && !label && !sentence
        })
        .map(|(path, text)| format!("{path}: {text:?}"))
        .collect()
}

#[test]
fn every_string_the_schema_declares_is_on_the_html_page() {
    let missing = missing_from(&html_text(&html()));
    assert!(
        missing.is_empty(),
        "{} schema strings are on no part of the HTML page:\n{}",
        missing.len(),
        missing.join("\n")
    );
}

#[test]
fn every_string_the_schema_declares_is_on_the_markdown_page() {
    let missing = missing_from(&markdown_text(&markdown()));
    assert!(
        missing.is_empty(),
        "{} schema strings are on no part of the Markdown page:\n{}",
        missing.len(),
        missing.join("\n")
    );
}

/// The HTML of the construct section with id `id`.
fn section<'h>(html: &'h str, id: &str) -> &'h str {
    let start = html
        .find(&format!(r#"<section class="construct" id="{id}">"#))
        .unwrap_or_else(|| panic!("a section #{id}"));
    let end = html[start..]
        .find("</section>")
        .expect("the section closes")
        + start;
    &html[start..end]
}

#[test]
fn every_indexed_shorthand_is_shown_on_the_construct_it_names() {
    let schema = schema();
    let html = html();
    let mut missing = Vec::new();
    for entry in schema["shorthands"]["index"]
        .as_sequence()
        .expect("shorthands.index is a list")
    {
        let construct = entry["construct"].as_str().expect("a construct");
        let id = construct.to_lowercase();
        let body = squash(&html_text(section(&html, &id)));
        let at = entry["at"].as_str().unwrap_or("the value itself");
        let expands: Value = entry["expands_to"].clone();
        let expands = squash(&serde_yaml::to_string(&expands).expect("YAML"));
        let expands_leaf = expands.trim_end_matches('\n').to_owned();
        let shown = body.contains("You may also write")
            && body.contains(at)
            && expands_leaf
                .split(|c: char| " {}[],:".contains(c))
                .filter(|word| !word.is_empty() && *word != "-")
                .all(|word| body.contains(word));
        if !shown {
            missing.push(format!("{construct} at {at}: {expands_leaf}"));
        }
    }
    assert!(
        missing.is_empty(),
        "shorthands.index entries missing from their construct:\n{}",
        missing.join("\n")
    );
}

fn hrefs(html: &str) -> BTreeSet<String> {
    html.split("href=\"#")
        .skip(1)
        .map(|rest| rest.split('"').next().unwrap_or_default().to_owned())
        .collect()
}

fn ids(html: &str) -> BTreeSet<String> {
    html.split(" id=\"")
        .skip(1)
        .map(|rest| rest.split('"').next().unwrap_or_default().to_owned())
        .collect()
}

#[test]
fn every_html_link_lands_on_an_id_of_the_page() {
    let html = html();
    let ids = ids(&html);
    let dangling: Vec<String> = hrefs(&html).difference(&ids).cloned().collect();
    assert!(dangling.is_empty(), "links to no id: {dangling:?}");
    assert!(hrefs(&html).len() > 60, "the page links");
}

/// The id the documentation site derives from a heading (github-slugger), deduplicated as the
/// site does: a second heading with the same id gets `-1`.
fn site_slug(text: &str) -> String {
    text.to_lowercase()
        .chars()
        .filter_map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' {
                Some(c)
            } else if c == ' ' {
                Some('-')
            } else {
                None
            }
        })
        .collect()
}

#[test]
fn every_markdown_link_lands_on_a_heading_the_site_generates() {
    let markdown = markdown();
    let mut headings = BTreeSet::new();
    let mut fenced = false;
    for line in markdown.lines() {
        if line.starts_with("```") {
            fenced = !fenced;
            continue;
        }
        if fenced {
            continue;
        }
        let hashes = line.chars().take_while(|c| *c == '#').count();
        if (1..=6).contains(&hashes) && line[hashes..].starts_with(' ') {
            let text = markdown_text(line[hashes + 1..].trim());
            let mut slug = site_slug(&text);
            let base = slug.clone();
            let mut n = 1;
            while headings.contains(&slug) {
                slug = format!("{base}-{n}");
                n += 1;
            }
            headings.insert(slug);
        }
    }
    let mut dangling = BTreeSet::new();
    for piece in markdown.split("](#").skip(1) {
        let anchor = piece.split(')').next().unwrap_or_default();
        if !headings.contains(anchor) {
            dangling.insert(anchor.to_owned());
        }
    }
    assert!(dangling.is_empty(), "links to no heading: {dangling:?}");
}

#[test]
fn rendering_twice_gives_identical_bytes() {
    assert_eq!(html(), html());
    assert_eq!(markdown(), markdown());
}

#[test]
fn the_committed_markdown_equals_a_fresh_render_through_run() {
    let page = root().join("website/docs/reference/ess-ui.md");
    let args = ess_ui_docs::DocsArgs {
        out: page.clone(),
        format: None,
        schema: None,
        check: true,
    };
    let line = ess_ui_docs::run(&args).expect("the committed page is current");
    assert!(line.ends_with("is current"), "{line}");
}

/// Every `<pre class="example">` of the page, as the plain text it shows.
fn examples(html: &str) -> Vec<String> {
    html.split("<pre class=\"example\"><code>")
        .skip(1)
        .map(|rest| html_text(rest.split("</code></pre>").next().unwrap_or_default()))
        .collect()
}

#[test]
fn a_highlighted_example_reads_back_as_the_construct_example() {
    let schema = schema();
    let html = html();
    let mut checked = 0;
    for (name, construct) in schema["constructs"].as_mapping().expect("constructs") {
        let name = name.as_str().expect("a name");
        let body = section(&html, &name.to_lowercase());
        let shown = examples(body);
        let last = shown
            .last()
            .unwrap_or_else(|| panic!("{name} shows an example"));
        let back: Value = serde_yaml::from_str(last)
            .unwrap_or_else(|error| panic!("{name}'s example is not YAML ({error}):\n{last}"));
        assert_eq!(back, construct["example"], "{name}'s example as shown");
        checked += 1;
    }
    assert!(checked > 40);
}

#[test]
fn a_markdown_example_reads_back_as_the_construct_example() {
    let schema = schema();
    let markdown = markdown();
    for (name, construct) in schema["constructs"].as_mapping().expect("constructs") {
        let name = name.as_str().expect("a name");
        let start = markdown
            .find(&format!("\n### {name}\n"))
            .unwrap_or_else(|| panic!("{name} has a heading"));
        let rest = &markdown[start..];
        let at = rest.find("**Example**").expect("an example label");
        let block = rest[at..]
            .split("```yaml\n")
            .nth(1)
            .and_then(|b| b.split("```\n").next())
            .expect("a fenced example");
        let back: Value = serde_yaml::from_str(block)
            .unwrap_or_else(|error| panic!("{name}'s example is not YAML ({error}):\n{block}"));
        assert_eq!(back, construct["example"], "{name}'s Markdown example");
    }
}

// ------------------------------------------------------------------------------------------
// Fixtures
// ------------------------------------------------------------------------------------------

const VALID: &str = r#"format: ess-ui/1
type_rule:
  summary: Types are YAML structure.
  doc: A type is a primitive, a constructor map or a construct name.
  primitives:
    string: {note: text}
    name: {note: a node name}
  constructors:
    list: {form: {list: T}, options: {unique: {type: boolean}}}
    map: {form: {map: {key: K, value: V}}}
    optional: {form: {optional: T}}
    enum: {form: {enum: [a, b]}}
    one_of: {form: {one_of: [T1, T2]}}
    record: {form: {record: {field: T}}}
    ref: {form: {ref: kind}, kinds: [page, view]}
    const: {form: {const: value}}
groups:
  - {name: document, order: 1, title: "GROUP_TITLE", summary: "GROUP_SUMMARY"}
constructs:
  Document:
    group: document
    order: 1
    summary: "DOC_SUMMARY"
    doc: "DOC_DOC"
    fields:
      pages: {type: {map: {key: name, value: Page}}, required: true, note: "FIELD_NOTE", default: "FIELD_DEFAULT"}
      deep: {type: {optional: {list: {one_of: [{enum: [a, b]}, {record: {x: {optional: {map: {key: name, value: {list: Page, unique: true}}}}}}]}}}, note: deep}
    shorthand:
      - {at: pages, accepts: string, expands_to: "SHORTHAND_EXPANDS"}
    example: {pages: {home: {title: "EXAMPLE_TEXT"}}}
  Page:
    group: document
    order: 2
    summary: One route.
    doc: A page composes sections.
    fields:
      title: {type: string, note: header title}
      next: {type: {ref: page}, note: the page after this one}
    example: {title: Home}
checks:
  report_by: Page
  list:
    - {id: c1, subject: "pages.*", must: "CHECK_RULE", severity: error}
"#;

const HOSTILE: &str = r#"<script>alert("x")</script> & {y} <!-- z --> a<b"#;

fn hostile() -> String {
    let yaml_hostile = HOSTILE.replace('\\', "\\\\").replace('"', "\\\"");
    let mut text = VALID.to_owned();
    for marker in [
        "GROUP_TITLE",
        "GROUP_SUMMARY",
        "DOC_SUMMARY",
        "DOC_DOC",
        "FIELD_NOTE",
        "FIELD_DEFAULT",
        "SHORTHAND_EXPANDS",
        "EXAMPLE_TEXT",
        "CHECK_RULE",
    ] {
        text = text.replace(marker, &format!("{marker} {yaml_hostile}"));
    }
    text
}

#[test]
fn schema_text_is_escaped_on_the_html_page() {
    let html = ess_ui_docs::render_html_str(&hostile()).expect("the hostile fixture renders");
    let main = html.split("<main>").nth(1).expect("a main");
    assert!(
        !html.contains("<script>alert"),
        "raw script tag reached the page"
    );
    assert!(!main.contains("<!-- z"), "raw comment reached the page");
    assert!(!main.contains(" & "), "a bare ampersand reached the page");
    assert!(!main.contains("a<b"), "a bare `<` reached the page");
    for marker in [
        "GROUP_TITLE",
        "GROUP_SUMMARY",
        "DOC_SUMMARY",
        "DOC_DOC",
        "FIELD_NOTE",
        "FIELD_DEFAULT",
        "SHORTHAND_EXPANDS",
        "EXAMPLE_TEXT",
        "CHECK_RULE",
    ] {
        let text = html_text(main);
        assert!(
            text.contains(&format!("{marker} <script>alert"))
                || text.contains(&format!("{marker} \\u003cscript"))
                || text.contains(&format!("{marker} <script>alert(\\\"x\\\")")),
            "{marker} is not shown with its text"
        );
    }
}

/// Markdown outside the front matter, fenced blocks and code spans.
fn md_prose(markdown: &str) -> String {
    let body = markdown
        .strip_prefix("---\n")
        .and_then(|rest| rest.split_once("\n---\n"))
        .map(|(_, body)| body)
        .expect("front matter");
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
        let mut ticks;
        let mut open = 0usize;
        let mut chars = line.chars().peekable();
        while let Some(c) = chars.next() {
            if c == '`' {
                ticks = 1;
                while chars.peek() == Some(&'`') {
                    chars.next();
                    ticks += 1;
                }
                if open == 0 {
                    open = ticks;
                } else if open == ticks {
                    open = 0;
                }
                continue;
            }
            if open > 0 {
                continue;
            }
            if c == '\\' {
                chars.next();
                continue;
            }
            out.push(c);
        }
        out.push('\n');
    }
    out
}

#[test]
fn schema_text_is_mdx_safe_on_the_markdown_page() {
    let markdown =
        ess_ui_docs::render_markdown_str(&hostile()).expect("the hostile fixture renders");
    let front = markdown
        .strip_prefix("---\n")
        .and_then(|rest| rest.split_once("\n---\n"))
        .map(|(front, _)| front)
        .expect("front matter");
    let front: Value = serde_yaml::from_str(front).expect("the front matter is YAML");
    assert!(front["title"].is_string() && front["description"].is_string());
    let prose = md_prose(&markdown);
    for c in ['{', '}', '<', '>'] {
        let line = prose.lines().find(|line| line.contains(c));
        assert!(line.is_none(), "an unescaped `{c}` outside code: {line:?}");
    }
}

#[test]
fn a_deeply_nested_type_reads_as_words_with_links() {
    let html = ess_ui_docs::render_html_str(VALID).expect("the fixture renders");
    let document = html_text(section(&html, "document"));
    let expected = "optional list of (one of: a | b | record { x: optional map of name → \
                    list of Page, no duplicates })";
    assert!(document.contains(expected), "{document}");
    assert!(html.contains(r##"list of <a href="#page">Page</a>, no duplicates"##));
    let markdown = ess_ui_docs::render_markdown_str(VALID).expect("the fixture renders");
    assert!(
        markdown_text(&markdown.replace("](#page)", "").replace("[Page", "Page"))
            .contains(expected),
        "the Markdown reads the same: {:?}",
        markdown.lines().find(|line| line.starts_with("| `deep`"))
    );
}

// ------------------------------------------------------------------------------------------
// Refusals
// ------------------------------------------------------------------------------------------

fn refused(text: &str) -> String {
    let html = ess_ui_docs::render_html_str(text).expect_err("the schema is refused");
    let md = ess_ui_docs::render_markdown_str(text).expect_err("the schema is refused");
    assert_eq!(html, md);
    html.to_string()
}

#[test]
fn an_empty_summary_doc_or_example_is_refused_like_a_missing_one() {
    let mut rendered = Vec::new();
    for (key, from, to) in [
        ("summary", "summary: One route.", "summary: \"\""),
        ("summary", "summary: One route.", "summary: \"   \""),
        ("doc", "doc: A page composes sections.", "doc: \"\""),
        ("example", "example: {title: Home}", "example: {}"),
    ] {
        let fixture = VALID.replacen(from, to, 1);
        assert_ne!(fixture, VALID);
        match ess_ui_docs::render_html_str(&fixture) {
            Ok(_) => rendered.push(format!("Page with `{to}` renders; it documents no {key}")),
            Err(error) => assert!(
                error.to_string().contains("Page") && error.to_string().contains(key),
                "{error}"
            ),
        }
    }
    assert!(rendered.is_empty(), "{}", rendered.join("\n"));
}

#[test]
fn a_dangling_type_inside_a_shorthand_record_is_refused() {
    let message = refused(&VALID.replace(
        "{at: pages, accepts: string,",
        "{at: pages, accepts: {record: {to: {list: Screen}}},",
    ));
    assert!(
        message.contains("Document shorthand") && message.contains("`Screen`"),
        "{message}"
    );
}

#[test]
fn a_key_with_no_value_is_refused_naming_its_path() {
    let message = refused(&VALID.replace("note: the page after this one}", "note: }"));
    assert!(
        message.contains("constructs.Page.fields.next") && message.contains("`note`"),
        "{message}"
    );
    let message = refused(&VALID.replace("example: {title: Home}", "example:"));
    assert!(
        message.contains("Page") && message.contains("example"),
        "{message}"
    );
}

#[test]
fn a_construct_named_filter_does_not_share_its_id_with_the_sidebar_input() {
    let fixture = VALID.replace("Page", "Filter");
    match ess_ui_docs::render_html_str(&fixture) {
        Err(error) => assert!(error.to_string().contains("filter"), "{error}"),
        Ok(html) => {
            let count = html.matches(" id=\"filter\"").count();
            assert_eq!(count, 1, "the page carries id=\"filter\" {count} times");
        }
    }
}
