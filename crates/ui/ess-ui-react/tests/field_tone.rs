//! A field takes a tone (story:a-badge-field-takes-a-tone): a column or record field with `tone`
//! and `tone_by` is drawn as a badge in the tone of its value when `as: badge`, and as toned text
//! otherwise; a tone map written as bands tones a number by range; a field without the keys is
//! drawn as it was.
//!
//! Each case reads the `tone` and `toneBy` the generated page passes a column, then renders the
//! generated runtime's `FieldValue` with them under `node`.

mod support;

use std::path::{Path, PathBuf};

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
  sections: [{name: all, pages: [runs]}]
tone_maps:
  run_state: {failed: danger, done: success}
  load: {bands: [{from: 0, tone: success}, {from: 50, tone: warning}, {from: 80, tone: danger}]}
pages:
  runs:
    kind: list_page
    title: Runs
    sections:
      - name: runs
        component: collection
        reads: t.Runs
        columns:
          - {field: state, as: badge, tone: info, tone_by: run_state}
          - {field: load, as: badge, tone_by: load}
          - {field: owner, tone_by: run_state}
          - {field: title}
"#;

fn scratch(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("ess-ui-react-field-tone")
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

/// The text the generated page passes the column `name` as `prop`: an object literal or a
/// string, or `undefined` when it passes none.
fn column_prop(pages: &str, name: &str, prop: &str) -> String {
    let marker = format!("\"data-ui-path\": \"pages/runs/sections/runs/columns/{name}\"");
    let start = pages
        .find(&marker)
        .unwrap_or_else(|| panic!("the page passes the column `{name}`:\n{pages}"));
    let spec = &pages[start + marker.len()..];
    let spec = &spec[..spec.find("\"data-ui-path\": ").unwrap_or(spec.len())];
    let Some(open) = spec.find(&format!("{prop}: ")) else {
        return "undefined".to_owned();
    };
    let literal = &spec[open + prop.len() + 2..];
    if literal.starts_with('"') {
        let close = literal[1..].find('"').expect("the string closes") + 2;
        return literal[..close].to_owned();
    }
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
    panic!("`{prop}` of `{name}` does not close: {literal}")
}

/// Renders `FieldValue` for a field over a row and prints `[text, badge class, field class]`.
const RENDER: &str = r#"
const { FieldValue } = out("runtime/fields");
function shown(field, row) {
  const root = React.__root(jsx(FieldValue, { field, row })).settle();
  const badge = root.find((n) => n.tag === "span" && /\bui-badge\b/.test(String(n.props.className)))[0];
  const outer = root.find((n) => n.tag === "span" && /\bui-field\b/.test(String(n.props.className)))[0];
  return [root.text(), badge ? badge.props.className : null, outer.props.className];
}
"#;

fn render(project: &Path, name: &str, script: &str) {
    compile(project, &["runtime/fields.tsx"]);
    node(project, name, &format!("{RENDER}\n{script}"));
}

#[test]
fn the_page_passes_the_tone_and_the_resolved_map() {
    let (_, pages) = generate("spec");
    assert_eq!(column_prop(&pages, "state", "tone"), r#""info""#);
    assert_eq!(
        column_prop(&pages, "state", "toneBy"),
        r#"{ "failed": "danger", "done": "success" }"#
    );
    assert_eq!(column_prop(&pages, "title", "tone"), "undefined");
    assert_eq!(column_prop(&pages, "title", "toneBy"), "undefined");
}

#[test]
fn a_badge_column_takes_the_tone_of_its_value() {
    let (project, pages) = generate("badge");
    let tone = column_prop(&pages, "state", "tone");
    let tone_by = column_prop(&pages, "state", "toneBy");
    render(
        &project,
        "badge",
        &format!(
            r#"
const field = {{ "data-ui-path": "pages/runs/sections/runs/columns/state", field: "state", name: "state", as: "badge", tone: {tone}, toneBy: {tone_by} }};
const classes = (row) => shown(field, row)[1].split(" ");
assert.ok(classes({{ state: "failed" }}).includes("ui-tone-danger"));
assert.ok(classes({{ state: "done" }}).includes("ui-tone-success"));
assert.ok(classes({{ state: "queued" }}).includes("ui-tone-info"), "a value the map does not name takes the field's tone");
assert.ok(classes({{ state: "failed" }}).includes("ui-badge-failed"), "the class keeps the raw value");
"#
        ),
    );
}

#[test]
fn a_column_that_is_not_a_badge_is_toned_text() {
    let (project, pages) = generate("text");
    let tone_by = column_prop(&pages, "owner", "toneBy");
    render(
        &project,
        "text",
        &format!(
            r#"
const field = {{ "data-ui-path": "pages/runs/sections/runs/columns/owner", field: "owner", name: "owner", toneBy: {tone_by} }};
const [text, badge, outer] = shown(field, {{ owner: "failed" }});
assert.strictEqual(text, "failed");
assert.strictEqual(badge, null, "toned text is not a badge");
assert.ok(outer.split(" ").includes("ui-field-tone-danger"), outer);
const [, , fallback] = shown(field, {{ owner: "ana" }});
assert.ok(fallback.split(" ").includes("ui-field-tone-neutral"), fallback);
"#
        ),
    );
}

#[test]
fn a_band_map_tones_a_number_by_range() {
    let (project, pages) = generate("bands");
    let tone_by = column_prop(&pages, "load", "toneBy");
    render(
        &project,
        "bands",
        &format!(
            r#"
const field = {{ "data-ui-path": "pages/runs/sections/runs/columns/load", field: "load", name: "load", as: "badge", toneBy: {tone_by} }};
const tone = (value) => shown(field, {{ load: value }})[1].split(" ").find((c) => c.startsWith("ui-tone-"));
assert.strictEqual(tone(0), "ui-tone-success");
assert.strictEqual(tone(49.9), "ui-tone-success");
assert.strictEqual(tone(50), "ui-tone-warning");
assert.strictEqual(tone(79), "ui-tone-warning");
assert.strictEqual(tone(80), "ui-tone-danger");
assert.strictEqual(tone(1000), "ui-tone-danger");
assert.strictEqual(tone(-1), "ui-tone-neutral", "below the first band takes the node's tone");
assert.strictEqual(tone("abc"), "ui-tone-neutral", "a value that is no number takes the node's tone");
assert.strictEqual(tone(null), "ui-tone-neutral");
"#
        ),
    );
}

#[test]
fn a_field_without_the_keys_is_drawn_as_it_was() {
    let (project, pages) = generate("plain");
    let _ = pages;
    render(
        &project,
        "plain",
        r#"
const field = { "data-ui-path": "pages/runs/sections/runs/columns/title", field: "title", name: "title" };
assert.deepStrictEqual(shown(field, { title: "x" }), ["x", null, "ui-field ui-field-text"]);
const badge = { "data-ui-path": "pages/runs/sections/runs/columns/state", field: "state", name: "state", as: "badge" };
assert.deepStrictEqual(shown(badge, { state: "failed" }), ["failed", "ui-badge ui-badge-failed", "ui-field ui-field-badge"]);
"#,
    );
}
