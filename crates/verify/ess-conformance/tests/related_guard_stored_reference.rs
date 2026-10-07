//! A `when_related:` guard may read the related row's identity from a stored field of the
//! addressed subject, as it was just before the branch (beyond10x/ess#304, `ess/22`): a task stores
//! the task blocking it, Optional, and is completed only once that task is done.
//!
//! The synthesized suite witnesses each answer the stored reference gives — absent, naming a row in
//! the refusing state, naming a row in the accepting state, naming no row — and an implementation
//! that reads another row, the subject's own row, or refuses whenever a blocker is stored fails it.

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};

use ess_compiler::ir::{EssIr, ResolvedCondition, ResolvedRelatedTest};
use ess_compiler::{resolve::compile, source::SourceMap};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::scenario::{InstanceName, ScenarioStep};
use ess_conformance::target::*;
use ess_conformance::{synthesize::Synthesis, AdmittedSuite, Runner, ScenarioValue};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_primitives::node::Node;
use ess_primitives::predicate::Predicate;

const MODEL: &str = include_str!("fixtures/related-guard-stored-reference.yaml");
const ADD: &str = "demo.tasks.AddTask";
const COMPLETE: &str = "demo.tasks.CompleteTask";
const BLOCKER_MISSING: &str = "demo.tasks.CompleteTask/outcome/blocker-missing";
const BLOCKED: &str = "demo.tasks.CompleteTask/outcome/blocked";
const COMPLETED: &str = "demo.tasks.CompleteTask/outcome/completed";
const WRONG_STATE: &str = "demo.tasks.CompleteTask/outcome/wrong-state";

fn ir_of(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("stored-reference.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn ir() -> EssIr {
    ir_of(MODEL)
}

fn synthesis() -> Synthesis {
    ess_conformance::synthesize::synthesize(&ir())
}

fn has_scenario(result: &Synthesis, id: &str) -> bool {
    result
        .suite
        .scenarios
        .keys()
        .any(|key| key.to_string() == id)
}

/// What a task's stored `blocked_by` held when it was completed, as the scenario arranged it.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Blocker {
    /// Left out when the task was added.
    Absent,
    /// A task the scenario created, and whether it was done by then.
    Task { done: bool },
    /// An identity no task the scenario created carries.
    Dangling,
}

/// Every `CompleteTask` the scenario sends, with what the completed task's stored blocker held and
/// the outcome asserted.
fn completions(steps: &[ScenarioStep]) -> Vec<(Blocker, String)> {
    let mut pending: Option<(String, BTreeMap<String, ScenarioValue>)> = None;
    let mut stored: BTreeMap<InstanceName, Option<ScenarioValue>> = BTreeMap::new();
    let mut added: Option<Option<ScenarioValue>> = None;
    let mut done: BTreeSet<InstanceName> = BTreeSet::new();
    let mut found = Vec::new();
    for step in steps {
        match step {
            ScenarioStep::ExecuteCommand { command, input, .. } => {
                pending = Some((command.to_string(), input.clone()));
            }
            ScenarioStep::ExpectOutcome { outcome } => {
                let Some((command, input)) = pending.take() else {
                    continue;
                };
                let taken = outcome.outcome.to_string();
                if command == ADD && taken == "added" {
                    added = Some(input.get("blocked_by").cloned());
                }
                if command == COMPLETE {
                    let Some(ScenarioValue::Instance { instance }) = input.get("task_id") else {
                        continue;
                    };
                    let blocker = match stored.get(instance).cloned().flatten() {
                        None => Blocker::Absent,
                        Some(ScenarioValue::Instance { instance: blocker }) => Blocker::Task {
                            done: done.contains(&blocker),
                        },
                        Some(_) => Blocker::Dangling,
                    };
                    found.push((blocker, format!("{COMPLETE}/outcome/{taken}")));
                    if taken == "completed" {
                        done.insert(instance.clone());
                    }
                }
            }
            ScenarioStep::CaptureInstance { instance, .. } => {
                if let Some(blocked_by) = added.take() {
                    stored.insert(instance.clone(), blocked_by);
                }
            }
            _ => {}
        }
    }
    found
}

fn statuses<T: ConformanceTarget>(result: &Synthesis, target: &T) -> BTreeMap<String, Status> {
    let admitted =
        AdmittedSuite::from_suite(&result.suite).unwrap_or_else(|error| panic!("{error}"));
    Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, target)
        .into_report()
        .scenarios
        .into_iter()
        .filter(|run| run.scenario.to_string().contains("CompleteTask"))
        .map(|run| (run.scenario.to_string(), run.status))
        .collect()
}

fn assert_honest_baseline(result: &Synthesis) {
    let honest = statuses(result, &Interpreted::for_model(ir()));
    for id in [BLOCKER_MISSING, BLOCKED, COMPLETED, WRONG_STATE] {
        assert_eq!(
            honest.get(id),
            Some(&Status::Passed),
            "{id} executes and passes before a mutant is meaningful: {honest:#?}"
        );
    }
    assert!(
        honest.values().all(|status| *status == Status::Passed),
        "the declared interpreter passes before a mutant is meaningful: {honest:#?}"
    );
}

#[test]
fn a_blocked_task_is_refused_until_its_blocker_is_done() {
    let result = synthesis();
    for id in [BLOCKER_MISSING, BLOCKED, COMPLETED, WRONG_STATE] {
        assert!(
            has_scenario(&result, id),
            "{id} exists; refusals: {:#?}",
            result
                .refusals
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
        );
    }
    let witnessed: Vec<(Blocker, String)> = result
        .suite
        .scenarios
        .values()
        .flat_map(|scenario| completions(&scenario.steps))
        .collect();
    for (blocker, outcome) in [
        (Blocker::Task { done: false }, BLOCKED),
        (Blocker::Task { done: true }, COMPLETED),
        (Blocker::Absent, COMPLETED),
        (Blocker::Dangling, BLOCKER_MISSING),
    ] {
        assert!(
            witnessed
                .iter()
                .any(|(held, taken)| *held == blocker && taken == outcome),
            "{outcome} is witnessed with the stored blocker {blocker:?}: {witnessed:#?}"
        );
    }
    assert!(
        !witnessed
            .iter()
            .any(|(held, taken)| *held == Blocker::Task { done: false } && taken == COMPLETED),
        "no scenario completes a task whose stored blocker is open: {witnessed:#?}"
    );
    assert_honest_baseline(&result);
}

/// Links a task to the task created right after the one its `blocked_by` names, as an
/// implementation resolving the stored reference to the wrong row would.
struct NeighbourRow {
    inner: Interpreted,
    created: RefCell<Vec<Node>>,
}

impl ConformanceTarget for NeighbourRow {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        self.inner.identity()
    }

    fn begin_scenario(&self, context: &ScenarioContext) -> Result<(), TargetError> {
        self.created.borrow_mut().clear();
        self.inner.begin_scenario(context)
    }

    fn end_scenario(&self, context: &ScenarioContext) -> Result<(), TargetError> {
        self.inner.end_scenario(context)
    }

    fn execute_command(
        &self,
        mut request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        let adding = request.command.to_string() == ADD;
        if adding {
            let created = self.created.borrow();
            if let Some(named) = request.input.get("blocked_by") {
                if let Some(at) = created.iter().position(|held| held == named) {
                    if let Some(next) = created.get(at + 1) {
                        request.input.insert("blocked_by".to_owned(), next.clone());
                    }
                }
            }
        }
        let result = self.inner.execute_command(request)?;
        if adding {
            for event in &result.direct_events {
                if let Some(identity) = event.payload.get("task_id") {
                    self.created.borrow_mut().push(identity.clone());
                }
            }
        }
        Ok(result)
    }

    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        self.inner.query_view(request)
    }

    fn observe_events(
        &self,
        request: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        self.inner.observe_events(request)
    }

    fn configure_external_outcome(
        &self,
        control: ExternalOutcomeControl,
    ) -> Result<(), TargetError> {
        self.inner.configure_external_outcome(control)
    }

    fn redeliver_event(&self, request: RedeliveryRequest) -> Result<(), TargetError> {
        self.inner.redeliver_event(request)
    }
}

/// The model with the `blocked` branch's condition replaced: the implementation a mutant is.
fn with_blocked(condition: impl Fn(&ResolvedCondition) -> ResolvedCondition) -> EssIr {
    ir().with_commands_rewritten(|command| {
        let mut command = command.clone();
        if command.name.to_string() == COMPLETE {
            for outcome in &mut command.outcomes {
                if outcome.name.as_str() == "blocked" {
                    outcome.condition = condition(&outcome.condition);
                }
            }
        }
        command
    })
}

#[test]
fn a_target_reading_another_rows_state_fails() {
    let result = synthesis();
    assert_honest_baseline(&result);
    let faulty = statuses(
        &result,
        &NeighbourRow {
            inner: Interpreted::for_model(ir()),
            created: RefCell::new(Vec::new()),
        },
    );
    for id in [BLOCKED, COMPLETED] {
        assert_ne!(
            faulty.get(id),
            Some(&Status::Passed),
            "{id} names a row between rows in the opposite state: {faulty:#?}"
        );
    }
    assert_eq!(
        faulty.get(BLOCKER_MISSING),
        Some(&Status::Passed),
        "the mutant executes: a stored identity no row carries has no neighbour: {faulty:#?}"
    );
}

#[test]
fn a_target_reading_the_subjects_own_state_fails() {
    let result = synthesis();
    assert_honest_baseline(&result);
    let own_state = with_blocked(|condition| {
        let ResolvedCondition::Related {
            test: ResolvedRelatedTest::Holds { predicate },
            input,
            ..
        } = condition
        else {
            panic!("`blocked` reads the related row: {condition:?}")
        };
        ResolvedCondition::SubjectPredicate {
            predicate: predicate.clone(),
            input: input.clone(),
        }
    });
    let faulty = statuses(&result, &Interpreted::for_model(own_state));
    assert_ne!(
        faulty.get(COMPLETED),
        Some(&Status::Passed),
        "an open task whose blocker is done completes: {faulty:#?}"
    );
    // Every open task reads as blocked by itself, so no arrangement completes a task under this
    // mutant; the missing row is answered before the predicate is read, and still passes.
    assert_eq!(
        faulty.get(BLOCKER_MISSING),
        Some(&Status::Passed),
        "the mutant executes: {faulty:#?}"
    );
}

#[test]
fn a_target_refusing_whenever_a_blocker_is_set_fails() {
    let result = synthesis();
    assert_honest_baseline(&result);
    let any_blocker = with_blocked(|condition| {
        let ResolvedCondition::Related {
            via, entity, input, ..
        } = condition
        else {
            panic!("`blocked` reads the related row: {condition:?}")
        };
        let either: Predicate = Predicate::Any(vec![
            "state == Open".parse().expect("a predicate"),
            "state == Done".parse().expect("a predicate"),
        ]);
        ResolvedCondition::Related {
            via: via.clone(),
            entity: entity.clone(),
            test: ResolvedRelatedTest::Holds { predicate: either },
            input: input.clone(),
        }
    });
    let faulty = statuses(&result, &Interpreted::for_model(any_blocker));
    assert_ne!(
        faulty.get(COMPLETED),
        Some(&Status::Passed),
        "a task whose stored blocker is done completes: {faulty:#?}"
    );
    assert_eq!(
        faulty.get(BLOCKED),
        Some(&Status::Passed),
        "the mutant executes: an open blocker still refuses: {faulty:#?}"
    );
}

#[test]
fn a_dangling_reference_is_witnessed_or_noted_unreachable() {
    // `AddTask` stores the identity it is given unchecked: a stored identity no task carries is
    // reachable, and witnessed.
    let result = synthesis();
    let missing = result
        .suite
        .scenarios
        .iter()
        .find(|(id, _)| id.to_string() == BLOCKER_MISSING)
        .map_or_else(
            || panic!("{BLOCKER_MISSING} exists: {:#?}", result.refusals),
            |(_, scenario)| completions(&scenario.steps),
        );
    assert!(
        missing
            .iter()
            .any(|(held, taken)| *held == Blocker::Dangling && taken == BLOCKER_MISSING),
        "the missing row is a stored identity no task carries: {missing:#?}"
    );

    // Where the only writer refuses an identity no task carries, no run stores one: the branch is
    // unreachable, and the refusal under its id says so rather than sending an arrangement that
    // never happens.
    let checked = MODEL.replace(
        "    outcomes:\n      - name: added\n",
        "    outcomes:\n      - name: no-such-blocker\n        when_related: {via: input.blocked_by, exists: false}\n        error: demo.tasks.BlockerMissing\n      - name: added\n",
    );
    assert_ne!(checked, MODEL, "the writer gains its check");
    let result = ess_conformance::synthesize::synthesize(&ir_of(&checked));
    assert!(
        !has_scenario(&result, BLOCKER_MISSING),
        "no scenario arranges a stored identity no writer stores"
    );
    let noted: Vec<String> = result
        .refusals
        .iter()
        .map(|refusal| format!("{} {refusal}", refusal.code()))
        .filter(|text| text.contains(BLOCKER_MISSING))
        .collect();
    assert!(
        noted
            .iter()
            .any(|text| text.contains("ESS-SYNTH-003") && text.contains("unreachable")),
        "the branch is noted unreachable, naming why: {noted:#?}"
    );
}
