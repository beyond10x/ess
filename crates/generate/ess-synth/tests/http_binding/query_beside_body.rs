//! The fields a command route's `request.query` lists, read from the query string beside the body
//! by the generated Rust and Go servers (beyond10x/ess#754,
//! `story:generated-servers-read-chosen-inputs-from-the-query-beside-the-body`): a listed field is
//! read under its wire name as `encoding: query` reads it, the body naming it is `malformed`, and
//! a query key no listed field names is `malformed` under `refuse` and ignored under `ignore`. The
//! Go server answers every request with the Rust server's bytes.

use std::collections::BTreeMap;

use ess_synth::{synthesize_bound, OutputLayout, Synthesis, Target};

use super::super::shared::{self, Answer};
use super::adversary_k6_pass1::{go_run, ir_of, rust_run};

/// The shop gains two commands taking a `format` in the body and a `dry_run` (wire name `dryRun`)
/// and an optional `note` the routes read from the query string, each emitting an event that
/// carries every input field, so the answer's envelope shows what the server read.
fn model() -> String {
    let input = "      - {name: format, type: String}
      - {name: dry_run, type: Boolean, wire: dryRun}
      - {name: note, type: Optional<String>}
";
    let payload = "{format: input.format, dry_run: input.dry_run, note: input.note}";
    let edits = [
        (
            "events:\n  - name: shop.items.Put\n".to_owned(),
            format!(
                "events:
  - name: shop.items.Restocked
    fields:
{input}  - name: shop.items.Put\n"
            ),
        ),
        (
            "  - name: shop.items.Purge\n".to_owned(),
            format!(
                "  - name: shop.items.Restock
    input:
{input}    outcomes:
      - name: restocked
        emits: [shop.items.Restocked]
        payload:
          shop.items.Restocked: {payload}
  - name: shop.items.Recount
    input:
{input}    outcomes:
      - name: recounted
        emits: [shop.items.Restocked]
        payload:
          shop.items.Restocked: {payload}
  - name: shop.items.Purge\n"
            ),
        ),
        (
            "      - shop.items.Approve\ncomponents:\n".to_owned(),
            "      - shop.items.Approve
      - shop.items.Restock
      - shop.items.Recount
components:\n"
                .to_owned(),
        ),
        (
            "        - shop.items.Archive\n".to_owned(),
            "        - shop.items.Archive
        - shop.items.Restock
        - shop.items.Recount\n"
                .to_owned(),
        ),
        (
            "shop.items.Searched, shop.items.Approved]".to_owned(),
            "shop.items.Searched, shop.items.Approved, shop.items.Restocked]".to_owned(),
        ),
    ];
    let mut model = shared::MODEL.to_owned();
    for (anchor, replacement) in edits {
        assert_eq!(
            model.matches(&anchor).count(),
            1,
            "the model moved: {anchor}"
        );
        model = model.replace(&anchor, &replacement);
    }
    model
}

const ROUTES: &str = "routes:
  - command: shop.items.Restock
    method: post
    path: /restock
    request: {encoding: json, query: [input.dry_run]}
  - command: shop.items.Recount
    method: post
    path: /recount
    request: {encoding: form, unknown: ignore, query: [input.dry_run, input.note]}
";

const JSON: &str = "application/json";
const FORM: &str = "application/x-www-form-urlencoded";

const REQUESTS: &[[&str; 7]] = &[
    [
        "json",
        "y",
        "POST",
        "/restock",
        "dryRun=true",
        JSON,
        r#"{"format":"csv"}"#,
    ],
    [
        "json-absent",
        "y",
        "POST",
        "/restock",
        "",
        JSON,
        r#"{"format":"csv"}"#,
    ],
    [
        "json-in-body",
        "y",
        "POST",
        "/restock",
        "dryRun=true",
        JSON,
        r#"{"format":"csv","dryRun":true}"#,
    ],
    [
        "json-unknown",
        "y",
        "POST",
        "/restock",
        "dryRun=true&origin=web",
        JSON,
        r#"{"format":"csv"}"#,
    ],
    [
        "json-twice",
        "y",
        "POST",
        "/restock",
        "dryRun=true&dryRun=false",
        JSON,
        r#"{"format":"csv"}"#,
    ],
    [
        "json-model-name",
        "y",
        "POST",
        "/restock",
        "dry_run=true",
        JSON,
        r#"{"format":"csv"}"#,
    ],
    [
        "form",
        "y",
        "POST",
        "/recount",
        "dryRun=false&origin=web",
        FORM,
        "format=csv",
    ],
    [
        "form-note",
        "y",
        "POST",
        "/recount",
        "note=hi%20there&dryRun=true",
        FORM,
        "format=csv",
    ],
    [
        "form-in-body",
        "y",
        "POST",
        "/recount",
        "dryRun=true",
        FORM,
        "format=csv&dryRun=true",
    ],
];

/// Both servers' answers to [`REQUESTS`]: `(rust, go)`, once per test binary.
fn both() -> &'static (BTreeMap<String, Answer>, BTreeMap<String, Answer>) {
    static BOTH: std::sync::OnceLock<(BTreeMap<String, Answer>, BTreeMap<String, Answer>)> =
        std::sync::OnceLock::new();
    BOTH.get_or_init(|| {
        let ir = ir_of(&model());
        let bound = shared::binding(&ir, ROUTES);
        let synthesis = |target| -> Synthesis {
            synthesize_bound(&ir, target, OutputLayout::Workspace, Some(&bound))
                .unwrap_or_else(|failure| panic!("synthesizes for {target:?}: {failure}"))
        };
        (
            rust_run("w754query", &synthesis(Target::Rust), REQUESTS),
            go_run("w754query", &synthesis(Target::Go), REQUESTS),
        )
    })
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

/// `label` ran its command, and its answer holds each of `members`.
fn ran(label: &str, members: &[&str]) {
    let answer = answered(label);
    assert!(answer.status < 300, "{label}: {answer:?}");
    for member in members {
        assert!(
            answer.body.contains(member),
            "{label}: {member} in {answer:?}"
        );
    }
}

#[test]
fn a_listed_field_is_read_from_the_query_beside_the_body() {
    ran("json", &[r#""format":"csv""#, r#""dryRun":true"#]);
    ran("form", &[r#""format":"csv""#, r#""dryRun":false"#]);
    assert!(
        !answered("form").body.contains(r#""note""#),
        "an absent optional stays absent: {:?}",
        answered("form")
    );
    ran("form-note", &[r#""note":"hi there""#, r#""dryRun":true"#]);
}

#[test]
fn a_listed_field_out_of_its_place_or_its_rules_is_malformed() {
    // Required and absent: the decoder refuses it.
    assert_eq!(answered("json-absent").status, 400);
    malformed(
        "json-in-body",
        "body.dryRun: the query carries this field, so the body does not",
    );
    malformed(
        "form-in-body",
        "body.dryRun: the query carries this field, so the body does not",
    );
    malformed(
        "json-unknown",
        "query.origin: no field of this request is named so",
    );
    malformed(
        "json-model-name",
        "query.dry_run: no field of this request is named so",
    );
    malformed("json-twice", "query.dryRun: expected one value, found 2");
}

#[test]
fn the_go_server_answers_every_query_request_as_the_rust_server_does() {
    for [label, ..] in REQUESTS {
        answered(label);
    }
}
