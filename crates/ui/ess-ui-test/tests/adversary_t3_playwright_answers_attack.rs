//! Adversary pass on the routes the Playwright spec emits for declared fixture answers. The suite
//! of the unit matches substrings of the spec; these cases run the emitted route handlers under
//! `node`, against a stand-in `page` and `route`, and assert what each request is answered with,
//! as the terminal answers the same declaration.

use std::path::{Path, PathBuf};
use std::process::Command;

const DOCUMENT: &str = r"format: ess-ui/1
app: answers
model: t
placement_profile: fat
shells: {app: {regions: {main: {kind: page_outlet}}}}
navigation: {home: orders.detail, sections: [{name: all, pages: [orders.detail]}]}
pages:
  orders.detail:
    kind: detail_page
    title: Order
    params: {id: string}
    header:
      actions:
        - {name: cancel, label: Cancel, does: example.order.Cancel, bind: {order: params.id}}
    sections:
      - {name: other, component: record, reads: {view: example.order.Page, params: {order: params.id}}, fields: [id, customer]}
      - {name: lines, component: collection, reads: {view: example.line.Page}, columns: [id]}
      - {name: archive, component: collection, reads: {view: example.order.Archive}, columns: [id]}
";

const STEPS: &str = "    steps:
      - open: {page: orders.detail, params: {id: o-3}}
";

/// Stands in for Playwright: `page.route` registers, the last registered matching handler answers
/// a request (as Playwright tries handlers in reverse order), `page.evaluate` records the delay,
/// `route.fallback()` hands the request to the next earlier matching handler.
const HARNESS: &str = r#"
const routes = [];
const delays = [];
const page = {
  route(matcher, handler) { routes.push([matcher, handler]); },
  async evaluate(fn, ms) { delays.push(ms); },
};
function globMatches(glob, href) {
  const pattern = glob.split("**").map((part) => part.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")).join(".*");
  return new RegExp("^" + pattern + "$").test(href);
}
async function ask(href, body) {
  const url = new URL(href);
  for (let index = routes.length - 1; index >= 0; index--) {
    const [matcher, handler] = routes[index];
    const hit = typeof matcher === "function" ? matcher(url) : globMatches(matcher, href);
    if (!hit) continue;
    let answer = { kind: "unanswered" };
    let fellBack = false;
    const route = {
      request() { return { url: () => href, postDataJSON: () => body ?? null }; },
      async fulfill(options) { answer = { kind: "fulfill", status: options.status ?? 200, json: options.json ?? null, body: options.body ?? null }; },
      async continue() { answer = { kind: "continue" }; },
      async fallback() { fellBack = true; },
    };
    await handler(route);
    if (fellBack) continue;
    return answer;
  }
  return { kind: "network" };
}
(async () => {
  //SETUP
  const asks = JSON.parse(process.argv[2]);
  const answers = [];
  for (const [href, body] of asks) answers.push(await ask(href, body));
  console.log(JSON.stringify({ answers, delays }));
})();
"#;

fn file(tests: &str) -> ess_ui_test::TestFile {
    let text = format!("format: ess-ui-test/1\ndocument: ui.yaml\ntests:\n{tests}");
    ess_ui_test::parse_str(&text, Path::new("answers.test.yaml")).unwrap()
}

fn document() -> ess_ui::Document {
    ess_ui::load_str(DOCUMENT).unwrap()
}

fn plain(tests: &str) -> String {
    ess_ui_test::playwright(&[file(tests)], &document())
}

fn bound(tests: &str, binding: &str) -> String {
    let binding: ess_ui::binding::Binding = serde_json::from_str(binding).expect("binding");
    ess_ui_test::playwright_bound(&[file(tests)], &document(), &binding)
}

/// The lines a test runs before its clock is installed (its routes), as JavaScript.
fn setup(spec: &str) -> String {
    let start = spec.find("async ({ page }) => {").expect("a test") + "async ({ page }) => {".len();
    let end = spec
        .find("await page.clock.install")
        .expect("the clock line");
    spec[start..end]
        .replace(
            "const commands: { command: string; input: unknown }[] = [];",
            "const commands = [];",
        )
        .replace(
            "const refusals: Record<string, unknown> =",
            "const refusals =",
        )
}

fn scratch(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("ess-ui-test-adversary-t3")
        .join(name);
    std::fs::create_dir_all(&dir).expect("the scratch dir is created");
    dir
}

/// Runs the test's routes and answers each `(url, body)` in turn.
fn drive(name: &str, spec: &str, asks: &serde_json::Value) -> serde_json::Value {
    let dir = scratch(name);
    let script = dir.join("routes.cjs");
    std::fs::write(&script, HARNESS.replace("//SETUP", &setup(spec))).expect("written");
    let output = Command::new("node")
        .arg(&script)
        .arg(asks.to_string())
        .output()
        .unwrap_or_else(|error| panic!("node is on PATH: {error}"));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success(),
        "{stdout}{}\n\n{spec}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_str(stdout.trim()).unwrap_or_else(|error| panic!("{error}: {stdout}"))
}

fn view_url(view: &str, params: &serde_json::Value) -> String {
    let encoded: String = params
        .to_string()
        .bytes()
        .map(|byte| {
            if byte.is_ascii_alphanumeric() {
                (byte as char).to_string()
            } else {
                format!("%{byte:02X}")
            }
        })
        .collect();
    format!("http://app.test/views/{view}?params={encoded}")
}

// --- Pins: the unit's handlers do what the acceptance says (green, pinned under node). ---

#[test]
fn a_list_of_answers_chooses_by_params_else_the_default_else_no_rows() {
    let spec = plain(&format!(
        "  - name: by params
    fixtures:
      views:
        example.order.Page:
          - {{by_params: {{order: o-3}}, rows: [{{id: o-3}}]}}
          - {{rows: [{{id: o-0}}]}}
        example.line.Page:
          - {{by_params: {{order: o-3}}, rows: [{{id: l-1}}], total: 9}}
{STEPS}"
    ));
    let out = drive(
        "list",
        &spec,
        &serde_json::json!([
            [
                view_url("example.order.Page", &serde_json::json!({"order": "o-3"})),
                null
            ],
            [
                view_url("example.order.Page", &serde_json::json!({"order": "o-9"})),
                null
            ],
            [view_url("example.order.Page", &serde_json::json!({})), null],
            [
                view_url("example.line.Page", &serde_json::json!({"order": "o-3"})),
                null
            ],
            [
                view_url("example.line.Page", &serde_json::json!({"order": "o-4"})),
                null
            ],
        ]),
    );
    let json = |index: usize| out["answers"][index]["json"].clone();
    assert_eq!(
        json(0),
        serde_json::json!({"rows": [{"id": "o-3"}]}),
        "{out}"
    );
    assert_eq!(
        json(1),
        serde_json::json!({"rows": [{"id": "o-0"}]}),
        "{out}"
    );
    assert_eq!(
        json(2),
        serde_json::json!({"rows": [{"id": "o-0"}]}),
        "{out}"
    );
    assert_eq!(
        json(3),
        serde_json::json!({"rows": [{"id": "l-1"}], "total": 9}),
        "{out}"
    );
    assert_eq!(json(4), serde_json::json!({"rows": []}), "{out}");
}

#[test]
fn a_failing_view_and_a_refusal_carry_their_text_exactly() {
    let message = "it's \\\"down\\\" \\\\ ${now} </script>";
    let spec = plain(&format!(
        "  - name: quoting
    latency: {{example.order.Page: 250ms}}
    fixtures:
      views:
        example.order.Page: {{fails: \"{message}\"}}
      commands:
        example.order.Cancel: {{refuses: \"a\\\"b\", payload: {{why: \"x'y\"}}}}
{STEPS}"
    ));
    let out = drive(
        "quoting",
        &spec,
        &serde_json::json!([
            [view_url("example.order.Page", &serde_json::json!({})), null],
            ["http://app.test/commands/example.order.Cancel", {"order": "o-3"}],
            ["http://app.test/commands/example.order.Create", {}],
        ]),
    );
    assert_eq!(out["answers"][0]["status"], 500, "{out}");
    assert_eq!(
        out["answers"][0]["body"], "it's \"down\" \\ ${now} </script>",
        "{out}"
    );
    assert_eq!(out["delays"], serde_json::json!([250]), "{out}");
    assert_eq!(out["answers"][1]["status"], 422, "{out}");
    assert_eq!(
        out["answers"][1]["json"],
        serde_json::json!({"outcome": "refused", "published": [], "error": "a\"b", "payload": {"why": "x'y"}}),
        "{out}"
    );
    assert_eq!(
        out["answers"][2]["json"],
        serde_json::json!({"accepted": true}),
        "{out}"
    );
}

// --- Attacks. ---

/// The story's own reproduction is the unbound spec (`ess ui test --playwright` without
/// `--model`), whose header says to run the app with `setDataAdapter(httpAdapter(<base>))`, and
/// the reference says the 422 body is "the body the HTTP data adapter reads as a refusal". The
/// plain project's `httpAdapter` reads no body of a failed command: `answer()` throws
/// `DataError("422 Unprocessable Entity")`, and the action shows `<command> failed: 422 …`, where
/// the terminal shows the refusal's error beside the action.
#[test]
fn the_unbound_refusal_body_is_read_as_a_refusal_by_the_plain_http_adapter() {
    let spec = plain(&format!(
        "  - name: refuses
    fixtures:
      commands:
        example.order.Cancel: {{refuses: already_cancelled}}
{STEPS}"
    ));
    assert!(spec.contains("status: 422"), "{spec}");
    assert!(!spec.contains("test.fixme"), "{spec}");
    let project = ess_ui_react::render(&document(), Path::new(".")).expect("renders");
    let data = &project.files["src/runtime/data.ts"];
    let adapter = &data[data
        .find("export function httpAdapter")
        .expect("httpAdapter")..];
    assert!(
        adapter.contains("refused") || adapter.contains("classify("),
        "the plain httpAdapter the unbound spec runs against reads no refusal from a 422 body:\n{adapter}"
    );
}

/// An `ess-http/1` binding carries a path variable (beyond10x/ess#713): the app sends
/// `POST /orders/o-3/cancel`, and the spec's matcher is `endsWith("/orders/{order}/cancel")`, so
/// the declared refusal is never answered and the command reaches the network.
#[test]
#[ignore = "pre-existing: bound matchers compare a {var} path literally; backlog"]
fn a_refusal_on_a_command_with_a_path_variable_is_answered() {
    let binding = r#"{"system": "t", "components": {"orders": {
        "views": {"example.order.Page": {"path": "/orders/{order}", "params": [{"name": "order", "wire": "order", "required": true, "scalar": "String"}], "path_params": {"order": "order"}}},
        "commands": {"example.order.Cancel": {"path": "/orders/{order}/cancel", "path_params": {"order": "order"}, "body_required": false,
          "errors": {"already_cancelled": {"status": 409, "display": "Already cancelled"}}}}}},
        "names": {}}"#;
    let spec = bound(
        &format!(
            "  - name: refuses
    fixtures:
      commands:
        example.order.Cancel: {{refuses: already_cancelled}}
{STEPS}"
        ),
        binding,
    );
    let out = drive(
        "path-variable",
        &spec,
        &serde_json::json!([["http://app.test/orders/o-3/cancel", {}]]),
    );
    assert_eq!(out["answers"][0]["status"], 422, "{out}\n\n{spec}");
    assert_eq!(
        out["answers"][0]["json"]["error"], "already_cancelled",
        "{out}"
    );
}

/// Two bound views where one path ends the other (`/orders`, `/archive/orders`): the test
/// replaces the archive and delays `example.order.Page`. The delay-only route, registered last,
/// also matches `/archive/orders`, holds it, then `route.continue()`s it to the network past the
/// replacement. The terminal answers the archive from the replacement.
#[test]
fn a_delayed_view_does_not_take_over_a_replaced_view_whose_path_it_ends() {
    let binding = r#"{"system": "t", "components": {"orders": {
        "views": {
          "example.order.Page": {"path": "/orders", "params": []},
          "example.order.Archive": {"path": "/archive/orders", "params": []}},
        "commands": {}}},
        "names": {}}"#;
    let spec = bound(
        &format!(
            "  - name: overlap
    latency: {{example.order.Page: 1s}}
    fixtures:
      views:
        example.order.Archive: {{rows: [{{id: a-1}}]}}
{STEPS}"
        ),
        binding,
    );
    let out = drive(
        "overlap",
        &spec,
        &serde_json::json!([["http://app.test/archive/orders", null]]),
    );
    assert_eq!(out["answers"][0]["kind"], "fulfill", "{out}\n\n{spec}");
    assert_eq!(
        out["answers"][0]["json"],
        serde_json::json!({"rows": [{"id": "a-1"}]}),
        "{out}"
    );
}

/// The bound `httpAdapter` sends an object or list param as JSON (`query.set(wire,
/// JSON.stringify(value))`); the handler compares `String(params[key])` (`["open","held"]`) to
/// `String(value)` (`open,held`), so a `by_params` list never matches bound. The terminal compares
/// both as display text (`open, held`) and answers the matching entry.
#[test]
fn a_bound_list_param_chooses_its_answer_as_the_terminal_does() {
    let binding = r#"{"system": "t", "components": {"orders": {
        "views": {"example.line.Page": {"path": "/lines", "params": [{"name": "status", "wire": "status", "required": false, "scalar": "String"}]}},
        "commands": {}}},
        "names": {}}"#;
    let spec = bound(
        &format!(
            "  - name: list param
    fixtures:
      views:
        example.line.Page:
          - {{by_params: {{status: [open, held]}}, rows: [{{id: l-1}}]}}
          - {{rows: [{{id: l-0}}]}}
{STEPS}"
        ),
        binding,
    );
    let out = drive(
        "list-param",
        &spec,
        &serde_json::json!([[
            "http://app.test/lines?status=%5B%22open%22%2C%22held%22%5D",
            null
        ]]),
    );
    assert_eq!(
        out["answers"][0]["json"],
        serde_json::json!({"rows": [{"id": "l-1"}]}),
        "{out}\n\n{spec}"
    );
}
