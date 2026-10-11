//! Adversary pass 2 on `ess-http/1` path templates, parameters and route collisions: the
//! correction of pass 1, and `HttpIr::from_json` on hand-edited IR.

use ess_compiler::resolve::compile as compile_model;
use ess_compiler::source::SourceMap;
use ess_compiler::EssIr;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_http::{compile, DerivedRoute, HttpIr, HttpSpec, Method, Operation};

const MODEL: &str = include_str!("fixtures/catalog.yaml");

fn model() -> EssIr {
    let raw = RawSpecFile::parse(MODEL).expect("well formed");
    let spec = Specification::assemble([(Source::new("catalog.yaml"), raw)]).expect("validates");
    let mut sources = SourceMap::new();
    sources.insert("catalog.yaml", MODEL);
    compile_model(&spec, &sources).expect("compiles")
}

/// `post /items/commands/<command>` per accepted command, `get /items/views/<view>` per view of
/// the network component: what `ess_gen::http::routes` derives for the fixture.
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
    table
}

fn header(ir: &EssIr) -> String {
    format!(
        "type: ess-http/1\nspecification:\n  system: catalog\n  version: v1\n  source_digest: sha256:{}\n",
        ir.source_digest()
    )
}

fn refusals(ir: &EssIr, routes: &str) -> Vec<(&'static str, String)> {
    let spec = HttpSpec::from_yaml(&(header(ir) + routes)).expect("parses");
    match compile(&spec, ir, &derived(ir)) {
        Ok(_) => Vec::new(),
        Err(diagnostics) => diagnostics
            .0
            .iter()
            .map(|diagnostic| (diagnostic.code, diagnostic.path.clone()))
            .collect(),
    }
}

fn refused(code: &'static str, path: &str) -> (&'static str, String) {
    (code, path.to_owned())
}

/// Compiled IR of a binding naming only `TouchItem` on `get /items/{item_id}/touch`, so it holds
/// one route with parameters and two derived routes.
fn compiled(ir: &EssIr) -> String {
    let spec = HttpSpec::from_yaml(
        &(header(ir)
            + "routes:\n  - {command: catalog.items.TouchItem, method: get, path: '/items/{item_id}/touch', parameters: {item_id: input.item_id}}\n"),
    )
    .expect("parses");
    compile(&spec, ir, &derived(ir))
        .expect("compiles")
        .to_canonical_json()
}

fn edited(json: &str, edit: impl FnOnce(&mut serde_json::Value)) -> String {
    let mut value: serde_json::Value = serde_json::from_str(json).unwrap();
    edit(&mut value);
    let text = serde_json::to_string_pretty(&value).unwrap() + "\n";
    assert_ne!(text, json, "the edit changed nothing");
    text
}

// --- The correction, compiled -------------------------------------------------------------------

/// The F3 decision: refused "at the second variable in path order", not in mapping order.
#[test]
fn a_source_read_twice_is_refused_at_the_second_variable_of_the_path() {
    let ir = model();
    assert_eq!(
        refusals(
            &ir,
            "routes:\n  - {command: catalog.items.TouchItem, method: post, path: '/touch/{b}/{a}', parameters: {a: input.item_id, b: input.item_id}}\n"
        ),
        [refused("ESS-HTTP-008", "routes[0].parameters.a")]
    );
}

/// An operation routed twice (`ESS-HTTP-005`) takes part in collision checks with both of its
/// routes, and never collides with itself.
#[test]
fn both_routes_of_an_operation_routed_twice_are_checked_for_collisions() {
    let ir = model();
    assert_eq!(
        refusals(
            &ir,
            "routes:
  - {command: catalog.items.ListItem, method: post, path: /x}
  - {command: catalog.items.ListItem, method: post, path: /y}
  - {command: catalog.items.TouchItem, method: post, path: /y}
"
        ),
        [
            refused("ESS-HTTP-005", "routes[1]"),
            refused("ESS-HTTP-010", "routes[2]"),
        ]
    );
}

// --- `HttpIr::from_json` on hand-edited IR ------------------------------------------------------

/// Compiled IR never routes one operation twice (`ESS-HTTP-005`); `HttpIr::route` returns the
/// first, so a reader that admits two answers for the operation silently drops the other.
#[test]
fn ir_routing_one_operation_twice_does_not_read_back() {
    let ir = model();
    let json = compiled(&ir);
    let twice = edited(&json, |value| {
        let routes = value["routes"].as_array_mut().unwrap();
        let mut again = routes[0].clone();
        again["path"] = serde_json::json!("/items/{item_id}/touch/again");
        routes.push(again);
    });
    assert!(
        HttpIr::from_json(&twice).is_err(),
        "IR routing `TouchItem` twice reads back"
    );
}

/// `derived_routes` holds "the derived route of every operation the binding does not name".
#[test]
fn ir_holding_a_derived_route_for_a_bound_operation_does_not_read_back() {
    let ir = model();
    let json = compiled(&ir);
    let both = edited(&json, |value| {
        let derived = value["derived_routes"].as_array_mut().unwrap();
        // Between `ListItem` and `ItemById`, so the IR stays in canonical order.
        derived.insert(
            1,
            serde_json::json!({
                "component": "items-service",
                "command": "catalog.items.TouchItem",
                "method": "post",
                "path": "/items/commands/TouchItem",
            }),
        );
    });
    assert!(
        HttpIr::from_json(&both).is_err(),
        "IR holding a route and a derived route for `TouchItem` reads back"
    );
}

/// `DerivedRoute::path` is "the derived path, which holds no variable", and `from_json` refuses a
/// route path that breaks the grammar; a derived path is read with no check at all.
#[test]
fn ir_with_a_derived_path_that_is_no_literal_path_does_not_read_back() {
    let ir = model();
    let json = compiled(&ir);
    // Each sorts before `/items/views/ItemById`, the derived route after it, so the IR stays in
    // canonical order and only the path is wrong.
    for path in ["/items/commands/{x}", "-items/commands/ListItem", "/a//b"] {
        let broken = json.replacen("/items/commands/ListItem", path, 1);
        assert_ne!(broken, json);
        assert!(
            HttpIr::from_json(&broken).is_err(),
            "a derived path `{path}` reads back"
        );
    }
}

/// A compiled route maps each variable of its path exactly once and nothing else; IR whose route
/// maps no variable of `/items/{item_id}/touch`, or maps one the path does not have, reads back.
#[test]
fn ir_whose_parameters_disagree_with_its_path_does_not_read_back() {
    let ir = model();
    let json = compiled(&ir);
    let unmapped = edited(&json, |value| {
        value["routes"][0]
            .as_object_mut()
            .unwrap()
            .remove("parameters");
    });
    assert!(
        HttpIr::from_json(&unmapped).is_err(),
        "a route with an unmapped path variable reads back"
    );
    let stray = edited(&json, |value| {
        value["routes"][0]["parameters"]["zz"] = serde_json::json!("input.item_id");
    });
    assert!(
        HttpIr::from_json(&stray).is_err(),
        "a route mapping a variable its path does not have reads back"
    );
}

/// The decided contract of `HttpIr::from_json`: it reads canonical or compact JSON in canonical
/// order and writes canonical bytes. Compact JSON of the same IR reads back, writes back the
/// canonical pretty bytes, and writing those bytes again is a fixed point.
#[test]
fn compact_ir_reads_back_and_writes_back_canonical_bytes() {
    let ir = model();
    let json = compiled(&ir);
    let value: serde_json::Value = serde_json::from_str(&json).unwrap();
    let compact = serde_json::to_string(&value).unwrap();
    assert_ne!(compact, json, "the compact form is other bytes");
    let written = HttpIr::from_json(&compact)
        .expect("compact IR in canonical order reads back")
        .to_canonical_json();
    assert_eq!(written, json, "compact IR writes back the canonical bytes");
    assert_eq!(
        HttpIr::from_json(&written)
            .expect("canonical IR reads back")
            .to_canonical_json(),
        written,
        "canonical bytes are a fixed point"
    );
}
