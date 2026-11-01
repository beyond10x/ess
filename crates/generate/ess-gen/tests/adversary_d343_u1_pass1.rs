//! D343 U1 adversary pass 1: `ess generate types --http` against its acceptance.
//!
//! The unit's own cases bind one path variable to an unrenamed field of a flat input. These bind
//! two path variables, one to a wire-renamed field, beside a nested struct with a renamed member
//! and an optional field, under the default `json` and under `form` encoding, and a command that
//! declares no input field: each `<command>.input` must equal the bound `{command}.Input`, decode
//! the body the bound document publishes, and refuse a body carrying a path field under its wire
//! name; a command with no body under the binding is refused for the reason that holds.

use std::collections::BTreeSet;

use ess_compiler::{resolve::compile, source::SourceMap, EssIr};
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source,
};
use ess_gen::http::{derived, Binding};
use ess_gen::schema::ModelTypes;
use serde_json::{json, Value};

const SOURCE: &str = "format: ess/22
system: demo
version: v1
domain: demo.board
types:
  - {name: demo.board.BoardId, kind: newtype, of: String}
  - {name: demo.board.CardId, kind: newtype, of: String}
  - name: demo.board.Label
    kind: struct
    fields:
      - {name: text, type: String}
      - {name: colour, type: String, wire: hexColour}
entities:
  - name: demo.board.Card
    identity: {name: card_id, type: demo.board.CardId}
    lifecycle: {initial: Open, states: [Open], terminal: [Open], transitions: []}
errors:
  - name: demo.board.Rejected
commands:
  - name: demo.board.Move
    input:
      - {name: board_id, type: demo.board.BoardId, wire: boardId}
      - {name: card_id, type: demo.board.CardId, wire: cardId}
      - {name: position, type: Integer}
      - {name: label, type: demo.board.Label}
      - {name: note, type: Optional<String>}
    outcomes: [{name: refused, error: demo.board.Rejected}]
  - name: demo.board.Rename
    input:
      - {name: board_id, type: demo.board.BoardId, wire: boardId}
      - {name: card_id, type: demo.board.CardId, wire: cardId}
      - {name: title, type: String, wire: cardTitle}
      - {name: note, type: Optional<String>}
    outcomes: [{name: refused, error: demo.board.Rejected}]
  - name: demo.board.Ping
    input: []
    outcomes: [{name: refused, error: demo.board.Rejected}]
views:
  - name: demo.board.Cards
    source: demo.board.Card
    fields:
      - {name: card_id, type: demo.board.CardId}
components:
  - component: board
    owns: {domains: [demo.board]}
    accepts: {commands: [demo.board.Move, demo.board.Rename, demo.board.Ping]}
    reached_by: network
";

/// `demo.board.Move` with both identities in its path, its body JSON; `demo.board.Ping` derived.
const JSON_BODY: &str = "routes:
  - command: demo.board.Move
    method: put
    path: /boards/{board}/cards/{card}
    parameters: {board: input.board_id, card: input.card_id}
";

/// `demo.board.Rename` on the same identities, its scalar body form-encoded (a `form` body has no
/// spelling for a nested struct, `ESS-HTTP-021`): the bound document still publishes
/// `{command}.Input`.
const FORM_BODY: &str = "routes:
  - command: demo.board.Rename
    method: post
    path: /boards/{board}/cards/{card}/title
    parameters: {board: input.board_id, card: input.card_id}
    request: {encoding: form}
";

fn compiled() -> EssIr {
    let mut sources = SourceMap::new();
    sources.insert(Source::DOCUMENT, SOURCE.to_owned());
    let specification =
        Specification::assemble([(Source::document(), RawSpecFile::parse(SOURCE).unwrap())])
            .unwrap_or_else(|errors| panic!("validates: {errors}"));
    compile(&specification, &sources).unwrap_or_else(|errors| panic!("compiles: {errors}"))
}

fn bound_by(ir: &EssIr, routes: &str) -> Binding {
    let text = format!(
        "type: ess-http/1\nspecification:\n  system: demo\n  version: v1\n  source_digest: sha256:{}\n{routes}",
        ir.source_digest()
    );
    let spec = ess_http::HttpSpec::from_yaml(&text).expect("parses");
    let http = ess_http::compile(&spec, ir, &derived(ir))
        .unwrap_or_else(|diagnostics| panic!("compiles: {diagnostics}"));
    Binding::new(http).expect("every route has a path")
}

fn roots(names: &[&str]) -> BTreeSet<String> {
    names.iter().map(|name| (*name).to_owned()).collect()
}

fn published(ir: &EssIr, binding: &Binding) -> Value {
    let component = ir.components().values().next().expect("one component");
    let document = ess_gen::openapi::bound_json(ir, component, Some(binding))
        .replace("#/components/schemas/", "#/$defs/");
    let document: Value = serde_json::from_str(&document).expect("JSON");
    document["components"]["schemas"].clone()
}

fn validator(defs: &Value, root: &str) -> jsonschema::Validator {
    jsonschema::validator_for(&json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "$defs": defs,
        "$ref": format!("#/$defs/{root}"),
    }))
    .expect("a valid schema")
}

fn keys(value: &Value) -> BTreeSet<&str> {
    value
        .as_object()
        .unwrap_or_else(|| panic!("an object: {value}"))
        .keys()
        .map(String::as_str)
        .collect()
}

/// A1 (acceptance: renamed wire names, two path variables, nested and optional fields, json).
/// Green on 7978d1b5d is the expected outcome; red means the bound root left the document.
#[test]
fn a_bound_input_with_renamed_path_fields_nested_and_optional_members_is_the_published_body() {
    let ir = compiled();
    for (encoding, routes) in [("json", JSON_BODY)] {
        let binding = bound_by(&ir, routes);
        let projected = published(&ir, &binding);
        assert!(
            projected.get("demo.board.Move.Input").is_some(),
            "{encoding}: the bound document publishes no body: {projected:#}"
        );
        let selection =
            ModelTypes::select_bound(&ir, &roots(&["demo.board.Move.input"]), Some(&binding))
                .unwrap_or_else(|errors| panic!("{encoding}: selects: {errors:?}"));
        let schema: Value = serde_json::from_str(&selection.to_json()).expect("JSON");
        let generated = &schema["$defs"];

        assert_eq!(
            generated["demo.board.Move.input"], projected["demo.board.Move.Input"],
            "{encoding}: the bound input is not the published body"
        );
        assert_eq!(
            generated["demo.board.Label"], projected["demo.board.Label"],
            "{encoding}: the nested type differs from the published one"
        );
        assert_eq!(
            keys(&generated["demo.board.Move.input"]["properties"]),
            BTreeSet::from(["position", "label", "note"]),
            "{encoding}: both renamed path fields leave the body"
        );
        assert_eq!(
            keys(generated),
            BTreeSet::from(["demo.board.Move.input", "demo.board.Label"]),
            "{encoding}: the path fields' newtypes are not reached by the body: {schema:#}"
        );
        assert_eq!(
            selection.declared_field_order()["/$defs/demo.board.Move.input"],
            ["position", "label", "note"],
            "{encoding}: the body keeps its declared order"
        );

        let ours = validator(generated, "demo.board.Move.input");
        let theirs = validator(&projected, "demo.board.Move.Input");
        let body = json!({"position": 2, "label": {"text": "urgent", "hexColour": "#f00"}});
        let mut with_note = body.clone();
        with_note["note"] = json!("top of the column");
        for accepted in [body.clone(), with_note] {
            assert!(
                theirs.is_valid(&accepted),
                "{encoding}: projection refuses {accepted}"
            );
            assert!(
                ours.is_valid(&accepted),
                "{encoding}: the bound input refuses the published body {accepted}"
            );
        }
        let mut refused = Vec::new();
        for path_field in ["boardId", "cardId", "board_id", "card_id"] {
            let mut carrying = body.clone();
            carrying[path_field] = json!("b-1");
            refused.push(carrying);
        }
        let mut nested_declared_name = body.clone();
        nested_declared_name["label"] = json!({"text": "urgent", "colour": "#f00"});
        refused.push(nested_declared_name);
        let mut missing = body.clone();
        missing.as_object_mut().unwrap().remove("label");
        refused.push(missing);
        for body in refused {
            assert!(
                !theirs.is_valid(&body),
                "{encoding}: projection accepts {body}"
            );
            assert!(
                !ours.is_valid(&body),
                "{encoding}: the bound input accepts {body}"
            );
        }
    }
}

/// A2 (mutant the suite misses): the `it declares no input field` branch of `bound_input` is
/// never selected by the unit's cases; deleting it would refuse `demo.board.Ping.input` as
/// "its route carries every input field in its path", which is false for a derived route.
#[test]
fn a_command_with_no_input_field_is_refused_under_a_binding_for_having_none() {
    let ir = compiled();
    let binding = bound_by(&ir, JSON_BODY);
    let projected = published(&ir, &binding);
    assert!(
        projected.get("demo.board.Ping.Input").is_none(),
        "the bound document publishes a body for a command with no input"
    );
    let errors = ModelTypes::select_bound(&ir, &roots(&["demo.board.Ping.input"]), Some(&binding))
        .expect_err("a body the binding does not publish is refused");
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert_eq!(errors[0].name, "demo.board.Ping.input");
    assert_eq!(errors[0].rule, "unpublished_body", "{errors:?}");
    let detail = &errors[0].detail;
    assert!(
        detail.contains("`demo.board.Ping`") && detail.contains("no input field"),
        "the refusal names the command and why: {detail}"
    );
    assert!(
        !detail.contains("path") && !detail.contains("query"),
        "the refusal gives a reason that does not hold for an unrouted command: {detail}"
    );
    assert_eq!(
        ModelTypes::command_roots(&ir),
        roots(&[
            "demo.board.Move.input",
            "demo.board.Ping.input",
            "demo.board.Rename.input",
        ]),
        "without a binding every command has an input root"
    );
    assert_eq!(
        ModelTypes::bound_command_roots(&ir, Some(&binding)),
        roots(&["demo.board.Move.input", "demo.board.Rename.input"]),
        "under a binding a command with no published body has no input root"
    );
}

/// A3 (acceptance: a `form` body). The bound document publishes `{command}.Input` for a
/// form-encoded route too; the root must equal it and refuse the renamed path fields.
#[test]
fn a_form_encoded_bound_input_is_the_published_body() {
    let ir = compiled();
    let binding = bound_by(&ir, FORM_BODY);
    let projected = published(&ir, &binding);
    assert!(
        projected.get("demo.board.Rename.Input").is_some(),
        "the bound document publishes no body: {projected:#}"
    );
    let selection =
        ModelTypes::select_bound(&ir, &roots(&["demo.board.Rename.input"]), Some(&binding))
            .unwrap_or_else(|errors| panic!("selects: {errors:?}"));
    let schema: Value = serde_json::from_str(&selection.to_json()).expect("JSON");
    let generated = &schema["$defs"];
    assert_eq!(
        generated["demo.board.Rename.input"], projected["demo.board.Rename.Input"],
        "the bound form input is not the published body"
    );
    assert_eq!(
        keys(&generated["demo.board.Rename.input"]["properties"]),
        BTreeSet::from(["cardTitle", "note"]),
        "both renamed path fields leave the body"
    );
    let ours = validator(generated, "demo.board.Rename.input");
    let theirs = validator(&projected, "demo.board.Rename.Input");
    let body = json!({"cardTitle": "Ship it"});
    assert!(theirs.is_valid(&body), "projection refuses {body}");
    assert!(ours.is_valid(&body), "the bound input refuses {body}");
    for path_field in ["boardId", "cardId"] {
        let mut carrying = body.clone();
        carrying[path_field] = json!("b-1");
        assert!(!theirs.is_valid(&carrying), "projection accepts {carrying}");
        assert!(
            !ours.is_valid(&carrying),
            "the bound input accepts {carrying}"
        );
    }
}
