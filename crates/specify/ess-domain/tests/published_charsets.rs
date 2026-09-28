//! A raw field constrained by a charset publishes that charset.
//!
//! `story:a-field-constrained-by-a-charset-publishes-that-charset`. `schemas/generated/ess.schema.json`
//! is what a schema-aware editor validates an authored document against. A `Raw*` field that is a
//! bare `type: string` in that schema, while its constructor refuses most strings, hands an author a
//! green editor and a red `ess validate`.
//!
//! Each case holds one field to its constructor in both directions the story names: the pattern the
//! type derives is the constructor's `PATTERN`, and the committed schema refuses the spellings the
//! constructor refuses — and admits the ones it reads, so a pattern stricter than the parser would
//! be caught too.
//!
//! # The class
//!
//! Every text field of a `Raw*` type in `ess-domain` whose value a constructor with a `PATTERN`
//! parses, as of this story:
//!
//! | field | parsed by | published |
//! |---|---|---|
//! | `RawComponentSpec.name` | `ComponentName::new` | here |
//! | `RawCommandLineSurface.binary` | `CliName::new` | here |
//! | `RawCommandGroup.name` | `CliName::new` | here |
//! | `RawBindingSpec.name` | `BindingName::new` | here |
//! | `RawComponentSetting.name` | `CliName::new` | `component_settings.rs` |
//! | `RawField.name`, `RawInputField.name`, `RawViewField.name` | `Field::PATTERN` | already |
//! | `RawTopology.workloads` keys | `ComponentName::new` | not yet: a map key needs `propertyNames` |
//!
//! Outcome, outcome-group and state names are not in it: their `Raw*` fields are typed
//! `OutcomeName`, `OutcomeGroupName` and `StateName`, which publish their own pattern. The other
//! text fields of `Raw*` types are free text (`summary`, `external`, `alphabet`, `prefix`) or name
//! a declaration that is resolved by lookup rather than parsed by a charset (`instance`,
//! `group_by`, `RawSubjectField`, union `tag`, `fixture_inputs` keys). `RawRelated` is parsed by
//! the field-name charset, but its shape is also a valid `RawNestedSources`, so a pattern on it
//! would refuse nothing.

use ess_domain::binding::{BindingName, RawBindingSpec};
use ess_domain::component::{
    CliName, ComponentName, RawCommandGroup, RawCommandLineSurface, RawComponentSpec,
};
use serde_json::{json, Value};

/// Spellings every lower-case-hyphenated constructor reads.
const READ: &[&str] = &["invoice-service", "s3", "serve-hosted", "a", "x2-y3"];

/// Spellings it refuses, one per rule the constructor applies.
const REFUSED: &[&str] = &[
    "",
    "Invoice",
    "invoice_service",
    "invoice-",
    "-invoice",
    "in--voice",
    "1invoice",
    "invoice service",
    "invoice.service",
    "ïnvoice",
];

fn document_schema() -> Value {
    let text = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../schemas/generated/ess.schema.json"),
    )
    .expect("the generated schema is committed");
    serde_json::from_str(&text).expect("the generated schema is JSON")
}

/// A validator for one property of one definition in the committed document schema.
fn property_validator(definition: &str, property: &str) -> jsonschema::Validator {
    let schema = document_schema();
    let property_schema = &schema["definitions"][definition]["properties"][property];
    assert!(
        !property_schema.is_null(),
        "the document schema carries `{definition}.{property}`"
    );
    jsonschema::validator_for(&json!({
        "$schema": schema["$schema"],
        "definitions": schema["definitions"],
        "allOf": [property_schema],
    }))
    .expect("a usable schema")
}

/// The pattern the type derives for `property`, read from its own generated schema.
fn derived_pattern(root: &schemars::schema::RootSchema, property: &str) -> Value {
    serde_json::to_value(root).expect("serialises")["properties"][property]["pattern"].clone()
}

/// The committed schema and `parses` agree on every spelling in [`READ`] and [`REFUSED`].
fn assert_parity(definition: &str, property: &str, parses: impl Fn(&str) -> bool) {
    let validator = property_validator(definition, property);
    for &written in READ.iter().chain(REFUSED) {
        let reads = parses(written);
        assert_eq!(
            reads,
            READ.contains(&written),
            "the corpus is wrong about {written:?}"
        );
        let admitted = validator.is_valid(&json!(written));
        assert_eq!(
            admitted,
            reads,
            "{definition}.{property} {written:?}: the schema {} it and the parser {} it",
            if admitted { "admits" } else { "refuses" },
            if reads { "reads" } else { "refuses" },
        );
    }
}

#[test]
fn a_command_line_binary_publishes_the_cli_name_charset() {
    assert_eq!(
        derived_pattern(&schemars::schema_for!(RawCommandLineSurface), "binary"),
        json!(CliName::PATTERN),
        "a schema that accepts what the parser refuses is worse than no schema"
    );
    assert_parity("RawCommandLineSurface", "binary", |written| {
        CliName::new(written).is_ok()
    });
}

#[test]
fn a_command_group_name_publishes_the_cli_name_charset() {
    assert_eq!(
        derived_pattern(&schemars::schema_for!(RawCommandGroup), "name"),
        json!(CliName::PATTERN),
        "a schema that accepts what the parser refuses is worse than no schema"
    );
    assert_parity("RawCommandGroup", "name", |written| {
        CliName::new(written).is_ok()
    });
}

#[test]
fn a_component_name_publishes_the_component_name_charset() {
    assert_eq!(
        derived_pattern(&schemars::schema_for!(RawComponentSpec), "name"),
        json!(ComponentName::PATTERN),
        "a schema that accepts what the parser refuses is worse than no schema"
    );
    assert_parity("RawComponentSpec", "name", |written| {
        ComponentName::new(written).is_ok()
    });
}

#[test]
fn a_binding_name_publishes_the_binding_name_charset() {
    assert_eq!(
        derived_pattern(&schemars::schema_for!(RawBindingSpec), "name"),
        json!(BindingName::PATTERN),
        "a schema that accepts what the parser refuses is worse than no schema"
    );
    assert_parity("RawBindingSpec", "name", |written| {
        BindingName::new(written).is_ok()
    });
}
#[test]
fn a_workload_key_publishes_the_component_name_charset() {
    use ess_domain::topology::RawTopology;

    let derived = serde_json::to_value(schemars::schema_for!(RawTopology)).expect("serialises");
    assert_eq!(
        derived["properties"]["workloads"]["propertyNames"]["$ref"],
        json!("#/definitions/ComponentName"),
        "a schema that accepts what the parser refuses is worse than no schema"
    );
    let validator = property_validator("RawTopology", "workloads");
    for &written in READ.iter().chain(REFUSED) {
        let reads = ComponentName::new(written).is_ok();
        let admitted = validator.is_valid(&json!({ written: {} }));
        assert_eq!(
            admitted, reads,
            "RawTopology.workloads key {written:?}: the schema and the parser disagree"
        );
    }
}
