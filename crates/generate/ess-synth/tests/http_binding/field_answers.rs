//! A view route answering each row as one field's value
//! (`story:a-view-may-answer-each-row-as-one-scalar-value`, BA-D1 A, BA-D2 yes), sent to both
//! generated servers: `ItemById` answers `body: single.title` and `ItemsTitled` `body:
//! array.item_id`, the other field of each carried by the route's path. An empty answer keeps its
//! own form.

use std::collections::BTreeMap;

use ess_synth::{synthesize_bound, OutputLayout, Synthesis, Target};

use super::super::shared::{self, Answer};
use super::adversary_k6_pass1::{go_run_with, ir_of, rust_run_with};
use super::answer_forms::{go_harness, rust_harness, MODEL_EDITS};

/// `(anchor, replacement)` in the shared routes, each anchor found exactly once.
const EDITS: &[(&str, &str)] = &[
    ("    body: single\n", "    body: single.title\n"),
    (
        "  - command: shop.items.Purge\n",
        "  - view: shop.items.ItemsTitled
    method: get
    path: /titled/{title}
    parameters: {title: param.title}
    body: array.item_id
  - command: shop.items.Purge\n",
    ),
];

const REQUESTS: &[[&str; 7]] = &[
    [
        "put-1",
        "y",
        "PUT",
        "/items/t1",
        "",
        "",
        r#"{"title":"Lamp"}"#,
    ],
    [
        "put-2",
        "y",
        "PUT",
        "/items/t2",
        "",
        "",
        r#"{"title":"Lamp"}"#,
    ],
    ["get", "y", "GET", "/items/t1", "", "", ""],
    ["get-missing", "y", "GET", "/items/zz", "", "", ""],
    ["titled", "y", "GET", "/titled/Lamp", "", "", ""],
    ["titled-none", "y", "GET", "/titled/Desk", "", "", ""],
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

/// Both servers' answers to [`REQUESTS`]: `(rust, go)`, once per test binary.
fn both() -> &'static (BTreeMap<String, Answer>, BTreeMap<String, Answer>) {
    static BOTH: std::sync::OnceLock<(BTreeMap<String, Answer>, BTreeMap<String, Answer>)> =
        std::sync::OnceLock::new();
    BOTH.get_or_init(|| {
        let ir = ir_of(&edited(shared::MODEL, MODEL_EDITS));
        let bound = shared::binding(&ir, &edited(shared::ROUTES, EDITS));
        let synthesis = |target| -> Synthesis {
            synthesize_bound(&ir, target, OutputLayout::Workspace, Some(&bound))
                .unwrap_or_else(|failure| panic!("synthesizes for {target:?}: {failure}"))
        };
        (
            rust_run_with(
                "fieldans",
                &synthesis(Target::Rust),
                REQUESTS,
                &rust_harness(),
            ),
            go_run_with("fieldans", &synthesis(Target::Go), REQUESTS, &go_harness()),
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
    rust
}

#[test]
fn both_servers_answer_each_row_as_one_field_value() {
    let get = answered("get");
    assert_eq!(get.status, 200, "{get:?}");
    assert_eq!(get.body, r#""Lamp""#, "{get:?}");
    let missing = answered("get-missing");
    assert_eq!(missing.status, 404, "{missing:?}");
    assert!(missing.body.is_empty(), "{missing:?}");
    let titled = answered("titled");
    assert_eq!(titled.status, 200, "{titled:?}");
    assert_eq!(titled.body, r#"["t1","t2"]"#, "{titled:?}");
    assert_eq!(answered("titled-none").body, "[]");
}
