//! An outcome that answers one response field bare (`body: response.<field>`) and a route whose
//! error outcomes share one answer (`errors:`)
//! (`story:an-outcome-may-answer-a-response-field-bare-and-errors-share-a-default-status`), sent
//! to both generated servers, and the `OpenAPI` document both embed.
//!
//! The shared routes change on two commands. `Place` answers `placed` with its response's `code`
//! alone and gives every error outcome, its one `when:` refusal `refused` among them, `422` with
//! `{error: error}` through `errors`, no longer listing `refused`. `Order` gives its one `when:`
//! refusal `409` with no body through `errors` and lists nothing.

use std::collections::BTreeMap;

use ess_synth::{synthesize_bound, OutputLayout, Synthesis, Target};

use super::super::shared::{self, Answer};
use super::adversary_k6_pass1::{go_run, ir_of, rust_run};

/// `(anchor, replacement)` in the shared routes, each anchor found exactly once.
const EDITS: &[(&str, &str)] = &[
    (
        "    outcomes:
      placed:
        status: 201
        body: response
        headers:
          Cache-Control: {literal: no-store}
          X-Item: {source: input.item_id}
          X-Code: {source: response.code}
      refused:
        status: 409
        body: {error: error}
        headers:
          X-Error: {source: error.wire}
",
        "    outcomes:
      placed:
        status: 201
        body: response.code
        headers:
          X-Code: {source: response.code}
    errors:
      status: 422
      body: {error: error}
      headers:
        X-Error: {source: error.wire}
",
    ),
    (
        "    path: /orders
    request: {encoding: form, unknown: ignore}
",
        "    path: /orders
    request: {encoding: form, unknown: ignore}
    errors: {status: 409, body: none}
",
    ),
];

const FORM: &str = "application/x-www-form-urlencoded";

const REQUESTS: &[[&str; 7]] = &[
    [
        "placed",
        "y",
        "POST",
        "/places",
        "",
        "",
        r#"{"item_id":"i1","count":1}"#,
    ],
    [
        "refused",
        "y",
        "POST",
        "/places",
        "",
        "",
        r#"{"item_id":"i1","count":9}"#,
    ],
    [
        "ordered",
        "y",
        "POST",
        "/orders",
        "",
        FORM,
        "item_id=i1&count=1",
    ],
    [
        "out-of-stock",
        "y",
        "POST",
        "/orders",
        "",
        FORM,
        "item_id=i1&count=9",
    ],
];

/// The shared fixture with its routes edited by [`EDITS`], synthesized for both targets.
fn synthesized() -> &'static (Synthesis, Synthesis) {
    static BOTH: std::sync::OnceLock<(Synthesis, Synthesis)> = std::sync::OnceLock::new();
    BOTH.get_or_init(|| {
        let mut routes = shared::ROUTES.to_owned();
        for (anchor, replacement) in EDITS {
            assert_eq!(
                routes.matches(anchor).count(),
                1,
                "the routes moved: {anchor}"
            );
            routes = routes.replace(anchor, replacement);
        }
        let ir = ir_of(shared::MODEL);
        let bound = shared::binding(&ir, &routes);
        let rust = synthesize_bound(&ir, Target::Rust, OutputLayout::Workspace, Some(&bound))
            .unwrap_or_else(|failure| panic!("synthesizes for Rust: {failure}"));
        let go = synthesize_bound(&ir, Target::Go, OutputLayout::Workspace, Some(&bound))
            .unwrap_or_else(|failure| panic!("synthesizes for Go: {failure}"));
        (rust, go)
    })
}

/// Both servers' answers to [`REQUESTS`]: `(rust, go)`, once per test binary.
fn both() -> &'static (BTreeMap<String, Answer>, BTreeMap<String, Answer>) {
    static BOTH: std::sync::OnceLock<(BTreeMap<String, Answer>, BTreeMap<String, Answer>)> =
        std::sync::OnceLock::new();
    BOTH.get_or_init(|| {
        let (rust, go) = synthesized();
        (
            rust_run("u9dbare", rust, REQUESTS),
            go_run("u9dbare", go, REQUESTS),
        )
    })
}

/// Each server's answer to `label`, held to the same status, body bytes and declared headers.
fn answered(label: &str) -> [&'static Answer; 2] {
    let (rust, go) = both();
    let rust = rust
        .get(label)
        .unwrap_or_else(|| panic!("the Rust server answered `{label}`"));
    let go = go
        .get(label)
        .unwrap_or_else(|| panic!("the Go server answered `{label}`"));
    assert_eq!(
        (
            rust.status,
            serde_json::from_str::<serde_json::Value>(&rust.body).ok()
        ),
        (
            go.status,
            serde_json::from_str::<serde_json::Value>(&go.body).ok()
        ),
        "{label}: the servers disagree:\n  rust {rust:?}\n  go   {go:?}"
    );
    for name in ["X-Code", "X-Error"] {
        assert_eq!(
            rust.header(name),
            go.header(name),
            "{label}: {name}:\n  rust {rust:?}\n  go   {go:?}"
        );
    }
    [rust, go]
}

#[test]
fn both_servers_answer_one_response_field_bare() {
    for answer in answered("placed") {
        assert_eq!(answer.status, 201, "{answer:?}");
        assert_eq!(answer.json(), serde_json::json!("C-i1"), "{answer:?}");
        assert_eq!(answer.header("X-Code"), Some("C-i1"), "{answer:?}");
    }
}

#[test]
fn both_servers_answer_an_unlisted_error_outcome_as_errors_declares() {
    for answer in answered("refused") {
        assert_eq!(answer.status, 422, "{answer:?}");
        assert_eq!(
            answer.json(),
            serde_json::json!({"error": "out_of_stock"}),
            "{answer:?}"
        );
        assert_eq!(answer.header("X-Error"), Some("out_of_stock"), "{answer:?}");
    }
    for answer in answered("out-of-stock") {
        assert_eq!(answer.status, 409, "{answer:?}");
        assert!(answer.body.is_empty(), "no body: {answer:?}");
    }
}

#[test]
fn errors_leaves_a_success_its_envelope() {
    for answer in answered("ordered") {
        assert_eq!(answer.status, 202, "{answer:?}");
        assert_eq!(answer.json()["outcome"], "ordered", "{answer:?}");
    }
}

/// The `OpenAPI` document each server embeds describes the two routes as the servers answer them.
#[test]
fn the_embedded_openapi_spells_a_bare_field_and_errors_as_the_servers_answer_them() {
    let (rust, go) = synthesized();
    for (target, synthesis) in [("rust", rust), ("go", go)] {
        let documents: Vec<serde_json::Value> = synthesis
            .artifacts
            .iter()
            .filter(|(path, _)| path.ends_with(".openapi.json"))
            .map(|(_, artifact)| {
                serde_json::from_str(&artifact.contents).expect("the embedded document is JSON")
            })
            .collect();
        assert_eq!(documents.len(), 1, "{target}");
        let document = &documents[0];
        let place = &document["paths"]["/places"]["post"]["responses"];
        assert_eq!(
            place["201"]["content"]["application/json"]["schema"],
            serde_json::json!({"$ref": "#/components/schemas/shop.items.Place.placed.Response"}),
            "{target}: {place}"
        );
        assert_eq!(
            document["components"]["schemas"]["shop.items.Place.placed.Response"]["type"], "string",
            "{target}"
        );
        assert_eq!(
            place["422"]["x-ess-outcomes"],
            serde_json::json!(["refused"]),
            "{target}: {place}"
        );
        let order = &document["paths"]["/orders"]["post"]["responses"];
        assert_eq!(
            order["409"]["x-ess-outcomes"],
            serde_json::json!(["out-of-stock"]),
            "{target}: {order}"
        );
        assert!(order["409"].get("content").is_none(), "{target}: {order}");
        assert!(order.get("422").is_none(), "{target}: {order}");
    }
}
