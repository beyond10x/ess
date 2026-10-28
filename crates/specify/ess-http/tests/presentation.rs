//! An `ess-http/1` binding names the `operationId` of a route and the servers of a component
//! (`docs/design/http-presentation-binding.md`, "Operation ids and servers"): `routes[i].operation_id`
//! and `servers: {<component>: [{url, description}]}`, both optional, both omitted from the IR when
//! absent, so a binding without them keeps its bytes.

use ess_compiler::resolve::compile as compile_model;
use ess_compiler::source::SourceMap;
use ess_compiler::EssIr;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_http::{
    compile, compile_refining, DerivedRoute, HttpDiagnostics, HttpIr, HttpSpec, Method, Operation,
    Refined,
};

const MODEL: &str = include_str!("fixtures/catalog.yaml");

/// `catalog.yaml`'s second file: `items-worker`, which declares no `reached_by`.
const WORKER: &str = include_str!("fixtures/worker.yaml");

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

fn model() -> EssIr {
    model_of(&[("catalog.yaml", MODEL)])
}

/// The fixture with a second component, `items-worker`, which declares no `reached_by`: its callers
/// are deployed with it.
fn two_components() -> EssIr {
    model_of(&[("catalog.yaml", MODEL), ("worker.yaml", WORKER)])
}

/// The routes ESS derives for `ir`, as `tests/compile.rs` writes them out.
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

fn compiled(ir: &EssIr, keys: &str) -> Result<HttpIr, HttpDiagnostics> {
    let spec = HttpSpec::from_yaml(&(header(ir) + keys)).expect("parses");
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

/// Two routes, the command's with an operation id.
const NAMED: &str = "routes:
  - view: catalog.items.ItemById
    method: get
    path: /items/{item_id}
    parameters:
      item_id: param.item_id
  - command: catalog.items.ListItem
    operation_id: listItem
    method: put
    path: /items/{item_id}
    parameters:
      item_id: input.item_id
";

#[test]
fn a_route_operation_id_is_compiled_into_the_ir() {
    let ir = model();
    let http = compiled(&ir, NAMED).expect("compiles");
    let list = http
        .route(&Operation::Command("catalog.items.ListItem".to_owned()))
        .expect("routed");
    assert_eq!(list.operation_id.as_deref(), Some("listItem"));
    let view = http
        .route(&Operation::View("catalog.items.ItemById".to_owned()))
        .expect("routed");
    assert_eq!(view.operation_id, None);
    let json = http.to_canonical_json();
    assert!(
        json.contains(
            "      \"command\": \"catalog.items.ListItem\",\n      \"operation_id\": \"listItem\",\n      \"method\": \"put\",\n"
        ),
        "{json}"
    );
    assert_eq!(
        json.matches("operation_id").count(),
        1,
        "an unset id is absent: {json}"
    );
    assert_eq!(
        HttpIr::from_json(&json)
            .expect("reads back")
            .to_canonical_json(),
        json
    );

    let unnamed = compiled(&ir, &NAMED.replace("    operation_id: listItem\n", "")).unwrap();
    let json = unnamed.to_canonical_json();
    assert!(!json.contains("operation_id"), "{json}");
    assert!(!json.contains("servers"), "{json}");
}

#[test]
fn a_duplicate_operation_id_is_refused_naming_both() {
    let ir = model();
    let routes = "routes:
  - command: catalog.items.ListItem
    operation_id: change
    method: post
  - command: catalog.items.TouchItem
    operation_id: change
    method: post
    path: /items/touch
";
    assert_eq!(
        located(compiled(&ir, routes)),
        [refused("ESS-HTTP-032", "routes[1].operation_id")]
    );
    let words = worded(compiled(&ir, routes));
    assert!(
        words[0].contains("`routes[0]`")
            && words[0].contains("catalog.items.ListItem")
            && words[0].contains("catalog.items.TouchItem"),
        "{words:?}"
    );
}

#[test]
fn an_operation_id_equal_to_another_operations_name_is_refused() {
    let ir = model();
    // `TouchItem` keeps its derived route, whose operationId is its qualified name.
    let routes = "routes:
  - command: catalog.items.ListItem
    operation_id: catalog.items.TouchItem
    method: post
";
    assert_eq!(
        located(compiled(&ir, routes)),
        [refused("ESS-HTTP-032", "routes[0].operation_id")]
    );
    let words = worded(compiled(&ir, routes));
    assert!(
        words[0].contains("command `catalog.items.TouchItem`"),
        "{words:?}"
    );
    // A route may restate its own qualified name.
    let own = "routes:
  - command: catalog.items.ListItem
    operation_id: catalog.items.ListItem
    method: post
";
    assert_eq!(located(compiled(&ir, own)), []);
}

#[test]
fn an_operation_id_outside_the_grammar_is_refused() {
    let ir = model();
    for bad in [
        "1list",
        "list item",
        "list/item",
        "''",
        "list{item}",
        "élan",
    ] {
        let routes = format!(
            "routes:
  - command: catalog.items.ListItem
    operation_id: {bad}
    method: post
"
        );
        assert_eq!(
            located(compiled(&ir, &routes)),
            [refused("ESS-HTTP-032", "routes[0].operation_id")],
            "{bad}"
        );
    }
    for good in ["listItem", "_list", "orders.list-item_2", "L"] {
        let routes = format!(
            "routes:
  - command: catalog.items.ListItem
    operation_id: {good}
    method: post
"
        );
        assert_eq!(located(compiled(&ir, &routes)), [], "{good}");
    }
}

/// A protocol binding that fixes the view's operation id and leaves the command without one.
const PROTOCOL: &str = "routes:
  - view: catalog.items.ItemById
    operation_id: getItem
    method: get
    path: /items/{item_id}
    parameters:
      item_id: param.item_id
  - command: catalog.items.ListItem
    method: post
";

#[test]
fn a_refinement_keeps_a_fixed_operation_id() {
    let ir = model();
    let refined = Refined::from_yaml((header(&ir) + PROTOCOL).as_bytes())
        .expect("the refined binding parses");
    let refining = |routes: &str| {
        let text = header(&ir) + &format!("refines: {}\n", refined.digest()) + routes;
        let spec = HttpSpec::from_yaml(&text).expect("parses");
        compile_refining(&spec, Some(&refined), &ir, &derived(&ir))
    };
    let kept = refining(
        "routes:
  - view: catalog.items.ItemById
    operation_id: getItem
    method: get
    path: /items/{item_id}
    parameters:
      item_id: param.item_id
  - command: catalog.items.ListItem
    operation_id: listItem
    method: post
    path: /api/items
",
    )
    .expect("an id added where the refined route has none");
    assert_eq!(
        kept.route(&Operation::Command("catalog.items.ListItem".to_owned()))
            .and_then(|route| route.operation_id.as_deref()),
        Some("listItem")
    );
    for (changed, word) in [
        ("operation_id: fetchItem\n    ", "`fetchItem`"),
        ("", "none"),
    ] {
        let routes = format!(
            "routes:
  - view: catalog.items.ItemById
    {changed}method: get
    path: /items/{{item_id}}
    parameters:
      item_id: param.item_id
  - command: catalog.items.ListItem
    method: post
"
        );
        assert_eq!(
            located(refining(&routes)),
            [refused("ESS-HTTP-012", "routes[0].operation_id")],
            "{routes}"
        );
        let words = worded(refining(&routes));
        assert!(
            words[0].contains("`getItem`") && words[0].contains(word),
            "{words:?}"
        );
    }
}

#[test]
fn servers_compile_per_network_component() {
    let ir = two_components();
    let http = compiled(
        &ir,
        "servers:
  items-service:
    - {url: 'https://api.example.test/v2', description: Production}
    - {url: /v2}
routes:
  - command: catalog.items.ListItem
    method: post
",
    )
    .expect("compiles");
    let servers = &http.servers()["items-service"];
    let urls: Vec<(&str, Option<&str>)> = servers
        .iter()
        .map(|server| (server.url.as_str(), server.description.as_deref()))
        .collect();
    assert_eq!(
        urls,
        [
            ("https://api.example.test/v2", Some("Production")),
            ("/v2", None)
        ],
        "entry order is kept"
    );
    let json = http.to_canonical_json();
    assert!(
        json.contains(
            "  \"servers\": {\n    \"items-service\": [\n      {\n        \"url\": \"https://api.example.test/v2\",\n        \"description\": \"Production\"\n      },\n      {\n        \"url\": \"/v2\"\n      }\n    ]\n  },\n  \"routes\": ["
        ),
        "{json}"
    );
    assert_eq!(
        HttpIr::from_json(&json)
            .expect("reads back")
            .to_canonical_json(),
        json
    );
}

#[test]
fn servers_on_a_component_not_reached_over_the_network_are_refused() {
    let ir = two_components();
    let keys = "servers:
  items-worker:
    - {url: 'https://worker.example.test'}
  nowhere:
    - {url: 'https://nowhere.example.test'}
routes:
  - command: catalog.items.ListItem
    method: post
";
    assert_eq!(
        located(compiled(&ir, keys)),
        [
            refused("ESS-HTTP-033", "servers.items-worker"),
            refused("ESS-HTTP-033", "servers.nowhere"),
        ]
    );
    let words = worded(compiled(&ir, keys));
    assert!(words[0].contains("reached_by: network"), "{words:?}");
    assert!(words[1].contains("no component `nowhere`"), "{words:?}");
}

#[test]
fn a_server_url_outside_the_rule_is_refused() {
    let ir = model();
    for bad in [
        "ftp://api.example.test",
        "api.example.test/v2",
        "",
        "//api.example.test",
        "https://",
        "https://api.example.test/v 2",
        "https://api.example.test/{version}",
        "https://api.example.test/v2?tenant=a",
        "https://api.example.test/v2#top",
        "https://user:secret@api.example.test/v2",
        "https://[::1/v2",
        "https://[::1]x/v2",
        "https://[]:8443/v2",
    ] {
        let keys = format!(
            "servers:
  items-service:
    - {{url: '{bad}'}}
routes:
  - command: catalog.items.ListItem
    method: post
"
        );
        assert_eq!(
            located(compiled(&ir, &keys)),
            [refused("ESS-HTTP-033", "servers.items-service[0].url")],
            "{bad}"
        );
    }
    let twice = "servers:
  items-service:
    - {url: /v2}
    - {url: /v2, description: again}
routes:
  - command: catalog.items.ListItem
    method: post
";
    assert_eq!(
        located(compiled(&ir, twice)),
        [refused("ESS-HTTP-033", "servers.items-service[1].url")]
    );
    let empty = "servers:
  items-service: []
routes:
  - command: catalog.items.ListItem
    method: post
";
    assert_eq!(
        located(compiled(&ir, empty)),
        [refused("ESS-HTTP-033", "servers.items-service")]
    );
    let repeated = "servers:
  items-service:
    - {url: /v1}
  items-service:
    - {url: /v2}
routes:
  - command: catalog.items.ListItem
    method: post
";
    let spec =
        HttpSpec::from_yaml(&(header(&ir) + repeated)).expect("a key written twice is read twice");
    assert_eq!(
        located(compile(&spec, &ir, &derived(&ir))),
        [refused("ESS-HTTP-033", "servers.items-service")]
    );
}

#[test]
fn the_ir_reader_refuses_duplicate_ids_and_bad_servers() {
    let ir = two_components();
    let http = compiled(
        &ir,
        "servers:
  items-service:
    - {url: /v2}
routes:
  - view: catalog.items.ItemById
    operation_id: getItem
    method: get
    path: /items/{item_id}
    parameters:
      item_id: param.item_id
  - command: catalog.items.ListItem
    operation_id: listItem
    method: post
",
    )
    .expect("compiles");
    let json = http.to_canonical_json();
    assert!(HttpIr::from_json(&json).is_ok(), "{json}");
    let refusal = |text: &str| {
        assert_ne!(text, json, "the mutation applied");
        HttpIr::from_json(text).expect_err("refused").to_string()
    };
    let duplicate = refusal(&json.replace("\"getItem\"", "\"listItem\""));
    assert!(duplicate.contains("operation_id"), "{duplicate}");
    let grammar = refusal(&json.replace("\"getItem\"", "\"get item\""));
    assert!(grammar.contains("operation_id"), "{grammar}");
    let named = refusal(&json.replace("\"getItem\"", "\"catalog.items.TouchItem\""));
    assert!(named.contains("operation_id"), "{named}");
    let url = refusal(&json.replace("\"/v2\"", "\"/v2?x=1\""));
    assert!(url.contains("servers.items-service[0].url"), "{url}");
    let empty = refusal(&json.replace(
        "\"items-service\": [\n      {\n        \"url\": \"/v2\"\n      }\n    ]",
        "\"items-service\": []",
    ));
    assert!(empty.contains("servers.items-service"), "{empty}");
    let ordered = refusal(&json.replace(
        "\"items-service\": [\n      {\n        \"url\": \"/v2\"\n      }\n    ]",
        "\"items-service\": [\n      {\n        \"url\": \"/v2\"\n      }\n    ],\n    \"items-api\": [\n      {\n        \"url\": \"/v1\"\n      }\n    ]",
    ));
    assert!(ordered.contains("canonical order"), "{ordered}");
    let twice = refusal(&json.replace(
        "\"items-service\": [\n      {\n        \"url\": \"/v2\"\n      }\n    ]",
        "\"items-service\": [\n      {\n        \"url\": \"/v2\"\n      },\n      {\n        \"url\": \"/v2\"\n      }\n    ]",
    ));
    assert!(twice.contains("servers.items-service[1].url"), "{twice}");
}
