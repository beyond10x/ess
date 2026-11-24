//! A member no declaration names, below the first level of a request body, sent to both generated
//! servers (`story:the-go-server-refuses-an-undeclared-member-at-any-depth`): in a list element, in
//! a struct two levels down and beside a union's tag, in an unbound command's body (`TagItem`, its
//! derived route), a bound wrapped body (`TagMore`) and a bound bare body (`TagOne`). A struct
//! declaring `undeclared_fields: ignored` (`Loose`) admits one, and still refuses one in a struct
//! it holds.
//!
//! The binding declares no `malformed` answer, so each refusal's detail names the member's path.

use std::collections::BTreeMap;

use ess_synth::{synthesize_bound, OutputLayout, Synthesis, Target};

use super::super::shared::{self, Answer};
use super::adversary_k6_pass1::{go_run, ir_of, rust_run};

/// The fields `TagItem` and `TagMore` both take.
const TAGGED: &str = "      - {name: item_id, type: shop.items.ItemId}
      - {name: tag, type: shop.items.Tag}
      - {name: labels, type: List<shop.items.Label>}
      - {name: loose, type: Optional<shop.items.Loose>}
      - {name: mark, type: Optional<shop.items.Mark>}
";

/// The outcome every added command answers.
const TOUCHED: &str = "    outcomes:
      - name: tagged
        emits: [shop.items.Touched]
        payload:
          shop.items.Touched: {item_id: input.item_id}
";

fn model() -> String {
    let edits = [
        // `undeclared_fields` is `ess/24`.
        ("format: ess/22\n".to_owned(), "format: ess/24\n".to_owned()),
        (
            "  - {name: shop.items.ReturnUri, kind: newtype, of: String}\n".to_owned(),
            "  - {name: shop.items.ReturnUri, kind: newtype, of: String}
  - name: shop.items.Label
    kind: struct
    fields:
      - {name: text, type: String}
  - name: shop.items.Tag
    kind: struct
    fields:
      - {name: label, type: shop.items.Label}
  - name: shop.items.Loose
    kind: struct
    undeclared_fields: ignored
    fields:
      - {name: label, type: shop.items.Label}
  - name: shop.items.Mark
    kind: union
    tag: kind
    variants:
      labelled: shop.items.Label
"
            .to_owned(),
        ),
        (
            "  - name: shop.items.Purge\n".to_owned(),
            format!(
                "  - name: shop.items.TagItem\n    input:\n{TAGGED}{TOUCHED}  - name: \
                 shop.items.TagMore\n    input:\n{TAGGED}{TOUCHED}  - name: shop.items.TagOne
    input:
      - {{name: item_id, type: shop.items.ItemId}}
      - {{name: tags, type: List<shop.items.Tag>}}
{TOUCHED}  - name: shop.items.Purge\n"
            ),
        ),
        (
            "      - shop.items.Approve\ncomponents:\n".to_owned(),
            "      - shop.items.Approve
      - shop.items.TagItem
      - shop.items.TagMore
      - shop.items.TagOne
components:
"
            .to_owned(),
        ),
        (
            "        - shop.items.Archive\n".to_owned(),
            "        - shop.items.Archive
        - shop.items.TagItem
        - shop.items.TagMore
        - shop.items.TagOne
"
            .to_owned(),
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

/// `TagItem` has no route, so it is served on its derived one.
const ROUTES: &str = "routes:
  - command: shop.items.TagMore
    method: post
    path: /tags/{item_id}
    parameters: {item_id: input.item_id}
  - command: shop.items.TagOne
    method: put
    path: /tags/{item_id}/one
    parameters: {item_id: input.item_id}
    request: {encoding: json, body: input.tags}
";

const ITEM: &str = "/items/commands/TagItem";
const MORE: &str = "/tags/a";
const ONE: &str = "/tags/a/one";

const REQUESTS: &[[&str; 7]] = &[
    [
        "item-ok",
        "y",
        "POST",
        ITEM,
        "",
        "",
        r#"{"item_id":"a","tag":{"label":{"text":"x"}},"labels":[{"text":"y"}]}"#,
    ],
    [
        "item-element",
        "y",
        "POST",
        ITEM,
        "",
        "",
        r#"{"item_id":"a","tag":{"label":{"text":"x"}},"labels":[{"text":"y","extra":1}]}"#,
    ],
    [
        "item-deep",
        "y",
        "POST",
        ITEM,
        "",
        "",
        r#"{"item_id":"a","tag":{"label":{"text":"x","extra":1}},"labels":[]}"#,
    ],
    [
        "item-loose",
        "y",
        "POST",
        ITEM,
        "",
        "",
        r#"{"item_id":"a","tag":{"label":{"text":"x"}},"labels":[],"loose":{"label":{"text":"z"},"extra":1}}"#,
    ],
    [
        "item-loose-deep",
        "y",
        "POST",
        ITEM,
        "",
        "",
        r#"{"item_id":"a","tag":{"label":{"text":"x"}},"labels":[],"loose":{"label":{"text":"z","extra":1}}}"#,
    ],
    [
        "item-mark",
        "y",
        "POST",
        ITEM,
        "",
        "",
        r#"{"item_id":"a","tag":{"label":{"text":"x"}},"labels":[],"mark":{"kind":"labelled","value":{"text":"z"},"extra":1}}"#,
    ],
    [
        "item-mark-ok",
        "y",
        "POST",
        ITEM,
        "",
        "",
        r#"{"item_id":"a","tag":{"label":{"text":"x"}},"labels":[],"mark":{"kind":"labelled","value":{"text":"z"}}}"#,
    ],
    [
        "more-ok",
        "y",
        "POST",
        MORE,
        "",
        "",
        r#"{"tag":{"label":{"text":"x"}},"labels":[{"text":"y"}]}"#,
    ],
    [
        "more-element",
        "y",
        "POST",
        MORE,
        "",
        "",
        r#"{"tag":{"label":{"text":"x"}},"labels":[{"text":"y","extra":1}]}"#,
    ],
    [
        "more-deep",
        "y",
        "POST",
        MORE,
        "",
        "",
        r#"{"tag":{"label":{"text":"x","extra":1}},"labels":[]}"#,
    ],
    [
        "more-loose",
        "y",
        "POST",
        MORE,
        "",
        "",
        r#"{"tag":{"label":{"text":"x"}},"labels":[],"loose":{"label":{"text":"z"},"extra":1}}"#,
    ],
    [
        "one-ok",
        "y",
        "PUT",
        ONE,
        "",
        "",
        r#"[{"label":{"text":"x"}}]"#,
    ],
    [
        "one-deep",
        "y",
        "PUT",
        ONE,
        "",
        "",
        r#"[{"label":{"text":"x","extra":1}}]"#,
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
            rust_run("w752deep", &synthesis(Target::Rust), REQUESTS),
            go_run("w752deep", &synthesis(Target::Go), REQUESTS),
        )
    })
}

/// Each server's answer to `label`, held to one status: `[rust, go]`.
fn answered(label: &str) -> [&'static Answer; 2] {
    let (rust, go) = both();
    let rust = rust
        .get(label)
        .unwrap_or_else(|| panic!("the Rust server answered `{label}`"));
    let go = go
        .get(label)
        .unwrap_or_else(|| panic!("the Go server answered `{label}`"));
    assert_eq!(
        rust.status, go.status,
        "{label}: the servers disagree:\n  rust {rust:?}\n  go   {go:?}"
    );
    [rust, go]
}

/// `label` is refused by both servers, the Go server naming `at`.
fn refused_at(label: &str, at: &str) {
    let [rust, go] = answered(label);
    assert_eq!(rust.status, 400, "{label}: {rust:?}");
    assert!(
        go.body
            .contains(&format!("{at}: expected no member: no field is named so")),
        "{label}: the Go server names {at}: {go:?}"
    );
}

#[test]
fn the_go_server_refuses_an_undeclared_member_at_any_depth() {
    for label in ["item-ok", "more-ok", "one-ok", "item-mark-ok"] {
        let [rust, _] = answered(label);
        assert_eq!(rust.status, 202, "{label}: {rust:?}");
    }
    for (label, at) in [
        ("item-element", "body.labels[0].extra"),
        ("item-deep", "body.tag.label.extra"),
        ("item-mark", "body.mark.extra"),
        ("more-element", "body.labels[0].extra"),
        ("more-deep", "body.tag.label.extra"),
        // A bare body is read as the input member it is, so the path names that member.
        ("one-deep", "body.tags[0].label.extra"),
    ] {
        refused_at(label, at);
    }
}

#[test]
fn a_struct_that_ignores_undeclared_fields_admits_one_and_refuses_one_in_what_it_holds() {
    for label in ["item-loose", "more-loose"] {
        let [rust, _] = answered(label);
        assert_eq!(rust.status, 202, "{label}: {rust:?}");
    }
    refused_at("item-loose-deep", "body.loose.label.extra");
}
