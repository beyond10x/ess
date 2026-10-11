//! The `OpenAPI` projection of an `ess-http/1` binding
//! (<https://github.com/beyond10x/ess/issues/493>, `docs/design/http-presentation-binding.md`,
//! "What generated artifacts change (stage 1)").
//!
//! A bound operation is served at its declared method and path, its path variables are required
//! `in: path` parameters, a command's mapped input field leaves the request body, and the operation
//! says `x-ess-route-source: binding`. An operation the binding does not name keeps its derived
//! route and its bytes, and a binding with an open route is refused, naming the operation.

use ess_compiler::resolve::compile as compile_model;
use ess_compiler::source::SourceMap;
use ess_compiler::EssIr;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_gen::artifact::run;
use ess_gen::http::{bound_routes, derived, routes, Binding};
use ess_gen::openapi::{BoundOpenApi, OpenApi};
use serde_json::{json, Value};

const MODEL: &str = include_str!("../../../specify/ess-http/tests/fixtures/catalog.yaml");

fn model() -> EssIr {
    let raw = RawSpecFile::parse(MODEL).expect("well formed");
    let spec = Specification::assemble([(Source::new("catalog.yaml"), raw)]).expect("validates");
    let mut sources = SourceMap::new();
    sources.insert("catalog.yaml", MODEL);
    compile_model(&spec, &sources).expect("compiles")
}

/// The binding `routes` declares, compiled against `ir` and the routes ESS derives for it.
fn compiled(ir: &EssIr, routes: &str) -> ess_http::HttpIr {
    let text = format!(
        "type: ess-http/1\nspecification:\n  system: catalog\n  version: v1\n  source_digest: sha256:{}\n{routes}",
        ir.source_digest()
    );
    let spec = ess_http::HttpSpec::from_yaml(&text).expect("parses");
    ess_http::compile(&spec, ir, &derived(ir)).expect("compiles")
}

fn binding(ir: &EssIr, routes: &str) -> Binding {
    Binding::new(compiled(ir, routes)).expect("every route has a path")
}

fn as_json(text: &str) -> Value {
    let yaml: serde_yaml::Value = serde_yaml::from_str(text).expect("YAML");
    serde_json::to_value(yaml).expect("JSON")
}

/// The items service's document, with `binding` beside the model.
fn bound(ir: &EssIr, binding: Binding) -> Value {
    let artifacts = run(&BoundOpenApi(binding), ir).expect("generates");
    as_json(&artifacts["openapi/items-service.yaml"].contents)
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
