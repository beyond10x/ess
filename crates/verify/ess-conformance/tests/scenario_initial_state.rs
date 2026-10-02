//! The declared logical namespace boundary, including historical wire compatibility.
use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::{synthesize::synthesize, AdmittedSuite};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};

fn model() -> EssIr {
    let source = include_str!("../../../specify/ess-compiler/tests/fixtures/set-effects.yaml");
    let spec = Specification::assemble([(
        Source::new("model.yaml"),
        RawSpecFile::parse(source).unwrap(),
    )])
    .unwrap();
    compile(&spec, &SourceMap::new()).unwrap()
}

#[test]
fn every_fresh_suite_declares_empty_logical_scenario_state_in_format34() {
    let synthesis = synthesize(&model());
    assert!(synthesis.refusals.is_empty());
    let bytes = synthesis.suite.to_canonical_json().unwrap();
    let wire: serde_json::Value = serde_json::from_str(&bytes).unwrap();
    assert_eq!(wire["provenance"]["scenario_initial_state"], "empty");
    assert_eq!(wire["provenance"]["suite_version"], "ess-conformance/34");
    assert_eq!(
        AdmittedSuite::from_json(&bytes).unwrap().original_json(),
        bytes
    );
}

fn empty_wire(major: u32) -> serde_json::Value {
    let synthesis = synthesize(&model());
    let mut wire: serde_json::Value =
        serde_json::from_str(&synthesis.suite.to_canonical_json().unwrap()).unwrap();
    wire["scenarios"] = serde_json::json!({});
    wire["provenance"]["suite_version"] = format!("ess-conformance/{major}").into();
    wire["provenance"]
        .as_object_mut()
        .unwrap()
        .remove("scenario_initial_state");
    wire
}

#[test]
fn new_major_requires_exact_initial_state_and_legacy_does_not_acquire_it() {
    let mut new = empty_wire(34);
    assert!(
        AdmittedSuite::from_json(&new.to_string()).is_err(),
        "missing requirement"
    );
    for invalid in [
        serde_json::Value::Null,
        serde_json::json!("shared"),
        serde_json::json!(false),
    ] {
        new["provenance"]["scenario_initial_state"] = invalid;
        assert!(AdmittedSuite::from_json(&new.to_string()).is_err());
    }
    new["provenance"]["scenario_initial_state"] = "empty".into();
    AdmittedSuite::from_json(&new.to_string()).unwrap();
    for major in [
        1, 2, 3, 4, 6, 8, 10, 12, 14, 16, 18, 20, 22, 24, 26, 28, 30, 32,
    ] {
        let mut old = empty_wire(major);
        let original = old.to_string();
        let admitted = AdmittedSuite::from_json(&original).unwrap();
        assert_eq!(admitted.original_json(), original);
        assert!(!admitted
            .suite()
            .to_canonical_json()
            .unwrap()
            .contains("scenario_initial_state"));
        old["provenance"]["scenario_initial_state"] = "empty".into();
        assert!(
            AdmittedSuite::from_json(&old.to_string()).is_err(),
            "legacy {major}"
        );
    }
}

#[test]
fn coverage_selection_preserves_requirement_and_exact_parent_bytes() {
    use ess_conformance::coverage::{AdmittedInput, Origins, Scope};
    let generated =
        ess_conformance::coverage_build::build(&model(), &[], Scope::System, Origins::Generated)
            .unwrap();
    let parent = generated.selected().original_json().to_owned();
    let id = generated
        .selected()
        .suite()
        .scenarios
        .keys()
        .next()
        .unwrap()
        .clone();
    let selected = generated.select(&[id]).unwrap();
    let bytes = selected.document().to_canonical_json().unwrap();
    let wire: serde_json::Value = serde_json::from_str(&bytes).unwrap();
    assert!(bytes.contains("scenario_initial_state"));
    assert_eq!(
        selected
            .selected()
            .suite()
            .provenance
            .suite_version
            .to_string(),
        "ess-conformance/35"
    );
    assert_eq!(
        selected
            .selected()
            .suite()
            .provenance
            .scenario_initial_state,
        generated
            .selected()
            .suite()
            .provenance
            .scenario_initial_state
    );
    let reparsed = AdmittedInput::from_json(&bytes).unwrap();
    assert_eq!(
        reparsed.selected().original_json(),
        selected.selected().original_json()
    );
    assert!(
        wire.to_string()
            .contains(&serde_json::to_string(&parent).unwrap()),
        "original parent is retained as an exact string"
    );
}
