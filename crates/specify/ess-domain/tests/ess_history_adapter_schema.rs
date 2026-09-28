//! `schemas/ess-history-adapter.schema.json` against the reader it claims to describe.
//!
//! `ess verify conform import-history` reads an `ess-history-adapter/1` document through
//! `ess_conformance::recorded::adapter`. The table in
//! `crates/verify/ess-conformance/tests/fixtures/recorded/adapter-documents.json` lists adapter
//! documents with the verdict that reader gives each: admitted or refused. This file holds the
//! schema to the same verdicts; ess-conformance's `recorded_adapter_documents` test holds the
//! reader to them, which ess-domain cannot do because it cannot call the reader. A document one
//! admits and the other refuses fails one of the two.
//!
//! The table is written in JSON, so it covers what a JSON document can say. The reader also reads
//! YAML, and a YAML spelling with no JSON equivalent (a non-string mapping key, a tag) is outside
//! what a JSON Schema can describe.

use serde_json::Value;

fn repository(relative: &str) -> String {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .join(relative);
    std::fs::read_to_string(&path).unwrap_or_else(|error| panic!("reading {relative}: {error}"))
}

fn validator() -> jsonschema::Validator {
    let schema: Value =
        serde_json::from_str(&repository("schemas/ess-history-adapter.schema.json"))
            .expect("the adapter schema is JSON");
    jsonschema::validator_for(&schema).expect("a usable schema")
}

/// `base` with each edit applied: `{pointer, value}` sets, `{pointer, remove: true}` removes.
fn apply(base: &Value, edits: &[Value]) -> Value {
    let mut document = base.clone();
    for edit in edits {
        let pointer = edit["pointer"].as_str().expect("an edit names a pointer");
        if pointer.is_empty() {
            document = edit["value"].clone();
            continue;
        }
        let (parent, key) = pointer.rsplit_once('/').expect("a pointer below the root");
        let object = document
            .pointer_mut(parent)
            .and_then(Value::as_object_mut)
            .unwrap_or_else(|| panic!("the edit's parent `{parent}` is an object"));
        if edit.get("remove") == Some(&Value::Bool(true)) {
            assert!(
                object.remove(key).is_some(),
                "`{pointer}` is there to remove"
            );
        } else {
            object.insert(key.to_owned(), edit["value"].clone());
        }
    }
    document
}

fn cases() -> Vec<(String, bool, Value)> {
    let table: Value = serde_json::from_str(&repository(
        "crates/verify/ess-conformance/tests/fixtures/recorded/adapter-documents.json",
    ))
    .expect("the table is JSON");
    table["cases"]
        .as_array()
        .expect("the table lists cases")
        .iter()
        .map(|case| {
            (
                case["name"].as_str().expect("a case is named").to_owned(),
                case["admitted"].as_bool().expect("a case has a verdict"),
                apply(
                    &table["base"],
                    case["edits"].as_array().expect("a case lists its edits"),
                ),
            )
        })
        .collect()
}

#[test]
fn the_schema_gives_every_tabled_document_the_readers_verdict() {
    let validator = validator();
    let cases = cases();
    assert!(
        cases.iter().filter(|(_, admitted, _)| *admitted).count() >= 5
            && cases.iter().filter(|(_, admitted, _)| !*admitted).count() >= 20,
        "the table holds too few cases to say anything"
    );
    let disagreements: Vec<String> = cases
        .iter()
        .filter(|(_, admitted, document)| validator.is_valid(document) != *admitted)
        .map(|(name, admitted, _)| {
            format!(
                "{name}: the reader {} it, the schema does not",
                if *admitted { "admits" } else { "refuses" }
            )
        })
        .collect();
    assert!(disagreements.is_empty(), "{}", disagreements.join("\n"));
}
