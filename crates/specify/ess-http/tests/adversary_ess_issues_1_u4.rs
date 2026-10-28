//! Adversarial cases for `routes[i].operation_id` (`ESS-HTTP-032`) and `servers`
//! (`ESS-HTTP-033`) in `ess-http/1` (`docs/design/http-presentation-binding.md`, "Operation ids
//! and servers"): the edges of the URL rule and the id grammar, the uniqueness rule against every
//! operation of the specification, the IR reader, and refinement.

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

fn two_components() -> EssIr {
    model_of(&[("catalog.yaml", MODEL), ("worker.yaml", WORKER)])
}

/// The routes ESS derives for `ir`, written out as `tests/presentation.rs` does.
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

fn refused(code: &'static str, path: &str) -> (&'static str, String) {
    (code, path.to_owned())
}

fn one_server(url: &str) -> String {
    format!(
        "servers:
  items-service:
    - {{url: '{url}'}}
routes:
  - command: catalog.items.ListItem
    method: post
"
    )
}

/// The decided rule is "absolute `http(s)://` with a host"; the implementation's own doc says
/// "names no host" for `https://`. An authority that is only a port names no host either.
#[test]
fn a_server_url_whose_authority_is_only_a_port_is_refused() {
    let ir = model();
    for bad in ["https://:8443/v2", "http://:80", "https://:"] {
        assert_eq!(
            located(compiled(&ir, &one_server(bad))),
            [refused("ESS-HTTP-033", "servers.items-service[0].url")],
            "{bad} names no host"
        );
    }
}

/// The IR reader holds the same URL rule (`HttpIr::from_json`), so it refuses the same URL.
#[test]
fn the_ir_reader_refuses_a_server_url_whose_authority_is_only_a_port() {
    let ir = model();
    let json = compiled(&ir, &one_server("https://api.example.test/v2"))
        .expect("compiles")
        .to_canonical_json();
    let mutated = json.replace("https://api.example.test/v2", "https://:8443/v2");
    assert_ne!(mutated, json, "the mutation applied");
    let error = HttpIr::from_json(&mutated)
        .err()
        .map(|error| error.to_string());
    assert!(
        error
            .as_deref()
            .is_some_and(|error| error.contains("servers.items-service[0].url")),
        "the reader admitted a URL without a host: {error:?}"
    );
}

/// Edges the rule admits: an IP literal with a port, a bare `/`, a trailing slash, no path.
#[test]
fn server_urls_at_the_edge_of_the_rule_are_admitted() {
    let ir = model();
    for good in [
        "https://[::1]:8443/v2",
        "/",
        "https://api.example.test/",
        "https://api.example.test",
        "http://localhost:8080",
        "/v2/%7Eitems",
    ] {
        assert_eq!(located(compiled(&ir, &one_server(good))), [], "{good}");
    }
}

/// Whitespace the rule names covers every Unicode whitespace and control character, and the
/// userinfo check is not fooled by a path that holds `@` after the host.
#[test]
fn server_urls_with_hidden_whitespace_or_userinfo_are_refused_and_an_at_in_the_path_is_not() {
    let ir = model();
    for bad in [
        "https://api.example.test/v\u{00a0}2",
        "https://api.example.test/v2\t",
        "https://user@api.example.test",
        "https://@api.example.test",
        "http://a:b@api.example.test:80/v2",
    ] {
        assert_eq!(
            located(compiled(&ir, &one_server(bad))),
            [refused("ESS-HTTP-033", "servers.items-service[0].url")],
            "{bad:?}"
        );
    }
    assert_eq!(
        located(compiled(&ir, &one_server("https://api.example.test/@v2"))),
        []
    );
    // YAML refuses a control character before compiling; an authored JSON binding carries one.
    let json = format!(
        r#"{{"type": "ess-http/1", "specification": {{"system": "catalog", "version": "v1", "source_digest": "sha256:{}"}}, "servers": {{"items-service": [{{"url": "https://api.example.test/v2\u0007"}}]}}, "routes": [{{"command": "catalog.items.ListItem", "method": "post"}}]}}"#,
        ir.source_digest()
    );
    let spec = HttpSpec::from_json(&json).expect("parses");
    assert_eq!(
        located(compile(&spec, &ir, &derived(&ir))),
        [refused("ESS-HTTP-033", "servers.items-service[0].url")]
    );
}

/// A refused URL is refused once, not also as a duplicate of an earlier one.
#[test]
fn a_bad_url_given_twice_is_refused_at_each_place_once() {
    let ir = model();
    let keys = "servers:
  items-service:
    - {url: 'https://api.example.test/v2?x'}
    - {url: 'https://api.example.test/v2?x'}
routes:
  - command: catalog.items.ListItem
    method: post
";
    assert_eq!(
        located(compiled(&ir, keys)),
        [
            refused("ESS-HTTP-033", "servers.items-service[0].url"),
            refused("ESS-HTTP-033", "servers.items-service[1].url"),
        ]
    );
}

/// `servers: {}` declares nothing: it compiles, and the IR omits the key.
#[test]
fn an_empty_servers_map_compiles_to_ir_without_the_key() {
    let ir = model();
    let http = compiled(
        &ir,
        "servers: {}
routes:
  - command: catalog.items.ListItem
    method: post
",
    )
    .expect("compiles");
    assert!(!http.to_canonical_json().contains("servers"));
}

/// Two components each name their own servers, the same URL included; the IR keeps component order
/// whatever the authored order.
#[test]
fn two_network_components_keep_their_own_servers_in_component_order() {
    let worker = WORKER.replace(
        "  - component: items-worker\n",
        "  - component: items-worker\n    reached_by: network\n",
    );
    let ir = model_of(&[("catalog.yaml", MODEL), ("worker.yaml", &worker)]);
    let http = compiled(
        &ir,
        "servers:
  items-worker:
    - {url: /shared}
  items-service:
    - {url: /shared}
routes:
  - command: catalog.items.ListItem
    method: post
",
    )
    .expect("compiles");
    let names: Vec<&str> = http.servers().keys().map(String::as_str).collect();
    assert_eq!(names, ["items-service", "items-worker"]);
    let json = http.to_canonical_json();
    assert_eq!(
        HttpIr::from_json(&json).expect("reads").to_canonical_json(),
        json
    );
}

/// The id may not be the qualified name of any other operation of the specification, a command no
/// component accepts and the view included, and the grammar's edges hold.
#[test]
fn operation_id_uniqueness_and_grammar_edges() {
    let ir = model();
    for taken in ["catalog.items.PurgeItems", "catalog.items.ItemById"] {
        let routes = format!(
            "routes:
  - command: catalog.items.ListItem
    operation_id: {taken}
    method: post
"
        );
        assert_eq!(
            located(compiled(&ir, &routes)),
            [refused("ESS-HTTP-032", "routes[0].operation_id")],
            "{taken}"
        );
    }
    for bad in [
        "-list",
        ".list",
        "list item",
        "'list\titem'",
        "'list\u{200b}'",
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
            "{bad:?}"
        );
    }
    for good in ["_", "a.", "a-", "A.b-C_9"] {
        let routes = format!(
            "routes:
  - command: catalog.items.ListItem
    operation_id: '{good}'
    method: post
"
        );
        assert_eq!(located(compiled(&ir, &routes)), [], "{good}");
    }
}

/// Three routes on one id: the second and third are refused, each naming the first.
#[test]
fn a_third_route_on_one_operation_id_names_the_first() {
    let ir = model();
    let routes = "routes:
  - command: catalog.items.ListItem
    operation_id: change
    method: post
  - command: catalog.items.TouchItem
    operation_id: change
    method: post
    path: /items/touch
  - view: catalog.items.ItemById
    operation_id: change
    method: get
    path: /items/{item_id}
    parameters:
      item_id: param.item_id
";
    let result = compiled(&ir, routes);
    let words: Vec<String> = result
        .as_ref()
        .err()
        .map(|diagnostics| diagnostics.0.iter().map(ToString::to_string).collect())
        .unwrap_or_default();
    assert_eq!(
        located(result),
        [
            refused("ESS-HTTP-032", "routes[1].operation_id"),
            refused("ESS-HTTP-032", "routes[2].operation_id"),
        ]
    );
    assert!(
        words.iter().all(|word| word.contains("`routes[0]`")),
        "{words:?}"
    );
}

/// What the IR reader writes back is what it was given: an `operation_id: null` and a server with
/// `description: null` or an unknown key are refused by name.
#[test]
fn the_ir_reader_refuses_null_and_unknown_presentation_keys() {
    let ir = model();
    let json = compiled(
        &ir,
        "servers:
  items-service:
    - {url: /v2, description: Production}
routes:
  - command: catalog.items.ListItem
    operation_id: listItem
    method: post
",
    )
    .expect("compiles")
    .to_canonical_json();
    assert!(HttpIr::from_json(&json).is_ok());
    for (from, to) in [
        ("\"listItem\"", "null"),
        ("\"Production\"", "null"),
        ("\"description\": \"Production\"", "\"variables\": {}"),
        (
            "\"operation_id\": \"listItem\"",
            "\"operationId\": \"listItem\"",
        ),
    ] {
        let mutated = json.replace(from, to);
        assert_ne!(mutated, json, "{from}");
        assert!(HttpIr::from_json(&mutated).is_err(), "{to}: {mutated}");
    }
}

/// A protocol binding's own servers are held to the rule under `refined.`, and the server binding's
/// servers are not compared with them.
#[test]
fn a_refined_bindings_servers_are_held_to_the_rule_and_not_compared() {
    let ir = two_components();
    let protocol = "servers:
  items-worker:
    - {url: /worker}
routes:
  - command: catalog.items.ListItem
    method: post
";
    let refined = Refined::from_yaml((header(&ir) + protocol).as_bytes()).expect("parses");
    let text = header(&ir)
        + &format!("refines: {}\n", refined.digest())
        + "routes:
  - command: catalog.items.ListItem
    method: post
    path: /items
";
    let spec = HttpSpec::from_yaml(&text).expect("parses");
    assert_eq!(
        located(compile_refining(&spec, Some(&refined), &ir, &derived(&ir))),
        [refused("ESS-HTTP-033", "refined.servers.items-worker")]
    );

    let protocol = "servers:
  items-service:
    - {url: /protocol}
routes:
  - command: catalog.items.ListItem
    operation_id: listItem
    method: post
";
    let refined = Refined::from_yaml((header(&ir) + protocol).as_bytes()).expect("parses");
    let text = header(&ir)
        + &format!("refines: {}\n", refined.digest())
        + "servers:
  items-service:
    - {url: 'https://api.example.test'}
routes:
  - command: catalog.items.ListItem
    operation_id: listItem
    method: post
    path: /items
";
    let spec = HttpSpec::from_yaml(&text).expect("parses");
    let http = compile_refining(&spec, Some(&refined), &ir, &derived(&ir)).expect("compiles");
    assert_eq!(
        http.servers()["items-service"][0].url,
        "https://api.example.test"
    );
}

/// A refinement may not move a fixed id to another operation and leave the fixed one without it.
#[test]
fn a_refinement_may_not_move_a_fixed_operation_id_to_another_route() {
    let ir = model();
    let protocol = "routes:
  - command: catalog.items.ListItem
    operation_id: listItem
    method: post
  - command: catalog.items.TouchItem
    method: post
    path: /touch
";
    let refined = Refined::from_yaml((header(&ir) + protocol).as_bytes()).expect("parses");
    let text = header(&ir)
        + &format!("refines: {}\n", refined.digest())
        + "routes:
  - command: catalog.items.ListItem
    method: post
    path: /items
  - command: catalog.items.TouchItem
    operation_id: listItem
    method: post
    path: /touch
";
    let spec = HttpSpec::from_yaml(&text).expect("parses");
    assert_eq!(
        located(compile_refining(&spec, Some(&refined), &ir, &derived(&ir))),
        [refused("ESS-HTTP-012", "routes[0].operation_id")]
    );
}

/// An authored id that YAML reads as a number is outside the grammar (`true` reads as the text
/// `true`, which is inside it). Whatever the
/// reader does with it, it must not be admitted as an id.
#[test]
fn a_non_string_operation_id_is_not_admitted() {
    let ir = model();
    for scalar in ["123", "1.5", "-1"] {
        let text = header(&ir)
            + &format!(
                "routes:
  - command: catalog.items.ListItem
    operation_id: {scalar}
    method: post
"
            );
        if let Ok(spec) = HttpSpec::from_yaml(&text) {
            assert_eq!(
                located(compile(&spec, &ir, &derived(&ir))),
                [refused("ESS-HTTP-032", "routes[0].operation_id")],
                "{scalar}"
            );
        }
    }
}
