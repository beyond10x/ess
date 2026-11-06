//! Adversary pass on unit U2 of the 0.89.0 wave, against both generated servers: a bare request
//! body (`story:a-command-route-takes-one-input-as-its-bare-request-body`) at its boundaries, and
//! a literal segment beside a variable
//! (`story:a-literal-path-segment-takes-precedence-over-a-variable-one`) reached by a path that
//! spells the literal with a percent-encoded unreserved character.
//!
//! The model and routes are `board_routes.rs`'s, copied: that module's helpers are private.

use std::collections::BTreeMap;

use ess_synth::{synthesize_bound, OutputLayout, Synthesis, Target};

use super::super::shared::{self, Answer};
use super::adversary_k6_pass1::{go_run, ir_of, rust_run};

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

const PRIORITY: &str = "/boards/b-1/cards/priority";

const REQUESTS: &[[&str; 7]] = &[
    ["bare-empty-list", "y", "PUT", PRIORITY, "", "", "[]"],
    [
        "bare-spaced-empty-list",
        "y",
        "PUT",
        PRIORITY,
        "",
        "",
        "  [ ]  ",
    ],
    ["bare-null", "y", "PUT", PRIORITY, "", "", "null"],
    ["bare-wrong-element", "y", "PUT", PRIORITY, "", "", "[1]"],
    ["bare-text", "y", "PUT", PRIORITY, "", "", r#""c-1""#],
    ["bare-trailing", "y", "PUT", PRIORITY, "", "", "[] x"],
    ["bare-two-documents", "y", "PUT", PRIORITY, "", "", "[][]"],
    [
        "bare-extra-member",
        "y",
        "PUT",
        PRIORITY,
        "",
        "",
        r#"[{"id":"c-1","priority":1,"extra":true}]"#,
    ],
    [
        "bare-text-plain",
        "y",
        "PUT",
        PRIORITY,
        "",
        "text/plain",
        r#"[{"id":"c-1","priority":1}]"#,
    ],
    [
        "bare-board-member",
        "y",
        "PUT",
        PRIORITY,
        "",
        "",
        r#"{"board_id":"b-2","priorities":[]}"#,
    ],
    ["literal-other-method", "y", "POST", PRIORITY, "", "", "[]"],
    [
        "tilde-plain",
        "y",
        "PUT",
        "/boards/b-1/cards/~archived",
        "",
        "",
        r#"{"title":"T"}"#,
    ],
    [
        "tilde-encoded",
        "y",
        "PUT",
        "/boards/b-1/cards/%7Earchived",
        "",
        "",
        r#"{"title":"T"}"#,
    ],
    [
        "priority-encoded",
        "y",
        "PUT",
        "/boards/b-1/cards/%70riority",
        "",
        "",
        r#"[{"id":"c-1","priority":1}]"#,
    ],
    [
        "priority-plain",
        "y",
        "PUT",
        PRIORITY,
        "",
        "",
        r#"[{"id":"c-1","priority":1}]"#,
    ],
];

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

fn both() -> &'static (BTreeMap<String, Answer>, BTreeMap<String, Answer>) {
    static BOTH: std::sync::OnceLock<(BTreeMap<String, Answer>, BTreeMap<String, Answer>)> =
        std::sync::OnceLock::new();
    BOTH.get_or_init(|| {
        let routes = format!("{}{BOARD_ROUTES}", shared::ROUTES);
        let ir = ir_of(&model());
        let bound = shared::binding(&ir, &routes);
        let rust: Synthesis =
            synthesize_bound(&ir, Target::Rust, OutputLayout::Workspace, Some(&bound))
                .unwrap_or_else(|failure| panic!("synthesizes for Rust: {failure}"));
        let go: Synthesis =
            synthesize_bound(&ir, Target::Go, OutputLayout::Workspace, Some(&bound))
                .unwrap_or_else(|failure| panic!("synthesizes for Go: {failure}"));
        (
            rust_run("w089u2", &rust, REQUESTS),
            go_run("w089u2", &go, REQUESTS),
        )
    })
}

/// Each server's answer to `label`, held to one status and one JSON body.
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

fn malformed(label: &str) {
    for answer in answered(label) {
        assert_eq!(answer.status, 400, "{label}: {answer:?}");
        assert_eq!(
            answer.json(),
            serde_json::json!({"error": "invalid_request"}),
            "{label}: {answer:?}"
        );
    }
}

/// An empty list is a list: the bare body `[]` reaches the command with no priorities, with or
/// without whitespace around it.
#[test]
fn adv_a_bare_empty_list_reaches_the_command() {
    for label in ["bare-empty-list", "bare-spaced-empty-list"] {
        for answer in answered(label) {
            assert_eq!(answer.status, 202, "{label}: {answer:?}");
            assert_eq!(
                shared::published(answer),
                serde_json::json!({"board_id": "b-1", "priorities": []}),
                "{label}: {answer:?}"
            );
        }
    }
}

/// `null`, a list of the wrong element, a text, trailing bytes and two documents are not the list
/// the body is.
#[test]
fn adv_a_bare_body_that_is_not_the_list_is_malformed() {
    for label in [
        "bare-null",
        "bare-wrong-element",
        "bare-text",
        "bare-trailing",
        "bare-two-documents",
    ] {
        malformed(label);
    }
}

/// An object holding the path's field and the list is not the list: the body is the list alone,
/// so the board in it does not reach the command.
#[test]
fn adv_a_bare_body_object_naming_the_path_field_is_malformed() {
    malformed("bare-board-member");
}

/// A member the list's element type does not declare is answered alike by both servers, in a bare
/// body: `malformed`, as the Rust server answers. The Go server's `bareBody` hands the value to
/// the decoder without `jsonBody`'s member check, and the Go decoder does not refuse a nested
/// member no field names.
#[test]
fn adv_the_servers_agree_on_an_extra_member_in_a_bare_body() {
    let (rust, go) = both();
    let [rust, go] = [&rust["bare-extra-member"], &go["bare-extra-member"]];
    assert_eq!(
        (go.status, go.json()),
        (rust.status, rust.json()),
        "defect: the Go server runs the command on a bare body whose element holds a member the \
         element type does not declare, which the Rust server answers `malformed`:\n  rust \
         {rust:?}\n  go   {go:?}"
    );
}

/// A body labelled `text/plain` is answered alike by both servers.
#[test]
fn adv_the_servers_agree_on_a_bare_body_not_labelled_json() {
    let _ = answered("bare-text-plain");
}

/// Two rows of `PUT` match the literal's path; another method is one `405` naming `PUT` once.
#[test]
fn adv_another_method_on_the_literal_path_is_one_405() {
    for answer in answered("literal-other-method") {
        assert_eq!(answer.status, 405, "{answer:?}");
        assert_eq!(answer.header("Allow"), Some("PUT"), "{answer:?}");
    }
}

/// RFC 3986 §6.2.2.2 and RFC 9110 §4.2.3: a percent-encoded unreserved character is the character
/// itself, so `/cards/%7Earchived` is the path `/cards/~archived`, where the literal route takes
/// precedence. Both servers match a literal against the path as spelled, so the encoded spelling
/// falls through to the variable route, which runs `UpdateCard` on a card whose id is the literal
/// its sibling route claims. Before this unit no variable route had a literal sibling, and the
/// spelled match failed closed with a `404`.
#[test]
fn adv_a_literal_spelled_with_a_percent_encoded_unreserved_character_takes_precedence() {
    let mut defects = Vec::new();
    for (plain_label, encoded_label) in [
        ("tilde-plain", "tilde-encoded"),
        ("priority-plain", "priority-encoded"),
    ] {
        let spelled = REQUESTS
            .iter()
            .find(|request| request[0] == encoded_label)
            .map_or("", |request| request[3]);
        let [rust_plain, go_plain] = answered(plain_label);
        let [rust_encoded, go_encoded] = answered(encoded_label);
        for (target, plain, encoded) in [
            ("rust", rust_plain, rust_encoded),
            ("go", go_plain, go_encoded),
        ] {
            if (encoded.status, encoded.json()) != (plain.status, plain.json()) {
                defects.push(format!(
                    "{target} `{spelled}`:\n    plain   {plain:?}\n    encoded {encoded:?}"
                ));
            }
        }
    }
    assert!(
        defects.is_empty(),
        "defect: a path spelling the literal with a percent-encoded unreserved character is the \
         literal's own path, and the variable route served it instead of the literal route that \
         takes precedence there:\n  {}",
        defects.join("\n  ")
    );
}
