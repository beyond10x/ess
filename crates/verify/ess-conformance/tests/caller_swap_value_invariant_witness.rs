//! beyond10x/ess#766, adversary: the caller-swapped run renames every instance it binds and
//! recaptures every observed reference it reads, the one a `value_invariants` witness holds
//! included. That witness selects the row the run arranged; in the swapped run it must select the
//! swapped run's row, never the first run's.

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::synthesize::synthesize;
use ess_conformance::value_invariant::{Selector, Witness};
use ess_conformance::{ScenarioStep, ScenarioValue, ViewExpectation};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};

const MODEL: &str = r#"format: ess/24
system: demo
version: v1
domain: demo.tags
types:
  - {name: demo.tags.BoardId, kind: newtype, of: Uuid}
  - name: demo.tags.Tag
    kind: newtype
    of: String
    alphabet: "abcdefghijklmnopqrstuvwxyz0123456789.-"
    invariants: [value.count >= 1]
entities:
  - name: demo.tags.Board
    identity: {name: board_id, type: demo.tags.BoardId}
    fields:
      - {name: tags, type: 'List<demo.tags.Tag>'}
      - {name: owner, type: String}
    lifecycle:
      initial: Open
      states: [Open]
      terminal: [Open]
actors:
  - name: demo.tags.Member
    attributes:
      - {name: owner, type: String}
    may: [demo.tags.OpenBoard]
events:
  - name: demo.tags.BoardOpened
    fields:
      - {name: board_id, type: demo.tags.BoardId}
commands:
  - name: demo.tags.OpenBoard
    input:
      - {name: tags, type: 'List<demo.tags.Tag>'}
    outcomes:
      - name: opened
        creates: demo.tags.Board
        instance: board_id
        sets: {tags: input.tags, owner: {caller: owner}}
        emits: [demo.tags.BoardOpened]
        payload:
          demo.tags.BoardOpened: {board_id: {generated: true}}
views:
  - name: demo.tags.Boards
    source: demo.tags.Board
    consistency: read_your_writes
    fields:
      - {name: board_id, type: demo.tags.BoardId}
      - {name: tags, type: 'List<demo.tags.Tag>'}
"#;

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}"));
    let spec = Specification::assemble([(Source::new("system.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn witness_of(step: &ScenarioStep) -> Option<ScenarioValue> {
    let expectation = match step {
        ScenarioStep::ExpectView { expectation, .. }
        | ScenarioStep::EventuallyView { expectation, .. } => expectation,
        _ => return None,
    };
    let ViewExpectation::ValueInvariants(observed) = expectation else {
        return None;
    };
    match &observed.witness {
        Witness::ArrangedRow {
            selector:
                Selector::ProjectedIdentity { expected, .. } | Selector::IdentityQuery { expected, .. },
        } => Some(expected.clone()),
        Witness::AnyObservedOccurrence => None,
    }
}

/// No swapped run re-checks the first run's row: a swapped run that reuses a value-invariant
/// witness reads a row it binds itself, and where it can capture none (the witness reads an
/// `observed` reference no earlier step expects) no swapped run is appended at all.
#[test]
fn no_swapped_run_of_a_value_invariant_scenario_rechecks_the_first_runs_row() {
    let ir = ir(MODEL);
    let synthesis = synthesize(&ir);
    let (id, scenario) = synthesis
        .suite
        .scenarios
        .iter()
        .find(|(id, _)| id.to_string().starts_with("demo.tags.Tag/"))
        .unwrap_or_else(|| {
            panic!(
                "a value-invariant scenario is synthesized: {:#?}",
                synthesis.refusals
            )
        });
    let text = serde_json::to_string_pretty(&scenario.steps).unwrap();
    let witnesses: Vec<ScenarioValue> = scenario.steps.iter().filter_map(witness_of).collect();
    assert!(
        !witnesses.is_empty(),
        "{id}: the first run checks a value invariant\n{text}"
    );
    for swapped in &witnesses[1..] {
        let ScenarioValue::Instance { instance } = swapped else {
            panic!(
                "{id}: a swapped run's witness reads the swapped run's captured row, not the \
                 first run's observed reference: {swapped:?}\n{text}"
            );
        };
        assert!(
            instance.as_str().ends_with("-swapped"),
            "{id}: a swapped run's witness names a row the swapped run binds: {instance:?}\n{text}"
        );
    }
}
