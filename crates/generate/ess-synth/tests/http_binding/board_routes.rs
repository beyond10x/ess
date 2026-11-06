//! A board's card priorities sent as one bare request body beside the board in the path
//! (`story:a-command-route-takes-one-input-as-its-bare-request-body`, beyond10x/ess#748), and
//! fixed sub-resources beside a card under one prefix, a literal taking precedence over a
//! variable (`story:a-literal-path-segment-takes-precedence-over-a-variable-one`), sent to both
//! generated servers.
//!
//! The shared model gains the board's types, events and commands, granted to `Clerk`, and the
//! shared routes `SetCardPriorities` on `put /boards/{board_id}/cards/priority`, its list input
//! the whole body, `ArchiveCards` on `put /boards/{board_id}/cards/~archived` and `UpdateCard` on
//! `put /boards/{board_id}/cards/{card_id}`. Every other route keeps the shared binding's
//! answers, `refusals.malformed` (`400`, `{"error": "invalid_request"}`) among them.

use std::collections::BTreeMap;

use ess_synth::{synthesize_bound, OutputLayout, Synthesis, Target};

use super::super::shared::{self, Answer};
use super::adversary_k6_pass1::{go_run, ir_of, rust_run};

/// `(anchor, replacement)` in the shared model, each anchor found exactly once.
const MODEL_EDITS: &[(&str, &str)] = &[
    (
        "  - {name: shop.items.ReturnUri, kind: newtype, of: String}\n",
        "  - {name: shop.items.ReturnUri, kind: newtype, of: String}
  - {name: shop.items.BoardId, kind: newtype, of: String}
  - {name: shop.items.CardId, kind: newtype, of: String}
  - name: shop.items.CardPriority
    kind: struct
    fields:
      - {name: id, type: shop.items.CardId}
      - {name: priority, type: Integer}
",
    ),
    (
        "  - name: shop.items.Approved\n",
        "  - name: shop.items.PrioritiesSet
    fields:
      - {name: board_id, type: shop.items.BoardId}
      - {name: priorities, type: List<shop.items.CardPriority>}
  - name: shop.items.CardUpdated
    fields:
      - {name: board_id, type: shop.items.BoardId}
      - {name: card_id, type: shop.items.CardId}
  - name: shop.items.CardsArchived
    fields:
      - {name: board_id, type: shop.items.BoardId}
  - name: shop.items.Approved
",
    ),
    (
        "  - name: shop.items.Purge\n",
        "  - name: shop.items.SetCardPriorities
    input:
      - {name: board_id, type: shop.items.BoardId}
      - {name: priorities, type: List<shop.items.CardPriority>}
    outcomes:
      - name: set
        emits: [shop.items.PrioritiesSet]
        payload:
          shop.items.PrioritiesSet: {board_id: input.board_id, priorities: input.priorities}
  - name: shop.items.UpdateCard
    input:
      - {name: board_id, type: shop.items.BoardId}
      - {name: card_id, type: shop.items.CardId}
      - {name: title, type: String}
    outcomes:
      - name: updated
        emits: [shop.items.CardUpdated]
        payload:
          shop.items.CardUpdated: {board_id: input.board_id, card_id: input.card_id}
  - name: shop.items.ArchiveCards
    input:
      - {name: board_id, type: shop.items.BoardId}
    outcomes:
      - name: archived
        emits: [shop.items.CardsArchived]
        payload:
          shop.items.CardsArchived: {board_id: input.board_id}
  - name: shop.items.Purge
",
    ),
    (
        "      - shop.items.Approve\ncomponents:\n",
        "      - shop.items.Approve
      - shop.items.SetCardPriorities
      - shop.items.UpdateCard
      - shop.items.ArchiveCards
components:
",
    ),
    (
        "        - shop.items.Archive\n",
        "        - shop.items.Archive
        - shop.items.SetCardPriorities
        - shop.items.UpdateCard
        - shop.items.ArchiveCards
",
    ),
    (
        "shop.items.Searched, shop.items.Approved]",
        "shop.items.Searched, shop.items.Approved, shop.items.PrioritiesSet, shop.items.CardUpdated, shop.items.CardsArchived]",
    ),
];

/// The board's routes, after the shared ones. `~archived` and `priority` are literals beside the
/// variable `{card_id}` at one position; `{` sorts after `priority` and before `~archived`.
const BOARD_ROUTES: &str = "  - command: shop.items.SetCardPriorities
    method: put
    path: /boards/{board_id}/cards/priority
    parameters: {board_id: input.board_id}
    request: {encoding: json, body: input.priorities}
  - command: shop.items.UpdateCard
    method: put
    path: /boards/{board_id}/cards/{card_id}
    parameters: {board_id: input.board_id, card_id: input.card_id}
  - command: shop.items.ArchiveCards
    method: put
    path: /boards/{board_id}/cards/~archived
    parameters: {board_id: input.board_id}
";

const PRIORITIES: &str = r#"[{"id":"c-1","priority":1},{"id":"c-2","priority":2}]"#;

const REQUESTS: &[[&str; 7]] = &[
    [
        "priorities-bare",
        "y",
        "PUT",
        "/boards/b-1/cards/priority",
        "",
        "",
        PRIORITIES,
    ],
    [
        "priorities-object",
        "y",
        "PUT",
        "/boards/b-1/cards/priority",
        "",
        "",
        r#"{"priorities":[{"id":"c-1","priority":1}]}"#,
    ],
    [
        "priorities-none",
        "y",
        "PUT",
        "/boards/b-1/cards/priority",
        "",
        "",
        "",
    ],
    [
        "priorities-not-json",
        "y",
        "PUT",
        "/boards/b-1/cards/priority",
        "",
        "",
        "[",
    ],
    [
        "card-c9",
        "y",
        "PUT",
        "/boards/b-1/cards/c-9",
        "",
        "",
        r#"{"title":"T"}"#,
    ],
    [
        "cards-archived",
        "y",
        "PUT",
        "/boards/b-1/cards/~archived",
        "",
        "",
        "",
    ],
];

/// The shared model edited by [`MODEL_EDITS`].
fn model() -> String {
    let mut model = shared::MODEL.to_owned();
    for (anchor, replacement) in MODEL_EDITS {
        assert_eq!(
            model.matches(anchor).count(),
            1,
            "the model moved: {anchor}"
        );
        model = model.replace(anchor, replacement);
    }
    model
}

/// The shared fixture with the board's model and routes, synthesized for both targets.
fn synthesized() -> &'static (Synthesis, Synthesis) {
    static BOTH: std::sync::OnceLock<(Synthesis, Synthesis)> = std::sync::OnceLock::new();
    BOTH.get_or_init(|| {
        let routes = format!("{}{BOARD_ROUTES}", shared::ROUTES);
        let ir = ir_of(&model());
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
            rust_run("board", rust, REQUESTS),
            go_run("board", go, REQUESTS),
        )
    })
}

/// Each server's answer to `label`, held to the same status and JSON body.
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
    [rust, go]
}

#[test]
fn both_servers_decode_a_bare_array_body() {
    for answer in answered("priorities-bare") {
        assert_eq!(answer.status, 202, "{answer:?}");
        assert_eq!(answer.json()["outcome"], "set", "{answer:?}");
        assert_eq!(
            shared::published(answer),
            serde_json::json!({
                "board_id": "b-1",
                "priorities": [{"id": "c-1", "priority": 1}, {"id": "c-2", "priority": 2}],
            }),
            "{answer:?}"
        );
    }
    // An object is not the list the body is, and no body or one that is not JSON is no list.
    for label in [
        "priorities-object",
        "priorities-none",
        "priorities-not-json",
    ] {
        for answer in answered(label) {
            assert_eq!(answer.status, 400, "{label}: {answer:?}");
            assert_eq!(
                answer.json(),
                serde_json::json!({"error": "invalid_request"}),
                "{label}: {answer:?}"
            );
        }
    }
}

/// story:a-literal-path-segment-takes-precedence-over-a-variable-one: at the position a literal
/// sits beside `{card_id}`, the literal route serves its own path, whether it sorts before `{`
/// (`priority`) or after it (`~archived`), and every other card id reaches `UpdateCard`.
#[test]
fn both_servers_serve_the_literal_route_before_the_variable_one() {
    for answer in answered("priorities-bare") {
        assert_eq!(answer.status, 202, "{answer:?}");
        assert_eq!(answer.json()["outcome"], "set", "{answer:?}");
    }
    for answer in answered("cards-archived") {
        assert_eq!(answer.status, 202, "{answer:?}");
        assert_eq!(answer.json()["outcome"], "archived", "{answer:?}");
        assert_eq!(
            shared::published(answer),
            serde_json::json!({"board_id": "b-1"}),
            "{answer:?}"
        );
    }
    for answer in answered("card-c9") {
        assert_eq!(answer.status, 202, "{answer:?}");
        assert_eq!(answer.json()["outcome"], "updated", "{answer:?}");
        assert_eq!(
            shared::published(answer),
            serde_json::json!({"board_id": "b-1", "card_id": "c-9"}),
            "{answer:?}"
        );
    }
}

/// The `OpenAPI` document each server embeds publishes the body as the list it reads.
#[test]
fn the_embedded_openapi_publishes_the_bare_body_the_servers_read() {
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
        let body = &documents[0]["paths"]["/boards/{board_id}/cards/priority"]["put"]
            ["requestBody"]["content"]["application/json"]["schema"];
        assert_eq!(
            body,
            &serde_json::json!({
                "type": "array",
                "items": {"$ref": "#/components/schemas/shop.items.CardPriority"},
            }),
            "{target}"
        );
    }
}
