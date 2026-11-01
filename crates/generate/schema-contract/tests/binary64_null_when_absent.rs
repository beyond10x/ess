//! A nullable `Optional<Binary64>` field realizes.
//!
//! A field declared `presence: null_when_absent` or `omitted_or_null` publishes its `Optional`
//! whole, as `anyOf [number, null]`, so its finite Binary64 identity sits at the `anyOf`'s number
//! member, not at the property. The model's Binary64 locations must point there, in a command's
//! input and response roots and in a declared struct alike, or Rust and Go realization refuse the
//! selection with `model_binary64_layout`.
//!
//! These cases build nothing. That the generated Rust library decodes such a member through the
//! Binary64 codec, writes a present `null` back and decodes the published input example is held
//! by the Binary64 wire corpus `binary64_structural.rs` builds once, whose model carries the same
//! struct, input and response (`fixtures/binary64_wire_tests.rs.txt`, module `nullable`).

use std::collections::BTreeSet;

use ess_compiler::{resolve::compile as compile_model, source::SourceMap, EssIr};
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source,
};
use ess_gen::schema::ModelTypes;
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

/// Rust and Go realize the selection, TypeScript names the number member in its obligation, and
/// the generated Rust declares each named member through the Binary64 codec: `fields` pairs a Rust
/// field with the type it must be declared as.
fn realizes(selected: &ModelTypes, expected: &BTreeSet<String>, fields: &[(&str, &str)]) {
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
        .unwrap_or_else(|refused| panic!("Rust realization: {refused:?}"))
        .declarations;
    for (field, declared) in fields {
        assert!(
            rust.contains(&format!("pub {field}: {declared},")),
            "`{field}` is not declared as `{declared}`:\n{rust}"
        );
    }
}

const NULLABLE: &str = "::std::option::Option<EssBinary64>";
const OMITTED_OR_NULL: &str = "EssPresence<::std::option::Option<EssBinary64>>";

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
    realizes(
        &selected,
        &expected,
        &[("value", NULLABLE), ("spare", OMITTED_OR_NULL)],
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
    realizes(
        &selected,
        &expected,
        &[
            ("value", NULLABLE),
            ("spare", OMITTED_OR_NULL),
            ("plain", "EssPresence<EssBinary64>"),
        ],
    );
}
