//! Root selection must reuse the normative wire mapping without losing model identity.

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
use serde_json::{json, Value};

const SOURCE: &str = include_str!("fixtures/model-types.yaml");

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
