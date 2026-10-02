//! Adversary pass 1 on story:ui-react-live-binding: the bound project against what a person clicks
//! in it. Each case drives the generated runtime (compiled with `tsc`, run under `node` against the
//! stand-ins in `support`) or the generator itself, and asserts what the story, the binding
//! contract or the schema promise.

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
        .join("ess-ui-react-adv-live-1")
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

/// The desk with one live section over the expected list; `reads` and `extra` are spliced in.
fn live_desk(reads: &str, extra: &str) -> String {
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
{extra}    sections:
      - name: expected
        component: collection
        reads: {reads}
        live: {{channel: visits, effect: insert_top}}
"
    )
}

fn gatepass(edit: impl Fn(String) -> String) -> Vec<(String, String)> {
    let root = root().join("examples/gatepass");
    vec![
        ("system.yaml".to_owned(), read(&root.join("system.yaml"))),
        (
            "components.yaml".to_owned(),
            read(&root.join("components.yaml")),
        ),
        (
            "domains/visit.yaml".to_owned(),
            edit(read(&root.join("domains/visit.yaml"))),
        ),
    ]
}

fn load(text: &str) -> Document {
    ess_ui::load_str(text).unwrap_or_else(|error| panic!("the document loads: {error}\n{text}"))
}

fn bind(document: &Document, sources: &[(String, String)]) -> Binding {
    ess_ui_check::binding(document, sources)
        .unwrap_or_else(|error| panic!("the document binds: {error}"))
}

fn bound(text: &str) -> GeneratedFiles {
    let document = load(text);
    let binding = bind(&document, &gatepass(|text| text));
    ess_ui_react::render_bound(&document, &root(), Some(&binding))
        .unwrap_or_else(|error| panic!("{error}"))
}

fn generated(name: &str, text: &str, sources: &[(String, String)]) -> PathBuf {
    let document = load(text);
    let binding = bind(&document, sources);
    let out = scratch(name);
    ess_ui_react::generate_bound(&document, &root(), &out, Some(&binding))
        .unwrap_or_else(|error| panic!("{error}"));
    out
}

/// The generated text that calls `usePoll`, or a panic naming every file.
fn poll_line(files: &GeneratedFiles) -> String {
    files
        .files
        .values()
        .flat_map(|text| text.lines())
        .find(|line| line.contains("usePoll(__read"))
        .map_or_else(
            || panic!("no section polls; files {:?}", files.files.keys()),
            |line| line.trim().to_owned(),
        )
}

// ── generator ───────────────────────────────────────────────────────────────────────────────

/// The story: a live section re-reads at `refresh:` or every 5 s.
#[test]
fn a_bound_live_section_polls_at_its_refresh_or_every_five_seconds() {
    let plain = bound(&live_desk("visit.ExpectedVisits", ""));
    assert_eq!(poll_line(&plain), "const __data = usePoll(__read, 5000);");
    let every = bound(&live_desk("{view: visit.ExpectedVisits, refresh: 2s}", ""));
    assert_eq!(poll_line(&every), "const __data = usePoll(__read, 2000);");
}

/// `refresh: 0s` matches the schema's `duration` pattern (`^[0-9]+(ms|s|m|h)$`); a bound live
/// section must not turn it into `setInterval(…, 0)`, which reads the server as fast as the event
/// loop turns.
#[test]
fn a_zero_refresh_does_not_poll_the_server_in_a_tight_loop() {
    let document = load(&live_desk("{view: visit.ExpectedVisits, refresh: 0s}", ""));
    let binding = bind(&document, &gatepass(|text| text));
    if let Ok(files) = ess_ui_react::render_bound(&document, &root(), Some(&binding)) {
        let line = poll_line(&files);
        assert_ne!(
            line, "const __data = usePoll(__read, 0);",
            "a zero refresh polls with a zero interval"
        );
    }
}

/// `degrades: {no_live: refuse}` on a live section of a bound project refuses generation, by the
/// section's `live` node path.
#[test]
fn a_bound_live_section_that_refuses_to_poll_is_refused_by_its_live_path() {
    let text = live_desk("visit.ExpectedVisits", "").replace(
        "        live: {channel: visits, effect: insert_top}\n",
        "        live: {channel: visits, effect: insert_top}\n        degrades: {no_live: refuse}\n",
    );
    let document = load(&text);
    let binding = bind(&document, &gatepass(|text| text));
    match ess_ui_react::render_bound(&document, &root(), Some(&binding)) {
        Ok(files) => panic!("generated: {}", poll_line(&files)),
        Err(error) => assert!(
            error
                .to_string()
                .contains("pages/desk/sections/expected/live"),
            "{error}"
        ),
    }
}

/// No served surface streams: a bound page whose header shows a channel's lifecycle must not
/// play that channel's fixture script against the real server's data.
#[test]
fn a_bound_page_header_plays_no_fixture_channel() {
    let files = bound(&live_desk(
        "visit.ExpectedVisits",
        "    header: {live: [visits]}\n",
    ));
    let opened: Vec<&str> = files
        .files
        .values()
        .flat_map(|text| text.lines())
        .filter(|line| line.contains("useChannels("))
        .collect();
    assert!(
        !files.files.contains_key("src/runtime/live.ts") && opened.is_empty(),
        "the bound project plays fixture channels: {opened:?}; files {:?}",
        files.files.keys()
    );
}

/// The schema's own example puts `does: session.SignOut` on an account menu entry. A bound
/// document whose account menu names a command the model does not have must be refused, as an
/// action naming one is, instead of generating an app whose menu entry posts nowhere.
#[test]
fn an_account_menu_command_the_model_lacks_is_refused() {
    let text = DESK.replace(
        "shells: {app: {regions: {main: {kind: page_outlet}}}}",
        "shells: {app: {regions: {main: {kind: page_outlet}, account: {kind: account_menu, \
         props: {actions: [{name: out, does: visit.NoSuchCommand, label: Sign out}]}}}}}",
    );
    let document = load(&text);
    let sources = gatepass(|text| text);
    let rendered = ess_ui_check::binding(&document, &sources)
        .map_err(|error| error.to_string())
        .and_then(|binding| {
            ess_ui_react::render_bound(&document, &root(), Some(&binding))
                .map_err(|error| error.to_string())
        });
    if let Ok(files) = rendered {
        let shell = files
            .files
            .iter()
            .find(|(path, _)| path.starts_with("src/shells/"))
            .map(|(_, text)| text.as_str())
            .unwrap_or_default();
        assert!(
            !shell.contains("visit.NoSuchCommand"),
            "bound and generated with an account menu entry posting `visit.NoSuchCommand`"
        );
    }
}

/// The epic excludes CORS: the app is served from the server's origin. The README must not advise
/// allowing cross-origin requests.
#[test]
fn the_bound_readme_advises_no_cross_origin_workaround() {
    let files = bound(DESK);
    let readme = files.files.get("README.md").expect("a README");
    for line in readme.lines() {
        let lower = line.to_lowercase();
        assert!(
            !(lower.contains("proxy") || (lower.contains("cors") && lower.contains("allow"))),
            "the README advises a cross-origin set-up: {line}"
        );
    }
}

/// No `//+bound` … `//-plain` marker reaches a bound project, and a bound project renders the same
/// bytes twice.
#[test]
fn no_template_marker_reaches_a_bound_project() {
    let text = live_desk("{view: visit.ExpectedVisits, refresh: 2s}", "");
    let files = bound(&text);
    for (path, body) in &files.files {
        for line in body.lines() {
            assert!(
                !matches!(
                    line.trim(),
                    "//+bound" | "//-bound" | "//+plain" | "//-plain"
                ),
                "{path} carries a template marker"
            );
        }
    }
    assert_eq!(files, bound(&text));
}

// ── runtime ─────────────────────────────────────────────────────────────────────────────────

/// A stand-in page with a base-URL meta tag, a scripted `fetch`, and helpers.
const STAGE: &str = r#"
const { ScopeLayer } = out("runtime/core");
const data = out("runtime/data");
const { httpAdapter, setDataAdapter, setAuthorization, useRead } = data;
const { binding } = out("binding");
const sent = [];
let inFlight = 0;
let peak = 0;
// `reply(call)` answers each call: { status, body, delay }.
let reply = (call) => (call.method === "GET" ? { status: 200, body: '{"rows":[]}' } : { status: 500, body: "{}" });
globalThis.fetch = async (url, init) => {
  const call = { url: String(url), method: (init && init.method) || "GET", headers: (init && init.headers) || {}, body: init && init.body };
  sent.push(call);
  const answer = reply(call);
  if (answer.throws) throw answer.throws;
  inFlight += 1;
  peak = Math.max(peak, inFlight);
  await new Promise((resolve) => setTimeout(resolve, answer.delay || 0));
  inFlight -= 1;
  return { status: answer.status, ok: answer.status < 300, statusText: "", text: async () => answer.body };
};
let metas = { "ess-base-url:pass-service": "http://desk.test" };
globalThis.document = {
  querySelector: (selector) => {
    const name = /^meta\[name="(.*)"\]$/.exec(selector);
    return name && metas[name[1]] !== undefined ? { getAttribute: () => metas[name[1]] } : null;
  },
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
const unhandled = [];
process.on("unhandledRejection", (error) => unhandled.push(String(error && error.message ? error.message : error)));
"#;

fn stage(name: &str, modules: &[&str]) -> PathBuf {
    let out = generated(name, DESK, &gatepass(|text| text));
    let mut all = vec!["binding.ts", "runtime/core.tsx", "runtime/data.ts"];
    all.extend_from_slice(modules);
    compile(&out, &all);
    out
}

fn script(body: &str) -> String {
    format!(
        "{STAGE}\n(async () => {{\n{body}\n}})().catch((error) => {{ console.error(error); \
         process.exit(1); }});\n"
    )
}

/// A polled read stops reading when its section unmounts.
#[test]
fn a_poll_stops_when_its_section_unmounts() {
    let out = stage("poll-unmount", &[]);
    node(
        &out,
        "poll-unmount",
        &script(
            r#"
let mounted = true;
function Poller() {
  const read = data.usePoll(useRead({ view: "visit.ExpectedVisits" }, {}), 30);
  return jsx("span", { children: read.status });
}
function Host() {
  return mounted ? jsx(Poller, {}) : null;
}
const root = React.__root(jsx(Host, {})).settle();
await run(root, 250);
const polled = gets().length;
assert.ok(polled >= 4, `polled ${polled} times in 250 ms at 30 ms`);
mounted = false;
root.settle();
const after = gets().length;
await run(root, 250);
assert.strictEqual(gets().length, after, "reads continue after the section unmounted");
"#,
        ),
    );
}

/// A server slower than the poll interval: every tick drops the read in flight and starts another,
/// so the section never shows what the server answered, and requests pile up on the server.
#[test]
fn a_poll_against_a_server_slower_than_its_interval_still_shows_the_answer() {
    let out = stage("poll-slow", &[]);
    node(
        &out,
        "poll-slow",
        &script(
            r#"
reply = () => ({ status: 200, body: '{"rows":[{"visit_id":"v-1"}]}', delay: 90 });
const seen = new Set();
function Poller() {
  const read = data.usePoll(useRead({ view: "visit.ExpectedVisits" }, {}), 40);
  seen.add(read.status);
  return jsx("span", { children: read.status });
}
const root = React.__root(jsx(Poller, {})).settle();
await run(root, 800);
assert.ok(seen.has("ready"), `the section never showed the answer: statuses ${[...seen]}, ${gets().length} reads sent`);
assert.ok(peak <= 1, `${peak} reads of one section in flight at once`);
"#,
        ),
    );
}

/// Reads: `GET`, the bound path, view parameters under their wire names, scalars written as text,
/// absent and `null` omitted; `Authorization` only when set; the base URL from the component's
/// meta tag without its trailing slash, the page's own origin when the tag is absent.
#[test]
fn requests_carry_the_bound_wire_shapes() {
    let out = stage("requests", &[]);
    node(
        &out,
        "requests",
        &script(
            r#"
const custom = {
  system: "s",
  names: { v: "s.V", c: "s.C" },
  components: {
    svc: {
      views: { "s.V": { path: "/x/views/v", params: [
        { name: "who", wire: "who-key", required: true, scalar: "String" },
        { name: "on", wire: "on", required: false, scalar: "Boolean" },
        { name: "off", wire: "off", required: false, scalar: "Boolean" },
        { name: "n", wire: "n", required: false, scalar: "Integer" },
        { name: "gone", wire: "gone", required: false, scalar: "String" },
        { name: "nil", wire: "nil", required: false, scalar: "String" },
      ] } },
      commands: { "s.C": { path: "/x/commands/c", body_required: true, errors: {} } },
    },
  },
};
metas = { "ess-base-url:svc": "http://api.test/base/" };
setAuthorization(undefined);
const adapter = httpAdapter(custom);
await adapter.read({ view: "v", params: { who: "Ada Lovelace ü&=", on: true, off: false, n: 0, nil: null } });
const first = sent[sent.length - 1];
assert.strictEqual(first.method, "GET");
const url = new URL(first.url);
assert.strictEqual(url.origin + url.pathname, "http://api.test/base/x/views/v");
assert.strictEqual(url.searchParams.get("who-key"), "Ada Lovelace ü&=");
assert.strictEqual(url.searchParams.get("on"), "true");
assert.strictEqual(url.searchParams.get("off"), "false");
assert.strictEqual(url.searchParams.get("n"), "0");
assert.ok(!url.searchParams.has("gone") && !url.searchParams.has("nil") && !url.searchParams.has("who"), first.url);
assert.ok(!("authorization" in first.headers), JSON.stringify(first.headers));
reply = () => ({ status: 200, body: '{"outcome":"done","published":[]}' });
setAuthorization("Bearer t");
metas = {};
await adapter.send("c", { a: 1 });
const second = sent[sent.length - 1];
assert.strictEqual(second.method, "POST");
assert.strictEqual(second.url, "/x/commands/c");
assert.deepStrictEqual(JSON.parse(second.body), { a: 1 });
assert.strictEqual(second.headers.authorization, "Bearer t");
assert.strictEqual(second.headers["content-type"], "application/json");
"#,
        ),
    );
}

/// Answers outside `answers.json`: no answer at all, an aborted request, an empty body, a `202`
/// carrying an `error`, an HTML error page — each rejects with a refusal, none resolves as done.
#[test]
fn a_command_answer_outside_the_vectors_is_never_taken_as_done() {
    let out = stage("answers", &[]);
    node(
        &out,
        "answers",
        &script(
            r#"
const cases = [
  ["network", { throws: new TypeError("Failed to fetch") }, "transport"],
  ["abort", { throws: Object.assign(new Error("aborted"), { name: "AbortError" }) }, "transport"],
  ["empty 200", { status: 200, body: "" }, "transport"],
  ["empty 204", { status: 204, body: "" }, "transport"],
  ["202 with error", { status: 202, body: '{"outcome":"x","published":[],"error":"gatepass.visit.InvalidVisitLength"}' }, "transport"],
  ["html 502", { status: 502, body: "<html>bad gateway</html>" }, "transport"],
  ["403 nobody", { status: 403, body: '{"refused":"no grant","actor":null}' }, "not_granted"],
];
for (const [name, answer, expected] of cases) {
  reply = () => answer;
  let outcome;
  try {
    await data.dataAdapter().send("visit.RegisterVisit", {});
    outcome = "resolved";
  } catch (error) {
    outcome = error instanceof data.CommandRefused ? error.answer.answer : `other ${error}`;
  }
  assert.strictEqual(outcome, expected, name);
}
"#,
        ),
    );
}

/// The declared error's display text, looked up by its wire code in the binding, not the wire code
/// itself, is what a refusal carries. (Gatepass declares no error `naming.display`, and it needs
/// `format: ess/4`, so the binding is written here.)
#[test]
fn a_refusal_carries_the_errors_display_text_not_its_code() {
    let out = stage("display", &[]);
    node(
        &out,
        "display",
        &script(
            r#"
const custom = {
  system: "s",
  names: { c: "s.C" },
  components: { svc: { views: {}, commands: { "s.C": { path: "/c", body_required: true, errors: {
    "too-short": { status: 422, display: "The visit is too short" },
  } } } } },
};
reply = () => ({ status: 422, body: '{"outcome":"refused","published":[],"error":"too-short","payload":{"submitted":0}}' });
let refused;
try {
  await httpAdapter(custom, () => "").send("c", {});
} catch (error) {
  refused = error;
}
assert.ok(refused instanceof data.CommandRefused, String(refused));
assert.strictEqual(refused.display, "The visit is too short");
assert.deepStrictEqual(refused.answer, { answer: "refused", error: "too-short", payload: { submitted: 0 } });
"#,
        ),
    );
}

/// Submitting twice before the first answer arrives registers the visit twice on the server.
#[test]
fn a_form_submitted_twice_before_its_answer_sends_one_command() {
    let out = stage("double", &["runtime/composites/form.tsx"]);
    node(
        &out,
        "double",
        &script(
            r#"
const { FormView } = out("runtime/composites/form");
reply = (call) => call.method === "GET" ? { status: 200, body: '{"rows":[]}' } :
  { status: 200, body: '{"outcome":"registered","published":[]}', delay: 50 };
const app = jsx(ScopeLayer, {
  values: { draft: { visitor: "Ada", building: "North", expected_minutes: 30 } },
  setters: { draft: () => undefined },
  actions: { close: () => undefined, notify: () => undefined },
  children: jsx(FormView, { "data-ui-path": "pages/desk/sections/register", does: "visit.RegisterVisit", submit: { closes: false } }),
});
const root = React.__root(app).settle();
const form = () => root.find((n) => n.tag === "form")[0];
form().props.onSubmit({ preventDefault() {} });
root.settle();
form().props.onSubmit({ preventDefault() {} });
await run(root, 150);
assert.strictEqual(posts().length, 1, "two RegisterVisit commands were sent");
"#,
        ),
    );
}

/// Two saves on change in quick succession: the first is refused after the second was accepted.
/// The revert must not throw away the accepted second value, which the server now holds.
#[test]
fn a_late_refusal_does_not_revert_a_later_accepted_change() {
    let out = stage("revert-race", &["runtime/composites/form.tsx"]);
    node(
        &out,
        "revert-race",
        &script(
            r#"
const { FormView } = out("runtime/composites/form");
let n = 0;
reply = (call) => {
  if (call.method === "GET") return { status: 200, body: '{"rows":[]}' };
  n += 1;
  return n === 1
    ? { status: 422, body: '{"outcome":"refused","published":[],"error":"gatepass.visit.InvalidVisitLength","payload":{"submitted":5}}', delay: 60 }
    : { status: 200, body: '{"outcome":"registered","published":[]}', delay: 0 };
};
const initial = { visitor: "Ada", building: "North", expected_minutes: 0 };
let draft = initial;
function Host() {
  const [value, setValue] = React.useState(initial);
  draft = value;
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
const input = () => root.find((n) => n.tag === "fieldset")[0];
const field = () => {
  const found = [];
  const visit = (nodes) => nodes.forEach((x) => { if (typeof x === "object") { if (x.tag === "input") found.push(x); if (x.children) visit(x.children); } });
  visit(input().children);
  return found[0];
};
field().props.onChange({ target: { value: "5" } });
root.settle();
field().props.onChange({ target: { value: "50" } });
root.settle();
await run(root, 200);
assert.strictEqual(posts().length, 2);
assert.strictEqual(JSON.parse(posts()[1].body).expected_minutes, 50);
assert.strictEqual(draft.expected_minutes, 50, `the accepted 50 was reverted to ${draft.expected_minutes}`);
"#,
        ),
    );
}

/// A refused account-menu command: the refusal is shown (beside the entry, or at least as a
/// notice), and is not an unhandled rejection nobody sees.
#[test]
fn a_refused_account_menu_command_is_not_lost() {
    let out = stage("menu", &["runtime/shell.tsx"]);
    node(
        &out,
        "menu",
        &script(
            r#"
const { AccountMenu } = out("runtime/shell");
reply = (call) => call.method === "GET" ? { status: 200, body: '{"rows":[]}' } :
  { status: 409, body: '{"outcome":"wrong-state","published":[],"error":"gatepass.visit.VisitStateConflict","payload":{"state":"Departed"}}' };
const notices = [];
const app = jsx(ScopeLayer, {
  values: { actor: { name: "Ada" } },
  actions: { notify: (message) => notices.push(message) },
  children: jsx(AccountMenu, { actions: [{ name: "out", label: "Sign out", does: "visit.SignOutVisitor" }] }),
});
const root = React.__root(app).settle();
root.find((n) => n.tag === "button")[0].props.onClick();
root.settle();
const entry = root.find((n) => n.tag === "button" && n.props.className === "ui-menu-item")[0];
entry.props.onClick();
await run(root, 80);
assert.strictEqual(posts().length, 1);
assert.deepStrictEqual(unhandled, [], "the refusal is an unhandled rejection");
const shown = root.find((n) => n.props && n.props["data-ui-refusal"] !== undefined).length > 0;
assert.ok(shown || notices.some((m) => m.includes("VisitStateConflict")), `nothing shown; notices ${JSON.stringify(notices)}`);
"#,
        ),
    );
}

/// The port against its source on bodies outside `answers.json`: each `(status, body)` is
/// classified by `ess_ui::binding::classify` here and by `runtime/answer.ts` under node.
/// A lone surrogate escape and 200-deep nesting were dropped: `JSON.parse` and `serde_json`
/// differ on them, and no served server emits either (review-result pass 1, the note).
#[test]
fn the_typescript_classifier_agrees_with_classify_beyond_the_vectors() {
    let cases: Vec<(u16, String)> = vec![
        (
            422,
            r#"{"outcome":"r","error":"e","payload":null}"#.to_owned(),
        ),
        (422, r#"{"outcome":"r","error":null}"#.to_owned()),
        (200, r#"{"outcome":"a","outcome":1}"#.to_owned()),
        (403, r#"{"refused":"x","actor":7}"#.to_owned()),
        (501, r#"{"refused":"x","committed":"yes"}"#.to_owned()),
        (200, " \n{\"outcome\":\"a\"}\t".to_owned()),
    ];
    let expected: Vec<serde_json::Value> = cases
        .iter()
        .map(|(status, body)| {
            serde_json::json!({
                "status": status,
                "body": body,
                "expected": ess_ui::binding::classify(*status, body),
            })
        })
        .collect();
    let out = generated("differential", DESK, &gatepass(|text| text));
    compile(&out, &["runtime/answer.ts"]);
    node(
        &out,
        "differential",
        &format!(
            "const {{ classify }} = out(\"runtime/answer\");\nconst cases = {};\n\
             const wrong = cases.filter((c) => {{ try {{ assert.deepStrictEqual(classify(c.status, c.body), c.expected); return false; }} catch {{ return true; }} }})\n\
             .map((c) => `${{c.status}} ${{c.body.slice(0, 60)}}: ts ${{JSON.stringify(classify(c.status, c.body))}}, rust ${{JSON.stringify(c.expected)}}`);\n\
             assert.deepStrictEqual(wrong, []);\n",
            serde_json::to_string(&expected).expect("the cases serialise")
        ),
    );
}
