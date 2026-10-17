//! `ess generate synthesize --target rust --http`: the generated Rust server serves an `ess-http/1`
//! binding (beyond10x/ess#493, stage 2, `docs/design/http-presentation-binding.md`, "Precedence"
//! and "What the servers write (k5, k6)").
//!
//! The model and the binding are `tests/fixtures/http-binding-shop.yaml` and
//! `tests/fixtures/http-binding-shop.routes.yaml`, and the requests are those of
//! `tests/http_binding/shared.rs`, which module [`go`] (`tests/http_binding/go.rs`) sends the Go
//! server too. One harness, built once per test binary, drives the generated surface's own
//! `dispatch` with every request and prints each answer; each test reads the answers it is about.
//! Its realization supplies what the plan owes: `Place`'s behaviour (unmet for a count of `0`, so
//! the realization is unfinished there) and `ItemById`'s query over the same store the generated
//! `PutItem` writes.

#[path = "http_binding/shared.rs"]
mod shared;

#[path = "http_binding/go.rs"]
mod go;

use ess_synth::{
    synthesize_bound, synthesize_laid_out, BindingRefusal, OutputLayout, SynthesisFailure, Target,
};
use shared::{binding, ir, published, Answer, ROUTES};

fn served(label: &str) -> &'static Answer {
    shared::rust_answers()
        .get(label)
        .unwrap_or_else(|| panic!("the harness answered `{label}`"))
}

/// Without a binding every synthesized byte is the unbound synthesis's, in both layouts; with one,
/// nothing outside the server crate moves.
#[test]
fn without_a_binding_the_rust_server_keeps_its_bytes() {
    let ir = ir();
    for layout in [OutputLayout::Workspace, OutputLayout::Crate] {
        let unbound = synthesize_laid_out(&ir, Target::Rust, layout).expect("synthesizes");
        let none = synthesize_bound(&ir, Target::Rust, layout, None).expect("synthesizes");
        assert_eq!(
            unbound
                .artifacts
                .iter()
                .map(|(path, artifact)| (path, &artifact.contents))
                .collect::<Vec<_>>(),
            none.artifacts
                .iter()
                .map(|(path, artifact)| (path, &artifact.contents))
                .collect::<Vec<_>>(),
            "{layout:?}"
        );
        assert!(
            !unbound
                .artifacts
                .values()
                .any(|artifact| artifact.contents.contains("http::matched")),
            "an unbound server dispatches on exact paths ({layout:?})"
        );
    }
    let unbound =
        synthesize_laid_out(&ir, Target::Rust, OutputLayout::Workspace).expect("synthesizes");
    let bound = synthesize_bound(
        &ir,
        Target::Rust,
        OutputLayout::Workspace,
        Some(&binding(&ir, ROUTES)),
    )
    .expect("synthesizes");
    assert_eq!(
        unbound.artifacts.keys().collect::<Vec<_>>(),
        bound.artifacts.keys().collect::<Vec<_>>()
    );
    let moved: Vec<&String> = bound
        .artifacts
        .iter()
        .filter(|(path, artifact)| unbound.artifacts[*path].contents != artifact.contents)
        .map(|(path, _)| path)
        .collect();
    assert!(
        moved
            .iter()
            .all(|path| path.starts_with("crates/shop-server/src/")),
        "only the server crate moves with a binding: {moved:?}"
    );
    assert!(
        moved
            .iter()
            .any(|path| path.ends_with("shop-service.openapi.json")),
        "the served document is the bound one: {moved:?}"
    );
}

/// `PUT`, `DELETE` and the view's `GET` share `/items/{item_id}`; the variable is percent-decoded
/// into the input, a path no template matches is `404` and a method none answers there is `405`.
#[test]
fn a_bound_command_is_dispatched_by_template_and_method() {
    let put = served("put");
    assert_eq!(put.status, 202, "{}", put.body);
    assert_eq!(
        published(put),
        serde_json::json!({"item_id": "a b", "title": "Lamp"})
    );
    let get = served("get");
    assert_eq!(get.status, 200, "{}", get.body);
    assert_eq!(
        get.json(),
        serde_json::json!({"item_id": "a b", "title": "Lamp"})
    );
    let missing = served("get-missing");
    assert_eq!(
        (
            missing.status,
            missing.body.as_str(),
            missing.content_type.as_str()
        ),
        (404, "", "")
    );
    let delete = served("delete");
    assert_eq!(delete.status, 202, "{}", delete.body);
    assert_eq!(published(delete), serde_json::json!({"item_id": "a b"}));
    let post = served("post-on-items");
    assert_eq!(post.status, 405, "{}", post.body);
    assert!(post.body.contains("DELETE, GET, PUT"), "{}", post.body);
    for label in ["nowhere", "empty-variable"] {
        assert_eq!(served(label).status, 404, "{label}: {}", served(label).body);
    }
    for label in ["put-body-names-path", "put-bad-escape"] {
        let answer = served(label);
        assert_eq!(
            (answer.status, answer.json()),
            (400, serde_json::json!({"error": "invalid_request"})),
            "{label}"
        );
    }
    assert_eq!(served("docs").status, 200);
    let openapi: serde_json::Value = served("openapi").json();
    assert!(
        openapi["paths"]["/items/{item_id}"]["put"].is_object(),
        "the served document is the bound one: {}",
        openapi["paths"]
    );
}

/// A `form` request decodes into the input; `ignore` admits an unknown parameter, `refuse`
/// answers `malformed`, and so does a scalar key given twice, an empty integer, and a body of
/// another media type.
#[test]
fn a_form_request_decodes_and_an_unknown_parameter_follows_the_binding() {
    let ignored = served("form-ignore");
    assert_eq!(ignored.status, 202, "{}", ignored.body);
    assert_eq!(
        published(ignored),
        serde_json::json!({"item_id": "x 1", "count": 2})
    );
    let declared = served("form-ignore-unlisted");
    assert_eq!(declared.status, 422, "{}", declared.body);
    assert_eq!(declared.json()["outcome"], "out-of-stock");
    let strict = served("form-strict-ok");
    assert_eq!(strict.status, 202, "{}", strict.body);
    assert_eq!(
        published(strict),
        serde_json::json!({"item_id": "é", "count": 3})
    );
    for label in [
        "form-refuse",
        "form-twice",
        "form-empty-integer",
        "form-json-type",
        "form-no-type",
    ] {
        let answer = served(label);
        assert_eq!(
            (answer.status, answer.json()),
            (400, serde_json::json!({"error": "invalid_request"})),
            "{label}"
        );
        assert_eq!(answer.header("Cache-Control"), Some("no-store"), "{label}");
    }
}

/// A `query` command reads its input from the query string, a `List` repeating its key.
#[test]
fn a_query_command_reads_its_inputs_from_the_query_string() {
    let listed = served("query-list");
    assert_eq!(listed.status, 202, "{}", listed.body);
    assert_eq!(
        published(listed),
        serde_json::json!({"words": ["a", "b c"], "limit": 3})
    );
    let none = served("query-list-none");
    assert_eq!(none.status, 202, "{}", none.body);
    assert_eq!(
        published(none),
        serde_json::json!({"words": [], "limit": 1})
    );
    assert_eq!(served("query-scalar-twice").status, 400);
}

/// A listed outcome answers its declared status, body and headers; a header sourced from input
/// that is not a field value is never written, and the committed effect answers `unfinished`.
#[test]
fn a_declared_status_body_and_headers_are_written_per_outcome() {
    let placed = served("placed");
    assert_eq!(placed.status, 201, "{}", placed.body);
    assert_eq!(placed.json(), serde_json::json!({"code": "C-i1"}));
    assert_eq!(placed.header("Cache-Control"), Some("no-store"));
    assert_eq!(placed.header("X-Item"), Some("i1"));
    assert_eq!(placed.header("X-Code"), Some("C-i1"));
    let refused = served("refused");
    assert_eq!(refused.status, 409, "{}", refused.body);
    assert_eq!(refused.json(), serde_json::json!({"error": "out_of_stock"}));
    assert_eq!(refused.header("X-Error"), Some("out_of_stock"));
    let injected = served("placed-header-injection");
    assert_eq!(
        (
            injected.status,
            injected.body.as_str(),
            injected.headers.len()
        ),
        (503, "", 0),
        "a header value holding CR LF is answered as the unfinished refusal the binding declares"
    );
}

/// A `401` carries the declared challenge; each param is a quoted string whose `"` and `\` are
/// escaped, so neither the error's wire name nor an input value can leave it.
#[test]
fn a_bearer_challenge_quotes_the_error_wire_name() {
    let challenged = served("challenge");
    assert_eq!(challenged.status, 401, "{}", challenged.body);
    assert_eq!(
        challenged.header("WWW-Authenticate"),
        Some(r#"Bearer error="invalid \"token\" \\ here", hint="a\"b\\c", realm="shop""#)
    );
    assert_eq!(
        challenged.json(),
        serde_json::json!({"error": "invalid \"token\" \\ here"})
    );
    let read = served("challenge-read");
    assert_eq!(read.status, 202, "{}", read.body);
    assert_eq!(read.header("WWW-Authenticate"), None);
    let control = served("challenge-control");
    assert_eq!(
        (control.status, control.headers.len()),
        (503, 0),
        "a param a quoted string cannot carry is never written"
    );
}

/// `mode: query` appends its parameters to a query the location already has, form-encoded in name
/// order, and omits an absent optional. A location that is not an absolute URI without a fragment is never written.
#[test]
fn a_query_redirect_keeps_an_existing_query_and_omits_an_absent_optional() {
    let query = served("redirect-query");
    assert_eq!(
        (
            query.status,
            query.body.as_str(),
            query.content_type.as_str()
        ),
        (302, "", "")
    );
    assert_eq!(
        query.header("Location"),
        Some("https://client.example/cb?x=1&client=c%26d&state=s+1")
    );
    assert_eq!(
        served("redirect-query-absent").header("Location"),
        Some("https://client.example/cb?client=c")
    );
    for label in ["redirect-relative", "redirect-with-fragment"] {
        let answer = served(label);
        assert_eq!(
            (answer.status, answer.headers.len()),
            (503, 0),
            "{label}: {}",
            answer.body
        );
    }
}

/// `mode: fragment` writes the parameters as the location's fragment, form-encoded in name order,
/// and omits an absent optional, as RFC 6749 §4.1.2.1 returns `state` only when it was sent.
#[test]
fn a_fragment_redirect_writes_its_parameters_and_omits_an_absent_optional() {
    let fragment = served("redirect-fragment");
    assert_eq!(fragment.status, 303, "{}", fragment.body);
    assert_eq!(
        fragment.header("Location"),
        Some("https://client.example/cb#error=access_denied&state=s")
    );
    assert_eq!(
        served("redirect-fragment-absent").header("Location"),
        Some("https://client.example/cb#error=access_denied")
    );
}

/// A route whose template a request for `/openapi.json` or `/docs` would match is refused at
/// synthesis, naming it; `ess specify http` accepts the document, and a longer template is served.
#[test]
fn a_route_claiming_openapi_json_or_docs_is_refused_at_synthesis() {
    let ir = ir();
    for path in ["/openapi.json", "/docs", "/{page}"] {
        let routes = format!(
            "routes:\n  - command: shop.items.Purge\n    method: post\n    path: '{path}'\n{}",
            if path == "/{page}" {
                "    parameters: {page: input.item_id}\n"
            } else {
                ""
            }
        );
        let bound = binding(&ir, &routes);
        let Err(SynthesisFailure::Binding(BindingRefusal::Unservable(refused))) =
            synthesize_bound(&ir, Target::Rust, OutputLayout::Workspace, Some(&bound))
        else {
            panic!("`{path}` is refused at synthesis");
        };
        let text = refused.to_string();
        assert!(
            text.contains("command `shop.items.Purge`")
                && (text.contains("`/openapi.json`") || text.contains("`/docs`")),
            "{path}: {text}"
        );
    }
    let longer = binding(
        &ir,
        "routes:\n  - command: shop.items.Purge\n    method: post\n    path: /docs/{item}\n    \
         parameters: {item: input.item_id}\n",
    );
    assert!(
        synthesize_bound(&ir, Target::Rust, OutputLayout::Workspace, Some(&longer)).is_ok(),
        "a template of two segments matches neither"
    );
    for target in [Target::Web, Target::Clap] {
        let refused = synthesize_bound(
            &ir,
            target,
            OutputLayout::Workspace,
            Some(&binding(&ir, ROUTES)),
        )
        .err()
        .map(|failure| failure.to_string())
        .unwrap_or_default();
        assert!(
            refused.contains(&format!("the `{}` target does not serve", target.name())),
            "{refused}"
        );
    }
}

/// `refusals:` answers `not_granted`, `unfinished` and `malformed` as declared on the routes the
/// binding names; a route it does not name keeps today's answer.
#[test]
fn a_declared_refusal_override_answers_not_granted_unfinished_and_malformed() {
    for label in ["not-granted", "not-granted-anonymous"] {
        let answer = served(label);
        assert_eq!(
            (answer.status, answer.json()),
            (403, serde_json::json!({"error": "access_denied"})),
            "{label}"
        );
    }
    let derived = served("not-granted-derived");
    assert_eq!(derived.status, 403, "{}", derived.body);
    assert_eq!(
        derived.json(),
        serde_json::json!({"refused": "not granted", "actor": "shop.items.Clerk"})
    );
    let unfinished = served("unfinished");
    assert_eq!(
        (
            unfinished.status,
            unfinished.body.as_str(),
            unfinished.content_type.as_str()
        ),
        (503, "", "")
    );
    let malformed = served("malformed-json");
    assert_eq!(
        (malformed.status, malformed.json()),
        (400, serde_json::json!({"error": "invalid_request"}))
    );
    assert_eq!(malformed.header("Cache-Control"), Some("no-store"));
}
