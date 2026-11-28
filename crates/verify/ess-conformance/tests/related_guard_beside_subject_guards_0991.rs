//! Hotfix 0.99.11: a command that reads a related row through its input (`when_related:`) and also
//! refuses by its own addressed row (`when_subject:`). Every scenario synthesized for such a command
//! must pass on the interpreter, the reference target: a scenario that expects an accepting branch
//! or a related refusal is sent with an input that no `when_subject:` refusal answers first.

use std::collections::BTreeMap;

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::synthesize::synthesize;
use ess_conformance::{AdmittedSuite, AdvancingClock, ConformanceSuite, Ids, Runner, RunnerConfig};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("model.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn run_on(suite: &ConformanceSuite, target: EssIr) -> BTreeMap<String, Status> {
    let admitted = AdmittedSuite::from_suite(suite).unwrap_or_else(|error| panic!("{error}"));
    Runner::new(
        RunnerConfig::default(),
        AdvancingClock::default(),
        Ids::for_suite(suite),
    )
    .run_admitted(&admitted, &Interpreted::for_model(target))
    .into_report()
    .scenarios
    .into_iter()
    .map(|scenario| (scenario.scenario.to_string(), scenario.status))
    .collect()
}

/// Synthesizes `text`, runs the suite on the interpreter, and fails naming every scenario that did
/// not pass, with its steps; every id in `expected` must be synthesized.
fn every_scenario_passes(text: &str, expected: &[&str]) {
    let model = ir(text);
    let synthesis = synthesize(&model);
    let suite = synthesis.suite;
    let statuses = run_on(&suite, model);
    eprintln!("statuses: {statuses:#?}");
    eprintln!("refusals: {:#?}", synthesis.refusals);
    let failing: Vec<_> = statuses
        .iter()
        .filter(|(_, status)| **status != Status::Passed)
        .collect();
    let steps: Vec<_> = suite
        .scenarios
        .iter()
        .filter(|(id, _)| failing.iter().any(|(name, _)| **name == id.to_string()))
        .map(|(id, scenario)| format!("{id}: {:#?}", scenario.steps))
        .collect();
    assert!(
        failing.is_empty(),
        "failing: {failing:#?}\n{}",
        steps.join("\n")
    );
    for id in expected {
        assert!(
            statuses.contains_key(*id),
            "{id} synthesized: {statuses:#?}"
        );
    }
}

const MODEL: &str = "format: ess/37
system: demo
version: v1
domain: demo.refs
types:
  - {name: demo.refs.PersonId, kind: newtype, of: Uuid}
  - {name: demo.refs.PointerId, kind: newtype, of: Uuid}
  - {name: demo.refs.Kind, kind: enum, variants: [Movable, Fixed]}
errors:
  - {name: demo.refs.PointerNotFound}
  - {name: demo.refs.PointerFixed}
  - {name: demo.refs.PointerMoved}
  - {name: demo.refs.UnknownPerson}
events:
  - name: demo.refs.PersonAdded
    fields:
      - {name: person_id, type: demo.refs.PersonId}
  - name: demo.refs.PointerCreated
    fields:
      - {name: pointer_id, type: demo.refs.PointerId}
  - name: demo.refs.PointerMovedEvent
    fields:
      - {name: pointer_id, type: demo.refs.PointerId}
views:
  - name: demo.refs.Pointers
    source: demo.refs.Pointer
    consistency: read_your_writes
    fields:
      - {name: pointer_id, type: demo.refs.PointerId}
      - {name: kind, type: demo.refs.Kind}
      - {name: target, type: String}
entities:
  - name: demo.refs.Person
    identity: {name: person_id, type: demo.refs.PersonId}
    fields:
      - {name: name, type: String}
    lifecycle: {initial: Active, states: [Active], terminal: [Active], transitions: []}
  - name: demo.refs.Pointer
    identity: {name: pointer_id, type: demo.refs.PointerId}
    fields:
      - {name: kind, type: demo.refs.Kind}
      - {name: target, type: String}
    lifecycle: {initial: Live, states: [Live], terminal: [Live], transitions: []}
commands:
  - name: demo.refs.AddPerson
    input:
      - {name: name, type: String}
    outcomes:
      - name: added
        creates: demo.refs.Person
        instance: person_id
        sets:
          name: input.name
        emits: [demo.refs.PersonAdded]
        payload:
          demo.refs.PersonAdded:
            person_id: {generated: true}
  - name: demo.refs.CreatePointer
    input:
      - {name: kind, type: demo.refs.Kind}
      - {name: target, type: String}
    outcomes:
      - name: created
        creates: demo.refs.Pointer
        instance: pointer_id
        sets:
          kind: input.kind
          target: input.target
        emits: [demo.refs.PointerCreated]
        payload:
          demo.refs.PointerCreated:
            pointer_id: {generated: true}
  - name: demo.refs.MovePointer
    input:
      - {name: pointer_id, type: demo.refs.PointerId}
      - {name: expected, type: String}
      - {name: to, type: String}
      - {name: by, type: demo.refs.PersonId}
    outcomes:
      - name: no-such-pointer
        unknown_instance: true
        error: demo.refs.PointerNotFound
      - name: fixed
        when_subject:
          predicate: kind == Fixed
        error: demo.refs.PointerFixed
      - name: moved-since
        when_subject:
          predicate: target != input.expected
        error: demo.refs.PointerMoved
      - name: unknown-person
        when_related: {via: input.by, exists: false}
        error: demo.refs.UnknownPerson
      - name: moved
        updates: demo.refs.Pointer
        instance: pointer_id
        sets:
          target: input.to
        emits: [demo.refs.PointerMovedEvent]
        payload:
          demo.refs.PointerMovedEvent:
            pointer_id: input.pointer_id
";

/// A compare-and-set move: the accepting branch and the related refusal are sent with `expected`
/// equal to the stored `target` and a pointer that is not fixed.
#[test]
fn a_related_guard_is_arranged_past_the_commands_own_subject_refusals() {
    every_scenario_passes(
        MODEL,
        &[
            "demo.refs.MovePointer/outcome/moved",
            "demo.refs.MovePointer/outcome/unknown-person",
            "demo.refs.MovePointer/outcome/moved-since",
            "demo.refs.MovePointer/outcome/fixed",
        ],
    );
}
