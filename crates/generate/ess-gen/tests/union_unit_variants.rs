//! A unit variant in the published JSON Schema (ess/22, beyond10x/ess#418,
//! `docs/design/union-unit-variants.md`).
//!
//! A payload variant is `{<tag>: <label>, <content>: <payload>}`; a unit variant is the same tag
//! and nothing else, `{<tag>: <label>}`. The branch pins the tag with a `const` like every other
//! branch, requires only the tag and closes the object, so a unit variant written with a content
//! member, and a payload variant written without one, are both refused by any conforming
//! validator.

use ess_compiler::{resolve::compile, source::SourceMap, EssIr};
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source,
};
use ess_gen::{artifact::run, schema::JsonSchema};
use serde_json::{json, Value};

const MODEL: &str =
    include_str!("../../../specify/ess-compiler/tests/fixtures/union-unit-variants.yaml");

fn compiled(text: &str) -> EssIr {
    let spec =
        Specification::assemble([(Source::new("work.yaml"), RawSpecFile::parse(text).unwrap())])
            .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap()
}

fn schema(text: &str, path: &str) -> Value {
    let artifacts = run(&JsonSchema, &compiled(text)).expect("generates");
    serde_json::from_str(
        &artifacts
            .get(path)
            .unwrap_or_else(|| panic!("no {path}: {:?}", artifacts.keys().collect::<Vec<_>>()))
            .contents,
    )
    .unwrap()
}

const STATUS: &str = "schema/types/demo.work.Status.schema.json";

#[test]
fn a_unit_variant_branch_is_the_tag_alone_and_closed() {
    let schema = schema(MODEL, STATUS);
    let status = &schema["$defs"]["demo.work.Status"];
    let branches = status["oneOf"]
        .as_array()
        .unwrap_or_else(|| panic!("{schema:#}"));
    assert_eq!(branches.len(), 2, "{schema:#}");
    let open = branches
        .iter()
        .find(|branch| branch["properties"]["kind"]["const"] == "Open")
        .unwrap_or_else(|| panic!("an `Open` branch: {schema:#}"));
    assert_eq!(open["type"], "object", "{open:#}");
    assert_eq!(open["required"], json!(["kind"]), "{open:#}");
    assert_eq!(open["additionalProperties"], json!(false), "{open:#}");
    assert_eq!(
        open["properties"]
            .as_object()
            .unwrap()
            .keys()
            .collect::<Vec<_>>(),
        ["kind"],
        "a unit variant has no content member: {open:#}"
    );
    let complete = branches
        .iter()
        .find(|branch| branch["properties"]["kind"]["const"] == "Complete")
        .unwrap_or_else(|| panic!("a `Complete` branch: {schema:#}"));
    assert_eq!(
        complete["required"],
        json!(["kind", "value"]),
        "{complete:#}"
    );
}

#[test]
fn a_validator_admits_both_variants_and_refuses_the_two_malformed_shapes() {
    let schema = schema(MODEL, STATUS);
    let validator = jsonschema::validator_for(&schema).expect("a valid schema");
    for valid in [
        json!({"kind": "Open"}),
        json!({"kind": "Complete", "value": {"outcome": "shipped"}}),
    ] {
        assert!(validator.is_valid(&valid), "{valid} against {schema:#}");
    }
    for invalid in [
        json!({"kind": "Open", "value": {"outcome": "shipped"}}),
        json!({"kind": "Open", "value": null}),
        json!({"kind": "Complete"}),
        json!({"kind": "Closed"}),
        json!({}),
    ] {
        assert!(
            !validator.is_valid(&invalid),
            "{invalid} against {schema:#}"
        );
    }
}

#[test]
fn the_event_carrying_the_union_admits_a_unit_variant() {
    let schema = schema(MODEL, "schema/events/demo.work.StatusReported.schema.json");
    let validator = jsonschema::validator_for(&schema).expect("a valid schema");
    assert!(
        validator.is_valid(&json!({"status": {"kind": "Open"}})),
        "{schema:#}"
    );
    assert!(
        !validator.is_valid(&json!({"status": {"kind": "Open", "value": {"outcome": "x"}}})),
        "{schema:#}"
    );
}

#[test]
fn a_union_named_value_moves_its_content_key_and_a_unit_variant_still_carries_none() {
    let renamed = MODEL.replace("    tag: kind\n", "    tag: value\n");
    assert_ne!(renamed, MODEL);
    let schema = schema(&renamed, STATUS);
    let validator = jsonschema::validator_for(&schema).expect("a valid schema");
    assert!(validator.is_valid(&json!({"value": "Open"})), "{schema:#}");
    assert!(
        validator.is_valid(&json!({"value": "Complete", "content": {"outcome": "x"}})),
        "{schema:#}"
    );
    assert!(
        !validator.is_valid(&json!({"value": "Open", "content": {"outcome": "x"}})),
        "{schema:#}"
    );
}
