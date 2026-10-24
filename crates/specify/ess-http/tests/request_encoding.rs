//! An `ess-http/1` route declares how its request is encoded: `request: {encoding, unknown}`
//! (`docs/design/http-presentation-binding.md`, "Request encoding" and "Stage 2: keys, precedence
//! and the units that build them").

use ess_compiler::resolve::compile as compile_model;
use ess_compiler::source::SourceMap;
use ess_compiler::EssIr;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_http::{
    compile, compile_refining, DerivedRoute, Encoding, HttpDiagnostics, HttpIr, HttpSpec, Method,
    Operation, Refined, Unknown,
};

const MODEL: &str = include_str!("fixtures/catalog.yaml");

/// `ess-http-ir/1` as 0.60.0 compiled `STAGE_1_ROUTES` against `MODEL`, its source digest written
/// `SOURCE_DIGEST`: the digest is of the compiled specification, which another ESS release may
/// spell differently, while every other byte is the binding's own.
const STAGE_1_IR: &str = include_str!("fixtures/catalog-stage-1.ir.json");

/// The stage-1 routes 0.60.0 compiled into `STAGE_1_IR`: a fixed view path, an open command route,
/// and a command on `get` whose only input field is mapped to the path.
const STAGE_1_ROUTES: &str = "routes:
  - view: catalog.items.ItemById
    method: get
    path: /items/{item_id}
    parameters:
      item_id: param.item_id
  - command: catalog.items.ListItem
    method: post
  - command: catalog.items.TouchItem
    method: get
    path: /items/{item_id}/touch
    parameters:
      item_id: input.item_id
";

fn model_from(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).expect("well formed");
    let spec = Specification::assemble([(Source::new("catalog.yaml"), raw)]).expect("validates");
    let mut sources = SourceMap::new();
    sources.insert("catalog.yaml", text);
    compile_model(&spec, &sources).expect("compiles")
}

fn model() -> EssIr {
    model_from(MODEL)
}

/// The fixture with two more accepted commands. `catalog.items.FindItems` has an input every
/// field of which has a flat spelling; `catalog.items.ShapeItems` adds a field of each type that
/// has none.
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

/// The routes ESS derives for `ir`, written out the way `ess_gen::http::routes` derives them for
/// these fixtures, as `tests/compile.rs` does.
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

fn worded(result: Result<HttpIr, HttpDiagnostics>) -> Vec<String> {
    result
        .expect_err("refused")
        .0
        .into_iter()
        .map(|diagnostic| diagnostic.to_string())
        .collect()
}

fn refused(code: &'static str, path: &str) -> (&'static str, String) {
    (code, path.to_owned())
}

#[test]
fn a_route_without_request_keeps_json_and_the_stage_1_ir_bytes() {
    let ir = model();
    let http = compiled(&ir, STAGE_1_ROUTES).expect("compiles");
    let expected = STAGE_1_IR.replace("SOURCE_DIGEST", &ir.source_digest());
    assert_ne!(
        expected, STAGE_1_IR,
        "the pinned IR names its source digest"
    );
    assert_eq!(
        http.to_canonical_json(),
        expected,
        "ess-http-ir/1 bytes changed"
    );
    for route in http.routes() {
        assert_eq!(route.request, None, "{route:?}");
    }
    // A stage-2 reader reads the stage-1 bytes back unchanged.
    let read = HttpIr::from_json(&expected).expect("the 0.60.0 IR reads");
    assert_eq!(read.to_canonical_json(), expected);
}

#[test]
fn form_and_query_encodings_compile_into_the_route() {
    let ir = shaped_model();
    let routes = "routes:
  - command: catalog.items.ListItem
    method: post
    path: /api/items
    request: {encoding: form, unknown: ignore}
  - command: catalog.items.FindItems
    method: get
    path: /api/items/{item_id}
    parameters: {item_id: input.item_id}
    request: {encoding: query, unknown: refuse}
  - command: catalog.items.TouchItem
    method: put
    path: /api/touch
    request: {encoding: form}
";
    let http = compiled(&ir, routes).expect("compiles");
    let list = http
        .route(&Operation::Command("catalog.items.ListItem".into()))
        .and_then(|route| route.request.as_ref())
        .expect("ListItem records its request");
    assert_eq!(
        (list.encoding, list.unknown),
        (Encoding::Form, Unknown::Ignore)
    );
    let find = http
        .route(&Operation::Command("catalog.items.FindItems".into()))
        .and_then(|route| route.request.as_ref())
        .expect("FindItems records its request");
    assert_eq!(
        (find.encoding, find.unknown),
        (Encoding::Query, Unknown::Refuse)
    );
    let json = http.to_canonical_json();
    // `unknown: ignore` is recorded; `refuse`, the default, is omitted wherever it was written.
    assert!(
        json.contains(
            "      \"path\": \"/api/items\",\n      \"request\": {\n        \"encoding\": \"form\",\n        \"unknown\": \"ignore\"\n      }\n"
        ),
        "{json}"
    );
    assert!(
        json.contains(
            "      \"parameters\": {\n        \"item_id\": \"input.item_id\"\n      },\n      \"request\": {\n        \"encoding\": \"query\"\n      }\n"
        ),
        "{json}"
    );
    assert!(
        json.contains(
            "      \"path\": \"/api/touch\",\n      \"request\": {\n        \"encoding\": \"form\"\n      }\n"
        ),
        "{json}"
    );
    assert!(!json.contains("refuse"), "{json}");
    let read = HttpIr::from_json(&json).expect("the compiled IR reads back");
    assert_eq!(read.to_canonical_json(), json, "round trip changed bytes");
}

#[test]
fn a_request_written_with_its_defaults_compiles_to_the_bytes_without_it() {
    let ir = model();
    let bare = compiled(&ir, STAGE_1_ROUTES)
        .expect("compiles")
        .to_canonical_json();
    for request in [
        "request: {}",
        "request: {encoding: json}",
        "request: {unknown: refuse}",
        "request: {encoding: json, unknown: refuse}",
    ] {
        let routes = STAGE_1_ROUTES.replace(
            "  - command: catalog.items.ListItem\n    method: post\n",
            &format!("  - command: catalog.items.ListItem\n    method: post\n    {request}\n"),
        );
        assert_ne!(routes, STAGE_1_ROUTES);
        let written = compiled(&ir, &routes).expect(request).to_canonical_json();
        assert_eq!(written, bare, "{request}");
    }
}

#[test]
fn ess_http_014_a_get_command_whose_input_is_query_encoded_maps_none_of_it() {
    let ir = shaped_model();
    // Without `request`, a `get` command's input must all be in the path, as in stage 1.
    let unmapped = "routes:
  - command: catalog.items.FindItems
    method: get
    path: /api/items/{item_id}
    parameters: {item_id: input.item_id}
";
    assert_eq!(
        located(compiled(&ir, unmapped)),
        ["labels", "limit", "shade", "flag"].map(|_| refused("ESS-HTTP-014", "routes[0]"))
    );
    // Query-encoded, every unmapped field rides in the query string, so none is refused.
    let query = format!("{unmapped}    request: {{encoding: query}}\n");
    assert_eq!(located(compiled(&ir, &query)), []);
    let open = "routes:
  - command: catalog.items.FindItems
    method: get
    request: {encoding: query}
";
    assert_eq!(located(compiled(&ir, open)), []);
}

#[test]
fn ess_http_020_an_encoding_outside_the_set_or_on_the_wrong_method_is_refused() {
    let ir = shaped_model();
    let routes = "routes:
  - command: catalog.items.ListItem
    method: post
    request: {encoding: xml}
  - command: catalog.items.TouchItem
    method: get
    path: /items/{item_id}/touch
    parameters: {item_id: input.item_id}
    request: {encoding: form}
  - command: catalog.items.FindItems
    method: get
    request: {encoding: json}
  - command: catalog.items.ShapeItems
    method: post
    request: {encoding: query}
";
    let result = compiled(&ir, routes);
    assert_eq!(
        located(result.clone()),
        [
            refused("ESS-HTTP-020", "routes[0].request.encoding"),
            refused("ESS-HTTP-020", "routes[1].request.encoding"),
            refused("ESS-HTTP-020", "routes[2].request.encoding"),
            refused("ESS-HTTP-020", "routes[3].request.encoding"),
        ]
    );
    let shown = worded(result);
    assert!(
        shown[0].contains("`xml`")
            && shown[0].contains("`json`")
            && shown[0].contains("`form`")
            && shown[0].contains("`query`"),
        "{shown:?}"
    );
    assert!(
        shown[1].contains("`form`") && shown[1].contains("`get`"),
        "{shown:?}"
    );
    assert!(
        shown[2].contains("`json`") && shown[2].contains("`get`"),
        "{shown:?}"
    );
    assert!(
        shown[3].contains("`query`") && shown[3].contains("`post`"),
        "{shown:?}"
    );
    // `query` is refused on every method but `get` and `delete`, and the refusal names both.
    for method in ["put", "patch"] {
        let routes = format!(
            "routes:\n  - command: catalog.items.ListItem\n    method: {method}\n    request: {{encoding: query}}\n"
        );
        let result = compiled(&ir, &routes);
        assert_eq!(
            located(result.clone()),
            [refused("ESS-HTTP-020", "routes[0].request.encoding")],
            "{method}"
        );
        let shown = worded(result);
        assert!(
            shown[0].contains("`get`") && shown[0].contains("`delete`"),
            "{shown:?}"
        );
    }
}

/// The adopter's reproduction: a command bound to `delete` whose `Optional<String>` input rides in
/// the query string, as a protocol whose `DELETE` takes query parameters needs.
fn forgetting_model() -> EssIr {
    let text = MODEL
        .replace(
            "  - name: catalog.items.PurgeItems\n",
            "  - name: catalog.items.ForgetItem
    input:
      - {name: item_id, type: catalog.items.ItemId}
      - {name: reason, type: Optional<String>}
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
    model_from(&text)
}

#[test]
fn ess_http_020_a_delete_command_may_encode_its_input_as_query() {
    let ir = forgetting_model();
    for routes in [
        // Every input field in the query string.
        "routes:
  - command: catalog.items.ForgetItem
    method: delete
    path: /items/forget
    request: {encoding: query}
",
        // The identity in the path, the rest in the query string, an unknown key ignored.
        "routes:
  - command: catalog.items.ForgetItem
    method: delete
    path: /items/{item_id}
    parameters: {item_id: input.item_id}
    request: {encoding: query, unknown: ignore}
",
    ] {
        let http = compiled(&ir, routes).unwrap_or_else(|refused| panic!("{refused}"));
        let route = http
            .route(&Operation::Command("catalog.items.ForgetItem".into()))
            .expect("ForgetItem is routed");
        assert_eq!(route.method, Method::Delete);
        assert_eq!(
            route.request.as_ref().map(|request| request.encoding),
            Some(Encoding::Query)
        );
        // The compiled IR reads back, byte for byte: `ess-http-ir/1` admits `query` on `delete`.
        let json = http.to_canonical_json();
        let read = HttpIr::from_json(&json).unwrap_or_else(|error| panic!("{error}: {json}"));
        assert_eq!(read.to_canonical_json(), json, "round trip changed bytes");
    }
    // A delete still may not leave a non-flat field to the query string (`ESS-HTTP-021`).
    let shaped = shaped_model();
    let routes = "routes:
  - command: catalog.items.ShapeItems
    method: delete
    path: /shapes
    request: {encoding: query}
";
    let found = located(compiled(&shaped, routes));
    assert!(
        !found.is_empty() && found.iter().all(|(code, _)| *code == "ESS-HTTP-021"),
        "{found:?}"
    );
}

#[test]
fn ess_http_020_request_on_a_view_and_unknown_out_of_its_rules_are_refused() {
    let ir = shaped_model();
    let routes = "routes:
  - view: catalog.items.ItemById
    method: get
    path: /items/{item_id}
    parameters: {item_id: param.item_id}
    request: {encoding: query}
  - command: catalog.items.ListItem
    method: post
    request: {unknown: ignore}
  - command: catalog.items.TouchItem
    method: post
    request: {encoding: json, unknown: ignore}
  - command: catalog.items.ShapeItems
    method: post
    path: /api/shape
    request: {encoding: form, unknown: skip}
";
    let result = compiled(&ir, routes);
    assert_eq!(
        located(result.clone()),
        [
            refused("ESS-HTTP-020", "routes[0].request"),
            refused("ESS-HTTP-020", "routes[1].request.unknown"),
            refused("ESS-HTTP-020", "routes[2].request.unknown"),
            refused("ESS-HTTP-020", "routes[3].request.unknown"),
            // The encoding itself is admitted, so its fields are still checked.
            refused("ESS-HTTP-021", "routes[3].request.encoding"),
            refused("ESS-HTTP-021", "routes[3].request.encoding"),
            refused("ESS-HTTP-021", "routes[3].request.encoding"),
            refused("ESS-HTTP-021", "routes[3].request.encoding"),
            refused("ESS-HTTP-021", "routes[3].request.encoding"),
        ]
    );
    let shown = worded(result);
    assert!(
        shown[0].contains("view `catalog.items.ItemById`"),
        "{shown:?}"
    );
    assert!(
        shown[1].contains("`json`") && shown[1].contains("`ignore`"),
        "{shown:?}"
    );
    assert!(
        shown[3].contains("`skip`")
            && shown[3].contains("`refuse`")
            && shown[3].contains("`ignore`"),
        "{shown:?}"
    );
    // An empty `request` on a view is refused too: a view has no request to encode.
    let empty = "routes:
  - view: catalog.items.ItemById
    method: get
    request: {}
";
    assert_eq!(
        located(compiled(&ir, empty)),
        [refused("ESS-HTTP-020", "routes[0].request")]
    );
    // An unknown key under `request` is a parse refusal naming it.
    let text = header(&ir)
        + "routes:\n  - command: catalog.items.ListItem\n    method: post\n    request: {media: form}\n";
    let error = HttpSpec::from_yaml(&text).expect_err("media").to_string();
    assert!(error.contains("unknown field `media`"), "{error}");
}

#[test]
fn ess_http_021_a_form_or_query_field_without_a_flat_spelling_is_refused() {
    let ir = shaped_model();
    let refused_fields = [
        ("pair", "catalog.items.Pair"),
        ("grid", "List<List<String>>"),
        ("counts", "Map<String, Integer>"),
        ("data", "Json"),
        ("maybe_labels", "Optional<List<String>>"),
    ];
    for routes in [
        "routes:
  - command: catalog.items.ShapeItems
    method: post
    request: {encoding: form}
",
        "routes:
  - command: catalog.items.ShapeItems
    method: get
    path: /shape/{item_id}
    parameters: {item_id: input.item_id}
    request: {encoding: query}
",
    ] {
        let result = compiled(&ir, routes);
        assert_eq!(
            located(result.clone()),
            refused_fields.map(|_| refused("ESS-HTTP-021", "routes[0].request.encoding")),
            "{routes}"
        );
        let shown = worded(result);
        for ((field, type_ref), message) in refused_fields.iter().zip(&shown) {
            assert!(
                message.contains(&format!("`{field}`"))
                    && message.contains(&format!("`{type_ref}`")),
                "{field}: {message}"
            );
        }
        // `labels: List<String>` and `limit: Optional<Integer>` have a flat spelling.
        for message in &shown {
            assert!(
                !message.contains("`labels`") && !message.contains("`limit`"),
                "{message}"
            );
        }
    }
    // A field of every flat shape compiles, under both encodings.
    for routes in [
        "routes:\n  - command: catalog.items.FindItems\n    method: post\n    request: {encoding: form}\n",
        "routes:\n  - command: catalog.items.FindItems\n    method: get\n    request: {encoding: query}\n",
    ] {
        assert_eq!(located(compiled(&ir, routes)), [], "{routes}");
    }
    // A JSON body carries any field, as it does today.
    let json = "routes:\n  - command: catalog.items.ShapeItems\n    method: post\n";
    assert_eq!(located(compiled(&ir, json)), []);
}

/// A protocol binding fixing a form-encoded `ListItem` that ignores unknown parameters.
const PROTOCOL: &str = "routes:
  - command: catalog.items.ListItem
    method: post
    path: /api/items
    request: {encoding: form, unknown: ignore}
  - command: catalog.items.TouchItem
    method: post
";

fn refined_from(ir: &EssIr, routes: &str) -> Refined {
    Refined::from_yaml((header(ir) + routes).as_bytes()).expect("the refined binding parses")
}

fn refining(ir: &EssIr, refined: &Refined, routes: &str) -> Result<HttpIr, HttpDiagnostics> {
    let text = header(ir) + &format!("refines: {}\n", refined.digest()) + routes;
    let spec = HttpSpec::from_yaml(&text).expect("parses");
    compile_refining(&spec, Some(refined), ir, &derived(ir))
}

#[test]
fn ess_http_012_a_refinement_restates_the_request_of_every_refined_route() {
    let ir = model();
    let refined = refined_from(&ir, PROTOCOL);
    // Restated exactly, and a default written out where the refined binding left it, compiles.
    let restated = "routes:
  - command: catalog.items.ListItem
    method: post
    path: /api/items
    request: {encoding: form, unknown: ignore}
  - command: catalog.items.TouchItem
    method: post
    path: /api/touch
    request: {encoding: json, unknown: refuse}
";
    assert_eq!(located(refining(&ir, &refined, restated)), []);
    let changed = "routes:
  - command: catalog.items.ListItem
    method: post
    path: /api/items
    request: {encoding: json}
  - command: catalog.items.TouchItem
    method: post
    path: /api/touch
    request: {encoding: form}
";
    let result = refining(&ir, &refined, changed);
    assert_eq!(
        located(result.clone()),
        [
            refused("ESS-HTTP-012", "routes[0].request.encoding"),
            refused("ESS-HTTP-012", "routes[0].request.unknown"),
            refused("ESS-HTTP-012", "routes[1].request.encoding"),
        ]
    );
    let shown = worded(result);
    assert!(
        shown[0].contains("`form`")
            && shown[0].contains("`json`")
            && shown[0].contains("routes[0]"),
        "{shown:?}"
    );
    assert!(
        shown[1].contains("`ignore`") && shown[1].contains("`refuse`"),
        "{shown:?}"
    );
    assert!(
        shown[2].contains("`json`") && shown[2].contains("`form`"),
        "{shown:?}"
    );
    // Dropping the refined route's `request` is a change too.
    let dropped = "routes:
  - command: catalog.items.ListItem
    method: post
    path: /api/items
  - command: catalog.items.TouchItem
    method: post
";
    assert_eq!(
        located(refining(&ir, &refined, dropped)),
        [
            refused("ESS-HTTP-012", "routes[0].request.encoding"),
            refused("ESS-HTTP-012", "routes[0].request.unknown"),
        ]
    );
}

#[test]
fn compiled_ir_with_a_request_out_of_its_rules_is_refused_naming_the_key() {
    let ir = shaped_model();
    let routes = "routes:
  - command: catalog.items.ListItem
    method: post
    path: /api/items
    request: {encoding: form, unknown: ignore}
";
    let json = compiled(&ir, routes).expect("compiles").to_canonical_json();
    let form =
        "\"request\": {\n        \"encoding\": \"form\",\n        \"unknown\": \"ignore\"\n      }";
    assert!(json.contains(form), "{json}");
    for (written, named) in [
        ("\"request\": {}", "request"),
        ("\"request\": {\"encoding\": \"json\"}", "encoding"),
        (
            "\"request\": {\"encoding\": \"form\", \"unknown\": \"refuse\"}",
            "unknown",
        ),
        ("\"request\": {\"unknown\": \"ignore\"}", "unknown"),
        ("\"request\": {\"encoding\": \"query\"}", "encoding"),
        ("\"request\": {\"encoding\": \"xml\"}", "xml"),
        (
            "\"request\": {\"encoding\": \"form\", \"media\": \"x\"}",
            "media",
        ),
    ] {
        let broken = json.replace(form, written);
        assert_ne!(broken, json);
        let error = HttpIr::from_json(&broken).expect_err(written).to_string();
        assert!(error.contains(named), "{written}: {error}");
    }
    let get = "routes:
  - command: catalog.items.FindItems
    method: get
    request: {encoding: query}
";
    let json = compiled(&ir, get).expect("compiles").to_canonical_json();
    let query = "\"request\": {\n        \"encoding\": \"query\"\n      }";
    assert!(json.contains(query), "{json}");
    let error = HttpIr::from_json(&json.replace(query, "\"request\": {\"encoding\": \"form\"}"))
        .expect_err("form on get")
        .to_string();
    assert!(error.contains("routes[0].request.encoding"), "{error}");
    let view = "routes:
  - view: catalog.items.ItemById
    method: get
";
    let json = compiled(&ir, view).expect("compiles").to_canonical_json();
    let error = HttpIr::from_json(&json.replace(
        "\"method\": \"get\"",
        "\"method\": \"get\",\n      \"request\": {\"encoding\": \"query\"}",
    ))
    .expect_err("request on a view")
    .to_string();
    assert!(error.contains("routes[0].request"), "{error}");
}
