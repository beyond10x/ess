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
//! `story:openapi-publishes-a-multipart-request-with-file-parts`).

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
    assert_eq!(schema["required"], json!(["caption", "tags", "count"]));
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

#[test]
fn a_binding_with_a_multipart_route_is_not_served_by_go_yet() {
    let ir = model_of(&[("imports.yaml", IMPORTS)]);
    let refused = binding(&ir, MULTIPART)
        .without_multipart()
        .expect_err("refused")
        .to_string();
    assert!(
        refused.contains(
            "command `demo.imports.ImportRows` takes a `multipart` request, which the generated Go \
             server does not read yet"
        ),
        "{refused}"
    );
}
