//! The interpreted target answering an await of a command the system issues itself (beyond10x/ess#542,
//! `docs/design/system-issued-commands.md`, *Every lane*, the interpreter row).
//!
//! The interpreter issues a system-issued command only when a scenario awaits it, so the hold is
//! trivial. Its identity input is the awaited instance; every other input is decided by candidates
//! in a fixed order — the identities of held rows a `when_related:` guard reads, in canonical
//! order; otherwise a value minted from the scenario's counter; for an `Optional` input, absent
//! first. The first candidate that reaches an accepting branch, or a forced external one, is the
//! answer. No candidate reaching means nothing was issued; candidates reaching different accepting
//! branches, or a compared input the model does not determine, mean unsupported.
//!
//! The await steps here are written by hand; `system_issued_synthesis.rs` runs synthesized ones.

use std::collections::{BTreeMap, BTreeSet};

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::{CheckCode, ScenarioResult, Status};
use ess_conformance::scenario::{
    CommandRef, ConformanceScenario, EntityRef, EventRef, Holds, InstanceName, LeafShape,
    OutcomeRef, PayloadShape, ScenarioId, ScenarioPurpose, ScenarioStep, ScenarioValue,
    SuiteFormat, SuiteProvenance,
};
use ess_conformance::target::{
    AwaitAnswer, AwaitRequest, ConformanceTarget, Deadline, ExternalOutcomeControl,
    ScenarioContext, SemanticCommandRequest, TargetError,
};
use ess_conformance::{AdmittedSuite, ConformanceSuite, Elapsed, Runner};
use ess_domain::binding::periodic::Period;
use ess_domain::types::Primitive;
use ess_domain::{spec::RawSpecFile, system::Source, QualifiedName, Specification};
use ess_primitives::evidence::SpecDigest;
use ess_primitives::ids::CorrelationId;
use ess_primitives::node::Node;
use ess_primitives::time::Timestamp;

const DIGEST: &str = "abababababababababababababababababababababababababababababababab";
const ADD_WORKER: &str = "queueing.entry.AddWorker";
const ENQUEUE: &str = "queueing.entry.Enqueue";
const ASSIGN: &str = "queueing.entry.Assign";
const ENQUEUED: &str = "queueing.entry.Enqueued";
const ASSIGNED: &str = "queueing.entry.Assigned";
const WORKER_ADDED: &str = "queueing.entry.WorkerAdded";

/// An entry waits; the system's scheduler assigns it to a free worker of its own choosing. A worker
/// nobody holds, or a busy one, is refused by the input; an assigned entry by its state; and the
/// dialler upstream may refuse an assignment, which only a scenario forcing it reaches.
fn model(await_within: &str) -> String {
    format!(
        "format: ess/25
system: queueing
version: v1
domain: queueing.entry
types:
  - {{name: queueing.entry.EntryId, kind: newtype, of: Uuid}}
  - {{name: queueing.entry.WorkerId, kind: newtype, of: Uuid}}
entities:
  - name: queueing.entry.Worker
    identity: {{name: worker_id, type: queueing.entry.WorkerId}}
    fields:
      - {{name: free, type: Boolean}}
    lifecycle: {{initial: Ready, states: [Ready], terminal: [Ready]}}
  - name: queueing.entry.Entry
    identity: {{name: entry_id, type: queueing.entry.EntryId}}
    fields:
      - {{name: worker, type: Optional<queueing.entry.WorkerId>}}
    lifecycle:
      initial: Waiting
      states: [Waiting, Assigned]
      terminal: [Assigned]
      transitions:
        - {{name: assign, from: [Waiting], to: Assigned}}
errors:
  - {{name: queueing.entry.EntryStateConflict, summary: The entry is not waiting., fields: []}}
  - {{name: queueing.entry.NoWorker, summary: No worker carries the identity., fields: []}}
  - {{name: queueing.entry.WorkerBusy, summary: The worker is busy., fields: []}}
  - {{name: queueing.entry.DiallerRefused, summary: The dialler refused the call., fields: []}}
events:
  - name: queueing.entry.WorkerAdded
    fields:
      - {{name: worker_id, type: queueing.entry.WorkerId}}
  - name: queueing.entry.Enqueued
    fields:
      - {{name: entry_id, type: queueing.entry.EntryId}}
  - name: queueing.entry.Assigned
    fields:
      - {{name: entry_id, type: queueing.entry.EntryId}}
      - {{name: worker, type: queueing.entry.WorkerId}}
actors:
  - name: queueing.entry.Caller
    may: [queueing.entry.AddWorker, queueing.entry.Enqueue]
  - name: queueing.entry.Scheduler
    system: true
    may: [queueing.entry.Assign]
commands:
  - name: queueing.entry.AddWorker
    input:
      - {{name: free, type: Boolean}}
    outcomes:
      - name: added
        creates: queueing.entry.Worker
        instance: worker_id
        emits: [queueing.entry.WorkerAdded]
        payload:
          queueing.entry.WorkerAdded: {{worker_id: {{generated: true}}}}
        sets:
          free: input.free
  - name: queueing.entry.Enqueue
    input: []
    outcomes:
      - name: enqueued
        creates: queueing.entry.Entry
        instance: entry_id
        emits: [queueing.entry.Enqueued]
        payload:
          queueing.entry.Enqueued: {{entry_id: {{generated: true}}}}
  - name: queueing.entry.Assign
{await_within}    input:
      - {{name: entry_id, type: queueing.entry.EntryId}}
      - {{name: worker, type: queueing.entry.WorkerId}}
    outcomes:
      - name: no-worker
        when_related: {{via: input.worker, exists: false}}
        error: queueing.entry.NoWorker
      - name: busy
        when_related: {{via: input.worker, predicate: free == false}}
        error: queueing.entry.WorkerBusy
      - name: dialler-refused
        external: the dialler refuses the call
        error: queueing.entry.DiallerRefused
      - name: assigned
        moves: queueing.entry.Entry.assign
        instance: entry_id
        emits: [queueing.entry.Assigned]
        sets: {{worker: input.worker}}
        payload:
          queueing.entry.Assigned: {{entry_id: input.entry_id, worker: input.worker}}
      - {{name: wrong-state, wrong_state: true, error: queueing.entry.EntryStateConflict}}
"
    )
}

const BOUND: &str = "    await_within: PT5S\n";

fn ir_of(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).expect("the model parses");
    let spec = Specification::assemble([(Source::new("queueing.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("the model is admitted: {errors}"));
    compile(&spec, &SourceMap::new()).expect("the model compiles")
}

fn name(value: &str) -> QualifiedName {
    QualifiedName::new(value).expect("a valid name")
}

fn command(value: &str) -> CommandRef {
    CommandRef::new(name(value))
}

fn event(value: &str) -> EventRef {
    EventRef::new(name(value))
}

fn outcome(command_name: &str, outcome: &str) -> OutcomeRef {
    OutcomeRef::new(
        command(command_name),
        outcome.parse().expect("an outcome name"),
    )
}

fn instance(value: &str) -> ScenarioValue {
    ScenarioValue::instance(InstanceName::new(value).unwrap())
}

fn uuid_leaf() -> LeafShape {
    LeafShape::required(Holds::Primitive {
        kind: Primitive::Uuid,
    })
}

/// The declared input of `Assign`, as the await step carries it.
fn input_shape() -> PayloadShape {
    let mut shape = PayloadShape::new();
    shape.insert("entry_id", uuid_leaf());
    shape.insert("worker", uuid_leaf());
    shape
}

fn period(within: bool) -> Option<Period> {
    within.then(|| Period::parse("PT5S").unwrap())
}

/// Adds a worker as the caller, and captures it as `as_instance`.
fn add_worker(free: bool, as_instance: &str) -> Vec<ScenarioStep> {
    vec![
        ScenarioStep::ExecuteCommand {
            command: command(ADD_WORKER),
            actor: None,
            caller: BTreeMap::new(),
            input: BTreeMap::from([("free".to_owned(), ScenarioValue::literal(Node::Bool(free)))]),
        },
        ScenarioStep::ExpectOutcome {
            outcome: outcome(ADD_WORKER, "added"),
        },
        ScenarioStep::CaptureInstance {
            instance: InstanceName::new(as_instance).unwrap(),
            entity: EntityRef::new(name("queueing.entry.Worker")),
            event: event(WORKER_ADDED),
            field: "worker_id".to_owned(),
        },
    ]
}

fn enqueue() -> Vec<ScenarioStep> {
    vec![
        ScenarioStep::ExecuteCommand {
            command: command(ENQUEUE),
            actor: None,
            caller: BTreeMap::new(),
            input: BTreeMap::new(),
        },
        ScenarioStep::ExpectOutcome {
            outcome: outcome(ENQUEUE, "enqueued"),
        },
        ScenarioStep::CaptureInstance {
            instance: InstanceName::new("entry").unwrap(),
            entity: EntityRef::new(name("queueing.entry.Entry")),
            event: event(ENQUEUED),
            field: "entry_id".to_owned(),
        },
    ]
}

fn await_assign(within: bool, stated: BTreeMap<String, ScenarioValue>) -> ScenarioStep {
    ScenarioStep::AwaitOutcome {
        command: command(ASSIGN),
        instance: instance("entry"),
        identity: "entry_id".to_owned(),
        within: period(within),
        input: stated,
        shape: input_shape(),
    }
}

fn not_issued(within: bool, on: ScenarioValue) -> ScenarioStep {
    ScenarioStep::ExpectNotIssued {
        command: command(ASSIGN),
        instance: on,
        within: period(within),
        unpublished: vec![event(ASSIGNED)],
    }
}

/// A busy worker and a free one; the system assigns the entry to the free one, publishes what it
/// reported, and does not assign the same entry again.
fn assigns_the_free_worker(within: bool, stated: bool) -> Vec<ScenarioStep> {
    let mut steps = add_worker(false, "busy");
    steps.extend(add_worker(true, "free"));
    steps.extend(enqueue());
    let stated = if stated {
        BTreeMap::from([("worker".to_owned(), instance("free"))])
    } else {
        BTreeMap::new()
    };
    steps.extend([
        await_assign(within, stated),
        ScenarioStep::ExpectOutcome {
            outcome: outcome(ASSIGN, "assigned"),
        },
        ScenarioStep::ExpectEventValues {
            event: event(ASSIGNED),
            payload: BTreeMap::from([
                ("entry_id".to_owned(), instance("entry")),
                (
                    "worker".to_owned(),
                    ScenarioValue::Awaited {
                        field: "worker".to_owned(),
                    },
                ),
            ]),
            shape: PayloadShape::new(),
        },
        not_issued(within, instance("entry")),
    ]);
    steps
}

/// Only a busy worker: every candidate the system has is refused by the input, so nothing happens.
fn nothing_without_a_free_worker(within: bool) -> Vec<ScenarioStep> {
    let mut steps = add_worker(false, "busy");
    steps.extend(enqueue());
    steps.push(not_issued(within, instance("entry")));
    steps
}

/// An entry nobody holds: the system has nothing to assign.
fn nothing_for_an_unknown_entry(within: bool) -> Vec<ScenarioStep> {
    let mut steps = add_worker(true, "free");
    steps.push(not_issued(
        within,
        ScenarioValue::literal(Node::Text(
            "00000000-0000-4000-8000-00000000ffff".to_owned(),
        )),
    ));
    steps
}

/// Awaits the system assigning an entry it has already assigned.
fn awaits_a_second_assignment(within: bool) -> Vec<ScenarioStep> {
    let mut steps = add_worker(true, "free");
    steps.extend(enqueue());
    steps.extend([
        await_assign(within, BTreeMap::new()),
        ScenarioStep::ExpectOutcome {
            outcome: outcome(ASSIGN, "assigned"),
        },
        await_assign(within, BTreeMap::new()),
    ]);
    steps
}

const FREE: &str = "queueing.entry/authored/assigns-the-free-worker";
const STATED: &str = "queueing.entry/authored/assigns-the-stated-free-worker";
const NO_FREE: &str = "queueing.entry/authored/nothing-without-a-free-worker";
const UNKNOWN: &str = "queueing.entry/authored/nothing-for-an-unknown-entry";
const SECOND: &str = "queueing.entry/authored/awaits-a-second-assignment";

fn suite(scenarios: Vec<(&str, Vec<ScenarioStep>)>) -> AdmittedSuite {
    let mut suite = ConformanceSuite::new(SuiteProvenance {
        suite_version: SuiteFormat::parse("ess-conformance/4").unwrap(),
        system: "queueing".to_owned(),
        specification_version: "v1".to_owned(),
        spec_digest: SpecDigest::new(DIGEST).unwrap(),
        contract_digest: SpecDigest::new(DIGEST).unwrap(),
        component: None,
        scenario_initial_state: None,
        synthesis_seeds: None,
    });
    for (id, steps) in scenarios {
        suite
            .insert(
                ScenarioId::parse(id).unwrap(),
                ConformanceScenario {
                    purpose: ScenarioPurpose::new("The system assigns a waiting entry").unwrap(),
                    steps,
                    source: BTreeSet::new(),
                    one_time_response: None,
                },
            )
            .unwrap();
    }
    suite.select_fresh_format();
    AdmittedSuite::from_suite(&suite).unwrap_or_else(|error| panic!("{error}"))
}

fn scenarios(within: bool) -> AdmittedSuite {
    suite(vec![
        (FREE, assigns_the_free_worker(within, false)),
        (STATED, assigns_the_free_worker(within, true)),
        (NO_FREE, nothing_without_a_free_worker(within)),
        (UNKNOWN, nothing_for_an_unknown_entry(within)),
        (SECOND, awaits_a_second_assignment(within)),
    ])
}

fn run<T: ConformanceTarget>(
    suite: &AdmittedSuite,
    target: &T,
) -> BTreeMap<String, ScenarioResult> {
    Runner::for_suite(suite.suite())
        .run_admitted(suite, target)
        .into_report()
        .scenarios
        .into_iter()
        .map(|result| (result.scenario.to_string(), result))
        .collect()
}

fn failing(result: &ScenarioResult) -> BTreeSet<CheckCode> {
    result
        .checks
        .iter()
        .filter(|check| check.status == Status::Failed)
        .map(|check| check.code)
        .collect()
}

/// Each scenario's verdict against the interpreted model, bounded or not.
fn assert_verdicts(results: &BTreeMap<String, ScenarioResult>) {
    for id in [FREE, STATED, NO_FREE, UNKNOWN] {
        let result = &results[id];
        assert_eq!(result.status, Status::Passed, "{id}: {result:#?}");
        assert!(
            result
                .checks
                .iter()
                .any(|check| check.code == CheckCode::Await && check.status == Status::Passed),
            "{id} checks the await: {result:#?}"
        );
    }
    // The second await finds the entry assigned: the model's refusal is decided by the record, so
    // the system issues nothing, which fails an await that requires an outcome.
    let second = &results[SECOND];
    assert_eq!(second.status, Status::Failed, "{second:#?}");
    assert_eq!(failing(second), BTreeSet::from([CheckCode::Await]));
}

#[test]
fn the_interpreter_answers_an_await_step() {
    let model = Interpreted::for_model(ir_of(&model(BOUND)));
    assert_verdicts(&run(&scenarios(true), &model));
}

#[test]
fn the_interpreter_answers_an_await_step_without_a_declared_bound() {
    let model = Interpreted::for_model(ir_of(&model("")));
    assert_verdicts(&run(&scenarios(false), &model));
}

// ---- the target, asked directly ---------------------------------------------------------------

fn correlation() -> CorrelationId {
    CorrelationId::new("await-direct").unwrap()
}

fn begin(target: &Interpreted) {
    target
        .begin_scenario(&ScenarioContext::new(
            "queueing.entry/authored/direct".parse().unwrap(),
            correlation(),
        ))
        .unwrap();
}

/// Runs `command` as the caller, and answers the identity its one event published in `field`.
fn created(target: &Interpreted, command_name: &str, input: &[(&str, Node)], field: &str) -> Node {
    let result = target
        .execute_command(SemanticCommandRequest {
            command: command(command_name),
            actor: None,
            caller: None,
            input: input
                .iter()
                .map(|(key, value)| ((*key).to_owned(), value.clone()))
                .collect(),
            correlation: correlation(),
        })
        .unwrap_or_else(|error| panic!("{command_name}: {error:?}"));
    result.direct_events[0].payload[field].clone()
}

fn ask(target: &Interpreted, on: &Node, within: bool, compared: &[&str]) -> AwaitRequest {
    let _ = target;
    AwaitRequest {
        command: command(ASSIGN),
        instance: on.clone(),
        within: within.then(|| Elapsed::seconds(5)),
        deadline: Deadline::at(Timestamp::EPOCH),
        compared: compared.iter().map(|field| (*field).to_owned()).collect(),
        correlation: correlation(),
    }
}

#[test]
fn the_first_free_held_row_is_the_input_the_system_decides() {
    let target = Interpreted::for_model(ir_of(&model(BOUND)));
    begin(&target);
    let busy = created(
        &target,
        ADD_WORKER,
        &[("free", Node::Bool(false))],
        "worker_id",
    );
    let first = created(
        &target,
        ADD_WORKER,
        &[("free", Node::Bool(true))],
        "worker_id",
    );
    let _second = created(
        &target,
        ADD_WORKER,
        &[("free", Node::Bool(true))],
        "worker_id",
    );
    assert!(busy < first, "held rows are tried in canonical order");
    let entry = created(&target, ENQUEUE, &[], "entry_id");

    let answer = target
        .await_outcome(ask(&target, &entry, true, &[]))
        .unwrap();
    let AwaitAnswer::Reached {
        result,
        input,
        elapsed_ms,
    } = answer
    else {
        panic!("the system assigns the entry: {answer:?}");
    };
    assert_eq!(
        result.outcome.as_ref().map(ToString::to_string).as_deref(),
        Some("queueing.entry.Assign/assigned")
    );
    assert_eq!(
        input,
        BTreeMap::from([
            ("entry_id".to_owned(), entry.clone()),
            ("worker".to_owned(), first.clone()),
        ]),
        "the identity input is the instance, and the worker is the first free held one"
    );
    assert_eq!(
        elapsed_ms,
        Some(0),
        "the interpreter takes the step at once"
    );
    assert_eq!(result.direct_events.len(), 1);
    assert_eq!(result.direct_events[0].payload["worker"], first);

    // The entry is assigned now: the record refuses the step, so nothing is issued however often
    // the system is asked, and the window is held for the whole bound.
    for _ in 0..2 {
        let answer = target
            .await_outcome(ask(&target, &entry, true, &[]))
            .unwrap();
        assert_eq!(
            answer,
            AwaitAnswer::NotIssued {
                elapsed_ms: Some(5_000)
            }
        );
    }
}

#[test]
fn nothing_is_issued_on_a_record_that_refuses_and_the_hold_changes_nothing() {
    let target = Interpreted::for_model(ir_of(&model("")));
    begin(&target);
    created(
        &target,
        ADD_WORKER,
        &[("free", Node::Bool(true))],
        "worker_id",
    );
    let unknown = Node::Text("00000000-0000-4000-8000-00000000ffff".to_owned());
    for _ in 0..3 {
        let answer = target
            .await_outcome(ask(&target, &unknown, false, &[]))
            .unwrap();
        assert!(
            matches!(answer, AwaitAnswer::NotIssued { .. }),
            "an unknown instance is a refusal the record decides: {answer:?}"
        );
    }
    // Asking about an unknown entry minted nothing: the next entry takes the identity it would
    // have taken anyway, and is still awaitable.
    let entry = created(&target, ENQUEUE, &[], "entry_id");
    let answer = target
        .await_outcome(ask(&target, &entry, false, &[]))
        .unwrap();
    assert!(matches!(answer, AwaitAnswer::Reached { .. }), "{answer:?}");
}

#[test]
fn a_compared_input_the_model_does_not_determine_is_unsupported() {
    let target = Interpreted::for_model(ir_of(&model(BOUND)));
    begin(&target);
    created(
        &target,
        ADD_WORKER,
        &[("free", Node::Bool(true))],
        "worker_id",
    );
    created(
        &target,
        ADD_WORKER,
        &[("free", Node::Bool(true))],
        "worker_id",
    );
    let entry = created(&target, ENQUEUE, &[], "entry_id");

    // Two free workers: either is the system's to pick, so a scenario comparing the worker asks
    // for a value the model leaves open.
    let error = target
        .await_outcome(ask(&target, &entry, true, &["worker"]))
        .unwrap_err();
    assert!(
        matches!(error, TargetError::Unsupported { .. }),
        "{error:?}"
    );
    // The identity input is the instance, which the scenario names: comparing it is determined.
    let answer = target
        .await_outcome(ask(&target, &entry, true, &["entry_id"]))
        .unwrap();
    assert!(matches!(answer, AwaitAnswer::Reached { .. }), "{answer:?}");
}

#[test]
fn a_forced_external_refusal_is_reached() {
    let target = Interpreted::for_model(ir_of(&model(BOUND)));
    begin(&target);
    created(
        &target,
        ADD_WORKER,
        &[("free", Node::Bool(true))],
        "worker_id",
    );
    let entry = created(&target, ENQUEUE, &[], "entry_id");
    target
        .configure_external_outcome(ExternalOutcomeControl {
            force: outcome(ASSIGN, "dialler-refused"),
            correlation: correlation(),
        })
        .unwrap();
    let answer = target
        .await_outcome(ask(&target, &entry, true, &[]))
        .unwrap();
    let AwaitAnswer::Reached { result, .. } = answer else {
        panic!("the system issued it, and the dialler refused: {answer:?}");
    };
    assert_eq!(
        result.outcome.as_ref().map(ToString::to_string).as_deref(),
        Some("queueing.entry.Assign/dialler-refused")
    );
    assert!(result.error.is_some());
    // The control lapses with the step it forced: the entry is still waiting, and the next await
    // assigns it.
    let answer = target
        .await_outcome(ask(&target, &entry, true, &[]))
        .unwrap();
    let AwaitAnswer::Reached { result, .. } = answer else {
        panic!("{answer:?}");
    };
    assert_eq!(
        result.outcome.as_ref().map(ToString::to_string).as_deref(),
        Some("queueing.entry.Assign/assigned")
    );
}

#[test]
fn a_driven_command_is_not_awaited() {
    let target = Interpreted::for_model(ir_of(&model(BOUND)));
    begin(&target);
    let entry = created(&target, ENQUEUE, &[], "entry_id");
    let mut request = ask(&target, &entry, true, &[]);
    request.command = command(ENQUEUE);
    let error = target.await_outcome(request).unwrap_err();
    assert!(
        matches!(error, TargetError::Unavailable { .. }),
        "a suite awaiting a driven command is a defect, never an answer or a missing capability: \
         {error:?}"
    );
}
