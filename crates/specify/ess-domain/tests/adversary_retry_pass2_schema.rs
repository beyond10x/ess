//! Adversary pass 2 (story:bounded-retry-bindings): the published schema against the reader.
//!
//! The reader refuses `attempts` below 2 and a `final:` name written twice; the generated schema
//! says `minimum: 0` and admits duplicates, so an editor validating against the schema accepts
//! documents the reader refuses.

use ess_domain::RawSpecFile;

fn bound_schema() -> serde_json::Value {
    let text = include_str!("../../../../schemas/generated/ess.schema.json");
    let schema: serde_json::Value = serde_json::from_str(text).expect("the schema is JSON");
    schema["definitions"]["RetryBound"].clone()
}

#[test]
fn adversary_retry_pass2_the_schema_states_the_attempts_floor_the_reader_enforces() {
    let bound = bound_schema();
    let minimum = bound["properties"]["attempts"]["minimum"]
        .as_f64()
        .expect("attempts has a minimum");
    assert!(
        minimum >= 2.0,
        "the reader refuses `attempts: 0` and `attempts: 1`, and the schema admits them: {bound:#}"
    );
}

#[test]
fn adversary_retry_pass2_the_reader_does_refuse_one_attempt() {
    // Guard for the case above: the refusal it compares against is real.
    let text = include_str!("../../ess-compiler/tests/fixtures/bounded-retry.yaml")
        .replace("attempts: 3", "attempts: 1");
    let raw = RawSpecFile::parse(&text).expect("parses");
    assert!(ess_domain::Specification::assemble([(
        ess_domain::system::Source::new("bounded-retry.yaml"),
        raw
    )])
    .is_err());
}
