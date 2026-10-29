//! A route may override a standard refusal for itself (`routes[].refusals`), a `single` view's
//! empty answer may carry a literal body, and a view route may answer its rows as a bare array
//! (`story:ess-http-admits-text-bodies-route-refusals-empty-bodies-and-array-views`, U5b;
//! `docs/design/http-presentation-binding.md`, "Standard refusals" and "Response body per
//! outcome").

use ess_compiler::resolve::compile as compile_model;
use ess_compiler::source::SourceMap;
use ess_compiler::EssIr;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_http::{
    compile, compile_refining, DerivedRoute, HttpDiagnostics, HttpIr, HttpSpec, Method, Operation,
    Refined,
};
use serde_json::{json, Value};

const MODEL: &str = include_str!("fixtures/orders.yaml");

fn model() -> EssIr {
    let raw = RawSpecFile::parse(MODEL).expect("well formed");
    let spec = Specification::assemble([(Source::new("orders.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("validates: {errors}"));
    let mut sources = SourceMap::new();
    sources.insert("orders.yaml", MODEL);
    compile_model(&spec, &sources).unwrap_or_else(|errors| panic!("compiles: {errors}"))
}

/// The routes ESS derives for `ir`, as `tests/outcomes.rs` writes them.
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

fn ir_json(http: &HttpIr) -> Value {
    serde_json::from_str(&http.to_canonical_json()).expect("canonical JSON")
}

/// Compiles, writes, reads back and writes again: the bytes do not move.
fn round_trips(http: &HttpIr) {
    let text = http.to_canonical_json();
    let read = HttpIr::from_json(&text)
        .unwrap_or_else(|error| panic!("the compiled IR reads back: {error}\n{text}"));
    assert_eq!(read.to_canonical_json(), text, "round trip changed bytes");
}

/// `PlaceOrder` on `post /api/orders` with `keys`, and `NoteOrder` on `post /api/orders/note`
/// with none: the IR orders them so, too.
fn two_commands(keys: &str) -> String {
    format!(
        "routes:\n  - command: catalog.orders.PlaceOrder\n    method: post\n    path: /api/orders\n{keys}  - command: catalog.orders.NoteOrder\n    method: post\n    path: /api/orders/note\n"
    )
}

/// A view route on `get /orders/{order_id}` with `keys` beside its path.
fn view_route(view: &str, keys: &str) -> String {
    format!(
        "routes:\n  - view: catalog.orders.{view}\n    method: get\n    path: /orders/{{order_id}}\n    parameters: {{order_id: param.order_id}}\n{keys}"
    )
}

const TOP_LEVEL: &str = "refusals:
  malformed: {status: 400, body: {error: {literal: invalid_request}}}
  not_granted: {status: 403, body: {error: {literal: access_denied}}}
";

// --- route refusals -----------------------------------------------------------------------------

#[test]
fn a_route_refusal_overrides_the_binding_refusal() {
    let ir = model();
    let document = TOP_LEVEL.to_owned()
        + &two_commands(
            "    refusals:\n      not_granted: {status: 404, body: none}\n      malformed: {status: 400, body: {error: {literal: unreadable}}}\n",
        );
    let http = compiled(&ir, &document).unwrap_or_else(|errors| panic!("compiles: {errors}"));
    let json = ir_json(&http);
    assert_eq!(
        json["routes"][0]["refusals"],
        json!({
            "malformed": {"status": 400, "body": {"error": {"literal": "unreadable"}}},
            "not_granted": {"status": 404, "body": "none"},
        }),
        "{json:#}"
    );
    assert!(json["routes"][1].get("refusals").is_none(), "{json:#}");
    assert_eq!(json["refusals"]["not_granted"]["status"], 403, "{json:#}");
    let route = &http.routes()[0];
    assert_eq!(
        route.refusals[&ess_http::RefusalKey::NotGranted].status,
        404
    );
    round_trips(&http);

    // Each override is held to the rules of `refusals:`, at its own path.
    for (keys, path) in [
        (
            "    refusals:\n      not_granted: {status: 500, body: none}\n",
            "routes[0].refusals.not_granted.status",
        ),
        (
            "    refusals:\n      not_granted: {status: 404, body: {error: error}}\n",
            "routes[0].refusals.not_granted.body",
        ),
        (
            "    refusals:\n      forbidden: {status: 404, body: none}\n",
            "routes[0].refusals.forbidden",
        ),
        (
            "    refusals:\n      unfinished: {status: 503, body: none, headers: {X-Item: {source: input.order_id}}}\n",
            "routes[0].refusals.unfinished.headers.X-Item",
        ),
    ] {
        let located = located(compiled(&ir, &two_commands(keys)));
        assert!(
            located
                .iter()
                .any(|(code, at)| (*code == "ESS-HTTP-029" || *code == "ESS-HTTP-026") && at == path),
            "{keys}: {located:?}"
        );
    }
}

#[test]
fn a_route_refusal_is_held_apart_from_its_outcomes() {
    let ir = model();
    let outcomes = "    outcomes:\n      out-of-stock: {status: 409, body: {error: error}}\n";
    // A route refusal on an outcome's status, with a body no reader tells apart from it.
    let located = located(compiled(
        &ir,
        &two_commands(&format!(
            "{outcomes}    refusals:\n      malformed: {{status: 409, body: none}}\n"
        )),
    ));
    assert_eq!(
        located,
        [refused("ESS-HTTP-025", "routes[0].refusals.malformed")]
    );
    // One a literal under the error's member tells apart compiles.
    compiled(
        &ir,
        &two_commands(&format!(
            "{outcomes}    refusals:\n      malformed: {{status: 409, body: {{error: {{literal: invalid_request}}}}}}\n"
        )),
    )
    .unwrap_or_else(|errors| panic!("compiles: {errors}"));
    // The route's own refusal replaces the binding's on that route, so the binding's is not
    // counted there.
    compiled(
        &ir,
        &("refusals:\n  malformed: {status: 409, body: none}\n".to_owned()
            + &two_commands(&format!(
                "{outcomes}    refusals:\n      malformed: {{status: 400, body: none}}\n"
            ))),
    )
    .unwrap_or_else(|errors| panic!("compiles: {errors}"));
    // Without it, the binding's refusal is counted on the route.
    assert_eq!(
        located_of(
            &ir,
            &("refusals:\n  malformed: {status: 409, body: none}\n".to_owned()
                + &two_commands(outcomes))
        ),
        [refused("ESS-HTTP-025", "refusals.malformed")]
    );
}

fn located_of(ir: &EssIr, document: &str) -> Vec<(&'static str, String)> {
    located(compiled(ir, document))
}

#[test]
fn a_refinement_restates_route_refusals() {
    let ir = model();
    let base = two_commands("    refusals:\n      malformed: {status: 400, body: none}\n");
    let refined = Refined::from_yaml((header(&ir) + &base).as_bytes()).expect("parses");
    let refining = |document: &str| {
        let text = header(&ir) + &format!("refines: {}\n", refined.digest()) + document;
        let spec = HttpSpec::from_yaml(&text).expect("parses");
        compile_refining(&spec, Some(&refined), &ir, &derived(&ir))
    };
    assert_eq!(located(refining(&base)), []);
    for changed in [
        two_commands("    refusals:\n      malformed: {status: 415, body: none}\n"),
        two_commands(""),
        two_commands(
            "    refusals:\n      malformed: {status: 400, body: none}\n      unfinished: {status: 503, body: none}\n",
        ),
    ] {
        let located = located(refining(&changed));
        assert!(
            located.iter().any(|(code, path)| *code == "ESS-HTTP-012"
                && path.starts_with("routes[0].refusals.")),
            "{changed}: {located:?}"
        );
    }
    let headers = two_commands(
        "    refusals:\n      malformed: {status: 400, body: none, headers: {Cache-Control: {literal: no-store}}}\n",
    );
    assert!(located(refining(&headers)).contains(&refused(
        "ESS-HTTP-012",
        "routes[0].refusals.malformed.headers"
    )));
}

// --- an empty answer's body ---------------------------------------------------------------------

#[test]
fn an_empty_answer_may_carry_a_literal_body() {
    let ir = model();
    let http = compiled(
        &ir,
        &view_route(
            "OrderById",
            "    body: single\n    empty: {status: 404, body: {error: {literal: not_found}}}\n",
        ),
    )
    .unwrap_or_else(|errors| panic!("compiles: {errors}"));
    assert_eq!(
        ir_json(&http)["routes"][0]["empty"],
        json!({"status": 404, "body": {"error": {"literal": "not_found"}}})
    );
    round_trips(&http);
    // Without a body, and with `none` written out, the IR keeps its bytes: no `body`.
    for empty in ["{status: 404}", "{status: 404, body: none}"] {
        let http = compiled(
            &ir,
            &view_route(
                "OrderById",
                &format!("    body: single\n    empty: {empty}\n"),
            ),
        )
        .unwrap_or_else(|errors| panic!("compiles: {errors}"));
        assert_eq!(ir_json(&http)["routes"][0]["empty"], json!({"status": 404}));
    }
    for body in [
        "{error: error}",
        "response",
        "envelope",
        "{'': {literal: not_found}}",
    ] {
        let located = located(compiled(
            &ir,
            &view_route(
                "OrderById",
                &format!("    body: single\n    empty: {{status: 404, body: {body}}}\n"),
            ),
        ));
        assert_eq!(
            located,
            [refused("ESS-HTTP-030", "routes[0].empty.body")],
            "{body}"
        );
    }
}

// --- an array view ------------------------------------------------------------------------------

#[test]
fn a_view_route_answers_a_bare_array() {
    let ir = model();
    let http = compiled(
        &ir,
        &view_route("OrdersOfSize", "    body: array\n").replace(
            "/orders/{order_id}\n    parameters: {order_id: param.order_id}",
            "/orders/size/{quantity}\n    parameters: {quantity: param.quantity}",
        ),
    )
    .unwrap_or_else(|errors| panic!("compiles: {errors}"));
    assert_eq!(ir_json(&http)["routes"][0]["body"], "array");
    round_trips(&http);
    // A paged view's answer carries its page, which a bare array cannot.
    assert_eq!(
        located(compiled(&ir, &view_route("OrderPage", "    body: array\n"))),
        [refused("ESS-HTTP-024", "routes[0].body")]
    );
    // `empty` answers a `single` view.
    assert_eq!(
        located(compiled(
            &ir,
            &view_route("OrderById", "    body: array\n    empty: {status: 404}\n")
        )),
        [refused("ESS-HTTP-030", "routes[0]")]
    );
}

// --- the IR reader ------------------------------------------------------------------------------

#[test]
fn compiled_ir_with_view_answers_out_of_rules_is_refused() {
    let ir = model();
    let single = compiled(
        &ir,
        &view_route("OrderById", "    body: single\n    empty: {status: 404}\n"),
    )
    .expect("compiles")
    .to_canonical_json();
    let empty = "\"empty\": {\n        \"status\": 404\n      }";
    assert!(single.contains(empty), "{single}");
    for (written, named) in [
        ("\"empty\": {\"status\": 404, \"body\": \"none\"}", "none"),
        (
            "\"empty\": {\"status\": 404, \"body\": {\"\": {\"literal\": \"x\"}}}",
            "member",
        ),
    ] {
        let broken = single.replace(empty, written);
        assert_ne!(broken, single);
        let error = HttpIr::from_json(&broken).expect_err(written).to_string();
        assert!(error.contains(named), "{written}: {error}");
    }
    let broken = single.replace("\"body\": \"single\"", "\"body\": \"array\"");
    assert_ne!(broken, single);
    let error = HttpIr::from_json(&broken)
        .expect_err("array beside empty")
        .to_string();
    assert!(error.contains("empty"), "{error}");

    let refusing = compiled(
        &ir,
        &two_commands(
            "    refusals:\n      malformed: {status: 400, body: none}\n      not_granted: {status: 404, body: none}\n",
        ),
    )
    .expect("compiles")
    .to_canonical_json();
    let route = ir_json(&HttpIr::from_json(&refusing).expect("reads back"));
    let written = serde_json::to_string(&route["routes"][0]["refusals"]).expect("JSON");
    let path = "\"path\": \"/api/orders\"";
    let without = |text: &str| {
        let value: Value = serde_json::from_str(text).expect("JSON");
        let mut value = value;
        value["routes"][0]
            .as_object_mut()
            .expect("a route")
            .remove("refusals");
        serde_json::to_string_pretty(&value).expect("JSON")
    };
    let bare = without(&refusing);
    assert!(bare.contains(path), "{bare}");
    for (refusals, named) in [
        (
            "{\"not_granted\": {\"status\": 404, \"body\": \"none\"}, \"malformed\": {\"status\": 400, \"body\": \"none\"}}"
                .to_owned(),
            "refusals",
        ),
        ("{}".to_owned(), "refusals"),
        (
            "{\"malformed\": {\"status\": 522, \"body\": \"none\"}}".to_owned(),
            "4xx",
        ),
    ] {
        let broken = bare.replace(path, &format!("{path}, \"refusals\": {refusals}"));
        assert_ne!(broken, bare);
        let error = HttpIr::from_json(&broken).expect_err(&refusals).to_string();
        assert!(error.contains(named), "{refusals}: {error}");
    }
    // Written as compiled, it reads.
    let fine = bare.replace(path, &format!("{path}, \"refusals\": {written}"));
    HttpIr::from_json(&fine).unwrap_or_else(|error| panic!("reads: {error}"));
}
