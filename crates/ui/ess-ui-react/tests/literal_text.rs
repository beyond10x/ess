//! A literal in an `expr` position renders as text (#353). The reference's rule, which
//! `ess-ui-check` applies: a string is an expression only when it reads a path or a function
//! form; anything else is literal text, and a lone `true`, `false`, `null`, number or quoted
//! string keeps its value.
//!
//! Each case takes its expression from the loaded document, after widget arguments are
//! substituted, and runs it through the generated runtime's `evaluate` under `node`.

mod support;

use std::path::{Path, PathBuf};

use ess_ui::{Body, Document, NodeRef, Primitive};
use serde_json::{json, Value};
use support::{compile, node};

const DOCUMENT: &str = r#"
format: ess-ui/1
app: t
model: t.system
placement_profile: fat
shells:
  app: {regions: {main: {kind: page_outlet}}}
navigation:
  home: p
  sections: [{name: all, pages: [p]}]
widgets:
  caption:
    summary: A caption.
    params: {label: {type: string, required: true, note: the text}}
    body: [{name: t, primitive: text, text: args.label}]
  status_badge:
    summary: A status as a badge.
    params: {status: {type: string, required: true, note: the status}}
    body: [{name: badge, primitive: badge, text: args.status}]
pages:
  p:
    kind: detail_page
    title: P
    sections:
      - name: summary
        component: record
        reads: t.ById
        item:
          - {name: limit, component: caption, args: {label: Limit in cents}}
          - {name: cost, component: caption, args: {label: Cost (cents)}}
          - {name: terms, component: caption, args: {label: Terms and conditions}}
          - {name: quoted, component: caption, args: {label: '"Quoted (text)"'}}
          - {name: count, component: caption, args: {label: 42}}
          - {name: stage, component: status_badge, args: {status: row.stage}}
          - {name: hidden, primitive: text, text: Never shown, visible: "false"}
"#;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

fn scratch(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("ess-ui-react-literal-text")
        .join(name);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("the old scratch project is removed");
    }
    dir
}

/// The expression a primitive at `path` holds: its `text`, or its `visible` with `visible: true`.
fn expression(document: &Document, path: &str, visible: bool) -> String {
    let located = document
        .nodes()
        .into_iter()
        .find(|located| located.path.to_string() == path)
        .unwrap_or_else(|| panic!("no node at {path}"));
    let NodeRef::Node(node) = located.node else {
        panic!("{path} is not a node");
    };
    let expr = if visible {
        node.common.visible.as_ref()
    } else {
        match &node.body {
            Body::Primitive(Primitive::Text(text)) => text.text.as_ref(),
            Body::Primitive(Primitive::Badge(badge)) => badge.text.as_ref(),
            other => panic!("{path} is not a text or badge: {other:?}"),
        }
    };
    expr.unwrap_or_else(|| panic!("{path} has no expression"))
        .0
        .clone()
}

#[test]
fn a_literal_reads_as_text_and_an_expression_still_evaluates() {
    let document = ess_ui::load_str(DOCUMENT).unwrap_or_else(|error| panic!("{error}"));
    let item = "pages/p/sections/summary/item";
    let text = |name: &str, body: &str| {
        expression(&document, &format!("{item}/{name}/body/{body}"), false)
    };
    let row = json!({"stage": "won"});
    let cases: Vec<Value> = vec![
        json!([text("limit", "t"), row, "Limit in cents"]),
        json!([text("cost", "t"), row, "Cost (cents)"]),
        json!([text("terms", "t"), row, "Terms and conditions"]),
        json!([text("quoted", "t"), row, "Quoted (text)"]),
        json!([text("count", "t"), row, 42]),
        json!([text("stage", "badge"), row, "won"]),
        json!([
            expression(&document, &format!("{item}/hidden"), true),
            row,
            false
        ]),
        json!(["Open", row, "Open"]),
        json!(["true", row, true]),
        json!(["null", row, null]),
        json!(["row.stage == won", row, true]),
        json!(["row.stage != won", row, false]),
        json!(["row.stage in [lead, won]", row, true]),
        json!(["not row.stage", row, false]),
    ];
    let project = scratch("cases");
    ess_ui_react::generate(&document, &root(), &project).unwrap_or_else(|error| panic!("{error}"));
    compile(&project, &["runtime/expr.ts"]);
    node(
        &project,
        "cases",
        &format!(
            r#"
const {{ evaluate }} = out("runtime/expr");
const cases = {cases};
const wrong = [];
for (const [text, row, expected] of cases) {{
  const got = evaluate(text, {{ row }});
  try {{ assert.deepStrictEqual(got, expected); }}
  catch {{ wrong.push(`${{JSON.stringify(text)}} => ${{JSON.stringify(got)}}, expected ${{JSON.stringify(expected)}}`); }}
}}
assert.deepStrictEqual(wrong, []);
"#,
            cases = Value::Array(cases)
        ),
    );
}
