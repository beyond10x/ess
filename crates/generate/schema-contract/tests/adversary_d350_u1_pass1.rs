//! Adversary, wave D350 unit U1, pass 1: a nullable `Optional<Binary64>` field realizes as a
//! Binary64 (story:a-null-when-absent-binary64-field-realizes).
//!
//! The unit's own cases decode `1.5` and `null`, which any number type decodes, and read a `null`
//! back through `Value`'s index, which answers `null` for a missing key too. These cases hold the
//! generated Rust library to what the Binary64 location is for: the finite codec at the `anyOf`'s
//! number member (a token past the binary64 range is refused, an integer token past 2^53 is read
//! as the nearest binary64), and a `null` written back as a present key.

#[allow(dead_code)]
#[path = "fixtures/normalization_model.rs"]
mod model;

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

use schema_contract::realize::Plan;
use serde_json::json;

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
  - name: demo.lab.Odd
    kind: struct
    fields:
      - {name: odd, wire: 'a/b~', type: 'Optional<Binary64>', presence: null_when_absent}
      - {name: rest, wire: 'c~1', type: 'Optional<Binary64>', presence: omitted_or_null}
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

fn compile_and_test(manifest: &str, source: &str, tests: &str) {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let root = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!(
        "adversary-d350-u1-{}-{}",
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

const WIRE: &str = r##"
use lab_types::*;
use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value;

fn written<T: DeserializeOwned + Serialize>(text: &str) -> String {
    let decoded: T = serde_json::from_str(text).unwrap_or_else(|error| panic!("{text}: {error}"));
    serde_json::to_string(&decoded).unwrap()
}

/// The body decodes and is written back as the same JSON document, keys and `null`s included.
fn round_trips<T: DeserializeOwned + Serialize>(text: &str) {
    let out = written::<T>(text);
    let read: Value = serde_json::from_str(text).unwrap();
    let back: Value = serde_json::from_str(&out).unwrap();
    assert_eq!(back, read, "{text} was written back as {out}");
}

fn refuses<T: DeserializeOwned>(text: &str) {
    assert!(serde_json::from_str::<T>(text).is_err(), "{text} decodes");
}

#[test]
fn a_null_is_written_back_as_a_present_null() {
    for text in [
        r#"{"value":null}"#,
        r#"{"value":null,"spare":null}"#,
        r#"{"value":1.5,"spare":null,"plain":2.5}"#,
    ] {
        round_trips::<DemoLabReading>(text);
    }
    for text in [
        r#"{"sample_id":"s-1","value":null}"#,
        r#"{"sample_id":"s-1","value":null,"spare":null}"#,
        r#"{"sample_id":"s-1","value":1.5}"#,
    ] {
        round_trips::<DemoLabRecordResponse>(text);
    }
    round_trips::<DemoLabRecordInput>(r#"{"sample_id":"s-1","value":null}"#);
}

#[test]
fn a_nullable_member_refuses_a_token_past_the_binary64_range() {
    refuses::<DemoLabReading>(r#"{"value":1e400}"#);
    refuses::<DemoLabReading>(r#"{"value":null,"spare":-1e400}"#);
    refuses::<DemoLabReading>(r#"{"value":null,"plain":1e400}"#);
    refuses::<DemoLabRecordResponse>(r#"{"sample_id":"s-1","value":1e400}"#);
    refuses::<DemoLabRecordResponse>(r#"{"sample_id":"s-1","value":null,"spare":1e400}"#);
    refuses::<DemoLabRecordInput>(r#"{"sample_id":"s-1","value":1e400}"#);
}

#[test]
fn a_nullable_member_reads_the_nearest_binary64() {
    for (text, key) in [
        (r#"{"value":9007199254740993}"#, "value"),
        (r#"{"value":null,"spare":9007199254740993}"#, "spare"),
        (r#"{"value":null,"plain":9007199254740993}"#, "plain"),
    ] {
        let out = written::<DemoLabReading>(text);
        let back: Value = serde_json::from_str(&out).unwrap();
        assert_eq!(back[key].as_f64(), Some(9007199254740992.0), "{out}");
        assert!(!out.contains("9007199254740993"), "{text} kept its exact token: {out}");
    }
    let out = written::<DemoLabRecordResponse>(r#"{"sample_id":"s-1","value":9007199254740993}"#);
    assert!(!out.contains("9007199254740993"), "the response kept its exact token: {out}");
}
"##;

/// The struct, the response and the input in one generated library, held to the Binary64 codec
/// at each nullable member and to a `null` written back as a key.
#[test]
fn nullable_binary64_members_decode_as_binary64_and_write_null_back() {
    let selected = model::selection(
        SOURCE,
        &[
            "demo.lab.Reading",
            "demo.lab.Record.response",
            "demo.lab.Record.input",
        ],
    );
    let plan = Plan::from_model(&selected).unwrap_or_else(|refused| panic!("plan: {refused:?}"));
    let rust = plan
        .rust("lab_types")
        .unwrap_or_else(|refused| panic!("Rust realization: {refused:?}"));
    compile_and_test(&rust.supporting["Cargo.toml"], &rust.declarations, WIRE);
}

/// A nullable member under a wire name that needs JSON Pointer escaping: the location escapes the
/// key and then descends into the `anyOf`, and every target realizes it.
#[test]
fn a_nullable_binary64_under_an_escaped_wire_name_realizes() {
    let selected = model::selection(SOURCE, &["demo.lab.Odd"]);
    let expected = BTreeSet::from([
        "/$defs/demo.lab.Odd/properties/a~1b~0/anyOf/0".to_owned(),
        "/$defs/demo.lab.Odd/properties/c~01/anyOf/0".to_owned(),
    ]);
    assert_eq!(selected.binary64_locations(), &expected);
    let plan = Plan::from_model(&selected).unwrap_or_else(|refused| panic!("plan: {refused:?}"));
    plan.rust("odd_types")
        .unwrap_or_else(|refused| panic!("Rust realization: {refused:?}"));
    plan.go("odd_types", "example.invalid/oddtypes")
        .unwrap_or_else(|refused| panic!("Go realization: {refused:?}"));
    let report = json!(plan.typescript().report);
    let pointers = report["obligations"]
        .as_array()
        .expect("obligations")
        .iter()
        .filter(|item| item["rule"] == "model_binary64")
        .map(|item| item["pointer"].as_str().expect("a pointer").to_owned())
        .collect::<BTreeSet<_>>();
    assert_eq!(pointers, expected, "{report:#}");
}
