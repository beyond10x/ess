//! Adversary pass 1 on the state-scoped unit (beyond10x/ess#201, #204).
//!
//! The format gate read from a JSON-form source, and the published JSON Schema for
//! `when_subject_state` read as the contract it claims to be.
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::error::{ValidationCode, ValidationErrors};

const SHIP: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/state-scoped-refusals.yaml");
const REPORT: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/state-in-subject-predicate.yaml");

/// `text` rewritten as JSON: the same document in the form a JSON-source author writes it.
fn as_json(text: &str) -> String {
    let value: serde_yaml::Value = serde_yaml::from_str(text).expect("fixture is YAML");
    serde_json::to_string_pretty(&value).expect("fixture is JSON-representable")
}

fn assemble(text: &str) -> Result<Specification, ValidationErrors> {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    Specification::assemble([(Source::new("orders.json"), raw)])
}

fn at(text: &str, format: &str) -> String {
    let out = text.replace("format: ess/18\n", &format!("format: {format}\n"));
    assert_ne!(out, text);
    out
}

fn format_refused(text: &str) -> bool {
    assemble(text).err().is_some_and(|errors| {
        errors.as_slice().iter().any(|error| {
            error.code == ValidationCode::UnsupportedFormatVersion
                && error.to_string().contains("ess/18")
        })
    })
}

#[test]
fn json_sources_are_gated_like_yaml_ones() {
    assemble(&as_json(SHIP)).unwrap_or_else(|errors| panic!("{errors}"));
    assemble(&as_json(REPORT)).unwrap_or_else(|errors| panic!("{errors}"));
    for (what, text) in [("listed", SHIP), ("state in when_subject", REPORT)] {
        assert!(
            format_refused(&as_json(&at(text, "ess/17"))),
            "{what}: a JSON source at ess/17 is not refused for its format"
        );
    }
}

/// The schema says the list is nonempty, and the model refuses an empty or repeated one: the
/// schema an editor validates against must refuse them too.
#[test]
fn the_published_schema_refuses_what_the_model_refuses() {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../schemas/generated/ess.schema.json"
    );
    let schema: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(path).expect("schema is readable"))
            .expect("schema is JSON");
    let held = &schema["definitions"]["HeldStates"];
    let list = held["anyOf"]
        .as_array()
        .expect("HeldStates is anyOf")
        .iter()
        .find(|branch| branch["type"] == "array")
        .expect("a list branch");
    assert_eq!(
        list["minItems"],
        serde_json::json!(1),
        "`when_subject_state: []` passes the schema and is refused by the model: {held}"
    );
    assert_eq!(
        list["uniqueItems"],
        serde_json::json!(true),
        "`when_subject_state: [A, A]` passes the schema and is refused by the model: {held}"
    );
}
