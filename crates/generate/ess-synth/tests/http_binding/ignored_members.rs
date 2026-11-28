//! `request: {unknown: ignore}` sent to both generated servers (beyond10x/ess#752,
//! `story:generated-servers-ignore-undeclared-json-members-and-check-echoed-path-fields`): a `json`
//! body drops a member no input field names at its top level, and a member the path carries when
//! it holds the path's value; one holding another value is malformed, a `form` or `query` key the
//! path carries follows the same rule, and a struct inside the body stays closed. Under `refuse`
//! an echoed key is malformed as before.
//!
//! `PutItem`'s route answers `malformed` its own way (`422`), so the status is the binding's; every
//! other route answers the default `400` naming the member.

use std::collections::BTreeMap;

use ess_synth::{synthesize_bound, OutputLayout, Synthesis, Target};

use super::super::shared::{self, Answer, Server};
use super::adversary_k6_pass1::{answers, ir_of};

/// The model gains a struct `Label` and a command `Relabel` taking one.
fn model() -> String {
    let edits = [
        (
            "  - {name: shop.items.ReturnUri, kind: newtype, of: String}\n",
            "  - {name: shop.items.ReturnUri, kind: newtype, of: String}
  - name: shop.items.Label
    kind: struct
    fields:
      - {name: text, type: String}
",
        ),
        (
            "  - name: shop.items.Purge\n",
            "  - name: shop.items.Relabel
    input:
      - {name: item_id, type: shop.items.ItemId}
      - {name: label, type: shop.items.Label}
    outcomes:
      - name: relabelled
        emits: [shop.items.Touched]
        payload:
          shop.items.Touched: {item_id: input.item_id}
  - name: shop.items.Purge\n",
        ),
        (
            "      - shop.items.Approve\ncomponents:\n",
            "      - shop.items.Approve\n      - shop.items.Relabel\ncomponents:\n",
        ),
        (
            "        - shop.items.Archive\n",
            "        - shop.items.Archive\n        - shop.items.Relabel\n",
        ),
    ];
    let mut model = shared::MODEL.to_owned();
    for (anchor, replacement) in edits {
        assert_eq!(
            model.matches(anchor).count(),
            1,
            "the model moved: {anchor}"
        );
        model = model.replace(anchor, replacement);
    }
    model
}

const ROUTES: &str = "routes:
  - command: shop.items.PutItem
    method: put
    path: /items/{item_id}
    parameters: {item_id: input.item_id}
    request: {unknown: ignore}
    refusals:
      malformed: {status: 422, body: {error: {literal: invalid_request}}}
  - command: shop.items.Relabel
    method: put
    path: /labels/{item_id}
    parameters: {item_id: input.item_id}
    request: {encoding: json, unknown: ignore}
  - command: shop.items.Place
    method: post
    path: /places/{count}
    parameters: {count: input.count}
    request: {unknown: ignore}
  - command: shop.items.Order
    method: post
    path: /orders/{item_id}
    parameters: {item_id: input.item_id}
    request: {encoding: form, unknown: ignore}
  - command: shop.items.ReadPrivate
    method: get
    path: /private/{item_id}
    parameters: {item_id: input.item_id}
    request: {encoding: query, unknown: ignore}
  - command: shop.items.StrictOrder
    method: post
    path: /strict/{item_id}
    parameters: {item_id: input.item_id}
    request: {encoding: form}
  - command: shop.items.DropItem
    method: delete
    path: /drop/{item_id}
    parameters: {item_id: input.item_id}
";

const FORM: &str = "application/x-www-form-urlencoded";

const REQUESTS: &[[&str; 7]] = &[
    [
        "put",
        "y",
        "PUT",
        "/items/s1",
        "",
        "",
        r#"{"title":"north"}"#,
    ],
    [
        "put-echo",
        "y",
        "PUT",
        "/items/s1",
        "",
        "",
        r#"{"title":"north","item_id":"s1"}"#,
    ],
    [
        "put-other",
        "y",
        "PUT",
        "/items/s1",
        "",
        "",
        r#"{"title":"north","item_id":"s2"}"#,
    ],
    [
        "put-extra",
        "y",
        "PUT",
        "/items/s1",
        "",
        "",
        r#"{"title":"north","origin":"web"}"#,
    ],
    [
        "label-nested",
        "y",
        "PUT",
        "/labels/s1",
        "",
        "",
        r#"{"label":{"text":"x","extra":1}}"#,
    ],
    [
        "label-echo-extra",
        "y",
        "PUT",
        "/labels/s1",
        "",
        "",
        r#"{"origin":1,"label":{"text":"x"},"item_id":"s1"}"#,
    ],
    [
        "place-whole",
        "y",
        "POST",
        "/places/2",
        "",
        "",
        r#"{"item_id":"i1","count":2.0}"#,
    ],
    [
        "place-other",
        "y",
        "POST",
        "/places/2",
        "",
        "",
        r#"{"item_id":"i1","count":3}"#,
    ],
    [
        "order-echo",
        "y",
        "POST",
        "/orders/s1",
        "",
        FORM,
        "count=2&item_id=s1",
    ],
    [
        "order-other",
        "y",
        "POST",
        "/orders/s1",
        "",
        FORM,
        "count=2&item_id=s2",
    ],
    [
        "order-extra",
        "y",
        "POST",
        "/orders/s1",
        "",
        FORM,
        "count=2&zzz=1",
    ],
    [
        "private-echo",
        "y",
        "GET",
        "/private/s1",
        "level=2&hint=h&item_id=s1",
        "",
        "",
    ],
    [
        "private-other",
        "y",
        "GET",
        "/private/s1",
        "level=2&hint=h&item_id=s%32",
        "",
        "",
    ],
    [
        "strict-echo",
        "y",
        "POST",
        "/strict/s1",
        "",
        FORM,
        "count=1&item_id=s1",
    ],
    [
        "drop-echo",
        "y",
        "DELETE",
        "/drop/s1",
        "",
        "",
        r#"{"item_id":"s1"}"#,
    ],
];

/// The servers [`both`] asks: `[rust, go]`.
pub(super) fn servers() -> Vec<Server> {
    let ir = ir_of(&model());
    let bound = shared::binding(&ir, ROUTES);
    let synthesis = |target| -> Synthesis {
        synthesize_bound(&ir, target, OutputLayout::Workspace, Some(&bound))
            .unwrap_or_else(|failure| panic!("synthesizes for {target:?}: {failure}"))
    };
    vec![
        Server::rust("w752ignore", &synthesis(Target::Rust), shared::RUST_HARNESS),
        Server::go("w752ignore", &synthesis(Target::Go), super::HARNESS),
    ]
}

/// Both servers' answers to [`REQUESTS`]: `(rust, go)`, once per test binary.
fn both() -> &'static (BTreeMap<String, Answer>, BTreeMap<String, Answer>) {
    static BOTH: std::sync::OnceLock<(BTreeMap<String, Answer>, BTreeMap<String, Answer>)> =
        std::sync::OnceLock::new();
    BOTH.get_or_init(|| answers(&servers(), REQUESTS))
}

/// The Rust server's answer to `label`, the Go server's held to its status, `Content-Type`, body
/// bytes and headers.
fn answered(label: &str) -> &'static Answer {
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
            rust.content_type.as_str(),
            rust.body.as_str(),
            &rust.headers
        ),
        (
            go.status,
            go.content_type.as_str(),
            go.body.as_str(),
            &go.headers
        ),
        "{label}: the servers disagree:\n  rust {rust:?}\n  go   {go:?}"
    );
    rust
}

/// `label` answered the default `malformed`, naming `detail`.
fn malformed(label: &str, detail: &str) {
    let answer = answered(label);
    assert_eq!(answer.status, 400, "{label}: {answer:?}");
    assert!(answer.body.contains(detail), "{label}: {answer:?}");
}

#[test]
fn a_json_body_under_ignore_drops_what_no_field_names_and_an_echo_of_the_path() {
    let put = answered("put");
    assert_eq!(put.status, 202, "{put:?}");
    for label in ["put-echo", "put-extra"] {
        let answer = answered(label);
        assert_eq!(
            (answer.status, answer.body.as_str()),
            (put.status, put.body.as_str()),
            "{label}: answered as `put`"
        );
    }
    let other = answered("put-other");
    assert_eq!(other.status, 422, "the route's own `malformed`: {other:?}");
    assert_eq!(
        other.json(),
        serde_json::json!({"error": "invalid_request"})
    );

    let relabelled = answered("label-echo-extra");
    assert_eq!(relabelled.status, 202, "{relabelled:?}");
    malformed(
        "label-nested",
        "body.label.extra: expected no member: no field is named so",
    );

    // A number the path carries is compared as the integer it denotes.
    let place = answered("place-whole");
    assert!(place.status < 300, "{place:?}");
    malformed(
        "place-other",
        "body.count: the path carries this field with another value",
    );
}

#[test]
fn a_form_or_query_key_the_path_carries_must_hold_the_path_value_under_ignore() {
    for label in ["order-echo", "order-extra", "private-echo"] {
        let answer = answered(label);
        assert_eq!(answer.status, 202, "{label}: {answer:?}");
    }
    malformed(
        "order-other",
        "body.item_id: the path carries this field with another value",
    );
    malformed(
        "private-other",
        "query.item_id: the path carries this field with another value",
    );
}

#[test]
fn under_refuse_a_key_or_member_the_path_carries_stays_malformed() {
    malformed(
        "strict-echo",
        "body.item_id: the path carries this field, so it is not a parameter here",
    );
    malformed(
        "drop-echo",
        "body.item_id: the path carries this field, so the request does not",
    );
}
