//! The held lifecycle state as a value source, `{subject: state}` (ess/23, beyond10x/ess#458): an
//! error field moved from a literal to the held state is the error-payload change it always was,
//! and the delta names the new source. No delta format moves.

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};

const MODEL: &str = include_str!("../../ess-conformance/tests/fixtures/subject-state-source.yaml");

fn ir(text: &str) -> EssIr {
    let spec = Specification::assemble([(
        Source::new("docs.yaml"),
        RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}")),
    )])
    .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap()
}

#[test]
fn an_error_field_moved_to_the_held_state_is_named() {
    let literal = MODEL.replacen(
        "{doc_id: input.doc_id, current: {subject: state}, requested: Published}",
        "{doc_id: input.doc_id, current: Archived, requested: Published}",
        1,
    );
    assert_ne!(literal, MODEL);
    let delta = ess_diff::classified(&ir(&literal), &ir(MODEL)).unwrap();
    let json = delta.to_canonical_json();
    let document: serde_json::Value = serde_json::from_str(&json).unwrap();
    let changed = document["changes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|entry| {
            entry["id"]
                .as_str()
                .is_some_and(|id| id.starts_with("command/demo.docs.PublishDoc/"))
        })
        .unwrap_or_else(|| panic!("a change to PublishDoc in {json}"));
    let text = changed.to_string();
    assert!(
        text.contains("current <- the state the subject held before the outcome"),
        "{text}"
    );
    assert!(text.contains("current <- literal `Archived`"), "{text}");
    // The format the delta already had: nothing about the held state needs a newer one.
    let unchanged = ess_diff::classified(&ir(&literal), &ir(&literal)).unwrap();
    assert_eq!(delta.format, unchanged.format);
}
