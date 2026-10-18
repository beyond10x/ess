//! A binding whose invoked command chooses its branch by a `when_related` guard over the row an
//! event field names (downstream report, 2026-10-18).
//!
//! The row is named by an input the trigger's caller supplies, so a scenario can arrange it before
//! the trigger: the flow, mapping and delivery scenarios create the `Plan` through `DefinePlan` so
//! the invocation reaches the accepting branch, and send `Join` naming it. A guard that also reads
//! the invoked command's input, or a row named by a value the implementation mints, still leaves
//! the branch to values the scenario does not choose, and keeps its refusal.

use ess_compiler::{resolve::compile, source::SourceMap, EssIr};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::synthesize::{synthesize, Synthesis};
use ess_conformance::{AdmittedSuite, Runner};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use serde_json::Value;

const FIXTURE: &str = r"
format: ess/24
system: demo
version: v1
domain: demo.seats
types:
  - {name: demo.seats.PlanId, kind: newtype, of: Uuid}
  - {name: demo.seats.MemberId, kind: newtype, of: Uuid}
entities:
  - name: demo.seats.Plan
    identity: {name: plan_id, type: demo.seats.PlanId}
    fields:
      - {name: eligible, type: Boolean}
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
  - name: demo.seats.Seat
    identity: {name: member_id, type: demo.seats.MemberId}
    fields:
      - {name: plan_id, type: demo.seats.PlanId}
    lifecycle: {initial: Issued, states: [Issued], terminal: [Issued]}
errors:
  - {name: demo.seats.NoSuchPlan, fields: []}
  - {name: demo.seats.SeatHeld, fields: []}
events:
  - name: demo.seats.PlanDefined
    fields: [{name: plan_id, type: demo.seats.PlanId}]
  - name: demo.seats.MemberJoined
    fields:
      - {name: member_id, type: demo.seats.MemberId}
      - {name: plan_id, type: demo.seats.PlanId}
  - name: demo.seats.SeatSkipped
    fields:
      - {name: member_id, type: demo.seats.MemberId}
  - name: demo.seats.SeatIssued
    fields:
      - {name: member_id, type: demo.seats.MemberId}
      - {name: plan_id, type: demo.seats.PlanId}
commands:
  - name: demo.seats.DefinePlan
    input: [{name: eligible, type: Boolean}]
    outcomes:
      - name: defined
        creates: demo.seats.Plan
        instance: plan_id
        sets: {eligible: input.eligible}
        emits: [demo.seats.PlanDefined]
        payload: {demo.seats.PlanDefined: {plan_id: {generated: true}}}
  - name: demo.seats.Join
    input:
      - {name: member_id, type: demo.seats.MemberId}
      - {name: plan_id, type: demo.seats.PlanId}
    outcomes:
      - name: joined
        emits: [demo.seats.MemberJoined]
        payload:
          demo.seats.MemberJoined: {member_id: input.member_id, plan_id: input.plan_id}
  - name: demo.seats.IssueSeat
    input:
      - {name: member_id, type: demo.seats.MemberId}
      - {name: plan_id, type: demo.seats.PlanId}
    outcomes:
      - {name: held, existing_instance: true, error: demo.seats.SeatHeld}
      - name: no-such-plan
        when_related: {via: input.plan_id, exists: false}
        error: demo.seats.NoSuchPlan
      - name: ineligible
        when_related: {via: input.plan_id, predicate: eligible == false}
        emits: [demo.seats.SeatSkipped]
        payload: {demo.seats.SeatSkipped: {member_id: input.member_id}}
      - name: issued
        creates: demo.seats.Seat
        instance: member_id
        sets: {plan_id: input.plan_id}
        emits: [demo.seats.SeatIssued]
        payload:
          demo.seats.SeatIssued: {member_id: input.member_id, plan_id: input.plan_id}
bindings:
  - id: seat-on-join
    when: {event: demo.seats.MemberJoined}
    invoke: {command: demo.seats.IssueSeat}
    mapping: {member_id: event.member_id, plan_id: event.plan_id}
    delivery: at_least_once
    on_failure: drop
views:
  - name: demo.seats.Plans
    source: demo.seats.Plan
    consistency: read_your_writes
    fields:
      - {name: plan_id, type: demo.seats.PlanId}
      - {name: eligible, type: Boolean}
";

/// The guard that reads the related row alone.
const ROW_ONLY: &str = "predicate: eligible == false}";

fn model(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).expect("the fixture parses");
    let spec = Specification::assemble([(Source::new("seats.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("the fixture validates:\n{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|d| panic!("the fixture resolves:\n{d}"))
}

fn rewrite(from: &str, to: &str) -> String {
    assert_eq!(FIXTURE.matches(from).count(), 1, "`{from}` occurs once");
    FIXTURE.replacen(from, to, 1)
}

fn refusals(synthesis: &Synthesis) -> Vec<String> {
    synthesis.refusals.iter().map(ToString::to_string).collect()
}

fn steps_of(synthesis: &Synthesis, id: &str) -> Vec<Value> {
    let document: Value =
        serde_json::from_str(&synthesis.suite.to_canonical_json().expect("serialises"))
            .expect("JSON");
    document["scenarios"][id]["steps"]
        .as_array()
        .unwrap_or_else(|| panic!("`{id}` is synthesized: {:#?}", refusals(synthesis)))
        .clone()
}

fn refused_as(synthesis: &Synthesis, id: &str) -> Vec<String> {
    refusals(synthesis)
        .into_iter()
        .filter(|refusal| refusal.contains(&format!("`{id}`")))
        .collect()
}

#[test]
fn a_binding_into_a_related_row_branch_gets_its_flow_and_delivery() {
    let synthesis = synthesize(&model(FIXTURE));
    for aspect in ["flow", "mapping", "delivery"] {
        let id = format!("seat-on-join/binding/{aspect}");
        assert_eq!(refused_as(&synthesis, &id), Vec::<String>::new(), "{id}");
        let steps = steps_of(&synthesis, &id);
        // The plan is created and captured before the trigger, and `Join` is sent naming it.
        let captured: Vec<&str> = steps
            .iter()
            .filter(|step| {
                step["step"] == "capture_instance" && step["entity"] == "demo.seats.Plan"
            })
            .filter_map(|step| step["instance"].as_str())
            .collect();
        let join = steps
            .iter()
            .find(|step| step["command"] == "demo.seats.Join")
            .expect("the trigger");
        let named = join["input"]["plan_id"]["instance"]
            .as_str()
            .unwrap_or_else(|| panic!("{id}: `Join` names an arranged plan: {join}"));
        assert!(
            captured.contains(&named),
            "{id}: {named} is arranged: {captured:?}"
        );
        let defined = steps
            .iter()
            .position(|step| step["command"] == "demo.seats.DefinePlan")
            .expect("a plan is defined");
        let sent = steps
            .iter()
            .position(|step| step["command"] == "demo.seats.Join")
            .expect("the trigger");
        assert!(defined < sent, "{id}: the plan exists before the trigger");
    }
    // The accepting branch is the default: the arranged plan is eligible, and the flow waits for
    // the seat, never for the skip.
    let flow = steps_of(&synthesis, "seat-on-join/binding/flow");
    let eventually: Vec<&str> = flow
        .iter()
        .filter(|step| step["step"] == "eventually_event")
        .filter_map(|step| step["event"].as_str())
        .collect();
    assert_eq!(eventually, ["demo.seats.SeatIssued"]);
}

#[test]
fn the_related_row_suite_passes_against_the_interpreter() {
    let ir = model(FIXTURE);
    let suite = synthesize(&ir).suite;
    let admitted = AdmittedSuite::from_suite(&suite).expect("the suite is admitted");
    let report = Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &Interpreted::for_model(ir))
        .into_report();
    let not_passed: Vec<String> = report
        .scenarios
        .iter()
        .filter(|scenario| scenario.status != Status::Passed)
        .map(|scenario| format!("{} {:?}", scenario.scenario, scenario.status))
        .collect();
    assert_eq!(not_passed, Vec::<String>::new());
    for aspect in ["flow", "mapping", "delivery"] {
        let id = format!("seat-on-join/binding/{aspect}");
        assert!(
            report
                .scenarios
                .iter()
                .any(|scenario| scenario.scenario.to_string() == id),
            "`{id}` ran"
        );
    }
}

#[test]
fn a_dropped_binding_fails_the_related_row_flow() {
    let suite = synthesize(&model(FIXTURE)).suite;
    let without = rewrite(
        "  - id: seat-on-join
    when: {event: demo.seats.MemberJoined}
    invoke: {command: demo.seats.IssueSeat}
    mapping: {member_id: event.member_id, plan_id: event.plan_id}
    delivery: at_least_once
    on_failure: drop
",
        "",
    );
    let admitted = AdmittedSuite::from_suite(&suite).expect("the suite is admitted");
    let report = Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &Interpreted::for_model(model(&without)))
        .into_report();
    let flow = report
        .scenarios
        .iter()
        .find(|scenario| scenario.scenario.to_string() == "seat-on-join/binding/flow")
        .expect("the flow ran");
    assert_ne!(flow.status, Status::Passed);
}

#[test]
fn a_guard_that_reads_the_invoked_input_keeps_its_refusal() {
    // The plan names the one member it skips: which branch is taken depends on the member the
    // event carries, which the upstream implementation chose.
    let text = rewrite(ROW_ONLY, "predicate: skipped == input.member_id}").replacen(
        "      - {name: eligible, type: Boolean}\n",
        "      - {name: eligible, type: Boolean}\n      - {name: skipped, type: demo.seats.MemberId}\n",
        1,
    );
    let synthesis = synthesize(&model(&text));
    let flow = refused_as(&synthesis, "seat-on-join/binding/flow");
    assert!(
        flow.iter()
            .any(|refusal| refusal.contains("whose branch is decided by an input the event fills")),
        "{flow:#?}"
    );
}

#[test]
fn a_row_named_by_a_minted_identity_keeps_its_refusal() {
    // `Join` publishes a plan identity the implementation mints: no row can be arranged for it.
    let text = rewrite(
        "demo.seats.MemberJoined: {member_id: input.member_id, plan_id: input.plan_id}",
        "demo.seats.MemberJoined: {member_id: input.member_id, plan_id: {generated: true}}",
    );
    let synthesis = synthesize(&model(&text));
    let flow = refused_as(&synthesis, "seat-on-join/binding/flow");
    assert!(
        flow.iter()
            .any(|refusal| refusal.contains("whose branch is decided by an input the event fills")),
        "{flow:#?}"
    );
}
