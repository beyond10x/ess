//! Two outcomes of one command answering `500` with `{error: error}` over two errors of one wire
//! name, one of which always writes a field the other never writes (`ESS-HTTP-025`,
//! `docs/design/http-presentation-binding.md`, "Status per outcome"), sent to both generated
//! servers: each outcome answers with its own fields.
//!
//! The shared model gains `Rename`, whose `boom` reports an error with the field `message` and
//! whose `boom-more` reports one of the same wire name with `message` and `details`; the shared
//! routes gain a route for it that binds the pair on one status without `alike:`.

use std::collections::BTreeMap;

use ess_synth::{synthesize_bound, OutputLayout, Synthesis, Target};

use super::super::shared::{self, Answer};
use super::adversary_k6_pass1::{go_run, ir_of, rust_run};

/// `(anchor, replacement)` in the shared model, each anchor found exactly once.
const MODEL_EDITS: &[(&str, &str)] = &[
    (
        "errors:\n",
        "errors:
  - name: shop.items.Boom
    naming: {wire: internal_error}
    fields:
      - {name: message, type: String}
  - name: shop.items.BoomMore
    naming: {wire: internal_error}
    fields:
      - {name: message, type: String}
      - {name: details, type: String}
",
    ),
    (
        "views:\n  - name: shop.items.ItemById\n",
        "  - name: shop.items.Rename
    input:
      - {name: item_id, type: shop.items.ItemId}
      - {name: count, type: Integer}
      - {name: message, type: String}
      - {name: details, type: String}
    outcomes:
      - name: boom-more
        when: count > 9
        error: shop.items.BoomMore
        payload:
          shop.items.BoomMore: {message: input.message, details: input.details}
      - name: boom
        when: count > 5
        error: shop.items.Boom
        payload:
          shop.items.Boom: {message: input.message}
      - name: renamed
        emits: [shop.items.Touched]
        payload:
          shop.items.Touched: {item_id: input.item_id}
views:
  - name: shop.items.ItemById
",
    ),
    (
        "      - shop.items.Approve\ncomponents:\n",
        "      - shop.items.Approve\n      - shop.items.Rename\ncomponents:\n",
    ),
    (
        "        - shop.items.Archive\n    publishes:\n",
        "        - shop.items.Archive\n        - shop.items.Rename\n    publishes:\n",
    ),
];

/// The route of `Rename`, after the shared routes.
const ROUTE: &str = "  - command: shop.items.Rename
    method: put
    path: /api/items/{item_id}
    parameters: {item_id: input.item_id}
    outcomes:
      boom: {status: 500, body: {error: error}}
      boom-more: {status: 500, body: {error: error}}
      renamed: {status: 200, body: none}
";

const REQUESTS: &[[&str; 7]] = &[
    [
        "boom",
        "y",
        "PUT",
        "/api/items/i1",
        "",
        "",
        r#"{"count":7,"message":"m","details":"d"}"#,
    ],
    [
        "boom-more",
        "y",
        "PUT",
        "/api/items/i1",
        "",
        "",
        r#"{"count":10,"message":"m","details":"d"}"#,
    ],
];

fn synthesized() -> (Synthesis, Synthesis) {
    let mut model = shared::MODEL.to_owned();
    for (anchor, replacement) in MODEL_EDITS {
        assert_eq!(
            model.matches(anchor).count(),
            1,
            "the model moved: {anchor}"
        );
        model = model.replace(anchor, replacement);
    }
    let routes = format!("{}{ROUTE}", shared::ROUTES);
    let ir = ir_of(&model);
    let bound = shared::binding(&ir, &routes);
    let rust = synthesize_bound(&ir, Target::Rust, OutputLayout::Workspace, Some(&bound))
        .unwrap_or_else(|failure| panic!("synthesizes for Rust: {failure}"));
    let go = synthesize_bound(&ir, Target::Go, OutputLayout::Workspace, Some(&bound))
        .unwrap_or_else(|failure| panic!("synthesizes for Go: {failure}"));
    (rust, go)
}

/// Both generated servers answer each outcome of the pair with its own fields, and the document
/// each embeds describes both bodies on the `500` response.
#[test]
fn both_servers_answer_each_outcome_of_one_wire_name_with_its_own_fields() {
    let (rust, go) = synthesized();
    let answers: [(&str, BTreeMap<String, Answer>); 2] = [
        ("rust", rust_run("errwire", &rust, REQUESTS)),
        ("go", go_run("errwire", &go, REQUESTS)),
    ];
    for (target, answered) in &answers {
        let boom = &answered["boom"];
        assert_eq!(boom.status, 500, "{target}: {boom:?}");
        assert_eq!(
            boom.json(),
            serde_json::json!({"error": "internal_error", "message": "m"}),
            "{target}: {boom:?}"
        );
        let more = &answered["boom-more"];
        assert_eq!(more.status, 500, "{target}: {more:?}");
        assert_eq!(
            more.json(),
            serde_json::json!({"error": "internal_error", "message": "m", "details": "d"}),
            "{target}: {more:?}"
        );
    }
    for (target, synthesis) in [("rust", &rust), ("go", &go)] {
        let documents: Vec<serde_json::Value> = synthesis
            .artifacts
            .iter()
            .filter(|(path, _)| path.ends_with(".openapi.json"))
            .map(|(_, artifact)| {
                serde_json::from_str(&artifact.contents).expect("the embedded document is JSON")
            })
            .collect();
        assert_eq!(documents.len(), 1, "{target}");
        let response = &documents[0]["paths"]["/api/items/{item_id}"]["put"]["responses"]["500"];
        assert_eq!(
            response["x-ess-outcomes"],
            serde_json::json!(["boom-more", "boom"]),
            "{target}: {response}"
        );
        let schemas = response["content"]["application/json"]["schema"]["oneOf"]
            .as_array()
            .unwrap_or_else(|| panic!("{target}: both bodies on the response: {response}"));
        assert_eq!(schemas.len(), 2, "{target}: {response}");
    }
}
