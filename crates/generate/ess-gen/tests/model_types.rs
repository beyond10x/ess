//! Root selection must reuse the normative wire mapping without losing model identity.

use std::collections::BTreeSet;

use ess_compiler::{resolve::compile, source::SourceMap, EssIr};
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source,
};
use ess_gen::{
    artifact::run,
    schema::{JsonSchema, ModelRoot, ModelTypes},
};
use serde_json::{json, Value};

const SOURCE: &str = include_str!("fixtures/model-types.yaml");
const EVENTS: &str = include_str!("fixtures/model-event-types.yaml");

fn event_roots() -> BTreeSet<ModelRoot> {
    BTreeSet::from([ModelRoot::Event("telemetry.data.Recorded".parse().unwrap())])
}

#[test]
fn event_root_selects_existing_payload_and_exact_closure() {
    let ir = compiled(EVENTS);
    let selection = ModelTypes::select_roots(&ir, &event_roots()).unwrap();
    let artifacts = run(&JsonSchema, &ir).unwrap();
    let mut original: Value = serde_json::from_str(
        &artifacts["schema/events/telemetry.data.Recorded.schema.json"].contents,
    )
    .unwrap();
    let mut definitions = original
        .as_object_mut()
        .unwrap()
        .remove("$defs")
        .unwrap()
        .as_object()
        .unwrap()
        .clone();
    for framing in ["$schema", "x-ess-provenance"] {
        original.as_object_mut().unwrap().remove(framing);
    }
    assert_eq!(selection.definitions()["telemetry.data.Recorded"], original);
    definitions.insert("telemetry.data.Recorded".into(), original);
    assert_eq!(
        serde_json::to_value(selection.definitions()).unwrap(),
        Value::Object(definitions)
    );
    assert_eq!(selection.definitions().len(), 3);
    assert!(!selection
        .definitions()
        .contains_key("telemetry.data.Unused"));
    assert_eq!(
        selection.roots(),
        &BTreeSet::from(["telemetry.data.Recorded".into()])
    );
    assert_eq!(
        selection.binary64_locations(),
        &BTreeSet::from([
            "/$defs/telemetry.data.Recorded/properties/weight".into(),
            "/$defs/telemetry.data.Recorded/properties/measurements/additionalProperties/anyOf/0"
                .into(),
        ])
    );
}

#[test]
fn mixed_roots_share_reachable_closure_and_preserve_legacy_selection() {
    let ir = compiled(EVENTS);
    let mut roots = event_roots();
    roots.insert(ModelRoot::Type("telemetry.data.Details".parse().unwrap()));
    let selected = ModelTypes::select_roots(&ir, &roots).unwrap();
    assert_eq!(selected.definitions().len(), 3);
    assert_eq!(selected.roots().len(), 2);
    roots.remove(&ModelRoot::Event(
        "telemetry.data.Recorded".parse().unwrap(),
    ));
    let legacy =
        ModelTypes::select(&ir, &BTreeSet::from(["telemetry.data.Details".into()])).unwrap();
    assert_eq!(
        legacy.to_json(),
        ModelTypes::select_roots(&ir, &roots).unwrap().to_json()
    );
    assert!(ModelTypes::select(&ir, &BTreeSet::from(["telemetry.data.Recorded".into()])).is_err());
    assert!(ModelTypes::select_roots(
        &ir,
        &BTreeSet::from([ModelRoot::Event("telemetry.data.Details".parse().unwrap())])
    )
    .is_err());
}

#[test]
fn typed_event_provenance_changes_only_for_selected_contract() {
    let before = ModelTypes::select_roots(&compiled(EVENTS), &event_roots()).unwrap();
    let unrelated = ModelTypes::select_roots(
        &compiled(&EVENTS.replace("ignored, type: Boolean", "ignored, type: Integer")),
        &event_roots(),
    )
    .unwrap();
    assert_eq!(before.definitions(), unrelated.definitions());
    assert_eq!(
        before.provenance().contract_digest,
        unrelated.provenance().contract_digest
    );
    assert_ne!(
        before.provenance().source_digest,
        unrelated.provenance().source_digest
    );
    for source in [
        EVENTS.replace("wire: record-id", "wire: event-id"),
        EVENTS.replace("title, type: String", "title, type: Boolean"),
    ] {
        let after = ModelTypes::select_roots(&compiled(&source), &event_roots()).unwrap();
        assert_ne!(
            before.provenance().contract_digest,
            after.provenance().contract_digest
        );
        assert_ne!(before.definitions(), after.definitions());
    }
}

#[test]
fn event_wire_collisions_and_cross_kind_source_collisions_refuse() {
    let duplicate_wire = EVENTS.replace("wire: record-id", "wire: details");
    assert!(Specification::assemble([(
        Source::document(),
        RawSpecFile::parse(&duplicate_wire).unwrap()
    )])
    .is_err());
    let duplicate_name = EVENTS.replace(
        "name: telemetry.data.Unused",
        "name: telemetry.data.Recorded",
    );
    assert!(Specification::assemble([(
        Source::document(),
        RawSpecFile::parse(&duplicate_name).unwrap()
    )])
    .is_err());
}

fn compiled(text: &str) -> EssIr {
    let mut sources = SourceMap::new();
    sources.insert(Source::DOCUMENT, text.to_owned());
    let specification =
        Specification::assemble([(Source::document(), RawSpecFile::parse(text).unwrap())]).unwrap();
    compile(&specification, &sources).unwrap()
}

#[test]
fn selection_is_the_exact_existing_schema_closure_with_model_provenance() {
    let ir = compiled(SOURCE);
    let roots = BTreeSet::from(["sample.data.Record".to_owned()]);
    let selected = ModelTypes::select(&ir, &roots).unwrap();
    let schema: Value = serde_json::from_str(&selected.to_json()).unwrap();
    let artifacts = run(&JsonSchema, &ir).unwrap();
    let original: Value =
        serde_json::from_str(&artifacts["schema/types/sample.data.Record.schema.json"].contents)
            .unwrap();
    assert_eq!(schema["$defs"], original["$defs"]);
    assert_eq!(selected.definitions().len(), 4);
    assert!(!selected.definitions().contains_key("sample.data.OtherId"));
    assert_eq!(
        selected.newtypes(),
        &BTreeSet::from(["sample.data.Id".to_owned()])
    );
    assert_eq!(selected.provenance().source_digest, ir.source_digest());
    assert_eq!(
        selected.to_json(),
        ModelTypes::select(&ir, &roots).unwrap().to_json()
    );
    let record = &schema["$defs"]["sample.data.Record"];
    assert!(record["required"]
        .as_array()
        .unwrap()
        .contains(&json!("record-id")));
    assert!(!record["required"]
        .as_array()
        .unwrap()
        .contains(&json!("label")));
    assert_eq!(record["properties"]["label"]["type"], "string");
    assert_eq!(
        record["properties"]["numbers"]["additionalProperties"]["anyOf"][0]["type"],
        "string"
    );
    assert_eq!(
        record["properties"]["samples"]["items"]["anyOf"][1]["type"],
        "null"
    );
    assert_eq!(
        schema["$defs"]["sample.data.Choice"]["oneOf"][0]["properties"]["value"]["const"],
        "item"
    );
    assert!(
        schema["$defs"]["sample.data.Choice"]["oneOf"][0]["properties"]
            .get("content")
            .is_some()
    );
}

#[test]
fn missing_roots_and_wire_collisions_are_refused_before_map_serialization() {
    let ir = compiled(SOURCE);
    assert_eq!(
        ModelTypes::select(&ir, &BTreeSet::new()).unwrap_err()[0].rule,
        "empty_roots"
    );
    let errors = ModelTypes::select(
        &ir,
        &BTreeSet::from(["missing.One".to_owned(), "missing.Two".to_owned()]),
    )
    .unwrap_err();
    assert_eq!(errors.len(), 2);
    assert!(errors.iter().all(|error| error.rule == "unknown_root"));
    let collision = SOURCE.replace("wire: record-id", "wire: score");
    let errors =
        Specification::assemble([(Source::document(), RawSpecFile::parse(&collision).unwrap())])
            .unwrap_err();
    assert!(errors.to_string().contains("wire field \"score\""));
}

#[test]
fn model_only_changes_outside_the_selection_do_not_expand_its_contract() {
    let roots = BTreeSet::from(["sample.data.Record".to_owned()]);
    let before = ModelTypes::select(&compiled(SOURCE), &roots).unwrap();
    let after = ModelTypes::select(
        &compiled(&SOURCE.replace(
            "name: sample.data.OtherId\n    kind: newtype\n    of: String",
            "name: sample.data.OtherId\n    kind: newtype\n    of: Uuid",
        )),
        &roots,
    )
    .unwrap();
    assert_eq!(before.definitions(), after.definitions());
    assert_eq!(
        before.provenance().contract_digest,
        after.provenance().contract_digest
    );
    assert_ne!(
        before.provenance().source_digest,
        after.provenance().source_digest
    );
}
