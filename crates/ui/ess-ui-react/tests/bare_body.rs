//! A command route whose `ess-http/1` binding takes one input field as its bare request body
//! (`story:a-command-route-takes-one-input-as-its-bare-request-body`, beyond10x/ess#748): the bound
//! adapter sends that field's value alone as the body, never the input object. The generated
//! runtime is compiled with `tsc` and run under `node` against a scripted `fetch`.

mod support;

use std::path::{Path, PathBuf};

use support::{compile, node};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
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
      - name: register
        component: form
        does: visit.RegisterVisit
        fields: [visitor, building, {field: expected_minutes, as: number}]
";

/// The desk generated against gatepass, its `RegisterVisit` route sending `visitor` bare on
/// `PUT`, as a binding whose route reads `request: {body: input.visitor}` hands it over.
fn generated(name: &str, alias: Option<&str>) -> PathBuf {
    generated_from(name, DESK, |binding| {
        let route = binding
            .components
            .get_mut("pass-service")
            .and_then(|served| served.commands.get_mut("gatepass.visit.RegisterVisit"))
            .expect("the desk sends RegisterVisit");
        route.method = Some("PUT".to_owned());
        route.body = Some(alias.map_or("visitor", |_| "visitor_name").to_owned());
        route.body_alias = alias.map(str::to_owned);
    })
}

/// `document` generated against gatepass, its binding changed by `edit` as an `ess-http/1`
/// binding's routes would change it.
fn generated_from(
    name: &str,
    document: &str,
    edit: impl FnOnce(&mut ess_ui::binding::Binding),
) -> PathBuf {
    let gatepass = root().join("examples/gatepass");
    let sources = vec![
        (
            "system.yaml".to_owned(),
            read(&gatepass.join("system.yaml")),
        ),
        (
            "components.yaml".to_owned(),
            read(&gatepass.join("components.yaml")),
        ),
        (
            "domains/visit.yaml".to_owned(),
            read(&gatepass.join("domains/visit.yaml")),
        ),
    ];
    let document = ess_ui::load_str(document).unwrap_or_else(|error| panic!("{error}"));
    let mut binding = ess_ui_check::binding(&document, &sources)
        .unwrap_or_else(|error| panic!("the document binds: {error}"));
    edit(&mut binding);
    assert!(binding.routed(), "the edited binding is a routed one");
    let out = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("ess-ui-react-bare-body")
        .join(name);
    if out.exists() {
        std::fs::remove_dir_all(&out).expect("the old scratch project is removed");
    }
    ess_ui_react::generate_bound(&document, &root(), &out, Some(&binding))
        .unwrap_or_else(|error| panic!("{error}"));
    compile(&out, &["binding.ts", "runtime/core.tsx", "runtime/data.ts"]);
    out
}

/// A scripted `fetch` that records each call and answers a command as accepted.
const STAGE: &str = r#"
const data = out("runtime/data");
const { httpAdapter, setAuthorization } = data;
const { binding } = out("binding");
const sent = [];
globalThis.fetch = async (url, init) => {
  sent.push({ url: String(url), method: (init && init.method) || "GET", headers: (init && init.headers) || {}, body: init && init.body });
  return { status: 202, ok: true, statusText: "", text: async () => '{"outcome":"registered","published":[]}' };
};
globalThis.document = { querySelector: () => null };
setAuthorization(undefined);
const adapter = httpAdapter(binding);
"#;

#[test]
fn a_bare_body_route_sends_the_fields_value_as_the_body() {
    let out = generated("wire", None);
    node(
        &out,
        "bare-body",
        &format!(
            "{STAGE}\n(async () => {{\n{}\n}})().catch((error) => {{ console.error(error); \
             process.exit(1); }});\n",
            r#"
await adapter.send("visit.RegisterVisit", { visitor: ["Ada", "Lovelace"], building: "B", expected_minutes: 5 });
const call = sent[sent.length - 1];
assert.strictEqual(call.method, "PUT");
assert.deepStrictEqual(JSON.parse(call.body), ["Ada", "Lovelace"], call.body);
assert.strictEqual(call.headers["content-type"], "application/json");
await adapter.send("visit.RegisterVisit", { visitor: "Ada", building: "B" });
assert.strictEqual(sent[sent.length - 1].body, '"Ada"');
await assert.rejects(adapter.send("visit.RegisterVisit", { building: "B" }), /`visitor`/);
"#
        ),
    );
}

#[test]
fn a_bare_body_renamed_on_the_wire_is_read_under_either_name() {
    let out = generated("alias", Some("visitor"));
    node(
        &out,
        "bare-body-alias",
        &format!(
            "{STAGE}\n(async () => {{\n{}\n}})().catch((error) => {{ console.error(error); \
             process.exit(1); }});\n",
            r#"
await adapter.send("visit.RegisterVisit", { visitor_name: "Wire", visitor: "Model" });
assert.strictEqual(sent[sent.length - 1].body, '"Wire"');
await adapter.send("visit.RegisterVisit", { visitor: "Model" });
assert.strictEqual(sent[sent.length - 1].body, '"Model"');
"#
        ),
    );
}

/// The desk with a list of expected visits beside its form: one read and one command.
const DESK_READS: &str = "format: ess-ui/1
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
      - name: register
        component: form
        does: visit.RegisterVisit
        fields: [visitor, building, {field: expected_minutes, as: number}]
";

/// A scripted `fetch` answering each request as `answer(url, init)` says, recording each.
const ANSWERING: &str = r#"
const data = out("runtime/data");
const { httpAdapter, setAuthorization } = data;
const { binding } = out("binding");
const sent = [];
let answer = () => [500, ""];
globalThis.fetch = async (url, init) => {
  const call = { url: String(url), method: (init && init.method) || "GET", headers: (init && init.headers) || {}, body: init && init.body };
  sent.push(call);
  const [status, text] = answer(call);
  return { status, ok: status >= 200 && status < 300, statusText: "", text: async () => text };
};
globalThis.document = { querySelector: () => null };
setAuthorization(undefined);
const adapter = httpAdapter(binding);
"#;

fn script(body: &str) -> String {
    format!(
        "{ANSWERING}\n(async () => {{\n{body}\n}})().catch((error) => {{ console.error(error); \
         process.exit(1); }});\n"
    )
}

/// `RegisterVisit` answered as an `ess-http/1` binding declares: `204` with no body, `200` with
/// the response under `visit`, and its refusal as `{code: <wire name>, ...}` at `409`
/// (beyond10x/ess#723).
#[test]
fn a_command_answer_is_read_as_its_binding_declares_it() {
    use ess_ui::binding::{DeclaredAnswer, DeclaredError};
    let out = generated_from("declared-command", DESK_READS, |binding| {
        let route = binding
            .components
            .get_mut("pass-service")
            .and_then(|served| served.commands.get_mut("gatepass.visit.RegisterVisit"))
            .expect("the desk sends RegisterVisit");
        let code = route
            .errors
            .keys()
            .find(|code| code.ends_with("InvalidVisitLength"))
            .expect("RegisterVisit declares InvalidVisitLength")
            .clone();
        let answer = |status, body: &str, member: Option<&str>, error| DeclaredAnswer {
            status,
            outcome: "registered".to_owned(),
            body: body.to_owned(),
            member: member.map(str::to_owned),
            error,
            text: false,
        };
        route.answers = vec![
            answer(204, "none", None, None),
            answer(200, "response", Some("visit"), None),
            answer(
                409,
                "error",
                Some("code"),
                Some(DeclaredError {
                    wire: "visit_too_short".to_owned(),
                    code,
                }),
            ),
        ];
    });
    node(
        &out,
        "declared-command",
        &script(
            r#"
const input = { visitor: "Ada", building: "North", expected_minutes: 5 };
answer = () => [204, ""];
assert.deepStrictEqual(await adapter.send("visit.RegisterVisit", input), {});
answer = () => [200, '{"visit":{"visit_id":"v-1"},"extra":1}'];
assert.deepStrictEqual(await adapter.send("visit.RegisterVisit", input), { visit_id: "v-1" });
answer = () => [409, '{"code":"visit_too_short","submitted":0}'];
const route = Object.values(binding.components)[0].commands["gatepass.visit.RegisterVisit"];
const code = route.answers[2].error.code;
await assert.rejects(adapter.send("visit.RegisterVisit", input), (error) => {
  assert.deepStrictEqual(error.answer, { answer: "refused", error: code, payload: { submitted: 0 } });
  assert.strictEqual(error.display, route.errors[code].display);
  return true;
});
answer = () => [409, '{"code":"another"}'];
await assert.rejects(adapter.send("visit.RegisterVisit", input), (error) => {
  assert.notStrictEqual(error.answer.answer, "accepted");
  return true;
});
"#,
        ),
    );
}

/// `ExpectedVisits` answered as a bare array, as an array under `items`, and as one row whose
/// empty answer is `404` (beyond10x/ess#723).
#[test]
fn a_view_answer_is_read_as_its_binding_declares_it() {
    let edit = |form: &'static str, member: Option<&'static str>, empty: Option<u16>| {
        move |binding: &mut ess_ui::binding::Binding| {
            let route = binding
                .components
                .get_mut("pass-service")
                .and_then(|served| served.views.get_mut("gatepass.visit.ExpectedVisits"))
                .expect("the desk reads ExpectedVisits");
            route.answer = Some(form.to_owned());
            route.member = member.map(str::to_owned);
            route.empty = empty;
        }
    };
    let array = generated_from("declared-array", DESK_READS, edit("array", None, None));
    node(
        &array,
        "declared-array",
        &script(
            r#"
const read = () => adapter.read({ view: "visit.ExpectedVisits", params: {} });
answer = () => [200, '[{"visitor":"Ada"},{"visitor":"Bo"}]'];
assert.deepStrictEqual(await read(), { rows: [{ visitor: "Ada" }, { visitor: "Bo" }] });
answer = () => [200, '{"rows":[{"visitor":"Ada"}]}'];
await assert.rejects(read(), /not the array/);
"#,
        ),
    );
    let wrapped = generated_from(
        "declared-wrapped",
        DESK_READS,
        edit("array", Some("items"), None),
    );
    node(
        &wrapped,
        "declared-wrapped",
        &script(
            r#"
const read = () => adapter.read({ view: "visit.ExpectedVisits", params: {} });
answer = () => [200, '{"items":[{"visitor":"Ada"}],"next":null}'];
assert.deepStrictEqual(await read(), { rows: [{ visitor: "Ada" }] });
"#,
        ),
    );
    let single = generated_from(
        "declared-single",
        DESK_READS,
        edit("single", None, Some(404)),
    );
    node(
        &single,
        "declared-single",
        &script(
            r#"
const read = () => adapter.read({ view: "visit.ExpectedVisits", params: {} });
answer = () => [200, '{"visitor":"Ada"}'];
assert.deepStrictEqual(await read(), { rows: [{ visitor: "Ada" }] });
answer = () => [404, '{"refused":"no such visit"}'];
assert.deepStrictEqual(await read(), { rows: [] });
answer = () => [500, ""];
await assert.rejects(read(), /500/);
"#,
        ),
    );
}

/// A `401` asks `renew` once for every request it answers at once, each sent once more with the
/// new value; a second `401` stays the answer, read as no credential (beyond10x/ess#723).
#[test]
fn a_call_answered_401_is_sent_once_more_with_a_renewed_credential() {
    let out = generated("renew", None);
    node(
        &out,
        "renew",
        &script(
            r#"
const input = { visitor: "Ada", building: "North" };
let renewals = 0;
answer = (call) => call.headers.authorization === "Bearer test-token-2"
  ? [202, '{"outcome":"registered","published":[]}']
  : [401, '{"refused":"unauthenticated"}'];
setAuthorization("Bearer test-token-1", async () => {
  renewals++;
  await new Promise((resolve) => setTimeout(resolve, 10));
  return "Bearer test-token-2";
});
await Promise.all([adapter.send("visit.RegisterVisit", input), adapter.send("visit.RegisterVisit", input)]);
assert.strictEqual(renewals, 1);
assert.deepStrictEqual(sent.map((call) => call.headers.authorization),
  ["Bearer test-token-1", "Bearer test-token-1", "Bearer test-token-2", "Bearer test-token-2"]);
sent.length = 0;
answer = () => [401, '{"refused":"unauthenticated"}'];
setAuthorization("Bearer test-token-3", async () => "Bearer test-token-4");
await assert.rejects(adapter.send("visit.RegisterVisit", input), (error) => {
  assert.deepStrictEqual(error.answer, { answer: "not_granted", actor: null });
  return true;
});
assert.strictEqual(sent.length, 2);
sent.length = 0;
setAuthorization("Bearer test-token-5", async () => undefined);
await assert.rejects(adapter.send("visit.RegisterVisit", input), (error) => error.answer.answer === "not_granted");
assert.strictEqual(sent.length, 1);
sent.length = 0;
setAuthorization("Bearer test-token-6");
await assert.rejects(adapter.send("visit.RegisterVisit", input), (error) => error.answer.answer === "not_granted");
assert.strictEqual(sent.length, 1);
"#,
        ),
    );
}
