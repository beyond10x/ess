//! Adversarial case, fourth pass: where the status refusal of `ESS-HTTP-025` says "with a body a
//! reader cannot tell apart".
//!
//! The sentence is said where `same_answer` holds: the same body form, and both or neither a
//! redirect. The design page says it is said "only of outcomes whose answers are the same". Two
//! outcomes answering `201` with `response.note_id`, one `encoding: text` and one not, have one
//! body form and are not one answer: the first is `text/plain; charset=utf-8` and the bare text
//! (`n1`), the second `application/json` and a JSON string (`"n1"`). A reader tells them apart by
//! `Content-Type` and by the quotes. The pair is refused (the rule does not count `encoding`, which
//! this case does not dispute); the sentence printed of it is false.

use ess_compiler::resolve::compile as compile_model;
use ess_compiler::source::SourceMap;
use ess_compiler::EssIr;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_http::{compile, DerivedRoute, HttpSpec, Method, Operation};

const NOTES: &str = include_str!("fixtures/notes.yaml");

fn model() -> EssIr {
    let raw = RawSpecFile::parse(NOTES).expect("well formed");
    let spec = Specification::assemble([(Source::new("notes.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("validates: {errors}"));
    let mut sources = SourceMap::new();
    sources.insert("notes.yaml", NOTES);
    compile_model(&spec, &sources).unwrap_or_else(|errors| panic!("compiles: {errors}"))
}

fn derived(ir: &EssIr) -> Vec<DerivedRoute> {
    let mut table = Vec::new();
    for component in ir.components().values() {
        for handle in &component.accepts {
            table.push(DerivedRoute {
                component: component.name.to_string(),
                operation: Operation::Command(handle.name().to_string()),
                method: Method::Post,
                path: format!("/derived/commands/{}", handle.name().local()),
            });
        }
    }
    table.sort_by(|left, right| {
        (&left.component, &left.path, left.method, &left.operation).cmp(&(
            &right.component,
            &right.path,
            right.method,
            &right.operation,
        ))
    });
    table
}

#[test]
fn a_text_and_a_json_answer_of_one_field_are_not_said_to_be_one_body() {
    let ir = model();
    let text = format!(
        "type: ess-http/1\nspecification:\n  system: demo\n  version: v1\n  source_digest: sha256:{}\nroutes:\n  - command: demo.notes.CreateNote\n    method: post\n    path: /api/notes\n    outcomes:\n      fetched: {{status: 201, body: response.note_id, encoding: text}}\n      inline: {{status: 201, body: response.note_id}}\n",
        ir.source_digest()
    );
    let spec = HttpSpec::from_yaml(&text).unwrap_or_else(|error| panic!("parses: {error}\n{text}"));
    let words = compile(&spec, &ir, &derived(&ir)).map_or_else(
        |diagnostics| diagnostics.to_string(),
        |_| panic!("the rule does not count `encoding`, so the pair is refused"),
    );
    assert!(
        words.contains("outcome `fetched`") && words.contains("`inline` (`response.note_id`)"),
        "the pair is refused at `fetched`, naming `inline`:\n{words}"
    );
    assert!(
        !words.contains("`inline` (`response.note_id`), with a body a reader cannot tell apart"),
        "`text/plain` `n1` against `application/json` `\"n1\"`: a reader tells the two apart, yet \
         the refusal says it cannot:\n{words}"
    );
}
