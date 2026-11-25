//! The `OpenAPI` projection of an `ess-http/1` binding's operation ids and servers
//! (<https://github.com/beyond10x/ess/issues/653>, `docs/design/http-presentation-binding.md`,
//! "Operation ids and servers").
//!
//! A route's `operation_id` is its operation's `operationId`; an operation without one keeps its
//! qualified name. A component's `servers` are that component's document `servers`, after `info`,
//! in the order written, and no other document has any.
//!
//! A `multipart` route's request body is `multipart/form-data`, one part per input field, a file
//! part a string of raw bytes (beyond10x/ess#754,
//! `story:openapi-publishes-a-multipart-request-with-file-parts`). A field `request.query` lists is
//! an `in: query` parameter and in no request body (beyond10x/ess#754,
//! `story:a-command-route-reads-chosen-inputs-from-the-query-beside-its-body`).

use std::fmt::Write as _;

use ess_compiler::resolve::compile as compile_model;
use ess_compiler::source::SourceMap;
use ess_compiler::EssIr;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_gen::artifact::run;
use ess_gen::http::{derived, Binding};
use ess_gen::openapi::{bound_json, BoundOpenApi};
use serde_json::{json, Value};

const MODEL: &str = include_str!("../../../specify/ess-http/tests/fixtures/catalog.yaml");

/// A second file of the same system: `items-worker`, which declares no `reached_by`.
const WORKER: &str = include_str!("../../../specify/ess-http/tests/fixtures/worker.yaml");

fn model_of(files: &[(&str, &str)]) -> EssIr {
    let raws = files.iter().map(|(file, text)| {
        let raw = RawSpecFile::parse(text).expect("well formed");
        (Source::new(*file), raw)
    });
    let spec = Specification::assemble(raws).unwrap_or_else(|errors| panic!("validates: {errors}"));
    let mut sources = SourceMap::new();
    for (file, text) in files {
        sources.insert(*file, *text);
    }
    compile_model(&spec, &sources).unwrap_or_else(|errors| panic!("compiles: {errors}"))
}

fn binding(ir: &EssIr, keys: &str) -> Binding {
    let text = format!(
        "type: ess-http/1\nspecification:\n  system: {}\n  version: v1\n  source_digest: sha256:{}\n{keys}",
        ir.system(),
        ir.source_digest()
    );
    let spec = ess_http::HttpSpec::from_yaml(&text).expect("parses");
    let http = ess_http::compile(&spec, ir, &derived(ir))
        .unwrap_or_else(|diagnostics| panic!("compiles: {diagnostics}"));
    Binding::new(http).expect("every route has a path")
}

fn documents(ir: &EssIr, binding: Binding) -> std::collections::BTreeMap<String, String> {
    run(&BoundOpenApi(binding), ir)
        .expect("generates")
        .into_iter()
        .map(|(path, artifact)| (path, artifact.contents))
        .collect()
}

fn as_json(text: &str) -> Value {
    let yaml: ess_yaml::Value = ess_yaml::from_str(text).expect("YAML");
    serde_json::to_value(yaml).expect("JSON")
}

#[test]
fn a_route_operation_id_is_projected() {
    let ir = model_of(&[("catalog.yaml", MODEL)]);
    let documents = documents(
        &ir,
        binding(
            &ir,
            "routes:
  - command: catalog.items.ListItem
    operation_id: listItem
    method: put
    path: /items/{item_id}
    parameters: {item_id: input.item_id}
  - view: catalog.items.ItemById
    method: get
    path: /items/{item_id}
    parameters: {item_id: param.item_id}
",
        ),
    );
    let document = as_json(&documents["openapi/items-service.yaml"]);
    let item = &document["paths"]["/items/{item_id}"];
    assert_eq!(item["put"]["operationId"], "listItem");
    assert_eq!(
        item["get"]["operationId"], "catalog.items.ItemById",
        "a bound route without an id keeps its qualified name"
    );
    assert_eq!(
        document["paths"]["/items/commands/TouchItem"]["post"]["operationId"],
        "catalog.items.TouchItem",
        "a derived route keeps its qualified name"
    );
}

#[test]
fn servers_are_projected_on_their_components_document() {
    let ir = model_of(&[("catalog.yaml", MODEL), ("worker.yaml", WORKER)]);
    let keys = "servers:
  items-service:
    - {url: 'https://api.example.test/v2', description: Production}
    - {url: /v2}
routes:
  - command: catalog.items.ListItem
    method: post
    path: /items
";
    let documents = documents(&ir, binding(&ir, keys));
    let service = &documents["openapi/items-service.yaml"];
    let document = as_json(service);
    assert_eq!(
        document["servers"],
        json!([
            {"url": "https://api.example.test/v2", "description": "Production"},
            {"url": "/v2"}
        ]),
        "{service}"
    );
    let at = |key: &str| {
        service
            .find(&format!("\n{key}:"))
            .unwrap_or_else(|| panic!("no top-level `{key}` in {service}"))
    };
    assert!(
        at("info") < at("servers") && at("servers") < at("paths"),
        "`servers` follows `info`: {service}"
    );
    let worker = as_json(&documents["openapi/items-worker.yaml"]);
    assert!(worker.get("servers").is_none(), "{worker}");

    // The served copy carries them too.
    let component = ir
        .components()
        .values()
        .find(|component| component.name.to_string() == "items-service")
        .expect("component");
    let served: Value =
        serde_json::from_str(&bound_json(&ir, component, Some(&binding(&ir, keys)))).unwrap();
    assert_eq!(served["servers"], document["servers"]);
}

/// The model the `multipart` routes bind: a file with its filename beside plain fields, an
/// optional photo with its name and media type, and a list of files.
const IMPORTS: &str = include_str!("../../../specify/ess-http/tests/fixtures/imports.yaml");

const MULTIPART: &str = "routes:
  - command: demo.imports.ImportRows
    method: post
    path: /reports/rows/import
    request:
      encoding: multipart
      parts:
        file:
          filename: input.file_name
          media_types: [text/csv, application/vnd.ms-excel]
  - command: demo.imports.AttachPhoto
    method: post
    path: /reports/{report_id}/photos
    parameters: {report_id: input.report_id}
    request:
      encoding: multipart
      unknown: ignore
      parts:
        photo: {filename: input.photo_name, media_type: input.photo_type}
  - command: demo.imports.AttachFiles
    method: post
    path: /reports/{report_id}/files
    parameters: {report_id: input.report_id}
    request: {encoding: multipart}
";

#[test]
fn a_multipart_route_publishes_its_parts_as_multipart_form_data() {
    let ir = model_of(&[("imports.yaml", IMPORTS)]);
    let documents = documents(&ir, binding(&ir, MULTIPART));
    let document = as_json(&documents["openapi/imports.yaml"]);
    let body = &document["paths"]["/reports/rows/import"]["post"]["requestBody"];
    assert_eq!(
        body["content"].as_object().map(serde_json::Map::len),
        Some(1),
        "{body}"
    );
    assert_eq!(body["required"], true, "{body}");
    let media = &body["content"]["multipart/form-data"];
    let schema = &media["schema"];
    assert_eq!(
        schema["properties"]["file"],
        json!({
            "type": "string",
            "contentMediaType": "application/octet-stream",
            "description": "A file part, holding the bytes as sent. Its filename is read into the \
                            input field `file_name`.",
        }),
        "{schema}"
    );
    let mut named: Vec<&str> = schema["properties"]
        .as_object()
        .expect("properties")
        .keys()
        .map(String::as_str)
        .collect();
    named.sort_unstable();
    assert_eq!(named, ["dry_run", "file", "format"], "{schema}");
    assert!(
        schema["properties"]["format"]["$ref"]
            .as_str()
            .is_some_and(|reference| reference.ends_with("demo.imports.Format")),
        "{schema}"
    );
    assert_eq!(schema["properties"]["dry_run"]["type"], "boolean");
    assert_eq!(schema["required"], json!(["file", "format", "dry_run"]));
    assert_eq!(schema["additionalProperties"], false);
    assert_eq!(
        media["encoding"],
        json!({"file": {"contentType": "text/csv, application/vnd.ms-excel"}})
    );
    let components = &document["components"]["schemas"];
    assert!(
        components.get("demo.imports.ImportRows.Input").is_none(),
        "a multipart route's parts are not `{{command}}.Input`"
    );

    // An `Optional` photo whose filename and media type are read into the input, beside a path
    // variable, under `unknown: ignore`.
    let body = &document["paths"]["/reports/{report_id}/photos"]["post"]["requestBody"];
    let media = &body["content"]["multipart/form-data"];
    let schema = &media["schema"];
    assert_eq!(
        schema["properties"]["photo"],
        json!({
            "type": "string",
            "contentMediaType": "application/octet-stream",
            "description": "A file part, holding the bytes as sent. Its filename is read into the \
                            input field `photo_name`. Its media type is read into the input field \
                            `photo_type`.",
        }),
        "{schema}"
    );
    for absent in ["photo_name", "photo_type", "report_id"] {
        assert!(
            schema["properties"].get(absent).is_none(),
            "{absent}: {schema}"
        );
    }
    // `tags`, a `List`, repeats its part once per element: no part is the empty list, as both
    // servers read it, so the part is not required.
    assert_eq!(schema["required"], json!(["caption", "count"]));
    assert_eq!(schema["additionalProperties"], true);
    assert!(media.get("encoding").is_none(), "{media}");
    assert!(
        body["description"]
            .as_str()
            .is_some_and(|text| text.contains("A part no field names is ignored")),
        "{body}"
    );

    // A `List<Bytes>`: one file part per element.
    let schema = &document["paths"]["/reports/{report_id}/files"]["post"]["requestBody"]["content"]
        ["multipart/form-data"]["schema"];
    assert_eq!(
        schema["properties"]["files"],
        json!({
            "type": "array",
            "items": {"type": "string", "contentMediaType": "application/octet-stream"},
            "description": "One file part per element, each holding the bytes as sent.",
        }),
        "{schema}"
    );
}

/// Each `in: query` parameter of `operation`, by name.
fn query_parameters(operation: &Value) -> Vec<&Value> {
    operation["parameters"]
        .as_array()
        .map(|parameters| {
            parameters
                .iter()
                .filter(|parameter| parameter["in"] == "query")
                .collect()
        })
        .unwrap_or_default()
}

#[test]
fn a_listed_query_field_is_a_query_parameter_and_in_no_body() {
    // A `json` route: `item_id` from the query string, `title` in the body.
    let ir = model_of(&[("catalog.yaml", MODEL)]);
    let written = documents(
        &ir,
        binding(
            &ir,
            "routes:
  - command: catalog.items.ListItem
    method: post
    path: /items
    request: {query: [input.item_id]}
",
        ),
    );
    let document = as_json(&written["openapi/items-service.yaml"]);
    let operation = &document["paths"]["/items"]["post"];
    let listed = query_parameters(operation);
    assert_eq!(listed.len(), 1, "{operation}");
    assert_eq!(listed[0]["name"], "item_id");
    assert_eq!(listed[0]["required"], true);
    assert!(listed[0].get("style").is_none(), "{operation}");
    let input = &document["components"]["schemas"]["catalog.items.ListItem.Input"];
    assert!(input["properties"].get("item_id").is_none(), "{input}");
    assert!(input["properties"].get("title").is_some(), "{input}");
    assert_eq!(input["required"], json!(["title"]));

    // A `multipart` route: `dry_run` from the query string, not a part.
    let ir = model_of(&[("imports.yaml", IMPORTS)]);
    let routes = MULTIPART
        .replace(
            "          media_types: [text/csv, application/vnd.ms-excel]\n",
            "          media_types: [text/csv, application/vnd.ms-excel]\n      query: [input.dry_run]\n",
        )
        .replace(
            "      unknown: ignore\n",
            "      unknown: ignore\n      query: [input.tags]\n",
        );
    assert_eq!(routes.matches("query: [").count(), 2);
    let written = documents(&ir, binding(&ir, &routes));
    let document = as_json(&written["openapi/imports.yaml"]);
    let operation = &document["paths"]["/reports/rows/import"]["post"];
    let listed = query_parameters(operation);
    assert_eq!(listed.len(), 1, "{operation}");
    assert_eq!(listed[0]["name"], "dry_run");
    assert_eq!(listed[0]["required"], true);
    assert_eq!(listed[0]["schema"]["type"], "boolean");
    let schema = &operation["requestBody"]["content"]["multipart/form-data"]["schema"];
    assert!(schema["properties"].get("dry_run").is_none(), "{schema}");
    assert_eq!(schema["required"], json!(["file", "format"]));

    // A `List` repeats its key, as under `encoding: query`, and is not required.
    let operation = &document["paths"]["/reports/{report_id}/photos"]["post"];
    let listed = query_parameters(operation);
    assert_eq!(listed.len(), 1, "{operation}");
    assert_eq!(
        (
            &listed[0]["name"],
            &listed[0]["required"],
            &listed[0]["style"],
            &listed[0]["explode"]
        ),
        (&json!("tags"), &json!(false), &json!("form"), &json!(true))
    );
    let schema = &operation["requestBody"]["content"]["multipart/form-data"]["schema"];
    assert!(schema["properties"].get("tags").is_none(), "{schema}");
}

// --- `request.members` (`ae1d-a-json-request-reads-an-input-from-a-nested-member`) ---

const BOARD: &str = include_str!("../../../specify/ess-http/tests/fixtures/board.yaml");

#[test]
fn a_nested_member_is_written_nested_with_required_per_level() {
    let ir = model_of(&[("board.yaml", BOARD)]);
    let routes = "routes:
  - command: catalog.board.RenameBoard
    method: post
    path: /hooks
    request: {members: {board_id: event.board_id, title: event.data.title, note: event.data.note}}
";
    let written = documents(&ir, binding(&ir, routes));
    let document = as_json(&written["openapi/board-service.yaml"]);
    let input = &document["components"]["schemas"]["catalog.board.RenameBoard.Input"];
    assert_eq!(
        input["properties"]
            .as_object()
            .map(|object| object.keys().collect::<Vec<_>>()),
        Some(vec![&"event".to_owned()]),
        "{input}"
    );
    assert_eq!(input["required"], json!(["event"]), "{input}");
    let event = &input["properties"]["event"];
    assert_eq!(event["type"], "object", "{event}");
    assert_eq!(event["additionalProperties"], false, "{event}");
    assert_eq!(event["required"], json!(["board_id", "data"]), "{event}");
    let data = &event["properties"]["data"];
    assert_eq!(data["required"], json!(["title"]), "{data}");
    assert_eq!(data["properties"]["title"]["type"], "string", "{data}");
    assert!(data["properties"].get("note").is_some(), "{data}");
    assert_eq!(
        document["paths"]["/hooks"]["post"]["requestBody"]["content"]["application/json"]["schema"]
            ["$ref"],
        "#/components/schemas/catalog.board.RenameBoard.Input"
    );

    // An object holding only `Optional` fields is not required; `unknown: ignore` opens each level.
    let routes = "routes:
  - command: catalog.board.RenameBoard
    method: post
    path: /hooks
    request: {unknown: ignore, members: {note: meta.note}}
";
    let written = documents(&ir, binding(&ir, routes));
    let document = as_json(&written["openapi/board-service.yaml"]);
    let input = &document["components"]["schemas"]["catalog.board.RenameBoard.Input"];
    assert_eq!(input["required"], json!(["board_id", "title"]), "{input}");
    assert_eq!(
        input["properties"]["meta"]["additionalProperties"], true,
        "{input}"
    );
    assert!(
        input["properties"]["meta"].get("required").is_none(),
        "{input}"
    );

    // Without `members` the schema is flat, as before.
    let routes = "routes:
  - command: catalog.board.RenameBoard
    method: post
    path: /hooks
";
    let written = documents(&ir, binding(&ir, routes));
    let document = as_json(&written["openapi/board-service.yaml"]);
    let input = &document["components"]["schemas"]["catalog.board.RenameBoard.Input"];
    assert_eq!(input["required"], json!(["board_id", "title"]), "{input}");
    assert!(input["properties"].get("note").is_some(), "{input}");
}

// --- a shared route (`ae1b-openapi-writes-a-shared-route-as-one-operation`) ---

/// A desk accepting `catalog.desk.Hook0` to `Hook<count - 1>`, each taking `ticket_id`, a `String`
/// `kind` and an `Optional<Boolean>` `reopen`.
fn desk(count: usize) -> EssIr {
    let names: Vec<String> = (0..count)
        .map(|n| format!("catalog.desk.Hook{n}"))
        .collect();
    let commands = names.iter().fold(String::new(), |mut commands, name| {
        write!(
            commands,
            "  - name: {name}
    input:
      - {{name: ticket_id, type: catalog.desk.TicketId}}
      - {{name: kind, type: String}}
      - {{name: reopen, type: Optional<Boolean>}}
    outcomes:
      - name: done
        emits: [catalog.desk.Hooked]
        payload:
          catalog.desk.Hooked: {{ticket_id: input.ticket_id}}
"
        )
        .expect("writes");
        commands
    });
    let text = format!(
        "format: ess/22
system: catalog
version: v1
domain: catalog.desk
types:
  - {{name: catalog.desk.TicketId, kind: newtype, of: String}}
entities:
  - name: catalog.desk.Ticket
    identity: {{name: ticket_id, type: catalog.desk.TicketId}}
    fields:
      - {{name: title, type: String}}
    lifecycle: {{initial: Open, states: [Open], terminal: [Open], transitions: []}}
events:
  - name: catalog.desk.Hooked
    fields:
      - {{name: ticket_id, type: catalog.desk.TicketId}}
commands:
{commands}components:
  - component: desk
    reached_by: network
    owns: {{domains: [catalog.desk]}}
    accepts:
      commands: [{}]
    publishes:
      events: [catalog.desk.Hooked]
",
        names.join(", ")
    );
    model_of(&[("desk.yaml", text.as_str())])
}

const OPEN_SET: &str = "routes:
  - {command: catalog.desk.Hook2, method: post, path: /hooks, select: {kind: closed, data.reopen: true}}
  - {command: catalog.desk.Hook0, method: post, path: /hooks, select: {kind: opened}}
  - command: catalog.desk.Hook3
    method: post
    path: /hooks
    outcomes: {done: {status: 204, body: none}}
  - {command: catalog.desk.Hook1, method: post, path: /hooks, select: {kind: closed, data.reopen: false}}
";

fn shared_document(ir: &EssIr, routes: &str) -> (String, Value) {
    let written = documents(ir, binding(ir, routes));
    let text = written["openapi/desk.yaml"].clone();
    let document = as_json(&text);
    (text, document)
}

#[test]
fn a_shared_route_is_one_operation_naming_every_command() {
    let ir = desk(4);
    let (text, document) = shared_document(&ir, OPEN_SET);
    let operation = &document["paths"]["/hooks"]["post"];
    let listed = operation["x-ess-operations"].as_array().expect("listed");
    assert_eq!(listed.len(), 4, "{operation}");
    assert_eq!(operation["operationId"], "catalog.desk.Hook0");
    assert_eq!(
        listed[0],
        json!({"operationId": "catalog.desk.Hook0", "command": "catalog.desk.Hook0", "select": {"kind": "opened"}})
    );
    assert_eq!(
        listed[1]["select"],
        json!({"data.reopen": false, "kind": "closed"})
    );
    assert_eq!(listed[3]["select"], "otherwise");
    let schema = &operation["requestBody"]["content"]["application/json"]["schema"];
    let one_of = schema["oneOf"].as_array().expect("oneOf");
    assert_eq!(one_of.len(), 4, "{schema}");
    assert!(schema.get("discriminator").is_none(), "{schema}");
    assert_eq!(
        one_of[0]["$ref"],
        "#/components/schemas/catalog.desk.Hook0.Input"
    );
    assert!(one_of[3]["description"].is_string(), "{schema}");
    let schemas = &document["components"]["schemas"];
    let opened = &schemas["catalog.desk.Hook0.Input"];
    assert_eq!(opened["properties"]["kind"]["const"], "opened", "{opened}");
    let reopened = &schemas["catalog.desk.Hook2.Input"];
    assert_eq!(
        reopened["properties"]["data"]["properties"]["reopen"]["const"], true,
        "{reopened}"
    );
    assert!(
        reopened["required"]
            .as_array()
            .is_some_and(|required| required.contains(&json!("data"))),
        "{reopened}"
    );
    let catch_all = &schemas["catalog.desk.Hook3.Input"];
    assert!(
        catch_all["properties"]["kind"].get("const").is_none(),
        "{catch_all}"
    );
    // The catch-all's `204` and the others' answers are merged per status.
    let responses = operation["responses"].as_object().expect("responses");
    assert!(responses.contains_key("204"), "{operation}");
    assert!(responses.len() >= 2, "{operation}");
    // The same routes written in another order give the same bytes.
    let swapped = OPEN_SET.replace(
        "  - {command: catalog.desk.Hook2, method: post, path: /hooks, select: {kind: closed, data.reopen: true}}\n  - {command: catalog.desk.Hook0, method: post, path: /hooks, select: {kind: opened}}\n",
        "  - {command: catalog.desk.Hook0, method: post, path: /hooks, select: {kind: opened}}\n  - {command: catalog.desk.Hook2, method: post, path: /hooks, select: {kind: closed, data.reopen: true}}\n",
    );
    assert_ne!(swapped, OPEN_SET);
    assert_eq!(shared_document(&ir, &swapped).0, text);
}

#[test]
fn a_closed_set_states_its_malformed_answer_and_one_string_member_is_a_discriminator() {
    let ir = desk(9);
    let mut routes = String::from("routes:\n");
    let mut n = 0;
    for object in ["label", "ticket", "user"] {
        for action in ["added", "removed", "changed"] {
            writeln!(
                routes,
                "  - {{command: catalog.desk.Hook{n}, method: post, path: /v2/hooks, select: {{object: {object}, action: {action}}}}}"
            )
            .expect("writes");
            n += 1;
        }
    }
    let (_, document) = shared_document(&ir, &routes);
    let operation = &document["paths"]["/v2/hooks"]["post"];
    let schema = &operation["requestBody"]["content"]["application/json"]["schema"];
    assert_eq!(
        schema["oneOf"].as_array().map(Vec::len),
        Some(9),
        "{schema}"
    );
    assert!(schema.get("discriminator").is_none(), "{schema}");
    let input = &document["components"]["schemas"]["catalog.desk.Hook4.Input"];
    assert_eq!(input["properties"]["object"]["const"], "ticket", "{input}");
    assert_eq!(input["properties"]["action"]["const"], "removed", "{input}");
    let unselected = operation["responses"]
        .as_object()
        .expect("responses")
        .values()
        .any(|response| {
            response["description"]
                .as_str()
                .is_some_and(|text| text.contains("no route selects"))
        });
    assert!(unselected, "{operation}");

    // One top-level `String` member, a closed set: OpenAPI's discriminator.
    let routes = "routes:
  - {command: catalog.desk.Hook0, method: post, path: /v3/hooks, select: {kind: a}}
  - {command: catalog.desk.Hook1, method: post, path: /v3/hooks, select: [{kind: b}, {kind: c}]}
";
    let (_, document) = shared_document(&ir, routes);
    let schema = &document["paths"]["/v3/hooks"]["post"]["requestBody"]["content"]
        ["application/json"]["schema"];
    assert_eq!(
        schema["discriminator"],
        json!({"propertyName": "kind", "mapping": {
            "a": "#/components/schemas/catalog.desk.Hook0.Input",
            "b": "#/components/schemas/catalog.desk.Hook1.Input",
            "c": "#/components/schemas/catalog.desk.Hook1.Input"
        }}),
        "{schema}"
    );
}

// --- authentication (beyond10x/ess#681, `story:an-http-binding-projects-its-security-schemes`) ---

/// The orders fixture with an actor granted both commands: `authentication` needs an actor.
fn guarded_orders() -> EssIr {
    let text = include_str!("../../../specify/ess-http/tests/fixtures/orders.yaml").to_owned()
        + "actors:\n  - name: catalog.orders.Clerk\n    may: [catalog.orders.NoteOrder, catalog.orders.PlaceOrder]\n";
    model_of(&[("orders.yaml", &text)])
}

/// Four schemes, one of each kind; `PlaceOrder` takes a bearer token or a key, `NoteOrder` a
/// `basic` credential or a client certificate and answers its own `unauthenticated`, and the view
/// is open.
const GUARDED: &str = "authentication:
  schemes:
    bearer_auth: {kind: bearer}
    key_auth: {kind: api_key, in: header, name: X-Api-Key}
    basic_auth: {kind: basic, realm: orders}
    tls: {kind: mutual_tls}
  required: [bearer_auth]
routes:
  - command: catalog.orders.PlaceOrder
    method: post
    path: /orders
    authentication: [bearer_auth, key_auth]
  - command: catalog.orders.NoteOrder
    method: post
    path: /orders/note
    authentication: [basic_auth, tls]
    refusals:
      unauthenticated:
        status: 401
        body: {error: {literal: login_required}}
        headers:
          WWW-Authenticate: {challenge: {scheme: Basic, params: {realm: {literal: orders}}}}
  - view: catalog.orders.OrderById
    method: get
    path: /orders/{order_id}
    parameters: {order_id: param.order_id}
    authentication: []
";

#[test]
fn a_binding_projects_its_security_schemes_and_each_routes_requirement() {
    let ir = guarded_orders();
    let written = documents(&ir, binding(&ir, GUARDED));
    let text = &written["openapi/orders-service.yaml"];
    let document = as_json(text);
    assert_eq!(
        document["components"]["securitySchemes"],
        json!({
            "basic_auth": {"type": "http", "scheme": "basic"},
            "bearer_auth": {"type": "http", "scheme": "bearer"},
            "key_auth": {"type": "apiKey", "in": "header", "name": "X-Api-Key"},
            "tls": {"type": "mutualTLS"},
        }),
        "{text}"
    );
    let paths = &document["paths"];
    assert_eq!(
        paths["/orders"]["post"]["security"],
        json!([{"bearer_auth": []}, {"key_auth": []}])
    );
    assert_eq!(
        paths["/orders/note"]["post"]["security"],
        json!([{"basic_auth": []}, {"tls": []}])
    );
    assert_eq!(paths["/orders/{order_id}"]["get"]["security"], json!([]));
    // The credential is no ordinary input: no parameter carries it.
    for operation in [&paths["/orders"]["post"], &paths["/orders/note"]["post"]] {
        for parameter in operation["parameters"].as_array().into_iter().flatten() {
            let name = parameter["name"].as_str().unwrap_or_default();
            assert!(
                !name.eq_ignore_ascii_case("authorization")
                    && !name.eq_ignore_ascii_case("x-api-key"),
                "{operation}"
            );
        }
    }
}

#[test]
fn a_guarded_operation_lists_its_unauthenticated_answer() {
    let ir = guarded_orders();
    let written = documents(&ir, binding(&ir, GUARDED));
    let document = as_json(&written["openapi/orders-service.yaml"]);
    let paths = &document["paths"];

    // The default: `401`, one `Bearer` challenge, `{"refused": "unauthenticated"}`.
    let default = &paths["/orders"]["post"]["responses"]["401"];
    let challenge = &default["headers"]["WWW-Authenticate"];
    assert_eq!(challenge["required"], true, "{default}");
    assert!(
        challenge["description"]
            .as_str()
            .is_some_and(|text| text.contains("`Bearer`")),
        "{default}"
    );
    assert_eq!(
        default["content"]["application/json"]["schema"]["properties"]["refused"]["const"],
        "unauthenticated",
        "{default}"
    );

    // The route's override.
    let own = &paths["/orders/note"]["post"]["responses"]["401"];
    assert_eq!(
        own["content"]["application/json"]["schema"]["properties"]["error"]["const"],
        "login_required",
        "{own}"
    );
    assert!(own["headers"].get("WWW-Authenticate").is_some(), "{own}");

    // An open route answers no `401`.
    assert!(
        paths["/orders/{order_id}"]["get"]["responses"]
            .get("401")
            .is_none(),
        "{document}"
    );
}

#[test]
fn a_binding_without_authentication_projects_no_security() {
    let ir = guarded_orders();
    let open = "routes:
  - command: catalog.orders.PlaceOrder
    method: post
    path: /orders
  - command: catalog.orders.NoteOrder
    method: post
    path: /orders/note
  - view: catalog.orders.OrderById
    method: get
    path: /orders/{order_id}
    parameters: {order_id: param.order_id}
";
    let written = documents(&ir, binding(&ir, open));
    let text = &written["openapi/orders-service.yaml"];
    assert!(!text.contains("unauthenticated"), "{text}");
    let document = as_json(text);
    assert!(
        document["components"].get("securitySchemes").is_none(),
        "{text}"
    );
    for (path, item) in document["paths"].as_object().expect("paths") {
        for (method, operation) in item.as_object().expect("path item") {
            assert!(
                operation.get("security").is_none() && operation["responses"].get("401").is_none(),
                "{method} {path}: {operation}"
            );
        }
    }
}

// --- `cors:` (`ae3a-a-binding-declares-which-origins-may-read-its-answers`) ---

const CORS_ROUTES: &str = "routes:
  - command: catalog.items.ListItem
    method: put
    path: /items/{item_id}
    parameters: {item_id: input.item_id}
  - view: catalog.items.ItemById
    method: get
    path: /items/{item_id}
    parameters: {item_id: param.item_id}
";

fn cors_document(ir: &EssIr, keys: &str) -> (String, Value) {
    let written = documents(ir, binding(ir, keys));
    let text = written["openapi/items-service.yaml"].clone();
    let document = as_json(&text);
    (text, document)
}

/// The `const` or `enum` of one header of `response`.
fn header_schema<'a>(response: &'a Value, name: &str) -> &'a Value {
    &response["headers"][name]["schema"]
}

#[test]
fn a_cors_policy_writes_the_preflight_and_the_origin_header_of_each_covered_answer() {
    let ir = model_of(&[("catalog.yaml", MODEL)]);
    let policy = "cors:
  origins: [https://desk.example.test]
  credentials: true
  headers: [Authorization]
  max_age: 600
";
    let (_, document) = cors_document(&ir, &format!("{policy}{CORS_ROUTES}"));
    assert_eq!(
        document["x-ess-cors"],
        json!({
            "origins": ["https://desk.example.test"],
            "credentials": true,
            "headers": ["Authorization"],
            "max_age": 600
        })
    );
    let item = &document["paths"]["/items/{item_id}"];
    let preflight = &item["options"]["responses"]["204"];
    assert!(preflight.is_object(), "{item}");
    assert!(item["options"].get("operationId").is_none(), "{item}");
    assert_eq!(
        header_schema(preflight, "Access-Control-Allow-Methods")["const"],
        "GET, PUT"
    );
    assert_eq!(
        header_schema(preflight, "Access-Control-Allow-Origin")["enum"],
        json!(["https://desk.example.test"])
    );
    assert_eq!(
        header_schema(preflight, "Access-Control-Allow-Headers")["const"],
        "Authorization"
    );
    assert_eq!(
        header_schema(preflight, "Access-Control-Allow-Credentials")["const"],
        "true"
    );
    assert_eq!(
        header_schema(preflight, "Access-Control-Max-Age")["const"],
        600
    );
    assert_eq!(header_schema(preflight, "Vary")["const"], "Origin");
    for method in ["put", "get"] {
        let responses = item[method]["responses"].as_object().expect("responses");
        assert!(!responses.is_empty());
        for (status, response) in responses {
            assert_eq!(
                header_schema(response, "Access-Control-Allow-Origin")["enum"],
                json!(["https://desk.example.test"]),
                "{method} {status}"
            );
            assert_eq!(
                header_schema(response, "Vary")["const"],
                "Origin",
                "{method} {status}"
            );
        }
    }
    // The binding's policy covers a derived route too.
    let derived = &document["paths"]["/items/commands/TouchItem"];
    assert_eq!(
        header_schema(
            &derived["options"]["responses"]["204"],
            "Access-Control-Allow-Methods"
        )["const"],
        "POST"
    );
}

#[test]
fn a_cors_policy_of_any_origin_none_or_a_routes_own_writes_its_own_headers() {
    let ir = model_of(&[("catalog.yaml", MODEL)]);
    // Any origin: `*`, and no `Vary`.
    let (_, document) = cors_document(&ir, &format!("cors: {{origins: any}}\n{CORS_ROUTES}"));
    let item = &document["paths"]["/items/{item_id}"];
    let answer = item["put"]["responses"]
        .as_object()
        .and_then(|responses| responses.values().next())
        .expect("an answer");
    assert_eq!(
        header_schema(answer, "Access-Control-Allow-Origin")["const"],
        "*"
    );
    assert!(answer["headers"].get("Vary").is_none(), "{answer}");
    let preflight = &item["options"]["responses"]["204"];
    for absent in [
        "Access-Control-Allow-Headers",
        "Access-Control-Allow-Credentials",
        "Access-Control-Max-Age",
        "Vary",
    ] {
        assert!(preflight["headers"].get(absent).is_none(), "{absent}");
    }
    // A route under `cors: none` gets no header and leaves the preflight's methods.
    let closed = CORS_ROUTES.replace(
        "    parameters: {item_id: param.item_id}\n",
        "    parameters: {item_id: param.item_id}\n    cors: none\n",
    );
    let (_, document) = cors_document(&ir, &format!("cors: {{origins: any}}\n{closed}"));
    let item = &document["paths"]["/items/{item_id}"];
    assert_eq!(
        header_schema(
            &item["options"]["responses"]["204"],
            "Access-Control-Allow-Methods"
        )["const"],
        "PUT"
    );
    assert!(
        item["get"]["responses"]["200"]["headers"]
            .get("Access-Control-Allow-Origin")
            .is_none(),
        "{item}"
    );
    // A route's own policy only: the document states no binding policy, the path of the route
    // alone has a preflight.
    let own = CORS_ROUTES.replace(
        "    parameters: {item_id: input.item_id}\n",
        "    parameters: {item_id: input.item_id}\n    cors: {origins: any}\n",
    );
    let (_, document) = cors_document(&ir, &own);
    assert!(document.get("x-ess-cors").is_none(), "{document}");
    assert!(
        document["paths"]["/items/commands/TouchItem"]
            .get("options")
            .is_none(),
        "{document}"
    );
    assert_eq!(
        header_schema(
            &document["paths"]["/items/{item_id}"]["options"]["responses"]["204"],
            "Access-Control-Allow-Methods"
        )["const"],
        "PUT"
    );
    // Without `cors`, no CORS byte.
    let (text, _) = cors_document(&ir, CORS_ROUTES);
    for absent in ["Access-Control", "x-ess-cors", "options:", "Vary"] {
        assert!(!text.contains(absent), "{absent}");
    }
}
