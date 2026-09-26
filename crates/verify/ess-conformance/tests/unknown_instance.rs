//! An unknown instance is answered with the command's `wrong_state` outcome (beyond10x/ess#113).
//!
//! `docs/design/typed-literals-and-unknown-instances.md`, section 2. A command whose input selects
//! a `moves:` or `updates:` branch, and whose `instance:` names no record, answers its
//! `wrong_state` branch when it declares one. Synthesis witnesses the rule once per such command,
//! with a fresh identity, under the `wrong_state` outcome's own id. A command declaring no
//! `wrong_state` has no declared answer, and that is a coverage note rather than a refusal.

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::synthesize::{Note, Synthesis};
use ess_conformance::{ConformanceScenario, ConformanceSuite, ScenarioStep, ScenarioValue};
use ess_domain::{spec::RawSpecFile, spec::Specification, system::Source};
use ess_primitives::node::Node;

const DOORS: &str = "format: ess/4
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
errors:
  - name: sample.door.DoorStateConflict
  - name: sample.door.TooForceful
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
      - name: slammed
        when: force > 10
        error: sample.door.TooForceful
      - name: closed
        moves: sample.door.Door.close
        instance: door_id
        emits: [sample.door.Closed]
        payload:
          sample.door.Closed: {door_id: input.door_id}
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
views:
  - name: sample.door.Doors
    source: sample.door.Door
    consistency: read_your_writes
    fields:
      - {name: door_id, type: Uuid}
      - {name: colour, type: String}
      - {name: state, type: sample.door.Door.State}
";

const UNKNOWN: &str = "sample.door.Close/outcome/wrong-state";

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}"));
    let spec = Specification::assemble([(Source::new("doors.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap()
}

fn synthesis(text: &str) -> Synthesis {
    ess_conformance::synthesize::synthesize(&ir(text))
}

fn scenario<'a>(suite: &'a ConformanceSuite, id: &str) -> &'a ConformanceScenario {
    suite
        .scenarios
        .iter()
        .find(|(key, _)| key.to_string() == id)
        .map_or_else(
            || {
                panic!(
                    "no scenario {id}; the suite holds:\n{}",
                    suite
                        .scenarios
                        .keys()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                        .join("\n")
                )
            },
            |(_, scenario)| scenario,
        )
}

/// Every literal `door_id` a scenario sends, other than in the scenario named `except`.
fn door_ids_sent_elsewhere(suite: &ConformanceSuite, except: &str) -> Vec<Node> {
    let mut sent = Vec::new();
    for (id, scenario) in &suite.scenarios {
        if id.to_string() == except {
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

#[test]
fn a_command_declaring_wrong_state_is_witnessed_on_an_identity_no_record_carries() {
    let synthesis = synthesis(DOORS);
    let unknown = scenario(&synthesis.suite, UNKNOWN);

    // Nothing is arranged: the identity names no record because none was made.
    assert!(
        !unknown.steps.iter().any(|step| matches!(
            step,
            ScenarioStep::EstablishEntity { .. } | ScenarioStep::CaptureInstance { .. }
        )),
        "the scenario arranges no instance:\n{:#?}",
        unknown.steps
    );
    let ScenarioStep::ExecuteCommand { command, input, .. } = &unknown.steps[0] else {
        panic!("the command is the first step:\n{:#?}", unknown.steps);
    };
    assert_eq!(command.to_string(), "sample.door.Close");
    let Some(ScenarioValue::Literal { value: identity }) = input.get("door_id") else {
        panic!("the identity is a literal the suite chose: {input:#?}");
    };
    // The input reaches the moving branch — `slammed` is decided by input first and would answer
    // its own error whatever the instance names.
    let Some(ScenarioValue::Literal {
        value: Node::Number(force),
    }) = input.get("force")
    else {
        panic!("`force` is a number: {input:#?}");
    };
    assert!(
        force.get() <= 10.0,
        "the input must not reach `slammed`: {force:?}"
    );

    // Fresh: no other scenario sends it, so a target shared between scenarios holds no record of it.
    assert!(
        !door_ids_sent_elsewhere(&synthesis.suite, UNKNOWN).contains(identity),
        "`{identity:?}` is sent by another scenario too"
    );

    // The declared answer: the branch, its error, and nothing published.
    let outcome = unknown.steps.iter().any(|step| {
        matches!(step, ScenarioStep::ExpectOutcome { outcome }
            if outcome.to_string() == "sample.door.Close/wrong-state")
    });
    let error = unknown.steps.iter().any(|step| {
        matches!(step, ScenarioStep::ExpectError { error, .. }
            if error.to_string() == "sample.door.DoorStateConflict")
    });
    let quiet: Vec<String> = unknown
        .steps
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::ExpectNoEvent { event } => Some(event.to_string()),
            _ => None,
        })
        .collect();
    assert!(
        outcome && error,
        "branch and error required:\n{:#?}",
        unknown.steps
    );
    for event in [
        "sample.door.Installed",
        "sample.door.Closed",
        "sample.door.Painted",
    ] {
        assert!(
            quiet.contains(&event.to_owned()),
            "{event} must not be published: {quiet:?}"
        );
    }
}

#[test]
fn a_command_without_wrong_state_is_a_coverage_note_and_not_a_refusal() {
    let synthesis = synthesis(DOORS);
    assert!(
        synthesis.notes.iter().any(|note| matches!(
            note,
            Note::UnknownInstanceUnanswered { command } if command.to_string() == "sample.door.Paint"
        )),
        "Paint declares no wrong_state: {:?}",
        synthesis.notes
    );
    assert!(
        !synthesis
            .notes
            .iter()
            .any(|note| note.to_string().contains("sample.door.Close")),
        "Close declares one, so it is witnessed and not noted: {:?}",
        synthesis.notes
    );
    // Not a refusal: the specification states nothing left unwitnessed.
    assert!(
        synthesis
            .refusals
            .iter()
            .all(|refusal| !refusal.to_string().contains("sample.door.Paint/outcome")),
        "{:#?}",
        synthesis.refusals
    );
    let note = synthesis.notes[0].to_string();
    assert!(
        note.contains("wrong_state") && note.contains("no record"),
        "the note says what is undeclared: {note}"
    );
}

#[test]
fn a_command_that_creates_only_has_no_unknown_instance_to_answer() {
    let synthesis = synthesis(DOORS);
    assert!(
        !synthesis
            .notes
            .iter()
            .any(|note| note.to_string().contains("sample.door.Install")),
        "{:?}",
        synthesis.notes
    );
}
