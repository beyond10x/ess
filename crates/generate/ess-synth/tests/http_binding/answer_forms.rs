//! Answer forms an `ess-http/1` binding declares beyond one answer per outcome, sent to both
//! generated servers: outcomes declared `alike`
//! (`story:two-outcomes-answered-alike-may-share-a-status-and-body`), and a response field answered
//! as text, a route's own standard refusals, an empty answer's literal body and a view answered as
//! a bare array (`story:ess-http-admits-text-bodies-route-refusals-empty-bodies-and-array-views`).
//!
//! The shared routes change on two commands. `Place` answers `placed` with its response's `code`
//! as text. `Approve` answers its success `approved` and its `when:` refusal `denied` alike: `302`
//! to `return_to` with `client` and `state` in its query, no body. `PutItem` answers `malformed`
//! its own way, and `Purge` `not_granted`; every other route keeps the binding's. `ItemById`'s
//! empty answer carries `{"error": "not_found"}`. The model gains `ItemsTitled`, routed with
//! `body: array`.

use std::collections::BTreeMap;

use ess_synth::{synthesize_bound, OutputLayout, Synthesis, Target};

use super::super::shared::{self, Answer};
use super::adversary_k6_pass1::{go_run_with, ir_of, rust_run_with};

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
    (
        "    body: single
    empty: {status: 404}
",
        "    body: single
    empty: {status: 404, body: {error: {literal: not_found}}}
",
    ),
    (
        "  - command: shop.items.PutItem
    method: put
    path: /items/{item_id}
    parameters: {item_id: input.item_id}
",
        "  - command: shop.items.PutItem
    method: put
    path: /items/{item_id}
    parameters: {item_id: input.item_id}
    refusals:
      malformed: {status: 422, body: {error: {literal: unreadable}}}
",
    ),
    (
        "  - command: shop.items.Purge
    method: post
    path: /purge/{item_id}
    parameters: {item_id: input.item_id}
",
        "  - command: shop.items.Purge
    method: post
    path: /purge/{item_id}
    parameters: {item_id: input.item_id}
    refusals:
      not_granted: {status: 404, body: none}
  - view: shop.items.ItemsTitled
    method: get
    path: /titled/{title}
    parameters: {title: param.title}
    body: array
",
    ),
];

/// `(anchor, replacement)` in the shared model: a view of every item with one title.
const MODEL_EDITS: &[(&str, &str)] = &[(
    "actors:\n",
    "  - name: shop.items.ItemsTitled
    source: shop.items.Item
    consistency: read_your_writes
    params:
      - {name: title, type: String}
    filter: title == param.title
    fields:
      - {name: item_id, type: shop.items.ItemId}
      - {name: title, type: String}
actors:
",
)];

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
    [
        "put-lamp",
        "y",
        "PUT",
        "/items/t1",
        "",
        "",
        r#"{"title":"Lamp"}"#,
    ],
    ["titled", "y", "GET", "/titled/Lamp", "", "", ""],
    ["titled-none", "y", "GET", "/titled/Desk", "", "", ""],
    ["get-missing", "y", "GET", "/items/zz", "", "", ""],
    ["put-malformed", "y", "PUT", "/items/m1", "", "", "{"],
    [
        "strict-malformed",
        "y",
        "POST",
        "/strict-orders",
        "",
        "application/x-www-form-urlencoded",
        "item_id=s1&count=x",
    ],
    ["purge", "y", "POST", "/purge/p1", "", "", ""],
    [
        "archive",
        "y",
        "POST",
        "/items/commands/Archive",
        "",
        "",
        r#"{"item_id":"p1"}"#,
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
        let mut model = shared::MODEL.to_owned();
        for (anchor, replacement) in MODEL_EDITS {
            assert_eq!(
                model.matches(anchor).count(),
                1,
                "the model moved: {anchor}"
            );
            model = model.replace(anchor, replacement);
        }
        let ir = ir_of(&model);
        let bound = shared::binding(&ir, &routes);
        let rust = synthesize_bound(&ir, Target::Rust, OutputLayout::Workspace, Some(&bound))
            .unwrap_or_else(|failure| panic!("synthesizes for Rust: {failure}"));
        let go = synthesize_bound(&ir, Target::Go, OutputLayout::Workspace, Some(&bound))
            .unwrap_or_else(|failure| panic!("synthesizes for Go: {failure}"));
        (rust, go)
    })
}

/// `text` with `addition` written in front of `anchor`, found exactly once.
fn before(text: &str, anchor: &str, addition: &str) -> String {
    assert_eq!(
        text.matches(anchor).count(),
        1,
        "the harness moved: {anchor}"
    );
    text.replace(anchor, &format!("{addition}{anchor}"))
}

/// [`shared::RUST_HARNESS`] answering `ItemsTitled` over the stored items, too.
fn rust_harness() -> String {
    let text = before(
        shared::RUST_HARNESS,
        "ItemByIdQuery, PlaceBehavior}",
        "ItemsTitledQuery, ",
    );
    before(
        &text,
        "fn clean(text: &str) -> String {",
        "impl ItemsTitledQuery for Ports {
    fn items_titled(&self, title: String) -> Result<Vec<items::ItemsTitled>, UnmetObligation> {
        Ok(self
            .items
            .iter()
            .filter(|row| row.data.title == title)
            .map(|row| items::ItemsTitled { item_id: row.data.item_id.clone(), title: row.data.title.clone() })
            .collect())
    }
}

",
    )
}

/// The Go harness answering `ItemsTitled` over the stored items, too.
fn go_harness() -> String {
    before(
        super::HARNESS,
        "func clean(text string) string {",
        "func (p *ports) ItemsTitled(title string) ([]items.ItemsTitled, *obligation.UnmetObligation) {
	found := []items.ItemsTitled{}
	for _, row := range p.rows {
		if row.Data.Title == title {
			found = append(found, items.ItemsTitled{ItemId: row.Data.ItemId, Title: row.Data.Title})
		}
	}
	return found, nil
}

",
    )
}

/// Both servers' answers to [`REQUESTS`]: `(rust, go)`, once per test binary.
fn both() -> &'static (BTreeMap<String, Answer>, BTreeMap<String, Answer>) {
    static BOTH: std::sync::OnceLock<(BTreeMap<String, Answer>, BTreeMap<String, Answer>)> =
        std::sync::OnceLock::new();
    BOTH.get_or_init(|| {
        let (rust, go) = synthesized();
        (
            rust_run_with("u5forms", rust, REQUESTS, &rust_harness()),
            go_run_with("u5forms", go, REQUESTS, &go_harness()),
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

#[test]
fn both_servers_answer_route_refusals_empty_bodies_and_arrays() {
    // A route's own `malformed`: its status and body, without the binding's `Cache-Control`.
    for answer in answered("put-malformed") {
        assert_eq!(answer.status, 422, "{answer:?}");
        assert_eq!(answer.body, r#"{"error":"unreadable"}"#, "{answer:?}");
        assert_eq!(answer.header("Cache-Control"), None, "{answer:?}");
    }
    // Another declared route keeps the binding's.
    for answer in answered("strict-malformed") {
        assert_eq!(answer.status, 400, "{answer:?}");
        assert_eq!(answer.body, r#"{"error":"invalid_request"}"#, "{answer:?}");
        assert_eq!(
            answer.header("Cache-Control"),
            Some("no-store"),
            "{answer:?}"
        );
    }
    // A route's own `not_granted`; a derived route keeps the standard `403`.
    for answer in answered("purge") {
        assert_eq!(answer.status, 404, "{answer:?}");
        assert!(answer.body.is_empty(), "no body: {answer:?}");
    }
    // Its standard body is JSON whose members the two servers write in their own order.
    let (rust, go) = both();
    for answer in [&rust["archive"], &go["archive"]] {
        assert_eq!(answer.status, 403, "{answer:?}");
        assert_eq!(
            answer.json(),
            serde_json::json!({"refused": "not granted", "actor": "shop.items.Clerk"}),
            "{answer:?}"
        );
    }
    // An empty answer's literal body.
    for answer in answered("get-missing") {
        assert_eq!(answer.status, 404, "{answer:?}");
        assert_eq!(answer.body, r#"{"error":"not_found"}"#, "{answer:?}");
        assert_eq!(answer.content_type, "application/json", "{answer:?}");
    }
    // The rows, bare.
    for answer in answered("titled") {
        assert_eq!(answer.status, 200, "{answer:?}");
        assert_eq!(
            answer.json(),
            serde_json::json!([{"item_id": "t1", "title": "Lamp"}]),
            "{answer:?}"
        );
    }
    for answer in answered("titled-none") {
        assert_eq!(answer.status, 200, "{answer:?}");
        assert_eq!(answer.body, "[]", "{answer:?}");
    }
}
