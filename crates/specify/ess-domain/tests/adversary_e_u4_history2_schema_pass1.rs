//! Adversary, E-U4 pass 1: `schemas/ess-history.schema.json` against the `ess-history/2` reader.
//!
//! `ess_history_schema_adversary.rs` states the rule this file holds the history2 change to: "A
//! document the schema admits and the reader refuses as malformed is a writer that passes its own
//! validation and is then refused." The reader refuses an `ess-history/1` document whose operation
//! carries `decision_time` (`crates/verify/ess-conformance/src/history.rs`, `read_admitting`,
//! pinned by `history_format2.rs::a_format1_relabel_carrying_a_time_is_refused`), and draft 04 can
//! say the same thing (`anyOf` over the format and the operations' items).

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

/// The committed format 2 document the Rust, Go and TypeScript writers all write.
fn written() -> Value {
    let text = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../verify/ess-conformance/tests/fixtures/history2/written.json"),
    )
    .expect("the history2 fixture is committed");
    serde_json::from_str(&text).expect("JSON")
}

#[test]
fn the_schema_admits_the_committed_format2_document() {
    assert!(validator().is_valid(&written()));
}

#[test]
fn the_schema_refuses_a_format1_relabel_the_reader_refuses() {
    let mut relabelled = written();
    relabelled["format"] = json!("ess-history/1");
    assert!(
        !validator().is_valid(&relabelled),
        "the schema admits an `ess-history/1` document carrying `decision_time`, which the reader \
         refuses as history.malformed: a writer validating against the schema passes and is refused"
    );
}
