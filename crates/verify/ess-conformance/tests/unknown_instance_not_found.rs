//! An unknown instance is answered with the command's declared not-found outcome, where it declares
//! one, and only otherwise with its `wrong_state` outcome.
//!
//! `docs/design/typed-literals-and-unknown-instances.md`, section 2a. A command acting on an
//! input-named instance may declare an externally decided refusal whose error reports that very
//! identity — `not-found: {external: …, error: DoorNotFound}`, `DoorNotFound {door_id}`. That is the
//! command's declared answer for an identity naming no record. The fresh-identity scenario expects
//! it, filed under its own outcome id in place of the injected one, and no scenario asserts
//! `wrong_state` for an instance that does not exist: a door that is not there is in no state.

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::synthesize::{Note, Synthesis};
use ess_conformance::{ConformanceScenario, ConformanceSuite, ScenarioStep, ScenarioValue};
use ess_domain::{spec::RawSpecFile, spec::Specification, system::Source};
use ess_primitives::node::Node;

const DOORS: &str = "format: ess/6
system: sample
version: v1
domain: sample.door
entities:
  - name: sample.door.Door
    identity: {name: door_id, type: Uuid}
    fields:
      - {name: colour, type: String}
    lifecycle:
      initial: Open
      states: [Open, Closed]
      terminal: [Closed]
      transitions:
        - {name: close, from: [Open], to: Closed}
events:
  - name: sample.door.Installed
    fields: [{name: door_id, type: Uuid}]
  - name: sample.door.Closed
    fields: [{name: door_id, type: Uuid}]
  - name: sample.door.Painted
    fields: [{name: door_id, type: Uuid}]
  - name: sample.door.Oiled
    fields: [{name: door_id, type: Uuid}]
errors:
  - name: sample.door.DoorStateConflict
    fields: [{name: door_id, type: Uuid}]
  - name: sample.door.DoorNotFound
    fields: [{name: door_id, type: Uuid}]
  - name: sample.door.DoorArchived
    fields: [{name: door_id, type: Uuid}]
  - name: sample.door.Jammed
    fields: [{name: force, type: Integer}]
commands:
  - name: sample.door.Install
    input:
      - {name: door_id, type: Uuid}
    outcomes:
      - name: installed
        creates: sample.door.Door
        instance: door_id
        sets: {colour: white}
        emits: [sample.door.Installed]
        payload:
          sample.door.Installed: {door_id: input.door_id}
  - name: sample.door.Close
    input:
      - {name: door_id, type: Uuid}
      - {name: force, type: Integer}
    outcomes:
      - name: closed
        moves: sample.door.Door.close
        instance: door_id
        emits: [sample.door.Closed]
        payload:
          sample.door.Closed: {door_id: input.door_id}
      - name: jammed
        external: the hinge is jammed
        error: sample.door.Jammed
      - name: not-found
        external: no door carries input.door_id
        error: sample.door.DoorNotFound
      - name: wrong-state
        wrong_state: true
        error: sample.door.DoorStateConflict
  - name: sample.door.Paint
    input:
      - {name: door_id, type: Uuid}
      - {name: colour, type: String}
    outcomes:
      - name: painted
        updates: sample.door.Door
        instance: door_id
        sets: {colour: input.colour}
        emits: [sample.door.Painted]
        payload:
          sample.door.Painted: {door_id: input.door_id}
      - name: not-found
        external: no door carries input.door_id
        error: sample.door.DoorNotFound
  - name: sample.door.Oil
    input:
      - {name: door_id, type: Uuid}
    outcomes:
      - name: oiled
        updates: sample.door.Door
        instance: door_id
        emits: [sample.door.Oiled]
        payload:
          sample.door.Oiled: {door_id: input.door_id}
      - name: not-found
        external: no door carries input.door_id
        error: sample.door.DoorNotFound
      - name: archived
        external: the door was archived
        error: sample.door.DoorArchived
views:
  - name: sample.door.Doors
    source: sample.door.Door
    consistency: read_your_writes
    fields:
      - {name: door_id, type: Uuid}
      - {name: colour, type: String}
      - {name: state, type: sample.door.Door.State}
";

fn synthesis() -> Synthesis {
    let raw = RawSpecFile::parse(DOORS).unwrap_or_else(|error| panic!("{error}"));
    let spec = Specification::assemble([(Source::new("doors.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    let ir: EssIr = compile(&spec, &SourceMap::new()).unwrap();
    ess_conformance::synthesize::synthesize(&ir)
}

fn find<'a>(suite: &'a ConformanceSuite, id: &str) -> Option<&'a ConformanceScenario> {
    suite
        .scenarios
        .iter()
        .find(|(key, _)| key.to_string() == id)
        .map(|(_, scenario)| scenario)
}

fn scenario<'a>(suite: &'a ConformanceSuite, id: &str) -> &'a ConformanceScenario {
    find(suite, id).unwrap_or_else(|| {
        panic!(
            "no scenario {id}; the suite holds:\n{}",
            suite
                .scenarios
                .keys()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join("\n")
        )
    })
}

fn forces(scenario: &ConformanceScenario) -> bool {
    scenario
        .steps
        .iter()
        .any(|step| matches!(step, ScenarioStep::ConfigureExternalOutcome { .. }))
}

/// Every literal `door_id` a scenario sends, other than in `except` and in the other fresh-identity
/// witnesses: those create nothing, so sharing one identity between them names no record either.
fn door_ids_sent_elsewhere(suite: &ConformanceSuite, except: &str) -> Vec<Node> {
    let mut sent = Vec::new();
    for (id, scenario) in &suite.scenarios {
        let executes = scenario
            .steps
            .iter()
            .filter(|step| matches!(step, ScenarioStep::ExecuteCommand { .. }))
            .count();
        let arranges = scenario.steps.iter().any(|step| {
            matches!(
                step,
                ScenarioStep::EstablishEntity { .. } | ScenarioStep::CaptureInstance { .. }
            )
        });
        if id.to_string() == except || (executes == 1 && !arranges && !forces(scenario)) {
            continue;
        }
        for step in &scenario.steps {
            if let ScenarioStep::ExecuteCommand { input, .. } = step {
                if let Some(ScenarioValue::Literal { value }) = input.get("door_id") {
                    sent.push(value.clone());
                }
            }
        }
    }
    sent
}

/// The fresh-identity witness: nothing arranged, nothing forced, one fresh identity sent, and the
/// not-found branch with its error required.
fn assert_fresh_identity_expects(suite: &ConformanceSuite, id: &str, command: &str, outcome: &str) {
    let unknown = scenario(suite, id);
    assert!(
        !forces(unknown),
        "{id} witnesses the cause for real instead of injecting it:\n{:#?}",
        unknown.steps
    );
    assert!(
        !unknown.steps.iter().any(|step| matches!(
            step,
            ScenarioStep::EstablishEntity { .. } | ScenarioStep::CaptureInstance { .. }
        )),
        "{id} arranges no instance:\n{:#?}",
        unknown.steps
    );
    let ScenarioStep::ExecuteCommand {
        command: sent,
        input,
        ..
    } = &unknown.steps[0]
    else {
        panic!("the command is the first step:\n{:#?}", unknown.steps);
    };
    assert_eq!(sent.to_string(), command);
    let Some(ScenarioValue::Literal { value: identity }) = input.get("door_id") else {
        panic!("the identity is a literal the suite chose: {input:#?}");
    };
    assert!(
        !door_ids_sent_elsewhere(suite, id).contains(identity),
        "`{identity:?}` is sent by another scenario too"
    );
    let expected = format!("{command}/{outcome}");
    assert!(
        unknown.steps.iter().any(|step| matches!(step,
            ScenarioStep::ExpectOutcome { outcome } if outcome.to_string() == expected)),
        "{id} expects {expected}:\n{:#?}",
        unknown.steps
    );
    assert!(
        unknown.steps.iter().any(|step| matches!(step,
            ScenarioStep::ExpectError { error, .. } if error.to_string() == "sample.door.DoorNotFound")),
        "{id} expects DoorNotFound:\n{:#?}",
        unknown.steps
    );
}

#[test]
fn a_declared_not_found_outcome_answers_the_fresh_identity_instead_of_wrong_state() {
    let synthesis = synthesis();
    assert_fresh_identity_expects(
        &synthesis.suite,
        "sample.door.Close/outcome/not-found",
        "sample.door.Close",
        "not-found",
    );
    // A door that does not exist is in no state: nothing asserts wrong-state for it.
    assert!(
        find(&synthesis.suite, "sample.door.Close/outcome/wrong-state").is_none(),
        "no scenario expects wrong-state for an identity no record carries"
    );
    for (id, scenario) in &synthesis.suite.scenarios {
        let expects_conflict = scenario.steps.iter().any(|step| matches!(step,
            ScenarioStep::ExpectError { error, .. } if error.to_string() == "sample.door.DoorStateConflict"));
        let arranges = scenario.steps.iter().any(|step| {
            matches!(
                step,
                ScenarioStep::EstablishEntity { .. } | ScenarioStep::CaptureInstance { .. }
            )
        }) || scenario
            .steps
            .iter()
            .filter(|step| matches!(step, ScenarioStep::ExecuteCommand { .. }))
            .count()
            > 1;
        assert!(
            !expects_conflict || arranges,
            "{id} expects DoorStateConflict without arranging a door:\n{:#?}",
            scenario.steps
        );
    }
    // A sibling external refusal whose error does not report the identity is not the answer, and
    // keeps its injected scenario.
    assert!(forces(scenario(
        &synthesis.suite,
        "sample.door.Close/outcome/jammed"
    )));
    assert!(
        synthesis.refusals.is_empty(),
        "nothing is refused: {:#?}",
        synthesis.refusals
    );
}

#[test]
fn a_not_found_outcome_answers_a_command_that_declares_no_wrong_state() {
    let synthesis = synthesis();
    assert_fresh_identity_expects(
        &synthesis.suite,
        "sample.door.Paint/outcome/not-found",
        "sample.door.Paint",
        "not-found",
    );
    assert!(
        !synthesis
            .notes
            .iter()
            .any(|note| note.to_string().contains("sample.door.Paint")),
        "Paint declares its answer, so it is witnessed and not noted: {:?}",
        synthesis.notes
    );
}

#[test]
fn two_candidate_answers_are_a_note_and_neither_is_assumed() {
    let synthesis = synthesis();
    assert!(
        synthesis.notes.iter().any(|note| matches!(
            note,
            Note::UnknownInstanceAmbiguous { command, .. } if command.to_string() == "sample.door.Oil"
        )),
        "Oil declares two refusals reporting the identity: {:?}",
        synthesis.notes
    );
    for outcome in ["not-found", "archived"] {
        assert!(
            forces(scenario(
                &synthesis.suite,
                &format!("sample.door.Oil/outcome/{outcome}")
            )),
            "Oil/{outcome} keeps its injected scenario"
        );
    }
}
