//! Adversary pass 2 on story:ui-react-live-binding, against correction 1: the poll that skips a
//! tick while its read is in flight, the form's pending guard and its revert-only-if-current rule,
//! and the bound `refresh:` validation. Runtime cases compile the generated project with `tsc` and
//! run it under `node` against the stand-ins in `support`, as pass 1 does.

mod support;

use std::path::{Path, PathBuf};

use ess_ui::binding::Binding;
use ess_ui::Document;
use ess_ui_react::GeneratedFiles;
use support::{compile, node};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

fn scratch(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("ess-ui-react-adv-live-2")
        .join(name);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("the old scratch project is removed");
    }
    dir
}

fn read(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

const DESK: &str = "format: ess-ui/1
app: desk
model: gatepass
placement_profile: fat
shells: {app: {regions: {main: {kind: page_outlet}}}}
navigation: {home: desk, sections: [{name: all, pages: [desk]}]}
pages:
  desk:
    kind: detail_page
    title: Desk
    sections:
      - name: expected
        component: collection
        reads: visit.ExpectedVisits
        actions: [{name: admit, does: visit.AdmitVisitor, label: Admit}]
      - name: register
        component: form
        does: visit.RegisterVisit
        fields: [visitor, building, {field: expected_minutes, as: number}]
      - name: depart
        component: confirm
        does: visit.SignOutVisitor
        body: Sign the visitor out?
";

/// The desk with one live section over the expected list; `reads` is spliced in.
fn live_desk(reads: &str) -> String {
    format!(
        "format: ess-ui/1
app: desk
model: gatepass
placement_profile: fat
shells: {{app: {{regions: {{main: {{kind: page_outlet}}}}}}}}
navigation: {{home: desk, sections: [{{name: all, pages: [desk]}}]}}
channels:
  visits:
    carries: {{events: [visit.VisitRegistered]}}
    direction: server_to_client
    delivery: every_event
    resume: refetch
pages:
  desk:
    kind: detail_page
    title: Desk
    sections:
      - name: expected
        component: collection
        reads: {reads}
        live: {{channel: visits, effect: insert_top}}
"
    )
}

fn gatepass() -> Vec<(String, String)> {
    let root = root().join("examples/gatepass");
    ["system.yaml", "components.yaml", "domains/visit.yaml"]
        .iter()
        .map(|file| ((*file).to_owned(), read(&root.join(file))))
        .collect()
}

fn load(text: &str) -> Document {
    ess_ui::load_str(text).unwrap_or_else(|error| panic!("the document loads: {error}\n{text}"))
}

fn bind(document: &Document) -> Binding {
    ess_ui_check::binding(document, &gatepass())
        .unwrap_or_else(|error| panic!("the document binds: {error}"))
}

fn render(text: &str) -> Result<GeneratedFiles, String> {
    let document = load(text);
    let binding = bind(&document);
    ess_ui_react::render_bound(&document, &root(), Some(&binding)).map_err(|e| e.to_string())
}

/// The generated text that calls `usePoll`, if any section polls.
fn poll_line(files: &GeneratedFiles) -> Option<String> {
    files
        .files
        .values()
        .flat_map(|text| text.lines())
        .find(|line| line.contains("usePoll(__read"))
        .map(|line| line.trim().to_owned())
}

// ── generator: `refresh:` ───────────────────────────────────────────────────────────────────

/// Browsers and node hold a timer delay in a signed 32-bit integer: a delay above 2^31 - 1 ms
/// (about 24.8 days) is taken as 1 ms (HTML timers, "if timeout is greater than 2^31 - 1, set it
/// to 1"; node prints `TimeoutOverflowWarning` and does the same). `refresh: 600h` matches the
/// schema's duration pattern and passes the 1 s floor, so the bound app polls the server every
/// tick of the event loop — the busy loop the floor exists to stop. Measured under node with the
/// interval the generator emits.
#[test]
fn a_refresh_beyond_the_timer_range_does_not_poll_the_server_every_millisecond() {
    let text = live_desk("{view: visit.ExpectedVisits, refresh: 600h}");
    let Ok(files) = render(&text) else {
        return; // refused: nothing polls
    };
    let line = poll_line(&files).expect("a live section polls");
    let every: u64 = line
        .trim_start_matches("const __data = usePoll(__read, ")
        .trim_end_matches(");")
        .parse()
        .unwrap_or_else(|error| panic!("{line}: {error}"));
    let out = stage("timer-range");
    node(
        &out,
        "timer-range",
        &script(&format!(
            r#"
process.removeAllListeners("warning");
function Poller() {{
  const read = data.usePoll(useRead({{ view: "visit.ExpectedVisits" }}, {{}}), {every});
  return jsx("span", {{ children: read.status }});
}}
const root = React.__root(jsx(Poller, {{}})).settle();
await run(root, 300);
assert.ok(gets().length <= 2, `{line}: ${{gets().length}} reads in 300 ms`);
"#
        )),
    );
}

/// `millis` multiplies without checking: a `refresh:` the schema's pattern accepts overflows `u64`
/// and the generator panics (debug) or wraps to an unrelated interval (release).
#[test]
fn a_refresh_too_large_for_milliseconds_does_not_panic_the_generator() {
    let text = live_desk("{view: visit.ExpectedVisits, refresh: 5124095576031h}");
    let generated = render(&text);
    if let Ok(files) = generated {
        let line = poll_line(&files).expect("a live section polls");
        assert_ne!(line, "const __data = usePoll(__read, 2048384);", "wrapped");
    }
}

/// The schema types `refresh:` as `one_of: [duration, expr]`, and the loader holds it as an
/// expression, so `refresh: 0.5s` and `refresh: page.every` load. The bound generator reads it
/// with `millis`, gets nothing, and polls at the 5 s default: a declared interval is dropped with
/// no refusal at `…/reads/refresh`.
#[test]
fn a_refresh_the_generator_cannot_read_is_refused_not_replaced_by_the_default() {
    for refresh in ["0.5s", "page.every"] {
        let text = live_desk(&format!(
            "{{view: visit.ExpectedVisits, refresh: {refresh}}}"
        ));
        match render(&text) {
            Ok(files) => panic!(
                "refresh: {refresh} generated `{}`",
                poll_line(&files).unwrap_or_default()
            ),
            Err(error) => assert!(
                error.contains("pages/desk/sections/expected/reads/refresh"),
                "refresh: {refresh}: {error}"
            ),
        }
    }
}

// ── runtime ─────────────────────────────────────────────────────────────────────────────────

/// A stand-in page with a base-URL meta tag, a scripted `fetch`, and helpers. `hang: true` never
/// answers; `throws` rejects after `delay`.
const STAGE: &str = r#"
const { ScopeLayer } = out("runtime/core");
const data = out("runtime/data");
const { httpAdapter, setDataAdapter, setAuthorization, useRead } = data;
const { binding } = out("binding");
const sent = [];
let reply = (call) => (call.method === "GET" ? { status: 200, body: '{"rows":[]}' } : { status: 500, body: "{}" });
globalThis.fetch = async (url, init) => {
  const call = { url: String(url), method: (init && init.method) || "GET", headers: (init && init.headers) || {}, body: init && init.body };
  sent.push(call);
  const answer = reply(call);
  if (answer.hang) return new Promise(() => undefined);
  await new Promise((resolve) => setTimeout(resolve, answer.delay || 0));
  if (answer.throws) throw answer.throws;
  return { status: answer.status, ok: answer.status < 300, statusText: "", text: async () => answer.body };
};
globalThis.document = {
  querySelector: (selector) =>
    selector === 'meta[name="ess-base-url:pass-service"]' ? { getAttribute: () => "http://desk.test" } : null,
};
setAuthorization("Actor gatepass.visit.Receptionist");
setDataAdapter(httpAdapter(binding));
const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms));
const run = async (root, ms) => {
  const end = Date.now() + ms;
  while (Date.now() < end) {
    await sleep(5);
    root.settle();
  }
};
const posts = () => sent.filter((call) => call.method === "POST");
const gets = () => sent.filter((call) => call.method === "GET");
const refused = (status, error) => ({ status, body: JSON.stringify({ outcome: "refused", published: [], error, payload: {} }) });
const accepted = { status: 200, body: '{"outcome":"registered","published":[]}' };
// A form whose one group saves `expected_minutes` on change; `draft()` is what the form holds.
function saveOnChange(FormView, initial) {
  let held = initial;
  function Host() {
    const [value, setValue] = React.useState(initial);
    held = value;
    return jsx(ScopeLayer, {
      values: { draft: value },
      setters: { draft: setValue },
      actions: { close: () => undefined, notify: () => undefined },
      children: jsx(FormView, {
        "data-ui-path": "pages/desk/sections/register",
        does: "visit.RegisterVisit",
        groups: [{
          "data-ui-path": "pages/desk/sections/register/groups/length",
          name: "length",
          save: "on_change",
          fields: [{ "data-ui-path": "pages/desk/sections/register/groups/length/fields/expected_minutes", field: "expected_minutes", name: "expected_minutes", as: "number" }],
        }],
      }),
    });
  }
  const root = React.__root(jsx(Host, {})).settle();
  const type = (text) => {
    const input = root.find((n) => n.tag === "fieldset")[0];
    const found = [];
    const visit = (nodes) => nodes.forEach((x) => { if (typeof x === "object") { if (x.tag === "input") found.push(x); if (x.children) visit(x.children); } });
    visit(input.children);
    found[0].props.onChange({ target: { value: text } });
    root.settle();
  };
  return { root, type, draft: () => held };
}
"#;

fn stage(name: &str) -> PathBuf {
    let document = load(DESK);
    let binding = bind(&document);
    let out = scratch(name);
    ess_ui_react::generate_bound(&document, &root(), &out, Some(&binding))
        .unwrap_or_else(|error| panic!("{error}"));
    compile(
        &out,
        &[
            "binding.ts",
            "runtime/core.tsx",
            "runtime/data.ts",
            "runtime/composites/form.tsx",
            "runtime/composites/confirm.tsx",
        ],
    );
    out
}

fn script(body: &str) -> String {
    format!(
        "{STAGE}\n(async () => {{\n{body}\n}})().catch((error) => {{ console.error(error); \
         process.exit(1); }});\n"
    )
}

/// Correction 1 skips a tick while the read is in flight, and `fetch` has no deadline: one read
/// the server never answers leaves the section on its last rows and stops every later tick. In a
/// browser that lasts until the network stack gives up on the request (minutes), and against a
/// server that holds the connection open, for as long as it holds it.
#[test]
fn a_poll_reads_again_after_a_read_that_never_answers() {
    let out = stage("poll-hang");
    node(
        &out,
        "poll-hang",
        &script(
            r#"
let n = 0;
reply = () => (++n === 1 ? { hang: true } : { status: 200, body: '{"rows":[{"visit_id":"v-1"}]}' });
let last;
function Poller() {
  last = data.usePoll(useRead({ view: "visit.ExpectedVisits" }, {}), 30);
  return jsx("span", { children: last.status });
}
const root = React.__root(jsx(Poller, {})).settle();
await run(root, 600);
assert.ok(gets().length >= 2, `${gets().length} read in 600 ms at 30 ms; the section is ${last.status}`);
"#,
        ),
    );
}

/// Three saves on change, typed faster than the server answers (`1`, `12`, `120`), all refused and
/// answered in order. The server holds none of them. The first two refusals do not revert (a later
/// draft is current); the third reverts to the draft before it — `12`, the second refused value —
/// and the form shows a value the server refused.
#[test]
fn three_refused_saves_leave_the_draft_the_server_holds() {
    let out = stage("revert-three");
    node(
        &out,
        "revert-three",
        &script(
            r#"
const { FormView } = out("runtime/composites/form");
reply = (call) => call.method === "GET" ? { status: 200, body: '{"rows":[]}' } :
  { ...refused(422, "gatepass.visit.InvalidVisitLength"), delay: 60 };
const form = saveOnChange(FormView, { visitor: "Ada", building: "North", expected_minutes: 0 });
form.type("1");
form.type("12");
form.type("120");
await run(form.root, 250);
assert.strictEqual(posts().length, 3);
assert.strictEqual(form.draft().expected_minutes, 0, `every save was refused, and the form shows ${form.draft().expected_minutes}`);
"#,
        ),
    );
}

/// The current-draft test compares by value (`stableKey`), not by which change it was. Toggling a
/// value away and back (`5`, `50`, `5`): the first save fails late (no answer from the server),
/// the two after it are accepted. The late failure finds a draft equal to its own and reverts it
/// to the value before all three, over the accepted third save the server now holds.
#[test]
fn a_late_failure_does_not_revert_an_equal_later_accepted_change() {
    let out = stage("revert-aba");
    node(
        &out,
        "revert-aba",
        &script(
            r#"
const { FormView } = out("runtime/composites/form");
let n = 0;
reply = (call) => {
  if (call.method === "GET") return { status: 200, body: '{"rows":[]}' };
  n += 1;
  return n === 1 ? { throws: new TypeError("Failed to fetch"), delay: 80 } : accepted;
};
const form = saveOnChange(FormView, { visitor: "Ada", building: "North", expected_minutes: 0 });
form.type("5");
form.type("50");
form.type("5");
await run(form.root, 250);
assert.strictEqual(posts().length, 3);
assert.strictEqual(JSON.parse(posts()[2].body).expected_minutes, 5);
assert.strictEqual(form.draft().expected_minutes, 5, `the accepted 5 was reverted to ${form.draft().expected_minutes}`);
"#,
        ),
    );
}

/// Correction 1 guards the form's submit only. The confirm's button has no guard: two clicks
/// before the answer sign the visitor out twice on the server.
#[test]
fn a_confirm_clicked_twice_before_its_answer_sends_one_command() {
    let out = stage("confirm-double");
    node(
        &out,
        "confirm-double",
        &script(
            r#"
const { ConfirmView } = out("runtime/composites/confirm");
reply = (call) => call.method === "GET" ? { status: 200, body: '{"rows":[]}' } :
  { status: 200, body: '{"outcome":"departed","published":[]}', delay: 50 };
const app = jsx(ScopeLayer, {
  values: { row: { id: "v-1" } },
  actions: { close: () => undefined, notify: () => undefined },
  children: jsx(ConfirmView, { "data-ui-path": "pages/desk/sections/depart", does: "visit.SignOutVisitor", body: "Sign out?" }),
});
const root = React.__root(app).settle();
const confirm = () => root.find((n) => n.tag === "button" && String(n.props.className).includes("ui-tone-danger"))[0];
confirm().props.onClick();
root.settle();
confirm().props.onClick();
await run(root, 150);
assert.strictEqual(posts().length, 1, "two SignOutVisitor commands were sent");
"#,
        ),
    );
}
