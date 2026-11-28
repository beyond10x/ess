//! Answers wrapped in one named member, sent to both generated servers (beyond10x/ess#755):
//! `ItemById` answers `body: {item: single}`, `ItemsTitled` `body: {items: array}` and `Place`'s
//! `placed` `body: {data: response}`. An empty answer keeps its own form, unwrapped.

use std::collections::BTreeMap;

use ess_synth::{synthesize_bound, OutputLayout, Synthesis, Target};

use super::super::shared::{self, Answer, Server};
use super::adversary_k6_pass1::{answers, ir_of};
use super::answer_forms::{go_harness, rust_harness, MODEL_EDITS};

/// `(anchor, replacement)` in the shared routes, each anchor found exactly once.
const EDITS: &[(&str, &str)] = &[
    ("    body: single\n", "    body: {item: single}\n"),
    (
        "      placed:\n        status: 201\n        body: response\n",
        "      placed:\n        status: 201\n        body: {data: response}\n",
    ),
    (
        "  - command: shop.items.Purge\n",
        "  - view: shop.items.ItemsTitled
    method: get
    path: /titled/{title}
    parameters: {title: param.title}
    body: {items: array}
  - command: shop.items.Purge\n",
    ),
];

const REQUESTS: &[[&str; 7]] = &[
    [
        "put-lamp",
        "y",
        "PUT",
        "/items/t1",
        "",
        "",
        r#"{"title":"Lamp"}"#,
    ],
    ["get", "y", "GET", "/items/t1", "", "", ""],
    ["get-missing", "y", "GET", "/items/zz", "", "", ""],
    ["titled", "y", "GET", "/titled/Lamp", "", "", ""],
    ["titled-none", "y", "GET", "/titled/Desk", "", "", ""],
    [
        "placed",
        "y",
        "POST",
        "/places",
        "",
        "",
        r#"{"item_id":"i1","count":1}"#,
    ],
];

/// `text` with every [`EDITS`]-like `(anchor, replacement)` applied, each anchor found once.
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
        Server::rust("w755", &synthesis(Target::Rust), &rust_harness()),
        Server::go("w755", &synthesis(Target::Go), &go_harness()),
    ]
}

/// Both servers' answers to [`REQUESTS`]: `(rust, go)`, once per test binary.
fn both() -> &'static (BTreeMap<String, Answer>, BTreeMap<String, Answer>) {
    static BOTH: std::sync::OnceLock<(BTreeMap<String, Answer>, BTreeMap<String, Answer>)> =
        std::sync::OnceLock::new();
    BOTH.get_or_init(|| answers(&servers(), REQUESTS))
}

/// Each server's answer to `label`, held to the same status, `Content-Type` and body bytes.
fn answered(label: &str) -> &'static Answer {
    let (rust, go) = both();
    let rust = &rust[label];
    let go = &go[label];
    assert_eq!(
        (rust.status, rust.content_type.as_str(), rust.body.as_str()),
        (go.status, go.content_type.as_str(), go.body.as_str()),
        "{label}: the servers disagree:\n  rust {rust:?}\n  go   {go:?}"
    );
    assert_eq!(rust.header("X-Code"), go.header("X-Code"), "{label}");
    rust
}

#[test]
fn both_servers_answer_a_wrapped_row_rows_and_response() {
    let get = answered("get");
    assert_eq!(get.status, 200, "{get:?}");
    assert_eq!(
        get.body, r#"{"item":{"item_id":"t1","title":"Lamp"}}"#,
        "{get:?}"
    );

    let missing = answered("get-missing");
    assert_eq!(missing.status, 404, "{missing:?}");
    assert!(
        missing.body.is_empty(),
        "the empty answer is not wrapped: {missing:?}"
    );

    let titled = answered("titled");
    assert_eq!(titled.status, 200, "{titled:?}");
    assert_eq!(
        titled.body, r#"{"items":[{"item_id":"t1","title":"Lamp"}]}"#,
        "{titled:?}"
    );
    assert_eq!(answered("titled-none").body, r#"{"items":[]}"#);

    let placed = answered("placed");
    assert_eq!(placed.status, 201, "{placed:?}");
    assert_eq!(
        placed.json(),
        serde_json::json!({"data": {"code": "C-i1"}}),
        "{placed:?}"
    );
    assert_eq!(placed.header("X-Code"), Some("C-i1"), "{placed:?}");
}
