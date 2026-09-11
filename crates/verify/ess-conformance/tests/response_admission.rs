//! Direct typed callers must obey the same response boundary as persisted readers.
use ess_compiler::{resolve::compile, source::SourceMap};
use ess_conformance::{scenario::SuiteFormat, ConformanceSuite, ScenarioStep};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};

fn suite() -> ConformanceSuite {
    let text = include_str!("fixtures/response-payload.yaml");
    let spec = Specification::assemble([(
        Source::new("response.yaml"),
        RawSpecFile::parse(text).unwrap(),
    )])
    .unwrap();
    let mut sources = SourceMap::new();
    sources.insert("response.yaml", text);
    ess_conformance::synthesize(&compile(&spec, &sources).unwrap()).suite
}

#[test]
fn typed_response_writers_refuse_old_pins_and_invalid_observations() {
    let admitted = suite();
    assert!(admitted.to_canonical_json().is_ok());
    assert!(admitted.to_compact_json().is_ok());
    let mut old = admitted.clone();
    old.provenance.suite_version = SuiteFormat::parse("ess-conformance/4").unwrap();
    assert!(old.to_canonical_json().is_err());
    assert!(old.to_compact_json().is_err());

    let mut malformed = admitted;
    let response = malformed
        .scenarios
        .values_mut()
        .flat_map(|scenario| &mut scenario.steps)
        .find_map(|step| match step {
            ScenarioStep::ExpectResponsePayload { response } => Some(response),
            _ => None,
        })
        .unwrap();
    *response.mappings.values_mut().next().unwrap() = "undeclared_response_field".into();
    assert!(malformed.to_canonical_json().is_err());
    assert!(malformed.to_compact_json().is_err());
}
