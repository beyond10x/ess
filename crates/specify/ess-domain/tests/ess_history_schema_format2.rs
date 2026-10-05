//! `schemas/ess-history.schema.json` admits what the Rust, Go and TypeScript writers write, in both
//! formats, and refuses what the reader refuses about `decision_time` (beyond10x/ess#244).
//!
//! The two documents are `crates/verify/ess-conformance/tests/fixtures/history2/written.json` and `written-format1.json`:
//! the bytes every writer produces for the same operations (`history_format2.rs`,
//! `src/ts/explore.test.ts`), so a writer validating its own output against the schema passes in
//! both formats.

use serde_json::{json, Value};

fn validator() -> jsonschema::Validator {
    let text = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../schemas/ess-history.schema.json"),
    )
    .expect("the history schema is committed");
    let schema: Value = serde_json::from_str(&text).expect("the history schema is JSON");
    jsonschema::validator_for(&schema).expect("a usable schema")
}

fn written(name: &str) -> Value {
    let text = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../verify/ess-conformance/tests/fixtures/history2")
            .join(name),
    )
    .expect("the history2 fixture is committed");
    serde_json::from_str(&text).expect("JSON")
}

#[test]
fn the_writers_documents_validate_in_both_formats() {
    let validator = validator();
    for name in ["written.json", "written-format1.json"] {
        let document = written(name);
        let errors: Vec<String> = validator
            .iter_errors(&document)
            .map(|error| error.to_string())
            .collect();
        assert_eq!(errors.len(), 0, "{name}: {errors:?}");
    }
}

#[test]
fn the_schema_refuses_what_the_reader_refuses_about_decision_time() {
    let validator = validator();
    let mut null = written("written.json");
    null["operations"][2]["decision_time"] = Value::Null;
    assert!(!validator.is_valid(&null), "an explicit null");

    let mut offset = written("written.json");
    offset["operations"][0]["decision_time"] = json!("2000-06-01T01:00:00+01:00");
    assert!(
        !validator.is_valid(&offset),
        "another spelling of the instant"
    );

    let mut trailing = written("written.json");
    trailing["operations"][0]["decision_time"] = json!("2000-06-01T00:00:00.50Z");
    assert!(!validator.is_valid(&trailing), "a trailing zero");

    let mut one = written("written-format1.json");
    one["operations"][4]["decision_time"] = json!("2000-07-01T00:00:00.5Z");
    assert!(
        !validator.is_valid(&one),
        "an ess-history/1 document carrying decision_time on its last operation"
    );

    let mut future = written("written.json");
    future["format"] = json!("ess-history/3");
    assert!(!validator.is_valid(&future), "an unknown future major");
}
