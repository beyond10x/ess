//! Adversary pass 1 against `story:field-names-underscore-and-newtype-map-keys`.
//!
//! `FactPath::new` now admits a leading underscore (beyond10x/ess#141). `FactPath::PATTERN` is "the
//! pattern published in generated JSON Schema" for the same type, and it was not changed, so the
//! published schema of a fact path refuses `_url` while the parser reads it.

use ess_primitives::facts::FactPath;

#[test]
fn the_fact_path_schema_publishes_the_rule_the_parser_enforces() {
    FactPath::new("_url.host").expect("the parser admits a leading underscore");
    let schema = serde_json::to_value(schemars::schema_for!(FactPath)).expect("serialises");
    let published = schema["pattern"]
        .as_str()
        .expect("a fact path publishes a pattern");
    // The literal the unit's own runtime guard names "the Rust `FactPath` rule".
    assert_eq!(
        published, "^_*[A-Za-z][A-Za-z0-9_-]*(\\.[A-Za-z0-9_-]+)*$",
        "the published fact-path pattern still refuses `_url.host`, which `FactPath::new` reads"
    );
}
