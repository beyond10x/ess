//! A command's input, a command's response and a view's row are selectable as generated types
//! (beyond10x/ess#656).
//!
//! `--root <command>.input`, `--root <command>.response` and `--root <view>.row` select the body
//! the HTTP projection publishes for that message, built by the same type mapping, so a library
//! generated from the model holds exactly the request and response bodies the `OpenAPI` document
//! describes. A view with a declared `shape:` answers rows of that shape, and its row root is a
//! reference to it.

use std::collections::BTreeSet;

use ess_compiler::{resolve::compile, source::SourceMap, EssIr};
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source,
};
use ess_gen::schema::ModelTypes;
use serde_json::{json, Value};

const SOURCE: &str = include_str!("fixtures/command-view-roots.yaml");

fn compiled() -> EssIr {
    let mut sources = SourceMap::new();
    sources.insert(Source::DOCUMENT, SOURCE.to_owned());
    let specification =
        Specification::assemble([(Source::document(), RawSpecFile::parse(SOURCE).unwrap())])
            .unwrap();
    compile(&specification, &sources).unwrap()
}

fn selected(roots: &[&str]) -> Value {
    let ir = compiled();
    let roots = roots.iter().map(|root| (*root).to_owned()).collect();
    let selection = ModelTypes::select(&ir, &roots).expect("selects");
    serde_json::from_str(&selection.to_json()).expect("JSON")
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
fn a_command_input_is_a_root() {
    let schema = selected(&["demo.notes.Create.input"]);
    assert_eq!(
        keys(&schema["$defs"]),
        BTreeSet::from(["demo.notes.Create.input", "demo.notes.NoteId"]),
        "{schema:#}"
    );
    let input = &schema["$defs"]["demo.notes.Create.input"];
    assert_eq!(input["x-ess-kind"], "command-input", "{input:#}");
    assert_eq!(
        keys(&input["properties"]),
        BTreeSet::from(["note_id", "title", "isPinned"]),
        "the input's wire names"
    );
    assert_eq!(input["additionalProperties"], false, "{input:#}");
}

#[test]
fn a_command_response_is_a_root() {
    let schema = selected(&["demo.notes.Create.response"]);
    let response = &schema["$defs"]["demo.notes.Create.response"];
    assert_eq!(response["x-ess-kind"], "command-response", "{response:#}");
    assert_eq!(
        keys(&response["properties"]),
        BTreeSet::from(["note_id", "titleLength"]),
        "the response's wire names"
    );
    assert!(schema["$defs"]["demo.notes.NoteId"].is_object());
}

#[test]
fn a_view_row_is_a_root() {
    let schema = selected(&["demo.notes.Notes.row", "demo.notes.Summaries.row"]);
    let row = &schema["$defs"]["demo.notes.Notes.row"];
    assert_eq!(row["x-ess-kind"], "view-row", "{row:#}");
    assert_eq!(
        keys(&row["properties"]),
        BTreeSet::from(["note_id", "title"])
    );
    let shaped = &schema["$defs"]["demo.notes.Summaries.row"];
    assert_eq!(shaped["$ref"], "#/$defs/demo.notes.Summary", "{shaped:#}");
    assert!(schema["$defs"]["demo.notes.Summary"].is_object());
}

#[test]
fn a_misspelled_or_absent_body_is_an_unknown_root() {
    let ir = compiled();
    for root in [
        "demo.notes.Create.row",
        "demo.notes.Notes.input",
        "demo.notes.Missing.input",
        "demo.notes.Create.Input",
    ] {
        let errors =
            ModelTypes::select(&ir, &BTreeSet::from([root.to_owned()])).expect_err("refused");
        assert_eq!(errors[0].rule, "unknown_root", "{root}");
        assert!(errors[0].detail.contains(".input"), "{}", errors[0].detail);
    }
}

/// A command that declares no response has no `<command>.response` root: `--all-commands` leaves
/// it out and the `OpenAPI` document publishes no result for it, so naming it is refused rather
/// than selecting an empty object.
#[test]
fn a_command_without_a_response_has_no_response_root_by_name_either() {
    let ir = compiled();
    assert!(
        !ModelTypes::command_roots(&ir).contains("demo.notes.Pin.response"),
        "the fixture's `demo.notes.Pin` declares no response"
    );
    let errors = ModelTypes::select(&ir, &BTreeSet::from(["demo.notes.Pin.response".to_owned()]))
        .expect_err("a command without a response has no response root");
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert_eq!(errors[0].name, "demo.notes.Pin.response");
    assert_eq!(errors[0].rule, "unknown_root", "{errors:?}");
    assert!(
        errors[0]
            .detail
            .contains("command `demo.notes.Pin` declares no response"),
        "the refusal says why the root is unknown: {}",
        errors[0].detail
    );
    assert!(
        ModelTypes::select(&ir, &BTreeSet::from(["demo.notes.Pin.input".to_owned()])).is_ok(),
        "its input is still a root"
    );
}

/// `components.schemas` of the `OpenAPI` document, its references retargeted to `$defs`, so one
/// validator reads both documents the same way.
fn published() -> Value {
    let ir = compiled();
    let component = ir.components().values().next().expect("one component");
    let document =
        ess_gen::openapi::json(&ir, component).replace("#/components/schemas/", "#/$defs/");
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

/// The body the projection's own examples spell: each input property's published `examples`.
fn example(schema: &Value) -> Value {
    Value::Object(
        schema["properties"]
            .as_object()
            .expect("properties")
            .iter()
            .map(|(key, property)| (key.clone(), property["examples"][0].clone()))
            .collect(),
    )
}

#[test]
fn the_types_match_the_http_projection() {
    let projected = published();
    let schema = selected(&[
        "demo.notes.Create.input",
        "demo.notes.Create.response",
        "demo.notes.Notes.row",
        "demo.notes.Summaries.row",
    ]);
    let generated = &schema["$defs"];
    for (root, component) in [
        ("demo.notes.Create.input", "demo.notes.Create.Input"),
        ("demo.notes.Create.response", "demo.notes.Create.Result"),
        ("demo.notes.Notes.row", "demo.notes.Notes.Row"),
        ("demo.notes.Summary", "demo.notes.Summary"),
        ("demo.notes.NoteId", "demo.notes.NoteId"),
    ] {
        assert_eq!(
            generated[root], projected[component],
            "{root} is not the schema the projection publishes as {component}"
        );
    }

    let input = example(&projected["demo.notes.Create.Input"]);
    assert_eq!(
        input,
        json!({"note_id": "n-1", "title": "Buy milk", "isPinned": true})
    );
    let bodies = [
        (
            "demo.notes.Create.input",
            "demo.notes.Create.Input",
            input.clone(),
        ),
        (
            "demo.notes.Create.response",
            "demo.notes.Create.Result",
            json!({"note_id": "n-1", "titleLength": 8}),
        ),
        (
            "demo.notes.Notes.row",
            "demo.notes.Notes.Row",
            json!({"note_id": "n-1", "title": "Buy milk"}),
        ),
        (
            "demo.notes.Summaries.row",
            "demo.notes.Summary",
            json!({"note_id": "n-1", "title": "Buy milk"}),
        ),
    ];
    for (root, component, body) in bodies {
        let ours = validator(generated, root);
        let theirs = validator(&projected, component);
        assert!(theirs.is_valid(&body), "the projection refuses {body}");
        assert!(ours.is_valid(&body), "{root} refuses {body}");
        let mut undeclared = body.clone();
        undeclared["extra"] = json!(1);
        let mut missing = body.clone();
        missing.as_object_mut().unwrap().remove("note_id");
        let mut mistyped = body.clone();
        mistyped["note_id"] = json!(7);
        for refused in [undeclared, missing, mistyped] {
            assert!(
                !theirs.is_valid(&refused),
                "the projection accepts {refused}"
            );
            assert!(!ours.is_valid(&refused), "{root} accepts {refused}");
        }
    }
}
