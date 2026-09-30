//! A schema the reference cannot document completely is refused, naming what is missing.

use std::fs;
use std::path::PathBuf;

/// A minimal schema: two constructs, one linking to the other.
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

fn write(name: &str, text: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("ess-ui-docs-refusals");
    fs::create_dir_all(&dir).expect("create the fixture directory");
    let path = dir.join(format!("{name}.schema.yaml"));
    fs::write(&path, text).expect("write the fixture");
    path
}

fn refusal(name: &str, text: &str) -> String {
    let path = write(name, text);
    let html = ess_ui_docs::render_html(&path).expect_err("the schema is refused as HTML");
    let markdown =
        ess_ui_docs::render_markdown(&path).expect_err("the schema is refused as Markdown");
    assert_eq!(html, markdown, "both formats refuse for the same reasons");
    html.to_string()
}

#[test]
fn the_valid_fixture_renders_in_both_formats() {
    let path = write("valid", VALID);
    let html = ess_ui_docs::render_html(&path).expect("the fixture renders as HTML");
    assert!(html.contains(r##"<a href="#page">Page</a>"##), "{html}");
    let markdown = ess_ui_docs::render_markdown(&path).expect("the fixture renders as Markdown");
    assert!(markdown.contains("[Page](#page)"), "{markdown}");
}

#[test]
fn a_construct_without_a_summary_doc_or_example_is_refused() {
    for (key, line) in [
        ("summary", "    summary: One route.\n"),
        ("doc", "    doc: A page composes sections.\n"),
        ("example", "    example: {title: Home}\n"),
    ] {
        assert!(VALID.contains(line), "the fixture carries {key}");
        let message = refusal(&format!("no-{key}"), &VALID.replacen(line, "", 1));
        assert!(
            message.contains(&format!("Page has no `{key}`")),
            "missing {key}: {message}"
        );
    }
}

#[test]
fn a_type_naming_a_construct_that_does_not_exist_is_refused() {
    let message = refusal(
        "unknown-construct",
        &VALID.replace(
            "{type: string, note: header title}",
            "{type: {list: {optional: Header}}, note: header title}",
        ),
    );
    assert!(
        message.contains("Page.title") && message.contains("`Header`"),
        "{message}"
    );
}

#[test]
fn a_lowercase_name_that_is_neither_a_primitive_nor_a_construct_is_refused() {
    let message = refusal(
        "unknown-primitive",
        &VALID.replace(
            "{type: string, note: header title}",
            "{type: text, note: x}",
        ),
    );
    assert!(
        message.contains("Page.title") && message.contains("`text`"),
        "{message}"
    );
}

#[test]
fn a_ref_to_a_kind_the_type_rule_does_not_declare_is_refused() {
    let message = refusal(
        "unknown-kind",
        &VALID.replace("{ref: page}", "{ref: screen}"),
    );
    assert!(
        message.contains("Page.next") && message.contains("`screen`"),
        "{message}"
    );
}

#[test]
fn a_constructor_the_type_rule_does_not_declare_is_refused() {
    let message = refusal(
        "unknown-constructor",
        &VALID.replace(
            "{type: string, note: header title}",
            "{type: {set: string}, note: x}",
        ),
    );
    assert!(
        message.contains("Page.title") && message.contains("`set`"),
        "{message}"
    );
}

#[test]
fn text_split_by_an_unquoted_comma_in_a_flow_mapping_is_refused() {
    let message = refusal(
        "split-note",
        &VALID.replace("note: header title}", "note: header title, shown on top}"),
    );
    assert!(
        message.contains("constructs.Page.fields.title") && message.contains("`shown on top`"),
        "{message}"
    );
    let message = refusal(
        "split-title",
        &VALID.replace("title: The document,", "title: The document, and more,"),
    );
    assert!(
        message.contains("groups[0]") && message.contains("`and more`"),
        "{message}"
    );
}

#[test]
fn an_explicit_null_constant_is_not_a_split() {
    let path = write(
        "const-null",
        &VALID.replace(
            "{type: string, note: header title}",
            "{type: {const: null}, note: x}",
        ),
    );
    ess_ui_docs::render_html(&path).expect("`{const: null}` is a value, not a split");
}

#[test]
fn every_required_text_refuses_blank_as_it_refuses_missing() {
    for (case, from, to, expected) in [
        (
            "blank-summary",
            "summary: One route.",
            "summary: \"  \"",
            "Page has no `summary`",
        ),
        (
            "blank-doc",
            "doc: A page composes sections.",
            "doc: \"\"",
            "Page has no `doc`",
        ),
        (
            "empty-example-map",
            "example: {title: Home}",
            "example: {}",
            "Page has no `example`",
        ),
        (
            "empty-example-list",
            "example: {title: Home}",
            "example: []",
            "Page has no `example`",
        ),
        (
            "blank-note",
            "note: the page after this one}",
            "note: \" \"}",
            "Page.next has no `note`",
        ),
        (
            "blank-group-title",
            "title: The document,",
            "title: \"\",",
            "the group",
        ),
        (
            "blank-group-summary",
            "summary: The root.}",
            "summary: \" \"}",
            "the group",
        ),
        (
            "blank-group",
            "    group: document\n    order: 2",
            "    group: \"\"\n    order: 2",
            "Page names no declared group",
        ),
    ] {
        assert!(VALID.contains(from), "{case}: the fixture carries `{from}`");
        let message = refusal(case, &VALID.replacen(from, to, 1));
        assert!(message.contains(expected), "{case}: {message}");
    }
}

#[test]
fn every_text_outside_constructs_is_required_where_its_section_is_present() {
    for (case, section, expected) in [
        (
            "check-severity",
            "checks:\n  list:\n    - {id: c1, subject: \"pages.*\", must: x}\n",
            "checks.list[0].severity has no text",
        ),
        (
            "check-subject",
            "checks:\n  list:\n    - {id: c1, must: x, severity: error}\n",
            "checks.list[0].subject has no text",
        ),
        (
            "lowering",
            "lowering:\n  - {construct: reads}\n",
            "lowering[0].lowers_to has no text",
        ),
        (
            "unmapped-marker",
            "unmapped_marker:\n  pattern: x\n",
            "unmapped_marker.summary has no text",
        ),
        (
            "shorthands",
            "shorthands:\n  placeholders: {$value: the value}\n",
            "shorthands.summary has no text",
        ),
    ] {
        let message = refusal(case, &format!("{VALID}{section}"));
        assert!(message.contains(expected), "{case}: {message}");
    }
}

#[test]
fn a_file_that_is_not_a_schema_is_refused_by_its_shape_first() {
    let message = refusal(
        "not-a-schema",
        "format: ess-ui/1\napp: portal\npages: {a: {b, c}}\n",
    );
    assert_eq!(
        message,
        "this is not an ess-ui schema: missing `type_rule`, `groups`, `constructs`"
    );
}

#[test]
fn a_construct_in_an_undeclared_group_is_refused() {
    let message = refusal(
        "unknown-group",
        &VALID.replacen(
            "    group: document\n    order: 2",
            "    group: pages\n    order: 2",
            1,
        ),
    );
    assert!(
        message.contains("Page names no declared group"),
        "{message}"
    );
}
