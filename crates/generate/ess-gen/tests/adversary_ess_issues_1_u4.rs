//! Adversarial cases for the `OpenAPI` projection of an `ess-http/1` binding's operation ids and
//! servers, and for the first paragraph of a document whose component is not reached over the
//! network (<https://github.com/beyond10x/ess/issues/653>, <https://github.com/beyond10x/ess/issues/651>).

use ess_compiler::resolve::compile as compile_model;
use ess_compiler::source::SourceMap;
use ess_compiler::EssIr;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_gen::artifact::run;
use ess_gen::http::{derived, Binding};
use ess_gen::openapi::{bound_json, BoundOpenApi, OpenApi};
use serde_json::Value;

const MODEL: &str = include_str!("../../../specify/ess-http/tests/fixtures/catalog.yaml");
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
        "type: ess-http/1\nspecification:\n  system: catalog\n  version: v1\n  source_digest: sha256:{}\n{keys}",
        ir.source_digest()
    );
    let spec = ess_http::HttpSpec::from_yaml(&text).expect("parses");
    let http = ess_http::compile(&spec, ir, &derived(ir))
        .unwrap_or_else(|diagnostics| panic!("compiles: {diagnostics}"));
    Binding::new(http).expect("every route has a path")
}

fn as_json(text: &str) -> Value {
    let yaml: ess_yaml::Value = ess_yaml::from_str(text).expect("YAML");
    serde_json::to_value(yaml).expect("JSON")
}

fn component<'a>(ir: &'a EssIr, name: &str) -> &'a ess_compiler::ir::ResolvedComponent {
    ir.components()
        .values()
        .find(|component| component.name.to_string() == name)
        .expect("component")
}

fn first_paragraph(document: &Value) -> String {
    document["info"]["description"]
        .as_str()
        .expect("description")
        .split("\n\n")
        .next()
        .expect("a paragraph")
        .to_owned()
}

/// Every `operationId` of a document, sorted.
fn operation_ids(document: &Value) -> Vec<String> {
    let mut ids: Vec<String> = document["paths"]
        .as_object()
        .expect("paths")
        .values()
        .flat_map(|item| item.as_object().expect("path item").values())
        .filter_map(|operation| operation.get("operationId"))
        .map(|id| id.as_str().expect("an id").to_owned())
        .collect();
    ids.sort();
    ids
}

/// The served JSON copy is the committed YAML document: same ids, same servers.
#[test]
fn the_served_document_carries_the_same_operation_ids_and_servers() {
    let ir = model_of(&[("catalog.yaml", MODEL)]);
    let keys = "servers:
  items-service:
    - {url: 'https://api.example.test/v2'}
routes:
  - command: catalog.items.ListItem
    operation_id: listItem
    method: put
    path: /items/{item_id}
    parameters: {item_id: input.item_id}
  - view: catalog.items.ItemById
    operation_id: getItem
    method: get
    path: /items/{item_id}
    parameters: {item_id: param.item_id}
";
    let documents = run(&BoundOpenApi(binding(&ir, keys)), &ir).expect("generates");
    let committed = as_json(&documents["openapi/items-service.yaml"].contents);
    let served: Value = serde_json::from_str(&bound_json(
        &ir,
        component(&ir, "items-service"),
        Some(&binding(&ir, keys)),
    ))
    .expect("JSON");
    assert_eq!(served, committed);
    assert_eq!(
        operation_ids(&committed),
        ["catalog.items.TouchItem", "getItem", "listItem"]
    );
    assert_eq!(
        committed["servers"][0]["url"],
        "https://api.example.test/v2"
    );
}

/// A binding may route an in-process component's command and name its id; the id is projected on
/// that component's document, which still says no request reaches its paths, and has no servers.
#[test]
fn an_in_process_route_takes_its_operation_id_and_keeps_its_sentence() {
    let ir = model_of(&[("catalog.yaml", MODEL), ("worker.yaml", WORKER)]);
    let keys = "routes:
  - command: catalog.work.Sweep
    operation_id: sweep
    method: post
    path: /sweep
";
    let documents = run(&BoundOpenApi(binding(&ir, keys)), &ir).expect("generates");
    let worker = as_json(&documents["openapi/items-worker.yaml"].contents);
    assert_eq!(worker["paths"]["/sweep"]["post"]["operationId"], "sweep");
    assert!(worker.get("servers").is_none());
    assert!(
        first_paragraph(&worker).ends_with(
            "It is not declared `reached_by: network`: its callers are deployed with it, so these \
             paths describe its commands and no request reaches them."
        ),
        "{}",
        first_paragraph(&worker)
    );
    let service = as_json(&documents["openapi/items-service.yaml"].contents);
    assert_eq!(
        first_paragraph(&service),
        "The HTTP surface of `items-service`, one of the components of `catalog` v1."
    );
}

/// Without a binding, the served and committed documents of a non-network component carry the
/// same sentence, and a network one's carries none.
#[test]
fn the_sentence_is_in_the_served_copy_too_and_only_off_the_network() {
    let ir = model_of(&[("catalog.yaml", MODEL), ("worker.yaml", WORKER)]);
    let documents = run(&OpenApi, &ir).expect("generates");
    for (name, sentence) in [("items-worker", true), ("items-service", false)] {
        let committed = as_json(&documents[&format!("openapi/{name}.yaml")].contents);
        let served: Value =
            serde_json::from_str(&ess_gen::openapi::json(&ir, component(&ir, name))).unwrap();
        assert_eq!(served, committed, "{name}");
        assert_eq!(
            first_paragraph(&committed).contains("no request reaches them"),
            sentence,
            "{name}"
        );
        assert_eq!(
            committed["info"]["description"]
                .as_str()
                .unwrap()
                .matches("reached_by: network")
                .count(),
            1,
            "the sentence and the network paragraph are exclusive: {name}"
        );
    }
}

/// A binding with neither key projects every document byte for byte as before: the documents of a
/// binding that names an id only differ from the unnamed one on that id's line.
#[test]
fn an_operation_id_moves_only_its_own_line() {
    let ir = model_of(&[("catalog.yaml", MODEL)]);
    let unnamed = "routes:
  - command: catalog.items.ListItem
    method: post
    path: /items
";
    let named = "routes:
  - command: catalog.items.ListItem
    operation_id: listItem
    method: post
    path: /items
";
    let before = run(&BoundOpenApi(binding(&ir, unnamed)), &ir).expect("generates");
    let after = run(&BoundOpenApi(binding(&ir, named)), &ir).expect("generates");
    let before = &before["openapi/items-service.yaml"].contents;
    let after = &after["openapi/items-service.yaml"].contents;
    let differ: Vec<(&str, &str)> = before
        .lines()
        .zip(after.lines())
        .filter(|(left, right)| left != right)
        .collect();
    assert_eq!(before.lines().count(), after.lines().count());
    assert_eq!(
        differ,
        [(
            "      operationId: catalog.items.ListItem",
            "      operationId: listItem"
        )],
        "{after}"
    );
}
