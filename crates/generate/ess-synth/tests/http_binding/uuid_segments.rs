//! A path segment carrying a `Uuid` (beyond10x/ess#755), sent to both generated servers: the shared
//! model's `ItemId` is a newtype of `Uuid`, so `PUT /items/{item_id}` and `GET /items/{item_id}`
//! carry one, and a segment that is not one is `malformed`. The binding's own `malformed` answer is
//! left out, so the standard one, which names what was refused, is the answer: the field where the
//! input's decoder reads it, `query.<wire>` for a view and `body.<wire>` for a command.

use std::collections::BTreeMap;

use ess_synth::{synthesize_bound, OutputLayout, Synthesis, Target};

use super::super::shared::{self, Answer, Server};
use super::adversary_k6_pass1::{answers, ir_of};

/// `(anchor, replacement)` in the shared model.
const MODEL_EDITS: &[(&str, &str)] = &[(
    "  - {name: shop.items.ItemId, kind: newtype, of: String}\n",
    "  - {name: shop.items.ItemId, kind: newtype, of: Uuid}\n",
)];

/// `(anchor, replacement)` in the shared routes: the binding's `malformed` answer left out.
const EDITS: &[(&str, &str)] = &[(
    "  malformed:
    status: 400
    body: {error: {literal: invalid_request}}
    headers:
      Cache-Control: {literal: no-store}
",
    "",
)];

/// `(anchor, replacement)` in the harnesses: the code a placed item answers reads the UUID's text.
const RUST_EDITS: &[(&str, &str)] = &[("input.item_id.0)", "input.item_id.0.0)")];
const GO_EDITS: &[(&str, &str)] = &[("input.ItemId.Value()}", "input.ItemId.Value().Value()}")];

const UUID: &str = "0b6f3c1e-8f0a-4c55-9d0e-6f1a2b3c4d5e";

const REQUESTS: &[[&str; 7]] = &[
    [
        "put",
        "y",
        "PUT",
        "/items/0b6f3c1e-8f0a-4c55-9d0e-6f1a2b3c4d5e",
        "",
        "",
        r#"{"title":"Lamp"}"#,
    ],
    [
        "get",
        "y",
        "GET",
        "/items/0b6f3c1e-8f0a-4c55-9d0e-6f1a2b3c4d5e",
        "",
        "",
        "",
    ],
    [
        "get-not-a-uuid",
        "y",
        "GET",
        "/items/not-a-uuid",
        "",
        "",
        "",
    ],
    [
        "put-not-a-uuid",
        "y",
        "PUT",
        "/items/not-a-uuid",
        "",
        "",
        r#"{"title":"Lamp"}"#,
    ],
];

/// `text` with every `(anchor, replacement)` applied, each anchor found once.
fn edited(text: &str, edits: &[(&str, &str)]) -> String {
    let mut text = text.to_owned();
    for (anchor, replacement) in edits {
        assert_eq!(
            text.matches(anchor).count(),
            1,
            "the fixture moved: {anchor}"
        );
        text = text.replace(anchor, replacement);
    }
    text
}

/// The servers [`both`] asks: `[rust, go]`.
pub(super) fn servers() -> Vec<Server> {
    let ir = ir_of(&edited(shared::MODEL, MODEL_EDITS));
    let bound = shared::binding(&ir, &edited(shared::ROUTES, EDITS));
    let synthesis = |target| -> Synthesis {
        synthesize_bound(&ir, target, OutputLayout::Workspace, Some(&bound))
            .unwrap_or_else(|failure| panic!("synthesizes for {target:?}: {failure}"))
    };
    vec![
        Server::rust(
            "u755uuid",
            &synthesis(Target::Rust),
            &edited(shared::RUST_HARNESS, RUST_EDITS),
        ),
        Server::go(
            "u755uuid",
            &synthesis(Target::Go),
            &edited(super::HARNESS, GO_EDITS),
        ),
    ]
}

/// Both servers' answers to [`REQUESTS`]: `(rust, go)`, once per test binary.
fn both() -> &'static (BTreeMap<String, Answer>, BTreeMap<String, Answer>) {
    static BOTH: std::sync::OnceLock<(BTreeMap<String, Answer>, BTreeMap<String, Answer>)> =
        std::sync::OnceLock::new();
    BOTH.get_or_init(|| answers(&servers(), REQUESTS))
}

/// Each server's answer to `label`, held to the same status and `Content-Type`, and to the same
/// body: bytes where the binding writes it, JSON where ESS's standard refusal does.
fn answered(label: &str) -> &'static Answer {
    let (rust, go) = both();
    let rust = &rust[label];
    let go = &go[label];
    assert_eq!(
        (rust.status, rust.content_type.as_str(), rust.json()),
        (go.status, go.content_type.as_str(), go.json()),
        "{label}: the servers disagree:\n  rust {rust:?}\n  go   {go:?}"
    );
    rust
}

#[test]
fn both_servers_read_a_uuid_from_a_path_segment() {
    let put = answered("put");
    assert_eq!(put.status, 202, "{put:?}");
    let get = answered("get");
    assert_eq!(get.status, 200, "{get:?}");
    assert_eq!(
        get.json(),
        serde_json::json!({"item_id": UUID, "title": "Lamp"}),
        "{get:?}"
    );
    // `malformed`, the field named where the input's decoder reads it.
    for (label, at) in [
        ("get-not-a-uuid", "query.item_id"),
        ("put-not-a-uuid", "body.item_id"),
    ] {
        let refused = answered(label);
        assert_eq!(refused.status, 400, "{label}: {refused:?}");
        assert_eq!(
            refused.json(),
            serde_json::json!({"refused": format!("{at}: expected a UUID, found `not-a-uuid`")}),
            "{label}: {refused:?}"
        );
    }
}
