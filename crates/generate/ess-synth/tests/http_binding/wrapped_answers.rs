//! Answers wrapped in a named member, sent to both generated servers (beyond10x/ess#755):
//! `ItemById` answers `body: {item: single, success: {literal: ok}}`, `ItemsTitled`
//! `body: {items: array}`, `ItemsLabelled` `body: {items: array, kind: {literal: item}, total:
//! count}` and `Place`'s `placed` `body: {data: response, success: {literal: ok}}`. An empty answer
//! keeps its own form, unwrapped.

use std::collections::BTreeMap;

use ess_synth::{synthesize_bound, OutputLayout, Synthesis, Target};

use super::super::shared::{self, Answer};
use super::adversary_k6_pass1::{go_run_with, ir_of, rust_run_with};
use super::answer_forms::{go_harness, rust_harness, MODEL_EDITS};

/// `(anchor, replacement)` in the shared routes, each anchor found exactly once.
const EDITS: &[(&str, &str)] = &[
    (
        "    body: single\n",
        "    body: {item: single, success: {literal: ok}}\n",
    ),
    (
        "      placed:\n        status: 201\n        body: response\n",
        "      placed:\n        status: 201\n        body: {data: response, success: {literal: ok}}\n",
    ),
    (
        "  - command: shop.items.Purge\n",
        "  - view: shop.items.ItemsTitled
    method: get
    path: /titled/{title}
    parameters: {title: param.title}
    body: {items: array}
  - view: shop.items.ItemsLabelled
    method: get
    path: /labelled/{title}
    parameters: {title: param.title}
    body: {items: array, kind: {literal: item}, total: count}
  - command: shop.items.Purge\n",
    ),
];

/// A second view of the rows `ItemsTitled` reads, answered with literal members and a count.
const MODEL_LABELLED: &[(&str, &str)] = &[(
    "actors:\n",
    "  - name: shop.items.ItemsLabelled
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
    ["labelled", "y", "GET", "/labelled/Lamp", "", "", ""],
    ["labelled-none", "y", "GET", "/labelled/Desk", "", "", ""],
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

/// Both servers' answers to [`REQUESTS`]: `(rust, go)`, once per test binary.
fn both() -> &'static (BTreeMap<String, Answer>, BTreeMap<String, Answer>) {
    static BOTH: std::sync::OnceLock<(BTreeMap<String, Answer>, BTreeMap<String, Answer>)> =
        std::sync::OnceLock::new();
    BOTH.get_or_init(|| {
        let ir = ir_of(&edited(&edited(shared::MODEL, MODEL_EDITS), MODEL_LABELLED));
        let bound = shared::binding(&ir, &edited(shared::ROUTES, EDITS));
        let synthesis = |target| -> Synthesis {
            synthesize_bound(&ir, target, OutputLayout::Workspace, Some(&bound))
                .unwrap_or_else(|failure| panic!("synthesizes for {target:?}: {failure}"))
        };
        (
            rust_run_with("w755", &synthesis(Target::Rust), REQUESTS, &rust_harness()),
            go_run_with("w755", &synthesis(Target::Go), REQUESTS, &go_harness()),
        )
    })
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
        get.body, r#"{"item":{"item_id":"t1","title":"Lamp"},"success":"ok"}"#,
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

    // Literal members and the count of the rows beside them (decision HB-D4).
    let labelled = answered("labelled");
    assert_eq!(labelled.status, 200, "{labelled:?}");
    assert_eq!(
        labelled.body, r#"{"items":[{"item_id":"t1","title":"Lamp"}],"kind":"item","total":1}"#,
        "{labelled:?}"
    );
    assert_eq!(
        answered("labelled-none").body,
        r#"{"items":[],"kind":"item","total":0}"#
    );

    let placed = answered("placed");
    assert_eq!(placed.status, 201, "{placed:?}");
    assert_eq!(
        placed.json(),
        serde_json::json!({"data": {"code": "C-i1"}, "success": "ok"}),
        "{placed:?}"
    );
    assert_eq!(placed.header("X-Code"), Some("C-i1"), "{placed:?}");
}
