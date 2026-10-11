//! `ess-http/1` compiles against the specification it names, or says every reason it does not
//! (`docs/design/http-presentation-binding.md`, stage 1).

use std::fmt::Write as _;

use ess_compiler::resolve::compile as compile_model;
use ess_compiler::source::SourceMap;
use ess_compiler::EssIr;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_http::{compile, HttpIr, HttpSpec, Method, Operation};

const MODEL: &str = include_str!("fixtures/catalog.yaml");

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

fn header(ir: &EssIr) -> String {
    format!(
        "type: ess-http/1\nspecification:\n  system: catalog\n  version: v1\n  source_digest: sha256:{}\n",
        ir.source_digest()
    )
}

/// One route of each admitted shape: a fixed view path, an open command route, and a command on
/// `get` whose only input is mapped to the path.
const ROUTES: &str = "routes:
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

fn refusals(ir: &EssIr, text: &str) -> Vec<(&'static str, String)> {
    let spec = HttpSpec::from_yaml(text).expect("parses");
    match compile(&spec, ir) {
        Ok(_) => Vec::new(),
        Err(diagnostics) => diagnostics
            .0
            .iter()
            .map(|diagnostic| (diagnostic.code, diagnostic.path.clone()))
            .collect(),
    }
}

fn routes_refusals(ir: &EssIr, routes: &str) -> Vec<(&'static str, String)> {
    refusals(ir, &(header(ir) + routes))
}

fn refused(code: &'static str, path: &str) -> (&'static str, String) {
    (code, path.to_owned())
}

#[test]
fn a_binding_compiles_to_canonical_ir_ordered_by_path_then_method() {
    let ir = model();
    let spec = HttpSpec::from_yaml(&(header(&ir) + ROUTES)).expect("parses");
    let http = compile(&spec, &ir).expect("compiles");
    let order: Vec<(Option<&str>, Method)> = http
        .routes()
        .iter()
        .map(|route| (route.path.as_deref(), route.method))
        .collect();
    assert_eq!(
        order,
        [
            (None, Method::Post),
            (Some("/items/{item_id}"), Method::Get),
            (Some("/items/{item_id}/touch"), Method::Get),
        ]
    );
    let json = http.to_canonical_json();
    let expected = format!(
        "{{\n  \"format\": \"ess-http-ir/1\",\n  \"specification\": {{\n    \"system\": \"catalog\",\n    \"version\": \"v1\",\n    \"source_digest\": \"sha256:{}\"\n  }},\n  \"routes\": [\n    {{\n      \"command\": \"catalog.items.ListItem\",\n      \"method\": \"post\"\n    }},\n    {{\n      \"view\": \"catalog.items.ItemById\",\n      \"method\": \"get\",\n      \"path\": \"/items/{{item_id}}\",\n      \"parameters\": {{\n        \"item_id\": \"param.item_id\"\n      }}\n    }},\n    {{\n      \"command\": \"catalog.items.TouchItem\",\n      \"method\": \"get\",\n      \"path\": \"/items/{{item_id}}/touch\",\n      \"parameters\": {{\n        \"item_id\": \"input.item_id\"\n      }}\n    }}\n  ]\n}}\n",
        ir.source_digest()
    );
    assert_eq!(json, expected, "ess-http-ir/1 bytes changed");
}

#[test]
fn equal_input_gives_equal_bytes_whatever_the_authored_order() {
    let ir = model();
    let forward = HttpSpec::from_yaml(&(header(&ir) + ROUTES)).expect("parses");
    let reversed_routes = "routes:
  - command: catalog.items.TouchItem
    method: get
    path: /items/{item_id}/touch
    parameters: {item_id: input.item_id}
  - command: catalog.items.ListItem
    method: post
  - {view: catalog.items.ItemById, method: get, path: '/items/{item_id}', parameters: {item_id: param.item_id}}
";
    let reversed = HttpSpec::from_yaml(&(header(&ir) + reversed_routes)).expect("parses");
    let first = compile(&forward, &ir)
        .expect("compiles")
        .to_canonical_json();
    assert_eq!(
        first,
        compile(&forward, &ir).expect("again").to_canonical_json()
    );
    assert_eq!(
        first,
        compile(&reversed, &ir)
            .expect("reordered")
            .to_canonical_json()
    );
    let json = serde_json::to_string(&serde_json::json!({
        "type": "ess-http/1",
        "specification": {
            "system": "catalog",
            "version": "v1",
            "source_digest": format!("sha256:{}", ir.source_digest()),
        },
        "routes": [
            {"command": "catalog.items.ListItem", "method": "post"},
            {"view": "catalog.items.ItemById", "method": "get", "path": "/items/{item_id}",
             "parameters": {"item_id": "param.item_id"}},
            {"command": "catalog.items.TouchItem", "method": "get", "path": "/items/{item_id}/touch",
             "parameters": {"item_id": "input.item_id"}},
        ],
    }))
    .unwrap();
    let from_json = HttpSpec::from_json(&json).expect("parses as JSON");
    assert_eq!(
        first,
        compile(&from_json, &ir)
            .expect("from JSON")
            .to_canonical_json()
    );
}

#[test]
fn compiled_ir_reads_back_and_renders_a_route_table() {
    let ir = model();
    let spec = HttpSpec::from_yaml(&(header(&ir) + ROUTES)).expect("parses");
    let json = compile(&spec, &ir).expect("compiles").to_canonical_json();
    let read = HttpIr::from_json(&json).expect("the compiled IR reads back");
    assert_eq!(read.to_canonical_json(), json, "round trip changed bytes");
    let mut table = String::from("| operation | method | path |\n|---|---|---|\n");
    for route in read.routes() {
        let operation = match &route.operation {
            Operation::Command(name) => format!("command `{name}`"),
            Operation::View(name) => format!("view `{name}`"),
        };
        let path = route.path.as_deref().unwrap_or("(open)");
        writeln!(
            table,
            "| {operation} | {} | `{path}` |",
            route.method.as_str()
        )
        .unwrap();
    }
    assert_eq!(
        table,
        "| operation | method | path |\n|---|---|---|\n\
         | command `catalog.items.ListItem` | post | `(open)` |\n\
         | view `catalog.items.ItemById` | get | `/items/{item_id}` |\n\
         | command `catalog.items.TouchItem` | get | `/items/{item_id}/touch` |\n"
    );
    let other = json.replace("ess-http-ir/1", "ess-http-ir/2");
    assert!(
        HttpIr::from_json(&other).is_err(),
        "another IR format reads"
    );
    let unknown_route_key = json.replacen(
        "\"method\": \"post\"",
        "\"method\": \"post\",\n      \"status\": 200",
        1,
    );
    assert_ne!(unknown_route_key, json);
    let error = HttpIr::from_json(&unknown_route_key)
        .expect_err("an unknown route key reads")
        .to_string();
    assert!(error.contains("status"), "{error}");
    let unknown_key = json.replacen("\"routes\"", "\"derived\": [],\n  \"routes\"", 1);
    let error = HttpIr::from_json(&unknown_key)
        .expect_err("an unknown key reads")
        .to_string();
    assert!(error.contains("unknown field `derived`"), "{error}");
}

#[test]
fn an_empty_binding_is_valid_and_routes_nothing() {
    let ir = model();
    let http = compile(
        &HttpSpec::from_yaml(&(header(&ir) + "routes: []\n")).expect("parses"),
        &ir,
    )
    .expect("compiles");
    assert_eq!(http.routes(), &[]);
}

#[test]
fn ess_http_001_another_type_is_refused() {
    let ir = model();
    let text = header(&ir).replace("ess-http/1", "ess-http/2") + ROUTES;
    assert_eq!(refusals(&ir, &text), [refused("ESS-HTTP-001", "type")]);
}

#[test]
fn ess_http_002_each_disagreeing_specification_key_is_refused() {
    let ir = model();
    let text = header(&ir)
        .replace("system: catalog", "system: other")
        .replace("version: v1", "version: v2")
        .replace("sha256:", "sha256:00")
        + ROUTES;
    assert_eq!(
        refusals(&ir, &text),
        [
            refused("ESS-HTTP-002", "specification.system"),
            refused("ESS-HTTP-002", "specification.version"),
            refused("ESS-HTTP-002", "specification.source_digest"),
        ]
    );
}

#[test]
fn ess_http_004_a_route_names_one_operation_that_exists() {
    let ir = model();
    assert_eq!(
        routes_refusals(&ir, "routes:\n  - {method: post}\n"),
        [refused("ESS-HTTP-004", "routes[0]")]
    );
    assert_eq!(
        routes_refusals(
            &ir,
            "routes:\n  - {command: catalog.items.ListItem, view: catalog.items.ItemById, method: post}\n"
        ),
        [refused("ESS-HTTP-004", "routes[0]")]
    );
    assert_eq!(
        routes_refusals(
            &ir,
            "routes:\n  - {command: catalog.items.ListItem, method: post}\n  - {command: catalog.items.Missing, method: post}\n"
        ),
        [refused("ESS-HTTP-004", "routes[1].command")]
    );
    assert_eq!(
        routes_refusals(
            &ir,
            "routes:\n  - {view: catalog.items.Missing, method: get}\n"
        ),
        [refused("ESS-HTTP-004", "routes[0].view")]
    );
    // A command's name in `view:` names no view.
    assert_eq!(
        routes_refusals(
            &ir,
            "routes:\n  - {view: catalog.items.ListItem, method: get}\n"
        ),
        [refused("ESS-HTTP-004", "routes[0].view")]
    );
}

#[test]
fn ess_http_005_one_operation_routed_twice_is_refused() {
    let ir = model();
    let found = routes_refusals(
        &ir,
        "routes:
  - {command: catalog.items.ListItem, method: post}
  - {view: catalog.items.ItemById, method: get}
  - {command: catalog.items.ListItem, method: put, path: /items}
",
    );
    assert_eq!(found, [refused("ESS-HTTP-005", "routes[2]")]);
    let spec = HttpSpec::from_yaml(
        &(header(&ir)
            + "routes:\n  - {command: catalog.items.ListItem, method: post}\n  - {command: catalog.items.ListItem, method: put}\n"),
    )
    .unwrap();
    let message = &compile(&spec, &ir).expect_err("refused").0[0].message;
    assert!(
        message.contains("routes[0]") && message.contains("catalog.items.ListItem"),
        "{message}"
    );
}

#[test]
fn ess_http_006_a_method_outside_the_closed_set_is_refused() {
    let ir = model();
    for method in ["GET", "Post", "connect", "fetch", ""] {
        assert_eq!(
            routes_refusals(
                &ir,
                &format!("routes:\n  - {{command: catalog.items.ListItem, method: '{method}'}}\n")
            ),
            [refused("ESS-HTTP-006", "routes[0].method")],
            "{method:?}"
        );
    }
}

#[test]
fn ess_http_006_a_view_is_bound_to_get_only() {
    let ir = model();
    for method in ["put", "post", "delete", "options", "head", "patch", "trace"] {
        assert_eq!(
            routes_refusals(
                &ir,
                &format!("routes:\n  - {{view: catalog.items.ItemById, method: {method}}}\n")
            ),
            [refused("ESS-HTTP-006", "routes[0].method")],
            "{method}"
        );
    }
    assert_eq!(
        routes_refusals(
            &ir,
            "routes:\n  - {view: catalog.items.ItemById, method: get}\n"
        ),
        []
    );
}

#[test]
fn ess_http_006_a_command_is_refused_on_head_options_and_trace() {
    let ir = model();
    for method in ["head", "options", "trace"] {
        assert_eq!(
            routes_refusals(
                &ir,
                &format!("routes:\n  - {{command: catalog.items.ListItem, method: {method}}}\n")
            ),
            [refused("ESS-HTTP-006", "routes[0].method")],
            "{method}"
        );
    }
    for method in ["post", "put", "patch", "delete"] {
        assert_eq!(
            routes_refusals(
                &ir,
                &format!("routes:\n  - {{command: catalog.items.ListItem, method: {method}}}\n")
            ),
            [],
            "{method}"
        );
    }
}

#[test]
fn ess_http_013_an_operation_without_an_http_surface_is_refused() {
    let ir = model();
    assert_eq!(
        routes_refusals(
            &ir,
            "routes:\n  - {command: catalog.items.PurgeItems, method: post}\n"
        ),
        [refused("ESS-HTTP-013", "routes[0].command")]
    );
    let in_process = model_from(&MODEL.replace("    reached_by: network\n", ""));
    assert_eq!(
        routes_refusals(
            &in_process,
            "routes:\n  - {view: catalog.items.ItemById, method: get}\n  - {command: catalog.items.ListItem, method: post}\n"
        ),
        [refused("ESS-HTTP-013", "routes[0].view")]
    );
}

#[test]
fn ess_http_014_a_command_on_get_maps_every_input_field_to_the_path() {
    let ir = model();
    let found = routes_refusals(
        &ir,
        "routes:
  - command: catalog.items.ListItem
    method: get
    path: /items/{item_id}
    parameters: {item_id: input.item_id}
  - command: catalog.items.TouchItem
    method: get
    path: /touch
",
    );
    assert_eq!(
        found,
        [
            refused("ESS-HTTP-014", "routes[0]"),
            refused("ESS-HTTP-014", "routes[1]"),
        ]
    );
    let spec = HttpSpec::from_yaml(
        &(header(&ir)
            + "routes:\n  - {command: catalog.items.ListItem, method: get, path: '/items/{item_id}', parameters: {item_id: input.item_id}}\n"),
    )
    .unwrap();
    let message = &compile(&spec, &ir).expect_err("refused").0[0].message;
    assert!(
        message.contains("catalog.items.ListItem") && message.contains("`title`"),
        "{message}"
    );
}

#[test]
fn every_refusal_is_returned_at_once() {
    let ir = model();
    let text = header(&ir).replace("version: v1", "version: v2")
        + "routes:
  - {method: post}
  - {view: catalog.items.ItemById, method: post}
  - {command: catalog.items.PurgeItems, method: post}
  - {command: catalog.items.TouchItem, method: get, path: /touch}
  - {view: catalog.items.ItemById, method: get}
";
    assert_eq!(
        refusals(&ir, &text),
        [
            refused("ESS-HTTP-002", "specification.version"),
            refused("ESS-HTTP-004", "routes[0]"),
            refused("ESS-HTTP-006", "routes[1].method"),
            refused("ESS-HTTP-013", "routes[2].command"),
            refused("ESS-HTTP-014", "routes[3]"),
            refused("ESS-HTTP-005", "routes[4]"),
        ]
    );
}

#[test]
fn a_stage_2_key_is_refused_as_an_unknown_field_naming_it() {
    let ir = model();
    for key in [
        "request: {encoding: form}",
        "response: {}",
        "status: 200",
        "headers: {Cache-Control: {literal: no-store}}",
        "redirect: {location: input.title}",
        "outcomes: {}",
        "body: single",
        "empty: {status: 404}",
    ] {
        let name = key.split(':').next().unwrap();
        let text = header(&ir)
            + &format!(
                "routes:\n  - command: catalog.items.ListItem\n    method: post\n    {key}\n"
            );
        let error = HttpSpec::from_yaml(&text).expect_err(name).to_string();
        assert!(
            error.contains(&format!("unknown field `{name}`")),
            "{name}: {error}"
        );
    }
    let text = header(&ir) + "refusals: {}\n" + ROUTES;
    let error = HttpSpec::from_yaml(&text)
        .expect_err("refusals")
        .to_string();
    assert!(error.contains("unknown field `refusals`"), "{error}");
    let text = header(&ir).replace("version: v1", "version: v1\n  extra: 1") + ROUTES;
    let error = HttpSpec::from_yaml(&text)
        .expect_err("specification")
        .to_string();
    assert!(error.contains("unknown field `extra`"), "{error}");
}

#[test]
fn every_code_this_crate_can_emit_is_one_of_the_stage_1_codes() {
    for code in ess_http::CODES {
        assert!(
            matches!(
                *code,
                "ESS-HTTP-001"
                    | "ESS-HTTP-002"
                    | "ESS-HTTP-004"
                    | "ESS-HTTP-005"
                    | "ESS-HTTP-006"
                    | "ESS-HTTP-013"
                    | "ESS-HTTP-014"
            ),
            "{code}"
        );
    }
    let source = include_str!("../src/lib.rs");
    let mut emitted: Vec<&str> = source
        .match_indices("\"ESS-HTTP-")
        .map(|(at, _)| &source[at + 1..at + 13])
        .collect();
    emitted.sort_unstable();
    emitted.dedup();
    assert_eq!(
        emitted,
        ess_http::CODES,
        "a code in the source is not in CODES"
    );
}
