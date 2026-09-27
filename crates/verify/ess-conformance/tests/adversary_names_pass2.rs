//! Adversary pass 2 on story:field-names-underscore-and-newtype-map-keys (beyond10x/ess#143).
//!
//! The specification now admits `Map<demo.orders.ItemId, Boolean>` where `ItemId` is a newtype of
//! `String`. An authored scenario (`ess-scenario/3`) declares its fixtures with the same type
//! spelling the specification uses, and is read by a loader that is not `RawSpecFile::parse`.

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source,
};

const MODEL: &str = r#"
format: ess/14
system: demo
version: v1
domain: demo.orders
types:
  - name: demo.orders.ItemId
    kind: newtype
    of: String
events:
  - name: demo.orders.Checked
    fields:
      - {name: results, type: "Map<demo.orders.ItemId, Boolean>"}
commands:
  - name: demo.orders.Check
    input:
      - {name: results, type: "Map<demo.orders.ItemId, Boolean>"}
    fixture_inputs:
      results: current-results
    outcomes:
      - name: checked
        emits: [demo.orders.Checked]
        payload:
          demo.orders.Checked:
            results: input.results
"#;

const AUTHORED: &str = r#"
type: ess-scenario/3
domain: demo.orders
scenario: checked-results
summary: Provisioned results are echoed on the event
fixtures:
  current-results: "FIXTURE_TYPE"
timeline:
  - at: '2026-09-22T00:00:00Z'
    command: demo.orders.Check
    input:
      results: {$fixture: current-results}
    outcome: checked
    events:
      - event: demo.orders.Checked
        payload:
          results: {$fixture: current-results}
"#;

fn ir() -> EssIr {
    let raw = RawSpecFile::parse(MODEL).unwrap_or_else(|error| panic!("well formed: {error}"));
    let spec = Specification::assemble([(Source::new("orders.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("validates: {errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|errors| panic!("compiles: {errors}"))
}

fn authored(fixture_type: &str) -> ess_conformance::authored::Authoring {
    ess_conformance::authored::compile(
        &ir(),
        &[ess_conformance::authored::Source::new(
            "authored.yaml",
            AUTHORED.replace("FIXTURE_TYPE", fixture_type),
        )],
    )
}

#[test]
fn adversary_pass2_control_a_string_keyed_fixture_compiles() {
    let result = authored("Map<String, Boolean>");
    assert!(result.refusals.is_empty(), "{:#?}", result.refusals);
    assert_eq!(result.scenarios.len(), 1);
}

#[test]
fn adversary_pass2_an_authored_fixture_may_spell_the_newtype_keyed_map_the_model_admits() {
    let result = authored("Map<demo.orders.ItemId, Boolean>");
    assert!(
        result.refusals.is_empty(),
        "beyond10x/ess#143: the model admits `Map<demo.orders.ItemId, Boolean>`, so a scenario \
         fixture spelled the same way must be readable: {:#?}",
        result.refusals
    );
    assert_eq!(result.scenarios.len(), 1);
}
