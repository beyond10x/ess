//! The `OpenAPI` projection of `input_absent: true` (ess/16, beyond10x/ess#170): the request body
//! is not required where a command declares an answer for its absence, and stays required where it
//! does not. The declared answer is a `400`, described as the branch for a request with no input.

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};

const MODEL: &str = include_str!("../../../specify/ess-compiler/tests/fixtures/absent-input.yaml");

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).expect("the model parses");
    let spec = Specification::assemble([(Source::new("absent-input.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("the model is admitted: {errors}"));
    compile(&spec, &SourceMap::new()).expect("the model compiles")
}

fn document(ir: &EssIr) -> serde_json::Value {
    let component = ir
        .components()
        .values()
        .next()
        .expect("the model has a component");
    serde_json::from_str(&ess_gen::openapi::json(ir, component)).expect("the document is JSON")
}

/// The operation whose body is `command`'s input.
fn operation<'d>(document: &'d serde_json::Value, command: &str) -> &'d serde_json::Value {
    let input = format!("{command}.Input");
    document["paths"]
        .as_object()
        .expect("paths")
        .values()
        .filter_map(|item| item.get("post"))
        .find(|post| {
            post["requestBody"]["content"]["application/json"]["schema"]["$ref"]
                .as_str()
                .is_some_and(|reference| reference.ends_with(&input))
        })
        .unwrap_or_else(|| panic!("no operation takes {input}:\n{document:#}"))
}

#[test]
fn the_body_is_optional_exactly_where_its_absence_has_a_declared_answer() {
    let document = document(&ir(MODEL));
    let submit = operation(&document, "demo.notes.SubmitNote");
    assert_eq!(submit["requestBody"]["required"], false, "{submit:#}");
    let reword = operation(&document, "demo.notes.RewordNote");
    assert_eq!(reword["requestBody"]["required"], true, "{reword:#}");
}

#[test]
fn the_declared_answer_is_a_refusal_described_as_the_absent_input() {
    let document = document(&ir(MODEL));
    let submit = operation(&document, "demo.notes.SubmitNote");
    let refused = &submit["responses"]["400"];
    assert!(refused.is_object(), "{submit:#}");
    let described = document.to_string();
    assert!(
        described.contains("Taken when the request carries no input at all"),
        "{document:#}"
    );
}

#[test]
fn a_model_without_the_marker_keeps_the_body_required() {
    let model = MODEL.replace(
        "      - name: body-missing\n        input_absent: true\n        error: demo.notes.BodyMissing\n",
        "",
    );
    assert_ne!(model, MODEL);
    let document = document(&ir(&model));
    let submit = operation(&document, "demo.notes.SubmitNote");
    assert_eq!(submit["requestBody"]["required"], true);
}
