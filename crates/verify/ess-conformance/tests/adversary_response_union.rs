//! A response union must obey the same closed shape as its generated schema.
use ess_compiler::{resolve::compile, source::SourceMap};
use ess_conformance::{response::Observation, ScenarioStep};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::node::Node;
use std::collections::BTreeMap;

#[test]
fn actual_response_union_rejects_undeclared_members() {
    let source = r"
format: ess/4
system: example
version: v1
domain: example.api
types:
  - name: example.api.Result
    kind: union
    tag: kind
    variants:
      text: String
      count: Integer
events:
  - name: example.api.Returned
    fields:
      - {name: item, type: example.api.Result}
commands:
  - name: example.api.Fetch
    response:
      - {name: item, type: example.api.Result}
    outcomes:
      - name: returned
        emits: [example.api.Returned]
        payload:
          example.api.Returned:
            item: {response: item}
";
    let spec = Specification::assemble([(
        Source::new("response-union.yaml"),
        RawSpecFile::parse(source).unwrap(),
    )])
    .unwrap();
    let ir = compile(&spec, &SourceMap::new()).unwrap();
    let synthesis = ess_conformance::synthesize::synthesize(&ir);
    assert!(synthesis.refusals.is_empty(), "{:?}", synthesis.refusals);
    let observation: &Observation = synthesis
        .suite
        .scenarios
        .values()
        .flat_map(|scenario| &scenario.steps)
        .find_map(|step| match step {
            ScenarioStep::ExpectResponsePayload { response } => Some(response),
            _ => None,
        })
        .unwrap();
    let valid: BTreeMap<String, Node> =
        serde_json::from_str(r#"{"item":{"kind":"text","value":"ok"}}"#).unwrap();
    observation.compare(Some(&valid), &valid).unwrap();

    let invalid: BTreeMap<String, Node> =
        serde_json::from_str(r#"{"item":{"kind":"text","value":"ok","unexpected":true}}"#).unwrap();
    let result = observation.compare(Some(&invalid), &invalid);
    assert!(
        result.is_err(),
        "an undeclared union member must fail even when actual response and emitted payload agree: {result:?}"
    );
}
