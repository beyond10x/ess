//! Adversary, unit U9c, pass 1: `request: {encoding: query}` on a `delete` command
//! (`story:a-delete-command-may-take-its-input-from-the-query-string`) and a view parameter's
//! wire name (`story:a-view-parameter-carries-its-wire-name-on-every-http-reader`).

use ess_compiler::resolve::compile as compile_model;
use ess_compiler::source::SourceMap;
use ess_compiler::EssIr;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_http::{
    compile, compile_refining, DerivedRoute, Encoding, HttpDiagnostics, HttpIr, HttpSpec, Method,
    Operation, Refined,
};

const MODEL: &str = include_str!("fixtures/catalog.yaml");

fn model_from(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).expect("well formed");
    let spec = Specification::assemble([(Source::new("catalog.yaml"), raw)]).expect("validates");
    let mut sources = SourceMap::new();
    sources.insert("catalog.yaml", text);
    compile_model(&spec, &sources).expect("compiles")
}

/// The fixture with `FindItems` (every flat kind: newtype, `List`, `Optional<Integer>`, enum,
/// `Boolean`, a wire-named `Optional<String>`) and `ShapeItems` (every kind without a flat
/// spelling) accepted.
fn shaped_model() -> EssIr {
    let text = MODEL
        .replace(
            "  - {name: catalog.items.ItemId, kind: newtype, of: String}\n",
            "  - {name: catalog.items.ItemId, kind: newtype, of: String}
  - {name: catalog.items.Shade, kind: enum, variants: [red, blue]}
  - name: catalog.items.Pair
    kind: struct
    fields:
      - {name: left, type: String}
",
        )
        .replace(
            "  - name: catalog.items.PurgeItems\n",
            "  - name: catalog.items.FindItems
    input:
      - {name: item_id, type: catalog.items.ItemId}
      - {name: labels, type: List<String>}
      - {name: limit, type: Optional<Integer>}
      - {name: shade, type: catalog.items.Shade}
      - {name: flag, type: Boolean}
      - {name: note, type: Optional<String>, wire: noteText}
    outcomes:
      - name: found
        emits: [catalog.items.ItemTouched]
        payload:
          catalog.items.ItemTouched: {item_id: input.item_id}
  - name: catalog.items.ShapeItems
    input:
      - {name: item_id, type: catalog.items.ItemId}
      - {name: labels, type: List<String>}
      - {name: limit, type: Optional<Integer>}
      - {name: pair, type: catalog.items.Pair}
      - {name: grid, type: List<List<String>>}
      - name: counts
        type: Map<String, Integer>
      - {name: data, type: Json}
      - {name: maybe_labels, type: Optional<List<String>>}
    outcomes:
      - name: shaped
        emits: [catalog.items.ItemTouched]
        payload:
          catalog.items.ItemTouched: {item_id: input.item_id}
  - name: catalog.items.PurgeItems
",
        )
        .replace(
            "accepts: {commands: [catalog.items.ListItem, catalog.items.TouchItem]}",
            "accepts: {commands: [catalog.items.FindItems, catalog.items.ListItem, catalog.items.ShapeItems, catalog.items.TouchItem]}",
        );
    model_from(&text)
}

fn derived(ir: &EssIr) -> Vec<DerivedRoute> {
    let mut table = Vec::new();
    for component in ir.components().values() {
        for handle in &component.accepts {
            table.push(DerivedRoute {
                component: component.name.to_string(),
                operation: Operation::Command(handle.name().to_string()),
                method: Method::Post,
                path: format!("/items/commands/{}", handle.name().local()),
            });
        }
        if component.reached_by == ess_domain::component::Reach::Network {
            for domain in &component.owns {
                for handle in &ir.domain(domain).views {
                    table.push(DerivedRoute {
                        component: component.name.to_string(),
                        operation: Operation::View(handle.name().to_string()),
                        method: Method::Get,
                        path: format!("/items/views/{}", handle.name().local()),
                    });
                }
            }
        }
    }
    table.sort_by(|left, right| {
        (&left.component, &left.path, left.method, &left.operation).cmp(&(
            &right.component,
            &right.path,
            right.method,
            &right.operation,
        ))
    });
    table
}

fn header(ir: &EssIr) -> String {
    format!(
        "type: ess-http/1\nspecification:\n  system: catalog\n  version: v1\n  source_digest: sha256:{}\n",
        ir.source_digest()
    )
}

fn compiled(ir: &EssIr, routes: &str) -> Result<HttpIr, HttpDiagnostics> {
    let spec = HttpSpec::from_yaml(&(header(ir) + routes)).expect("parses");
    compile(&spec, ir, &derived(ir))
}

fn located(result: Result<HttpIr, HttpDiagnostics>) -> Vec<(&'static str, String)> {
    match result {
        Ok(_) => Vec::new(),
        Err(diagnostics) => diagnostics
            .0
            .into_iter()
            .map(|diagnostic| (diagnostic.code, diagnostic.path))
            .collect(),
    }
}

/// Every flat kind rides a `delete` query string, with one field in the path, and the IR reads
/// back byte for byte.
#[test]
fn every_flat_kind_compiles_on_a_delete_query_and_round_trips() {
    let ir = shaped_model();
    let routes = "routes:
  - command: catalog.items.FindItems
    method: delete
    path: /items/{item_id}/find
    parameters: {item_id: input.item_id}
    request: {encoding: query}
";
    let http = compiled(&ir, routes).unwrap_or_else(|refused| panic!("{refused}"));
    let route = http
        .route(&Operation::Command("catalog.items.FindItems".into()))
        .expect("routed");
    assert_eq!(route.method, Method::Delete);
    assert_eq!(
        route.request.map(|request| request.encoding),
        Some(Encoding::Query)
    );
    let json = http.to_canonical_json();
    let read = HttpIr::from_json(&json).unwrap_or_else(|error| panic!("{error}: {json}"));
    assert_eq!(read.to_canonical_json(), json);
}

/// `ESS-HTTP-021` on a `delete` query names exactly the fields it names on a `get` query.
#[test]
fn ess_http_021_on_delete_names_the_same_fields_as_on_get() {
    let ir = shaped_model();
    let on = |method: &str| {
        located(compiled(
            &ir,
            &format!(
                "routes:
  - command: catalog.items.ShapeItems
    method: {method}
    path: /shapes/{{item_id}}
    parameters: {{item_id: input.item_id}}
    request: {{encoding: query}}
"
            ),
        ))
    };
    let get = on("get");
    let delete = on("delete");
    assert_eq!(
        get.len(),
        5,
        "pair, grid, counts, data, maybe_labels: {get:?}"
    );
    assert_eq!(delete, get);
}

/// A compiled `delete` query route whose method is hand-edited to `put`, `patch` or `post` is
/// refused by the reader, naming the encoding; `delete` itself reads.
#[test]
fn the_reader_admits_query_on_delete_and_still_refuses_it_on_every_other_body_method() {
    let ir = shaped_model();
    let routes = "routes:
  - command: catalog.items.FindItems
    method: delete
    path: /find
    request: {encoding: query}
";
    let json = compiled(&ir, routes).expect("compiles").to_canonical_json();
    assert!(json.contains("\"method\": \"delete\""), "{json}");
    for method in ["put", "patch", "post"] {
        let broken = json.replace(
            "\"method\": \"delete\"",
            &format!("\"method\": \"{method}\""),
        );
        let error = HttpIr::from_json(&broken).expect_err(method).to_string();
        assert!(error.contains("request.encoding"), "{method}: {error}");
    }
}

/// A refinement fills an open `delete` query route with a path, restating its request.
#[test]
fn a_refinement_may_fill_an_open_delete_route_with_a_query_request() {
    let ir = shaped_model();
    let open = "routes:
  - command: catalog.items.FindItems
    method: delete
    request: {encoding: query, unknown: ignore}
";
    let refined_text = header(&ir) + open;
    let refined = Refined::from_yaml(refined_text.as_bytes()).expect("parses");
    let digest = refined.digest();
    let refining = format!(
        "{}refines: {digest}
routes:
  - command: catalog.items.FindItems
    method: delete
    path: /find/{{item_id}}
    parameters: {{item_id: input.item_id}}
    request: {{encoding: query, unknown: ignore}}
",
        header(&ir)
    );
    let spec = HttpSpec::from_yaml(&refining).expect("parses");
    let http = compile_refining(&spec, Some(&refined), &ir, &derived(&ir))
        .unwrap_or_else(|refused| panic!("{refused}"));
    let route = http
        .route(&Operation::Command("catalog.items.FindItems".into()))
        .expect("routed");
    assert_eq!(
        route.request.map(|request| request.encoding),
        Some(Encoding::Query)
    );
}

/// Whether `text` parses, assembles and compiles, and the first refusal when it does not.
fn compiles(text: &str) -> Result<(), String> {
    let raw = RawSpecFile::parse(text).map_err(|error| format!("parse: {error}"))?;
    let spec = Specification::assemble([(Source::new("catalog.yaml"), raw)])
        .map_err(|error| format!("assemble: {error}"))?;
    let mut sources = SourceMap::new();
    sources.insert("catalog.yaml", text);
    compile_model(&spec, &sources)
        .map(|_| ())
        .map_err(|error| format!("compile: {error}"))
}

/// The views guide (`website/docs/guides/specify/views.md`, "A view parameter is named on the
/// wire by its `wire`") says a parameter "also takes the nested `naming: {wire: ownerId}`
/// spelling, but a view field takes only the flat `wire:` and refuses `naming:`".
#[test]
fn a_view_parameter_takes_nested_naming_and_a_view_field_refuses_it_as_the_guide_says() {
    let param_anchor = "    params:\n      - {name: item_id, type: catalog.items.ItemId}\n";
    assert_eq!(MODEL.matches(param_anchor).count(), 1, "the fixture moved");
    let param = MODEL.replace(
        param_anchor,
        "    params:\n      - {name: item_id, type: catalog.items.ItemId, naming: {wire: itemKey}}\n",
    );
    assert_eq!(
        compiles(&param),
        Ok(()),
        "a view parameter takes `naming: {{wire}}`"
    );
    let field_anchor = "      - {name: title, type: String}\ncomponents:";
    assert_eq!(MODEL.matches(field_anchor).count(), 1, "the fixture moved");
    let field = MODEL.replace(
        field_anchor,
        "      - {name: title, type: String, naming: {wire: itemTitle}}\ncomponents:",
    );
    let refused = compiles(&field).expect_err("a view field refuses `naming:`");
    assert!(refused.contains("naming"), "{refused}");
    let flat = MODEL.replace(
        field_anchor,
        "      - {name: title, type: String, wire: itemTitle}\ncomponents:",
    );
    assert_eq!(
        compiles(&flat),
        Ok(()),
        "a view field takes the flat `wire:`"
    );
}
