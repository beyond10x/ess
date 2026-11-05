//! Values in the form their field declares, in the generated React project
//! (story:a-value-renders-in-the-form-its-field-declares): an epoch-millisecond date as a date, an
//! audio URL as a player, a link as an anchor, object tags by their label, a dotted path into a
//! list as its values, a byte count scaled in 1024 steps; a JSON field sends the parsed value and
//! a list field adds and removes entries. A duration renders `h:mm:ss` from milliseconds or
//! seconds as its `unit` says (story:a-duration-renders-as-hours-minutes-seconds).
//!
//! Each case renders the generated runtime's `FieldValue`, `FieldInput` or `MetricView` under
//! `node`, with the field the generated page passes.

mod support;

use std::path::{Path, PathBuf};
use std::process::Command;

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
      - {name: stored, component: metric, reads: t.Usage, format: bytes}
      - name: calls
        component: collection
        reads: t.Calls
        columns:
          - {field: started, as: date}
          - {field: recording, as: audio}
          - {field: site, as: link}
          - {field: labels, as: tags}
          - {field: owners.name, name: owners}
          - {field: size, as: bytes}
          - {field: talk, as: duration}
          - {field: hold, as: duration, unit: s}
";

fn scratch(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("ess-ui-react-value-formats")
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

/// The field the page passes for the column `name` of the calls collection, up to the next node.
fn column(pages: &str, name: &str) -> String {
    let marker = format!("\"data-ui-path\": \"pages/calls/sections/calls/columns/{name}\"");
    let start = pages
        .find(&marker)
        .unwrap_or_else(|| panic!("the page passes the column `{name}`:\n{pages}"));
    let spec = &pages[start + marker.len()..];
    spec[..spec.find("\"data-ui-path\": ").unwrap_or(spec.len())].to_owned()
}

/// `shown(field, row)` renders `FieldValue` and returns the root; `host(field, draft)` renders
/// `FieldInput` over a draft it holds, which `draft()` returns.
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

fn render(project: &Path, name: &str, script: &str) {
    compile(
        project,
        &["runtime/fields.tsx", "runtime/composites/metric.tsx"],
    );
    node(project, name, &format!("{RENDER}\n{script}"));
}

#[test]
fn an_epoch_ms_number_under_as_date_renders_a_date() {
    let (project, pages) = generate("date");
    assert!(
        column(&pages, "started").contains(r#"as: "date""#),
        "the page passes `as: date`"
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
        "date",
        r#"
const ms = 1700000000000;
const date = shown(column("started", { as: "date" }), { started: ms }).text();
assert.strictEqual(date, new Date(ms).toLocaleDateString(), "an epoch-ms number is a date");
assert.notStrictEqual(date, new Intl.NumberFormat().format(ms));
const time = shown(column("started", { as: "time" }), { started: ms }).text();
assert.strictEqual(time, new Date(ms).toLocaleString(), "an epoch-ms number is a time");
const iso = "2026-11-04T10:00:00Z";
assert.strictEqual(shown(column("started", { as: "date" }), { started: iso }).text(), new Date(iso).toLocaleDateString());
assert.strictEqual(shown(column("started", { as: "date" }), { started: null }).text(), "");
// A number without `as: date` stays a number.
assert.strictEqual(shown(column("started"), { started: 1234 }).text(), new Intl.NumberFormat().format(1234));
"#,
    );
}

#[test]
fn an_audio_field_renders_a_player() {
    let (project, pages) = generate("audio");
    assert!(column(&pages, "recording").contains(r#"as: "audio""#));
    render(
        &project,
        "audio",
        r#"
const url = "https://media.test/calls/c-1.mp3";
const root = shown(column("recording", { as: "audio" }), { recording: url });
const players = root.find((n) => n.tag === "audio");
assert.strictEqual(players.length, 1, "one player");
assert.strictEqual(players[0].props.src, url);
assert.strictEqual(players[0].props.controls, true, "the player has controls");
assert.ok(!root.text().includes(url), `the URL is not printed: ${root.text()}`);
const none = shown(column("recording", { as: "audio" }), { recording: null });
assert.strictEqual(none.find((n) => n.tag === "audio").length, 0, "no recording, no player");
const script = shown(column("recording", { as: "audio" }), { recording: "javascript:alert(1)" });
assert.strictEqual(script.find((n) => n.tag === "audio").length, 0, "a script URL is no source");
"#,
    );
}

#[test]
fn a_link_column_renders_an_anchor() {
    let (project, pages) = generate("link");
    assert!(column(&pages, "site").contains(r#"as: "link""#));
    render(
        &project,
        "link",
        r#"
const url = "https://shop.test/items/7";
const root = shown(column("site", { as: "link" }), { site: url });
const anchors = root.find((n) => n.tag === "a");
assert.strictEqual(anchors.length, 1, "one anchor");
assert.strictEqual(anchors[0].props.href, url);
assert.strictEqual(anchors[0].props.target, "_blank");
assert.strictEqual(anchors[0].props.rel, "noreferrer");
assert.strictEqual(root.text(), url, "the anchor reads as the address");
const relative = shown(column("site", { as: "link" }), { site: "/items/7" }).find((n) => n.tag === "a");
assert.strictEqual(relative[0].props.href, "/items/7");
const script = shown(column("site", { as: "link" }), { site: "javascript:alert(1)" });
assert.strictEqual(script.find((n) => n.tag === "a").length, 0, "a script URL is no anchor");
assert.strictEqual(script.text(), "javascript:alert(1)", "it shows as text");
const none = shown(column("site", { as: "link" }), { site: null });
assert.strictEqual(none.find((n) => n.tag === "a").length, 0);
"#,
    );
}

#[test]
fn object_tags_render_their_label() {
    let (project, _) = generate("tags");
    render(
        &project,
        "tags",
        r#"
const tags = (root) => root.find((n) => n.tag === "span" && n.props.className === "ui-tag").map((n) => n.children.join(""));
const objects = shown(column("labels", { as: "tags" }), { labels: [{ id: "l-1", label: "Urgent" }, { id: "l-2", label: "Billing" }] });
assert.deepStrictEqual(tags(objects), ["Urgent", "Billing"]);
assert.ok(!objects.text().includes("[object Object]"), objects.text());
const plain = shown(column("labels", { as: "tags" }), { labels: ["a", "b"] });
assert.deepStrictEqual(tags(plain), ["a", "b"]);
// An object without a label shows as data, never as `[object Object]`.
const bare = shown(column("labels", { as: "tags" }), { labels: [{ id: "l-3" }] });
assert.deepStrictEqual(tags(bare), ['{"id":"l-3"}']);
"#,
    );
}

#[test]
fn a_dotted_path_into_a_list_renders_its_values() {
    let (project, pages) = generate("dotted");
    assert!(
        column(&pages, "owners").contains(r#"field: "owners.name""#),
        "the page passes the dotted path:\n{}",
        column(&pages, "owners")
    );
    render(
        &project,
        "dotted",
        r#"
const row = { owners: [{ name: "Ada" }, { name: "Lin" }] };
const owners = { "data-ui-path": "pages/calls/sections/calls/columns/owners", field: "owners.name", name: "owners" };
assert.strictEqual(shown(owners, row).text(), "Ada, Lin");
const tagged = shown({ ...owners, as: "tags" }, row);
const tags = tagged.find((n) => n.tag === "span" && n.props.className === "ui-tag").map((n) => n.children.join(""));
assert.deepStrictEqual(tags, ["Ada", "Lin"]);
"#,
    );
}

#[test]
fn a_json_field_sends_the_parsed_value_and_refuses_unparsable_text_at_the_field() {
    let (project, _) = generate("json");
    render(
        &project,
        "json",
        r#"
const field = { "data-ui-path": "f/config", field: "config", name: "config", as: "json" };
const root = host(field, { config: { retries: 1 } });
const area = () => root.find((n) => n.tag === "textarea")[0];
assert.deepStrictEqual(JSON.parse(area().props.value), { retries: 1 }, "the value shows as JSON");
area().props.onChange({ target: { value: '{"retries": 3, "tags": ["a"]}' } });
root.settle();
assert.deepStrictEqual(draft().config, { retries: 3, tags: ["a"] }, "the parsed value is sent");
assert.ok(!area().props["aria-invalid"], "valid JSON is not marked");
assert.strictEqual(root.find((n) => n.props && n.props.role === "alert").length, 0);

area().props.onChange({ target: { value: '{"retries": ' } });
root.settle();
assert.deepStrictEqual(draft().config, { retries: 3, tags: ["a"] }, "unparsable text is never sent");
assert.strictEqual(area().props.value, '{"retries": ', "the text being written stays");
assert.strictEqual(area().props["aria-invalid"], true, "the field is marked invalid");
const alerts = root.find((n) => n.props && n.props.role === "alert");
assert.strictEqual(alerts.length, 1, "the field says why");
// The browser refuses to submit the form while the field holds a validity message.
let validity;
area().props.ref({ setCustomValidity: (message) => { validity = message; } });
assert.ok(validity && validity.length > 0, "the field carries a validity message");

area().props.onChange({ target: { value: "[1, 2]" } });
root.settle();
assert.deepStrictEqual(draft().config, [1, 2]);
area().props.ref({ setCustomValidity: (message) => { validity = message; } });
assert.strictEqual(validity, "", "a parsed value clears the message");
assert.strictEqual(root.find((n) => n.props && n.props.role === "alert").length, 0);

area().props.onChange({ target: { value: "  " } });
root.settle();
assert.strictEqual(draft().config, undefined, "an empty field sends nothing");
"#,
    );
}

#[test]
fn a_list_field_adds_and_removes_entries() {
    let (project, _) = generate("list");
    render(
        &project,
        "list",
        r#"
const field = { "data-ui-path": "f/aliases", field: "aliases", name: "aliases", as: "list" };
const root = host(field, {});
const entries = () => root.find((n) => n.tag === "input");
const button = (label) => root.find((n) => n.tag === "button" && String(n.props["aria-label"]).startsWith(label));
assert.strictEqual(entries().length, 0, "an empty list has no entries");
click(button("Add")[0]);
root.settle();
assert.deepStrictEqual(draft().aliases, [""], "the first entry is added");
entries()[0].props.onChange({ target: { value: "north" } });
root.settle();
click(button("Add")[0]);
root.settle();
entries()[1].props.onChange({ target: { value: "south" } });
root.settle();
assert.deepStrictEqual(draft().aliases, ["north", "south"]);
assert.deepStrictEqual(entries().map((n) => n.props.value), ["north", "south"]);
assert.strictEqual(button("Remove").length, 2, "each entry can be removed");
click(button("Remove")[0]);
root.settle();
assert.deepStrictEqual(draft().aliases, ["south"]);
assert.deepStrictEqual(entries().map((n) => n.props.value), ["south"]);
for (const n of button("Add").concat(button("Remove"))) assert.strictEqual(n.props.type, "button", "no button submits the form");
"#,
    );
}

/// The cases of B7-1: a byte count and what it shows, `B` to `GB` in 1024 steps.
const BYTES: &str = r"
const cases = [
  [0, /^0 B$/],
  [512, /^512 B$/],
  [1023, /^1023 B$/],
  [1024, /^1 KB$/],
  [1536, /^1[.,]5 KB$/],
  [1048575, /^1 MB$/],
  [1048576, /^1 MB$/],
  [5 * 1024 ** 3, /^5 GB$/],
  [3 * 1024 ** 4, /^3072 GB$/],
  [-1536, /^-1[.,]5 KB$/],
];
";

#[test]
fn a_bytes_field_scales_in_1024_steps() {
    let (project, pages) = generate("bytes-field");
    assert!(column(&pages, "size").contains(r#"as: "bytes""#));
    render(
        &project,
        "bytes-field",
        &format!(
            r#"{BYTES}
for (const [value, expected] of cases) {{
  const text = shown(column("size", {{ as: "bytes" }}), {{ size: value }}).text();
  assert.match(text, expected, `${{value}} bytes`);
}}
assert.strictEqual(shown(column("size", {{ as: "bytes" }}), {{ size: null }}).text(), "");
"#
        ),
    );
}

#[test]
fn a_bytes_metric_scales_in_1024_steps() {
    let (project, _) = generate("bytes-metric");
    render(
        &project,
        "bytes-metric",
        &format!(
            r#"{BYTES}
const {{ MetricView }} = out("runtime/composites/metric");
for (const [value, expected] of cases) {{
  const root = React.__root(jsx(ScopeLayer, {{ values: {{ state: {{ size: value }} }}, children: jsx(MetricView, {{ "data-ui-path": "m", from: "state.size", format: "bytes" }}) }})).settle();
  const shown = root.find((n) => n.tag === "span" && n.props.className === "ui-metric-value")[0];
  assert.match(shown.children.join(""), expected, `${{value}} bytes`);
}}
"#
        ),
    );
}

#[test]
fn the_page_passes_a_duration_unit() {
    let (_, pages) = generate("duration-unit");
    let talk = column(&pages, "talk");
    assert!(talk.contains(r#"as: "duration""#), "{talk}");
    assert!(
        !talk.contains("unit:"),
        "no unit written, none passed: {talk}"
    );
    let hold = column(&pages, "hold");
    assert!(hold.contains(r#"as: "duration""#), "{hold}");
    assert!(
        hold.contains(r#"unit: "s""#),
        "the page passes `unit: s`: {hold}"
    );
}

#[test]
fn a_duration_field_renders_hours_minutes_and_seconds() {
    let (project, _) = generate("duration");
    render(
        &project,
        "duration",
        r#"
const talk = column("talk", { as: "duration" });
const hold = column("hold", { as: "duration", unit: "s" });
const ms = (value) => shown(talk, { talk: value }).text();
const s = (value) => shown(hold, { hold: value }).text();
assert.strictEqual(ms(73000), "0:01:13", "73000 ms with the default unit");
assert.strictEqual(shown(column("talk", { as: "duration", unit: "ms" }), { talk: 73000 }).text(), "0:01:13");
assert.strictEqual(s(73), "0:01:13", "73 s");
assert.strictEqual(ms(26 * 3600 * 1000), "26:00:00", "26 hours stay hours");
assert.strictEqual(s(26 * 3600), "26:00:00", "26 hours stay hours");
assert.strictEqual(s(100 * 3600 + 61), "100:01:01");
assert.strictEqual(ms(0), "0:00:00");
assert.strictEqual(ms(59999), "0:00:59", "whole seconds, never rounded up");
assert.strictEqual(s(3599), "0:59:59");
assert.strictEqual(s(3600), "1:00:00", "hours are not padded");
assert.strictEqual(s(-73), "-0:01:13");
assert.strictEqual(ms(null), "");
assert.strictEqual(ms("soon"), "soon", "a value that is no number shows as it is");
// A number without `as: duration` is still a number.
assert.strictEqual(shown(column("talk"), { talk: 73000 }).text(), new Intl.NumberFormat().format(73000));
"#,
    );
}
