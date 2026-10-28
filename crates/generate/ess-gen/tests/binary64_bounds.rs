//! A `Binary64` bound an invariant states becomes a numeric keyword (beyond10x/ess#687).
//!
//! `>` publishes `exclusiveMinimum`, `>=` `minimum`, `<` `exclusiveMaximum`, `<=` `maximum`, and
//! `==` both `minimum` and `maximum`, on a struct field, an entity field and a newtype's `value`,
//! but only for a literal that is the shortest spelling of its own binary64: anything else would
//! publish a bound a binary64 validator reads as a different number. Every invariant stays under
//! `x-ess-invariants`. A literal on the left (`0 < x`) is read as its mirror; no authored spelling
//! writes one, so `src/types.rs` tests that on a constructed predicate.

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
domain: demo.sensor
types:
  - {name: demo.sensor.ProbeId, kind: newtype, of: Uuid}
  - name: demo.sensor.Rate
    kind: newtype
    of: Binary64
    invariants:
      - value > 0
      - value <= 1
  - name: demo.sensor.Humidity
    kind: newtype
    of: Binary64
    invariants:
      - value >= 0
      - value < 100.5
  - name: demo.sensor.Sample
    kind: struct
    fields:
      - {name: selection_rate, type: Binary64}
      - {name: humidity, type: Optional<Binary64>}
      - {name: gain, type: Binary64}
      - {name: offset, type: Optional<Binary64>}
      - {name: level, type: Binary64}
      - {name: tare, type: Binary64}
      - {name: rate, type: demo.sensor.Rate}
      - {name: moisture, type: demo.sensor.Humidity}
    invariants:
      - selection_rate > 0
      - selection_rate <= 1
      - humidity >= 0
      - humidity < 100.5
      - gain == 2.5
      - offset == -1.25
      - level < 9007199254740993
      - tare != 0
entities:
  - name: demo.sensor.Probe
    identity: {name: probe_id, type: demo.sensor.ProbeId}
    fields:
      - {name: reading, type: Binary64}
      - {name: pressure, type: Binary64}
    invariants:
      - reading > -40
      - reading <= 85.5
      - pressure >= 0.5
      - pressure < 2000
    lifecycle: {initial: Active, states: [Active], terminal: [Active], transitions: []}
";

fn compiled(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("well formed: {error}"));
    let spec = Specification::assemble([(Source::new("sensor.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("validates: {errors}"));
    let mut sources = SourceMap::new();
    sources.insert("sensor.yaml", text);
    compile(&spec, &sources).unwrap_or_else(|errors| panic!("compiles: {errors}"))
}

fn document(text: &str, path: &str) -> Value {
    let artifacts = run(&JsonSchema, &compiled(text)).expect("generates");
    let artifact = artifacts.get(path).unwrap_or_else(|| {
        panic!(
            "no `{path}` among {:?}",
            artifacts.keys().collect::<Vec<_>>()
        )
    });
    serde_json::from_str(&artifact.contents).expect("JSON")
}

fn sample() -> Value {
    document(MODEL, "schema/types/demo.sensor.Sample.schema.json")
}

fn property(schema: &Value, name: &str) -> Value {
    schema["$defs"]["demo.sensor.Sample"]["properties"][name].clone()
}

/// The four bound keywords a node carries, absent ones as `null`.
fn bounds(node: &Value) -> Value {
    json!({
        "minimum": node.get("minimum"),
        "exclusiveMinimum": node.get("exclusiveMinimum"),
        "maximum": node.get("maximum"),
        "exclusiveMaximum": node.get("exclusiveMaximum"),
    })
}

#[test]
fn a_strict_binary64_bound_becomes_an_exclusive_keyword() {
    let schema = sample();
    // An integral bound is an integer token: `json!(0)` is not `json!(0.0)`.
    assert_eq!(
        bounds(&property(&schema, "selection_rate")),
        json!({"minimum": null, "exclusiveMinimum": 0, "maximum": 1, "exclusiveMaximum": null}),
        "{schema:#}"
    );
    assert_eq!(
        bounds(&property(&schema, "humidity")),
        json!({"minimum": 0, "exclusiveMinimum": null, "maximum": null, "exclusiveMaximum": 100.5}),
        "{schema:#}"
    );
    assert_eq!(property(&schema, "selection_rate")["type"], json!("number"));

    let defs = &schema["$defs"];
    assert_eq!(
        bounds(&defs["demo.sensor.Rate"]),
        json!({"minimum": null, "exclusiveMinimum": 0, "maximum": 1, "exclusiveMaximum": null}),
        "{schema:#}"
    );
    assert_eq!(
        bounds(&defs["demo.sensor.Humidity"]),
        json!({"minimum": 0, "exclusiveMinimum": null, "maximum": null, "exclusiveMaximum": 100.5}),
        "{schema:#}"
    );

    let entity = document(MODEL, "schema/entities/demo.sensor.Probe.schema.json");
    assert_eq!(
        bounds(&entity["properties"]["reading"]),
        json!({"minimum": null, "exclusiveMinimum": -40, "maximum": 85.5, "exclusiveMaximum": null}),
        "{entity:#}"
    );
    assert_eq!(
        bounds(&entity["properties"]["pressure"]),
        json!({"minimum": 0.5, "exclusiveMinimum": null, "maximum": null, "exclusiveMaximum": 2000}),
        "{entity:#}"
    );
}

#[test]
fn binary64_equality_becomes_minimum_and_maximum() {
    let schema = sample();
    for (name, value) in [("gain", json!(2.5)), ("offset", json!(-1.25))] {
        let node = property(&schema, name);
        assert!(node.get("const").is_none(), "`{name}`: {node:#}");
        assert_eq!(
            bounds(&node),
            json!({"minimum": value, "exclusiveMinimum": null, "maximum": value,
                "exclusiveMaximum": null}),
            "`{name}`: {node:#}"
        );
    }
}

#[test]
fn a_literal_that_is_not_its_own_shortest_spelling_stays_an_annotation() {
    let schema = sample();
    let unbounded = json!({"minimum": null, "exclusiveMinimum": null, "maximum": null,
        "exclusiveMaximum": null});
    for name in ["level", "tare"] {
        let node = property(&schema, name);
        assert_eq!(bounds(&node), unbounded, "`{name}`: {node:#}");
        assert!(node.get("const").is_none(), "`{name}`: {node:#}");
    }
    let invariants = &schema["$defs"]["demo.sensor.Sample"]["x-ess-invariants"];
    assert!(
        invariants
            .as_array()
            .expect("invariants")
            .contains(&json!("level < 9007199254740993")),
        "{schema:#}"
    );
    assert_eq!(invariants.as_array().map(Vec::len), Some(8), "{schema:#}");
}

#[test]
fn a_validator_refuses_what_the_binary64_invariants_refuse() {
    let schema = sample();
    let validator = jsonschema::validator_for(&schema).expect("a valid schema");
    let valid = json!({
        "selection_rate": 0.5, "humidity": 50, "gain": 2.5, "level": 3, "tare": 1,
        "rate": 0.5, "moisture": 10
    });
    assert!(validator.is_valid(&valid), "{valid} against {schema:#}");
    let mut one = valid.clone();
    one["selection_rate"] = json!(1);
    assert!(validator.is_valid(&one), "`selection_rate <= 1` admits 1");
    for (field, value) in [
        ("selection_rate", json!(0)),
        ("selection_rate", json!(-0.0)),
        ("selection_rate", json!(1.5)),
        ("humidity", json!(-0.5)),
        ("humidity", json!(100.5)),
        ("gain", json!(2.4)),
        ("rate", json!(0)),
        ("rate", json!(1.5)),
        ("moisture", json!(100.5)),
    ] {
        let mut instance = valid.clone();
        instance[field] = value.clone();
        assert!(
            !validator.is_valid(&instance),
            "{field} = {value} was accepted"
        );
    }
}
