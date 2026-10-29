//! Answer forms an `ess-http/1` binding declares beyond one answer per outcome, sent to both
//! generated servers: outcomes declared `alike`
//! (`story:two-outcomes-answered-alike-may-share-a-status-and-body`) and a response field answered
//! as text (`story:ess-http-admits-text-bodies-route-refusals-empty-bodies-and-array-views`).
//!
//! The shared routes change on two commands. `Place` answers `placed` with its response's `code`
//! as text. `Approve` answers its success `approved` and its `when:` refusal `denied` alike: `302`
//! to `return_to` with `client` and `state` in its query, no body.

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
",
        "    outcomes:
      placed:
        status: 201
        body: response.code
        encoding: text
        headers:
          X-Code: {source: response.code}
",
    ),
    (
        "      denied:
        status: 303
        body: none
        redirect:
          location: input.return_to
          mode: fragment
          parameters:
            error: error.wire
            state: input.state
",
        "      denied:
        status: 302
        body: none
        redirect:
          location: input.return_to
          mode: query
          parameters:
            client: input.client
            state: input.state
    alike: [[approved, denied]]
",
    ),
];

/// The `Content-Type` of a text answer.
const TEXT: &str = "text/plain; charset=utf-8";

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
    // A code holding a quote, a backslash and a letter outside ASCII: as text, unescaped.
    [
        "placed-quoted",
        "y",
        "POST",
        "/places",
        "",
        "",
        r#"{"item_id":"é\"\\1","count":1}"#,
    ],
    [
        "approved",
        "y",
        "GET",
        "/approve",
        "return_to=https%3A%2F%2Fdemo.example%2Fcb&state=s1&client=c1&level=2",
        "",
        "",
    ],
    [
        "denied",
        "y",
        "GET",
        "/approve",
        "return_to=https%3A%2F%2Fdemo.example%2Fcb&state=s1&client=c1&level=0",
        "",
        "",
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
            rust_run("u5forms", rust, REQUESTS),
            go_run("u5forms", go, REQUESTS),
        )
    })
}

/// Each server's answer to `label`, held to the same status, `Content-Type`, body bytes and
/// declared headers.
fn answered(label: &str) -> [&'static Answer; 2] {
    let (rust, go) = both();
    let rust = rust
        .get(label)
        .unwrap_or_else(|| panic!("the Rust server answered `{label}`"));
    let go = go
        .get(label)
        .unwrap_or_else(|| panic!("the Go server answered `{label}`"));
    assert_eq!(
        (rust.status, rust.content_type.as_str(), rust.body.as_str()),
        (go.status, go.content_type.as_str(), go.body.as_str()),
        "{label}: the servers disagree:\n  rust {rust:?}\n  go   {go:?}"
    );
    for name in ["X-Code", "Location"] {
        assert_eq!(
            rust.header(name),
            go.header(name),
            "{label}: {name}:\n  rust {rust:?}\n  go   {go:?}"
        );
    }
    [rust, go]
}

#[test]
fn both_servers_answer_outcomes_declared_alike_with_one_answer() {
    let [approved, _] = answered("approved");
    let [denied, _] = answered("denied");
    for answer in [approved, denied] {
        assert_eq!(answer.status, 302, "{answer:?}");
        assert!(answer.body.is_empty(), "no body: {answer:?}");
        assert_eq!(
            answer.header("Location"),
            Some("https://demo.example/cb?client=c1&state=s1"),
            "{answer:?}"
        );
    }
    assert_eq!(
        (
            approved.status,
            &approved.content_type,
            &approved.body,
            approved.header("Location")
        ),
        (
            denied.status,
            &denied.content_type,
            &denied.body,
            denied.header("Location")
        ),
        "one answer for both outcomes"
    );
}

#[test]
fn both_servers_answer_a_text_body() {
    for answer in answered("placed") {
        assert_eq!(answer.status, 201, "{answer:?}");
        assert_eq!(answer.content_type, TEXT, "{answer:?}");
        assert_eq!(
            answer.body, "C-i1",
            "the string alone, not JSON: {answer:?}"
        );
        assert_eq!(answer.header("X-Code"), Some("C-i1"), "{answer:?}");
    }
    for answer in answered("placed-quoted") {
        assert_eq!(answer.content_type, TEXT, "{answer:?}");
        assert_eq!(answer.body, "C-\u{e9}\"\\1", "unescaped: {answer:?}");
    }
}

/// The `OpenAPI` document each server embeds describes both routes as the servers answer them.
#[test]
fn the_embedded_openapi_spells_alike_and_text_as_the_servers_answer_them() {
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
        let place = &document["paths"]["/places"]["post"]["responses"]["201"];
        assert_eq!(
            place["content"]["text/plain"]["schema"],
            serde_json::json!({"$ref": "#/components/schemas/shop.items.Place.placed.Response"}),
            "{target}: {place}"
        );
        let approve = &document["paths"]["/approve"]["get"]["responses"];
        assert_eq!(
            approve["302"]["x-ess-outcomes"],
            serde_json::json!(["denied", "approved"]),
            "{target}: {approve}"
        );
        assert!(approve.get("303").is_none(), "{target}: {approve}");
        assert!(
            approve["302"].get("content").is_none(),
            "{target}: {approve}"
        );
    }
}
