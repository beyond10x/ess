//! Synthesis asks the precedence plan which branches answer before a witness
//! (`story:synthesis-reads-selection-plan`, `docs/design/selection-plan.md`).
//!
//! `Operate` closes an open door pushed as `PUSH` names (`shut`, a held-state branch) and declares
//! the external `stalled` beside it. Under the precedence order the held state (step 4) answers
//! before the external branch (step 6), so `stalled`'s witness on an open door is pushed the other
//! way, which `shut` does not claim (beyond10x/ess#464). With the two phases exchanged through the
//! plan's one test seam (`with_phase_order`), `stalled` answers first and nothing it has to refute
//! is left: its witness is the plain one, the first variant, `Gentle`. With `PUSH` = `Gentle` that is
//! the input `shut` claims, so the witness moves. The model is compiled once, outside the exchange;
//! only synthesis and the model interpreter run inside it.
use std::collections::BTreeMap;

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::scenario::ConformanceSuite;
use ess_conformance::synthesize::Synthesis;
use ess_conformance::{AdmittedSuite, ConformanceScenario, Runner, ScenarioStep, ScenarioValue};
use ess_domain::command::precedence::{with_phase_order, Phase};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

const DOOR: &str = r"format: ess/20
system: demo
version: v1
domain: demo.doors
summary: A door closed gently, or jammed when its motor stalls.
types:
  - {name: demo.doors.DoorId, kind: newtype, of: Uuid}
  - {name: demo.doors.Push, kind: enum, variants: [Gentle, Rough]}
entities:
  - name: demo.doors.Door
    identity: {name: door_id, type: demo.doors.DoorId}
    fields:
      - {name: label, type: String}
    lifecycle:
      initial: Open
      states: [Open, Closed, Jammed]
      terminal: [Closed, Jammed]
      transitions:
        - {name: close, from: [Open], to: Closed}
        - {name: jam, from: [Open], to: Jammed}
actors:
  - name: demo.doors.Keeper
    may: [demo.doors.Install, demo.doors.Operate]
commands:
  - name: demo.doors.Install
    input:
      - {name: label, type: String}
    outcomes:
      - name: installed
        creates: demo.doors.Door
        instance: door_id
        sets: {label: input.label}
        emits: [demo.doors.Installed]
        payload:
          demo.doors.Installed: {door_id: {generated: true}}
  - name: demo.doors.Operate
    input:
      - {name: door_id, type: demo.doors.DoorId}
      - {name: push, type: demo.doors.Push}
    outcomes:
      - name: shut
        when_subject_state: Open
        when: push == PUSH
        moves: demo.doors.Door.close
        instance: door_id
        emits: [demo.doors.Shut]
        payload:
          demo.doors.Shut: {door_id: input.door_id}
      - name: stalled
        external: the motor stalls
        moves: demo.doors.Door.jam
        instance: door_id
        emits: [demo.doors.Stalled]
        payload:
          demo.doors.Stalled: {door_id: input.door_id}
      - name: refused
        error: demo.doors.NotOpen
events:
  - name: demo.doors.Installed
    fields: [{name: door_id, type: demo.doors.DoorId}]
  - name: demo.doors.Shut
    fields: [{name: door_id, type: demo.doors.DoorId}]
  - name: demo.doors.Stalled
    fields: [{name: door_id, type: demo.doors.DoorId}]
errors:
  - name: demo.doors.NotOpen
views:
  - name: demo.doors.Doors
    source: demo.doors.Door
    consistency: read_your_writes
    fields:
      - {name: door_id, type: demo.doors.DoorId}
      - {name: label, type: String}
      - {name: state, type: demo.doors.Door.State}
";

const OPERATE: &str = "demo.doors.Operate";
const STALLED: &str = "demo.doors.Operate/outcome/stalled";
const JAMMED: &str = "demo.doors.Door/transition/jam/by/demo.doors.Operate/stalled";

/// The precedence order with the held-state and accepting/external phases exchanged.
fn exchanged() -> [Phase; 8] {
    Phase::PRECEDENCE.map(|phase| match phase {
        Phase::HeldState => Phase::Accepting,
        Phase::Accepting => Phase::HeldState,
        other => other,
    })
}

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let mut texts = SourceMap::new();
    texts.insert("model.yaml".to_owned(), text.to_owned());
    let spec = Specification::assemble([(Source::new("model.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &texts).unwrap_or_else(|error| panic!("{error:?}"))
}

fn scenario<'a>(result: &'a Synthesis, id: &str) -> &'a ConformanceScenario {
    result
        .suite
        .scenarios
        .iter()
        .find(|(key, _)| key.to_string() == id)
        .map_or_else(
            || {
                let refusals: Vec<String> =
                    result.refusals.iter().map(ToString::to_string).collect();
                panic!("no scenario {id}; refusals: {refusals:#?}")
            },
            |(_, scenario)| scenario,
        )
}

/// The input of the send `stalled`'s provider is forced for.
fn forced_push(scenario: &ConformanceScenario) -> ScenarioValue {
    let at = scenario
        .steps
        .iter()
        .position(|step| {
            matches!(step, ScenarioStep::ConfigureExternalOutcome { force, .. }
                if force.command.name().to_string() == OPERATE
                    && force.outcome.to_string() == "stalled")
        })
        .unwrap_or_else(|| panic!("no forced provider: {scenario:#?}"));
    match &scenario.steps[at + 1..]
        .iter()
        .find(|step| matches!(step, ScenarioStep::ExecuteCommand { .. }))
    {
        Some(ScenarioStep::ExecuteCommand { command, input, .. })
            if command.name().to_string() == OPERATE =>
        {
            input["push"].clone()
        }
        other => panic!("the forced provider is not followed by a send of {OPERATE}: {other:#?}"),
    }
}

fn variant(name: &str) -> ScenarioValue {
    ScenarioValue::literal(ess_primitives::node::Node::Text(name.into()))
}

/// Every scenario of `suite` run on the model interpreter of `ir`, by id, where it did not pass.
fn not_passed(ir: EssIr, suite: &ConformanceSuite) -> Vec<String> {
    let admitted = AdmittedSuite::from_suite(suite).unwrap_or_else(|error| panic!("{error}"));
    let statuses: BTreeMap<String, Status> = Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &Interpreted::for_model(ir))
        .into_report()
        .scenarios
        .into_iter()
        .map(|result| (result.scenario.to_string(), result.status))
        .collect();
    statuses
        .into_iter()
        .filter(|(_, status)| *status != Status::Passed)
        .map(|(id, status)| format!("{id}: {status:?}"))
        .collect()
}

#[test]
fn exchanging_held_state_and_accepting_moves_the_external_witness() {
    // The plain witness `stalled` is sent where nothing it must refute is left: the first variant.
    let plain = "Gentle";
    for (push, other) in [("Gentle", "Rough"), ("Rough", "Gentle")] {
        let model = ir(&DOOR.replace("PUSH", push));
        let declared = ess_conformance::synthesize::synthesize(&model);
        let swapped = with_phase_order(exchanged(), || {
            ess_conformance::synthesize::synthesize(&model)
        });
        for id in [STALLED, JAMMED] {
            assert_eq!(
                forced_push(scenario(&declared, id)),
                variant(other),
                "{push}: {id}: under the precedence order `shut` answers first, so `stalled` is \
                 pushed the way `shut` does not claim"
            );
            assert_eq!(
                forced_push(scenario(&swapped, id)),
                variant(plain),
                "{push}: {id}: with the phases exchanged `stalled` answers before `shut`, so its \
                 witness is the plain one, whether or not `shut` claims it"
            );
        }
        let failed = with_phase_order(exchanged(), || not_passed(model.clone(), &swapped.suite));
        assert_eq!(
            failed,
            Vec::<String>::new(),
            "{push}: the interpreter, reading the same exchanged order, passes the suite"
        );
    }
}
