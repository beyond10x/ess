//! Adversary, wave D350 unit U1, pass 1: a nullable `Optional<Binary64>` field realizes as a
//! Binary64 (story:a-null-when-absent-binary64-field-realizes).
//!
//! The unit's own cases decode `1.5` and `null`, which any number type decodes, and read a `null`
//! back through `Value`'s index, which answers `null` for a missing key too. These cases hold the
//! generated Rust library to what the Binary64 location is for: the finite codec at the `anyOf`'s
//! number member (a token past the binary64 range is refused, an integer token past 2^53 is read
//! as the nearest binary64), and a `null` written back as a present key.
//!
//! The decoding cases run in the Binary64 wire corpus `binary64_structural.rs` builds once
//! (`fixtures/binary64_wire_tests.rs.txt`, module `nullable`), because no test spawns a build of
//! its own (`ess-xtask/tests/no_test_builds.rs`); this file holds what needs no build.

#[allow(dead_code)]
#[path = "fixtures/normalization_model.rs"]
mod model;

use std::collections::BTreeSet;

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

/// The struct, the response and the input in one generated library, each nullable member declared
/// through the Binary64 codec. Decoding them (a `null` written back as a key, a token past the
/// binary64 range refused, an integer past 2^53 read as the nearest binary64) runs in the Binary64
/// wire corpus `binary64_structural.rs` builds once (`fixtures/binary64_wire_tests.rs.txt`, module
/// `nullable`), so this file spawns no build of its own.
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
        .unwrap_or_else(|refused| panic!("Rust realization: {refused:?}"))
        .declarations;
    for declared in [
        "pub value: ::std::option::Option<EssBinary64>,",
        "pub spare: EssPresence<::std::option::Option<EssBinary64>>,",
        "pub plain: EssPresence<EssBinary64>,",
    ] {
        assert!(
            rust.contains(declared),
            "story:a-null-when-absent-binary64-field-realizes: no `{declared}` in\n{rust}"
        );
    }
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
