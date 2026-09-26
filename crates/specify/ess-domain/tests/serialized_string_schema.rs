//! The committed document schema describes `ExternalRef` and `Period` as the strings they serialize to.
//!
//! `story:schema-describes-serialized-strings`. Both types are structs in Rust and strings on the
//! wire — `jira:DEV-630`, `PT2S` — and the derived schema described the Rust shape: an object with
//! `provider` and `reference`, and a bare integer. An editor holding that schema refuses every
//! document ESS reads and offers the author a shape ESS refuses.
//!
//! So each case reads `schemas/generated/ess.schema.json` — the committed projection, the thing an
//! adopter's editor actually loads — and holds the two directions together over one corpus: a
//! spelling the schema admits is one the parser reads, and the reverse. `projection-check` keeps
//! the committed file equal to what the types generate, so these cases are about the types too.

use ess_domain::binding::periodic::Period;
use ess_domain::refs::ExternalRef;
use serde_json::{json, Value};

fn document_schema() -> Value {
    let text = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../schemas/generated/ess.schema.json"),
    )
    .expect("the generated schema is committed");
    serde_json::from_str(&text).expect("the generated schema is JSON")
}

/// A validator for one definition, resolved inside the committed document so its own `$ref`s work.
fn validator(definition: &str) -> jsonschema::Validator {
    let schema = document_schema();
    assert!(
        !schema["definitions"][definition].is_null(),
        "the document schema carries `{definition}`"
    );
    jsonschema::validator_for(&json!({
        "$schema": schema["$schema"],
        "definitions": schema["definitions"],
        "$ref": format!("#/definitions/{definition}"),
    }))
    .expect("a usable schema")
}

fn assert_parity(validator: &jsonschema::Validator, candidate: &str, parses: bool, name: &str) {
    let admitted = validator.is_valid(&json!(candidate));
    assert_eq!(
        admitted,
        parses,
        "{name} {candidate:?}: the schema {} it and the parser {} it",
        if admitted { "admits" } else { "refuses" },
        if parses { "reads" } else { "refuses" },
    );
}

#[test]
fn an_external_reference_validates_as_the_provider_key_string_it_serializes_to() {
    let validator = validator("ExternalRef");
    for written in [
        "jira:DEV-630",
        "zendesk:204519",
        "wiki:space:Runbooks/ACD",
        "a-1:x",
    ] {
        let serialized =
            serde_json::to_value(ExternalRef::parse(written).expect("valid reference"))
                .expect("serializes");
        assert_eq!(serialized, json!(written), "the wire form is the string");
        assert!(
            validator.is_valid(&serialized),
            "the schema refuses the serialized reference {serialized}"
        );
    }
    assert!(
        !validator.is_valid(&json!({"provider": "jira", "reference": "DEV-630"})),
        "the Rust field shape is not a document ESS reads, so the schema must refuse it"
    );
}

#[test]
fn the_external_reference_schema_admits_exactly_what_the_parser_reads() {
    let validator = validator("ExternalRef");
    for candidate in [
        "jira:DEV-630",
        "wiki:space:Runbooks/ACD",
        "jira:a",
        "jira: x",
        "jira:x ",
        "jira:\tx",
        "jira:x\n",
        "jira:x\u{0085}",
        "jira:\u{3000}x",
        "jira:x\u{00a0}",
        "jira:x y",
        "jira:x/y",
        "jira:é",
        "jira:x:",
        "jira::",
        "jira:",
        ":DEV-630",
        "DEV-630",
        "",
        "Jira:DEV-630",
        "ji ra:DEV-630",
        "jira_x:1",
        "https://tracker.example/DEV-630",
        "jira://x",
        "jira:/x",
        "jira:x//y",
    ] {
        assert_parity(
            &validator,
            candidate,
            ExternalRef::parse(candidate).is_ok(),
            "ExternalRef",
        );
    }
}

#[test]
fn a_period_validates_as_the_iso_duration_string_it_serializes_to() {
    let validator = validator("Period");
    for written in ["PT1S", "PT2S", "PT60S", "PT86400S"] {
        let serialized = serde_json::to_value(Period::parse(written).expect("valid period"))
            .expect("serializes");
        assert_eq!(serialized, json!(written), "the wire form is the string");
        assert!(
            validator.is_valid(&serialized),
            "the schema refuses the serialized period {serialized}"
        );
    }
    assert!(
        !validator.is_valid(&json!(60)),
        "the Rust representation, whole seconds as an integer, is not a document ESS reads"
    );
    assert!(
        !validator.is_valid(&json!({"seconds": 60})),
        "an object is not a period either"
    );
}

#[test]
fn the_period_schema_admits_exactly_what_the_parser_reads_across_every_range_split() {
    let validator = validator("Period");
    // Every boundary of the digit-by-digit bound on `u32::MAX`, both sides of it.
    for seconds in [
        1_u64,
        9,
        10,
        99,
        100,
        999_999_999,
        1_000_000_000,
        3_999_999_999,
        4_000_000_000,
        4_199_999_999,
        4_200_000_000,
        4_289_999_999,
        4_290_000_000,
        4_293_999_999,
        4_294_000_000,
        4_294_899_999,
        4_294_900_000,
        4_294_959_999,
        4_294_960_000,
        4_294_966_999,
        4_294_967_000,
        4_294_967_199,
        4_294_967_200,
        4_294_967_289,
        4_294_967_290,
        4_294_967_295,
        4_294_967_296,
        4_294_967_299,
        4_294_967_300,
        4_294_968_000,
        4_295_000_000,
        4_300_000_000,
        5_000_000_000,
        9_999_999_999,
        10_000_000_000,
    ] {
        let candidate = format!("PT{seconds}S");
        assert_parity(
            &validator,
            &candidate,
            Period::parse(&candidate).is_ok(),
            "Period",
        );
    }
    for candidate in [
        "",
        "PTS",
        "PT0S",
        "PT00S",
        "PT01S",
        "PT+1S",
        "PT-1S",
        "PT1.0S",
        "P1S",
        "PT1",
        "PT1M",
        "PT1H",
        "pt1s",
        "XPT1S",
        "PT1SX",
        " PT1S",
        "PT1S ",
        "PT1S\n",
        "PT1S\r\n",
        "\nPT1S",
        "PT1S\u{0000}",
        "PT1S\u{0085}",
        "PT\u{ff11}S",
        "PT1 S",
    ] {
        assert_parity(
            &validator,
            candidate,
            Period::parse(candidate).is_ok(),
            "Period",
        );
    }
}
