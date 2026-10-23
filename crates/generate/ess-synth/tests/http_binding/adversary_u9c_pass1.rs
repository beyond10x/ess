//! Adversary, unit U9c, pass 1: a `delete` command encoded `query`
//! (`story:a-delete-command-may-take-its-input-from-the-query-string`), every flat kind on its
//! query string, sent to both generated servers, and the `OpenAPI` document both embed.
//!
//! The fixture gains `Forget` (a path identity, a required `Integer`, a wire-named
//! `Optional<String>`, a `List<String>` and a required enum) on `delete /forget/{item_id}`, and
//! `Unsearch`, `Search`'s input on `delete /search`, the path `Search` already holds on `get`.

use std::collections::BTreeMap;

use ess_synth::{synthesize_bound, OutputLayout, Synthesis, Target};

use super::super::shared::{self, Answer};
use super::adversary_k6_pass1::{go_run, ir_of, rust_run};

const EDITS: &[(&str, &str)] = &[
    (
        "  - {name: shop.items.ReturnUri, kind: newtype, of: String}\n",
        "  - {name: shop.items.ReturnUri, kind: newtype, of: String}
  - {name: shop.items.Mode, kind: enum, variants: [fast, slow]}
",
    ),
    (
        "  - name: shop.items.Approved\n",
        "  - name: shop.items.Forgotten
    fields:
      - {name: item_id, type: shop.items.ItemId}
      - {name: count, type: Integer}
      - {name: note, type: Optional<String>}
      - {name: tags, type: List<String>}
      - {name: mode, type: shop.items.Mode}
  - name: shop.items.Unsearched
    fields:
      - {name: words, type: List<String>}
      - {name: limit, type: Integer}
  - name: shop.items.Approved\n",
    ),
    (
        "  - name: shop.items.Purge\n",
        "  - name: shop.items.Forget
    input:
      - {name: item_id, type: shop.items.ItemId}
      - {name: count, type: Integer}
      - {name: note, type: Optional<String>, wire: noteText}
      - {name: tags, type: List<String>}
      - {name: mode, type: shop.items.Mode}
    outcomes:
      - name: forgotten
        emits: [shop.items.Forgotten]
        payload:
          shop.items.Forgotten: {item_id: input.item_id, count: input.count, note: input.note, tags: input.tags, mode: input.mode}
  - name: shop.items.Unsearch
    input:
      - {name: words, type: List<String>}
      - {name: limit, type: Integer}
    outcomes:
      - name: unsearched
        emits: [shop.items.Unsearched]
        payload:
          shop.items.Unsearched: {words: input.words, limit: input.limit}
  - name: shop.items.Purge\n",
    ),
    (
        "      - shop.items.Approve\ncomponents:",
        "      - shop.items.Approve
      - shop.items.Forget
      - shop.items.Unsearch
components:",
    ),
    (
        "        - shop.items.Archive\n",
        "        - shop.items.Archive
        - shop.items.Forget
        - shop.items.Unsearch
",
    ),
    (
        "shop.items.Searched, shop.items.Approved]",
        "shop.items.Searched, shop.items.Approved, shop.items.Forgotten, shop.items.Unsearched]",
    ),
];

const ROUTES: &str = "  - command: shop.items.Forget
    method: delete
    path: /forget/{item_id}
    parameters: {item_id: input.item_id}
    request: {encoding: query}
  - command: shop.items.Unsearch
    method: delete
    path: /search
    request: {encoding: query}
";

const REQUESTS: &[[&str; 7]] = &[
    [
        "all",
        "y",
        "DELETE",
        "/forget/w1",
        "count=3&noteText=a%20b+c&tags=x&tags=y%26z&mode=fast",
        "",
        "",
    ],
    [
        "no-optional",
        "y",
        "DELETE",
        "/forget/w1",
        "count=3&mode=fast",
        "",
        "",
    ],
    [
        "model-name",
        "y",
        "DELETE",
        "/forget/w1",
        "count=3&mode=fast&note=x",
        "",
        "",
    ],
    [
        "scalar-twice",
        "y",
        "DELETE",
        "/forget/w1",
        "count=3&count=4&mode=fast",
        "",
        "",
    ],
    [
        "not-integer",
        "y",
        "DELETE",
        "/forget/w1",
        "count=x&mode=fast",
        "",
        "",
    ],
    [
        "missing-required",
        "y",
        "DELETE",
        "/forget/w1",
        "mode=fast",
        "",
        "",
    ],
    [
        "empty-optional",
        "y",
        "DELETE",
        "/forget/w1",
        "count=3&mode=fast&noteText=",
        "",
        "",
    ],
    [
        "empty-integer",
        "y",
        "DELETE",
        "/forget/w1",
        "count=&mode=fast",
        "",
        "",
    ],
    [
        "path-in-query",
        "y",
        "DELETE",
        "/forget/w1",
        "count=3&mode=fast&item_id=w2",
        "",
        "",
    ],
    [
        "bad-enum",
        "y",
        "DELETE",
        "/forget/w1",
        "count=3&mode=medium",
        "",
        "",
    ],
    [
        "encoded-key",
        "y",
        "DELETE",
        "/forget/w1",
        "cou%6Et=3&mode=fast",
        "",
        "",
    ],
    [
        "negative",
        "y",
        "DELETE",
        "/forget/w1",
        "count=-7&mode=fast",
        "",
        "",
    ],
    [
        "huge-integer",
        "y",
        "DELETE",
        "/forget/w1",
        "count=99999999999999999999&mode=fast",
        "",
        "",
    ],
    [
        "decimal-integer",
        "y",
        "DELETE",
        "/forget/w1",
        "count=1.5&mode=fast",
        "",
        "",
    ],
    [
        "empty-tag",
        "y",
        "DELETE",
        "/forget/w1",
        "count=3&mode=fast&tags=",
        "",
        "",
    ],
    [
        "body-only",
        "y",
        "DELETE",
        "/forget/w1",
        "",
        "application/json",
        r#"{"count":3,"mode":"fast"}"#,
    ],
    [
        "encoded-path",
        "y",
        "DELETE",
        "/forget/a%20b",
        "count=3&mode=fast",
        "",
        "",
    ],
    [
        "bad-escape",
        "y",
        "DELETE",
        "/forget/w1",
        "count=3&mode=fast&noteText=%zz",
        "",
        "",
    ],
    [
        "not-granted",
        "n",
        "DELETE",
        "/forget/w1",
        "count=3&mode=fast",
        "",
        "",
    ],
    [
        "unsearch",
        "y",
        "DELETE",
        "/search",
        "words=a&words=b+c&limit=1",
        "",
        "",
    ],
    [
        "search",
        "y",
        "GET",
        "/search",
        "words=a&words=b+c&limit=1",
        "",
        "",
    ],
];

/// The fixture with [`EDITS`], its routes with [`ROUTES`], synthesized for both targets.
fn synthesized() -> &'static (Synthesis, Synthesis) {
    static BOTH: std::sync::OnceLock<(Synthesis, Synthesis)> = std::sync::OnceLock::new();
    BOTH.get_or_init(|| {
        let mut model = shared::MODEL.to_owned();
        for (anchor, replacement) in EDITS {
            assert_eq!(
                model.matches(anchor).count(),
                1,
                "the fixture moved: {anchor}"
            );
            model = model.replace(anchor, replacement);
        }
        let ir = ir_of(&model);
        let routes = format!("{}{ROUTES}", shared::ROUTES);
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
            rust_run("u9cadv", rust, REQUESTS),
            go_run("u9cadv", go, REQUESTS),
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

/// The one event `answer` published: its name and payload.
fn event(answer: &Answer) -> (String, serde_json::Value) {
    let body = answer.json();
    let published = body["published"]
        .as_array()
        .unwrap_or_else(|| panic!("an envelope lists what it published: {body}"));
    assert_eq!(published.len(), 1, "{body}");
    (
        published[0]["event"]
            .as_str()
            .unwrap_or_default()
            .to_owned(),
        published[0]["payload"].clone(),
    )
}

fn accepted(label: &str) -> serde_json::Value {
    let [rust, go] = answered(label);
    assert_eq!(rust.status, 202, "{label}: {rust:?}");
    let (_, payload) = event(rust);
    assert_eq!(event(go).1, payload, "{label}");
    payload
}

fn malformed(label: &str) {
    for answer in answered(label) {
        assert_eq!(answer.status, 400, "{label}: {answer:?}");
    }
}

#[test]
fn every_flat_kind_is_read_from_a_delete_query_string_by_both_servers() {
    assert_eq!(
        accepted("all"),
        serde_json::json!({
            "item_id": "w1", "count": 3, "note": "a b c", "tags": ["x", "y&z"], "mode": "fast"
        })
    );
    let none = accepted("no-optional");
    assert_eq!(none["tags"], serde_json::json!([]), "{none}");
    assert!(
        none.get("note").is_none_or(serde_json::Value::is_null),
        "{none}"
    );
    assert_eq!(accepted("encoded-key")["count"], 3);
    assert_eq!(accepted("negative")["count"], -7);
    assert_eq!(accepted("empty-optional")["note"], "");
    assert_eq!(accepted("empty-tag")["tags"], serde_json::json!([""]));
    assert_eq!(accepted("encoded-path")["item_id"], "a b");
}

#[test]
fn a_delete_query_string_out_of_its_rules_is_malformed_on_both_servers() {
    for label in [
        "model-name",
        "scalar-twice",
        "not-integer",
        "missing-required",
        "empty-integer",
        "path-in-query",
        "bad-enum",
        "huge-integer",
        "decimal-integer",
        "body-only",
        "bad-escape",
    ] {
        malformed(label);
    }
}

#[test]
fn an_ungranted_caller_is_refused_on_a_delete_query_route() {
    for answer in answered("not-granted") {
        assert_eq!(answer.status, 403, "{answer:?}");
    }
}

#[test]
fn get_and_delete_on_one_path_reach_their_own_commands() {
    let [rust, go] = answered("unsearch");
    assert_eq!(rust.status, 202, "{rust:?}");
    for answer in [rust, go] {
        let (name, payload) = event(answer);
        assert_eq!(name, "shop.items.Unsearched", "{answer:?}");
        assert_eq!(
            payload,
            serde_json::json!({"words": ["a", "b c"], "limit": 1})
        );
    }
    let [rust, go] = answered("search");
    assert_eq!(rust.status, 202, "{rust:?}");
    for answer in [rust, go] {
        assert_eq!(event(answer).0, "shop.items.Searched", "{answer:?}");
    }
}

/// The `OpenAPI` document each server embeds describes the `delete` query route as the servers read
/// it: every unmapped field `in: query` under its wire name, required exactly when the input
/// requires it, a `List` exploded, and no body.
#[test]
fn the_embedded_openapi_spells_the_delete_query_route_as_the_servers_read_it() {
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
        let forget = &document["paths"]["/forget/{item_id}"]["delete"];
        assert!(forget.is_object(), "{target}: {}", document["paths"]);
        assert!(forget.get("requestBody").is_none(), "{target}: {forget}");
        let placed: Vec<(String, String, bool, Option<bool>)> = forget["parameters"]
            .as_array()
            .expect("parameters")
            .iter()
            .map(|parameter| {
                (
                    parameter["in"].as_str().unwrap_or_default().to_owned(),
                    parameter["name"].as_str().unwrap_or_default().to_owned(),
                    parameter["required"].as_bool().unwrap_or(false),
                    parameter["explode"].as_bool(),
                )
            })
            .collect();
        let expected: Vec<(String, String, bool, Option<bool>)> = [
            ("path", "item_id", true, None),
            ("query", "count", true, None),
            ("query", "noteText", false, None),
            ("query", "tags", false, Some(true)),
            ("query", "mode", true, None),
        ]
        .into_iter()
        .map(|(location, name, required, explode)| {
            (location.to_owned(), name.to_owned(), required, explode)
        })
        .collect();
        assert_eq!(placed, expected, "{target}: {forget}");
        let unsearch = &document["paths"]["/search"]["delete"];
        assert!(
            unsearch.get("requestBody").is_none(),
            "{target}: {unsearch}"
        );
        let search = &document["paths"]["/search"]["get"];
        assert_eq!(
            unsearch["parameters"], search["parameters"],
            "{target}: one input on one path, one query surface"
        );
    }
}
