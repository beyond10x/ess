//! A poll's interval and required authority are semantic changes without changing its command.
use ess_compiler::{resolve::compile_locating, source::SourceMap, EssIr};
use ess_domain::{system::Source, RawSpecFile, Specification};
fn model(text: &str) -> EssIr {
    let specification = Specification::assemble([(
        Source::new("periodic.yaml"),
        RawSpecFile::parse(text).unwrap(),
    )])
    .unwrap();
    let mut sources = SourceMap::new();
    sources.insert("periodic.yaml", text);
    compile_locating(&specification, &sources, &["periodic.yaml"]).unwrap()
}
#[test]
fn periodic_interval_and_authority_changes_have_typed_diff_entries() {
    let source = include_str!("../../../specify/ess-domain/tests/fixtures/periodic.yaml");
    for changed in [
        source.replace("PT2S", "PT3S"),
        source.replace("authenticated-session-status", "authenticated-peer-status"),
    ] {
        let delta = ess_diff::diff(&model(source), &model(&changed)).unwrap();
        assert_eq!(delta.format.major(), 3);
        assert!(delta
            .to_canonical_json_for(ess_diff::DeltaFormat::parse("ess-diff/2").unwrap())
            .is_err());
        let raw: ess_diff::RawEssDelta = serde_json::from_str(&delta.to_canonical_json()).unwrap();
        let roundtrip = ess_diff::EssDelta::try_from(raw).unwrap();
        assert_eq!(roundtrip, delta);
        let json = serde_json::to_value(delta).unwrap();
        let changes = json["changes"].as_array().unwrap();
        assert!(
            changes
                .iter()
                .any(|change| change.to_string().contains("cause-changed")),
            "{json}"
        );
        assert!(
            !changes
                .iter()
                .any(|change| change.to_string().contains("mapping-value-changed")),
            "{json}"
        );
        assert!(
            !changes
                .iter()
                .any(|change| change.to_string().contains("command-changed")),
            "{json}"
        );
    }
}
