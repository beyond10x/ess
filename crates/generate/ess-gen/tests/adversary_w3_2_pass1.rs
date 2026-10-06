//! Adversary pass 1 on unit W3-2 (beyond10x/ess#445): the `AsyncAPI` reaction states a binding
//! constant as "the JSON value the invoked command receives" (`asyncapi.rs`, `literal_value`). The
//! same document types a `Decimal` as a decimal *string* (`types.rs`, `Primitive::Decimal`), and
//! the unit's own `x-ess-attributes` annotation writes a `Decimal` as "the decimal string a
//! `Decimal` is on the wire". A `Decimal` constant stated as a JSON number is a value the input's
//! own schema refuses.

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

const CALLS: &str = "format: ess/22
system: demo
version: v1
domain: demo.calls
events:
  - name: demo.calls.LegJoined
    fields:
      - {name: leg_id, type: String}
  - name: demo.calls.LegRecorded
    fields:
      - {name: leg_id, type: String}
      - {name: share, type: Decimal}
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
      - {name: share, type: Decimal}
    outcomes:
      - name: recorded
        emits: [demo.calls.LegRecorded]
        payload:
          demo.calls.LegRecorded: {leg_id: input.leg_id, share: input.share}
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
      share: 0.5
    delivery: at_least_once
    on_failure: drop
";

fn ir() -> EssIr {
    let raw = RawSpecFile::parse(CALLS).unwrap();
    let spec = Specification::assemble([(Source::new("calls.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap()
}

fn documents() -> Vec<serde_json::Value> {
    let generator = ess_gen::generator("asyncapi").expect("the generator exists");
    ess_gen::artifact::run(generator.as_ref(), &ir())
        .expect("generates")
        .into_iter()
        .map(|(path, artifact)| {
            if std::path::Path::new(&path)
                .extension()
                .is_some_and(|extension| extension.eq_ignore_ascii_case("json"))
            {
                serde_json::from_str(&artifact.contents).expect("JSON")
            } else {
                serde_yaml::from_str(&artifact.contents).expect("YAML")
            }
        })
        .collect()
}

fn walk<'a>(value: &'a serde_json::Value, visit: &mut impl FnMut(&'a serde_json::Value)) {
    visit(value);
    match value {
        serde_json::Value::Object(map) => map.values().for_each(|value| walk(value, visit)),
        serde_json::Value::Array(items) => items.iter().for_each(|value| walk(value, visit)),
        _ => {}
    }
}

#[test]
fn adv_w3_2_asyncapi_decimal_constant_is_a_value_of_its_input_schema() {
    let documents = documents();
    let mut decimal_schemas = 0;
    let mut share = Vec::new();
    for document in &documents {
        walk(document, &mut |node| {
            if node.get("format").and_then(serde_json::Value::as_str) == Some("decimal") {
                assert_eq!(
                    node["type"], "string",
                    "a Decimal is a decimal string: {node}"
                );
                decimal_schemas += 1;
            }
            if node.get("target").and_then(serde_json::Value::as_str) == Some("share")
                && node["source"]["kind"] == "literal"
            {
                share.push(node["source"]["value"].clone());
            }
        });
    }
    assert_ne!(
        decimal_schemas, 0,
        "the document types `share` as a decimal string"
    );
    assert_ne!(share.len(), 0, "the reaction states the constant");
    for value in share {
        assert_eq!(
            value,
            serde_json::json!("0.5"),
            "the `Decimal` constant is stated as a value its input's decimal-string schema admits"
        );
    }
}
