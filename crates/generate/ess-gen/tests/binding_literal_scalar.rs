//! A binding constant over a `Boolean`, `Integer` or `Decimal` input (beyond10x/ess#445) is
//! projected typed: the `AsyncAPI` binding extension carries `true` and `3` as JSON scalars and
//! `0.5` as the decimal string its input is typed as, and the generated documentation says the
//! compiler checked the value against the input.

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

const CALLS: &str = "format: ess/22
system: demo
version: v1
domain: demo.calls
types:
  - {name: demo.calls.Weight, kind: newtype, of: Integer}
events:
  - name: demo.calls.LegJoined
    fields:
      - {name: leg_id, type: String}
  - name: demo.calls.LegRecorded
    fields:
      - {name: leg_id, type: String}
commands:
  - name: demo.calls.JoinLeg
    input:
      - {name: leg_id, type: String}
    outcomes:
      - name: joined
        emits: [demo.calls.LegJoined]
        payload:
          demo.calls.LegJoined: {leg_id: input.leg_id}
  - name: demo.calls.RecordLeg
    input:
      - {name: leg_id, type: String}
      - {name: is_bridged, type: Boolean}
      - {name: weight, type: demo.calls.Weight}
      - {name: share, type: Decimal}
      - {name: template, type: String}
    outcomes:
      - name: recorded
        emits: [demo.calls.LegRecorded]
        payload:
          demo.calls.LegRecorded: {leg_id: input.leg_id}
components:
  - component: calls-service
    summary: Records the legs of a call.
    owns:
      domains: [demo.calls]
    accepts:
      commands: [demo.calls.JoinLeg, demo.calls.RecordLeg]
    publishes:
      events: [demo.calls.LegJoined, demo.calls.LegRecorded]
bindings:
  - id: joined
    when: {event: demo.calls.LegJoined}
    invoke: {command: demo.calls.RecordLeg}
    mapping:
      leg_id: event.leg_id
      is_bridged: true
      weight: 3
      share: 0.5
      template: invoice-created
    delivery: at_least_once
    on_failure: drop
";

fn ir() -> EssIr {
    let raw = RawSpecFile::parse(CALLS).unwrap();
    let spec = Specification::assemble([(Source::new("calls.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap()
}

fn artifacts(name: &str) -> Vec<(String, String)> {
    let generator = ess_gen::generator(name).expect("the generator exists");
    ess_gen::artifact::run(generator.as_ref(), &ir())
        .expect("generates")
        .into_iter()
        .map(|(path, artifact)| (path, artifact.contents))
        .collect()
}

/// Every `{"kind":"literal"}` mapped source of the `AsyncAPI` document, by target.
fn literals(value: &serde_json::Value, out: &mut Vec<(String, serde_json::Value)>) {
    match value {
        serde_json::Value::Object(map) => {
            if let (Some(target), Some(source)) = (map.get("target"), map.get("source")) {
                if source["kind"] == "literal" {
                    out.push((
                        target.as_str().unwrap_or_default().to_owned(),
                        source["value"].clone(),
                    ));
                }
            }
            map.values().for_each(|value| literals(value, out));
        }
        serde_json::Value::Array(items) => items.iter().for_each(|value| literals(value, out)),
        _ => {}
    }
}

#[test]
fn binding_literal_asyncapi_and_docs_read_the_constant_typed() {
    let mut found = Vec::new();
    for (path, contents) in artifacts("asyncapi") {
        let document: serde_json::Value = if std::path::Path::new(&path)
            .extension()
            .is_some_and(|extension| extension.eq_ignore_ascii_case("json"))
        {
            serde_json::from_str(&contents).expect("JSON")
        } else {
            serde_yaml::from_str(&contents).expect("YAML")
        };
        literals(&document, &mut found);
    }
    found.sort_by(|a, b| a.0.cmp(&b.0));
    assert_eq!(
        found,
        vec![
            ("is_bridged".to_owned(), serde_json::json!(true)),
            ("share".to_owned(), serde_json::json!("0.5")),
            ("template".to_owned(), serde_json::json!("invoice-created")),
            ("weight".to_owned(), serde_json::json!(3)),
        ]
    );

    let docs: String = artifacts("docs")
        .into_iter()
        .map(|(_, contents)| contents)
        .collect();
    assert!(
        docs.contains(
            "the literal `true`. The compiler verified that this is a value of `Boolean`."
        ),
        "{docs}"
    );
    assert!(
        docs.contains("the literal `3`. The compiler verified that this is a value of `Integer`."),
        "{docs}"
    );
}
