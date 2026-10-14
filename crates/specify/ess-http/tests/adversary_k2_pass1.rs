//! Adversarial cases for the stage-2 answers of an `ess-http/1` route (outcome status and body,
//! view `single` and `empty`, top-level `refusals:`), held to
//! `docs/design/http-presentation-binding.md`, "Stage 2: keys, precedence and the units that build
//! them", and to `compile`'s own table of refusals.

use ess_compiler::resolve::compile as compile_model;
use ess_compiler::source::SourceMap;
use ess_compiler::EssIr;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_http::{compile, DerivedRoute, HttpDiagnostics, HttpIr, HttpSpec, Method, Operation};

const MODEL: &str = include_str!("fixtures/orders.yaml");

/// Views the fixture lacks, each selecting by something that is not `<identity> == param.<name>`
/// at the top level or in a top-level `all`.
const MORE_VIEWS: &str = "  - name: catalog.orders.OrderByLiteral
    source: catalog.orders.Order
    filter: 'order_id == \"o-1\"'
    fields:
      - {name: order_id, type: catalog.orders.OrderId}
  - name: catalog.orders.OrderByIdInAnyInAll
    source: catalog.orders.Order
    params:
      - {name: order_id, type: catalog.orders.OrderId}
    filter: {all: [quantity > 0, {any: [order_id == param.order_id, quantity > 10]}]}
    fields:
      - {name: order_id, type: catalog.orders.OrderId}
components:
";

fn model() -> EssIr {
    let text = MODEL.replacen("components:\n", MORE_VIEWS, 1);
    assert_ne!(text, MODEL);
    let raw = RawSpecFile::parse(&text).expect("well formed");
    let spec = Specification::assemble([(Source::new("orders.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("validates: {errors}"));
    let mut sources = SourceMap::new();
    sources.insert("orders.yaml", &text);
    compile_model(&spec, &sources).unwrap_or_else(|errors| panic!("compiles: {errors}"))
}

fn derived(ir: &EssIr) -> Vec<DerivedRoute> {
    let mut table = Vec::new();
    for component in ir.components().values() {
        for handle in &component.accepts {
            table.push(DerivedRoute {
                component: component.name.to_string(),
                operation: Operation::Command(handle.name().to_string()),
                method: Method::Post,
                path: format!("/orders/commands/{}", handle.name().local()),
            });
        }
        for domain in &component.owns {
            for handle in &ir.domain(domain).views {
                table.push(DerivedRoute {
                    component: component.name.to_string(),
                    operation: Operation::View(handle.name().to_string()),
                    method: Method::Get,
                    path: format!("/orders/views/{}", handle.name().local()),
                });
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

fn compiled(ir: &EssIr, document: &str) -> Result<HttpIr, HttpDiagnostics> {
    let spec = HttpSpec::from_yaml(&(header(ir) + document))
        .unwrap_or_else(|error| panic!("parses: {error}\n{document}"));
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

fn place_order(outcomes: &str) -> String {
    format!(
        "routes:\n  - command: catalog.orders.PlaceOrder\n    method: post\n    path: /api/orders\n    outcomes:\n{outcomes}"
    )
}

fn view_route(view: &str, keys: &str) -> String {
    format!(
        "routes:\n  - view: catalog.orders.{view}\n    method: get\n    path: /orders/{{order_id}}\n    parameters: {{order_id: param.order_id}}\n{keys}"
    )
}

/// `compile`'s table: ESS-HTTP-025 is "Reported once, at the first listed outcome in document
/// order, naming the others and the status". Three outcomes on one status with one body that tells
/// none of them apart are one collision.
#[test]
fn three_outcomes_on_one_status_no_body_tells_apart_are_reported_once() {
    let ir = model();
    let result = compiled(
        &ir,
        &place_order(
            "      out-of-stock: {status: 409, body: none}\n      too-many: {status: 409, body: none}\n      mislabelled: {status: 409, body: none}\n",
        ),
    );
    assert_eq!(
        located(result),
        [refused("ESS-HTTP-025", "routes[0].outcomes.out-of-stock")]
    );
}

/// One mistake, one refusal: `204` on an outcome with an error is refused for its class
/// (`ESS-HTTP-023`); the status is not compiled, so the body is not held to a `204` it will never
/// answer. Writing `body: none` does not change the mistake, and gives one refusal.
#[test]
fn a_refused_204_status_is_not_also_refused_for_its_body() {
    let ir = model();
    let with_none = compiled(
        &ir,
        &place_order("      out-of-stock: {status: 204, body: none}\n"),
    );
    assert_eq!(
        located(with_none),
        [refused(
            "ESS-HTTP-023",
            "routes[0].outcomes.out-of-stock.status"
        )]
    );
    let without_body = compiled(&ir, &place_order("      out-of-stock: {status: 204}\n"));
    assert_eq!(
        located(without_body),
        [refused(
            "ESS-HTTP-023",
            "routes[0].outcomes.out-of-stock.status"
        )]
    );
}

/// One mistake, one refusal: an `empty` on a route that answers `rows` is refused as `empty`
/// without `single`; its status is not held to a `4xx` as well. On a command route the same `empty`
/// already earns exactly one `ESS-HTTP-030`.
#[test]
fn an_empty_without_single_is_refused_once_whatever_its_status() {
    let ir = model();
    let command = "routes:
  - command: catalog.orders.NoteOrder
    method: post
    empty: {status: 200}
";
    assert_eq!(
        located(compiled(&ir, command)),
        [refused("ESS-HTTP-030", "routes[0]")]
    );
    assert_eq!(
        located(compiled(
            &ir,
            &view_route("OrderById", "    empty: {status: 200}\n")
        )),
        [refused("ESS-HTTP-030", "routes[0]")]
    );
}

/// `HttpIr::from_json` reads canonical IR only: compiled IR omits an empty `outcomes` and an empty
/// `refusals`, so IR holding either is refused rather than read and written back as other bytes.
#[test]
fn compiled_ir_holding_an_empty_outcomes_or_refusals_is_refused() {
    let ir = model();
    let http = compiled(
        &ir,
        "routes:\n  - command: catalog.orders.PlaceOrder\n    method: post\n    path: /api/orders\n",
    )
    .expect("compiles");
    let text = http.to_canonical_json();
    let route = "\"command\": \"catalog.orders.PlaceOrder\",";
    assert!(text.contains(route), "{text}");
    let outcomes = text.replace(
        route,
        "\"command\": \"catalog.orders.PlaceOrder\", \"outcomes\": {},",
    );
    let specification = "\"specification\": {";
    assert!(text.contains(specification), "{text}");
    let refusals = text.replace(specification, "\"refusals\": {}, \"specification\": {");
    for broken in [outcomes, refusals] {
        assert_ne!(broken, text);
        match HttpIr::from_json(&broken) {
            Err(_) => {}
            Ok(read) => assert_eq!(
                read.to_canonical_json(),
                broken,
                "read non-canonical IR and wrote back other bytes"
            ),
        }
    }
}

/// `ESS-HTTP-031`: the identity compared with a literal, or with a parameter only inside an `any`
/// beneath a top-level `all`, does not select one row by a parameter.
#[test]
fn single_on_a_view_selecting_by_a_literal_or_an_any_in_all_is_refused() {
    let ir = model();
    for view in ["OrderByLiteral", "OrderByIdInAnyInAll"] {
        let document = format!(
            "routes:\n  - view: catalog.orders.{view}\n    method: get\n    body: single\n    empty: {{status: 404}}\n"
        );
        assert_eq!(
            located(compiled(&ir, &document)),
            [refused("ESS-HTTP-031", "routes[0].body")],
            "{view}"
        );
    }
}

/// Compile, read, write: the canonical bytes of a binding using every stage-2 key of this unit read
/// back and write back unchanged, and a document listing the same answers in another order
/// compiles to the same bytes.
#[test]
fn every_k2_key_round_trips_and_authored_order_does_not_reach_the_bytes() {
    let ir = model();
    let one = "refusals:
  unfinished: {status: 503, body: none}
  malformed: {status: 400, body: {error: {literal: invalid_request}}}
routes:
  - command: catalog.orders.PlaceOrder
    method: post
    path: /api/orders
    outcomes:
      placed: {status: 200, body: response}
      out-of-stock: {status: 400, body: {error: error}}
      too-many: {status: 400, body: {error: error}}
  - view: catalog.orders.OrderById
    method: get
    path: /orders/{order_id}
    parameters: {order_id: param.order_id}
    body: single
    empty: {status: 404}
";
    let other = "refusals:
  malformed: {body: {error: {literal: invalid_request}}, status: 400}
  unfinished: {body: none, status: 503}
routes:
  - view: catalog.orders.OrderById
    method: get
    path: /orders/{order_id}
    parameters: {order_id: param.order_id}
    empty: {status: 404}
    body: single
  - command: catalog.orders.PlaceOrder
    method: post
    path: /api/orders
    outcomes:
      too-many: {body: {error: error}, status: 400}
      out-of-stock: {status: 400, body: {error: error}}
      placed: {body: response, status: 200}
";
    let first = compiled(&ir, one).expect("compiles").to_canonical_json();
    let second = compiled(&ir, other).expect("compiles").to_canonical_json();
    assert_eq!(first, second);
    let read = HttpIr::from_json(&first).expect("reads back");
    assert_eq!(read.to_canonical_json(), first);
}
