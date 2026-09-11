//! Response-only union closure preserves optional content and the alternate content key.
use ess_compiler::{resolve::compile, source::SourceMap};
use ess_conformance::ScenarioStep;
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::node::Node;
use std::collections::BTreeMap;

#[test]
fn union_response_checks_closed_keys_without_changing_optional_content() {
    let mut contracts = Vec::new();
    for tag in ["kind", "value"] {
        let content = ess_gen::schema::union_content_key(tag);
        let source = include_str!("fixtures/response-union.yaml")
            .replace("tag: kind", &format!("tag: {tag}"))
            .replace("text: String", "text: Optional<String>");
        let spec = Specification::assemble([(
            Source::new("union.yaml"),
            RawSpecFile::parse(&source).unwrap(),
        )])
        .unwrap();
        let ir = compile(&spec, &SourceMap::new()).unwrap();
        let synthesis = ess_conformance::synthesize::synthesize(&ir);
        assert!(synthesis.refusals.is_empty(), "{:?}", synthesis.refusals);
        let observation = synthesis
            .suite
            .scenarios
            .values()
            .flat_map(|s| &s.steps)
            .find_map(|step| {
                if let ScenarioStep::ExpectResponsePayload { response } = step {
                    Some(response)
                } else {
                    None
                }
            })
            .unwrap();
        for inner in [
            serde_json::json!({tag: "text", content: "ok"}),
            serde_json::json!({tag: "text", content: null}),
            serde_json::json!({tag: "text"}),
        ] {
            let body: BTreeMap<String, Node> =
                serde_json::from_value(serde_json::json!({"item": inner})).unwrap();
            observation.compare(Some(&body), &body).unwrap();
            let mut invalid = body.clone();
            let Node::Map(value) = invalid.get_mut("item").unwrap() else {
                panic!("union");
            };
            value.insert("unexpected".into(), Node::Bool(true));
            assert!(observation.compare(Some(&invalid), &invalid).is_err());
        }
        for inner in [
            serde_json::json!({tag: "unknown", content: "ok"}),
            serde_json::json!({tag: "count"}),
            serde_json::json!({tag: "text", content: true}),
        ] {
            let body: BTreeMap<String, Node> =
                serde_json::from_value(serde_json::json!({"item":inner})).unwrap();
            assert!(observation.compare(Some(&body), &body).is_err());
        }
        contracts.push(observation.clone());
    }
    if let Some(path) = std::env::var_os("ESS_RESPONSE_UNION_OUT") {
        std::fs::write(path, serde_json::to_string(&contracts).unwrap()).unwrap();
    }
}
