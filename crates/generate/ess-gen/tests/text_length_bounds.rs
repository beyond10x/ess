//! A text-length invariant becomes `minLength` and `maxLength` (beyond10x/ess#685).
//!
//! `<s>.count <op> <integer>`, where `s` is a newtype's `value` of `String` or a top-level `String`
//! field of a struct or an entity, publishes `minLength` (`>=`, `>`), `maxLength` (`<=`, `<`) or
//! both (`==`). `.count` is the number of Unicode scalar values, which is what JSON Schema's length
//! keywords count. `utf8_bytes`, `!=` and a field typed by a `String` newtype stay annotations in
//! `x-ess-invariants`.

use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_compiler::EssIr;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_gen::artifact::run;
use ess_gen::schema::JsonSchema;
use serde_json::{json, Value};

const MODEL: &str = "format: ess/22
system: demo
version: v1
domain: demo.text
types:
  - {name: demo.text.RecordId, kind: newtype, of: Uuid}
  - name: demo.text.Digest
    kind: newtype
    of: String
    invariants:
      - value.count == 64
  - name: demo.text.Mapped
    kind: newtype
    of: String
    invariants:
      - value.count: {gte: 1, lte: 10}
  - name: demo.text.Paired
    kind: newtype
    of: String
    invariants:
      - value.count >= 1
      - value.count <= 10
  - name: demo.text.Strict
    kind: newtype
    of: String
    invariants:
      - value.count > 0
      - value.count < 11
  - name: demo.text.Note
    kind: struct
    fields:
      - {name: name, type: String}
      - {name: remark, type: Optional<String>}
      - {name: code, type: Optional<String>, presence: null_when_absent}
      - {name: label, type: String}
      - {name: tag, type: String}
      - {name: digest, type: demo.text.Digest}
    invariants:
      - name.count >= 1
      - remark.count <= 140
      - code.count == 2
      - label.utf8_bytes <= 8
      - tag.count != 3
      - digest.count == 64
entities:
  - name: demo.text.Record
    identity: {name: record_id, type: demo.text.RecordId}
    fields:
      - {name: title, type: String}
    invariants:
      - title.count <= 20
    lifecycle: {initial: Open, states: [Open], terminal: [Open], transitions: []}
";

fn compiled(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("well formed: {error}"));
    let spec = Specification::assemble([(Source::new("text.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("validates: {errors}"));
    let mut sources = SourceMap::new();
    sources.insert("text.yaml", text);
    compile(&spec, &sources).unwrap_or_else(|errors| panic!("compiles: {errors}"))
}

fn document(path: &str) -> Value {
    let artifacts = run(&JsonSchema, &compiled(MODEL)).expect("generates");
    let artifact = artifacts.get(path).unwrap_or_else(|| {
        panic!(
            "no `{path}` among {:?}",
            artifacts.keys().collect::<Vec<_>>()
        )
    });
    serde_json::from_str(&artifact.contents).expect("JSON")
}

fn note() -> Value {
    document("schema/types/demo.text.Note.schema.json")
}

/// The two length keywords a node carries, absent ones as `null`.
fn lengths(node: &Value) -> Value {
    json!({"minLength": node.get("minLength"), "maxLength": node.get("maxLength")})
}

#[test]
fn a_count_equality_on_a_string_newtype_becomes_min_and_max_length() {
    let schema = note();
    let digest = &schema["$defs"]["demo.text.Digest"];
    assert_eq!(
        lengths(digest),
        json!({"minLength": 64, "maxLength": 64}),
        "{schema:#}"
    );
    assert_eq!(digest["x-ess-invariants"], json!(["value.count == 64"]));
}

#[test]
fn the_mapping_form_a_pair_and_a_mirrored_literal_project_alike() {
    for name in ["demo.text.Mapped", "demo.text.Paired", "demo.text.Strict"] {
        let schema = document(&format!("schema/types/{name}.schema.json"));
        assert_eq!(
            lengths(&schema["$defs"][name]),
            json!({"minLength": 1, "maxLength": 10}),
            "`{name}`: {schema:#}"
        );
    }
    // The literal on the left has no authored spelling; `src/types.rs`
    // `a_literal_on_the_left_lowers_as_its_mirror` reads `1 <= value.count` on a constructed
    // predicate, through the same reader.
}

#[test]
fn a_count_bound_on_a_struct_or_entity_string_field_lands_on_its_property() {
    let schema = note();
    let property = |name: &str| schema["$defs"]["demo.text.Note"]["properties"][name].clone();
    assert_eq!(
        lengths(&property("name")),
        json!({"minLength": 1, "maxLength": null}),
        "{schema:#}"
    );
    assert_eq!(
        lengths(&property("remark")),
        json!({"minLength": null, "maxLength": 140}),
        "{schema:#}"
    );
    // A nullable field: the keywords sit beside its `anyOf`, and ignore the `null`.
    assert_eq!(
        lengths(&property("code")),
        json!({"minLength": 2, "maxLength": 2}),
        "{schema:#}"
    );

    let entity = document("schema/entities/demo.text.Record.schema.json");
    assert_eq!(
        lengths(&entity["properties"]["title"]),
        json!({"minLength": null, "maxLength": 20}),
        "{entity:#}"
    );
}

#[test]
fn utf8_bytes_and_a_newtype_typed_field_stay_annotations() {
    let schema = note();
    let unbounded = json!({"minLength": null, "maxLength": null});
    for name in ["label", "tag", "digest"] {
        let node = &schema["$defs"]["demo.text.Note"]["properties"][name];
        assert_eq!(lengths(node), unbounded, "`{name}`: {schema:#}");
    }
    assert_eq!(
        schema["$defs"]["demo.text.Note"]["x-ess-invariants"]
            .as_array()
            .map(Vec::len),
        Some(6),
        "{schema:#}"
    );
}

#[test]
fn a_validator_counts_scalar_values_as_the_model_does() {
    let schema = note();
    let validator = jsonschema::validator_for(&schema).expect("a valid schema");
    let note = |digest: String| json!({"name": "n", "code": null, "label": "l", "tag": "t", "digest": digest});
    let smiles = |count: usize| "\u{1F600}".repeat(count);
    assert!(validator.is_valid(&note(smiles(64))), "64 scalar values");
    assert!(!validator.is_valid(&note(smiles(63))), "63 scalar values");
    assert!(!validator.is_valid(&note(smiles(65))), "65 scalar values");
    let mut unnamed = note(smiles(64));
    unnamed["name"] = json!("");
    assert!(!validator.is_valid(&unnamed), "`name.count >= 1`");
}
