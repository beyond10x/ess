//! An `ess-http/1` route declares the status and body of each outcome, a view route `rows` or
//! `single` with `empty`, and top-level `refusals:` overrides the standard refusals
//! (`docs/design/http-presentation-binding.md`, "Response body per outcome", "Status per outcome",
//! "Standard refusals" and "Stage 2: keys, precedence and the units that build them").

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

/// The routes ESS derives for `ir`, written out the way `ess_gen::http::routes` derives them for
/// this fixture, as `tests/compile.rs` does.
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

fn ir_json(http: &HttpIr) -> Value {
    serde_json::from_str(&http.to_canonical_json()).expect("canonical JSON")
}

/// A `PlaceOrder` route on `post /api/orders` listing `outcomes`.
fn place_order(outcomes: &str) -> String {
    format!(
        "routes:\n  - command: catalog.orders.PlaceOrder\n    method: post\n    path: /api/orders\n    outcomes:\n{outcomes}"
    )
}

/// A view route on `get /orders/{order_id}` with `keys` beside its path.
fn view_route(view: &str, keys: &str) -> String {
    format!(
        "routes:\n  - view: catalog.orders.{view}\n    method: get\n    path: /orders/{{order_id}}\n    parameters: {{order_id: param.order_id}}\n{keys}"
    )
}

/// The design page's example: `placed` answers `200` with the bare response, `out-of-stock` `400`
/// with `{"error": "out_of_stock", …}`; a view answers its single row, `404` when it holds none;
/// and a malformed request is answered `400` with a literal body.
const EXAMPLE: &str = "refusals:
  malformed: {status: 400, body: {error: {literal: invalid_request}}}
routes:
  - command: catalog.orders.PlaceOrder
    method: post
    path: /api/orders
    outcomes:
      placed:
        status: 200
        body: response
      out-of-stock:
        status: 400
        body: {error: error}
  - view: catalog.orders.OrderById
    method: get
    path: /orders/{order_id}
    parameters: {order_id: param.order_id}
    body: single
    empty: {status: 404}
";

#[test]
fn an_outcome_status_and_body_compile_into_the_route() {
    let ir = model();
    let http = compiled(&ir, EXAMPLE).expect("compiles");
    let json = ir_json(&http);
    let place = json["routes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|route| route["command"] == "catalog.orders.PlaceOrder")
        .expect("PlaceOrder is routed");
    // `too-many`, `mislabelled` are not listed and are absent: they keep `outcome_status` and the
    // envelope.
    assert_eq!(
        place["outcomes"],
        json!({
            "out-of-stock": {"status": 400, "body": {"error": "error"}},
            "placed": {"status": 200, "body": "response"},
        }),
        "{json:#}"
    );
    let view = json["routes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|route| route["view"] == "catalog.orders.OrderById")
        .expect("OrderById is routed");
    assert_eq!(view["body"], json!("single"), "{json:#}");
    assert_eq!(view["empty"], json!({"status": 404}), "{json:#}");
    assert_eq!(
        json["refusals"],
        json!({"malformed": {"status": 400, "body": {"error": {"literal": "invalid_request"}}}}),
        "{json:#}"
    );
    // The canonical bytes read back unchanged.
    let text = http.to_canonical_json();
    let read = HttpIr::from_json(&text).expect("the compiled IR reads back");
    assert_eq!(read.to_canonical_json(), text, "round trip changed bytes");
}

#[test]
fn a_default_written_out_compiles_to_the_bytes_without_it() {
    let ir = model();
    let bare = compiled(
        &ir,
        &place_order("      placed: {status: 200}\n      out-of-stock: {status: 400}\n"),
    )
    .expect("compiles")
    .to_canonical_json();
    let written = compiled(
        &ir,
        &place_order(
            "      placed: {status: 200, body: envelope}\n      out-of-stock: {status: 400, body: envelope}\n",
        ),
    )
    .expect("compiles")
    .to_canonical_json();
    assert_eq!(written, bare);
    assert!(!bare.contains("envelope"), "{bare}");
    let rows = compiled(&ir, &view_route("OrderById", "    body: rows\n"))
        .expect("compiles")
        .to_canonical_json();
    let none = compiled(&ir, &view_route("OrderById", ""))
        .expect("compiles")
        .to_canonical_json();
    assert_eq!(rows, none);
    assert!(!rows.contains("rows"), "{rows}");
    // An empty `outcomes` on a command and an empty `refusals` write nothing.
    let empty = compiled(
        &ir,
        "refusals: {}\nroutes:\n  - command: catalog.orders.PlaceOrder\n    method: post\n    path: /api/orders\n    outcomes: {}\n",
    )
    .expect("compiles")
    .to_canonical_json();
    let without = compiled(
        &ir,
        "routes:\n  - command: catalog.orders.PlaceOrder\n    method: post\n    path: /api/orders\n",
    )
    .expect("compiles")
    .to_canonical_json();
    assert_eq!(empty, without);
}

#[test]
fn a_missing_status_or_body_is_a_parse_refusal_naming_the_key() {
    let ir = model();
    for (document, key) in [
        (place_order("      placed: {body: response}\n"), "status"),
        (
            "refusals:\n  malformed: {body: none}\nroutes: []\n".to_owned(),
            "status",
        ),
        (
            "refusals:\n  malformed: {status: 400}\nroutes: []\n".to_owned(),
            "body",
        ),
        (
            view_route("OrderById", "    body: single\n    empty: {}\n"),
            "status",
        ),
    ] {
        let error = HttpSpec::from_yaml(&(header(&ir) + &document))
            .expect_err(key)
            .to_string();
        assert!(error.contains(&format!("missing field `{key}`")), "{error}");
    }
    for (document, key) in [
        (
            place_order("      placed: {status: 200, code: 1}\n"),
            "code",
        ),
        (
            view_route(
                "OrderById",
                "    body: single\n    empty: {status: 404, body: none}\n",
            ),
            "body",
        ),
        (
            "refusals:\n  malformed: {status: 400, body: none, retry: true}\nroutes: []\n"
                .to_owned(),
            "retry",
        ),
    ] {
        let error = HttpSpec::from_yaml(&(header(&ir) + &document))
            .expect_err(key)
            .to_string();
        assert!(error.contains(&format!("unknown field `{key}`")), "{error}");
    }
}

#[test]
fn ess_http_022_an_unknown_outcome_or_outcomes_on_a_view_is_refused() {
    let ir = model();
    let document = "routes:
  - command: catalog.orders.PlaceOrder
    method: post
    path: /api/orders
    outcomes:
      placed: {status: 200}
      shipped: {status: 200}
  - view: catalog.orders.OrderById
    method: get
    path: /orders/{order_id}
    parameters: {order_id: param.order_id}
    outcomes:
      found: {status: 200}
";
    let result = compiled(&ir, document);
    assert_eq!(
        located(result.clone()),
        [
            refused("ESS-HTTP-022", "routes[0].outcomes.shipped"),
            refused("ESS-HTTP-022", "routes[1].outcomes"),
        ]
    );
    let shown = worded(result);
    assert!(
        shown[0].contains("`shipped`") && shown[0].contains("command `catalog.orders.PlaceOrder`"),
        "{shown:?}"
    );
    assert!(
        shown[0].contains("`out-of-stock`") && shown[0].contains("`placed`"),
        "names the outcomes there are: {shown:?}"
    );
    assert!(
        shown[1].contains("view `catalog.orders.OrderById`"),
        "{shown:?}"
    );
    // Even an empty `outcomes` on a view is refused: a view has no outcome to list.
    assert_eq!(
        located(compiled(
            &ir,
            &view_route("OrderById", "    outcomes: {}\n")
        )),
        [refused("ESS-HTTP-022", "routes[0].outcomes")]
    );
}

#[test]
fn ess_http_023_a_status_out_of_range_or_in_a_forbidden_class_is_refused() {
    let ir = model();
    let at = |outcome: &str| format!("routes[0].outcomes.{outcome}.status");
    for (outcome, status, named) in [
        ("placed", 99, "100"),
        ("placed", 600, "599"),
        ("placed", 101, "1xx"),
        ("out-of-stock", 100, "1xx"),
        ("out-of-stock", 201, "2xx"),
        ("out-of-stock", 204, "2xx"),
        ("placed", 404, "4xx"),
        ("placed", 503, "5xx"),
        ("placed", 300, "300"),
        ("out-of-stock", 304, "304"),
        ("placed", 305, "305"),
        ("placed", 306, "306"),
    ] {
        let body = if status == 204 { ", body: none" } else { "" };
        let document = place_order(&format!("      {outcome}: {{status: {status}{body}}}\n"));
        let result = compiled(&ir, &document);
        assert_eq!(
            located(result.clone()),
            [refused("ESS-HTTP-023", &at(outcome))],
            "{outcome} {status}"
        );
        let shown = worded(result);
        assert!(
            shown[0].contains(&format!("`{status}`"))
                && shown[0].contains(&format!("`{outcome}`"))
                && shown[0].contains(named),
            "{outcome} {status}: {shown:?}"
        );
    }
    // `3xx` is admitted for both classes, on the five redirect statuses: `ESS-HTTP-023` passes
    // them, and only the `redirect` each requires (`ESS-HTTP-028`, `tests/headers.rs`) is missing.
    for status in [301, 302, 303, 307, 308] {
        for outcome in ["placed", "out-of-stock"] {
            let document = place_order(&format!("      {outcome}: {{status: {status}}}\n"));
            assert_eq!(
                located(compiled(&ir, &document)),
                [refused(
                    "ESS-HTTP-028",
                    &format!("routes[0].outcomes.{outcome}.redirect")
                )],
                "{outcome} {status}"
            );
        }
    }
    // A success on `2xx`, an error on `4xx` or `5xx`.
    let document = place_order(
        "      placed: {status: 201}\n      out-of-stock: {status: 409}\n      too-many: {status: 503}\n",
    );
    assert_eq!(located(compiled(&ir, &document)), []);
}

#[test]
fn ess_http_024_a_body_form_the_outcome_does_not_admit_is_refused() {
    let ir = model();
    let body_at = |outcome: &str| format!("routes[0].outcomes.{outcome}.body");
    for (outcome, written, named) in [
        // `response` without `returns: true`, or on an outcome with an error.
        ("out-of-stock", "{status: 400, body: response}", "error"),
        // `{…: error}` on an outcome without an error.
        ("placed", "{status: 200, body: {error: error}}", "no error"),
        // A literal body belongs to a standard refusal.
        (
            "out-of-stock",
            "{status: 400, body: {error: {literal: nope}}}",
            "literal",
        ),
        // A view's bodies.
        ("placed", "{status: 200, body: single}", "view"),
        ("placed", "{status: 200, body: rows}", "view"),
        // A body on `204`, the envelope by default.
        ("placed", "{status: 204}", "204"),
        ("placed", "{status: 204, body: response}", "204"),
        // A member equal to the wire name of one of the error's fields.
        (
            "mislabelled",
            "{status: 400, body: {error: error}}",
            "`kind`",
        ),
        // An empty member.
        ("out-of-stock", "{status: 400, body: {'': error}}", "empty"),
        // A word that is no body.
        ("placed", "{status: 200, body: bare}", "`bare`"),
    ] {
        let document = place_order(&format!("      {outcome}: {written}\n"));
        let result = compiled(&ir, &document);
        assert_eq!(
            located(result.clone()),
            [refused("ESS-HTTP-024", &body_at(outcome))],
            "{outcome}: {written}"
        );
        let shown = worded(result);
        assert!(shown[0].contains(named), "{written}: {shown:?}");
    }
    let document = "routes:
  - command: catalog.orders.NoteOrder
    method: post
    outcomes:
      noted: {status: 200, body: response}
";
    let result = compiled(&ir, document);
    assert_eq!(
        located(result.clone()),
        [refused("ESS-HTTP-024", "routes[0].outcomes.noted.body")]
    );
    assert!(worded(result)[0].contains("returns: true"));
    // `single` or `rows` on a command route.
    for body in ["single", "rows"] {
        let document = format!(
            "routes:\n  - command: catalog.orders.NoteOrder\n    method: post\n    body: {body}\n"
        );
        assert_eq!(
            located(compiled(&ir, &document)),
            [refused("ESS-HTTP-024", "routes[0].body")],
            "{body}"
        );
    }
    // An outcome form on a view route.
    for body in ["envelope", "response", "none", "{error: error}"] {
        let result = compiled(
            &ir,
            &view_route("OrderById", &format!("    body: {body}\n")),
        );
        assert_eq!(
            located(result.clone()),
            [refused("ESS-HTTP-024", "routes[0].body")],
            "{body}"
        );
        assert!(worded(result)[0].contains("view `catalog.orders.OrderById`"));
    }
    // `single` on a paged view.
    let document = "routes:
  - view: catalog.orders.OrderPage
    method: get
    body: single
    empty: {status: 404}
";
    let result = compiled(&ir, document);
    assert_eq!(
        located(result.clone()),
        [refused("ESS-HTTP-024", "routes[0].body")]
    );
    assert!(worded(result)[0].contains("paged"));
    // Admitted: `none` anywhere and with `204`, `response` on a returning success, and the error
    // object on an error whose fields do not travel under the member.
    let document = place_order(
        "      placed: {status: 204, body: none}\n      out-of-stock: {status: 400, body: {code: error}}\n      mislabelled: {status: 422, body: {code: error}}\n      too-many: {status: 429, body: none}\n",
    );
    assert_eq!(located(compiled(&ir, &document)), []);
    let document = place_order("      placed: {status: 201, body: response}\n");
    assert_eq!(located(compiled(&ir, &document)), []);
}

#[test]
fn ess_http_025_two_outcomes_on_one_status_the_bodies_cannot_tell_apart_are_refused() {
    let ir = model();
    for (outcomes, earlier, later, status) in [
        // The error object beside no body.
        (
            "      out-of-stock: {status: 400, body: {error: error}}\n      too-many: {status: 400, body: none}\n",
            "out-of-stock",
            "too-many",
            400,
        ),
        // The error object under two members.
        (
            "      out-of-stock: {status: 400, body: {error: error}}\n      too-many: {status: 400, body: {code: error}}\n",
            "out-of-stock",
            "too-many",
            400,
        ),
        // The envelope beside the error object.
        (
            "      out-of-stock: {status: 409}\n      too-many: {status: 409, body: {error: error}}\n",
            "out-of-stock",
            "too-many",
            409,
        ),
        // Two empty bodies.
        (
            "      out-of-stock: {status: 409, body: none}\n      mislabelled: {status: 409, body: none}\n",
            "out-of-stock",
            "mislabelled",
            409,
        ),
    ] {
        let result = compiled(&ir, &place_order(outcomes));
        assert_eq!(
            located(result.clone()),
            [refused(
                "ESS-HTTP-025",
                &format!("routes[0].outcomes.{earlier}")
            )],
            "{outcomes}"
        );
        let shown = worded(result);
        assert!(
            shown[0].contains(&format!("`{earlier}`"))
                && shown[0].contains(&format!("`{later}`"))
                && shown[0].contains(&format!("`{status}`")),
            "{shown:?}"
        );
    }
    // An unlisted outcome is counted at its derived status with the envelope: `too-many` is a
    // `when:` refusal, answered `422`.
    let result = compiled(
        &ir,
        &place_order("      out-of-stock: {status: 422, body: {error: error}}\n"),
    );
    assert_eq!(
        located(result.clone()),
        [refused("ESS-HTTP-025", "routes[0].outcomes.out-of-stock")]
    );
    let shown = worded(result);
    assert!(
        shown[0].contains("`too-many`") && shown[0].contains("`422`"),
        "{shown:?}"
    );
    // Admitted: two envelopes, and two error objects on one member with distinct wire names.
    for outcomes in [
        "      out-of-stock: {status: 400}\n      too-many: {status: 400}\n",
        "      out-of-stock: {status: 400, body: {error: error}}\n      too-many: {status: 400, body: {error: error}}\n",
        // The listed envelope beside the unlisted ones on `422`.
        "      out-of-stock: {status: 422}\n",
    ] {
        assert_eq!(
            located(compiled(&ir, &place_order(outcomes))),
            [],
            "{outcomes}"
        );
    }
}

/// Telling two outcomes apart is not transitive: two envelopes on `409` are apart, and an error
/// object on `409` is apart from neither. Each pair no body tells apart is named once, so the
/// error object's clash with the second envelope is not lost when the first envelope's refusal has
/// already named it.
#[test]
fn ess_http_025_names_every_pair_no_body_tells_apart_once() {
    let ir = model();
    let result = compiled(
        &ir,
        &place_order(
            "      out-of-stock: {status: 409}\n      too-many: {status: 409, body: {error: error}}\n      mislabelled: {status: 409}\n",
        ),
    );
    assert_eq!(
        located(result.clone()),
        [
            refused("ESS-HTTP-025", "routes[0].outcomes.out-of-stock"),
            refused("ESS-HTTP-025", "routes[0].outcomes.too-many"),
        ]
    );
    let shown = worded(result);
    assert!(
        shown[0].contains("`too-many`") && !shown[0].contains("`mislabelled`"),
        "{shown:?}"
    );
    assert!(
        shown[1].contains("`mislabelled`") && !shown[1].contains("`out-of-stock` ("),
        "{shown:?}"
    );
}

/// `ESS-HTTP-025` holds a `refusals:` override to the same rule: on a route the binding names, an
/// override on the status of an outcome of the command, with a body a reader cannot tell apart from
/// that outcome's, is refused at `refusals.<key>`, naming the route, the outcomes and the status.
/// Only `{<member>: error}` beside `{<member>: {literal: <text>}}` on one member, the error's wire
/// name differing from the text, tells the two apart.
#[test]
fn ess_http_025_a_refusal_override_no_body_tells_apart_from_an_outcome_is_refused() {
    let ir = model();
    for (document, outcome, status) in [
        // Two empty bodies.
        (
            format!(
                "refusals:\n  malformed: {{status: 400, body: none}}\n{}",
                place_order("      out-of-stock: {status: 400, body: none}\n")
            ),
            "out-of-stock",
            400,
        ),
        // The literal text is the error's wire name, on the same member.
        (
            format!(
                "refusals:\n  malformed: {{status: 400, body: {{error: {{literal: out_of_stock}}}}}}\n{}",
                place_order("      out-of-stock: {status: 400, body: {error: error}}\n")
            ),
            "out-of-stock",
            400,
        ),
        // The literal under another member than the error object's.
        (
            format!(
                "refusals:\n  malformed: {{status: 400, body: {{code: {{literal: invalid_request}}}}}}\n{}",
                place_order("      out-of-stock: {status: 400, body: {error: error}}\n")
            ),
            "out-of-stock",
            400,
        ),
        // An outcome the route does not list, counted at its derived status with `envelope`:
        // `too-many` is a `when:` refusal, answered `422`.
        (
            "refusals:\n  malformed: {status: 422, body: {error: {literal: invalid_request}}}\nroutes:\n  - command: catalog.orders.PlaceOrder\n    method: post\n    path: /api/orders\n".to_owned(),
            "too-many",
            422,
        ),
    ] {
        let result = compiled(&ir, &document);
        assert_eq!(
            located(result.clone()),
            [refused("ESS-HTTP-025", "refusals.malformed")],
            "{document}"
        );
        let shown = worded(result);
        assert!(
            shown[0].contains("`malformed`")
                && shown[0].contains(&format!("`{outcome}`"))
                && shown[0].contains(&format!("`{status}`"))
                && shown[0].contains("routes[0]"),
            "{shown:?}"
        );
    }
    // Admitted: a literal and an error object on one member with distinct texts, an override on a
    // status no outcome of a bound command answers, and an override beside view routes only, whose
    // `empty` status is no outcome.
    for document in [
        format!(
            "refusals:\n  malformed: {{status: 400, body: {{error: {{literal: invalid_request}}}}}}\n{}",
            place_order("      out-of-stock: {status: 400, body: {error: error}}\n")
        ),
        format!(
            "refusals:\n  malformed: {{status: 400, body: none}}\n{}",
            place_order("      out-of-stock: {status: 409, body: none}\n")
        ),
        format!(
            "refusals:\n  not_granted: {{status: 404, body: none}}\n{}",
            view_route("OrderById", "    body: single\n    empty: {status: 404}\n")
        ),
    ] {
        assert_eq!(located(compiled(&ir, &document)), [], "{document}");
    }
}

#[test]
fn ess_http_029_a_refusal_key_outside_the_set_or_in_the_wrong_class_is_refused() {
    let ir = model();
    let document = "refusals:
  forbidden: {status: 403, body: none}
  malformed: {status: 500, body: none}
  not_granted: {status: 200, body: none}
  unfinished: {status: 400, body: {error: {literal: unfinished}}}
routes: []
";
    let result = compiled(&ir, document);
    assert_eq!(
        located(result.clone()),
        [
            refused("ESS-HTTP-029", "refusals.forbidden"),
            refused("ESS-HTTP-029", "refusals.malformed.status"),
            refused("ESS-HTTP-029", "refusals.not_granted.status"),
            refused("ESS-HTTP-029", "refusals.unfinished.status"),
        ]
    );
    let shown = worded(result);
    assert!(
        shown[0].contains("`forbidden`")
            && shown[0].contains("`malformed`")
            && shown[0].contains("`not_granted`")
            && shown[0].contains("`unfinished`"),
        "{shown:?}"
    );
    assert!(
        shown[1].contains("`500`") && shown[1].contains("4xx"),
        "{shown:?}"
    );
    assert!(
        shown[3].contains("`400`") && shown[3].contains("5xx"),
        "{shown:?}"
    );
    for body in [
        "envelope",
        "response",
        "rows",
        "single",
        "{error: error}",
        "bare",
        "{'': {literal: x}}",
    ] {
        let document =
            format!("refusals:\n  malformed: {{status: 400, body: {body}}}\nroutes: []\n");
        assert_eq!(
            located(compiled(&ir, &document)),
            [refused("ESS-HTTP-029", "refusals.malformed.body")],
            "{body}"
        );
    }
    let document = "refusals:
  malformed: {status: 422, body: {error: {literal: invalid_request}}}
  not_granted: {status: 403, body: none}
  unfinished: {status: 503, body: {error: {literal: temporarily_unavailable}}}
routes: []
";
    assert_eq!(located(compiled(&ir, document)), []);
}

#[test]
fn ess_http_030_single_without_empty_or_empty_without_single_is_refused() {
    let ir = model();
    for keys in [
        "    body: single\n",
        "    empty: {status: 404}\n",
        "    body: rows\n    empty: {status: 404}\n",
        "    body: single\n    empty: {status: 500}\n",
        "    body: single\n    empty: {status: 204}\n",
    ] {
        assert_eq!(
            located(compiled(&ir, &view_route("OrderById", keys))),
            [refused("ESS-HTTP-030", "routes[0]")],
            "{keys}"
        );
    }
    let result = compiled(&ir, &view_route("OrderById", "    body: single\n"));
    assert!(worded(result)[0].contains("`empty`"));
    // `empty` on a command route is `empty` without `single`.
    let document = "routes:
  - command: catalog.orders.NoteOrder
    method: post
    empty: {status: 404}
";
    assert_eq!(
        located(compiled(&ir, document)),
        [refused("ESS-HTTP-030", "routes[0]")]
    );
    assert_eq!(
        located(compiled(
            &ir,
            &view_route("OrderById", "    body: single\n    empty: {status: 410}\n")
        )),
        []
    );
}

#[test]
fn ess_http_031_single_on_a_view_not_selected_by_identity_is_refused() {
    let ir = model();
    for view in ["OrdersOfSize", "AnyOrderById"] {
        let document = format!(
            "routes:\n  - view: catalog.orders.{view}\n    method: get\n    body: single\n    empty: {{status: 404}}\n"
        );
        let result = compiled(&ir, &document);
        assert_eq!(
            located(result.clone()),
            [refused("ESS-HTTP-031", "routes[0].body")],
            "{view}"
        );
        let shown = worded(result);
        assert!(
            shown[0].contains(&format!("view `catalog.orders.{view}`"))
                && shown[0].contains("`catalog.orders.Order`")
                && shown[0].contains("`order_id`"),
            "{shown:?}"
        );
    }
    // The identity compared with a parameter, in either operand order, alone or in a top-level
    // `all`, nested `all` included.
    for view in ["OrderById", "OrderByIdReversed", "LargeOrderById"] {
        let document = view_route(view, "    body: single\n    empty: {status: 404}\n");
        assert_eq!(located(compiled(&ir, &document)), [], "{view}");
    }
    // Without `empty` and without the identity, both refusals are reported.
    let document = "routes:
  - view: catalog.orders.OrdersOfSize
    method: get
    body: single
";
    assert_eq!(
        located(compiled(&ir, document)),
        [
            refused("ESS-HTTP-030", "routes[0]"),
            refused("ESS-HTTP-031", "routes[0].body"),
        ]
    );
}

/// A protocol binding fixing `PlaceOrder`'s answers, the single `OrderById` and a refusal.
const PROTOCOL: &str = "refusals:
  malformed: {status: 400, body: {error: {literal: invalid_request}}}
routes:
  - command: catalog.orders.PlaceOrder
    method: post
    path: /api/orders
    outcomes:
      placed: {status: 200, body: response}
      out-of-stock: {status: 400, body: {error: error}}
  - view: catalog.orders.OrderById
    method: get
    path: /orders/{order_id}
    parameters: {order_id: param.order_id}
    body: single
    empty: {status: 404}
  - view: catalog.orders.OrdersOfSize
    method: get
";

fn refining(ir: &EssIr, refined: &Refined, document: &str) -> Result<HttpIr, HttpDiagnostics> {
    let text = header(ir) + &format!("refines: {}\n", refined.digest()) + document;
    let spec = HttpSpec::from_yaml(&text).expect("parses");
    compile_refining(&spec, Some(refined), ir, &derived(ir))
}

#[test]
fn ess_http_012_a_refinement_restates_every_answer_of_the_refined_binding() {
    let ir = model();
    let refined = Refined::from_yaml((header(&ir) + PROTOCOL).as_bytes()).expect("parses");
    // Restated, with a default written out where the refined binding left it, compiles.
    let restated = PROTOCOL.replace(
        "  - view: catalog.orders.OrdersOfSize\n    method: get\n",
        "  - view: catalog.orders.OrdersOfSize\n    method: get\n    body: rows\n",
    );
    assert_ne!(restated, PROTOCOL);
    assert_eq!(located(refining(&ir, &refined, &restated)), []);
    // `415`, not `422`: `422` is the derived status of the unlisted `when:` refusals, which the
    // literal would collide with (`ESS-HTTP-025`).
    let changed = "refusals:
  malformed: {status: 415, body: {error: {literal: invalid_request}}}
  not_granted: {status: 403, body: none}
routes:
  - command: catalog.orders.PlaceOrder
    method: post
    path: /api/orders
    outcomes:
      placed: {status: 201, body: response}
      too-many: {status: 400}
  - view: catalog.orders.OrderById
    method: get
    path: /orders/{order_id}
    parameters: {order_id: param.order_id}
    body: single
    empty: {status: 410}
  - view: catalog.orders.OrdersOfSize
    method: get
";
    let result = refining(&ir, &refined, changed);
    assert_eq!(
        located(result.clone()),
        [
            refused("ESS-HTTP-012", "refusals.malformed"),
            refused("ESS-HTTP-012", "refusals.not_granted"),
            refused("ESS-HTTP-012", "routes[0].outcomes.out-of-stock"),
            refused("ESS-HTTP-012", "routes[0].outcomes.placed"),
            refused("ESS-HTTP-012", "routes[0].outcomes.too-many"),
            refused("ESS-HTTP-012", "routes[1].empty"),
        ]
    );
    let shown = worded(result);
    assert!(
        shown[0].contains("400") && shown[0].contains("415"),
        "{shown:?}"
    );
    assert!(
        shown[3].contains("`200`") && shown[3].contains("`201`"),
        "{shown:?}"
    );
    assert!(
        shown[5].contains("404") && shown[5].contains("410"),
        "{shown:?}"
    );
    // Dropping the view's `single` is a change too, as is a body on a view that had none.
    let dropped = PROTOCOL
        .replace("    body: single\n    empty: {status: 404}\n", "")
        .replace(
            "  - view: catalog.orders.OrdersOfSize\n    method: get\n",
            "  - view: catalog.orders.OrdersOfSize\n    method: get\n    body: single\n    empty: {status: 404}\n",
        );
    let result = refining(&ir, &refined, &dropped);
    let found = located(result);
    for expected in [
        refused("ESS-HTTP-012", "routes[1].body"),
        refused("ESS-HTTP-012", "routes[1].empty"),
        refused("ESS-HTTP-012", "routes[2].body"),
        refused("ESS-HTTP-012", "routes[2].empty"),
    ] {
        assert!(found.contains(&expected), "{expected:?} in {found:?}");
    }
    // Dropping `refusals:` altogether is a change.
    let unrefused = PROTOCOL.replace(
        "refusals:\n  malformed: {status: 400, body: {error: {literal: invalid_request}}}\n",
        "",
    );
    assert_eq!(
        located(refining(&ir, &refined, &unrefused)),
        [refused("ESS-HTTP-012", "refusals.malformed")]
    );
}

#[test]
fn ess_http_012_statuses_and_bodies_compare_as_values_not_as_text() {
    let ir = model();
    // A refusal's literal body whose member and text spell another member and text is not that
    // body: both compile on their own, and neither is a restatement of the other.
    let literal =
        |body: &str| format!("refusals:\n  malformed: {{status: 400, body: {body}}}\nroutes: []\n");
    let one = literal("{'m: {literal: t}}, {n': {literal: u}}");
    let refined_one = Refined::from_yaml((header(&ir) + &one).as_bytes()).expect("parses");
    assert_eq!(located(compiled(&ir, &one)), []);
    let two = literal("{m: {literal: 't}}, {n: {literal: u'}}");
    assert_eq!(located(compiled(&ir, &two)), []);
    assert_eq!(
        located(refining(&ir, &refined_one, &two)),
        [refused("ESS-HTTP-012", "refusals.malformed")]
    );
    // An outcome's body written as a word that spells `{error: error}` is not `{error: error}`:
    // `ESS-HTTP-024` refuses the word, and the refinement does not restate the refined body.
    let refined = Refined::from_yaml((header(&ir) + PROTOCOL).as_bytes()).expect("parses");
    let worded_body = PROTOCOL.replace(
        "out-of-stock: {status: 400, body: {error: error}}",
        "out-of-stock: {status: 400, body: '{error: error}'}",
    );
    assert_ne!(worded_body, PROTOCOL);
    let found: Vec<(&str, String)> = located(refining(&ir, &refined, &worded_body))
        .into_iter()
        .filter(|(code, _)| *code == "ESS-HTTP-012")
        .collect();
    assert_eq!(
        found,
        [refused("ESS-HTTP-012", "routes[0].outcomes.out-of-stock")]
    );
}

#[test]
fn compiled_ir_with_an_answer_out_of_its_rules_is_refused_naming_the_key() {
    let ir = model();
    let http = compiled(&ir, EXAMPLE).expect("compiles");
    let text = http.to_canonical_json();
    let placed =
        "\"placed\": {\n          \"status\": 200,\n          \"body\": \"response\"\n        }";
    assert!(text.contains(placed), "{text}");
    for (written, named) in [
        (
            "\"placed\": {\"status\": 200, \"body\": \"envelope\"}",
            "envelope",
        ),
        ("\"placed\": {\"status\": 99}", "99"),
        ("\"placed\": {\"status\": 101}", "101"),
        ("\"placed\": {\"status\": 304}", "304"),
        (
            "\"placed\": {\"status\": 204, \"body\": \"response\"}",
            "204",
        ),
        (
            "\"placed\": {\"status\": 200, \"body\": \"single\"}",
            "single",
        ),
        (
            "\"placed\": {\"status\": 200, \"body\": {\"e\": {\"literal\": \"x\"}}}",
            "literal",
        ),
        (
            "\"placed\": {\"status\": 200, \"body\": {\"\": \"error\"}}",
            "member",
        ),
        ("\"placed\": {\"body\": \"response\"}", "status"),
        ("\"placed\": {\"status\": 200, \"code\": 1}", "code"),
    ] {
        let broken = text.replace(placed, written);
        assert_ne!(broken, text);
        let error = HttpIr::from_json(&broken).expect_err(written).to_string();
        assert!(error.contains(named), "{written}: {error}");
    }
    // Outcomes out of name order.
    let outcomes = "\"outcomes\": {\n        \"out-of-stock\"";
    assert!(text.contains(outcomes), "{text}");
    let swapped = text
        .replace(
            "\"out-of-stock\": {\n          \"status\": 400,\n          \"body\": {\n            \"error\": \"error\"\n          }\n        },\n        \"placed\": {\n          \"status\": 200,\n          \"body\": \"response\"\n        }",
            "\"placed\": {\"status\": 200, \"body\": \"response\"}, \"out-of-stock\": {\"status\": 400, \"body\": {\"error\": \"error\"}}",
        );
    assert_ne!(swapped, text);
    let error = HttpIr::from_json(&swapped).expect_err("order").to_string();
    assert!(error.contains("outcomes"), "{error}");
    // The view's answer.
    let single = "\"body\": \"single\",\n      \"empty\": {\n        \"status\": 404\n      }";
    assert!(text.contains(single), "{text}");
    for (written, named) in [
        ("\"body\": \"single\"", "empty"),
        ("\"body\": \"rows\", \"empty\": {\"status\": 404}", "rows"),
        ("\"empty\": {\"status\": 404}", "empty"),
        ("\"body\": \"single\", \"empty\": {\"status\": 500}", "500"),
        (
            "\"body\": \"single\", \"empty\": {\"status\": 404}, \"outcomes\": {\"found\": {\"status\": 200}}",
            "outcomes",
        ),
    ] {
        let broken = text.replace(single, written);
        assert_ne!(broken, text);
        let error = HttpIr::from_json(&broken).expect_err(written).to_string();
        assert!(error.contains(named), "{written}: {error}");
    }
    // The refusals.
    let malformed = "\"malformed\": {\n      \"status\": 400,";
    assert!(text.contains(malformed), "{text}");
    for (written, named) in [
        ("\"malformed\": {\n      \"status\": 500,", "500"),
        ("\"forbidden\": {\n      \"status\": 400,", "forbidden"),
    ] {
        let broken = text.replace(malformed, written);
        let error = HttpIr::from_json(&broken).expect_err(written).to_string();
        assert!(error.contains(named), "{written}: {error}");
    }
    // An `outcomes` on a command route is fine, a view `body` on one is not.
    let command = "\"command\": \"catalog.orders.PlaceOrder\",";
    assert!(text.contains(command), "{text}");
    let broken = text.replace(
        command,
        "\"command\": \"catalog.orders.PlaceOrder\", \"body\": \"single\",",
    );
    let error = HttpIr::from_json(&broken)
        .expect_err("body on a command")
        .to_string();
    assert!(error.contains("body"), "{error}");
}

/// Compiled IR omits an empty map or list, a `null` and a default; IR that writes one out reads
/// back as other bytes, so `HttpIr::from_json` refuses it, naming the key, whichever field holds it.
#[test]
fn compiled_ir_writing_out_a_key_it_omits_is_refused_naming_the_key() {
    type Mutation = fn(&mut Value);
    let ir = model();
    let http = compiled(&ir, EXAMPLE).expect("compiles");
    let canonical = ir_json(&http);
    assert_eq!(
        canonical["routes"][0]["command"],
        "catalog.orders.PlaceOrder"
    );
    let cases: [(&str, Mutation); 7] = [
        ("`outcomes` is empty", |json| {
            json["routes"][0]["outcomes"] = json!({});
        }),
        ("`refusals` is empty", |json| json["refusals"] = json!({})),
        ("`derived_routes`", |json| {
            json["derived_routes"] = json!([]);
        }),
        ("`refines`", |json| json["refines"] = Value::Null),
        ("`routes[0].request`", |json| {
            json["routes"][0]["request"] = Value::Null;
        }),
        ("`routes[0].empty`", |json| {
            json["routes"][0]["empty"] = Value::Null;
        }),
        ("`routes[1].parameters`", |json| {
            json["routes"][1]["path"] = json!("/orders");
            json["routes"][1]["parameters"] = json!({});
        }),
    ];
    for (named, mutate) in cases {
        let mut json = canonical.clone();
        mutate(&mut json);
        let text = serde_json::to_string(&json).expect("serializes");
        let error = HttpIr::from_json(&text).expect_err(named).to_string();
        assert!(error.contains(named), "{named}: {error}");
    }
    // The canonical bytes, and the same IR written compactly, read back.
    let compact = serde_json::to_string(&canonical).expect("serializes");
    let read = HttpIr::from_json(&compact).expect("reads compact IR");
    assert_eq!(read.to_canonical_json(), http.to_canonical_json());
}
