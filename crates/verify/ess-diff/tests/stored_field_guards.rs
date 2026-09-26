//! A changed stored-field guard is a semantic change, rendered as the predicate it is (ess#75).
use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_diff::diff;
use ess_domain::{spec::RawSpecFile, system::Source, Specification};

const PARCELS: &str = include_str!("../../ess-conformance/tests/fixtures/stored-field-guards.yaml");

fn ir(text: &str) -> EssIr {
    let spec = Specification::assemble([(
        Source::new("parcels.yaml"),
        RawSpecFile::parse(text).unwrap(),
    )])
    .unwrap();
    compile(&spec, &SourceMap::new()).unwrap()
}

#[test]
fn moving_the_weight_limit_is_an_outcome_condition_change_in_the_existing_diff_format() {
    let before = ir(PARCELS);
    let after = ir(&PARCELS.replace("weight_kg > 20", "weight_kg > 30"));
    let delta = diff(&before, &after).unwrap();
    let json = delta.to_canonical_json();
    assert!(json.contains("outcome-condition-changed"), "{json}");
    assert!(
        json.contains("when subject fields satisfy (service == Express and weight_kg > 20)")
            && json.contains("when subject fields satisfy (service == Express and weight_kg > 30)"),
        "{json}"
    );
    assert!(!json.contains("SubjectPredicate"), "{json}");
    assert_eq!(
        delta.format.to_string(),
        "ess-diff/2",
        "no new diff format: the condition is already a rendered string"
    );
}
