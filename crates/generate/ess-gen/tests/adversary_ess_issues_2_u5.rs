//! Adversarial cases against the `OpenAPI` projection of outcomes declared `alike` and of a route's
//! own `refusals:` (`docs/design/http-presentation-binding.md`, "What `OpenAPI` writes (k4)").
//!
//! A group declared `alike` is written as one response with one schema, the first member's. That
//! holds only while every member's body matches that schema.

use ess_compiler::resolve::compile as compile_model;
use ess_compiler::source::SourceMap;
use ess_compiler::EssIr;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_gen::artifact::run;
use ess_gen::http::{derived, Binding};
use ess_gen::openapi::BoundOpenApi;
use serde_json::{json, Value};

const NOTES: &str = include_str!("../../../specify/ess-http/tests/fixtures/notes.yaml");

const ORDERS: &str = include_str!("../../../specify/ess-http/tests/fixtures/orders.yaml");

fn model_of(file: &str, text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).expect("well formed");
    let spec = Specification::assemble([(Source::new(file), raw)])
        .unwrap_or_else(|errors| panic!("validates: {errors}"));
    let mut sources = SourceMap::new();
    sources.insert(file, text);
    compile_model(&spec, &sources).unwrap_or_else(|errors| panic!("compiles: {errors}"))
}

fn document(ir: &EssIr, system: &str, component: &str, routes: &str) -> Value {
    let text = format!(
        "type: ess-http/1\nspecification:\n  system: {system}\n  version: v1\n  source_digest: sha256:{}\n{routes}",
        ir.source_digest()
    );
    let spec = ess_http::HttpSpec::from_yaml(&text).expect("parses");
    let http = ess_http::compile(&spec, ir, &derived(ir))
        .unwrap_or_else(|diagnostics| panic!("compiles: {diagnostics}"));
    let artifacts = run(
        &BoundOpenApi(Binding::new(http).expect("every route has a path")),
        ir,
    )
    .expect("generates");
    let yaml: ess_yaml::Value =
        ess_yaml::from_str(&artifacts[&format!("openapi/{component}.yaml")].contents)
            .expect("YAML");
    serde_json::to_value(yaml).expect("JSON")
}

fn resolved<'a>(document: &'a Value, schema: &Value) -> &'a Value {
    let pointer = schema["$ref"].as_str().expect("a reference");
    let key = pointer
        .strip_prefix("#/components/schemas/")
        .expect("into components");
    let found = &document["components"]["schemas"][key];
    assert!(!found.is_null(), "{pointer} resolves to nothing");
    found
}

/// Every object schema `schema` may be: itself, or each member of its `oneOf`, references followed.
fn alternatives<'a>(document: &'a Value, schema: &'a Value) -> Vec<&'a Value> {
    let members: Vec<&Value> = match schema.get("oneOf").and_then(Value::as_array) {
        Some(members) => members.iter().collect(),
        None => vec![schema],
    };
    members
        .into_iter()
        .map(|member| {
            if member.get("$ref").is_some() {
                resolved(document, member)
            } else {
                member
            }
        })
        .collect()
}

/// Whether the closed object schema `schema` admits `body`: every required member present, every
/// member declared, each `const` equal.
fn admits(schema: &Value, body: &Value) -> bool {
    let Some(object) = body.as_object() else {
        return false;
    };
    let properties = schema["properties"]
        .as_object()
        .cloned()
        .unwrap_or_default();
    let required = schema["required"].as_array().cloned().unwrap_or_default();
    required
        .iter()
        .all(|name| object.contains_key(name.as_str().unwrap_or_default()))
        && object
            .iter()
            .all(|(name, value)| match properties.get(name) {
                None => schema["additionalProperties"] != json!(false),
                Some(property) => property
                    .get("const")
                    .is_none_or(|constant| constant == value),
            })
}

/// `negative` reports `Refused`, `blank` reports `Declined`; both travel as `refused`, and each
/// carries one `String` field travelling as `detail` (`Declined`'s is named `why`). Their error
/// objects are one body, so `ess-http` admits the group `alike` (errors whose fields differ are
/// refused, `ess-http`'s `alike_error_objects_whose_errors_carry_different_fields_are_refused`).
/// The one schema the `409` response publishes must admit what each member answers.
#[test]
fn an_alike_response_admits_every_members_body() {
    let model = NOTES
        .replace(
            "  - name: demo.notes.Refused\n    naming: {wire: refused}\n",
            "  - name: demo.notes.Refused\n    naming: {wire: refused}\n    fields:\n      - {name: detail, type: String}\n  - name: demo.notes.Declined\n    naming: {wire: refused}\n    fields:\n      - {name: why, type: String, naming: {wire: detail}}\n",
        )
        .replace(
            "      - name: blank\n        when: size == 0\n        error: demo.notes.Refused\n",
            "      - name: blank\n        when: size == 0\n        error: demo.notes.Declined\n",
        );
    assert_ne!(model, NOTES);
    let ir = model_of("notes.yaml", &model);
    let document = document(
        &ir,
        "demo",
        "notes-service",
        "routes:\n  - command: demo.notes.CreateNote\n    method: post\n    path: /api/notes\n    outcomes:\n      negative: {status: 409, body: {error: error}}\n      blank: {status: 409, body: {error: error}}\n    alike: [[negative, blank]]\n",
    );
    let response = &document["paths"]["/api/notes"]["post"]["responses"]["409"];
    assert_eq!(response["x-ess-outcomes"], json!(["negative", "blank"]));
    let schema = &response["content"]["application/json"]["schema"];
    assert!(schema.get("oneOf").is_none(), "one schema: {schema}");
    // What `negative` and `blank` answer; a body under another member is not admitted.
    for (body, admitted) in [
        (json!({"error": "refused", "detail": "negative"}), true),
        (json!({"error": "refused", "detail": "empty"}), true),
        (json!({"error": "refused", "why": "empty"}), false),
    ] {
        assert_eq!(
            alternatives(&document, schema)
                .into_iter()
                .any(|alternative| admits(alternative, &body)),
            admitted,
            "{body} against the `409` schema {schema} -> {:#?}",
            alternatives(&document, schema)
                .into_iter()
                .cloned()
                .collect::<Vec<_>>()
        );
    }
}

/// A view route's own `not_granted` stands on that operation alone; another view keeps the
/// binding's.
#[test]
fn a_view_routes_own_refusal_stands_on_that_operation_alone() {
    let model = ORDERS.replace(
        "components:\n",
        "actors:\n  - name: catalog.orders.Clerk\n    may: [catalog.orders.OrderById, catalog.orders.OrdersOfSize]\ncomponents:\n",
    );
    let ir = model_of("orders.yaml", &model);
    let document = document(
        &ir,
        "catalog",
        "orders-service",
        "refusals:\n  not_granted: {status: 403, body: {error: {literal: access_denied}}}\nroutes:\n  - view: catalog.orders.OrderById\n    method: get\n    path: /orders/{order_id}\n    parameters: {order_id: param.order_id}\n    refusals:\n      not_granted: {status: 404, body: none}\n  - view: catalog.orders.OrdersOfSize\n    method: get\n    path: /orders/size/{quantity}\n    parameters: {quantity: param.quantity}\n    body: array\n",
    );
    let by_id = &document["paths"]["/orders/{order_id}"]["get"]["responses"];
    assert!(by_id.get("403").is_none(), "{by_id:#}");
    assert!(by_id.get("404").is_some(), "{by_id:#}");
    assert!(by_id["404"].get("content").is_none(), "{by_id:#}");
    let of_size = &document["paths"]["/orders/size/{quantity}"]["get"]["responses"];
    assert_eq!(
        of_size["403"]["content"]["application/json"]["schema"]["properties"]["error"]["const"],
        "access_denied",
        "{of_size:#}"
    );
    assert!(of_size.get("404").is_none(), "{of_size:#}");
}
