//! A command's input root under an `ess-http/1` binding is the request body the bound `OpenAPI`
//! document publishes (beyond10x/ess#656).
//!
//! `ModelTypes::select_bound` takes the binding `ess generate types --http` compiles. A route that
//! carries input fields in its path publishes `{command}.Input` without them, so the
//! `<command>.input` root drops them too; a route that publishes no body (every field in the path,
//! or a `query` request) has no `<command>.input` root: naming it is refused by the command and the
//! reason, and [`ModelTypes::bound_command_roots`] leaves it out. A command whose route publishes no
//! `{command}.Result` (every outcome answers a field, an error or nothing) has no
//! `<command>.response` root, refused as `unknown_root`. Other responses and view rows are
//! published unchanged under a binding, so their roots are unchanged; without a binding every root
//! keeps its bytes.

use std::collections::BTreeSet;

use ess_compiler::{resolve::compile, source::SourceMap, EssIr};
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source,
};
use ess_gen::http::{derived, Binding};
use ess_gen::schema::ModelTypes;
use serde_json::{json, Value};

/// The model `command_view_roots.rs` reads, unchanged: `demo.notes.Create` takes `note_id`,
/// `title` and `pinned` (wire `isPinned`), `demo.notes.Pin` takes `note_id` alone.
const SOURCE: &str = include_str!("fixtures/command-view-roots.yaml");

/// `demo.notes.Create` at `PUT /notes/{note_id}`, its `note_id` in the path, and
/// `demo.notes.Pin` at `POST /notes/{note_id}/pin`, its only field in the path.
const ON_PATHS: &str = "routes:
  - command: demo.notes.Create
    method: put
    path: /notes/{note_id}
    parameters: {note_id: input.note_id}
  - command: demo.notes.Pin
    method: post
    path: /notes/{note_id}/pin
    parameters: {note_id: input.note_id}
";

/// `demo.notes.Create` at `GET /notes/{note_id}`, every other field in the query string.
const IN_THE_QUERY: &str = "routes:
  - command: demo.notes.Create
    method: get
    path: /notes/{note_id}
    parameters: {note_id: input.note_id}
    request: {encoding: query}
";

/// The body the bound document publishes for `demo.notes.Create`, from the adopter's report.
fn published_body() -> Value {
    json!({"isPinned": true, "title": "Buy milk"})
}

fn compiled() -> EssIr {
    let mut sources = SourceMap::new();
    sources.insert(Source::DOCUMENT, SOURCE.to_owned());
    let specification =
        Specification::assemble([(Source::document(), RawSpecFile::parse(SOURCE).unwrap())])
            .unwrap();
    compile(&specification, &sources).unwrap()
}

/// `routes` compiled against `ir` as `ess specify http compile` compiles it, and held to a path on
/// every route as `ess generate --kind openapi --http` holds it.
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

/// The selection's JSON Schema document.
fn document(selection: &ModelTypes) -> Value {
    serde_json::from_str(&selection.to_json()).expect("JSON")
}

/// `components.schemas` of the bound `OpenAPI` document, its references retargeted to `$defs`, so
/// one validator reads both documents the same way.
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
        .expect("object")
        .keys()
        .map(String::as_str)
        .collect()
}

#[test]
fn a_bound_command_input_decodes_the_request_body_the_bound_projection_publishes() {
    let ir = compiled();
    let binding = bound_by(&ir, ON_PATHS);
    let projected = published(&ir, &binding);
    let selection =
        ModelTypes::select_bound(&ir, &roots(&["demo.notes.Create.input"]), Some(&binding))
            .expect("selects");
    let schema = document(&selection);
    let generated = &schema["$defs"];

    assert_eq!(
        generated["demo.notes.Create.input"], projected["demo.notes.Create.Input"],
        "the bound input is not the body the bound OpenAPI document publishes"
    );
    assert_eq!(
        keys(&generated["demo.notes.Create.input"]["properties"]),
        BTreeSet::from(["title", "isPinned"]),
        "the field the route carries in its path leaves the body"
    );
    assert_eq!(
        keys(generated),
        BTreeSet::from(["demo.notes.Create.input"]),
        "the body reaches no named type once `note_id` is in the path: {schema:#}"
    );
    assert_eq!(
        selection.declared_field_order()["/$defs/demo.notes.Create.input"],
        ["title", "isPinned"],
        "the body keeps its declared order"
    );

    let ours = validator(generated, "demo.notes.Create.input");
    let theirs = validator(&projected, "demo.notes.Create.Input");
    let body = published_body();
    assert!(theirs.is_valid(&body), "the projection refuses {body}");
    assert!(
        ours.is_valid(&body),
        "the bound input refuses the published body {body}"
    );
    let mut with_path_field = body.clone();
    with_path_field["note_id"] = json!("n-1");
    let mut missing = body.clone();
    missing.as_object_mut().unwrap().remove("title");
    let mut mistyped = body.clone();
    mistyped["isPinned"] = json!("yes");
    for refused in [with_path_field, missing, mistyped] {
        assert!(
            !theirs.is_valid(&refused),
            "the projection accepts {refused}"
        );
        assert!(
            !ours.is_valid(&refused),
            "the bound input accepts {refused}"
        );
    }
}

#[test]
fn a_bound_route_that_publishes_no_body_has_no_input_root() {
    let ir = compiled();
    for (routes, command, reason) in [
        (ON_PATHS, "demo.notes.Pin", "path"),
        (IN_THE_QUERY, "demo.notes.Create", "query"),
    ] {
        let binding = bound_by(&ir, routes);
        let projected = published(&ir, &binding);
        assert!(
            projected.get(format!("{command}.Input")).is_none(),
            "the bound document publishes a body for {command}"
        );
        let root = format!("{command}.input");
        let errors = ModelTypes::select_bound(&ir, &roots(&[root.as_str()]), Some(&binding))
            .expect_err("a body the binding does not publish is refused");
        assert_eq!(errors.len(), 1, "{errors:?}");
        assert_eq!(errors[0].name, root);
        assert_eq!(errors[0].rule, "unpublished_body", "{errors:?}");
        assert!(
            errors[0].detail.contains(&format!("`{command}`")) && errors[0].detail.contains(reason),
            "the refusal names the command and why: {}",
            errors[0].detail
        );
        assert!(
            !ModelTypes::bound_command_roots(&ir, Some(&binding)).contains(&root),
            "{root} is selected by every command"
        );
    }

    let binding = bound_by(&ir, ON_PATHS);
    assert_eq!(
        ModelTypes::bound_command_roots(&ir, Some(&binding)),
        roots(&["demo.notes.Create.input", "demo.notes.Create.response"]),
        "every command's published body and its response"
    );
    let binding = bound_by(&ir, IN_THE_QUERY);
    assert_eq!(
        ModelTypes::bound_command_roots(&ir, Some(&binding)),
        roots(&["demo.notes.Create.response", "demo.notes.Pin.input"]),
        "a command the binding does not route keeps its derived body"
    );
    let selection =
        ModelTypes::select_bound(&ir, &roots(&["demo.notes.Pin.input"]), Some(&binding))
            .expect("selects");
    let unbound =
        ModelTypes::select(&ir, &roots(&["demo.notes.Pin.input"])).expect("selects unbound");
    assert_eq!(selection.to_json(), unbound.to_json());
}

#[test]
fn responses_and_rows_are_unchanged_under_a_binding() {
    let ir = compiled();
    let binding = bound_by(&ir, ON_PATHS);
    let names = roots(&[
        "demo.notes.Create.response",
        "demo.notes.Notes.row",
        "demo.notes.Summaries.row",
    ]);
    let bound = ModelTypes::select_bound(&ir, &names, Some(&binding)).expect("selects");
    let unbound = ModelTypes::select(&ir, &names).expect("selects");
    assert_eq!(bound.to_json(), unbound.to_json());

    let projected = published(&ir, &binding);
    let generated = document(&bound)["$defs"].clone();
    for (root, component) in [
        ("demo.notes.Create.response", "demo.notes.Create.Result"),
        ("demo.notes.Notes.row", "demo.notes.Notes.Row"),
        ("demo.notes.Summary", "demo.notes.Summary"),
    ] {
        assert_eq!(
            generated[root], projected[component],
            "{root} is not the schema the bound projection publishes as {component}"
        );
    }
}

#[test]
fn without_a_binding_every_root_keeps_its_bytes() {
    let ir = compiled();
    let mut names = ModelTypes::command_roots(&ir);
    names.extend(ModelTypes::view_roots(&ir));
    names.extend([
        "demo.notes.NoteId".to_owned(),
        "demo.notes.Created".to_owned(),
    ]);
    let unbound = ModelTypes::select(&ir, &names).expect("selects");
    let none = ModelTypes::select_bound(&ir, &names, None).expect("selects");
    assert_eq!(none.to_json(), unbound.to_json());
    assert_eq!(none.declared_field_order(), unbound.declared_field_order());
    assert_eq!(none.binary64_locations(), unbound.binary64_locations());
    assert_eq!(
        ModelTypes::bound_command_roots(&ir, None),
        ModelTypes::command_roots(&ir)
    );
}

/// A command whose only route outcome answers `response.token`, so the bound document publishes
/// no `tokens.auth.Issue.Result`.
const TOKENS: &str = "format: ess/27
system: tokens
version: v1
domain: tokens.auth
actors:
  - {name: tokens.auth.Client, may: [tokens.auth.Issue]}
commands:
  - name: tokens.auth.Issue
    input: [{name: subject, type: String}]
    response:
      - {name: token, type: String}
      - {name: expires_in, type: Integer}
    outcomes:
      - {name: issued, returns: true}
components:
  - component: auth
    owns: {domains: [tokens.auth]}
    accepts: {commands: [tokens.auth.Issue]}
    reached_by: network
";

const TOKEN_ROUTE: &str = "routes:
  - command: tokens.auth.Issue
    method: post
    path: /token
    outcomes:
      issued: {status: 200, body: response.token}
";

#[test]
fn a_bound_route_that_publishes_no_result_has_no_response_root() {
    let mut sources = SourceMap::new();
    sources.insert(Source::DOCUMENT, TOKENS.to_owned());
    let specification =
        Specification::assemble([(Source::document(), RawSpecFile::parse(TOKENS).unwrap())])
            .unwrap();
    let ir = compile(&specification, &sources).unwrap();
    let text = format!(
        "type: ess-http/1\nspecification:\n  system: tokens\n  version: v1\n  source_digest: sha256:{}\n{TOKEN_ROUTE}",
        ir.source_digest()
    );
    let spec = ess_http::HttpSpec::from_yaml(&text).expect("parses");
    let http = ess_http::compile(&spec, &ir, &derived(&ir))
        .unwrap_or_else(|diagnostics| panic!("compiles: {diagnostics}"));
    let binding = Binding::new(http).expect("every route has a path");

    let projected = published(&ir, &binding);
    assert!(
        projected.get("tokens.auth.Issue.Result").is_none(),
        "the bound document publishes a Result"
    );

    let root = "tokens.auth.Issue.response";
    let errors = ModelTypes::select_bound(&ir, &roots(&[root]), Some(&binding))
        .expect_err("a response the route does not publish is refused");
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert_eq!(errors[0].name, root);
    assert_eq!(errors[0].rule, "unknown_root", "{errors:?}");
    assert!(
        errors[0].detail.contains("`tokens.auth.Issue`")
            && errors[0].detail.contains("publishes no `Result`"),
        "the refusal names the command and why: {}",
        errors[0].detail
    );
    assert_eq!(
        ModelTypes::bound_command_roots(&ir, Some(&binding)),
        roots(&["tokens.auth.Issue.input"])
    );
    assert!(ModelTypes::bound_command_roots(&ir, None).contains(root));
}

// Adversary pass 1 (story:bound-response-root-follows-the-published-result): a command the binding
// leaves answered by the envelope, whose outcome carries no result, is published by the bound
// document exactly as by the unbound one.

/// `tokens.auth.Issue` as in [`TOKENS`], and `tokens.auth.Revoke`, which declares a response
/// whose only outcome neither returns nor is replayed: no document publishes
/// `tokens.auth.Revoke.Result`, bound or not.
const TOKENS_AND_REVOKE: &str = "format: ess/27
system: tokens
version: v1
domain: tokens.auth
actors:
  - {name: tokens.auth.Client, may: [tokens.auth.Issue, tokens.auth.Revoke]}
commands:
  - name: tokens.auth.Issue
    input: [{name: subject, type: String}]
    response:
      - {name: token, type: String}
      - {name: expires_in, type: Integer}
    outcomes:
      - {name: issued, returns: true}
  - name: tokens.auth.Revoke
    input: [{name: token, type: String}]
    response:
      - {name: revoked, type: Boolean}
    outcomes:
      - name: revoked
        emits: [tokens.auth.Revoked]
        payload:
          tokens.auth.Revoked: {token: input.token}
events:
  - name: tokens.auth.Revoked
    fields:
      - {name: token, type: String}
components:
  - component: auth
    owns: {domains: [tokens.auth]}
    accepts: {commands: [tokens.auth.Issue, tokens.auth.Revoke]}
    publishes: {events: [tokens.auth.Revoked]}
    reached_by: network
";

/// `tokens.auth.Revoke` routed with no answer listed: every outcome keeps the envelope.
const REVOKE_ROUTE_NO_ANSWERS: &str = "  - command: tokens.auth.Revoke
    method: post
    path: /revoke
";

fn tokens_and_revoke() -> EssIr {
    let mut sources = SourceMap::new();
    sources.insert(Source::DOCUMENT, TOKENS_AND_REVOKE.to_owned());
    let specification = Specification::assemble([(
        Source::document(),
        RawSpecFile::parse(TOKENS_AND_REVOKE).unwrap(),
    )])
    .unwrap();
    compile(&specification, &sources).unwrap()
}

fn tokens_binding(ir: &EssIr, routes: &str) -> Binding {
    let text = format!(
        "type: ess-http/1\nspecification:\n  system: tokens\n  version: v1\n  source_digest: sha256:{}\n{routes}",
        ir.source_digest()
    );
    let spec = ess_http::HttpSpec::from_yaml(&text).expect("parses");
    let http = ess_http::compile(&spec, ir, &derived(ir))
        .unwrap_or_else(|diagnostics| panic!("compiles: {diagnostics}"));
    Binding::new(http).expect("every route has a path")
}

fn unbound_published(ir: &EssIr) -> Value {
    let component = ir.components().values().next().expect("one component");
    let document: Value =
        serde_json::from_str(&ess_gen::openapi::bound_json(ir, component, None)).expect("JSON");
    document["components"]["schemas"].clone()
}

/// The bindings under which `tokens.auth.Revoke` is answered as without one: not routed, and
/// routed with no answer listed.
fn envelope_bindings(ir: &EssIr) -> [(&'static str, Binding); 2] {
    [
        ("unrouted", tokens_binding(ir, TOKEN_ROUTE)),
        (
            "routed, no answer listed",
            tokens_binding(ir, &format!("{TOKEN_ROUTE}{REVOKE_ROUTE_NO_ANSWERS}")),
        ),
    ]
}

#[test]
fn adversary_a_command_answered_by_the_envelope_keeps_its_response_root_under_a_binding() {
    let ir = tokens_and_revoke();
    let root = "tokens.auth.Revoke.response";
    let schema = "tokens.auth.Revoke.Result";
    assert!(
        ModelTypes::bound_command_roots(&ir, None).contains(root),
        "without a binding the root is offered"
    );
    let unbound = ModelTypes::select(&ir, &roots(&[root])).expect("selects unbound");
    for (case, binding) in envelope_bindings(&ir) {
        assert_eq!(
            published(&ir, &binding).get(schema).is_some(),
            unbound_published(&ir).get(schema).is_some(),
            "{case}: the binding changed whether {schema} is published"
        );
        assert!(
            ModelTypes::bound_command_roots(&ir, Some(&binding)).contains(root),
            "{case}: the bound document publishes {root}'s command as the unbound one does, yet \
             --all-commands --http drops the root that --all-commands offers"
        );
        let bound = ModelTypes::select_bound(&ir, &roots(&[root]), Some(&binding))
            .unwrap_or_else(|errors| panic!("{case}: refused: {errors:?}"));
        assert_eq!(bound.to_json(), unbound.to_json(), "{case}");
    }
}

#[test]
fn adversary_a_refused_response_root_does_not_blame_answers_the_route_never_lists() {
    let ir = tokens_and_revoke();
    let root = "tokens.auth.Revoke.response";
    for (case, binding) in envelope_bindings(&ir) {
        if let Err(errors) = ModelTypes::select_bound(&ir, &roots(&[root]), Some(&binding)) {
            assert!(
                !errors[0]
                    .detail
                    .contains("every outcome answers a field, an error or nothing"),
                "{case}: tokens.auth.Revoke's only outcome is answered by the envelope, yet the \
                 refusal says: {}",
                errors[0].detail
            );
        }
    }
}
