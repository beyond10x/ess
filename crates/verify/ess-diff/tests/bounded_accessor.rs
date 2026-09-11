//! Binding deltas expose changes to result type and traversal presence.
use ess_compiler::{resolve::compile_locating, source::SourceMap, EssIr};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};

fn compile(text: &str) -> EssIr {
    let spec = Specification::assemble([(
        Source::new("accessor.yaml"),
        RawSpecFile::parse(text).unwrap(),
    )])
    .unwrap();
    let mut sources = SourceMap::new();
    sources.insert("accessor.yaml", text);
    compile_locating(&spec, &sources, &["accessor.yaml"]).unwrap()
}

#[test]
fn same_path_with_different_presence_or_result_has_an_explanatory_binding_delta() {
    let text = include_str!("../../../generate/ess-synth/tests/fixtures/bounded-accessor.yaml");
    let before = compile(text);
    let total = compile(&text.replace("Optional<projection.core.Body>", "projection.core.Body"));
    let delta = ess_diff::diff(&before, &total).unwrap().to_canonical_json();
    assert!(delta.contains("event.partial.status : String (possibly unavailable)"));
    assert!(delta.contains("event.partial.status : String (total)"));
    let integer = compile(&text.replace("String", "Integer"));
    let delta = ess_diff::diff(&before, &integer)
        .unwrap()
        .to_canonical_json();
    assert!(delta.contains("event.data.status : String (total)"));
    assert!(delta.contains("event.data.status : Integer (total)"));
}
