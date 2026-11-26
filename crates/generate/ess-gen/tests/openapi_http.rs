//! The `OpenAPI` projection of an `ess-http/1` binding
//! (<https://github.com/beyond10x/ess/issues/493>, `docs/design/http-presentation-binding.md`,
//! "What generated artifacts change (stage 1)").
//!
//! A bound operation is served at its declared method and path, its path variables are required
//! `in: path` parameters, a command's mapped input field leaves the request body, and the operation
//! says `x-ess-route-source: binding`. An operation the binding does not name keeps its derived
//! route and its bytes, and a binding with an open route is refused, naming the operation.
//!
//! Stage 2 ("What `OpenAPI` writes (k4)"): a route's request encoding, the status, body and headers
//! of each listed outcome, a redirect, a `single` view and the standard refusals' overrides, each
//! response naming the outcomes it answers under `x-ess-outcomes`; still no `securitySchemes`.

use ess_compiler::resolve::compile as compile_model;
use ess_compiler::source::SourceMap;
use ess_compiler::EssIr;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_gen::artifact::run;
use ess_gen::http::{bound_routes, derived, outcome_status, routes, Binding};
use ess_gen::openapi::{BoundOpenApi, OpenApi};
use serde_json::{json, Value};

const MODEL: &str = include_str!("../../../specify/ess-http/tests/fixtures/catalog.yaml");

/// A command whose success returns its response beside three refusals, a command without
/// `returns`, and views selected by identity (`tests/outcomes.rs` of `ess-http`).
const ORDERS: &str = include_str!("../../../specify/ess-http/tests/fixtures/orders.yaml");

/// A command read behind a bearer token and an approval that redirects (`tests/headers.rs` of
/// `ess-http`).
const AUTH: &str = include_str!("../../../specify/ess-http/tests/fixtures/auth.yaml");

/// A command whose success returns a response of three fields, one under another wire name, beside
/// an `external:` refusal and a `when:` refusal (`tests/bare_field_and_errors.rs` of `ess-http`).
const TASKS: &str = include_str!("../../../specify/ess-http/tests/fixtures/tasks.yaml");

/// A command with two successes that return one response and two `when:` refusals reporting one
/// error (`tests/alike_and_text.rs` of `ess-http`).
const NOTES: &str = include_str!("../../../specify/ess-http/tests/fixtures/notes.yaml");

/// The items service's document as 0.60.0 generated it from [`ON_ONE_PATH`], less the whole-model
/// digest a generated file no longer stamps, its contract digest written `CONTRACT_DIGEST`: the
/// digest is of the compiled specification, which another ESS release may spell differently,
/// while every other byte is the projection's own.
const STAGE_1_DOCUMENT: &str = include_str!("fixtures/openapi-http/items-service.stage-1.yaml");

fn model() -> EssIr {
    model_of("catalog.yaml", MODEL)
}

fn model_of(file: &str, text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).expect("well formed");
    let spec = Specification::assemble([(Source::new(file), raw)])
        .unwrap_or_else(|errors| panic!("validates: {errors}"));
    let mut sources = SourceMap::new();
    sources.insert(file, text);
    compile_model(&spec, &sources).unwrap_or_else(|errors| panic!("compiles: {errors}"))
}

/// The binding `routes` declares, compiled against `ir` and the routes ESS derives for it.
fn compiled(ir: &EssIr, routes: &str) -> ess_http::HttpIr {
    let text = format!(
        "type: ess-http/1\nspecification:\n  system: catalog\n  version: v1\n  source_digest: sha256:{}\n{routes}",
        ir.source_digest()
    );
    let spec = ess_http::HttpSpec::from_yaml(&text).expect("parses");
    ess_http::compile(&spec, ir, &derived(ir))
        .unwrap_or_else(|diagnostics| panic!("compiles: {diagnostics}"))
}

fn binding(ir: &EssIr, routes: &str) -> Binding {
    Binding::new(compiled(ir, routes)).expect("every route has a path")
}

fn as_json(text: &str) -> Value {
    let yaml: ess_yaml::Value = ess_yaml::from_str(text).expect("YAML");
    serde_json::to_value(yaml).expect("JSON")
}

/// The items service's document, with `binding` beside the model.
fn bound(ir: &EssIr, binding: Binding) -> Value {
    bound_document(ir, binding, "items-service")
}

/// `component`'s document, with `binding` beside the model.
fn bound_document(ir: &EssIr, binding: Binding, component: &str) -> Value {
    let artifacts = run(&BoundOpenApi(binding), ir).expect("generates");
    as_json(&artifacts[&format!("openapi/{component}.yaml")].contents)
}

fn unbound(ir: &EssIr) -> Value {
    let artifacts = run(&OpenApi, ir).expect("generates");
    as_json(&artifacts["openapi/items-service.yaml"].contents)
}

/// Every route on one path bound: a command on `put` and one on `delete`, and the view on `get`.
const ON_ONE_PATH: &str = "routes:
  - command: catalog.items.ListItem
    method: put
    path: /items/{item_id}
    parameters: {item_id: input.item_id}
  - command: catalog.items.TouchItem
    method: delete
    path: /items/{item_id}
    parameters: {item_id: input.item_id}
  - view: catalog.items.ItemById
    method: get
    path: /items/{item_id}
    parameters: {item_id: param.item_id}
";

/// Only `ListItem` bound; `TouchItem` and the view keep their derived routes.
const ONE_ROUTE: &str = "routes:
  - command: catalog.items.ListItem
    method: put
    path: /listings/{item_id}
    parameters: {item_id: input.item_id}
";

#[test]
fn a_bound_operation_is_served_at_its_declared_method_and_path() {
    let ir = model();
    let document = bound(&ir, binding(&ir, ON_ONE_PATH));
    let paths = document["paths"].as_object().expect("paths");
    assert_eq!(
        paths.keys().collect::<Vec<_>>(),
        ["/items/{item_id}"],
        "every derived path of a bound operation is gone: {paths:?}"
    );
    let item = &paths["/items/{item_id}"];
    assert_eq!(item["put"]["operationId"], "catalog.items.ListItem");
    assert_eq!(item["delete"]["operationId"], "catalog.items.TouchItem");
    assert_eq!(item["get"]["operationId"], "catalog.items.ItemById");
    assert!(item.get("post").is_none(), "{item}");
    for method in ["put", "delete", "get"] {
        assert_eq!(item[method]["x-ess-route-source"], "binding", "{method}");
    }
}

#[test]
fn a_path_variable_is_a_required_path_parameter() {
    let ir = model();
    let document = bound(&ir, binding(&ir, ON_ONE_PATH));
    let item = &document["paths"]["/items/{item_id}"];
    for method in ["put", "delete", "get"] {
        let parameters = item[method]["parameters"].as_array().expect("parameters");
        let path: Vec<&Value> = parameters
            .iter()
            .filter(|parameter| parameter["in"] == "path")
            .collect();
        assert_eq!(path.len(), 1, "{method}: {parameters:?}");
        assert_eq!(path[0]["name"], "item_id", "{method}");
        assert_eq!(path[0]["required"], true, "{method}");
        assert_eq!(
            path[0]["schema"],
            json!({"$ref": "#/components/schemas/catalog.items.ItemId"}),
            "{method}"
        );
    }
}

#[test]
fn a_mapped_input_field_leaves_the_request_body() {
    let ir = model();
    let document = bound(&ir, binding(&ir, ON_ONE_PATH));
    let schemas = &document["components"]["schemas"];
    let input = &schemas["catalog.items.ListItem.Input"];
    assert_eq!(
        input["properties"]
            .as_object()
            .expect("properties")
            .keys()
            .collect::<Vec<_>>(),
        ["title"],
        "{input}"
    );
    assert_eq!(input["required"], json!(["title"]), "{input}");
    let put = &document["paths"]["/items/{item_id}"]["put"];
    assert_eq!(put["requestBody"]["required"], true, "{put}");
    assert_eq!(
        put["requestBody"]["content"]["application/json"]["schema"]["$ref"],
        "#/components/schemas/catalog.items.ListItem.Input"
    );

    // `TouchItem`'s one input field is in the path, so nothing is left for a body.
    let delete = &document["paths"]["/items/{item_id}"]["delete"];
    assert!(delete.get("requestBody").is_none(), "{delete}");
    assert!(
        schemas.get("catalog.items.TouchItem.Input").is_none(),
        "a body nothing sends has no schema: {schemas}"
    );
    // The field's type is still described: the path parameter points at it.
    assert!(schemas.get("catalog.items.ItemId").is_some(), "{schemas}");
}

#[test]
fn a_bound_view_reads_its_mapped_parameter_from_the_path_not_the_query() {
    let ir = model();
    let get = &bound(&ir, binding(&ir, ON_ONE_PATH))["paths"]["/items/{item_id}"]["get"];
    let parameters = get["parameters"].as_array().expect("parameters");
    assert!(
        parameters
            .iter()
            .all(|parameter| parameter["in"] != "query"),
        "{parameters:?}"
    );
    let unbound = unbound(&model());
    let query = &unbound["paths"]["/items/views/ItemById"]["get"]["parameters"][0];
    assert_eq!(
        query["in"], "query",
        "the view's parameter is a query parameter unbound"
    );
}

#[test]
fn an_unbound_operation_keeps_its_derived_route_and_bytes() {
    let ir = model();
    let document = bound(&ir, binding(&ir, ONE_ROUTE));
    let unbound = unbound(&ir);
    for (path, method) in [
        ("/items/commands/TouchItem", "post"),
        ("/items/views/ItemById", "get"),
    ] {
        assert_eq!(
            document["paths"][path][method], unbound["paths"][path][method],
            "{method} {path}"
        );
        assert!(document["paths"][path][method]
            .get("x-ess-route-source")
            .is_none());
    }
    assert!(document["paths"].get("/items/commands/ListItem").is_none());
    assert_eq!(
        document["paths"]["/listings/{item_id}"]["put"]["x-ess-route-source"],
        "binding"
    );
    assert_eq!(
        document["components"]["schemas"]["catalog.items.TouchItem.Input"],
        unbound["components"]["schemas"]["catalog.items.TouchItem.Input"]
    );
}

#[test]
fn a_binding_that_routes_nothing_keeps_every_byte() {
    let ir = model();
    let bound = run(&BoundOpenApi(binding(&ir, "routes: []\n")), &ir).expect("generates");
    let unbound = run(&OpenApi, &ir).expect("generates");
    assert_eq!(bound.len(), unbound.len());
    for (path, artifact) in &unbound {
        assert_eq!(bound[path].contents, artifact.contents, "{path}");
    }
}

#[test]
fn the_route_table_without_a_binding_is_todays_table() {
    let ir = model();
    for component in ir.components().values() {
        assert_eq!(bound_routes(&ir, component, None), routes(&ir, component));
    }
}

#[test]
fn a_binding_with_an_open_route_is_refused_naming_the_operation() {
    let ir = model();
    let open = compiled(
        &ir,
        "routes:
  - command: catalog.items.ListItem
    method: post
  - view: catalog.items.ItemById
    method: get
    path: /items/{item_id}
    parameters: {item_id: param.item_id}
",
    );
    let refused = Binding::new(open).expect_err("an open route has no path to project");
    let message = refused.to_string();
    assert!(
        message.contains("command `catalog.items.ListItem`"),
        "{message}"
    );
    assert!(!message.contains("ItemById"), "{message}");
}

// Stage 2 (`docs/design/http-presentation-binding.md`, "What OpenAPI writes (k4)").

/// The stage-1 document with its provenance digests written as placeholders, so the pin holds
/// every byte the projection writes and none the compiled specification's spelling decides.
fn without_digests(text: &str) -> String {
    let document = as_json(text);
    let provenance = &document["info"]["x-ess-provenance"];
    let contract = provenance["contract_digest"]
        .as_str()
        .expect("contract digest");
    text.replace(contract, "CONTRACT_DIGEST")
}

#[test]
fn a_stage_1_binding_keeps_its_openapi_bytes() {
    let ir = model();
    let artifacts = run(&BoundOpenApi(binding(&ir, ON_ONE_PATH)), &ir).expect("generates");
    let text = without_digests(&artifacts["openapi/items-service.yaml"].contents);
    assert!(text.contains("CONTRACT_DIGEST"));
    assert_eq!(
        text, STAGE_1_DOCUMENT,
        "a binding with no stage-2 key changed the OpenAPI bytes 0.60.0 wrote"
    );
    assert!(!text.contains("x-ess-outcomes"), "{text}");
}

/// The orders service bound with a form-encoded command whose outcomes answer a bare response
/// with a header and two error objects on one status, a command answering `204`, and a `single`
/// view.
const ORDERS_ROUTES: &str = "routes:
  - command: catalog.orders.PlaceOrder
    method: post
    path: /api/orders
    request: {encoding: form, unknown: ignore}
    outcomes:
      placed:
        status: 200
        body: response
        headers:
          Cache-Control: {literal: no-store}
      out-of-stock: {status: 400, body: {error: error}}
      too-many: {status: 400, body: {error: error}}
  - command: catalog.orders.NoteOrder
    method: post
    path: /api/orders/{order_id}/note
    parameters: {order_id: input.order_id}
    outcomes:
      noted: {status: 204, body: none}
  - view: catalog.orders.OrderById
    method: get
    path: /orders/{order_id}
    parameters: {order_id: param.order_id}
    body: single
    empty: {status: 404}
";

/// The auth service bound with a bearer-token refusal, a sourced header, and a query-encoded
/// approval that redirects.
const AUTH_ROUTES: &str = "routes:
  - command: catalog.auth.ReadPrivateItem
    method: get
    path: /private/items/{item_id}
    parameters: {item_id: input.item_id}
    outcomes:
      token-refused:
        status: 401
        body: {error: error}
        headers:
          WWW-Authenticate:
            challenge:
              scheme: Bearer
              params:
                realm: {literal: catalog}
                error: {source: error.wire}
          Retry-After: {source: error.retry_after}
      read:
        status: 200
        body: response
        headers:
          X-Item: {source: response.item_id}
  - command: catalog.auth.ApproveOrder
    method: get
    path: /approve
    request: {encoding: query}
    outcomes:
      approved:
        status: 302
        body: none
        redirect:
          location: input.return_to
          mode: query
          parameters:
            code: response.code
            state: input.state
      denied: {status: 400, body: {error: error}}
";

fn orders() -> (EssIr, Value) {
    let ir = model_of("orders.yaml", ORDERS);
    let document = bound_document(&ir, binding(&ir, ORDERS_ROUTES), "orders-service");
    (ir, document)
}

fn auth() -> Value {
    let ir = model_of("auth.yaml", AUTH);
    bound_document(&ir, binding(&ir, AUTH_ROUTES), "auth-service")
}

/// The schema a `#/components/schemas/<key>` pointer names.
fn resolved<'a>(document: &'a Value, schema: &Value) -> &'a Value {
    let pointer = schema["$ref"].as_str().expect("a reference");
    let key = pointer
        .strip_prefix("#/components/schemas/")
        .expect("into components");
    let found = &document["components"]["schemas"][key];
    assert!(!found.is_null(), "{pointer} resolves to nothing");
    found
}

/// An object's keys. `serde_json`'s map is sorted in this build (no `preserve_order`), so these
/// are in name order whatever order the document wrote them in: an order a test asserts is read
/// from an array, such as `required`.
fn keys(value: &Value) -> Vec<&str> {
    value
        .as_object()
        .unwrap_or_else(|| panic!("an object: {value}"))
        .keys()
        .map(String::as_str)
        .collect()
}

#[test]
fn a_form_route_takes_a_form_urlencoded_request_body() {
    let (_, document) = orders();
    let post = &document["paths"]["/api/orders"]["post"];
    let body = &post["requestBody"];
    assert_eq!(
        keys(&body["content"]),
        ["application/x-www-form-urlencoded"],
        "{body}"
    );
    assert_eq!(body["required"], true, "{body}");
    let schema = &body["content"]["application/x-www-form-urlencoded"]["schema"];
    assert_eq!(
        schema["$ref"],
        "#/components/schemas/catalog.orders.PlaceOrder.Input"
    );
    // The flat fields, under their wire names.
    let input = resolved(&document, schema);
    assert_eq!(keys(&input["properties"]), ["order_id", "quantity"]);
    // A JSON route of the same document keeps `application/json`: `NoteOrder` has no body left,
    // and the unbound items service is checked by the stage-1 pin.
    assert!(document["paths"]["/api/orders/{order_id}/note"]["post"]
        .get("requestBody")
        .is_none());
}

/// `request: {unknown: ignore}` (beyond10x/ess#752,
/// `story:a-json-request-may-ignore-undeclared-members-and-echoed-path-fields`): on a `json`
/// route `{command}.Input` is open and the request body says what is ignored and that a member
/// the path carries must equal the path's value; a `form` route that ignores is open as well, and
/// a route without `ignore` stays closed.
#[test]
fn a_route_that_ignores_unknown_members_publishes_an_open_input() {
    let ir = model();
    let routes = "routes:
  - command: catalog.items.ListItem
    method: put
    path: /listings/{item_id}
    parameters: {item_id: input.item_id}
    request: {unknown: ignore}
";
    let document = bound(&ir, binding(&ir, routes));
    let body = &document["paths"]["/listings/{item_id}"]["put"]["requestBody"];
    let input = resolved(&document, &body["content"]["application/json"]["schema"]);
    assert_eq!(input["additionalProperties"], true, "{input}");
    assert_eq!(keys(&input["properties"]), ["title"]);
    let description = body["description"].as_str().expect("a description");
    assert!(
        description.contains("A member no field names is ignored")
            && description.contains("a member the path carries must equal the path's value"),
        "{description}"
    );

    let closed = bound(&ir, binding(&ir, ONE_ROUTE));
    let body = &closed["paths"]["/listings/{item_id}"]["put"]["requestBody"];
    let input = resolved(&closed, &body["content"]["application/json"]["schema"]);
    assert_eq!(input["additionalProperties"], false, "{input}");
    assert_eq!(
        body["description"],
        "The input `catalog.items.ListItem` declares."
    );

    let (_, orders) = orders();
    let body = &orders["paths"]["/api/orders"]["post"]["requestBody"];
    let input = resolved(
        &orders,
        &body["content"]["application/x-www-form-urlencoded"]["schema"],
    );
    assert_eq!(input["additionalProperties"], true, "{input}");
    let description = body["description"].as_str().expect("a description");
    assert!(
        description.ends_with(" A parameter no field names is ignored."),
        "a route whose path carries nothing says nothing of it: {description}"
    );
}

#[test]
fn a_query_command_takes_its_unmapped_inputs_as_query_parameters_and_has_no_request_body() {
    let document = auth();
    let get = &document["paths"]["/approve"]["get"];
    assert!(get.get("requestBody").is_none(), "{get}");
    assert!(
        document["components"]["schemas"]
            .get("catalog.auth.ApproveOrder.Input")
            .is_none(),
        "a body nothing sends has no schema"
    );
    let parameters = get["parameters"].as_array().expect("parameters");
    let query: Vec<&Value> = parameters
        .iter()
        .filter(|parameter| parameter["in"] == "query")
        .collect();
    assert_eq!(
        query
            .iter()
            .map(|parameter| parameter["name"].as_str().expect("name"))
            .collect::<Vec<_>>(),
        [
            "return_to",
            "state",
            "client",
            "count",
            "maybe_uri",
            "scopes"
        ],
        "every input field, in declaration order: {parameters:?}"
    );
    let named = |name: &str| {
        *query
            .iter()
            .find(|parameter| parameter["name"] == name)
            .expect(name)
    };
    assert_eq!(named("return_to")["required"], true);
    assert_eq!(
        named("return_to")["schema"]["$ref"],
        "#/components/schemas/catalog.auth.ReturnUri"
    );
    assert_eq!(named("state")["required"], false);
    assert_eq!(named("maybe_uri")["required"], false);
    // A list repeats its key once per element.
    assert_eq!(named("scopes")["style"], "form");
    assert_eq!(named("scopes")["explode"], true);
    assert!(named("client").get("style").is_none());
    // The types the parameters point at are described.
    for key in ["catalog.auth.ReturnUri", "catalog.auth.Count"] {
        assert!(
            document["components"]["schemas"].get(key).is_some(),
            "{key}"
        );
    }
}

/// A command on `delete` whose `Optional<String>` input rides in the query string (an adopter's
/// protocol, whose `DELETE` takes query parameters).
#[test]
fn a_delete_command_encoded_query_takes_its_unmapped_inputs_as_query_parameters() {
    let text = MODEL
        .replace(
            "  - name: catalog.items.PurgeItems\n",
            "  - name: catalog.items.ForgetItem
    input:
      - {name: item_id, type: catalog.items.ItemId}
      - {name: reason, type: Optional<String>, wire: forgetReason}
    outcomes:
      - name: forgotten
        emits: [catalog.items.ItemTouched]
        payload:
          catalog.items.ItemTouched: {item_id: input.item_id}
  - name: catalog.items.PurgeItems
",
        )
        .replace(
            "accepts: {commands: [catalog.items.ListItem, catalog.items.TouchItem]}",
            "accepts: {commands: [catalog.items.ForgetItem, catalog.items.ListItem, catalog.items.TouchItem]}",
        );
    let ir = model_of("catalog.yaml", &text);
    let document = bound(
        &ir,
        binding(
            &ir,
            "routes:
  - command: catalog.items.ForgetItem
    method: delete
    path: /items/{item_id}
    parameters: {item_id: input.item_id}
    request: {encoding: query}
",
        ),
    );
    let delete = &document["paths"]["/items/{item_id}"]["delete"];
    assert!(delete.get("requestBody").is_none(), "{delete}");
    assert!(
        document["components"]["schemas"]
            .get("catalog.items.ForgetItem.Input")
            .is_none(),
        "a body nothing sends has no schema"
    );
    let parameters = delete["parameters"].as_array().expect("parameters");
    let placed: Vec<(&str, &str, bool)> = parameters
        .iter()
        .map(|parameter| {
            (
                parameter["in"].as_str().expect("in"),
                parameter["name"].as_str().expect("name"),
                parameter["required"].as_bool().unwrap_or(false),
            )
        })
        .collect();
    assert_eq!(
        placed,
        [("path", "item_id", true), ("query", "forgetReason", false)],
        "{parameters:?}"
    );
}

#[test]
fn each_declared_status_is_its_own_response_with_x_ess_outcomes() {
    let (ir, document) = orders();
    let responses = &document["paths"]["/api/orders"]["post"]["responses"];
    let command = ir
        .commands()
        .values()
        .find(|command| command.name.to_string() == "catalog.orders.PlaceOrder")
        .expect("PlaceOrder");
    let mislabelled = command
        .outcomes
        .iter()
        .find(|outcome| outcome.name.as_str() == "mislabelled")
        .expect("mislabelled");
    // The one outcome the binding does not list keeps its derived status and the envelope.
    let derived = outcome_status(&ir, mislabelled);
    let mut expected = vec!["200", "400", derived, "501"];
    expected.sort_unstable();
    assert_eq!(keys(responses), expected, "{responses}");
    assert_eq!(responses["200"]["x-ess-outcomes"], json!(["placed"]));
    assert_eq!(
        responses["400"]["x-ess-outcomes"],
        json!(["out-of-stock", "too-many"]),
        "in the order the command declares them"
    );
    assert_eq!(responses[derived]["x-ess-outcomes"], json!(["mislabelled"]));
    assert_eq!(
        responses[derived]["content"]["application/json"]["schema"]["$ref"],
        "#/components/schemas/catalog.orders.PlaceOrder.mislabelled.Response",
        "an unlisted outcome keeps the envelope"
    );
    // The standard refusal answers no declared outcome.
    assert_eq!(responses["501"]["x-ess-outcomes"], json!([]));

    let note = &document["paths"]["/api/orders/{order_id}/note"]["post"]["responses"];
    assert_eq!(keys(note), ["204", "501"]);
    assert_eq!(note["204"]["x-ess-outcomes"], json!(["noted"]));
    assert!(note["204"].get("content").is_none(), "{note}");
}

#[test]
fn a_bare_response_body_carries_the_declared_response_schema_without_the_envelope() {
    let (_, document) = orders();
    let ok = &document["paths"]["/api/orders"]["post"]["responses"]["200"];
    let schema = &ok["content"]["application/json"]["schema"];
    assert_eq!(
        schema,
        &json!({"$ref": "#/components/schemas/catalog.orders.PlaceOrder.Result"})
    );
    let response = resolved(&document, schema);
    assert_eq!(keys(&response["properties"]), ["order_id"]);
    for envelope in ["outcome", "published", "response"] {
        assert!(
            response["properties"].get(envelope).is_none(),
            "{envelope}: {response}"
        );
    }
    // The envelope this outcome no longer answers is not described.
    assert!(document["components"]["schemas"]
        .get("catalog.orders.PlaceOrder.placed.Response")
        .is_none());
}

#[test]
fn an_error_body_names_the_member_and_the_errors_fields() {
    let (_, document) = orders();
    let refused = &document["paths"]["/api/orders"]["post"]["responses"]["400"];
    let schema = &refused["content"]["application/json"]["schema"];
    let one_of = schema["oneOf"].as_array().expect("oneOf");
    assert_eq!(one_of.len(), 2, "{schema}");
    assert_eq!(schema["discriminator"]["propertyName"], "error");
    assert_eq!(
        schema["discriminator"]["mapping"],
        json!({
            "out_of_stock": "#/components/schemas/catalog.orders.PlaceOrder.out-of-stock.Response",
            "too_many": "#/components/schemas/catalog.orders.PlaceOrder.too-many.Response",
        })
    );

    let out_of_stock = resolved(&document, &one_of[0]);
    assert_eq!(keys(&out_of_stock["properties"]), ["available", "error"]);
    assert_eq!(out_of_stock["properties"]["error"]["const"], "out_of_stock");
    assert_eq!(out_of_stock["additionalProperties"], false);
    let required: Vec<&str> = out_of_stock["required"]
        .as_array()
        .expect("required")
        .iter()
        .map(|name| name.as_str().expect("name"))
        .collect();
    assert_eq!(required, ["error", "available"]);
    for envelope in ["outcome", "published", "payload"] {
        assert!(out_of_stock["properties"].get(envelope).is_none());
    }

    // An error that declares no field is the member alone.
    let too_many = resolved(&document, &one_of[1]);
    assert_eq!(keys(&too_many["properties"]), ["error"]);
    assert_eq!(too_many["properties"]["error"]["const"], "too_many");
    assert_eq!(too_many["required"], json!(["error"]));
}

#[test]
fn declared_headers_and_a_challenge_are_response_headers() {
    let (_, orders) = orders();
    let ok = &orders["paths"]["/api/orders"]["post"]["responses"]["200"];
    assert_eq!(keys(&ok["headers"]), ["Cache-Control"]);
    let cache = &ok["headers"]["Cache-Control"];
    assert_eq!(
        cache["schema"],
        json!({"type": "string", "const": "no-store"})
    );
    assert_eq!(cache["required"], true);

    let document = auth();
    let responses = &document["paths"]["/private/items/{item_id}"]["get"]["responses"];
    let refused = &responses["401"];
    assert_eq!(
        keys(&refused["headers"]),
        ["Retry-After", "WWW-Authenticate"]
    );
    let challenge = &refused["headers"]["WWW-Authenticate"];
    assert_eq!(challenge["schema"], json!({"type": "string"}));
    assert_eq!(challenge["required"], true);
    let description = challenge["description"].as_str().expect("description");
    for named in ["Bearer", "realm", "catalog", "error", "error.wire"] {
        assert!(description.contains(named), "{named}: {description}");
    }
    // An optional source omits its header when absent, so the header is not required.
    let retry = &refused["headers"]["Retry-After"];
    assert_eq!(retry["schema"], json!({"type": "string"}));
    assert!(retry.get("required").is_none(), "{retry}");
    assert!(retry["description"]
        .as_str()
        .expect("description")
        .contains("error.retry_after"));
    // A required source is always written.
    let item = &responses["200"]["headers"]["X-Item"];
    assert_eq!(item["required"], true);
    assert!(item["description"]
        .as_str()
        .expect("description")
        .contains("response.item_id"));
    // The error object beside the challenge.
    let schema = resolved(&document, &refused["content"]["application/json"]["schema"]);
    assert_eq!(schema["properties"]["error"]["const"], "invalid_token");
    assert_eq!(
        keys(&schema["properties"]),
        ["error", "error_description", "grant", "retry_after"]
    );
    // The member first, then the error's fields in declaration order; `retry_after` is optional.
    assert_eq!(
        schema["required"],
        json!(["error", "error_description", "grant"])
    );
}

#[test]
fn a_redirect_answers_its_status_with_a_location_header_and_no_content() {
    let document = auth();
    let responses = &document["paths"]["/approve"]["get"]["responses"];
    let found = &responses["302"];
    assert!(found.get("content").is_none(), "{found}");
    assert_eq!(found["x-ess-outcomes"], json!(["approved"]));
    assert_eq!(keys(&found["headers"]), ["Location"]);
    let location = &found["headers"]["Location"];
    assert_eq!(location["required"], true);
    assert_eq!(
        location["schema"],
        json!({"type": "string", "format": "uri"})
    );
    let description = location["description"].as_str().expect("description");
    for named in [
        "input.return_to",
        "query",
        "`code`",
        "response.code",
        "`state`",
        "input.state",
    ] {
        assert!(description.contains(named), "{named}: {description}");
    }
    // The denied branch is its own status, with the error object.
    assert_eq!(responses["400"]["x-ess-outcomes"], json!(["denied"]));
    let denied = resolved(
        &document,
        &responses["400"]["content"]["application/json"]["schema"],
    );
    assert_eq!(denied["properties"]["error"]["const"], "access_denied");
}

#[test]
fn a_single_view_answers_one_row_and_its_empty_status() {
    let (_, document) = orders();
    let get = &document["paths"]["/orders/{order_id}"]["get"];
    let responses = &get["responses"];
    assert_eq!(keys(responses), ["200", "404"]);
    assert_eq!(
        responses["200"]["content"]["application/json"]["schema"],
        json!({"$ref": "#/components/schemas/catalog.orders.OrderById.Row"})
    );
    let row = resolved(
        &document,
        &responses["200"]["content"]["application/json"]["schema"],
    );
    assert_eq!(keys(&row["properties"]), ["order_id", "quantity"]);
    assert!(responses["404"].get("content").is_none(), "{responses}");
    assert!(
        document["components"]["schemas"]
            .get("catalog.orders.OrderById.Response")
            .is_none(),
        "the rows wrapper nothing answers is not described"
    );
    // A view the binding does not name keeps `rows`.
    let other = &document["paths"]["/orders/views/OrderByIdReversed"]["get"]["responses"]["200"];
    assert_eq!(
        other["content"]["application/json"]["schema"]["$ref"],
        "#/components/schemas/catalog.orders.OrderByIdReversed.Response"
    );
}

/// Every key named `key` anywhere in `value`.
fn any_key(value: &Value, key: &str) -> bool {
    match value {
        Value::Object(map) => {
            map.contains_key(key) || map.values().any(|value| any_key(value, key))
        }
        Value::Array(items) => items.iter().any(|value| any_key(value, key)),
        _ => false,
    }
}

#[test]
fn no_security_schemes_are_emitted_for_a_challenge() {
    let document = auth();
    assert!(
        document["paths"]["/private/items/{item_id}"]["get"]["responses"]["401"]["headers"]
            .get("WWW-Authenticate")
            .is_some()
    );
    assert!(!any_key(&document, "securitySchemes"), "{document}");
    assert!(!any_key(&document, "security"), "{document}");
}

#[test]
fn a_refusal_override_replaces_the_standard_answer_on_every_bound_operation() {
    let ir = model_of("orders.yaml", ORDERS);
    let routes = format!(
        "refusals:
  malformed: {{status: 400, body: {{error: {{literal: invalid_request}}}}}}
  unfinished: {{status: 503, body: none, headers: {{Retry-After: {{literal: '120'}}}}}}
{ORDERS_ROUTES}"
    );
    let document = bound_document(&ir, binding(&ir, &routes), "orders-service");
    let place = &document["paths"]["/api/orders"]["post"]["responses"];
    assert!(place.get("501").is_none(), "{place}");
    let unfinished = &place["503"];
    assert_eq!(unfinished["x-ess-outcomes"], json!([]));
    assert!(unfinished.get("content").is_none(), "{unfinished}");
    assert_eq!(
        unfinished["headers"]["Retry-After"]["schema"],
        json!({"type": "string", "const": "120"})
    );
    // `malformed` shares `400` with two error objects; the literal tells it apart.
    let refused = &place["400"]["content"]["application/json"]["schema"]["oneOf"];
    let refused = refused.as_array().expect("oneOf");
    assert_eq!(refused.len(), 3, "{refused:?}");
    assert_eq!(
        refused[2],
        json!({
            "type": "object",
            "additionalProperties": false,
            "required": ["error"],
            "properties": {"error": {"type": "string", "const": "invalid_request"}},
        })
    );
    assert_eq!(
        place["400"]["x-ess-outcomes"],
        json!(["out-of-stock", "too-many"])
    );
    // A bound operation without a declared answer of its own takes the overrides too.
    let note = &document["paths"]["/api/orders/{order_id}/note"]["post"]["responses"];
    assert_eq!(keys(note), ["204", "400", "503"]);
    assert_eq!(note["400"]["x-ess-outcomes"], json!([]));
    // A derived route keeps today's answers.
    let unbound = unbound_document(&ir, "orders-service");
    let mut kept = 0;
    for (path, item) in unbound["paths"].as_object().expect("paths") {
        if let Some(bound) = document["paths"].get(path) {
            assert_eq!(bound, item, "{path}");
            kept += 1;
        }
    }
    assert!(kept > 0, "some derived route is left to compare");
}

fn unbound_document(ir: &EssIr, component: &str) -> Value {
    let artifacts = run(&OpenApi, ir).expect("generates");
    as_json(&artifacts[&format!("openapi/{component}.yaml")].contents)
}

/// `SendTask` on `post /api/tasks`, with `keys` beside its path, in the tasks service's document.
fn tasks(keys: &str) -> Value {
    let ir = model_of("tasks.yaml", TASKS);
    let routes = format!(
        "routes:\n  - command: catalog.tasks.SendTask\n    method: post\n    path: /api/tasks\n{keys}"
    );
    bound_document(&ir, binding(&ir, &routes), "tasks-service")
}

#[test]
fn a_response_field_body_is_that_fields_schema_alone() {
    let document = tasks("    outcomes:\n      sent: {status: 200, body: response.version}\n");
    let ok = &document["paths"]["/api/tasks"]["post"]["responses"]["200"];
    assert_eq!(ok["x-ess-outcomes"], json!(["sent"]));
    let schema = &ok["content"]["application/json"]["schema"];
    assert_eq!(
        schema,
        &json!({"$ref": "#/components/schemas/catalog.tasks.SendTask.sent.Response"})
    );
    let field = resolved(&document, schema);
    assert_eq!(field["type"], "string", "{field}");
    assert!(
        field.get("properties").is_none(),
        "no object around it: {field}"
    );
    let description = ok["description"].as_str().expect("a description");
    assert!(
        description.contains("response field `version`, bare"),
        "{description}"
    );
    // The whole response is answered by no outcome, so it is not described.
    assert!(document["components"]["schemas"]
        .get("catalog.tasks.SendTask.Result")
        .is_none());
}

#[test]
fn errors_answers_every_unlisted_error_outcome_under_one_status() {
    let document = tasks(
        "    outcomes:\n      sent: {status: 200, body: response.task_id}\n    errors: {status: 400, body: {error: error}}\n",
    );
    let responses = &document["paths"]["/api/tasks"]["post"]["responses"];
    // Neither the external refusal's derived 502 nor the `when:` refusal's derived 422 is left.
    assert!(responses.get("502").is_none(), "{responses}");
    assert!(responses.get("422").is_none(), "{responses}");
    let refused = &responses["400"];
    assert_eq!(
        refused["x-ess-outcomes"],
        json!(["unsupported", "too-large"])
    );
    let schema = &refused["content"]["application/json"]["schema"];
    assert_eq!(schema["discriminator"]["propertyName"], "error");
    assert_eq!(
        schema["discriminator"]["mapping"],
        json!({
            "too_large": "#/components/schemas/catalog.tasks.SendTask.too-large.Response",
            "unsupported": "#/components/schemas/catalog.tasks.SendTask.unsupported.Response",
        })
    );
    let too_large = resolved(
        &document,
        &json!({"$ref": "#/components/schemas/catalog.tasks.SendTask.too-large.Response"}),
    );
    assert_eq!(keys(&too_large["properties"]), ["error", "limit"]);
    assert_eq!(too_large["properties"]["error"]["const"], "too_large");
}

#[test]
fn errors_alone_makes_the_route_answered_by_the_binding() {
    let document = tasks("    errors: {status: 409, body: {error: error}}\n");
    let responses = &document["paths"]["/api/tasks"]["post"]["responses"];
    assert_eq!(
        responses["409"]["x-ess-outcomes"],
        json!(["unsupported", "too-large"])
    );
    assert_eq!(
        responses["409"]["content"]["application/json"]["schema"]["discriminator"]["propertyName"],
        "error"
    );
    // The success the binding does not list keeps the envelope at its derived status.
    let sent = &responses["200"];
    assert_eq!(sent["x-ess-outcomes"], json!(["sent"]));
    assert_eq!(
        sent["content"]["application/json"]["schema"]["$ref"],
        "#/components/schemas/catalog.tasks.SendTask.sent.Response"
    );
}

/// `CreateNote` on `post /api/notes`, with `keys` beside its path, in the notes service's
/// document.
fn notes(keys: &str) -> Value {
    let ir = model_of("notes.yaml", NOTES);
    let text = format!(
        "type: ess-http/1\nspecification:\n  system: demo\n  version: v1\n  source_digest: \
         sha256:{}\nroutes:\n  - command: demo.notes.CreateNote\n    method: post\n    path: \
         /api/notes\n{keys}",
        ir.source_digest()
    );
    let spec = ess_http::HttpSpec::from_yaml(&text).expect("parses");
    let http = ess_http::compile(&spec, &ir, &derived(&ir))
        .unwrap_or_else(|diagnostics| panic!("compiles: {diagnostics}"));
    bound_document(
        &ir,
        Binding::new(http).expect("every route has a path"),
        "notes-service",
    )
}

/// The schema `schema` comes to once every `$ref` on the way is followed: a field of a newtype is
/// a reference to the newtype's own schema.
fn followed<'a>(document: &'a Value, schema: &'a Value) -> &'a Value {
    let mut schema = schema;
    while schema.get("$ref").is_some() {
        schema = resolved(document, schema);
    }
    schema
}

#[test]
fn outcomes_declared_alike_project_one_response() {
    let document = notes(
        "    outcomes:\n      fetched: {status: 201, body: response}\n      inline: {status: 201, body: response}\n    alike: [[fetched, inline]]\n",
    );
    let responses = &document["paths"]["/api/notes"]["post"]["responses"];
    let created = &responses["201"];
    assert_eq!(created["x-ess-outcomes"], json!(["fetched", "inline"]));
    let schema = &created["content"]["application/json"]["schema"];
    assert_eq!(
        schema,
        &json!({"$ref": "#/components/schemas/demo.notes.CreateNote.Result"}),
        "one schema, no `oneOf`: {created}"
    );

    // A bare field and two error objects under one wire name are one schema each, too.
    let document = notes(
        "    outcomes:\n      fetched: {status: 200, body: response.note_id}\n      inline: {status: 200, body: response.note_id}\n      negative: {status: 400, body: {error: error}}\n      blank: {status: 400, body: {error: error}}\n    alike: [[blank, negative], [fetched, inline]]\n",
    );
    let responses = &document["paths"]["/api/notes"]["post"]["responses"];
    assert_eq!(
        responses["200"]["x-ess-outcomes"],
        json!(["fetched", "inline"])
    );
    let field = &responses["200"]["content"]["application/json"]["schema"];
    assert!(field.get("oneOf").is_none(), "{field}");
    let field = followed(&document, field);
    assert_eq!(field["type"], "string", "{field}");
    assert_eq!(
        responses["400"]["x-ess-outcomes"],
        json!(["negative", "blank"])
    );
    let refused = &responses["400"]["content"]["application/json"]["schema"];
    assert!(refused.get("oneOf").is_none(), "{refused}");
    assert_eq!(
        resolved(&document, refused)["properties"]["error"]["const"],
        "refused"
    );
}

#[test]
fn a_text_outcome_projects_text_plain() {
    let document = notes(
        "    outcomes:\n      inline: {status: 201, body: response.note_id, encoding: text}\n",
    );
    let created = &document["paths"]["/api/notes"]["post"]["responses"]["201"];
    assert_eq!(created["x-ess-outcomes"], json!(["inline"]));
    assert_eq!(keys(&created["content"]), ["text/plain"], "{created}");
    let schema = &created["content"]["text/plain"]["schema"];
    let field = followed(&document, schema);
    assert_eq!(field["type"], "string", "{field}");
    let description = created["description"].as_str().expect("a description");
    assert!(description.contains("as text"), "{description}");
}

/// The orders service with a route's own refusal over the binding's, an empty answer with a
/// literal body, and a view answered as a bare array.
const ANSWER_FORMS: &str = "refusals:
  malformed: {status: 400, body: {error: {literal: invalid_request}}}
routes:
  - command: catalog.orders.PlaceOrder
    method: post
    path: /api/orders
    refusals:
      malformed: {status: 400, body: {error: {literal: unreadable}}}
  - command: catalog.orders.NoteOrder
    method: post
    path: /api/orders/{order_id}/note
    parameters: {order_id: input.order_id}
  - view: catalog.orders.OrderById
    method: get
    path: /orders/{order_id}
    parameters: {order_id: param.order_id}
    body: single
    empty: {status: 404, body: {error: {literal: not_found}}}
  - view: catalog.orders.OrdersOfSize
    method: get
    path: /orders/size/{quantity}
    parameters: {quantity: param.quantity}
    body: array
";

/// The `const` of the one member `member` of the JSON body `response` answers.
fn literal<'a>(response: &'a Value, member: &str) -> &'a Value {
    &response["content"]["application/json"]["schema"]["properties"][member]["const"]
}

#[test]
fn route_refusals_empty_bodies_and_arrays_project() {
    let ir = model_of("orders.yaml", ORDERS);
    let document = bound_document(&ir, binding(&ir, ANSWER_FORMS), "orders-service");
    let paths = &document["paths"];

    // The route's own `malformed` stands on its route; the binding's on the other.
    let placed = &paths["/api/orders"]["post"]["responses"]["400"];
    assert_eq!(literal(placed, "error"), "unreadable", "{placed:#}");
    assert_eq!(placed["x-ess-outcomes"], json!([]), "{placed:#}");
    let description = placed["description"].as_str().expect("a description");
    assert!(description.contains("`malformed`"), "{description}");
    let noted = &paths["/api/orders/{order_id}/note"]["post"]["responses"]["400"];
    assert_eq!(literal(noted, "error"), "invalid_request", "{noted:#}");

    // An empty answer's literal body is its response's schema.
    let empty = &paths["/orders/{order_id}"]["get"]["responses"]["404"];
    assert_eq!(literal(empty, "error"), "not_found", "{empty:#}");
    assert_eq!(
        empty["content"]["application/json"]["schema"]["required"],
        json!(["error"])
    );

    // An array view answers the row schema in an array, and no rows wrapper is described.
    let rows = &paths["/orders/size/{quantity}"]["get"]["responses"]["200"];
    let schema = &rows["content"]["application/json"]["schema"];
    assert_eq!(schema["type"], "array", "{rows:#}");
    let row = resolved(&document, &schema["items"]);
    assert_eq!(keys(&row["properties"]), ["order_id"], "{row:#}");
    assert!(
        document["components"]["schemas"]
            .get("catalog.orders.OrdersOfSize.Response")
            .is_none(),
        "the rows wrapper nothing answers is not described"
    );
    let description = rows["description"].as_str().expect("a description");
    assert!(description.contains("`body: array`"), "{description}");

    // Without a route's own refusals or a top-level one, a route keeps the stage-1 answers.
    let plain = ANSWER_FORMS
        .replace(
            "refusals:\n  malformed: {status: 400, body: {error: {literal: invalid_request}}}\n",
            "",
        )
        .replace(
            "    refusals:\n      malformed: {status: 400, body: {error: {literal: unreadable}}}\n",
            "",
        );
    assert_ne!(plain, ANSWER_FORMS);
    let document = bound_document(&ir, binding(&ir, &plain), "orders-service");
    let placed = &document["paths"]["/api/orders"]["post"]["responses"];
    assert!(!any_key(placed, "x-ess-outcomes"), "{placed:#}");
}

/// A board whose card priorities are one bare list body beside the board in the path
/// (`story:a-command-route-takes-one-input-as-its-bare-request-body`, beyond10x/ess#748).
const BOARD: &str = include_str!("../../../specify/ess-http/tests/fixtures/board.yaml");

/// `SetCardPriorities` on `put /boards/{board_id}/cards/priority`, its list input the whole body.
const BARE_PRIORITIES: &str = "routes:
  - command: catalog.board.SetCardPriorities
    method: put
    path: /boards/{board_id}/cards/priority
    parameters: {board_id: input.board_id}
    request: {encoding: json, body: input.priorities}
";

#[test]
fn a_bare_list_body_is_an_array_request_schema() {
    let ir = model_of("board.yaml", BOARD);
    let document = bound_document(&ir, binding(&ir, BARE_PRIORITIES), "board-service");
    let put = &document["paths"]["/boards/{board_id}/cards/priority"]["put"];
    let body = &put["requestBody"];
    assert_eq!(keys(&body["content"]), ["application/json"], "{body}");
    assert_eq!(body["required"], true, "{body}");
    assert_eq!(
        body["content"]["application/json"]["schema"],
        json!({"type": "array", "items": {"$ref": "#/components/schemas/catalog.board.CardPriority"}}),
        "{body}"
    );
    let description = body["description"].as_str().expect("a description");
    assert!(description.contains("`priorities`"), "{description}");
    // No object wraps the list, and the item type it names is published.
    let schemas = &document["components"]["schemas"];
    assert!(
        schemas
            .get("catalog.board.SetCardPriorities.Input")
            .is_none(),
        "{schemas:#}"
    );
    assert_eq!(
        keys(&schemas["catalog.board.CardPriority"]["properties"]),
        ["id", "priority"]
    );
    // The board is the one path parameter.
    let parameters = put["parameters"].as_array().expect("parameters");
    assert_eq!(parameters.len(), 1, "{parameters:?}");
    assert_eq!(parameters[0]["name"], "board_id");
    assert_eq!(parameters[0]["in"], "path");
}

// Request limits (`docs/design/http-presentation-binding.md`, "What OpenAPI writes (k4)").

const EXTENSION: &str = "x-ess-request-max-bytes";

/// The schema of the default answer to a body larger than the bound.
fn too_large_schema() -> Value {
    json!({
        "type": "object",
        "additionalProperties": false,
        "required": ["refused"],
        "properties": {"refused": {"type": "string", "description": "Why the body was refused, in words."}},
    })
}

const ORDERS_VIEW: &str = "routes:
  - view: catalog.orders.OrderById
    method: get
    path: /orders/{order_id}
    parameters: {order_id: param.order_id}
    body: single
    empty: {status: 404}
";

fn orders_with(front: &str) -> Value {
    let ir = model_of("orders.yaml", ORDERS);
    bound_document(
        &ir,
        binding(&ir, &format!("{front}{ORDERS_ROUTES}")),
        "orders-service",
    )
}

fn bound_text(ir: &EssIr, routes: &str, component: &str) -> String {
    let artifacts = run(&BoundOpenApi(binding(ir, routes)), ir).expect("generates");
    artifacts[&format!("openapi/{component}.yaml")]
        .contents
        .clone()
}

fn digest_of(text: &str) -> String {
    ess_gen::ess_http::digest(without_digests(text).as_bytes())
}

/// The two orders rows were re-recorded when a `form` route with `unknown: ignore` (`PlaceOrder`)
/// started writing its `{command}.Input` with `additionalProperties: true` (beyond10x/ess#752).
#[test]
fn a_binding_that_declares_no_limit_keeps_every_stage_2_openapi_byte() {
    let orders = model_of("orders.yaml", ORDERS);
    let auth = model_of("auth.yaml", AUTH);
    let refused = format!(
        "refusals:
  malformed: {{status: 400, body: {{error: {{literal: invalid_request}}}}}}
  unfinished: {{status: 503, body: none, headers: {{Retry-After: {{literal: '120'}}}}}}
{ORDERS_ROUTES}"
    );
    let digests = [
        digest_of(&bound_text(&orders, ORDERS_ROUTES, "orders-service")),
        digest_of(&bound_text(&auth, AUTH_ROUTES, "auth-service")),
        digest_of(&bound_text(&orders, &refused, "orders-service")),
    ];
    assert_eq!(
        digests,
        [
            "sha256:58d9ead73146522044c1a8977ccb4a074426cfd15637c85ec481ad61497436a2",
            "sha256:3f9a5844122bf2eed0b5a920370eacc12397b6e153c59cc10998906ce2ffa7e3",
            "sha256:ca2bfe7d5d193be515362ff35c2a714f1c8fefd253ee8a69765ea3c894043ca9",
        ]
    );
}

#[test]
fn a_bounded_stage_1_route_documents_its_bound_and_one_more_response() {
    let ir = model();
    let limitless = bound(&ir, binding(&ir, ON_ONE_PATH));
    let document = bound(
        &ir,
        binding(
            &ir,
            &format!("limits: {{request_bytes: 64}}\n{ON_ONE_PATH}"),
        ),
    );
    let item = &document["paths"]["/items/{item_id}"];
    for method in ["put", "delete"] {
        let operation = &item[method];
        assert_eq!(operation[EXTENSION], json!(64), "{method}");
        let answer = &operation["responses"]["413"];
        assert!(answer.get("x-ess-outcomes").is_none(), "{answer}");
        assert_eq!(
            answer["content"]["application/json"]["schema"],
            too_large_schema()
        );
        let words = answer["description"].as_str().expect("description");
        assert!(words.starts_with("No declared outcome:"), "{words}");
        assert!(words.contains("64"), "{words}");
        let mut others = operation["responses"].clone();
        others.as_object_mut().expect("responses").remove("413");
        assert_eq!(
            others,
            limitless["paths"]["/items/{item_id}"][method]["responses"]
        );
    }
    let view = &item["get"];
    assert!(view.get(EXTENSION).is_none(), "{view}");
    assert!(view["responses"].get("413").is_none(), "{view}");
    let text = document.to_string();
    assert_eq!(text.matches("\"413\"").count(), 2, "{text}");
}

#[test]
fn a_bounded_route_in_the_bound_form_documents_the_same_answer() {
    let ir = model_of("orders.yaml", ORDERS);
    let (_, limitless) = orders();
    let document = orders_with("limits: {request_bytes: 64}\n");
    for path in ["/api/orders", "/api/orders/{order_id}/note"] {
        let operation = &document["paths"][path]["post"];
        assert_eq!(operation[EXTENSION], json!(64), "{path}");
        let answer = &operation["responses"]["413"];
        assert_eq!(answer["x-ess-outcomes"], json!([]), "{path}");
        assert_eq!(
            answer["content"]["application/json"]["schema"],
            too_large_schema()
        );
        let words = answer["description"].as_str().expect("description");
        assert!(words.starts_with("No declared outcome:"), "{words}");
        assert!(words.contains("64"), "{words}");
        let mut others = operation["responses"].clone();
        others.as_object_mut().expect("responses").remove("413");
        assert_eq!(
            others, limitless["paths"][path]["post"]["responses"],
            "{path}"
        );
    }
    let view = &document["paths"]["/orders/{order_id}"]["get"];
    assert!(view.get(EXTENSION).is_none(), "{view}");
    assert!(view["responses"].get("413").is_none(), "{view}");
    let unbound = unbound_document(&ir, "orders-service");
    let mut kept = 0;
    for (path, item) in unbound["paths"].as_object().expect("paths") {
        if let Some(bound) = document["paths"].get(path) {
            assert_eq!(bound, item, "{path}");
            kept += 1;
        }
    }
    assert!(kept > 0, "some derived route is left to compare");
}

#[test]
fn a_route_limit_and_a_declared_too_large_replace_the_binding_ones() {
    let ir = model_of("orders.yaml", ORDERS);
    let routes = ORDERS_ROUTES.replace(
        "    path: /api/orders/{order_id}/note\n",
        "    path: /api/orders/{order_id}/note\n    limits: {request_bytes: 256}\n    refusals: {too_large: {status: 413, body: none}}\n",
    );
    let routes = format!(
        "limits: {{request_bytes: 64}}
refusals:
  too_large: {{status: 418, body: {{error: {{literal: too_big}}}}, headers: {{X-Limit: {{literal: '64'}}}}}}
{routes}"
    );
    let document = bound_document(&ir, binding(&ir, &routes), "orders-service");
    let place = &document["paths"]["/api/orders"]["post"];
    assert_eq!(place[EXTENSION], json!(64));
    assert!(place["responses"].get("413").is_none(), "{place}");
    let refused = &place["responses"]["418"];
    assert_eq!(
        refused["content"]["application/json"]["schema"],
        json!({
            "type": "object",
            "additionalProperties": false,
            "required": ["error"],
            "properties": {"error": {"type": "string", "const": "too_big"}},
        })
    );
    assert_eq!(
        refused["headers"]["X-Limit"]["schema"],
        json!({"type": "string", "const": "64"})
    );
    let note = &document["paths"]["/api/orders/{order_id}/note"]["post"];
    assert_eq!(note[EXTENSION], json!(256));
    assert!(note["responses"]["413"].get("content").is_none(), "{note}");
    assert!(note["responses"].get("418").is_none(), "{note}");
}

#[test]
fn a_too_large_refusal_without_a_limit_documents_the_fixed_bound() {
    let document = orders_with("refusals: {too_large: {status: 413, body: none}}\n");
    assert_eq!(ess_http::FIXED_REQUEST_BYTES, 1_048_576);
    for path in ["/api/orders", "/api/orders/{order_id}/note"] {
        let operation = &document["paths"][path]["post"];
        assert_eq!(
            operation[EXTENSION],
            json!(ess_http::FIXED_REQUEST_BYTES),
            "{path}"
        );
        assert!(operation["responses"]["413"].get("content").is_none());
    }
    let view = &document["paths"]["/orders/{order_id}"]["get"];
    assert!(view.get(EXTENSION).is_none(), "{view}");
    assert!(view["responses"].get("413").is_none(), "{view}");
}

#[test]
fn a_limit_that_no_route_can_apply_changes_no_byte() {
    let ir = model_of("orders.yaml", ORDERS);
    let with = bound_text(
        &ir,
        &format!("limits: {{request_bytes: 64}}\n{ORDERS_VIEW}"),
        "orders-service",
    );
    assert_eq!(with, bound_text(&ir, ORDERS_VIEW, "orders-service"));
    assert!(!with.contains(EXTENSION) && !with.contains("413"), "{with}");
    let auth = model_of("auth.yaml", AUTH);
    let with = bound_text(
        &auth,
        &format!("limits: {{request_bytes: 64}}\n{AUTH_ROUTES}"),
        "auth-service",
    );
    assert_eq!(with, bound_text(&auth, AUTH_ROUTES, "auth-service"));
    assert!(!with.contains(EXTENSION), "{with}");
}

#[test]
fn the_description_names_the_bound_only_where_a_route_is_bounded() {
    let bounded = orders_with("limits: {request_bytes: 64}\n");
    let words = bounded["info"]["description"]
        .as_str()
        .expect("description");
    assert!(words.contains(EXTENSION), "{words}");
    let ir = model_of("orders.yaml", ORDERS);
    let view = format!("limits: {{request_bytes: 64}}\n{ORDERS_VIEW}");
    let document = bound_document(&ir, binding(&ir, &view), "orders-service");
    let plain = document["info"]["description"]
        .as_str()
        .expect("description");
    assert!(!plain.contains(EXTENSION), "{plain}");
    let (_, limitless) = orders();
    let held = limitless["info"]["description"]
        .as_str()
        .expect("description");
    assert!(
        words.starts_with(held),
        "the paragraphs it holds today are kept"
    );
}

/// [`ANSWER_FORMS`] with each answer under one named member (beyond10x/ess#755): the one order
/// under `order`, the orders of one size under `items`, and a placed order's response under
/// `data`.
fn wrapped_forms() -> String {
    let wrapped = ANSWER_FORMS
        .replace("    body: single\n", "    body: {order: single}\n")
        .replace("    body: array\n", "    body: {items: array}\n")
        .replace(
            "    path: /api/orders\n    refusals:",
            "    path: /api/orders\n    outcomes:\n      placed: {status: 201, body: {data: response}}\n    refusals:",
        );
    assert_eq!(wrapped.matches("{order: single}").count(), 1);
    assert_eq!(wrapped.matches("{items: array}").count(), 1);
    assert_eq!(wrapped.matches("{data: response}").count(), 1);
    wrapped
}

/// The object a wrapped answer is: its one member required, nothing else.
fn wrapper<'a>(response: &'a Value, member: &str) -> &'a Value {
    let schema = &response["content"]["application/json"]["schema"];
    assert_eq!(schema["type"], "object", "{response:#}");
    assert_eq!(schema["required"], json!([member]), "{response:#}");
    assert_eq!(schema["additionalProperties"], json!(false), "{response:#}");
    assert_eq!(keys(&schema["properties"]), [member], "{response:#}");
    &schema["properties"][member]
}

#[test]
fn wrapped_answers_project_an_object_of_one_member() {
    let ir = model_of("orders.yaml", ORDERS);
    let document = bound_document(&ir, binding(&ir, &wrapped_forms()), "orders-service");
    let paths = &document["paths"];

    // The one row under `order`; the empty answer as `empty` says.
    let one = &paths["/orders/{order_id}"]["get"]["responses"]["200"];
    let row = resolved(&document, wrapper(one, "order"));
    assert_eq!(
        keys(&row["properties"]),
        ["order_id", "quantity"],
        "{row:#}"
    );
    let description = one["description"].as_str().expect("a description");
    assert!(
        description.contains("`body: {order: single}`"),
        "{description}"
    );
    let empty = &paths["/orders/{order_id}"]["get"]["responses"]["404"];
    assert_eq!(literal(empty, "error"), "not_found", "{empty:#}");

    // The rows under `items`, an array of the row schema.
    let many = &paths["/orders/size/{quantity}"]["get"]["responses"]["200"];
    let items = wrapper(many, "items");
    assert_eq!(items["type"], "array", "{many:#}");
    let row = resolved(&document, &items["items"]);
    assert_eq!(keys(&row["properties"]), ["order_id"], "{row:#}");

    // The command's response under `data`.
    let placed = &paths["/api/orders"]["post"]["responses"]["201"];
    let data = wrapper(placed, "data");
    assert_eq!(
        data["$ref"], "#/components/schemas/catalog.orders.PlaceOrder.Result",
        "{placed:#}"
    );
    assert!(!resolved(&document, data).is_null());
}

/// A binding without a wrapped form writes the bytes it wrote before: the wrapped binding's
/// document differs only on the routes it wraps.
#[test]
fn a_binding_without_a_wrapped_form_keeps_its_document() {
    let ir = model_of("orders.yaml", ORDERS);
    let plain = bound_document(&ir, binding(&ir, ANSWER_FORMS), "orders-service");
    let wrapped = bound_document(&ir, binding(&ir, &wrapped_forms()), "orders-service");
    assert_ne!(plain, wrapped);
    assert_eq!(
        plain["paths"]["/api/orders/{order_id}/note"],
        wrapped["paths"]["/api/orders/{order_id}/note"]
    );
    let rows = &plain["paths"]["/orders/size/{quantity}"]["get"]["responses"]["200"];
    assert_eq!(
        rows["content"]["application/json"]["schema"]["type"],
        "array"
    );
}

/// Literal members and a count beside a wrapped answer (beyond10x/ess#755, decision HB-D4): each
/// literal a constant string, the count a whole number, every member required.
#[test]
fn literal_members_and_a_count_project_beside_the_wrapped_answer() {
    let ir = model_of("orders.yaml", ORDERS);
    let forms = wrapped_forms()
        .replace("{order: single}", "{order: single, success: {literal: ok}}")
        .replace(
            "{items: array}",
            "{items: array, success: {literal: ok}, total: count}",
        )
        .replace(
            "{data: response}",
            "{data: response, success: {literal: ok}}",
        );
    let document = bound_document(&ir, binding(&ir, &forms), "orders-service");
    let paths = &document["paths"];
    let schema = |response: &Value| response["content"]["application/json"]["schema"].clone();

    let one = schema(&paths["/orders/{order_id}"]["get"]["responses"]["200"]);
    assert_eq!(one["required"], json!(["order", "success"]), "{one:#}");
    assert_eq!(one["additionalProperties"], json!(false), "{one:#}");
    assert_eq!(
        one["properties"]["success"],
        json!({"type": "string", "const": "ok"}),
        "{one:#}"
    );

    let many = schema(&paths["/orders/size/{quantity}"]["get"]["responses"]["200"]);
    assert_eq!(
        many["required"],
        json!(["items", "success", "total"]),
        "{many:#}"
    );
    assert_eq!(
        many["properties"]["total"],
        json!({"type": "integer", "minimum": 0}),
        "{many:#}"
    );
    assert_eq!(many["properties"]["items"]["type"], "array", "{many:#}");

    let placed = schema(&paths["/api/orders"]["post"]["responses"]["201"]);
    assert_eq!(placed["required"], json!(["data", "success"]), "{placed:#}");
    assert_eq!(
        placed["properties"]["data"]["$ref"],
        "#/components/schemas/catalog.orders.PlaceOrder.Result",
        "{placed:#}"
    );
}

/// An identity that is a `Uuid` addresses its resource in the path (beyond10x/ess#755): the path
/// parameter is the field's schema, a reference to the newtype, which says `format: uuid`.
#[test]
fn a_uuid_identity_is_a_path_parameter_of_its_own_schema() {
    let text = ORDERS.replace(
        "  - {name: catalog.orders.OrderId, kind: newtype, of: String}\n",
        "  - {name: catalog.orders.OrderId, kind: newtype, of: Uuid}\n",
    );
    assert_ne!(text, ORDERS, "the fixture moved");
    let ir = model_of("orders.yaml", &text);
    let document = bound_document(&ir, binding(&ir, ORDERS_ROUTES), "orders-service");
    for (path, method) in [
        ("/api/orders/{order_id}/note", "post"),
        ("/orders/{order_id}", "get"),
    ] {
        let parameters = document["paths"][path][method]["parameters"]
            .as_array()
            .expect("parameters");
        let carried: Vec<&Value> = parameters
            .iter()
            .filter(|parameter| parameter["in"] == "path")
            .collect();
        assert_eq!(carried.len(), 1, "{path}: {parameters:?}");
        assert_eq!(carried[0]["name"], "order_id", "{path}");
        assert_eq!(
            carried[0]["schema"],
            json!({"$ref": "#/components/schemas/catalog.orders.OrderId"}),
            "{path}"
        );
    }
    let id = &document["components"]["schemas"]["catalog.orders.OrderId"];
    assert_eq!(id["type"], "string", "{id}");
    assert_eq!(id["format"], "uuid", "{id}");
}
