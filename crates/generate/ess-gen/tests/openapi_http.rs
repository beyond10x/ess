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

/// The items service's document as 0.60.0 generated it from [`ON_ONE_PATH`], its provenance
/// digests written `SOURCE_DIGEST` and `CONTRACT_DIGEST`: the digests are of the compiled
/// specification, which another ESS release may spell differently, while every other byte is the
/// projection's own.
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
    let source = provenance["source_digest"].as_str().expect("source digest");
    let contract = provenance["contract_digest"]
        .as_str()
        .expect("contract digest");
    text.replace(source, "SOURCE_DIGEST")
        .replace(contract, "CONTRACT_DIGEST")
}

#[test]
fn a_stage_1_binding_keeps_its_openapi_bytes() {
    let ir = model();
    let artifacts = run(&BoundOpenApi(binding(&ir, ON_ONE_PATH)), &ir).expect("generates");
    let text = without_digests(&artifacts["openapi/items-service.yaml"].contents);
    assert!(text.contains("SOURCE_DIGEST") && text.contains("CONTRACT_DIGEST"));
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
