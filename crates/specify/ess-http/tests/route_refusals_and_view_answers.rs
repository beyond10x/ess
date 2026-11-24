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

// --- request limits -----------------------------------------------------------------------------

#[test]
fn a_binding_that_declares_neither_key_keeps_its_ir_bytes() {
    let ir = model();
    let bare =
        compiled(&ir, &two_commands("")).unwrap_or_else(|errors| panic!("compiles: {errors}"));
    let bound = compiled(
        &ir,
        &(TOP_LEVEL.to_owned()
            + &two_commands(
                "    refusals:\n      malformed: {status: 400, body: none}\n      not_granted: {status: 404, body: none}\n",
            )),
    )
    .unwrap_or_else(|errors| panic!("compiles: {errors}"));
    for (name, http, pinned) in [
        (
            "bare",
            &bare,
            "sha256:8d4577309106eec43667ddb21b05765c9ad9af7d35aa95355433aaeb65e194d3",
        ),
        (
            "bound",
            &bound,
            "sha256:518c82b2b8f2d4eab45752ca53f51b46fdb54a1bd62b8878e6332a4f89c9c057",
        ),
    ] {
        let text = http.to_canonical_json();
        assert!(!text.contains("limits"), "{name}: {text}");
        assert!(!text.contains("too_large"), "{name}: {text}");
        assert_eq!(
            ess_http::digest(text.as_bytes()),
            pinned,
            "{name}: the IR bytes of a binding without limits moved"
        );
    }
}

const NOTE: &str = "catalog.orders.NoteOrder";

/// `NoteOrder` (its one input in the path) routed on `method`, with `keys` beside its path.
fn note_route(method: &str, keys: &str) -> String {
    format!(
        "routes:\n  - command: {NOTE}\n    method: {method}\n    path: /orders/{{order_id}}/note\n    parameters: {{order_id: input.order_id}}\n{keys}"
    )
}

fn worded(result: Result<HttpIr, HttpDiagnostics>) -> Vec<String> {
    result
        .expect_err("refused")
        .0
        .into_iter()
        .map(|diagnostic| diagnostic.message)
        .collect()
}

#[test]
fn a_binding_limit_and_a_route_limit_compile_and_round_trip() {
    let ir = model();
    let document = "limits: {request_bytes: 64}\n".to_owned()
        + &two_commands("    limits: {request_bytes: 256}\n");
    let http = compiled(&ir, &document).unwrap_or_else(|errors| panic!("compiles: {errors}"));
    let json = ir_json(&http);
    assert_eq!(json["limits"], json!({"request_bytes": 64}), "{json:#}");
    assert_eq!(
        json["routes"][0]["limits"],
        json!({"request_bytes": 256}),
        "{json:#}"
    );
    assert!(json["routes"][1].get("limits").is_none(), "{json:#}");
    assert_eq!(http.limits().map(|limits| limits.request_bytes), Some(64));
    round_trips(&http);
    let place = http
        .route(&Operation::Command("catalog.orders.PlaceOrder".into()))
        .expect("routed");
    let note = http
        .route(&Operation::Command(NOTE.into()))
        .expect("routed");
    assert_eq!(http.request_bytes(place), Some(256));
    assert_eq!(http.request_bytes(note), Some(64));
}

#[test]
fn a_limit_is_a_whole_number_of_bytes_from_one_to_the_ceiling() {
    let ir = model();
    for bad in ["0", "-1", "16777217"] {
        assert_eq!(
            located(compiled(
                &ir,
                &format!("limits: {{request_bytes: {bad}}}\n{}", two_commands(""))
            )),
            [refused("ESS-HTTP-034", "limits.request_bytes")],
            "{bad}"
        );
        assert_eq!(
            located(compiled(
                &ir,
                &two_commands(&format!("    limits: {{request_bytes: {bad}}}\n"))
            )),
            [refused("ESS-HTTP-034", "routes[0].limits.request_bytes")],
            "{bad}"
        );
    }
    for good in ["1", "16777216"] {
        compiled(
            &ir,
            &format!("limits: {{request_bytes: {good}}}\n{}", two_commands("")),
        )
        .unwrap_or_else(|errors| panic!("{good} compiles: {errors}"));
    }
    assert_eq!(
        worded(compiled(
            &ir,
            &format!("limits: {{request_bytes: 0}}\n{}", two_commands(""))
        )),
        ["`0` is not a request size: `limits.request_bytes` is a whole number of bytes from 1 to 16777216"]
    );
    for (written, named) in [
        ("limits: {request_bytes: 64.5}", "request_bytes"),
        ("limits: {request_bytes: \"64\"}", "request_bytes"),
        ("limits: {}", "missing field `request_bytes`"),
        (
            "limits: {request_bytes: 64, extra: 1}",
            "unknown field `extra`",
        ),
    ] {
        let error = HttpSpec::from_yaml(&(header(&ir) + written + "\n" + &two_commands("")))
            .expect_err(written)
            .to_string();
        assert!(error.contains(named), "{written}: {error}");
    }
}

#[test]
fn a_route_limit_on_a_route_that_reads_no_body_is_refused() {
    let ir = model();
    let limit = "    limits: {request_bytes: 64}\n";
    let bodiless = [
        view_route("OrderById", ""),
        note_route("get", ""),
        note_route("delete", "    request: {encoding: query}\n"),
    ];
    for document in &bodiless {
        assert_eq!(
            located(compiled(&ir, &format!("{document}{limit}"))),
            [refused("ESS-HTTP-034", "routes[0].limits")],
            "{document}"
        );
        let http = compiled(&ir, &format!("limits: {{request_bytes: 64}}\n{document}"))
            .unwrap_or_else(|errors| panic!("{document}: {errors}"));
        assert!(!http.routes()[0].reads_body(), "{document}");
        assert_eq!(http.request_bytes(&http.routes()[0]), None, "{document}");
    }
    for document in [
        note_route("post", "    request: {encoding: form}\n"),
        note_route("delete", "    request: {encoding: json}\n"),
    ] {
        let http = compiled(&ir, &format!("{document}{limit}"))
            .unwrap_or_else(|errors| panic!("{document}: {errors}"));
        assert!(http.routes()[0].reads_body(), "{document}");
        assert_eq!(http.request_bytes(&http.routes()[0]), Some(64));
    }
}

#[test]
fn the_fourth_refusal_is_too_large_and_takes_any_4xx() {
    let ir = model();
    for refusal in [
        "{status: 413, body: none}",
        "{status: 418, body: {error: {literal: too_big}}, headers: {X-Limit: {literal: '64'}}}",
    ] {
        for document in [
            format!("refusals:\n  too_large: {refusal}\n{}", two_commands("")),
            two_commands(&format!("    refusals:\n      too_large: {refusal}\n")),
        ] {
            let http =
                compiled(&ir, &document).unwrap_or_else(|errors| panic!("compiles: {errors}"));
            assert!(
                ir_json(&http).to_string().contains("too_large"),
                "{document}"
            );
            round_trips(&http);
        }
    }
    for (prefix, at) in [("", "refusals"), ("    ", "routes[0].refusals")] {
        let wrap = |entries: &str| {
            if prefix.is_empty() {
                format!("refusals:\n{entries}{}", two_commands(""))
            } else {
                two_commands(&format!("    refusals:\n{entries}"))
            }
        };
        let entry = |text: &str| format!("{prefix}  {text}\n");
        let status = entry("too_large: {status: 500, body: none}");
        assert_eq!(
            located(compiled(&ir, &wrap(&status))),
            [refused("ESS-HTTP-029", &format!("{at}.too_large.status"))]
        );
        let twice = entry("too_large: {status: 413, body: none}")
            + &entry("too_large: {status: 418, body: none}");
        assert_eq!(
            located(compiled(&ir, &wrap(&twice))),
            [refused("ESS-HTTP-029", &format!("{at}.too_large"))]
        );
        let dashed = entry("too-large: {status: 413, body: none}");
        let messages = worded(compiled(&ir, &wrap(&dashed)));
        assert!(
            messages
                .iter()
                .any(|message| message.contains("`too_large` and `unfinished`")),
            "{messages:?}"
        );
    }
    compiled(
        &ir,
        &view_route(
            "OrderById",
            "    refusals:\n      too_large: {status: 413, body: none}\n",
        ),
    )
    .unwrap_or_else(|errors| panic!("a view route takes it: {errors}"));
}

const THREE_ROUTES: &str = "routes:
  - command: catalog.orders.PlaceOrder
    method: post
    path: /api/orders
    limits: {request_bytes: 256}
  - command: catalog.orders.NoteOrder
    method: post
    path: /api/orders/note
  - view: catalog.orders.OrderById
    method: get
    path: /orders/{order_id}
    parameters: {order_id: param.order_id}
";

#[test]
fn the_effective_request_bound_is_one_answer() {
    assert_eq!(ess_http::FIXED_REQUEST_BYTES, 1_048_576);
    assert_eq!(ess_http::MAX_REQUEST_BYTES, 16_777_216);
    let ir = model();
    let place = Operation::Command("catalog.orders.PlaceOrder".into());
    let note = Operation::Command(NOTE.into());
    let view = Operation::View("catalog.orders.OrderById".into());

    let http = compiled(
        &ir,
        &format!("limits: {{request_bytes: 64}}\n{THREE_ROUTES}"),
    )
    .unwrap_or_else(|errors| panic!("compiles: {errors}"));
    for (operation, reads, bytes, bounds) in [
        (&place, true, Some(256), true),
        (&note, true, Some(64), true),
        (&view, false, None, false),
    ] {
        let route = http.route(operation).expect("routed");
        assert_eq!(route.reads_body(), reads, "{operation}");
        assert_eq!(http.request_bytes(route), bytes, "{operation}");
        assert_eq!(http.bounds_request(route), bounds, "{operation}");
    }
    assert!(http.bounds_requests());

    // A `too_large` with no `limits` bounds the route: the fixed bound answers with it.
    let refused_only = THREE_ROUTES.replace(
        "    limits: {request_bytes: 256}\n",
        "    refusals:\n      too_large: {status: 413, body: none}\n",
    );
    let http = compiled(&ir, &refused_only).unwrap_or_else(|errors| panic!("compiles: {errors}"));
    let route = http.route(&place).expect("routed");
    assert_eq!(http.request_bytes(route), None);
    assert!(http.bounds_request(route));
    assert!(!http.bounds_request(http.route(&note).expect("routed")));
    assert!(http.bounds_requests());
    // At the binding, it bounds every route that reads a body, and no view.
    let at_binding = format!(
        "refusals:\n  too_large: {{status: 413, body: none}}\n{}",
        THREE_ROUTES.replace("    limits: {request_bytes: 256}\n", "")
    );
    let http = compiled(&ir, &at_binding).unwrap_or_else(|errors| panic!("compiles: {errors}"));
    assert!(http.bounds_request(http.route(&note).expect("routed")));
    assert!(!http.bounds_request(http.route(&view).expect("routed")));

    // Neither key: nothing is bounded.
    let http = compiled(
        &ir,
        &THREE_ROUTES.replace("    limits: {request_bytes: 256}\n", ""),
    )
    .unwrap_or_else(|errors| panic!("compiles: {errors}"));
    assert!(!http.bounds_requests());
    // A limit that only views and bodiless routes could carry bounds nothing.
    let bodiless = format!(
        "limits: {{request_bytes: 64}}\n{}",
        view_route("OrderById", "")
    );
    let http = compiled(&ir, &bodiless).unwrap_or_else(|errors| panic!("compiles: {errors}"));
    assert!(!http.bounds_requests());
}

#[test]
fn a_refinement_restates_limits() {
    let ir = model();
    let base = "limits: {request_bytes: 64}\n".to_owned()
        + &two_commands("    limits: {request_bytes: 256}\n");
    let refined = Refined::from_yaml((header(&ir) + &base).as_bytes()).expect("parses");
    let refining = |document: &str| {
        let text = header(&ir) + &format!("refines: {}\n", refined.digest()) + document;
        let spec = HttpSpec::from_yaml(&text).expect("parses");
        compile_refining(&spec, Some(&refined), &ir, &derived(&ir))
    };
    assert_eq!(located(refining(&base)), []);
    let at_top = |top: &str, route: &str| format!("{top}{}", two_commands(route));
    for (changed, path) in [
        (at_top("", "    limits: {request_bytes: 256}\n"), "limits"),
        (
            at_top(
                "limits: {request_bytes: 65}\n",
                "    limits: {request_bytes: 256}\n",
            ),
            "limits",
        ),
        (at_top("limits: {request_bytes: 64}\n", ""), "routes[0].limits"),
        (
            at_top(
                "limits: {request_bytes: 64}\n",
                "    limits: {request_bytes: 257}\n",
            ),
            "routes[0].limits",
        ),
        (
            "limits: {request_bytes: 64}\n".to_owned()
                + "routes:\n  - command: catalog.orders.PlaceOrder\n    method: post\n    path: /api/orders\n    limits: {request_bytes: 256}\n  - command: catalog.orders.NoteOrder\n    method: post\n    path: /api/orders/note\n    limits: {request_bytes: 8}\n",
            "routes[1].limits",
        ),
    ] {
        let located = located(refining(&changed));
        assert!(
            located.contains(&refused("ESS-HTTP-012", path)),
            "{changed}: {located:?}"
        );
    }
}

#[test]
fn compiled_ir_with_limits_out_of_rules_is_refused() {
    let ir = model();
    let read = |document: &str| {
        let text = compiled(&ir, document)
            .unwrap_or_else(|errors| panic!("compiles: {errors}"))
            .to_canonical_json();
        serde_json::from_str::<Value>(&text).expect("JSON")
    };
    let refuse = |value: &Value, named: &str| {
        let text = serde_json::to_string_pretty(value).expect("JSON");
        let error = HttpIr::from_json(&text).expect_err(&text).to_string();
        assert!(error.contains(named), "{named}: {error}");
    };
    let with_limits = read(&format!(
        "limits: {{request_bytes: 64}}\n{}",
        two_commands("    limits: {request_bytes: 256}\n")
    ));
    HttpIr::from_json(&serde_json::to_string_pretty(&with_limits).expect("JSON"))
        .unwrap_or_else(|error| panic!("reads: {error}"));
    for bad in [0_u64, 16_777_217] {
        let mut value = with_limits.clone();
        value["limits"]["request_bytes"] = json!(bad);
        refuse(&value, "limits.request_bytes");
        let mut value = with_limits.clone();
        value["routes"][0]["limits"]["request_bytes"] = json!(bad);
        refuse(&value, "routes[0].limits.request_bytes");
    }
    let mut value = with_limits.clone();
    value["limits"] = json!({});
    refuse(&value, "request_bytes");
    let mut value = with_limits.clone();
    value["limits"]["request_bytes"] = json!(64.0);
    refuse(&value, "");
    for document in [view_route("OrderById", ""), note_route("get", "")] {
        let mut value = read(&document);
        value["routes"][0]["limits"] = json!({"request_bytes": 64});
        refuse(&value, "limits");
    }
    let mut value = with_limits.clone();
    value["refusals"] = json!({"too_large": {"status": 500, "body": "none"}});
    refuse(&value, "4xx");
}

// --- a wrapped view answer (beyond10x/ess#755) --------------------------------------------------

/// `OrdersOfSize` on `get /orders/size/{quantity}` with `keys` beside its path.
fn sized_route(keys: &str) -> String {
    view_route("OrdersOfSize", keys).replace(
        "/orders/{order_id}\n    parameters: {order_id: param.order_id}",
        "/orders/size/{quantity}\n    parameters: {quantity: param.quantity}",
    )
}

#[test]
fn a_view_route_answers_under_one_named_member() {
    let ir = model();
    let single = compiled(
        &ir,
        &view_route(
            "OrderById",
            "    body: {order: single}\n    empty: {status: 404}\n",
        ),
    )
    .unwrap_or_else(|errors| panic!("compiles: {errors}"));
    assert_eq!(
        ir_json(&single)["routes"][0]["body"],
        json!({"order": "single"})
    );
    assert_eq!(
        ir_json(&single)["routes"][0]["empty"],
        json!({"status": 404})
    );
    round_trips(&single);
    let array = compiled(&ir, &sized_route("    body: {items: array}\n"))
        .unwrap_or_else(|errors| panic!("compiles: {errors}"));
    assert_eq!(
        ir_json(&array)["routes"][0]["body"],
        json!({"items": "array"})
    );
    round_trips(&array);
    // `{rows: array}` on a view that is not paged is `rows`, ESS's own answer: the same bytes.
    let rows = compiled(&ir, &sized_route("    body: {rows: array}\n"))
        .unwrap_or_else(|errors| panic!("compiles: {errors}"));
    let plain = compiled(&ir, &sized_route("    body: rows\n"))
        .unwrap_or_else(|errors| panic!("compiles: {errors}"));
    assert_eq!(rows.to_canonical_json(), plain.to_canonical_json());
}

#[test]
fn a_wrapped_view_answer_is_refused_where_its_bare_form_is() {
    let ir = model();
    // A paged view's answer carries its page.
    assert_eq!(
        located(compiled(
            &ir,
            &view_route("OrderPage", "    body: {items: array}\n")
        )),
        [refused("ESS-HTTP-024", "routes[0].body")]
    );
    // `single` needs `empty`, and `empty` answers only a `single` view.
    assert_eq!(
        located(compiled(
            &ir,
            &view_route("OrderById", "    body: {order: single}\n")
        )),
        located(compiled(
            &ir,
            &view_route("OrderById", "    body: single\n")
        ))
    );
    assert_eq!(
        located(compiled(
            &ir,
            &view_route(
                "OrderById",
                "    body: {items: array}\n    empty: {status: 404}\n"
            )
        )),
        [refused("ESS-HTTP-030", "routes[0]")]
    );
    // `single` on a view not selected by its identity.
    assert_eq!(
        located(compiled(
            &ir,
            &sized_route("    body: {order: single}\n    empty: {status: 404}\n")
        )),
        located(compiled(
            &ir,
            &sized_route("    body: single\n    empty: {status: 404}\n")
        ))
    );
    assert!(!located(compiled(
        &ir,
        &sized_route("    body: single\n    empty: {status: 404}\n")
    ))
    .is_empty());
    // An empty member, a form a view does not answer, and `rows` under a member.
    for body in [
        "{'': array}",
        "{items: response}",
        "{items: rows}",
        "{items: envelope}",
    ] {
        assert_eq!(
            located(compiled(&ir, &sized_route(&format!("    body: {body}\n")))),
            [refused("ESS-HTTP-024", "routes[0].body")],
            "{body}"
        );
    }
}

#[test]
fn compiled_ir_with_a_wrapped_view_answer_out_of_rules_is_refused() {
    let ir = model();
    let single = compiled(
        &ir,
        &view_route(
            "OrderById",
            "    body: {order: single}\n    empty: {status: 404}\n",
        ),
    )
    .expect("compiles")
    .to_canonical_json();
    let written = "\"body\": {\n        \"order\": \"single\"\n      }";
    assert!(single.contains(written), "{single}");
    for (broken, named) in [
        ("\"body\": {\"order\": \"array\"}", "empty"),
        ("\"body\": {\"\": \"single\"}", "member"),
        ("\"body\": {\"rows\": \"array\"}", "rows"),
    ] {
        let text = single.replace(written, broken);
        assert_ne!(text, single);
        let error = HttpIr::from_json(&text).expect_err(broken).to_string();
        assert!(error.contains(named), "{broken}: {error}");
    }
}
