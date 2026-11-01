//! A command's input root under an `ess-http/1` binding is the request body the bound `OpenAPI`
//! document publishes (beyond10x/ess#656).
//!
//! `ModelTypes::select_bound` takes the binding `ess generate types --http` compiles. A route that
//! carries input fields in its path publishes `{command}.Input` without them, so the
//! `<command>.input` root drops them too; a route that publishes no body (every field in the path,
//! or a `query` request) has no `<command>.input` root: naming it is refused by the command and the
//! reason, and [`ModelTypes::bound_command_roots`] leaves it out. Responses and view rows are
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
