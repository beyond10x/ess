//! Integer bounds a struct's invariants state become JSON Schema keywords (beyond10x/ess#394).
//!
//! `amount >= -2147483648` and `amount <= 2147483647` on an `Integer` field publish `minimum` and
//! `maximum`, and `version == 2` publishes `const`, so a plain JSON Schema validator refuses what
//! the model refuses. Every invariant is still published verbatim under `x-ess-invariants`, and an
//! invariant no keyword says exactly (`!=`, a `Decimal`) is not lowered.

use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_compiler::EssIr;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_gen::artifact::run;
use ess_gen::schema::JsonSchema;
use serde_json::{json, Value};

const MODEL: &str = "format: ess/20
system: demo
version: v1
domain: demo.metering
types:
  - name: demo.metering.Reading
    kind: struct
    fields:
      - name: version
        type: Integer
      - name: amount
        type: Integer
      - name: retries
        type: Optional<Integer>
      - name: level
        type: Optional<Integer>
        presence: null_when_absent
      - name: weight
        type: Decimal
      - name: offset
        type: Integer
    invariants:
      - version == 2
      - amount >= -2147483648
      - amount <= 2147483647
      - retries < 10
      - retries >= 0
      - retries >= 1
      - level == 3
      - weight >= 0
      - offset != 5
";

fn compiled(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("well formed: {error}"));
    let spec = Specification::assemble([(Source::new("metering.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("validates: {errors}"));
    let mut sources = SourceMap::new();
    sources.insert("metering.yaml", text);
    compile(&spec, &sources).unwrap_or_else(|errors| panic!("compiles: {errors}"))
}

fn reading() -> Value {
    let artifacts = run(&JsonSchema, &compiled(MODEL)).expect("generates");
    let path = "schema/types/demo.metering.Reading.schema.json";
    let artifact = artifacts.get(path).unwrap_or_else(|| {
        panic!(
            "no `{path}` among {:?}",
            artifacts.keys().collect::<Vec<_>>()
        )
    });
    serde_json::from_str(&artifact.contents).expect("JSON")
}

fn property(schema: &Value, name: &str) -> Value {
    schema["$defs"]["demo.metering.Reading"]["properties"][name].clone()
}

#[test]
fn bounds_and_constants_become_keywords() {
    let schema = reading();
    assert_eq!(
        property(&schema, "version")["const"],
        json!(2),
        "{schema:#}"
    );
    let amount = property(&schema, "amount");
    assert_eq!(amount["minimum"], json!(-2_147_483_648_i64), "{schema:#}");
    assert_eq!(amount["maximum"], json!(2_147_483_647_i64), "{schema:#}");
}

#[test]
fn a_strict_comparison_and_the_tighter_bound_win() {
    let retries = property(&reading(), "retries");
    assert_eq!(
        retries["maximum"],
        json!(9),
        "`retries < 10` is `retries <= 9`"
    );
    assert_eq!(
        retries["minimum"],
        json!(1),
        "the tighter of `>= 0` and `>= 1`"
    );
}

#[test]
fn equality_on_a_field_that_may_be_null_does_not_become_const() {
    let level = property(&reading(), "level");
    assert!(level.get("const").is_none(), "{level:#}");
    assert_eq!(level["minimum"], json!(3), "{level:#}");
    assert_eq!(level["maximum"], json!(3), "{level:#}");
}

#[test]
fn invariants_no_keyword_says_exactly_stay_annotations() {
    let schema = reading();
    for name in ["weight", "offset"] {
        let node = property(&schema, name);
        for keyword in ["minimum", "maximum", "const"] {
            assert!(
                node.get(keyword).is_none(),
                "`{name}` gained `{keyword}`: {node:#}"
            );
        }
    }
    let invariants = &schema["$defs"]["demo.metering.Reading"]["x-ess-invariants"];
    assert_eq!(invariants.as_array().map(Vec::len), Some(9), "{schema:#}");
}

#[test]
fn a_validator_refuses_what_the_invariants_refuse() {
    let schema = reading();
    let validator = jsonschema::validator_for(&schema).expect("a valid schema");
    let valid = json!({
        "version": 2, "amount": 30, "retries": 3, "level": null,
        "weight": "1.5", "offset": 4
    });
    assert!(validator.is_valid(&valid), "{valid}");
    for (field, value) in [
        ("version", json!(3)),
        ("amount", json!(2_147_483_648_i64)),
        ("amount", json!(-2_147_483_649_i64)),
        ("retries", json!(10)),
        ("retries", json!(0)),
        ("level", json!(4)),
    ] {
        let mut instance = valid.clone();
        instance[field] = value.clone();
        assert!(
            !validator.is_valid(&instance),
            "{field} = {value} was accepted"
        );
    }
    let mut absent = valid.clone();
    absent.as_object_mut().expect("object").remove("retries");
    assert!(
        validator.is_valid(&absent),
        "an absent Optional stays valid"
    );
}
