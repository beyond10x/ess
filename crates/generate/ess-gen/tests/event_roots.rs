//! An event's payload is selectable as a generated type (beyond10x/ess#393).
//!
//! `--root <event>` selects the event's fields as a struct under the event's name, with the
//! event's wire names and the closure of the types its fields reach. The definition is the same
//! object the JSON Schema projection publishes for the event, so a producer library holds exactly
//! what the event carries and the model declares those fields once.

use std::collections::BTreeSet;

use ess_compiler::{resolve::compile, source::SourceMap, EssIr};
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source,
};
use ess_gen::{
    artifact::run,
    schema::{JsonSchema, ModelTypes},
};
use serde_json::Value;

const SOURCE: &str = "format: ess/20
system: demo
version: v1
domain: demo.metering
types:
  - name: demo.metering.Source
    kind: struct
    fields:
      - {name: service, type: String}
  - name: demo.metering.Unused
    kind: struct
    fields:
      - {name: nothing, type: String}
events:
  - name: demo.metering.UsageRecorded
    naming:
      wire: usage
      display: Usage recorded
    fields:
      - {name: id, type: String}
      - {name: source, type: demo.metering.Source}
      - {name: note, type: 'Optional<String>', presence: null_when_absent}
      - {name: created_at, type: Timestamp, wire: createdAt}
";

fn compiled(text: &str) -> EssIr {
    let mut sources = SourceMap::new();
    sources.insert(Source::DOCUMENT, text.to_owned());
    let specification =
        Specification::assemble([(Source::document(), RawSpecFile::parse(text).unwrap())]).unwrap();
    compile(&specification, &sources).unwrap()
}

fn selected(roots: &[&str]) -> Value {
    let ir = compiled(SOURCE);
    let roots = roots.iter().map(|root| (*root).to_owned()).collect();
    let selection = ModelTypes::select(&ir, &roots).expect("selects");
    serde_json::from_str(&selection.to_json()).expect("JSON")
}

#[test]
fn an_event_root_selects_its_payload_and_the_types_it_reaches() {
    let schema = selected(&["demo.metering.UsageRecorded"]);
    let defs = schema["$defs"].as_object().expect("defs");
    assert_eq!(
        defs.keys().map(String::as_str).collect::<Vec<_>>(),
        ["demo.metering.Source", "demo.metering.UsageRecorded"],
        "{schema:#}"
    );
    let payload = &defs["demo.metering.UsageRecorded"];
    assert_eq!(payload["x-ess-kind"], "event-payload", "{payload:#}");
    assert_eq!(payload["title"], "Usage recorded", "{payload:#}");
    let properties = payload["properties"].as_object().expect("properties");
    assert_eq!(
        properties
            .keys()
            .map(String::as_str)
            .collect::<BTreeSet<_>>(),
        BTreeSet::from(["id", "source", "note", "createdAt"]),
        "the event wire names"
    );
}

#[test]
fn the_payload_is_the_object_the_event_schema_publishes() {
    let ir = compiled(SOURCE);
    let artifacts = run(&JsonSchema, &ir).expect("generates");
    let published: Value = serde_json::from_str(
        &artifacts["schema/events/demo.metering.UsageRecorded.schema.json"].contents,
    )
    .expect("JSON");
    let schema = selected(&["demo.metering.UsageRecorded"]);
    let payload = &schema["$defs"]["demo.metering.UsageRecorded"];
    for keyword in ["type", "properties", "required", "additionalProperties"] {
        assert_eq!(payload[keyword], published[keyword], "`{keyword}` differs");
    }
}

#[test]
fn an_event_and_a_type_root_share_one_closure() {
    let schema = selected(&["demo.metering.UsageRecorded", "demo.metering.Unused"]);
    assert_eq!(
        schema["$defs"].as_object().map(serde_json::Map::len),
        Some(3)
    );
}

#[test]
fn an_unknown_root_names_both_kinds_it_looked_for() {
    let ir = compiled(SOURCE);
    let errors = ModelTypes::select(&ir, &BTreeSet::from(["demo.metering.Missing".to_owned()]))
        .expect_err("refused");
    assert_eq!(errors[0].rule, "unknown_root");
    assert!(
        errors[0].detail.contains("type or event"),
        "{}",
        errors[0].detail
    );
}
