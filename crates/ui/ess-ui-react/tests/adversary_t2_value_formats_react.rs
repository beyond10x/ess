//! Adversary cases for story:a-declared-field-format-renders-or-is-refused, React side.

mod support;

use std::path::{Path, PathBuf};

use support::{compile, node};

const DOCUMENT: &str = r"
format: ess-ui/1
app: t
model: t.system
placement_profile: fat
shells:
  app: {regions: {main: {kind: page_outlet}}}
navigation:
  home: calls
  sections: [{name: all, pages: [calls]}]
pages:
  calls:
    kind: list_page
    title: Calls
    sections:
      - name: calls
        component: collection
        reads: t.Calls
        columns:
          - {field: started, as: timestamp}
          - {field: rate, as: percent, scale: hundredths}
";

fn scratch(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("ess-ui-react-adversary-t2-value-formats")
        .join(name);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("the old scratch dir is removed");
    }
    std::fs::create_dir_all(&dir).expect("the scratch dir is created");
    dir
}

fn generate(name: &str) -> PathBuf {
    let dir = scratch(name);
    let document = ess_ui::load_str(DOCUMENT).unwrap_or_else(|error| panic!("{error}"));
    let project = dir.join("app");
    ess_ui_react::generate(&document, &dir, &project)
        .unwrap_or_else(|error| panic!("the document generates: {error}"));
    project
}

const RENDER: &str = r#"
const { FieldValue, FieldInput } = out("runtime/fields");
const { ScopeLayer } = out("runtime/core");
function shown(field, row) {
  return React.__root(jsx(FieldValue, { field, row })).settle();
}
let held;
function Host(props) {
  const [draft, setDraft] = React.useState(props.draft);
  held = draft;
  return jsx(ScopeLayer, { values: { draft }, setters: { draft: setDraft }, children: jsx(FieldInput, { field: props.field }) });
}
function host(field, draft) {
  return React.__root(jsx(Host, { field, draft })).settle();
}
const draft = () => held;
const column = (name, extra) => ({ "data-ui-path": `pages/calls/sections/calls/columns/${name}`, field: name, name, ...extra });
"#;

fn render(name: &str, script: &str) {
    let project = generate(name);
    compile(&project, &["runtime/fields.tsx"]);
    node(&project, name, &format!("{RENDER}\n{script}"));
}

/// Acceptance: `2026-01-02T03:04:05Z` shows `03:04`; the terminal shows `03:04` whatever the
/// zone. React shows the browser's local clock, so in any zone but UTC the two renderers disagree.
#[test]
fn a_utc_instant_shows_the_clock_the_terminal_shows() {
    render(
        "tz",
        r#"
process.env.TZ = "Europe/Berlin";
const iso = "2026-01-02T03:04:05Z";
const time = shown(column("started", { as: "time" }), { started: iso }).text();
assert.strictEqual(time, "03:04", `as: time of ${iso} shows 03:04 as the terminal does, got ${time}`);
const stamp = shown(column("started", { as: "timestamp" }), { started: iso }).text();
assert.ok(stamp.includes("03:04"), `as: timestamp of ${iso} shows 03:04 as the terminal does, got ${stamp}`);
"#,
    );
}

/// A form field `as: timestamp` edits an instant, as `as: time` and `as: date` get a date-time
/// and a date input; it does not fall back to a text box.
#[test]
fn a_timestamp_form_field_is_no_text_box() {
    render(
        "form-timestamp",
        r#"
const root = host(column("started", { as: "timestamp" }), { started: "2026-01-02T03:04" });
const input = root.find((n) => n.tag === "input")[0];
assert.strictEqual(input.props.type, "datetime-local", `a timestamp input: ${input.props.type}`);
"#,
    );
}

/// A form field `as: percent` edits a number, as `as: number` does; a text box sends a string.
#[test]
fn a_percent_form_field_sends_a_number() {
    render(
        "form-percent",
        r#"
const root = host(column("rate", { as: "percent", scale: "hundredths" }), { rate: 50 });
const input = () => root.find((n) => n.tag === "input")[0];
input().props.onChange({ target: { value: "75" } });
root.settle();
assert.strictEqual(draft().rate, 75, `a percent field sends a number, got ${JSON.stringify(draft().rate)}`);
"#,
    );
}
