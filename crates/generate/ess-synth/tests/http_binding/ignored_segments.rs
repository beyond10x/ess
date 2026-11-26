//! A path variable the binding declares `ignored`
//! (`story:a-route-may-declare-a-path-variable-the-service-reads-and-ignores`, HB-D3 B), sent to
//! both generated servers: the shared `ItemById` route moved under `/shelves/{shelf}`, `shelf`
//! ignored. Any non-empty segment there answers as the route without it, and an empty one is not
//! that route.

use std::collections::BTreeMap;

use ess_synth::{synthesize_bound, OutputLayout, Synthesis, Target};

use super::super::shared::{self, Answer};
use super::adversary_k6_pass1::{go_run_with, ir_of, rust_run_with};

/// `(anchor, replacement)` in the shared routes.
const EDITS: &[(&str, &str)] = &[(
    "    path: /items/{item_id}\n    parameters: {item_id: param.item_id}\n",
    "    path: /shelves/{shelf}/items/{item_id}\n    parameters: {shelf: ignored, item_id: param.item_id}\n",
)];

const REQUESTS: &[[&str; 7]] = &[
    ["put", "y", "PUT", "/items/a", "", "", r#"{"title":"Lamp"}"#],
    ["get", "y", "GET", "/shelves/s1/items/a", "", "", ""],
    [
        "get-other-shelf",
        "y",
        "GET",
        "/shelves/x%20y/items/a",
        "",
        "",
        "",
    ],
    [
        "get-empty-shelf",
        "y",
        "GET",
        "/shelves//items/a",
        "",
        "",
        "",
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

/// Both servers' answers to [`REQUESTS`]: `(rust, go)`, once per test binary.
fn both() -> &'static (BTreeMap<String, Answer>, BTreeMap<String, Answer>) {
    static BOTH: std::sync::OnceLock<(BTreeMap<String, Answer>, BTreeMap<String, Answer>)> =
        std::sync::OnceLock::new();
    BOTH.get_or_init(|| {
        let ir = ir_of(shared::MODEL);
        let bound = shared::binding(&ir, &edited(shared::ROUTES, EDITS));
        let synthesis = |target| -> Synthesis {
            synthesize_bound(&ir, target, OutputLayout::Workspace, Some(&bound))
                .unwrap_or_else(|failure| panic!("synthesizes for {target:?}: {failure}"))
        };
        (
            rust_run_with(
                "ignoredseg",
                &synthesis(Target::Rust),
                REQUESTS,
                shared::RUST_HARNESS,
            ),
            go_run_with(
                "ignoredseg",
                &synthesis(Target::Go),
                REQUESTS,
                super::HARNESS,
            ),
        )
    })
}

/// Each server's answer to `label`, the two byte for byte alike.
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
fn both_servers_accept_any_value_in_an_ignored_segment_and_read_none() {
    assert_eq!(answered("put").status, 202, "{:?}", answered("put"));
    let get = answered("get");
    assert_eq!(get.status, 200, "{get:?}");
    assert_eq!(
        get.json(),
        serde_json::json!({"item_id": "a", "title": "Lamp"}),
        "{get:?}"
    );
    assert_eq!(answered("get-other-shelf").body, get.body);
    let empty = answered("get-empty-shelf");
    assert_eq!(empty.status, 404, "{empty:?}");
}
