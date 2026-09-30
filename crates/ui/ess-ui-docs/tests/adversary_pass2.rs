//! Adversary pass 2: what correction 1 touched (blank texts, the reserved filter id) and what
//! pass 1 did not reach (texts outside constructs, anchors of odd names, front matter, `--check`
//! byte differences, a document given where a schema is expected).

use std::fs;
use std::path::{Path, PathBuf};

use ess_ui_docs::{DocsArgs, DocsError};
use serde_yaml::Value;

/// The same minimal schema `refusals.rs` uses.
const VALID: &str = r"format: ess-ui/1
type_rule:
  summary: Types are YAML structure.
  doc: A type is a primitive, a constructor map or a construct name.
  primitives:
    string: {note: text}
    name: {note: a node name}
  constructors:
    list: {form: {list: T}}
    map: {form: {map: {key: K, value: V}}}
    optional: {form: {optional: T}}
    enum: {form: {enum: [a, b]}}
    one_of: {form: {one_of: [T1, T2]}}
    record: {form: {record: {field: T}}}
    ref: {form: {ref: kind}, kinds: [page, view]}
    const: {form: {const: value}}
groups:
  - {name: document, order: 1, title: The document, summary: The root.}
constructs:
  Document:
    group: document
    order: 1
    summary: The root of a file.
    doc: One document describes one frontend.
    fields:
      pages: {type: {map: {key: name, value: Page}}, required: true, note: every route}
    example: {pages: {home: {title: Home}}}
  Page:
    group: document
    order: 2
    summary: One route.
    doc: A page composes sections.
    fields:
      title: {type: string, note: header title}
      next: {type: {ref: page}, note: the page after this one}
    example: {title: Home}
";

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

fn scratch() -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("ess-ui-docs-adversary-pass2");
    fs::create_dir_all(&dir).expect("create the scratch directory");
    dir
}

/// `VALID` with one more construct appended, named `name` (written as a quoted YAML key).
fn with_construct(name: &str) -> String {
    let key = serde_yaml::to_string(&Value::from(name)).expect("a YAML key");
    format!(
        "{VALID}  {}:\n    group: document\n    order: 3\n    summary: Extra.\n    doc: An extra construct.\n    fields:\n      x: {{type: string, note: a text}}\n    example: {{x: y}}\n",
        key.trim_end()
    )
}

fn both(text: &str) -> (Result<String, DocsError>, Result<String, DocsError>) {
    (
        ess_ui_docs::render_html_str(text),
        ess_ui_docs::render_markdown_str(text),
    )
}

// ── Blank texts correction 1 did not reach ──────────────────────────────────────────────────

#[test]
fn a_blank_string_example_is_refused_like_an_empty_map() {
    let from = "    example: {title: Home}\n";
    assert!(VALID.contains(from));
    for to in ["    example: \"\"\n", "    example: \"   \"\n"] {
        let (html, markdown) = both(&VALID.replacen(from, to, 1));
        assert!(
            html.is_err() && markdown.is_err(),
            "Page with `{}` renders; it documents no example",
            to.trim()
        );
    }
}

#[test]
fn a_blank_format_is_refused_like_a_missing_one() {
    for to in ["format: \"\"", "format: \"   \""] {
        let text = VALID.replacen("format: ess-ui/1", to, 1);
        let (html, markdown) = both(&text);
        assert!(
            html.is_err() && markdown.is_err(),
            "`{to}` renders; the page title is `{}`",
            markdown
                .ok()
                .and_then(|page| page
                    .lines()
                    .find(|l| l.starts_with("# "))
                    .map(str::to_owned))
                .unwrap_or_default()
        );
    }
}

#[test]
fn a_shorthand_without_an_expansion_is_refused_not_shown_as_null() {
    let from = "    example: {title: Home}\n";
    let text = VALID.replacen(
        from,
        "    shorthand:\n      - {at: title, accepts: string}\n    example: {title: Home}\n",
        1,
    );
    let html = ess_ui_docs::render_html_str(&text);
    assert!(
        html.is_err(),
        "a shorthand with no `expands_to` renders; its \"It means\" cell reads: {}",
        html.map(|page| page
            .split("You may also write")
            .nth(1)
            .and_then(|rest| rest.split("</tr>").nth(1))
            .unwrap_or_default()
            .to_owned())
            .unwrap_or_default()
    );
}

#[test]
fn an_absent_foundation_text_never_prints_null() {
    let baseline = ess_ui_docs::render_html_str(VALID).expect("the fixture renders");
    assert!(
        !baseline.contains("<code>null</code>"),
        "the fixture itself shows null"
    );
    let cases = [
        (
            "no type_rule.summary",
            VALID.replacen("  summary: Types are YAML structure.\n", "", 1),
        ),
        (
            "a layer without a note",
            VALID.replacen(
                "groups:\n",
                "layers:\n  - {name: page, contains: [section]}\ngroups:\n",
                1,
            ),
        ),
        (
            "an expression form without a note",
            VALID.replacen(
                "groups:\n",
                "expressions:\n  summary: Bindings.\n  doc: How to bind.\n  forms:\n    - {form: \"state.<name>\"}\ngroups:\n",
                1,
            ),
        ),
    ];
    let mut shown = Vec::new();
    for (case, text) in cases {
        if let Ok(page) = ess_ui_docs::render_html_str(&text) {
            if page.contains("<code>null</code>") {
                shown.push(case);
            }
        }
    }
    assert!(
        shown.is_empty(),
        "the page prints `null` for text the schema never wrote: {shown:?}"
    );
}

#[test]
fn a_type_that_admits_nothing_is_refused() {
    let from = "{type: string, note: header title}";
    let mut rendered = Vec::new();
    for ty in ["{one_of: []}", "{enum: []}", "{record: {}}"] {
        let text = VALID.replacen(from, &format!("{{type: {ty}, note: header title}}"), 1);
        if let Ok(page) = ess_ui_docs::render_markdown_str(&text) {
            let row = page
                .lines()
                .find(|line| line.starts_with("| `title` |"))
                .unwrap_or_default()
                .to_owned();
            rendered.push(format!("{ty}: {row}"));
        }
    }
    assert!(
        rendered.is_empty(),
        "types with no alternative render:\n{}",
        rendered.join("\n")
    );
}

// ── Anchors ────────────────────────────────────────────────────────────────────────────────

#[test]
fn a_name_with_no_slug_is_refused() {
    let mut rendered = Vec::new();
    for name in ["\u{1F642}", "", "   "] {
        if let Ok(page) = ess_ui_docs::render_html_str(&with_construct(name)) {
            let id = page
                .split("<section class=\"construct\" id=\"")
                .nth(3)
                .and_then(|rest| rest.split('"').next())
                .unwrap_or("?")
                .to_owned();
            rendered.push(format!("{name:?} -> id=\"{id}\""));
        }
    }
    let group = VALID.replacen("title: The document,", "title: \"???\",", 1);
    if ess_ui_docs::render_html_str(&group).is_ok() {
        rendered.push("group title \"???\" -> id=\"\"".to_owned());
    }
    assert!(
        rendered.is_empty(),
        "a heading whose anchor is empty renders:\n{}",
        rendered.join("\n")
    );
}

#[test]
fn names_that_collide_after_slugging_are_refused() {
    for name in [
        "Page\u{1F642}",
        "PAGE",
        "page",
        "ess-ui-filter",
        "Ess ui filter",
    ] {
        let (html, markdown) = both(&with_construct(name));
        assert!(
            html.is_err() && markdown.is_err(),
            "{name:?} renders beside an existing anchor"
        );
    }
}

#[test]
fn a_long_unicode_name_links_to_its_own_section() {
    let name = format!("Ünïcødé{}", "Lang".repeat(100));
    let html = ess_ui_docs::render_html_str(&with_construct(&name)).expect("renders");
    let id = format!("ünïcødé{}", "lang".repeat(100));
    assert!(html.contains(&format!("<section class=\"construct\" id=\"{id}\">")));
    assert!(html.contains(&format!("href=\"#{id}\"")));
}

#[test]
fn every_html_id_is_unique_and_the_filter_script_finds_its_input() {
    let schema = fs::read_to_string(repo().join("schemas/ui/ess-ui.schema.yaml")).expect("reads");
    for text in [VALID.to_owned(), schema] {
        let html = ess_ui_docs::render_html_str(&text).expect("renders");
        let ids: Vec<&str> = html
            .split(" id=\"")
            .skip(1)
            .filter_map(|rest| rest.split('"').next())
            .collect();
        let mut seen = std::collections::BTreeSet::new();
        for id in &ids {
            assert!(seen.insert(*id), "id=\"{id}\" appears twice");
        }
        assert_eq!(
            html.matches("id=\"ess-ui-filter\"").count(),
            1,
            "one filter input"
        );
        assert!(html.contains("getElementById('ess-ui-filter')"));
    }
}

/// `CommonMark` reads `_core_` between spaces as emphasis, so the site's heading text is
/// "The core document" and its id `the-core-document`; a link must use the id the site derives.
#[test]
fn a_chapter_title_with_underscores_links_to_the_id_the_site_derives() {
    let text = VALID.replacen("title: The document,", "title: The _core_ document,", 1);
    let page = ess_ui_docs::render_markdown_str(&text).expect("renders");
    let heading = page
        .lines()
        .find(|line| line.starts_with("## ") && line.contains("core"))
        .expect("the chapter heading");
    let site_id = if heading.contains("\\_core\\_") {
        "the-_core_-document"
    } else {
        "the-core-document"
    };
    let linked: Vec<&str> = page
        .split("](#")
        .skip(1)
        .filter_map(|rest| rest.split(')').next())
        .filter(|anchor| anchor.contains("core"))
        .collect();
    assert!(!linked.is_empty(), "the quick reference links the chapter");
    for anchor in linked {
        assert_eq!(
            anchor, site_id,
            "heading `{heading}` gets the site id `{site_id}`; the link says `#{anchor}`"
        );
    }
}

// ── Front matter ───────────────────────────────────────────────────────────────────────────

#[test]
fn front_matter_carries_the_format_verbatim() {
    let mut broken = Vec::new();
    for format in ["ess-ui: 1", "\"quoted\" ui", "ui #1", "ui/1 'x'"] {
        let quoted = serde_yaml::to_string(&Value::from(format)).expect("a YAML scalar");
        let text = VALID.replacen(
            "format: ess-ui/1",
            &format!("format: {}", quoted.trim_end()),
            1,
        );
        let page = ess_ui_docs::render_markdown_str(&text).expect("renders");
        let front = page
            .strip_prefix("---\n")
            .and_then(|rest| rest.split("\n---\n").next())
            .expect("front matter");
        match serde_yaml::from_str::<Value>(front) {
            Ok(meta) => {
                let title = meta["title"].as_str().unwrap_or_default();
                if title != format!("{format} reference") {
                    broken.push(format!("{format:?}: title reads {title:?}"));
                }
            }
            Err(error) => broken.push(format!("{format:?}: front matter is not YAML: {error}")),
        }
    }
    assert!(broken.is_empty(), "{}", broken.join("\n"));
}

// ── --check ────────────────────────────────────────────────────────────────────────────────

#[test]
fn check_accepts_only_the_exact_bytes_and_writes_nothing() {
    let dir = scratch();
    let schema = dir.join("check.schema.yaml");
    fs::write(&schema, VALID).expect("write the schema");
    let fresh = ess_ui_docs::render_markdown_str(VALID).expect("renders");
    let args = |out: &Path| DocsArgs {
        out: out.to_path_buf(),
        format: None,
        schema: Some(schema.clone()),
        check: true,
    };
    let exact = dir.join("exact.md");
    fs::write(&exact, &fresh).expect("write");
    ess_ui_docs::run(&args(&exact)).expect("the exact render is current");
    for (case, bytes) in [
        ("crlf", fresh.replace('\n', "\r\n")),
        (
            "no trailing newline",
            fresh.trim_end_matches('\n').to_owned(),
        ),
        ("extra trailing newline", format!("{fresh}\n")),
    ] {
        let out = dir.join(format!("{}.md", case.replace(' ', "-")));
        fs::write(&out, &bytes).expect("write");
        let error = ess_ui_docs::run(&args(&out)).expect_err(case);
        assert!(
            error.to_string().contains("not a fresh render"),
            "{case}: {error}"
        );
        assert_eq!(
            fs::read_to_string(&out).expect("read"),
            bytes,
            "{case}: --check wrote"
        );
    }
}

// ── A document where a schema belongs ──────────────────────────────────────────────────────

#[test]
fn a_document_given_as_the_schema_is_refused_without_blaming_a_comma() {
    let dir = scratch();
    let out = dir.join("document-as-schema.html");
    let _ = fs::remove_file(&out);
    let document = repo().join("examples/partner-portal/ui.yaml");
    assert!(document.is_file(), "the example document exists");
    for check in [false, true] {
        let error = ess_ui_docs::run(&DocsArgs {
            out: out.clone(),
            format: None,
            schema: Some(document.clone()),
            check,
        })
        .expect_err("a document is not a schema");
        assert!(!out.exists(), "a refused render wrote {}", out.display());
        let misleading: Vec<&String> = error
            .problems()
            .iter()
            .filter(|problem| problem.contains("unquoted comma"))
            .collect();
        assert!(
            misleading.is_empty(),
            "{} of {} problems tell the user to quote text in a file that is not a schema:\n{}",
            misleading.len(),
            error.problems().len(),
            error
        );
    }
}
