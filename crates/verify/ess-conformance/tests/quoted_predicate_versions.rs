//! New operand meaning has explicit reader authority, including nested selection predicates.
use ess_conformance::{scenario::SuiteFormat, AdmittedSuite, ConformanceSuite};
use serde_json::{json, Value};

fn document(predicate: &Value) -> Value {
    json!({"provenance":{"suite_version":"ess-conformance/4","system":"quoted","specification_version":"v1","spec_digest":"a".repeat(64),"contract_digest":"b".repeat(64)},"scenarios":{"quoted.core/authored/operand":{"purpose":"Check version authority","steps":[{"step":"expect_view","view":"quoted.core.Rows","expectation":{"expect":"satisfies","predicate":predicate}}],"source":[]}}})
}
#[test]
fn versions_distinguish_lossless_reader_from_compatible_fallback() {
    for text in ["true", "1", "  padded  ", "\"quoted\""] {
        let raw = document(&json!({"not":{"all":[{"to":{"eq":format!("\"{text}\"")}}]}}));
        let mut typed: ConformanceSuite = serde_json::from_value(raw.clone()).unwrap();
        assert!(
            ess_conformance::quoted_predicate_format::used_by(&typed),
            "{text:?}"
        );
        assert!(typed.to_canonical_json().is_err());
        assert!(AdmittedSuite::from_json(&raw.to_string()).is_err());
        typed.select_fresh_format();
        assert_eq!(typed.provenance.suite_version.major(), 8);
        AdmittedSuite::from_json(&typed.to_canonical_json().unwrap()).unwrap();
    }
    for predicate in [
        json!({"to":{"eq":"\"busy\" status"}}),
        json!("to == \"a.b\""),
        json!({"to":{"eq":"Ready"}}),
    ] {
        let mut typed: ConformanceSuite = serde_json::from_value(document(&predicate)).unwrap();
        assert!(!ess_conformance::quoted_predicate_format::used_by(&typed));
        typed.select_fresh_format();
        assert_eq!(typed.provenance.suite_version.major(), 4);
        AdmittedSuite::from_json(&typed.to_canonical_json().unwrap()).unwrap();
    }
    // Original structured bytes need the new reader even when the canonical compact spelling
    // of the normalized value would have been compatible.
    let mut raw = document(&json!({"to":{"eq":"\"a.b\""}}));
    assert!(AdmittedSuite::from_json(&raw.to_string()).is_err());
    raw["provenance"]["suite_version"] = json!("ess-conformance/8");
    AdmittedSuite::from_json(&raw.to_string()).unwrap();
}

#[test]
fn versions_do_not_interpret_literal_payloads_as_predicates() {
    let mut raw = document(&json!(true));
    raw["scenarios"]["quoted.core/authored/operand"]["steps"] = json!([{"step":"execute_command","command":"quoted.core.Send","input":{"payload":{"kind":"literal","value":{"to":{"eq":"\"true\""}}}}}]);
    let suite = AdmittedSuite::from_json(&raw.to_string()).unwrap();
    assert!(!ess_conformance::quoted_predicate_format::used_by(
        suite.suite()
    ));
}

#[test]
fn versions_cover_first_selection_predicates() {
    use ess_compiler::{ir::ResolvedMappingValue, resolve::compile_locating, source::SourceMap};
    use ess_domain::{
        selection::SelectionOperation, spec::RawSpecFile, system::Source, Specification,
    };
    let source = include_str!("fixtures/binding-selection.yaml");
    let raw = RawSpecFile::parse(source).unwrap();
    let spec = Specification::assemble([(Source::new("selection.yaml"), raw)]).unwrap();
    let mut sources = SourceMap::new();
    sources.insert("selection.yaml", source);
    let ir = compile_locating(&spec, &sources, &["selection.yaml"]).unwrap();
    let binding = ir.bindings().values().next().unwrap();
    let mapped = &binding.mapping[0];
    let ResolvedMappingValue::Selection {
        selector,
        projection,
        ..
    } = &mapped.value
    else {
        panic!("selection");
    };
    let mut observation = ess_conformance::selection::Observation::of(
        &ir,
        binding,
        *selector,
        projection,
        &mapped.target_type,
    )
    .unwrap();
    let SelectionOperation::First { predicate, .. } = &mut observation.plan.selectors[2].operation
    else {
        panic!("first");
    };
    *predicate =
        ess_primitives::predicate::Predicate::parse_expression("item.id == 'true'").unwrap();
    let mut raw = document(&json!(true));
    raw["provenance"]["suite_version"] = json!("ess-conformance/6");
    raw["scenarios"]["quoted.core/authored/operand"]["steps"] = json!([{"step":"expect_invocation","binding":"select","command":"selection.core.Apply","input":{"id":{"kind":"observed_selection","event":"selection.core.Changed","selection":observation}}}]);
    let mut suite: ConformanceSuite = serde_json::from_value(raw.clone()).unwrap();
    assert!(ess_conformance::quoted_predicate_format::used_by(&suite));
    assert!(suite.to_canonical_json().is_err());
    assert!(AdmittedSuite::from_json(&raw.to_string()).is_err());
    suite.provenance.suite_version = SuiteFormat::parse("ess-conformance/8").unwrap();
    // Typed traversal also covers values placed in query parameters, even though persisted
    // admission independently refuses selection vocabulary in that position.
    assert!(ess_conformance::quoted_predicate_format::used_by(&suite));
}
