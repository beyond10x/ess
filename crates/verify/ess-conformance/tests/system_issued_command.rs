//! The Rust runner awaiting a command the system issues itself (beyond10x/ess#542,
//! `docs/design/system-issued-commands.md`, *Target interface*, *The bound*, *How each faulty
//! target fails*).
//!
//! A hand-written queue answers the await: it assigns a waiting entry to its one free worker, and
//! takes no step on an assigned one. Against it the suite passes; against each
//! [`AwaitFault`](ess_conformance::faulty::AwaitFault) one named check fails; and a target written
//! before the capability existed keeps every old scenario green and reports each await scenario
//! unsupported, never passed.

use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, BTreeSet};

use ess_conformance::faulty::{AwaitFault, AwaitFaulty};
use ess_conformance::report::{CheckCode, ScenarioResult, Status};
use ess_conformance::scenario::{
    CommandRef, ConformanceScenario, EntityRef, EventRef, Holds, InstanceName, LeafShape,
    OutcomeRef, PayloadShape, ScenarioId, ScenarioPurpose, ScenarioStep, ScenarioValue,
    SuiteFormat, SuiteProvenance,
};
use ess_conformance::target::{
    AwaitAnswer, AwaitRequest, ConformanceTarget, EventObservationRequest, ExternalOutcomeControl,
    ImplementationIdentity, ObservedEvent, RedeliveryRequest, ScenarioContext,
    SemanticCommandRequest, SemanticCommandResult, SemanticViewRequest, SemanticViewResult,
    TargetError,
};
use ess_conformance::{AdmittedSuite, ConformanceSuite, Runner};
use ess_domain::binding::periodic::Period;
use ess_domain::types::Primitive;
use ess_domain::QualifiedName;
use ess_primitives::evidence::SpecDigest;
use ess_primitives::node::Node;

const DIGEST: &str = "abababababababababababababababababababababababababababababababab";
const ENQUEUE: &str = "queueing.entry.Enqueue";
const ASSIGN: &str = "queueing.entry.Assign";
const ENQUEUED: &str = "queueing.entry.Enqueued";
const ASSIGNED: &str = "queueing.entry.Assigned";

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

fn text(value: &str) -> Node {
    Node::Text(value.to_owned())
}

fn entry() -> ScenarioValue {
    ScenarioValue::instance(InstanceName::new("entry").unwrap())
}

fn string_leaf() -> LeafShape {
    LeafShape::required(Holds::Primitive {
        kind: Primitive::String,
    })
}

/// The declared input of `Assign`, as the await step carries it.
fn input_shape() -> PayloadShape {
    let mut shape = PayloadShape::new();
    shape.insert("entry_id", string_leaf());
    shape.insert("worker", string_leaf());
    shape
}

fn arrange() -> Vec<ScenarioStep> {
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

/// Arrange an entry, await the system assigning it, read what it published against what it
/// reported, and require that it does not assign the same entry again.
fn assigns_once(within: Option<u32>, stated: BTreeMap<String, ScenarioValue>) -> Vec<ScenarioStep> {
    let within = within.map(|seconds| Period::parse(&format!("PT{seconds}S")).unwrap());
    let mut steps = arrange();
    steps.extend([
        ScenarioStep::AwaitOutcome {
            command: command(ASSIGN),
            instance: entry(),
            identity: "entry_id".to_owned(),
            within,
            input: stated,
            shape: input_shape(),
        },
        ScenarioStep::ExpectOutcome {
            outcome: outcome(ASSIGN, "assigned"),
        },
        ScenarioStep::ExpectEventValues {
            event: event(ASSIGNED),
            payload: BTreeMap::from([
                ("entry_id".to_owned(), entry()),
                (
                    "worker".to_owned(),
                    ScenarioValue::Awaited {
                        field: "worker".to_owned(),
                    },
                ),
            ]),
            shape: PayloadShape::new(),
        },
        ScenarioStep::ExpectNotIssued {
            command: command(ASSIGN),
            instance: entry(),
            within,
            unpublished: vec![event(ASSIGNED)],
        },
    ]);
    steps
}

const BOUNDED: &str = "queueing.entry/authored/assigns-once-within-the-bound";
const UNBOUNDED: &str = "queueing.entry/authored/assigns-once-by-the-deadline";
const STATED: &str = "queueing.entry/authored/assigns-the-stated-worker";
const OLD: &str = "queueing.entry/authored/enqueues";

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

fn await_suite() -> AdmittedSuite {
    suite(vec![
        (BOUNDED, assigns_once(Some(5), BTreeMap::new())),
        (UNBOUNDED, assigns_once(None, BTreeMap::new())),
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

// ---- the queue -------------------------------------------------------------------------------

/// Entries by identity, each `None` while waiting and `Some(worker)` once assigned.
#[derive(Default)]
struct Queue {
    entries: RefCell<BTreeMap<String, Option<String>>>,
    log: RefCell<Vec<ObservedEvent>>,
    minted: Cell<u32>,
    awaits: Cell<u32>,
    compared: RefCell<Vec<BTreeSet<String>>>,
    implements_await: bool,
}

impl Queue {
    fn new() -> Self {
        Self {
            implements_await: true,
            ..Self::default()
        }
    }

    /// A target written before the await capability existed.
    fn before_await() -> Self {
        Self::default()
    }
}

impl ConformanceTarget for Queue {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("queue", "1"))
    }

    fn begin_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        self.entries.borrow_mut().clear();
        self.log.borrow_mut().clear();
        Ok(())
    }

    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        assert_eq!(
            request.command.to_string(),
            ENQUEUE,
            "the system's own step is never invoked"
        );
        self.minted.set(self.minted.get() + 1);
        let id = format!("e-{}", self.minted.get());
        self.entries.borrow_mut().insert(id.clone(), None);
        let published = ObservedEvent::new(event(ENQUEUED)).with("entry_id", text(&id));
        self.log.borrow_mut().push(published.clone());
        Ok(SemanticCommandResult::took(outcome(ENQUEUE, "enqueued")).emitting(published))
    }

    fn await_outcome(&self, request: AwaitRequest) -> Result<AwaitAnswer, TargetError> {
        if !self.implements_await {
            return Err(TargetError::unsupported(
                "awaiting",
                "written before awaiting",
            ));
        }
        assert_eq!(request.command.to_string(), ASSIGN);
        self.awaits.set(self.awaits.get() + 1);
        self.compared.borrow_mut().push(request.compared.clone());
        let Node::Text(id) = &request.instance else {
            panic!("{:?}", request.instance);
        };
        let mut entries = self.entries.borrow_mut();
        let Some(slot) = entries.get_mut(id) else {
            return Ok(AwaitAnswer::NotIssued {
                elapsed_ms: request.within.map(ess_conformance::Elapsed::millis),
            });
        };
        if slot.is_some() {
            // An assigned entry is not the scheduler's: nothing happens, for as long as asked.
            return Ok(AwaitAnswer::NotIssued {
                elapsed_ms: request.within.map(ess_conformance::Elapsed::millis),
            });
        }
        *slot = Some("w-1".to_owned());
        let published = ObservedEvent::new(event(ASSIGNED))
            .with("entry_id", text(id))
            .with("worker", text("w-1"));
        self.log.borrow_mut().push(published.clone());
        Ok(AwaitAnswer::Reached {
            result: SemanticCommandResult::took(outcome(ASSIGN, "assigned")).emitting(published),
            input: BTreeMap::from([
                ("entry_id".to_owned(), text(id)),
                ("worker".to_owned(), text("w-1")),
            ]),
            elapsed_ms: Some(1_000),
        })
    }

    fn query_view(&self, _: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        unreachable!("no scenario here reads a view")
    }

    fn observe_events(
        &self,
        request: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        Ok(self
            .log
            .borrow()
            .iter()
            .filter(|seen| seen.event == request.event)
            .cloned()
            .collect())
    }

    fn configure_external_outcome(&self, _: ExternalOutcomeControl) -> Result<(), TargetError> {
        unreachable!()
    }

    fn redeliver_event(&self, _: RedeliveryRequest) -> Result<(), TargetError> {
        unreachable!()
    }

    fn end_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        Ok(())
    }
}

// ---- the cases -------------------------------------------------------------------------------

#[test]
fn the_queue_passes_the_await_scenarios() {
    let suite = await_suite();
    let queue = Queue::new();
    let results = run(&suite, &queue);
    for (id, result) in &results {
        assert_eq!(result.status, Status::Passed, "{id}: {result:#?}");
        assert!(
            result
                .checks
                .iter()
                .any(|check| check.code == CheckCode::Await && check.status == Status::Passed),
            "{id} checks the await: {result:#?}"
        );
    }
    assert!(
        queue.awaits.get() > 4,
        "without a bound, `nothing` is asked again until the runner's deadline: {} awaits",
        queue.awaits.get()
    );
    assert!(
        queue.compared.borrow().iter().all(BTreeSet::is_empty),
        "synthesis-shaped steps compare no input value"
    );
}

#[test]
fn await_outcome_unsupported_by_default_keeps_old_targets_green() {
    let old = suite(vec![(OLD, arrange())]);
    let results = run(&old, &Queue::before_await());
    assert_eq!(results[OLD].status, Status::Passed, "{:#?}", results[OLD]);

    let results = run(&await_suite(), &Queue::before_await());
    for (id, result) in &results {
        assert_eq!(
            result.status,
            Status::Unsupported,
            "{id} is never passed by a target that cannot await: {result:#?}"
        );
    }
}

#[test]
fn a_stated_input_is_compared_and_named_to_the_target() {
    let stated = |worker: &str| {
        BTreeMap::from([("worker".to_owned(), ScenarioValue::literal(text(worker)))])
    };
    let queue = Queue::new();
    let results = run(
        &suite(vec![(STATED, assigns_once(Some(5), stated("w-1")))]),
        &queue,
    );
    assert_eq!(
        results[STATED].status,
        Status::Passed,
        "{:#?}",
        results[STATED]
    );
    assert!(
        queue
            .compared
            .borrow()
            .iter()
            .any(|names| names == &BTreeSet::from(["worker".to_owned()])),
        "the request names the compared field, never its value"
    );

    let results = run(
        &suite(vec![(STATED, assigns_once(Some(5), stated("w-2")))]),
        &Queue::new(),
    );
    assert_eq!(results[STATED].status, Status::Failed);
    assert_eq!(
        failing(&results[STATED]),
        BTreeSet::from([CheckCode::Await])
    );
}

#[test]
fn an_awaited_value_read_before_any_await_is_a_suite_defect() {
    let mut steps = arrange();
    steps.push(ScenarioStep::ExpectEventValues {
        event: event(ENQUEUED),
        payload: BTreeMap::from([(
            "entry_id".to_owned(),
            ScenarioValue::Awaited {
                field: "entry_id".to_owned(),
            },
        )]),
        shape: PayloadShape::new(),
    });
    let results = run(&suite(vec![(OLD, steps)]), &Queue::new());
    let result = &results[OLD];
    assert_eq!(result.status, Status::Error, "{result:#?}");
    assert!(result
        .checks
        .iter()
        .any(|check| check.code == CheckCode::Suite && check.status == Status::Error));
}

#[test]
fn the_forwarding_wrappers_forward_the_await() {
    let suite = await_suite();
    let untraced = ess_conformance::reference::Untraced(Queue::new());
    for result in run(&suite, &untraced).values() {
        assert_eq!(result.status, Status::Passed, "{result:#?}");
    }
    let faulty = ess_conformance::faulty::Faulty::new(
        Queue::new(),
        ess_conformance::faulty::Fault::StaleReadYourWrites,
    );
    for result in run(&suite, &faulty).values() {
        assert_eq!(result.status, Status::Passed, "{result:#?}");
    }
}

/// Where each faulty queue fails: per scenario, the codes of its failing checks. A scenario not
/// named passes.
fn caught(fault: AwaitFault) -> Vec<(&'static str, BTreeSet<CheckCode>)> {
    let both = |code: CheckCode| {
        vec![
            (BOUNDED, BTreeSet::from([code])),
            (UNBOUNDED, BTreeSet::from([code])),
        ]
    };
    match fault {
        // Nothing reached, an answer about another instance, or a step from a refusing state.
        AwaitFault::NeverTakesStep | AwaitFault::OtherInstance | AwaitFault::TakesRefusedStep => {
            both(CheckCode::Await)
        }
        // Late only against a stated bound; without one there is nothing to be late for. An early
        // `nothing` without a bound is asked again until the deadline and never believed; with
        // one, the short reading fails.
        AwaitFault::TooLate | AwaitFault::AnswersAtOnce => {
            vec![(BOUNDED, BTreeSet::from([CheckCode::Await]))]
        }
        AwaitFault::OtherBranch => both(CheckCode::Outcome),
        // A reported value that disagrees with the published one fails the event's payload check;
        // one of the wrong type fails the declared input's shape, and the payload check after it.
        AwaitFault::DisagreeingInput | AwaitFault::WrongInputType => both(CheckCode::Payload),
        AwaitFault::PublishesSilently => both(CheckCode::Event),
        other => panic!("no expectation written for {other:?}"),
    }
}

#[test]
fn each_faulty_queue_fails_at_its_step() {
    let suite = await_suite();
    assert_eq!(AwaitFault::ALL.len(), 9);
    for &fault in AwaitFault::ALL {
        let results = run(&suite, &AwaitFaulty::new(Queue::new(), fault));
        let expected: BTreeMap<&str, BTreeSet<CheckCode>> = caught(fault).into_iter().collect();
        for (id, result) in &results {
            match expected.get(id.as_str()) {
                Some(codes) => {
                    assert_eq!(result.status, Status::Failed, "{fault:?} {id}: {result:#?}");
                    assert_eq!(&failing(result), codes, "{fault:?} {id}: {result:#?}");
                }
                None => assert_eq!(result.status, Status::Passed, "{fault:?} {id}: {result:#?}"),
            }
        }
    }
}

// ---- the Go and TypeScript runners (stage 2) ---------------------------------------------------
//
// Each generated runtime replays the transcript a Rust target wrote while the Rust reference runner
// asked it (`support_go::Recorder`): the same answers to the same questions, so a verdict that
// differs is a difference in how the runner read the suite, and a request it sends that the
// reference runner did not is written down as a divergence.

mod support_go;
mod support_ts;

/// Every await scenario, one stating the worker, and one with no await at all.
fn runtime_suite(stated: &str) -> AdmittedSuite {
    suite(vec![
        (BOUNDED, assigns_once(Some(5), BTreeMap::new())),
        (UNBOUNDED, assigns_once(None, BTreeMap::new())),
        (
            STATED,
            assigns_once(
                Some(5),
                BTreeMap::from([("worker".to_owned(), ScenarioValue::literal(text(stated)))]),
            ),
        ),
        (OLD, arrange()),
    ])
}

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

fn every(suite: &AdmittedSuite, status: &str) -> BTreeMap<String, String> {
    suite
        .suite()
        .scenarios
        .keys()
        .map(|id| (id.to_string(), status.to_owned()))
        .collect()
}

/// `suite`'s verdicts with the await scenarios `status` and the one without an await passed.
fn awaiting(suite: &AdmittedSuite, status: &str) -> BTreeMap<String, String> {
    let mut verdicts = every(suite, status);
    verdicts.insert(OLD.to_owned(), "passed".to_owned());
    verdicts
}

fn go_replay<T>(
    directory: &std::path::Path,
    suite: &AdmittedSuite,
    recorder: &support_go::Recorder<T>,
    bare: bool,
) -> BTreeMap<String, String> {
    std::fs::write(
        directory.join("essconform").join("suite.json"),
        suite.original_json(),
    )
    .unwrap();
    let env: &[(&str, &str)] = if bare {
        &[("ESS_TRANSCRIPT_BARE", "1")]
    } else {
        &[]
    };
    let replayed = support_go::replay(directory, recorder, env);
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

fn ts_replay<T>(
    package: &support_ts::Package,
    suite: &AdmittedSuite,
    recorder: &support_go::Recorder<T>,
    bare: bool,
) -> BTreeMap<String, String> {
    let replayed = package.replay(suite, &recorder.transcript(), bare);
    assert_ne!(replayed.exit, Some(2), "{}", replayed.log);
    assert!(
        replayed.divergences.is_empty(),
        "the TypeScript runtime sent requests the reference runner did not:\n{}\n{}",
        replayed.divergences.join("\n"),
        replayed.log
    );
    replayed.outcomes
}

#[test]
fn the_go_runner_awaits_a_system_issued_command() {
    let suite = runtime_suite("w-1");
    let directory = support_go::package("await", suite.suite(), &[support_go::TRANSCRIPT_TARGET]);

    let (rust, recorder) = recorded(&suite, Queue::new());
    assert_eq!(rust, every(&suite, "passed"));
    assert_eq!(go_replay(&directory, &suite, &recorder, false), rust);
    // A Go target that does not implement AwaitTarget reports each await scenario unsupported,
    // never passed, and keeps the old one green.
    assert_eq!(
        go_replay(&directory, &suite, &recorder, true),
        awaiting(&suite, "unsupported")
    );

    let (rust, recorder) = recorded(&suite, Queue::before_await());
    assert_eq!(rust, awaiting(&suite, "unsupported"));
    assert_eq!(go_replay(&directory, &suite, &recorder, false), rust);

    let mismatched = runtime_suite("w-2");
    let (rust, recorder) = recorded(&mismatched, Queue::new());
    assert_eq!(rust[STATED], "failed", "{rust:?}");
    assert_eq!(go_replay(&directory, &mismatched, &recorder, false), rust);
}

#[test]
fn the_typescript_runner_awaits_a_system_issued_command() {
    let suite = runtime_suite("w-1");
    let package = support_ts::Package::new("await", suite.suite());

    let (rust, recorder) = recorded(&suite, Queue::new());
    assert_eq!(rust, every(&suite, "passed"));
    assert_eq!(ts_replay(&package, &suite, &recorder, false), rust);
    // A TypeScript target without `awaitOutcome` reports each await scenario unsupported, never
    // passed, and keeps the old one green.
    assert_eq!(
        ts_replay(&package, &suite, &recorder, true),
        awaiting(&suite, "unsupported")
    );

    let (rust, recorder) = recorded(&suite, Queue::before_await());
    assert_eq!(rust, awaiting(&suite, "unsupported"));
    assert_eq!(ts_replay(&package, &suite, &recorder, false), rust);

    let mismatched = runtime_suite("w-2");
    let (rust, recorder) = recorded(&mismatched, Queue::new());
    assert_eq!(rust[STATED], "failed", "{rust:?}");
    assert_eq!(ts_replay(&package, &mismatched, &recorder, false), rust);
}

/// The queue, its reached answers carrying `elapsed_ms` instead of the reading it took.
struct Reading {
    queue: Queue,
    elapsed_ms: Option<u64>,
}

impl ConformanceTarget for Reading {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        self.queue.identity()
    }
    fn begin_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.queue.begin_scenario(scenario)
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        self.queue.execute_command(request)
    }
    fn await_outcome(&self, request: AwaitRequest) -> Result<AwaitAnswer, TargetError> {
        Ok(match self.queue.await_outcome(request)? {
            AwaitAnswer::Reached { result, input, .. } => AwaitAnswer::Reached {
                result,
                input,
                elapsed_ms: self.elapsed_ms,
            },
            other @ AwaitAnswer::NotIssued { .. } => other,
        })
    }
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        self.queue.query_view(request)
    }
    fn observe_events(
        &self,
        request: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        self.queue.observe_events(request)
    }
    fn configure_external_outcome(
        &self,
        request: ExternalOutcomeControl,
    ) -> Result<(), TargetError> {
        self.queue.configure_external_outcome(request)
    }
    fn redeliver_event(&self, request: RedeliveryRequest) -> Result<(), TargetError> {
        self.queue.redeliver_event(request)
    }
    fn end_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.queue.end_scenario(scenario)
    }
}

/// A step reached exactly at the bound is within it; one millisecond later is not, and a reached
/// answer with no reading of the clock cannot be held to a bound at all. Without a bound neither
/// matters. The three runners agree (review 2026-10-19, T2).
#[test]
fn a_reading_at_the_bound_passes_and_one_past_it_or_none_fails_in_every_runner() {
    let suite = await_suite();
    let directory = support_go::package(
        "await-readings",
        suite.suite(),
        &[support_go::TRANSCRIPT_TARGET],
    );
    let package = support_ts::Package::new("await-readings", suite.suite());
    for (elapsed_ms, bounded) in [
        (Some(5_000), "passed"),
        (Some(5_001), "failed"),
        (None, "failed"),
    ] {
        let target = Reading {
            queue: Queue::new(),
            elapsed_ms,
        };
        let (rust, recorder) = recorded(&suite, target);
        let mut expected = every(&suite, "passed");
        expected.insert(BOUNDED.to_owned(), bounded.to_owned());
        assert_eq!(rust, expected, "{elapsed_ms:?}: the reference runner");
        if bounded == "failed" {
            let results = run(
                &suite,
                &Reading {
                    queue: Queue::new(),
                    elapsed_ms,
                },
            );
            assert_eq!(
                failing(&results[BOUNDED]),
                BTreeSet::from([CheckCode::Await]),
                "{elapsed_ms:?}: {:#?}",
                results[BOUNDED]
            );
        }
        assert_eq!(
            go_replay(&directory, &suite, &recorder, false),
            rust,
            "{elapsed_ms:?}: Go (left) and Rust (right)"
        );
        assert_eq!(
            ts_replay(&package, &suite, &recorder, false),
            rust,
            "{elapsed_ms:?}: TypeScript (left) and Rust (right)"
        );
    }
}

/// Every faulty queue, through the three runners: the same verdict per scenario, each the one
/// [`caught`] names. Among them the refusal read as "nothing within the bound": a target that takes
/// the step from a refusing state, one that publishes without saying so, and one that answers
/// `nothing` before the window has closed.
#[test]
fn all_runners_agree_on_an_await_refusal_within_the_bound() {
    let suite = await_suite();
    let directory = support_go::package(
        "await-faults",
        suite.suite(),
        &[support_go::TRANSCRIPT_TARGET],
    );
    let package = support_ts::Package::new("await-faults", suite.suite());
    for &fault in AwaitFault::ALL {
        let (rust, recorder) = recorded(&suite, AwaitFaulty::new(Queue::new(), fault));
        let mut expected = every(&suite, "passed");
        for (id, _) in caught(fault) {
            expected.insert(id.to_owned(), "failed".to_owned());
        }
        assert_eq!(rust, expected, "{fault:?}: the reference runner");
        assert_eq!(
            go_replay(&directory, &suite, &recorder, false),
            rust,
            "{fault:?}: Go (left) and Rust (right)"
        );
        assert_eq!(
            ts_replay(&package, &suite, &recorder, false),
            rust,
            "{fault:?}: TypeScript (left) and Rust (right)"
        );
    }
}
