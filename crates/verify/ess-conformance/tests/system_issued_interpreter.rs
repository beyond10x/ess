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
        "format: ess/27
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

// ---- an Optional input the system may leave out ---------------------------------------------

const FINISH: &str = "jobs.job.Finish";
const FINISHED: &str = "jobs.job.Finished";
const ARCHIVE: &str = "jobs.job.Archive";
const MOVED: &str = "jobs.job.Finish/outcome/moved";

/// A job a caller opens and an agent of the system finishes, with or without a note: `Finish`
/// copies its `Optional` input into the row by `sets:`, and the system may leave it out.
fn job_model() -> String {
    "format: ess/27
system: jobs
version: v1
domain: jobs.job
types:
  - {name: jobs.job.JobId, kind: newtype, of: Uuid}
  - name: jobs.job.Note
    kind: struct
    fields:
      - {name: text, type: String}
entities:
  - name: jobs.job.Job
    identity: {name: job_id, type: jobs.job.JobId}
    fields:
      - {name: note, type: Optional<jobs.job.Note>}
    lifecycle:
      initial: Open
      states: [Open, Done]
      terminal: [Done]
      transitions:
        - {name: finish, from: [Open], to: Done}
errors:
  - {name: jobs.job.JobStateConflict, summary: The job is not open., fields: []}
events:
  - name: jobs.job.Opened
    fields:
      - {name: job_id, type: jobs.job.JobId}
  - name: jobs.job.Finished
    fields:
      - {name: job_id, type: jobs.job.JobId}
actors:
  - name: jobs.job.Caller
    may: [jobs.job.Open]
  - name: jobs.job.Agent
    system: true
    may: [jobs.job.Finish]
views:
  - name: jobs.job.Jobs
    source: jobs.job.Job
    consistency: read_your_writes
    fields:
      - {name: job_id, type: jobs.job.JobId}
      - {name: note, type: Optional<jobs.job.Note>}
      - {name: state, type: jobs.job.Job.State}
components:
  - component: job-service
    owns: {domains: [jobs.job]}
    accepts:
      commands: [jobs.job.Open, jobs.job.Finish]
    publishes:
      events: [jobs.job.Opened, jobs.job.Finished]
    reached_by: network
commands:
  - name: jobs.job.Open
    input: []
    outcomes:
      - name: opened
        creates: jobs.job.Job
        instance: job_id
        emits: [jobs.job.Opened]
        payload:
          jobs.job.Opened: {job_id: {generated: true}}
  - name: jobs.job.Finish
    await_within: PT5S
    input:
      - {name: job_id, type: jobs.job.JobId}
      - {name: note, type: Optional<jobs.job.Note>}
    outcomes:
      - name: moved
        moves: jobs.job.Job.finish
        instance: job_id
        emits: [jobs.job.Finished]
        sets: {note: input.note}
        payload:
          jobs.job.Finished: {job_id: input.job_id}
      - {name: wrong-state, wrong_state: true, error: jobs.job.JobStateConflict}
"
    .to_owned()
}

/// The job model with the note relayed onwards as well as stored: `Finished` carries it, and a
/// binding hands it from `Finished` to `Archive`.
fn relaying_model() -> String {
    let model = job_model();
    let replace = |model: String, from: &str, to: &str| {
        assert!(model.contains(from), "the model holds `{from}`");
        model.replace(from, to)
    };
    let model = replace(
        model,
        "  - name: jobs.job.Finished
    fields:
      - {name: job_id, type: jobs.job.JobId}
",
        "  - name: jobs.job.Finished
    fields:
      - {name: job_id, type: jobs.job.JobId}
      - {name: note, type: Optional<jobs.job.Note>}
  - name: jobs.job.Archived
    fields:
      - {name: job_id, type: jobs.job.JobId}
",
    );
    let model = replace(
        model,
        "jobs.job.Finished: {job_id: input.job_id}",
        "jobs.job.Finished: {job_id: input.job_id, note: input.note}",
    );
    let model = replace(
        model,
        "commands: [jobs.job.Open, jobs.job.Finish]",
        "commands: [jobs.job.Open, jobs.job.Finish, jobs.job.Archive]",
    );
    let model = replace(
        model,
        "events: [jobs.job.Opened, jobs.job.Finished]",
        "events: [jobs.job.Opened, jobs.job.Finished, jobs.job.Archived]",
    );
    model
        + "  - name: jobs.job.Archive
    input:
      - {name: job_id, type: jobs.job.JobId}
      - {name: note, type: Optional<jobs.job.Note>}
    outcomes:
      - name: archived
        emits: [jobs.job.Archived]
        payload:
          jobs.job.Archived: {job_id: input.job_id}
bindings:
  - id: archive-on-finished
    when: {event: jobs.job.Finished}
    invoke: {command: jobs.job.Archive}
    mapping: {job_id: event.job_id, note: event.note}
    delivery: at_least_once
    on_failure: retry
"
}

fn job_ir() -> EssIr {
    ir_of(&job_model())
}

fn relaying_ir() -> EssIr {
    ir_of(&relaying_model())
}

/// The synthesized suite's results on `target`, by scenario.
fn synthesized_on<T: ConformanceTarget>(target: &T) -> BTreeMap<String, ScenarioResult> {
    run(&suite_for(&job_ir()), target)
}

/// The suite synthesis writes for `ir`, with [`ARCHIVING`] reading the awaited note.
fn suite_for(ir: &EssIr) -> AdmittedSuite {
    suite_keeping(ir, |_| true)
}

/// The scenario that awaits `Finish` and expects the binding to invoke `Archive`.
const ARCHIVING: &str = "archive-on-finished/binding/mapping";

/// The scenarios of the suite synthesis writes for `ir` that `keep` names.
///
/// Synthesis writes the note [`ARCHIVING`] expects in the invocation as the one `Finished`
/// published (`kind: observed`). It is rewritten to the one the awaited answer carried
/// (`kind: awaited`), what synthesis writes where an invocation relays a value it invented for
/// the awaited command, so the invocation half of the rule runs on a suite the runners admit.
fn suite_keeping(ir: &EssIr, keep: impl Fn(&str) -> bool) -> AdmittedSuite {
    let mut suite = ess_conformance::synthesize::synthesize(ir).suite;
    assert!(
        suite.scenario(&MOVED.parse().unwrap()).is_some(),
        "`{MOVED}` is synthesized"
    );
    let mut rewritten = 0;
    if let Some(scenario) = suite.scenarios.get_mut(&ARCHIVING.parse().unwrap()) {
        for step in &mut scenario.steps {
            if let ScenarioStep::ExpectInvocation { input, .. } = step {
                for value in input.values_mut() {
                    if matches!(value, ScenarioValue::Observed { event, field }
                        if event.to_string() == FINISHED && field == "note")
                    {
                        *value = ScenarioValue::Awaited {
                            field: "note".to_owned(),
                        };
                        rewritten += 1;
                    }
                }
            }
        }
        assert_eq!(rewritten, 1, "{:#?}", scenario.steps);
    }
    suite.scenarios.retain(|id, _| keep(&id.to_string()));
    AdmittedSuite::from_suite(&suite)
        .unwrap_or_else(|error| panic!("the synthesized suite is admitted: {error}"))
}

#[test]
fn an_optional_awaited_input_left_out_is_not_compared() {
    // The interpreter issues `Finish` with `note` absent first, which reaches `moved`.
    let results = synthesized_on(&Interpreted::for_model(job_ir()));
    let moved = &results[MOVED];
    assert_eq!(moved.status, Status::Passed, "{moved:#?}");
    for (id, result) in &results {
        assert_eq!(result.status, Status::Passed, "{id}: {result:#?}");
    }
}

/// How [`Misreport`] departs from the interpreted model around `Finish`'s `note`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Lie {
    /// Answers that the system issued `Finish` with a `note` it never stored or published.
    IssuedNote,
    /// Leaves `note` out, and publishes `Finished` carrying one anyway.
    PublishedNote,
    /// Leaves `note` out, and the binding invokes `Archive` with one anyway.
    InvokedNote,
    /// Answers the left-out `note` as an explicit `null`, and otherwise behaves as the model.
    NullNote,
    /// Tells no lie: the interpreted model itself.
    Truth,
}

/// The interpreted model, misreporting by `lie`. Records whether the lie was ever told, so a case
/// cannot pass because the lie never applied.
struct Misreport {
    inner: Interpreted,
    lie: Lie,
    told: std::rc::Rc<std::cell::Cell<bool>>,
}

impl Misreport {
    fn new(ir: EssIr, lie: Lie) -> Self {
        Self {
            inner: Interpreted::for_model(ir),
            lie,
            told: std::rc::Rc::default(),
        }
    }
}

fn note(text: &str) -> Node {
    Node::Map(BTreeMap::from([(
        "text".to_owned(),
        Node::Text(text.to_owned()),
    )]))
}

impl ConformanceTarget for Misreport {
    fn establish_entity(
        &self,
        request: ess_conformance::target::EntitySetupRequest,
    ) -> Result<(), TargetError> {
        self.inner.establish_entity(request)
    }
    fn identity(&self) -> Result<ess_conformance::target::ImplementationIdentity, TargetError> {
        self.inner.identity()
    }
    fn begin_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.inner.begin_scenario(scenario)
    }
    fn deliver_event(
        &self,
        request: ess_conformance::target::EventDeliveryRequest,
    ) -> Result<(), TargetError> {
        self.inner.deliver_event(request)
    }
    fn mark_instant(
        &self,
        request: ess_conformance::target::InstantMark,
    ) -> Result<(), TargetError> {
        self.inner.mark_instant(request)
    }
    fn observe_elapsed(
        &self,
        request: ess_conformance::target::ElapsedObservationRequest,
    ) -> Result<ess_conformance::target::ElapsedObservation, TargetError> {
        self.inner.observe_elapsed(request)
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<ess_conformance::target::SemanticCommandResult, TargetError> {
        self.inner.execute_command(request)
    }
    fn execute_command_recorded(
        &self,
        request: SemanticCommandRequest,
    ) -> ess_conformance::target::RecordedCommandCompletion {
        self.inner.execute_command_recorded(request)
    }
    fn execute_command_without_input(
        &self,
        request: ess_conformance::target::AbsentInputRequest,
    ) -> Result<ess_conformance::target::SemanticCommandResult, TargetError> {
        self.inner.execute_command_without_input(request)
    }
    fn query_view(
        &self,
        request: ess_conformance::target::SemanticViewRequest,
    ) -> Result<ess_conformance::target::SemanticViewResult, TargetError> {
        self.inner.query_view(request)
    }
    fn query_view_as(
        &self,
        request: ess_conformance::target::SemanticViewRequest,
        reader: &ess_conformance::scenario::ActorRef,
    ) -> Result<ess_conformance::target::SemanticViewResult, TargetError> {
        self.inner.query_view_as(request, reader)
    }
    fn query_view_anonymous(
        &self,
        request: ess_conformance::target::SemanticViewRequest,
    ) -> Result<ess_conformance::target::SemanticViewResult, TargetError> {
        self.inner.query_view_anonymous(request)
    }
    fn observe_events(
        &self,
        request: ess_conformance::target::EventObservationRequest,
    ) -> Result<Vec<ess_conformance::target::ObservedEvent>, TargetError> {
        self.inner.observe_events(request)
    }
    fn configure_external_outcome(
        &self,
        request: ExternalOutcomeControl,
    ) -> Result<(), TargetError> {
        self.inner.configure_external_outcome(request)
    }
    fn configure_external_outcome_repeatedly(
        &self,
        request: ExternalOutcomeControl,
        times: std::num::NonZeroU32,
    ) -> Result<(), TargetError> {
        self.inner
            .configure_external_outcome_repeatedly(request, times)
    }
    fn redeliver_event(
        &self,
        request: ess_conformance::target::RedeliveryRequest,
    ) -> Result<(), TargetError> {
        self.inner.redeliver_event(request)
    }
    fn observe_invocations(
        &self,
        request: ess_conformance::target::InvocationObservationRequest,
    ) -> Result<Vec<ess_conformance::target::ObservedInvocation>, TargetError> {
        let mut invocations = self.inner.observe_invocations(request)?;
        if self.lie == Lie::InvokedNote {
            for invocation in &mut invocations {
                if invocation.command.to_string() == ARCHIVE
                    && !invocation.input.contains_key("note")
                {
                    invocation
                        .input
                        .insert("note".to_owned(), note("never issued"));
                    self.told.set(true);
                }
            }
        }
        Ok(invocations)
    }
    fn await_outcome(&self, request: AwaitRequest) -> Result<AwaitAnswer, TargetError> {
        let is_finish = request.command.to_string() == FINISH;
        let mut answer = self.inner.await_outcome(request)?;
        let AwaitAnswer::Reached { input, result, .. } = &mut answer else {
            return Ok(answer);
        };
        if !is_finish || input.contains_key("note") {
            return Ok(answer);
        }
        match self.lie {
            Lie::IssuedNote => {
                input.insert("note".to_owned(), note("issued, never stored"));
                self.told.set(true);
            }
            Lie::PublishedNote => {
                for event in &mut result.direct_events {
                    if event.event.to_string() == FINISHED {
                        event
                            .payload
                            .insert("note".to_owned(), note("never issued"));
                        self.told.set(true);
                    }
                }
            }
            Lie::NullNote => {
                input.insert("note".to_owned(), Node::Null);
                self.told.set(true);
            }
            Lie::InvokedNote | Lie::Truth => {}
        }
        Ok(answer)
    }
    fn end_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.inner.end_scenario(scenario)
    }
}

#[test]
fn an_optional_awaited_input_issued_with_a_wrong_value_fails() {
    let target = Misreport::new(job_ir(), Lie::IssuedNote);
    let results = synthesized_on(&target);
    assert!(target.told.get(), "the lie was never told");
    let moved = &results[MOVED];
    assert_eq!(moved.status, Status::Failed, "{moved:#?}");
    assert!(
        moved
            .checks
            .iter()
            .all(|check| check.status != Status::Error),
        "an issued `note` is compared, not a suite defect: {moved:#?}"
    );
}

/// The relaying model's `moved` reads the awaited `note` in `Finished`, and [`ARCHIVING`] reads
/// it in the invocation of `Archive`, so the event and the invocation halves of the rule run.
#[test]
fn the_relaying_model_reads_the_awaited_note_in_an_event_and_an_invocation() {
    let suite = suite_for(&relaying_ir());
    let reads = |values: &BTreeMap<String, ScenarioValue>| {
        values
            .values()
            .any(|value| matches!(value, ScenarioValue::Awaited { field } if field == "note"))
    };
    let steps = |id: &str| &suite.suite().scenario(&id.parse().unwrap()).unwrap().steps;
    assert!(
        steps(MOVED).iter().any(|step| matches!(
            step,
            ScenarioStep::ExpectEventValues { payload, .. } if reads(payload)
        )),
        "{:#?}",
        steps(MOVED)
    );
    assert!(
        steps(ARCHIVING).iter().any(|step| matches!(
            step,
            ScenarioStep::ExpectInvocation { input, .. } if reads(input)
        )),
        "{:#?}",
        steps(ARCHIVING)
    );
}

/// The interpreter leaves `note` out, so it stores, publishes and relays none: every scenario
/// passes. Red against a runner that resolves the left-out value in the event or the invocation.
#[test]
fn an_optional_awaited_input_left_out_relays_nothing_and_passes() {
    let results = run(
        &suite_for(&relaying_ir()),
        &Interpreted::for_model(relaying_ir()),
    );
    for (id, result) in &results {
        assert_eq!(result.status, Status::Passed, "{id}: {result:#?}");
    }
}

/// Each lie told on the relaying model, and the scenario it fails (none: every scenario passes).
const RELAYING_LIES: [(Lie, Option<&str>); 5] = [
    (Lie::Truth, None),
    (Lie::IssuedNote, Some(MOVED)),
    (Lie::PublishedNote, Some(MOVED)),
    (Lie::InvokedNote, Some(ARCHIVING)),
    (Lie::NullNote, None),
];

/// A target that publishes or relays a note the system never issued, or issued a note it never
/// stored, fails; one that spells the left-out note `null` behaves as the model and passes.
#[test]
fn an_optional_awaited_input_left_out_is_absent_or_null_wherever_it_is_relayed() {
    for (lie, fails) in RELAYING_LIES {
        let target = Misreport::new(relaying_ir(), lie);
        let results = run(&suite_for(&relaying_ir()), &target);
        assert!(
            lie == Lie::Truth || target.told.get(),
            "{lie:?} was never told"
        );
        for (id, result) in &results {
            assert!(
                result
                    .checks
                    .iter()
                    .all(|check| check.status != Status::Error),
                "{lie:?} is compared, not a suite defect: {id}: {result:#?}"
            );
        }
        match fails {
            Some(id) => assert_eq!(results[id].status, Status::Failed, "{lie:?}: {id}"),
            None => {
                for (id, result) in &results {
                    assert_eq!(result.status, Status::Passed, "{lie:?}: {id}: {result:#?}");
                }
            }
        }
    }
}

// ---- the same rule in the generated Go and TypeScript runners ---------------------------------
//
// Each generated runtime replays the transcript the target wrote while the Rust reference runner
// asked it (`support_go::Recorder`), so a verdict that differs is a difference in how the runner
// read the suite.

mod support_go;
mod support_ts;

fn written(status: Status) -> String {
    match status {
        Status::Passed => "passed",
        Status::Failed => "failed",
        Status::Error => "error",
        Status::Unsupported => "unsupported",
    }
    .to_owned()
}

/// The reference verdicts against `target`, and the transcript of what it answered.
fn recorded<T: ConformanceTarget>(
    suite: &AdmittedSuite,
    target: T,
) -> (BTreeMap<String, String>, support_go::Recorder<T>) {
    let recorder = support_go::Recorder::new(target);
    let verdicts = run(suite, &recorder)
        .into_iter()
        .map(|(id, result)| (id, written(result.status)))
        .collect();
    (verdicts, recorder)
}

/// The synthesized scenarios of `ir` that await `moved` and read what it wrote, alone: the rest
/// of the suite reads no awaited input.
fn moved_suite(ir: &EssIr) -> AdmittedSuite {
    let suite = suite_keeping(ir, |id| id.ends_with("/moved") || id == ARCHIVING);
    let kept: Vec<String> = suite
        .suite()
        .scenarios
        .keys()
        .map(ToString::to_string)
        .collect();
    let archives = ir.bindings().values().next().is_some();
    assert_eq!(
        kept.len(),
        if archives { 3 } else { 2 },
        "`moved`, the transition it takes and the binding it triggers: {kept:?}"
    );
    suite
}

/// Each case a generated runner replays: the model, how the target lies, and the scenario it
/// fails, if any. Each half of the rule has a case that turns red without it: the row
/// (`IssuedNote` on the job model), the event (`PublishedNote`), the invocation (`InvokedNote`),
/// the left-out note read in each (`Truth` on the relaying model) and the `null` spelling
/// (`NullNote`).
fn replay_cases() -> Vec<(&'static str, EssIr, Lie, Option<&'static str>)> {
    let mut cases = vec![
        ("job", job_ir(), Lie::Truth, None),
        ("job", job_ir(), Lie::IssuedNote, Some(MOVED)),
    ];
    for (lie, fails) in RELAYING_LIES {
        cases.push(("relaying", relaying_ir(), lie, fails));
    }
    cases
}

/// The reference verdicts against `target` on `suite`, checked against the scenario the case
/// fails, and the transcript of what it answered.
fn reference(
    suite: &AdmittedSuite,
    target: Misreport,
    fails: Option<&str>,
) -> (BTreeMap<String, String>, support_go::Recorder<Misreport>) {
    let lie = target.lie;
    let told = std::rc::Rc::clone(&target.told);
    let (verdicts, recorder) = recorded(suite, target);
    assert!(lie == Lie::Truth || told.get(), "{lie:?} was never told");
    match fails {
        Some(id) => assert_eq!(verdicts[id], "failed", "{lie:?}: {verdicts:?}"),
        None => assert!(
            verdicts.values().all(|status| status == "passed"),
            "{lie:?}: {verdicts:?}"
        ),
    }
    (verdicts, recorder)
}

/// The interpreted model, which leaves `note` out, and the target spelling it `null`, pass every
/// scenario in the Go runner; each target issuing, publishing or relaying a note it should not
/// fails there, as in the reference runner.
#[test]
fn an_optional_awaited_input_left_out_or_issued_wrong_reads_alike_in_the_go_runner() {
    let mut packages = BTreeMap::new();
    for (model, ir, lie, fails) in replay_cases() {
        let suite = moved_suite(&ir);
        let directory = packages.entry(model).or_insert_with(|| {
            support_go::package(
                &format!("optional-await-{model}"),
                suite.suite(),
                &[support_go::TRANSCRIPT_TARGET],
            )
        });
        let (verdicts, recorder) = reference(&suite, Misreport::new(ir, lie), fails);
        assert_eq!(
            go_replay(&*directory, &recorder),
            verdicts,
            "{model} model, {lie:?}"
        );
    }
}

/// The same cases in the TypeScript runner.
#[test]
fn an_optional_awaited_input_left_out_or_issued_wrong_reads_alike_in_the_typescript_runner() {
    let mut packages = BTreeMap::new();
    for (model, ir, lie, fails) in replay_cases() {
        let suite = moved_suite(&ir);
        let package = packages.entry(model).or_insert_with(|| {
            support_ts::Package::new(&format!("optional-await-{model}"), suite.suite())
        });
        let (verdicts, recorder) = reference(&suite, Misreport::new(ir, lie), fails);
        let replayed = package.replay(&suite, &recorder.transcript(), false);
        assert_ne!(replayed.exit, Some(2), "{}", replayed.log);
        assert!(
            replayed.divergences.is_empty(),
            "the TypeScript runtime sent requests the reference runner did not:\n{}\n{}",
            replayed.divergences.join("\n"),
            replayed.log
        );
        assert_eq!(replayed.outcomes, verdicts, "{model} model, {lie:?}");
    }
}

/// The Go runtime's verdicts replaying what `recorder` wrote.
fn go_replay<T>(
    directory: &std::path::Path,
    recorder: &support_go::Recorder<T>,
) -> BTreeMap<String, String> {
    let replayed = support_go::replay(directory, recorder, &[]);
    assert!(
        replayed.divergences.is_empty(),
        "the Go runtime sent requests the reference runner did not:\n{}\n{}",
        replayed.divergences.join("\n"),
        replayed.go.log
    );
    assert!(
        !replayed.go.outcomes.is_empty(),
        "the Go run published no verdicts:\n{}",
        replayed.go.log
    );
    replayed.go.outcomes
}
