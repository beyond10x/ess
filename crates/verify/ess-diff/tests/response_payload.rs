//! Response declarations and source ownership use explicit semantic delta vocabulary.
use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_diff::{diff, EssDelta, RawEssDelta};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
const MODEL: &str = include_str!("../../ess-conformance/tests/fixtures/response-payload.yaml");
fn ir(text: &str) -> EssIr {
    let spec = Specification::assemble([(
        Source::new("response.yaml"),
        RawSpecFile::parse(text).unwrap(),
    )])
    .unwrap();
    compile(&spec, &SourceMap::new()).unwrap()
}
#[test]
fn response_attachment_and_mapping_changes_require_diff4() {
    for changed in [MODEL.replace("    response:\n      - {name: item, type: demo.api.Item}", "    response:\n      - {name: item, type: demo.api.Item}\n      - {name: message, type: String}"), MODEL.replace("item: {response: item}", "item: {generated: true}")] {
        let delta=diff(&ir(MODEL),&ir(&changed)).unwrap();
        assert_eq!(delta.format.to_string(),"ess-diff/4");
        let json=delta.to_canonical_json();
        assert!(json.contains("response-changed") || json.contains("outcome-response-payload-changed"));
        let downgraded=json.replace("ess-diff/4","ess-diff/3");
        let raw:RawEssDelta=serde_json::from_str(&downgraded).unwrap();
        assert!(EssDelta::try_from(raw).is_err());
    }
}
#[test]
fn unchanged_legacy_payload_vocabulary_does_not_require_diff4() {
    let before = MODEL
        .replace("ess/4", "ess/3")
        .replace(
            "    response:\n      - {name: item, type: demo.api.Item}\n",
            "",
        )
        .replace("            item: {response: item}\n", "")
        .replace("receipt: {generated: true}", "receipt: old");
    let after = before.replace("receipt: old", "receipt: new");
    let delta = diff(&ir(&before), &ir(&after)).unwrap();
    assert_eq!(delta.format.to_string(), "ess-diff/2");
}
