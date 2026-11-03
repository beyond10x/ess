//! A view field an outcome sets through a declared conversion
//! (`story:view-asserts-field-set-from-optional-input`).
//!
//! A downstream adopter's model stored `Optional<String>` inputs into `String` fields its view
//! publishes, behind a refusal answering an absent value first. Validation then requires the
//! declared conversion `Optional<String> -> String`, and synthesis left every such field out of the
//! row it asserts, so a `sets-retarget` mutant joining two of them survived.
//!
//! The conversion from `Optional<T>` to the `T` it wraps hands a present value on unchanged: the
//! row holds exactly what the scenario sent, and the scenario asserts it
//! (`docs/design/binding-mapping-bounded-accessor.md`, "Converted values in a row"). Any other
//! conversion still computes a value nothing declares; the field stays out of the row and a note
//! names the field and the conversion.

use ess_compiler::ir::EssIr;
use ess_compiler::source::SourceMap;
use ess_conformance::interpret::Interpreted;
use ess_conformance::mutate::{self, Document, MutantClass};
use ess_conformance::report::Status;
use ess_conformance::scenario::{ScenarioStep, ScenarioValue, ViewExpectation};
use ess_conformance::synthesize::{synthesize, Synthesis};
use ess_conformance::{AdmittedSuite, Runner, ScenarioId};
use ess_domain::spec::RawSpecFile;
use ess_domain::system::Source;
use ess_primitives::node::Node;
use std::collections::BTreeMap;

/// Variant 2 of the adopter's reproduction: two `Optional` inputs and a refusal on either's
/// absence, so `created` needs the declared conversion to store them.
const CONVERTED: &str = r"
format: ess/23
system: repro
version: v1
domain: repro.config

types:
  - name: repro.config.ConfigId
    kind: newtype
    of: String

conversions:
  - from: Optional<String>
    to: String
    because: missing-field refuses an absent value before created runs

entities:
  - name: repro.config.Config
    identity: {name: config_id, type: repro.config.ConfigId}
    fields:
      - {name: site, type: String}
      - {name: client_id, type: String}
    lifecycle: {initial: Configured, states: [Configured], terminal: [Configured]}

errors:
  - name: repro.config.MissingField
    summary: a field is absent
    fields: []

commands:
  - name: repro.config.CreateConfig
    input:
      - {name: config_id, type: repro.config.ConfigId}
      - {name: site, type: Optional<String>}
      - {name: client_id, type: Optional<String>}
    outcomes:
      - name: missing-field
        when:
          any: [{site: {exists: false}}, {client_id: {exists: false}}]
        error: repro.config.MissingField
      - name: created
        creates: repro.config.Config
        instance: config_id
        sets: {site: input.site, client_id: input.client_id}
        emits: [repro.config.ConfigCreated]
        payload:
          repro.config.ConfigCreated: {config_id: input.config_id}

events:
  - name: repro.config.ConfigCreated
    fields:
      - {name: config_id, type: repro.config.ConfigId}

views:
  - name: repro.config.Configs
    source: repro.config.Config
    consistency: read_your_writes
    fields:
      - {name: config_id, type: repro.config.ConfigId}
      - {name: site, type: String}
      - {name: client_id, type: String}
      - {name: state, type: repro.config.Config.State}
";

/// Variant 1: one `Optional` input whose own absence is refused first; no conversion is needed.
const UNCONVERTED: &str = r"
format: ess/23
system: repro
version: v1
domain: repro.config

types:
  - name: repro.config.ConfigId
    kind: newtype
    of: String

entities:
  - name: repro.config.Config
    identity: {name: config_id, type: repro.config.ConfigId}
    fields:
      - {name: site, type: String}
    lifecycle: {initial: Configured, states: [Configured], terminal: [Configured]}

errors:
  - name: repro.config.MissingField
    summary: site is absent
    fields: []

commands:
  - name: repro.config.CreateConfig
    input:
      - {name: config_id, type: repro.config.ConfigId}
      - {name: site, type: Optional<String>}
    outcomes:
      - name: missing-field
        when: {site: {exists: false}}
        error: repro.config.MissingField
      - name: created
        creates: repro.config.Config
        instance: config_id
        sets: {site: input.site}
        emits: [repro.config.ConfigCreated]
        payload:
          repro.config.ConfigCreated: {config_id: input.config_id}

events:
  - name: repro.config.ConfigCreated
    fields:
      - {name: config_id, type: repro.config.ConfigId}

views:
  - name: repro.config.Configs
    source: repro.config.Config
    consistency: read_your_writes
    fields:
      - {name: config_id, type: repro.config.ConfigId}
      - {name: site, type: String}
      - {name: state, type: repro.config.Config.State}
";

/// A conversion between two newtypes of different representations: what it computes is the
/// implementation's, so no scenario knows the value the row holds.
const COMPUTED: &str = r"
format: ess/23
system: repro
version: v1
domain: repro.parcel

types:
  - name: repro.parcel.ParcelId
    kind: newtype
    of: String
  - name: repro.parcel.Kilos
    kind: newtype
    of: Integer
  - name: repro.parcel.Grams
    kind: newtype
    of: Decimal

conversions:
  - from: repro.parcel.Kilos
    to: repro.parcel.Grams
    because: the scale factor is the implementation's and no declaration here states it

entities:
  - name: repro.parcel.Parcel
    identity: {name: parcel_id, type: repro.parcel.ParcelId}
    fields:
      - {name: weight, type: repro.parcel.Grams}
    lifecycle: {initial: Weighed, states: [Weighed], terminal: [Weighed]}

commands:
  - name: repro.parcel.WeighParcel
    input:
      - {name: parcel_id, type: repro.parcel.ParcelId}
      - {name: weight, type: repro.parcel.Kilos}
    outcomes:
      - name: weighed
        creates: repro.parcel.Parcel
        instance: parcel_id
        sets: {weight: input.weight}
        emits: [repro.parcel.ParcelWeighed]
        payload:
          repro.parcel.ParcelWeighed: {parcel_id: input.parcel_id}

events:
  - name: repro.parcel.ParcelWeighed
    fields:
      - {name: parcel_id, type: repro.parcel.ParcelId}

views:
  - name: repro.parcel.Parcels
    source: repro.parcel.Parcel
    consistency: read_your_writes
    fields:
      - {name: parcel_id, type: repro.parcel.ParcelId}
      - {name: weight, type: repro.parcel.Grams}
      - {name: state, type: repro.parcel.Parcel.State}
";

const CREATED: &str = "repro.config.CreateConfig/outcome/created";

fn documents(text: &str) -> (Vec<Document>, SourceMap) {
    let raw = RawSpecFile::parse(text).expect("the fixture is well formed");
    let mut texts = SourceMap::new();
    texts.insert("fixture.yaml".to_owned(), text.to_owned());
    (vec![(Source::new("fixture.yaml"), raw)], texts)
}

fn compiled((files, texts): &(Vec<Document>, SourceMap)) -> EssIr {
    mutate::compile(files.clone(), texts).unwrap_or_else(|stillborn| {
        panic!("the model compiles: {} {}", stillborn.code, stillborn.cause)
    })
}

fn steps<'a>(synthesis: &'a Synthesis, id: &str) -> &'a [ScenarioStep] {
    let parsed = ScenarioId::parse(id).expect("a scenario id");
    &synthesis
        .suite
        .scenario(&parsed)
        .unwrap_or_else(|| {
            panic!(
                "`{id}` is in the suite; it refuses {:?}",
                synthesis
                    .refusals
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
            )
        })
        .steps
}

/// The literal input the last `command` send of `id` carries.
fn sent(synthesis: &Synthesis, id: &str, command: &str) -> BTreeMap<String, Node> {
    steps(synthesis, id)
        .iter()
        .rev()
        .find_map(|step| match step {
            ScenarioStep::ExecuteCommand {
                command: sent,
                input,
                ..
            } if sent.to_string() == command => Some(
                input
                    .iter()
                    .filter_map(|(field, value)| {
                        value.as_literal().map(|node| (field.clone(), node.clone()))
                    })
                    .collect(),
            ),
            _ => None,
        })
        .unwrap_or_else(|| panic!("`{id}` sends `{command}`"))
}

/// Every row `id` requires present in a view.
fn required_rows(synthesis: &Synthesis, id: &str) -> Vec<BTreeMap<String, ScenarioValue>> {
    steps(synthesis, id)
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::ExpectView {
                expectation: ViewExpectation::Contains { fields },
                ..
            }
            | ScenarioStep::EventuallyView {
                expectation: ViewExpectation::Contains { fields },
                ..
            } => Some(fields.clone()),
            _ => None,
        })
        .collect()
}

#[test]
fn a_field_set_through_an_optional_unwrapping_conversion_is_asserted_at_the_value_sent() {
    let synthesis = synthesize(&compiled(&documents(CONVERTED)));
    let input = sent(&synthesis, CREATED, "repro.config.CreateConfig");
    let rows = required_rows(&synthesis, CREATED);
    assert!(!rows.is_empty(), "`{CREATED}` reads `Configs` back");
    for field in ["site", "client_id"] {
        let value = input
            .get(field)
            .unwrap_or_else(|| panic!("`{CREATED}` sends `{field}`: {input:?}"));
        assert!(
            rows.iter().any(|row| row.get(field)
                == Some(&ScenarioValue::Literal {
                    value: value.clone()
                })),
            "`{CREATED}` requires a `Configs` row holding `{field}` = {value:?}, the value it \
             sent across `Optional<String> -> String`; it requires {rows:#?}"
        );
    }
}

#[test]
fn a_field_set_from_an_optional_input_without_a_conversion_keeps_its_assertion() {
    let synthesis = synthesize(&compiled(&documents(UNCONVERTED)));
    let input = sent(&synthesis, CREATED, "repro.config.CreateConfig");
    let site = input.get("site").expect("`created` sends `site`");
    assert!(
        required_rows(&synthesis, CREATED)
            .iter()
            .any(|row| row.get("site")
                == Some(&ScenarioValue::Literal {
                    value: site.clone()
                })),
        "variant 1 asserts `site` as it did before"
    );
}

#[test]
fn a_sets_retarget_mutant_on_a_converted_field_is_killed() {
    let model = documents(CONVERTED);
    let synthesis = synthesize(&compiled(&model));
    let suite = &synthesis.suite;
    let admitted = AdmittedSuite::from_suite(suite).expect("the suite admits");
    let failing = |ir: EssIr| -> Vec<String> {
        Runner::for_suite(suite)
            .run_admitted(&admitted, &Interpreted::for_model(ir))
            .into_report()
            .scenarios
            .iter()
            .filter(|scenario| scenario.status != Status::Passed)
            .map(|scenario| scenario.scenario.to_string())
            .collect()
    };
    assert_eq!(
        failing(compiled(&model)),
        Vec::<String>::new(),
        "the model the suite was synthesized from passes it"
    );
    let (files, texts) = &model;
    let retargets: Vec<_> = mutate::mutants(files, &[MutantClass::SetsRetarget])
        .into_iter()
        .filter(|mutant| mutant.id.contains("site"))
        .collect();
    assert!(
        !retargets.is_empty(),
        "the audit finds a retarget on `site`"
    );
    for mutant in retargets {
        let mutated = mutate::apply(files, &mutant.mutation).expect("the site exists");
        let ir = mutate::compile(mutated, texts).unwrap_or_else(|stillborn| {
            panic!(
                "{} is not stillborn: {} {}",
                mutant.id, stillborn.code, stillborn.cause
            )
        });
        assert!(
            !failing(ir).is_empty(),
            "`{}` ({}) survives the suite",
            mutant.id,
            mutant.change
        );
    }
}

#[test]
fn a_conversion_synthesis_cannot_compute_leaves_the_field_out_with_a_note() {
    let synthesis = synthesize(&compiled(&documents(COMPUTED)));
    let id = "repro.parcel.WeighParcel/outcome/weighed";
    let rows = required_rows(&synthesis, id);
    assert!(!rows.is_empty(), "`{id}` reads `Parcels` back");
    assert!(
        rows.iter().all(|row| !row.contains_key("weight")),
        "`{id}` asserts `weight` across `Kilos -> Grams`, which computes a value nothing \
         declares: {rows:#?}"
    );
    let notes: Vec<String> = synthesis.notes.iter().map(ToString::to_string).collect();
    assert!(
        notes.iter().any(|note| note.contains("`weight`")
            && note.contains("repro.parcel.Kilos")
            && note.contains("repro.parcel.Grams")),
        "a note names `weight` and the conversion `Kilos -> Grams`: {notes:#?}"
    );
}
