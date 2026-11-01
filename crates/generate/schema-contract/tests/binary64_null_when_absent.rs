//! A nullable `Optional<Binary64>` field realizes.
//!
//! A field declared `presence: null_when_absent` or `omitted_or_null` publishes its `Optional`
//! whole, as `anyOf [number, null]`, so its finite Binary64 identity sits at the `anyOf`'s number
//! member, not at the property. The model's Binary64 locations must point there, in a command's
//! input and response roots and in a declared struct alike, or Rust and Go realization refuse the
//! selection with `model_binary64_layout`.

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

use ess_compiler::{resolve::compile as compile_model, source::SourceMap, EssIr};
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source,
};
use ess_gen::schema::ModelTypes;
use schema_contract::realize::Plan;
use serde_json::{json, Value};

const SOURCE: &str = r"format: ess/31
system: demo
version: v1
domain: demo.lab
types:
  - {name: demo.lab.SampleId, kind: newtype, of: String}
  - name: demo.lab.Reading
    kind: struct
    fields:
      - {name: value, type: 'Optional<Binary64>', presence: null_when_absent}
      - {name: spare, type: 'Optional<Binary64>', presence: omitted_or_null}
      - {name: plain, type: 'Optional<Binary64>'}
entities:
  - name: demo.lab.Sample
    identity: {name: sample_id, type: demo.lab.SampleId}
    fields:
      - {name: label, type: demo.lab.SampleId}
    lifecycle: {initial: Active, states: [Active], terminal: [Active], transitions: []}
commands:
  - name: demo.lab.Record
    input:
      - {name: sample_id, type: demo.lab.SampleId, example: s-1}
      - {name: value, type: 'Optional<Binary64>', presence: null_when_absent}
    response:
      - {name: sample_id, type: demo.lab.SampleId}
      - {name: value, type: 'Optional<Binary64>', presence: null_when_absent}
      - {name: spare, type: 'Optional<Binary64>', presence: omitted_or_null}
    outcomes:
      - name: recorded
        returns: true
        creates: demo.lab.Sample
        instance: sample_id
        sets: {label: input.sample_id}
        emits: [demo.lab.Recorded]
        payload:
          demo.lab.Recorded: {sample_id: input.sample_id}
events:
  - name: demo.lab.Recorded
    fields:
      - {name: sample_id, type: demo.lab.SampleId}
components:
  - component: lab-service
    reached_by: network
    owns: {domains: [demo.lab]}
    accepts: {commands: [demo.lab.Record]}
    publishes: {events: [demo.lab.Recorded]}
";

const RESPONSE: &str = "demo.lab.Record.response";
const INPUT: &str = "demo.lab.Record.input";
const STRUCT: &str = "demo.lab.Reading";

fn compiled() -> EssIr {
    let mut sources = SourceMap::new();
    sources.insert(Source::DOCUMENT, SOURCE.to_owned());
    let specification =
        Specification::assemble([(Source::document(), RawSpecFile::parse(SOURCE).unwrap())])
            .unwrap_or_else(|errors| panic!("validates: {errors}"));
    compile_model(&specification, &sources).unwrap_or_else(|errors| panic!("compiles: {errors}"))
}

fn selection(roots: &[&str]) -> ModelTypes {
    ModelTypes::select(
        &compiled(),
        &roots.iter().map(|root| (*root).to_owned()).collect(),
    )
    .unwrap_or_else(|errors| panic!("selects: {errors:?}"))
}

/// The pointer to `field`'s number under `root`'s definition: the `anyOf`'s first member for a
/// nullable field, the property itself for a field that is only left out when absent.
fn number(root: &str, field: &str, nullable: bool) -> String {
    let property = format!("/$defs/{root}/properties/{field}");
    if nullable {
        format!("{property}/anyOf/0")
    } else {
        property
    }
}

/// The example body the `OpenAPI` projection publishes for the command's input: each input
/// property's first `examples` entry, and `null` for a property that publishes none. A Binary64
/// input carries no `example:` (a conformance witness cannot carry one), so its `null_when_absent`
/// field is spelt the one way an absent value is.
fn published_input_example() -> Value {
    let ir = compiled();
    let component = ir.components().values().next().expect("one component");
    let document: Value =
        serde_json::from_str(&ess_gen::openapi::json(&ir, component)).expect("JSON");
    let input = &document["components"]["schemas"]["demo.lab.Record.Input"];
    Value::Object(
        input["properties"]
            .as_object()
            .unwrap_or_else(|| panic!("published input properties: {input:#}"))
            .iter()
            .map(|(key, property)| {
                let example = property
                    .get("examples")
                    .map_or(Value::Null, |examples| examples[0].clone());
                (key.clone(), example)
            })
            .collect(),
    )
}

fn compile_and_test(manifest: &str, source: &str, tests: &str) {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let root = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!(
        "binary64-null-when-absent-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir_all(root.join("tests")).unwrap();
    fs::write(root.join("Cargo.toml"), manifest).unwrap();
    fs::write(root.join("types.rs"), source).unwrap();
    fs::write(root.join("tests/wire.rs"), tests).unwrap();
    let result = Command::new(env!("CARGO"))
        .args(["test", "--offline", "--quiet", "--manifest-path"])
        .arg(root.join("Cargo.toml"))
        .env(
            "CARGO_TARGET_DIR",
            Path::new(env!("CARGO_TARGET_TMPDIR")).join("native-types-target"),
        )
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
}

/// Rust and Go realize the selection, TypeScript names the number member in its obligation, and
/// the generated Rust library passes `tests`.
fn realizes(selected: &ModelTypes, expected: &BTreeSet<String>, tests: &str) {
    for at in expected {
        assert!(
            selected.binary64_locations().contains(at),
            "{at} is not among {:?}",
            selected.binary64_locations()
        );
    }
    let plan = Plan::from_model(selected).unwrap_or_else(|refused| panic!("plan: {refused:?}"));
    let report = json!(plan.typescript().report);
    let obligations = report["obligations"]
        .as_array()
        .expect("obligations")
        .iter()
        .filter(|item| item["rule"] == "model_binary64")
        .map(|item| item["pointer"].as_str().expect("a pointer").to_owned())
        .collect::<BTreeSet<_>>();
    for at in expected {
        assert!(
            obligations.contains(at),
            "TypeScript names no Binary64 obligation at {at}: {report:#}"
        );
    }
    plan.go("lab_types", "example.invalid/labtypes")
        .unwrap_or_else(|refused| panic!("Go realization: {refused:?}"));
    let rust = plan
        .rust("lab_types")
        .unwrap_or_else(|refused| panic!("Rust realization: {refused:?}"));
    compile_and_test(
        &rust.supporting["Cargo.toml"],
        &rust.declarations,
        &format!("{WIRE}{tests}"),
    );
}

/// What every generated-library case below uses. A body round-trips only when the whole
/// re-encoded document equals it, so a `null_when_absent` key the type drops fails, where indexing
/// a `Value` would read the missing key as `null`. `binary64` reads a field through the finite
/// Binary64 codec's type, so a plain float in its place does not compile, and `-0` keeps its sign.
const WIRE: &str = r#"
use lab_types::*;
use serde::{de::DeserializeOwned, Serialize};
use serde_json::{json, Value};

fn round_trips<T: DeserializeOwned + Serialize>(body: Value) -> T {
    let text = body.to_string();
    let decoded: T = serde_json::from_str(&text).unwrap_or_else(|error| panic!("{text}: {error}"));
    let again: Value = serde_json::from_str(&serde_json::to_string(&decoded).unwrap()).unwrap();
    assert_eq!(again, body, "{text} re-encodes as another document");
    decoded
}

fn decodes<T: DeserializeOwned>(text: &str) -> T {
    serde_json::from_str(text).unwrap_or_else(|error| panic!("{text}: {error}"))
}

fn refuses<T: DeserializeOwned>(text: &str) {
    assert!(serde_json::from_str::<T>(text).is_err(), "{text}");
}

fn binary64(value: Option<EssBinary64>) -> Option<u64> {
    value.map(|value| value.get().to_bits())
}

const NEGATIVE_ZERO: u64 = 0x8000_0000_0000_0000;
"#;

/// A `null_when_absent` response field decodes a number and a present `null`, each written back
/// whole; the body without the key, a string and a token with no finite binary64 image (`1e400`)
/// are refused. The input decodes `EXAMPLE`, the body the `OpenAPI` projection publishes as the
/// input's example, and the same body with a number.
const RESPONSE_WIRE: &str = r#"
#[test]
fn a_null_when_absent_response_field_decodes() {
    let present: DemoLabRecordResponse = round_trips(json!({"sample_id": "s-1", "value": 1.5}));
    assert_eq!(binary64(present.value), Some(1.5f64.to_bits()));
    let absent: DemoLabRecordResponse = round_trips(json!({"sample_id": "s-1", "value": null}));
    assert_eq!(binary64(absent.value), None);
    round_trips::<DemoLabRecordResponse>(json!({"sample_id": "s-1", "value": 1.5, "spare": 2.5}));
    round_trips::<DemoLabRecordResponse>(json!({"sample_id": "s-1", "value": null, "spare": null}));
    let zero: DemoLabRecordResponse = decodes("{\"sample_id\":\"s-1\",\"value\":-0}");
    assert_eq!(binary64(zero.value), Some(NEGATIVE_ZERO));
    refuses::<DemoLabRecordResponse>("{\"sample_id\":\"s-1\"}");
    refuses::<DemoLabRecordResponse>("{\"sample_id\":\"s-1\",\"value\":\"1.5\"}");
    refuses::<DemoLabRecordResponse>("{\"sample_id\":\"s-1\",\"value\":1e400}");
    refuses::<DemoLabRecordResponse>("{\"sample_id\":\"s-1\",\"value\":null,\"spare\":1e400}");
}

#[test]
fn the_published_input_example_decodes() {
    let example: Value = serde_json::from_str(EXAMPLE).unwrap();
    assert_eq!(example, json!({"sample_id": "s-1", "value": null}));
    let decoded: DemoLabRecordInput = round_trips(example);
    assert_eq!(binary64(decoded.value), None);
    let present: DemoLabRecordInput = round_trips(json!({"sample_id": "s-1", "value": 1.5}));
    assert_eq!(binary64(present.value), Some(1.5f64.to_bits()));
    refuses::<DemoLabRecordInput>("{\"sample_id\":\"s-1\"}");
    refuses::<DemoLabRecordInput>("{\"sample_id\":\"s-1\",\"value\":1e400}");
}
"#;

/// The same for a declared struct, beside an `omitted_or_null` and a plain `Optional` field.
const STRUCT_WIRE: &str = r#"
#[test]
fn a_null_when_absent_struct_field_decodes() {
    let present: DemoLabReading = round_trips(json!({"value": 1.5}));
    assert_eq!(binary64(present.value), Some(1.5f64.to_bits()));
    let absent: DemoLabReading = round_trips(json!({"value": null}));
    assert_eq!(binary64(absent.value), None);
    round_trips::<DemoLabReading>(json!({"value": 1.5, "spare": 2.5, "plain": 3.5}));
    round_trips::<DemoLabReading>(json!({"value": null, "spare": null}));
    let zero: DemoLabReading = decodes("{\"value\":-0}");
    assert_eq!(binary64(zero.value), Some(NEGATIVE_ZERO));
    refuses::<DemoLabReading>("{\"spare\":2.5}");
    refuses::<DemoLabReading>("{\"value\":\"1.5\"}");
    refuses::<DemoLabReading>("{\"value\":1e400}");
    refuses::<DemoLabReading>("{\"value\":null,\"spare\":1e400}");
    refuses::<DemoLabReading>("{\"value\":null,\"plain\":1e400}");
}
"#;

#[test]
fn a_null_when_absent_binary64_response_field_realizes() {
    let selected = selection(&[RESPONSE, INPUT]);
    let expected = BTreeSet::from([
        number(RESPONSE, "value", true),
        number(RESPONSE, "spare", true),
        number(INPUT, "value", true),
    ]);
    assert_eq!(
        selected.binary64_locations(),
        &expected,
        "every Binary64 of the input and the response, at its number"
    );
    let example = published_input_example().to_string();
    realizes(
        &selected,
        &expected,
        &format!("const EXAMPLE: &str = {example:?};\n{RESPONSE_WIRE}"),
    );
}

#[test]
fn a_null_when_absent_binary64_struct_field_realizes() {
    let selected = selection(&[STRUCT]);
    let expected = BTreeSet::from([
        number(STRUCT, "value", true),
        number(STRUCT, "spare", true),
        number(STRUCT, "plain", false),
    ]);
    assert_eq!(
        selected.binary64_locations(),
        &expected,
        "every Binary64 of the struct, at its number"
    );
    realizes(&selected, &expected, STRUCT_WIRE);
}
