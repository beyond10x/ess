//! Value labels in the generated React project
//! (story:a-value-shows-its-declared-label-beside-its-tone): a column shows the label the document
//! declares for its value, through `labels:` or a choice's fixed options, while the badge's class
//! keeps the raw value its tone is chosen by; a value the map does not name shows as it did.
//!
//! Each case reads the `labels` the generated page passes a column, then renders the generated
//! runtime's `FieldValue` with them under `node`.

mod support;

use std::path::{Path, PathBuf};
use std::process::Command;

use support::{compile, node};

const DOCUMENT: &str = r#"
format: ess-ui/1
app: t
model: t.system
placement_profile: fat
shells:
  app: {regions: {main: {kind: page_outlet}}}
navigation:
  home: runs
  sections: [{name: all, pages: [runs, run]}]
label_maps:
  run_state: {in_progress: In progress}
pages:
  runs:
    kind: list_page
    title: Runs
    sections:
      - name: runs
        component: collection
        reads: t.Runs
        columns:
          - {field: state, as: badge, labels: run_state}
          - {field: enabled, as: badge, labels: {"true": Active, "false": Inactive}}
          - {field: priority, as: choice, choice: {component: choice, options: [{value: p1, label: Urgent}, {value: p2, label: Normal}]}}
          - {field: owner}
  run:
    kind: detail_page
    title: Run
    sections:
      - name: summary
        reads: t.ById
        children:
          - {name: state, primitive: badge, field: state, labels: {in_progress: In progress}}
"#;

fn scratch(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("ess-ui-react-value-labels")
        .join(name);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("the old scratch dir is removed");
    }
    std::fs::create_dir_all(&dir).expect("the scratch dir is created");
    dir
}

/// Generates the document into `<scratch>/app` and returns the project and its page sources.
fn generate(name: &str) -> (PathBuf, String) {
    let dir = scratch(name);
    let document = ess_ui::load_str(DOCUMENT).unwrap_or_else(|error| panic!("{error}"));
    let project = dir.join("app");
    let files = ess_ui_react::generate(&document, &dir, &project)
        .unwrap_or_else(|error| panic!("the document generates: {error}"));
    let pages: String = files
        .files
        .iter()
        .filter(|(path, _)| !path.starts_with("src/runtime/"))
        .map(|(_, text)| text.as_str())
        .collect();
    (project, pages)
}

/// The `labels` object literal the page passes the column `name` of the runs collection, or
/// `undefined` when it passes none.
fn column_labels(pages: &str, name: &str) -> String {
    let marker = format!("\"data-ui-path\": \"pages/runs/sections/runs/columns/{name}\"");
    let start = pages
        .find(&marker)
        .unwrap_or_else(|| panic!("the page passes the column `{name}`:\n{pages}"));
    let spec = &pages[start + marker.len()..];
    let spec = &spec[..spec.find("\"data-ui-path\": ").unwrap_or(spec.len())];
    let Some(open) = spec.find("labels: {") else {
        return "undefined".to_owned();
    };
    let literal = &spec[open + "labels: ".len()..];
    let mut depth = 0;
    for (at, character) in literal.char_indices() {
        match character {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return literal[..=at].to_owned();
                }
            }
            _ => {}
        }
    }
    panic!("the labels of `{name}` do not close: {literal}")
}

/// Renders `FieldValue` for the column `name` over each row and prints `[text, className]`.
const RENDER: &str = r#"
const { FieldValue } = out("runtime/fields");
function shown(field, row) {
  const root = React.__root(jsx(FieldValue, { field, row })).settle();
  const badge = root.find((n) => n.tag === "span" && /\bui-badge\b/.test(String(n.props.className)))[0];
  return [root.text(), badge ? badge.props.className : null];
}
"#;

fn render(project: &Path, name: &str, script: &str) {
    compile(project, &["runtime/fields.tsx"]);
    node(project, name, &format!("{RENDER}\n{script}"));
}

#[test]
fn a_badge_column_shows_the_label_and_keeps_the_raw_value_class() {
    let (project, pages) = generate("badge");
    let labels = column_labels(&pages, "state");
    assert_eq!(
        labels, r#"{ in_progress: "In progress" }"#,
        "the page passes the label map `labels: run_state` names"
    );
    let output = Command::new("tsc")
        .args(["-p", "tsconfig.offline.json", "--noEmit"])
        .current_dir(&project)
        .output()
        .expect("tsc is on PATH");
    assert!(
        output.status.success(),
        "the generated project type-checks:\n{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    render(
        &project,
        "badge",
        &format!(
            r#"
const field = {{ "data-ui-path": "pages/runs/sections/runs/columns/state", field: "state", name: "state", as: "badge", labels: {labels} }};
const [text, className] = shown(field, {{ state: "in_progress" }});
assert.strictEqual(text, "In progress");
assert.ok(className.split(" ").includes("ui-badge-in_progress"), `the class keeps the raw value: ${{className}}`);
"#
        ),
    );
}

#[test]
fn a_boolean_column_shows_its_labels() {
    let (project, pages) = generate("boolean");
    let labels = column_labels(&pages, "enabled");
    assert_eq!(labels, r#"{ false: "Inactive", true: "Active" }"#);
    render(
        &project,
        "boolean",
        &format!(
            r#"
const field = {{ "data-ui-path": "pages/runs/sections/runs/columns/enabled", field: "enabled", name: "enabled", as: "badge", labels: {labels} }};
assert.deepStrictEqual(shown(field, {{ enabled: true }}), ["Active", "ui-badge ui-badge-true"]);
assert.deepStrictEqual(shown(field, {{ enabled: false }}), ["Inactive", "ui-badge ui-badge-false"]);
"#
        ),
    );
}

#[test]
fn a_value_with_no_label_shows_the_raw_value() {
    let (project, pages) = generate("raw");
    let labels = column_labels(&pages, "state");
    assert_eq!(
        column_labels(&pages, "owner"),
        "undefined",
        "a column without labels passes none"
    );
    render(
        &project,
        "raw",
        &format!(
            r#"
const state = {{ "data-ui-path": "pages/runs/sections/runs/columns/state", field: "state", name: "state", as: "badge", labels: {labels} }};
assert.deepStrictEqual(shown(state, {{ state: "queued" }}), ["queued", "ui-badge ui-badge-queued"]);
assert.deepStrictEqual(shown(state, {{ state: null }}), ["", "ui-badge ui-badge-null"]);
// A key the map does not hold is not looked up on the object's prototype.
assert.deepStrictEqual(shown(state, {{ state: "constructor" }}), ["constructor", "ui-badge ui-badge-constructor"]);
const owner = {{ "data-ui-path": "pages/runs/sections/runs/columns/owner", field: "owner", name: "owner" }};
assert.deepStrictEqual(shown(owner, {{ owner: "in_progress" }}), ["in_progress", null]);
"#
        ),
    );
}

#[test]
fn a_choice_column_shows_its_option_labels() {
    let (project, pages) = generate("choice");
    let labels = column_labels(&pages, "priority");
    assert_eq!(
        labels, r#"{ p1: "Urgent", p2: "Normal" }"#,
        "the page passes the choice's fixed option labels"
    );
    render(
        &project,
        "choice",
        &format!(
            r#"
const field = {{ "data-ui-path": "pages/runs/sections/runs/columns/priority", field: "priority", name: "priority", as: "choice", labels: {labels} }};
assert.deepStrictEqual(shown(field, {{ priority: "p1" }}), ["Urgent", "ui-badge ui-badge-p1"]);
assert.deepStrictEqual(shown(field, {{ priority: "p3" }}), ["p3", "ui-badge ui-badge-p3"]);
"#
        ),
    );
}

/// The badge primitive takes `labels:` too, and its class keeps the tone it had.
#[test]
fn a_badge_primitive_shows_the_label() {
    let (project, pages) = generate("primitive");
    assert!(
        pages.contains(r#"labels={{ in_progress: "In progress" }}"#),
        "the page passes the badge its labels:\n{pages}"
    );
    compile(
        &project,
        &["runtime/core.tsx", "runtime/primitives/badge.tsx"],
    );
    node(
        &project,
        "primitive",
        r#"
const { ScopeLayer } = out("runtime/core");
const { BadgeView } = out("runtime/primitives/badge");
function shown(row) {
  const badge = jsx(BadgeView, { "data-ui-path": "pages/run/sections/summary/children/state", field: "state", labels: { in_progress: "In progress" } });
  const root = React.__root(jsx(ScopeLayer, { values: { row }, children: badge })).settle();
  const span = root.find((n) => n.tag === "span")[0];
  return [root.text(), span.props.className];
}
assert.deepStrictEqual(shown({ state: "in_progress" }), ["In progress", "ui-badge ui-tone-neutral"]);
assert.deepStrictEqual(shown({ state: "queued" }), ["queued", "ui-badge ui-tone-neutral"]);
"#,
    );
}
