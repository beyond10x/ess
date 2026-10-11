//! Adversary pass 1 on `ess-http/1` path templates, parameters and route collisions
//! (`docs/design/http-presentation-binding.md`, "Path templates", "What compiling checks
//! (stage 1)", "Formats and determinism (stage 1)").
//!
//! Each case asserts what the design says and fails where the implementation does otherwise.

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

/// The table `ess_gen::http::routes` derives for the fixture: `post /items/commands/<command>`
/// for each accepted command and `get /items/views/<view>` for the network component's view.
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

/// The design: "An operation the binding does not name keeps its derived route." `ListItem` is
/// named, so its derived `post /items/commands/ListItem` is gone from the surface whether or not
/// its own route earns a refusal. A refusal of `TouchItem` for claiming that path reports a
/// collision with a route the binding replaced.
#[test]
fn a_route_refused_for_its_source_still_replaces_its_derived_route() {
    let ir = model();
    let found = refusals(
        &ir,
        "routes:
  - {command: catalog.items.ListItem, method: post, path: '/items/{id}', parameters: {id: input.missing}}
  - {command: catalog.items.TouchItem, method: post, path: /items/commands/ListItem}
",
    );
    assert_eq!(
        found,
        [refused("ESS-HTTP-009", "routes[0].parameters.id")],
        "a collision with the derived route of an operation the binding names"
    );
}

/// The same with the named operation's method refused (`ESS-HTTP-006`).
#[test]
fn a_route_refused_for_its_method_still_replaces_its_derived_route() {
    let ir = model();
    let found = refusals(
        &ir,
        "routes:
  - {command: catalog.items.ListItem, method: head, path: /orders}
  - {command: catalog.items.TouchItem, method: post, path: /items/commands/ListItem}
",
    );
    assert_eq!(
        found,
        [refused("ESS-HTTP-006", "routes[0].method")],
        "a collision with the derived route of an operation the binding names"
    );
}

/// "Every refusal is returned at once." Both paths parse, so the collision between them is known
/// whatever is wrong with `routes[0]`'s source; it is reported only once the source is fixed.
#[test]
fn a_collision_is_reported_beside_a_refused_source() {
    let ir = model();
    let found = refusals(
        &ir,
        "routes:
  - {command: catalog.items.ListItem, method: post, path: '/items/{id}', parameters: {id: input.missing}}
  - {command: catalog.items.TouchItem, method: post, path: '/items/{x}', parameters: {x: input.item_id}}
",
    );
    assert!(
        found.contains(&refused("ESS-HTTP-009", "routes[0].parameters.id")),
        "{found:?}"
    );
    assert!(
        found.iter().any(|(code, _)| *code == "ESS-HTTP-010"),
        "the collision of two parsed templates is not reported with the other refusal: {found:?}"
    );
}

/// The same with `ESS-HTTP-014` on the first route.
#[test]
fn a_collision_is_reported_beside_an_unmapped_get_input() {
    let ir = model();
    let found = refusals(
        &ir,
        "routes:
  - {command: catalog.items.ListItem, method: get, path: '/items/{id}', parameters: {id: input.item_id}}
  - {view: catalog.items.ItemById, method: get, path: '/items/{item_id}', parameters: {item_id: param.item_id}}
",
    );
    assert!(
        found.contains(&refused("ESS-HTTP-014", "routes[0]")),
        "{found:?}"
    );
    assert!(
        found.iter().any(|(code, _)| *code == "ESS-HTTP-010"),
        "the collision of two parsed templates is not reported with the other refusal: {found:?}"
    );
}

/// `Parameters` keeps a key written twice so `compile` refuses it as `ESS-HTTP-008` "rather than
/// one entry silently replacing the other". YAML is the authored format the design shows; the
/// existing case covers JSON only.
#[test]
fn a_variable_mapped_twice_in_yaml_is_refused_with_ess_http_008() {
    let ir = model();
    let text = header(&ir)
        + "routes:
  - view: catalog.items.ItemById
    method: get
    path: /items/{item_id}
    parameters:
      item_id: param.item_id
      item_id: param.item_id
";
    let spec = match HttpSpec::from_yaml(&text) {
        Ok(spec) => spec,
        Err(error) => panic!("refused by the YAML reader with no code, not ESS-HTTP-008: {error}"),
    };
    let found: Vec<(&str, String)> = compile(&spec, &ir, &derived(&ir))
        .expect_err("a variable mapped twice")
        .0
        .into_iter()
        .map(|diagnostic| (diagnostic.code, diagnostic.path))
        .collect();
    assert_eq!(
        found,
        [refused("ESS-HTTP-008", "routes[0].parameters.item_id")]
    );
}

/// Two variables reading one input field: `/items/1/2` gives `item_id` two values, and nothing
/// says which one the command receives. "A mapped source is a scalar a path segment can carry ...
/// a path segment is always present and holds one value."
#[test]
fn one_source_read_by_two_variables_is_refused() {
    let ir = model();
    let found = refusals(
        &ir,
        "routes:
  - {command: catalog.items.TouchItem, method: post, path: '/touch/{a}/{b}', parameters: {a: input.item_id, b: input.item_id}}
",
    );
    // `/touch/...` shares no length and no literal with a derived route, so a refusal here is
    // about the mapping alone.
    assert!(
        found.iter().any(|(code, path)| *code != "ESS-HTTP-010"
            && *code != "ESS-HTTP-011"
            && path.starts_with("routes[0].parameters")),
        "one input field read from two segments compiles: {found:?}"
    );
}

/// "The IR orders routes by path, then method", and the derived routes "ordered by component, then
/// path, then method, then operation". `HttpIr::from_json` reads canonical IR back; IR whose
/// derived routes are out of that order is either refused or written back canonical, never
/// written back as read.
#[test]
fn ir_with_derived_routes_out_of_order_does_not_round_trip_as_canonical() {
    let ir = model();
    let spec = HttpSpec::from_yaml(&(header(&ir) + "routes: []\n")).expect("parses");
    let http = compile(&spec, &ir, &derived(&ir)).expect("compiles");
    assert_eq!(http.derived_routes().len(), 3);
    let canonical = http.to_canonical_json();
    let mut value: serde_json::Value = serde_json::from_str(&canonical).unwrap();
    value["derived_routes"]
        .as_array_mut()
        .expect("derived routes")
        .reverse();
    let shuffled = serde_json::to_string_pretty(&value).unwrap() + "\n";
    assert_ne!(shuffled, canonical);
    if let Ok(read) = HttpIr::from_json(&shuffled) {
        assert_eq!(
            read.to_canonical_json(),
            canonical,
            "out-of-order IR is read and written back out of order as canonical JSON"
        );
    }
}
