//! A `distinct:` invariant is asserted by the schema where JSON Schema says it exactly
//! (beyond10x/ess#686).
//!
//! `distinct: {in: F, as: x}` over a list of scalars publishes `uniqueItems: true` on `F`. With
//! `by: x.m1…mk` reaching an enum or a `Boolean` through required members, `F` carries one
//! `contains` per variant, at most once each: `{"contains": C, "minContains": 0, "maxContains": 1}`,
//! where `C` requires an object holding the key path and ends in the variant's wire spelling. An
//! element without the key is not counted, as the model reads an absent key as unknown. An open key
//! (text, a number, an identifier, an instant) or an `Optional` member on the path stays an
//! annotation in `x-ess-invariants`.

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
domain: demo.split
types:
  - {name: demo.split.SetId, kind: newtype, of: Uuid}
  - name: demo.split.Role
    kind: enum
    variants:
      - {name: Train, wire: train}
      - {name: HeldOut, wire: held_out}
  - name: demo.split.Purpose
    kind: struct
    fields:
      - {name: role, type: demo.split.Role}
  - name: demo.split.Split
    kind: struct
    fields:
      - {name: label, type: String}
      - {name: role, type: demo.split.Role}
      - {name: purpose, type: demo.split.Purpose}
      - {name: primary, type: Boolean}
      - {name: hint, type: Optional<demo.split.Role>}
  - name: demo.split.Tags
    kind: newtype
    of: List<String>
    invariants:
      - {distinct: {in: value, as: tag}}
  - name: demo.split.Plan
    kind: struct
    fields:
      - {name: tags, type: List<String>}
      - {name: codes, type: Optional<List<Integer>>}
      - {name: splits, type: List<demo.split.Split>}
      - {name: purposes, type: List<demo.split.Split>}
      - {name: flagged, type: List<demo.split.Split>}
      - {name: labelled, type: List<demo.split.Split>}
      - {name: hinted, type: List<demo.split.Split>}
      - {name: kept, type: demo.split.Tags}
    invariants:
      - {distinct: {in: tags, as: tag}}
      - {distinct: {in: codes, as: code}}
      - {distinct: {in: splits, as: split, by: split.role}}
      - {distinct: {in: purposes, as: split, by: split.purpose.role}}
      - {distinct: {in: flagged, as: split, by: split.primary}}
      - {distinct: {in: labelled, as: split, by: split.label}}
      - {distinct: {in: hinted, as: split, by: split.hint}}
entities:
  - name: demo.split.Set
    identity: {name: set_id, type: demo.split.SetId}
    fields:
      - {name: splits, type: List<demo.split.Split>}
    invariants:
      - {distinct: {in: splits, as: split, by: split.role}}
    lifecycle: {initial: Open, states: [Open], terminal: [Open], transitions: []}
";

fn compiled(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("well formed: {error}"));
    let spec = Specification::assemble([(Source::new("split.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("validates: {errors}"));
    let mut sources = SourceMap::new();
    sources.insert("split.yaml", text);
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

fn plan() -> Value {
    document("schema/types/demo.split.Plan.schema.json")
}

/// One `contains` entry per variant: at most one element holds `key` at the end of `path`.
fn at_most_once(path: &[&str], variants: &[Value]) -> Value {
    Value::Array(
        variants
            .iter()
            .map(|variant| {
                let contains = path.iter().rev().fold(json!({"const": variant}), |inner, member| {
                    json!({"type": "object", "properties": {*member: inner}, "required": [member]})
                });
                json!({"contains": contains, "minContains": 0, "maxContains": 1})
            })
            .collect(),
    )
}

#[test]
fn a_distinct_list_of_scalars_is_unique_items() {
    let schema = plan();
    let properties = &schema["$defs"]["demo.split.Plan"]["properties"];
    assert_eq!(properties["tags"]["uniqueItems"], json!(true), "{schema:#}");
    assert_eq!(
        properties["codes"]["uniqueItems"],
        json!(true),
        "{schema:#}"
    );
    assert_eq!(
        schema["$defs"]["demo.split.Tags"]["uniqueItems"],
        json!(true),
        "a list newtype's `value`: {schema:#}"
    );
    assert_eq!(properties["kept"].get("uniqueItems"), None, "{schema:#}");
}

#[test]
fn a_distinct_by_an_enum_key_is_one_bounded_contains_per_variant() {
    let schema = plan();
    let properties = &schema["$defs"]["demo.split.Plan"]["properties"];
    let variants = [json!("train"), json!("held_out")];
    assert_eq!(
        properties["splits"]["allOf"],
        at_most_once(&["role"], &variants),
        "{schema:#}"
    );
    assert_eq!(
        properties["purposes"]["allOf"],
        at_most_once(&["purpose", "role"], &variants),
        "a nested key path: {schema:#}"
    );
    assert_eq!(
        properties["splits"]["items"],
        json!({"$ref": "#/$defs/demo.split.Split"}),
        "{schema:#}"
    );

    let entity = document("schema/entities/demo.split.Set.schema.json");
    assert_eq!(
        entity["properties"]["splits"]["allOf"],
        at_most_once(&["role"], &variants),
        "{entity:#}"
    );
}

#[test]
fn a_distinct_by_a_boolean_key_is_bounded_like_an_enum() {
    let schema = plan();
    assert_eq!(
        schema["$defs"]["demo.split.Plan"]["properties"]["flagged"]["allOf"],
        at_most_once(&["primary"], &[json!(true), json!(false)]),
        "{schema:#}"
    );
}

#[test]
fn an_open_key_or_an_optional_path_stays_an_annotation() {
    let schema = plan();
    let plan = &schema["$defs"]["demo.split.Plan"];
    for name in ["labelled", "hinted"] {
        let property = &plan["properties"][name];
        assert_eq!(property.get("allOf"), None, "`{name}`: {schema:#}");
        assert_eq!(property.get("uniqueItems"), None, "`{name}`: {schema:#}");
    }
    assert_eq!(
        plan["x-ess-invariants"].as_array().map(Vec::len),
        Some(7),
        "{schema:#}"
    );
}

#[test]
fn a_validator_refuses_two_elements_of_one_variant_and_admits_keyless_ones() {
    let schema = plan();
    let validator = jsonschema::validator_for(&schema).expect("a valid schema");
    let split = |role: &str, primary: bool| json!({"label": role, "role": role, "purpose": {"role": role}, "primary": primary});
    let instance = |splits: Value| {
        json!({
            "tags": ["a", "b"], "splits": splits, "purposes": [], "flagged": [], "labelled": [],
            "hinted": [], "kept": ["a"]
        })
    };
    assert!(validator.is_valid(&instance(json!([]))), "an empty list");
    assert!(
        validator.is_valid(&instance(json!([
            split("train", true),
            split("held_out", true)
        ]))),
        "one of each variant"
    );
    assert!(
        !validator.is_valid(&instance(json!([
            split("train", true),
            split("held_out", true),
            split("train", false)
        ]))),
        "two `train` splits"
    );
    let mut repeated = instance(json!([]));
    repeated["tags"] = json!(["a", "a"]);
    assert!(!validator.is_valid(&repeated), "a repeated tag");

    // The bounded `contains` alone counts only elements that hold the key: an element without it,
    // or a scalar, is neither of the variants.
    let bounded = json!({
        "allOf": schema["$defs"]["demo.split.Plan"]["properties"]["splits"]["allOf"].clone()
    });
    let contains = jsonschema::validator_for(&bounded).expect("a valid schema");
    assert!(contains.is_valid(&json!([{}, {}])), "keyless elements");
    assert!(contains.is_valid(&json!(["train", "train"])), "scalars");
    assert!(
        !contains.is_valid(&json!([{"role": "train"}, {"role": "train"}])),
        "two keyed `train`s"
    );
}
