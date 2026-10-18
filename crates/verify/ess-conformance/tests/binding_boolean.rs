//! A binding condition and a binding selection comparing a Boolean leaf with `true` or `false`
//! (ess/27, downstream report, 2026-10-18).
//!
//! Two bindings split one event by a Boolean flag beside an enum, and a third selects the first
//! list item whose flag is `false`. The `condition-false` scenario flips the flag, the suite passes
//! against the interpreter and through the generated Go runner, and a target that ignores the flag
//! fails the scenario that is about it.

mod support_go;

use std::collections::BTreeMap;

use ess_compiler::{resolve::compile, source::SourceMap, EssIr};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::synthesize::{synthesize, Synthesis};
use ess_conformance::{AdmittedSuite, ConformanceSuite, Runner};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use serde_json::Value;

const CONDITIONS: &str = r"
format: ess/27
system: demo
version: v1
domain: demo.calls
types:
  - {name: demo.calls.Direction, kind: enum, variants: [inbound, outbound]}
events:
  - name: demo.calls.CallFinished
    fields:
      - {name: call_id, type: String}
      - {name: direction, type: demo.calls.Direction}
      - {name: bridged, type: Boolean}
  - name: demo.calls.DirectRecorded
    fields: [{name: call_id, type: String}]
  - name: demo.calls.BridgedRecorded
    fields: [{name: call_id, type: String}]
commands:
  - name: demo.calls.FinishCall
    input:
      - {name: call_id, type: String}
      - {name: direction, type: demo.calls.Direction}
      - {name: bridged, type: Boolean}
    outcomes:
      - name: finished
        emits: [demo.calls.CallFinished]
        payload:
          demo.calls.CallFinished: {call_id: input.call_id, direction: input.direction, bridged: input.bridged}
  - name: demo.calls.RecordDirect
    input: [{name: call_id, type: String}]
    outcomes:
      - name: recorded
        emits: [demo.calls.DirectRecorded]
        payload: {demo.calls.DirectRecorded: {call_id: input.call_id}}
  - name: demo.calls.RecordBridged
    input: [{name: call_id, type: String}]
    outcomes:
      - name: recorded
        emits: [demo.calls.BridgedRecorded]
        payload: {demo.calls.BridgedRecorded: {call_id: input.call_id}}
bindings:
  - id: direct
    when:
      event: demo.calls.CallFinished
      where: [event.direction == inbound, event.bridged == false]
    invoke: {command: demo.calls.RecordDirect}
    mapping: {call_id: event.call_id}
    delivery: at_least_once
    on_failure: drop
  - id: bridged
    when:
      event: demo.calls.CallFinished
      where: [event.bridged != false]
    invoke: {command: demo.calls.RecordBridged}
    mapping: {call_id: event.call_id}
    delivery: at_least_once
    on_failure: drop
";

const SELECTION: &str = r"
format: ess/27
system: demo
version: v1
domain: demo.calls
types:
  - name: demo.calls.Leg
    kind: struct
    fields:
      - {name: id, type: String}
      - {name: held, type: Boolean}
events:
  - name: demo.calls.CallEnded
    fields:
      - {name: call_id, type: String}
      - {name: legs, type: List<demo.calls.Leg>}
  - name: demo.calls.LegReleased
    fields: [{name: leg_id, type: Optional<String>}]
commands:
  - name: demo.calls.EndCall
    input:
      - {name: call_id, type: String}
      - {name: legs, type: List<demo.calls.Leg>}
    outcomes:
      - name: ended
        emits: [demo.calls.CallEnded]
        payload:
          demo.calls.CallEnded: {call_id: input.call_id, legs: input.legs}
  - name: demo.calls.ReleaseLeg
    input: [{name: leg_id, type: Optional<String>}]
    outcomes:
      - name: released
        emits: [demo.calls.LegReleased]
        payload: {demo.calls.LegReleased: {leg_id: input.leg_id}}
bindings:
  - id: release-free-leg
    when: {event: demo.calls.CallEnded}
    selection_inputs:
      - {name: legs, from: event.legs, as: List<demo.calls.Leg>}
    selections:
      - name: free
        first: {in: legs, where: item.held == false}
    invoke: {command: demo.calls.ReleaseLeg}
    mapping:
      leg_id: {selection: free, path: [id]}
    delivery: at_least_once
    on_failure: drop
";

fn model(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).expect("the model parses");
    let spec = Specification::assemble([(Source::new("calls.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("the model validates:\n{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|d| panic!("the model resolves:\n{d}"))
}

fn statuses(suite: &ConformanceSuite, target: &Interpreted) -> BTreeMap<String, Status> {
    let admitted = AdmittedSuite::from_suite(suite).expect("the suite is admitted");
    Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, target)
        .into_report()
        .scenarios
        .iter()
        .map(|scenario| (scenario.scenario.to_string(), scenario.status))
        .collect()
}

fn not_passed(statuses: &BTreeMap<String, Status>) -> Vec<String> {
    statuses
        .iter()
        .filter(|(_, status)| **status != Status::Passed)
        .map(|(id, status)| format!("{id} {status:?}"))
        .collect()
}

/// The `bridged` value each trigger of scenario `id` sends.
fn sent_flags(synthesis: &Synthesis, id: &str) -> Vec<Value> {
    let document: Value =
        serde_json::from_str(&synthesis.suite.to_canonical_json().expect("serialises"))
            .expect("JSON");
    document["scenarios"][id]["steps"]
        .as_array()
        .unwrap_or_else(|| panic!("`{id}` is synthesized"))
        .iter()
        .filter(|step| step["command"] == "demo.calls.FinishCall")
        .map(|step| step["input"]["bridged"]["value"].clone())
        .collect()
}

#[test]
fn condition_false_flips_the_flag_the_condition_holds_for() {
    let synthesis = synthesize(&model(CONDITIONS));
    for (binding, holds) in [("direct", false), ("bridged", true)] {
        assert_eq!(
            sent_flags(&synthesis, &format!("{binding}/binding/flow")),
            [Value::Bool(holds)],
            "{binding}: the flow sends the flag its condition holds for"
        );
        assert_eq!(
            sent_flags(&synthesis, &format!("{binding}/binding/condition-false")),
            [Value::Bool(!holds)],
            "{binding}: condition-false flips it"
        );
    }
}

#[test]
fn boolean_conditions_pass_the_interpreter_and_fail_a_target_that_ignores_the_flag() {
    let suite = synthesize(&model(CONDITIONS)).suite;
    let honest = statuses(&suite, &Interpreted::for_model(model(CONDITIONS)));
    assert_eq!(not_passed(&honest), Vec::<String>::new());
    // A dispatcher that runs `direct` for every inbound call, bridged or not.
    let ignoring = CONDITIONS.replace(
        "where: [event.direction == inbound, event.bridged == false]",
        "where: [event.direction == inbound]",
    );
    let faulty = statuses(&suite, &Interpreted::for_model(model(&ignoring)));
    assert_ne!(
        faulty["direct/binding/condition-false"],
        Status::Passed,
        "{faulty:#?}"
    );
}

#[test]
fn a_boolean_selection_passes_the_interpreter_and_fails_a_target_that_ignores_the_flag() {
    let suite = synthesize(&model(SELECTION)).suite;
    let honest = statuses(&suite, &Interpreted::for_model(model(SELECTION)));
    assert_eq!(not_passed(&honest), Vec::<String>::new());
    assert!(honest.contains_key("release-free-leg/binding/mapping"));
    // A selector that takes the first leg whatever its flag.
    let ignoring = SELECTION.replace("where: item.held == false", "where: defined(item.id)");
    let faulty = statuses(&suite, &Interpreted::for_model(model(&ignoring)));
    assert_ne!(
        faulty["release-free-leg/binding/mapping"],
        Status::Passed,
        "{faulty:#?}"
    );
}

#[test]
fn the_generated_go_runner_agrees_on_boolean_conditions_and_selections() {
    for (label, text) in [
        ("boolean-conditions", CONDITIONS),
        ("boolean-selection", SELECTION),
    ] {
        let suite = synthesize(&model(text)).suite;
        let verdicts =
            support_go::assert_parity(label, &suite, Interpreted::for_model(model(text)));
        let failed: Vec<_> = verdicts
            .iter()
            .filter(|(_, status)| *status != "passed")
            .collect();
        assert_eq!(failed, Vec::<(&String, &String)>::new(), "{label}");
    }
}

#[test]
fn below_ess_27_the_comparison_is_refused_naming_ess_27() {
    for text in [CONDITIONS, SELECTION] {
        let below = text.replace("format: ess/27", "format: ess/26");
        let raw = RawSpecFile::parse(&below).expect("the model parses");
        let errors = Specification::assemble([(Source::new("calls.yaml"), raw)])
            .expect_err("a Boolean comparison is refused below ess/27")
            .to_string();
        assert!(
            errors.contains("unsupported_format_version") && errors.contains("ess/27"),
            "{errors}"
        );
    }
}
