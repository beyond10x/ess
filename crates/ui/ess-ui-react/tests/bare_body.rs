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
    let document = ess_ui::load_str(DESK).unwrap_or_else(|error| panic!("{error}"));
    let mut binding = ess_ui_check::binding(&document, &sources)
        .unwrap_or_else(|error| panic!("the document binds: {error}"));
    let route = binding
        .components
        .get_mut("pass-service")
        .and_then(|served| served.commands.get_mut("gatepass.visit.RegisterVisit"))
        .expect("the desk sends RegisterVisit");
    route.method = Some("PUT".to_owned());
    route.body = Some(alias.map_or("visitor", |_| "visitor_name").to_owned());
    route.body_alias = alias.map(str::to_owned);
    assert!(binding.routed(), "a bare body is a routed binding");
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
