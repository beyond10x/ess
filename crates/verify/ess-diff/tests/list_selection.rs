//! Selection semantics remain visible when the mapped selector index stays unchanged.
use ess_compiler::{resolve::compile_locating, source::SourceMap, EssIr};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
fn compile(text: &str) -> EssIr {
    let spec = Specification::assemble([(
        Source::new("selection.yaml"),
        RawSpecFile::parse(text).unwrap(),
    )])
    .unwrap();
    let mut sources = SourceMap::new();
    sources.insert("selection.yaml", text);
    compile_locating(&spec, &sources, &["selection.yaml"]).unwrap()
}
#[test]
fn changed_predicate_exclusion_and_fallback_have_typed_versioned_deltas() {
    let model = include_str!("../../../generate/ess-synth/tests/fixtures/binding-selection.yaml");
    let before = compile(model);
    for changed in [
        model.replace("item.domain == Internal", "item.domain == External"),
        model.replace("excluding: [first_agent]", "excluding: []"),
        model.replace(
            "[first_agent, first_identified]",
            "[first_identified, first_agent]",
        ),
    ] {
        let delta = ess_diff::diff(&before, &compile(&changed))
            .unwrap()
            .to_canonical_json();
        assert!(delta.contains("ess-diff/3"));
        assert!(delta.contains("selection-plan-changed"), "{delta}");
    }
    let unchanged = ess_diff::diff(&before, &before)
        .unwrap()
        .to_canonical_json();
    assert!(unchanged.contains("ess-diff/2"));
}
