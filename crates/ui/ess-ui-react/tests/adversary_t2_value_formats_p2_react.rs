//! Adversary cases, pass 2, for story:a-declared-field-format-renders-or-is-refused, React side:
//! the UTC correction against the terminal's reading of the same values, and the form round trip
//! of `as: timestamp` and `as: percent`.

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
        .join("ess-ui-react-adversary-t2-value-formats-p2")
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
function retype(root, typed) {
  const input = root.find((n) => n.tag === "input")[0];
  input.props.onChange({ target: { value: typed === undefined ? input.props.value : typed } });
  root.settle();
  return root.find((n) => n.tag === "input")[0].props.value;
}
"#;

fn render(name: &str, script: &str) {
    let project = generate(name);
    compile(&project, &["runtime/fields.tsx"]);
    node(&project, name, &format!("{RENDER}\n{script}"));
}

/// The terminal reads an ISO text with no offset as UTC (`view.rs` `instant`: "no offset reads
/// as UTC ... as the React target shows it") and shows `03:04`. JavaScript reads a date-time with
/// no offset in the browser's zone, and the correction then prints it in UTC: `02:04` in Berlin.
/// Before the correction both showed `03:04`.
#[test]
fn a_zoneless_instant_shows_the_clock_the_terminal_shows() {
    render(
        "zoneless",
        r#"
process.env.TZ = "Europe/Berlin";
const naive = "2026-01-02T03:04:05";
const time = shown(column("started", { as: "time" }), { started: naive }).text();
assert.strictEqual(time, "03:04", `as: time of ${naive} shows 03:04 as the terminal does, got ${time}`);
const stamp = shown(column("started", { as: "timestamp" }), { started: naive }).text();
assert.ok(stamp.includes("03:04"), `as: timestamp of ${naive} shows 03:04 as the terminal does, got ${stamp}`);
"#,
    );
}

/// `as: timestamp` shows the UTC day (01/01/2026, 23:30) and the terminal's `as: date` shows
/// 2026-01-01; React's `as: date` still reads the browser's zone and shows the next day in Berlin.
#[test]
fn a_date_shows_the_day_its_timestamp_shows() {
    render(
        "date-day",
        r#"
process.env.TZ = "Europe/Berlin";
const iso = "2026-01-01T23:30:00Z";
const stamp = shown(column("started", { as: "timestamp" }), { started: iso }).text();
const date = shown(column("started", { as: "date" }), { started: iso }).text();
const utcDay = new Date(iso).toLocaleDateString(undefined, { timeZone: "UTC" });
assert.ok(stamp.includes("23:30"), `the timestamp is the UTC instant: ${stamp}`);
assert.strictEqual(date, utcDay, `as: date of ${iso} shows the UTC day the timestamp and the terminal show, got ${date}`);
"#,
    );
}

/// A held text with seconds and no offset is put into the datetime-local as it is (with its
/// seconds, as a browser keeps them); editing it writes `<value>:00Z`, which is no instant.
#[test]
fn a_zoneless_timestamp_with_seconds_writes_back_an_instant() {
    render(
        "form-seconds",
        r#"
const root = host(column("started", { as: "timestamp" }), { started: "2026-01-02T03:04:05" });
retype(root);
const written = draft().started;
assert.ok(!Number.isNaN(new Date(written).getTime()), `a timestamp written back is an instant, got ${JSON.stringify(written)}`);
"#,
    );
}

/// A held epoch-ms number written back unchanged is still a number: the command that took a
/// number does not receive an ISO text.
#[test]
fn an_epoch_ms_timestamp_writes_back_a_number() {
    render(
        "form-epoch",
        r#"
const root = host(column("started", { as: "timestamp" }), { started: 1767323045000 });
const shownValue = retype(root);
assert.strictEqual(shownValue, "2026-01-02T03:04", `the input shows the UTC minute: ${shownValue}`);
assert.strictEqual(typeof draft().started, "number", `an epoch-ms value stays a number, got ${JSON.stringify(draft().started)}`);
"#,
    );
}

/// Pins the round trip the suite does not: an offset value shows its UTC minute and writes the
/// same instant back; an empty input writes null; a fraction percent shows and writes hundredths
/// without drift; a hundredths percent writes the typed number.
#[test]
fn the_form_round_trip_keeps_the_instant_and_the_share() {
    render(
        "form-round-trip",
        r#"
process.env.TZ = "Europe/Berlin";
const stamp = host(column("started", { as: "timestamp" }), { started: "2026-01-02T03:04:00+05:45" });
assert.strictEqual(retype(stamp), "2026-01-01T21:19");
assert.strictEqual(new Date(draft().started).getTime(), Date.parse("2026-01-02T03:04:00+05:45"), `same instant: ${draft().started}`);
retype(stamp, "");
assert.strictEqual(draft().started, null, "an empty timestamp writes null");
const fraction = host(column("ratio", { as: "percent" }), { ratio: 0.123 });
assert.strictEqual(fraction.find((n) => n.tag === "input")[0].props.value, "12.3");
retype(fraction);
assert.strictEqual(draft().ratio, 0.123, `0.123 read and written back: ${draft().ratio}`);
retype(fraction, "0.07");
assert.strictEqual(draft().ratio, 0.0007, `0.07 percent: ${draft().ratio}`);
const hundredths = host(column("rate", { as: "percent", scale: "hundredths" }), { rate: 12.5 });
assert.strictEqual(hundredths.find((n) => n.tag === "input")[0].props.value, "12.5");
retype(hundredths, "");
assert.strictEqual(draft().rate, null, "an empty percent writes null");
"#,
    );
}
