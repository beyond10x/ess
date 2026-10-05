//! From `ess/23` a refusal selected by `when_subject:` carries `complete_refusal`
//! (beyond10x/ess#461). Moving a model from `ess/22` to `ess/23` therefore changes that header on
//! each such refusal, and the semantic diff reports it as the existing typed
//! `outcome-observation-changed` delta, in both directions, and as nothing unclassified.
use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_diff::diff;
use ess_domain::{spec::RawSpecFile, system::Source, Specification};

const MODEL: &str =
    include_str!("../../ess-conformance/tests/fixtures/subject-fact-complete-refusal.yaml");

fn ir(text: &str) -> EssIr {
    let spec = Specification::assemble([(
        Source::new("instance.yaml"),
        RawSpecFile::parse(text).unwrap(),
    )])
    .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap()
}

#[test]
fn complete_refusal_header_is_an_outcome_observation_change() {
    let old = ir(&MODEL.replace("format: ess/23\n", "format: ess/22\n"));
    let new = ir(MODEL);
    for (before, after, expected) in [(&old, &new, true), (&new, &old, false)] {
        let text = diff(before, after).unwrap().to_canonical_json();
        assert!(!text.contains("unclassified-changed"), "{text}");
        let json: serde_json::Value = serde_json::from_str(&text).unwrap();
        let changes = json["changes"].as_array().unwrap();
        let mut named: Vec<String> = Vec::new();
        for change in changes {
            let delta = &change["change"]["changed"];
            assert_eq!(delta["kind"], "outcome-observation-changed", "{text}");
            assert_eq!(delta["after"], expected, "{text}");
            named.push(delta["outcome"].as_str().unwrap_or_default().to_owned());
        }
        named.sort();
        assert_eq!(named, ["not-active", "seed-change-refused"], "{text}");
    }
}
