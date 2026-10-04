//! Adversary pass 1 against beyond10x/ess#266 and #267 (`binding_effects`).
//!
//! Two attacks, each driven from the contract in `docs/design/binding-arrangement-and-drop.md`:
//!
//! 1. "What drop can prove", step 5: the drop scenario compares the destination row against its
//!    pre-trigger snapshot. A second binding the same trigger sets off on the same row is part of
//!    the honest system; the synthesized drop scenario must still pass against it.
//! 2. "Arrangement with eventual bindings": bindings are eventual, so a synthesized suite must pass
//!    against an honest target whose dispatcher runs a binding *after* the triggering command has
//!    answered. The interpreter dispatches synchronously inside the trigger, which hides every
//!    immediate read of a row a binding is still moving. [`Deferred`] is the same interpreter with
//!    its bindings run by a dispatcher that lags one read behind.

mod support_binding_arrangement;

use std::cell::RefCell;
use std::collections::{BTreeMap, VecDeque};

use ess_compiler::ir::ResolvedMappingValue;
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::{ConformanceReport, Status};
use ess_conformance::scenario::{BindingRef, CommandRef, ConformanceSuite};
use ess_conformance::synthesize::synthesize;
use ess_conformance::target::*;
use ess_conformance::{AdmittedSuite, Runner};
use ess_domain::binding::Failure;
use ess_primitives::ids::CorrelationId;
use ess_primitives::node::Node;
use support_binding_arrangement::*;

fn run_against<T: ConformanceTarget>(suite: &ConformanceSuite, target: &T) -> ConformanceReport {
    let admitted = AdmittedSuite::from_suite(suite).expect("the suite is admitted");
    Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, target)
        .into_report()
}

fn not_passed(report: &ConformanceReport) -> BTreeMap<String, Vec<String>> {
    report
        .scenarios
        .iter()
        .filter(|scenario| scenario.status != Status::Passed)
        .map(|scenario| {
            (
                scenario.scenario.to_string(),
                scenario
                    .checks
                    .iter()
                    .filter(|check| check.status != Status::Passed)
                    .map(|check| format!("{:?} {:?} {}", check.status, check.code, check.about))
                    .collect(),
            )
        })
        .collect()
}

// ---- 1. a second binding on the same trigger and the same row ----------------------------------

/// The fixture with `Relabel` (an update that leaves the state alone) and `relabels-on-<event>`, a
/// second binding on `event` that relabels the job the event names. With `Kicked`: under the
/// specification, a `Kick` whose `Start` attempt is refused still relabels the job.
fn with_relabel_bound_to(event: &str) -> String {
    let text = rewrite(
        FIXTURE,
        "events:\n",
        "events:\n  - name: jobs.job.Relabelled\n    fields: [{name: job_id, type: jobs.job.JobId}]\n",
    );
    let text = rewrite(
        &text,
        "  - name: jobs.job.Stop\n",
        "  - name: jobs.job.Relabel
    input:
      - {name: job_id, type: jobs.job.JobId}
    outcomes:
      - name: relabelled
        updates: jobs.job.Job
        instance: job_id
        sets: {label: relabelled}
        emits: [jobs.job.Relabelled]
        payload: {jobs.job.Relabelled: {job_id: input.job_id}}
      - {name: not-found, unknown_instance: true, error: jobs.job.JobNotFound}

  - name: jobs.job.Stop
",
    );
    format!(
        "{text}
  - id: relabels-on-{lower}
    when: {{event: jobs.job.{event}}}
    invoke: {{command: jobs.job.Relabel}}
    mapping: {{job_id: event.job_id}}
    delivery: at_least_once
    on_failure: drop
",
        lower = event.to_lowercase()
    )
}

#[test]
fn adv_drop_passes_an_honest_target_whose_trigger_sets_off_a_second_binding_on_the_row() {
    let text = with_relabel_bound_to("Kicked");
    let synthesis = synthesize(&model(&text));
    assert!(
        synthesis
            .suite
            .scenarios
            .keys()
            .any(|id| id.to_string() == DROP),
        "the drop scenario is synthesized"
    );
    let honest = run_against(&synthesis.suite, &interpreted(&text));
    let failed = not_passed(&honest);
    assert_eq!(
        failed.get(DROP),
        None,
        "the honest interpreter runs both bindings the spec declares, and the synthesized drop \
         scenario calls the row it relabels `unchanged`"
    );
}

// ---- 2. a dispatcher that runs bindings after the trigger answers ------------------------------

struct Bound {
    name: BindingRef,
    event: String,
    command: CommandRef,
    mapping: Vec<(String, String)>,
    retries: bool,
}

/// The interpreter of the model without its bindings, plus a dispatcher that runs the bindings one
/// round per read: a read answers from the state as it is, then every delivery pending at that
/// moment is run, and whatever those publish waits for the next read. A command sent before any
/// read overtakes a pending binding. This is an honest eventual dispatcher.
struct Deferred {
    inner: Interpreted,
    bound: Vec<Bound>,
    pending: RefCell<VecDeque<ObservedEvent>>,
    published: RefCell<Vec<ObservedEvent>>,
    attempts: RefCell<Vec<(CorrelationId, ObservedInvocation)>>,
}

impl Deferred {
    fn new(text: &str) -> Self {
        let ir = model(text);
        let bound = ir
            .bindings()
            .values()
            .map(|binding| Bound {
                name: BindingRef::new(binding.name.clone()),
                event: binding
                    .cause
                    .event()
                    .expect("an event binding")
                    .name()
                    .to_string(),
                command: CommandRef::new(ir.command(&binding.command).name.clone()),
                mapping: binding
                    .mapping
                    .iter()
                    .map(|mapped| match &mapped.value {
                        ResolvedMappingValue::EventField { field, .. } => {
                            (mapped.target.clone(), field.clone())
                        }
                        other => panic!("only event-field mappings here: {other:?}"),
                    })
                    .collect(),
                retries: matches!(binding.failure, Failure::Retry),
            })
            .collect();
        let without = text
            .split("\nbindings:\n")
            .next()
            .expect("text before the bindings");
        Self {
            inner: interpreted(without),
            bound,
            pending: RefCell::default(),
            published: RefCell::default(),
            attempts: RefCell::default(),
        }
    }

    fn published(&self, result: &SemanticCommandResult, correlation: &CorrelationId) {
        for event in &result.direct_events {
            let mut event = event.clone();
            if event.correlation.is_none() {
                event.correlation = Some(correlation.clone());
            }
            self.published.borrow_mut().push(event.clone());
            self.pending.borrow_mut().push_back(event);
        }
    }

    /// One round: every delivery pending now.
    fn round(&self) -> Result<(), TargetError> {
        let now: Vec<ObservedEvent> = self.pending.borrow_mut().drain(..).collect();
        for event in now {
            let correlation = event.correlation.clone().expect("correlated");
            for bound in self
                .bound
                .iter()
                .filter(|bound| bound.event == event.event.name().to_string())
            {
                let input: BTreeMap<String, Node> = bound
                    .mapping
                    .iter()
                    .filter_map(|(to, from)| {
                        event
                            .payload
                            .get(from)
                            .map(|value| (to.clone(), value.clone()))
                    })
                    .collect();
                for _ in 0..8 {
                    self.attempts.borrow_mut().push((
                        correlation.clone(),
                        ObservedInvocation {
                            binding: bound.name.clone(),
                            command: bound.command.clone(),
                            input: input.clone(),
                        },
                    ));
                    let result = self.inner.execute_command(SemanticCommandRequest {
                        command: bound.command.clone(),
                        actor: None,
                        caller: None,
                        input: input.clone(),
                        correlation: correlation.clone(),
                    })?;
                    self.published(&result, &correlation);
                    if result.error.is_none() || !bound.retries {
                        break;
                    }
                }
            }
        }
        Ok(())
    }
}

impl ConformanceTarget for Deferred {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        self.inner.identity()
    }
    fn begin_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.pending.borrow_mut().clear();
        self.published.borrow_mut().clear();
        self.attempts.borrow_mut().clear();
        self.inner.begin_scenario(scenario)
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        let correlation = request.correlation.clone();
        let result = self.inner.execute_command(request)?;
        self.published(&result, &correlation);
        Ok(result)
    }
    fn execute_command_without_input(
        &self,
        request: AbsentInputRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        self.inner.execute_command_without_input(request)
    }
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        let answer = self.inner.query_view(request);
        self.round()?;
        answer
    }
    fn observe_events(
        &self,
        request: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        let answer = self.inner.observe_events(request);
        self.round()?;
        answer
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
    fn redeliver_event(&self, request: RedeliveryRequest) -> Result<(), TargetError> {
        let last = self
            .published
            .borrow()
            .iter()
            .rev()
            .find(|event| event.event == request.event)
            .cloned()
            .ok_or_else(|| TargetError::unsupported("deferred", "event not published"))?;
        self.pending.borrow_mut().push_back(last);
        Ok(())
    }
    fn observe_invocations(
        &self,
        request: InvocationObservationRequest,
    ) -> Result<Vec<ObservedInvocation>, TargetError> {
        // One call is one wait up to the request's deadline (the mapping step asks once), so the
        // dispatcher gets to run what is pending before the answer.
        for _ in 0..64 {
            if self.pending.borrow().is_empty() {
                break;
            }
            self.round()?;
        }
        let answer = self
            .attempts
            .borrow()
            .iter()
            .filter(|(correlation, invocation)| {
                *correlation == request.correlation
                    && invocation.binding == request.binding
                    && invocation.command == request.command
            })
            .map(|(_, invocation)| invocation.clone())
            .collect();
        Ok(answer)
    }
    fn end_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.inner.end_scenario(scenario)
    }
}

const STARTED_STOPS: &str = "  - id: started-stops
    when: {event: jobs.job.Started}
    invoke: {command: jobs.job.Stop}
    mapping: {job_id: event.job_id}
    delivery: at_least_once
    on_failure: drop
";

/// The deferred dispatcher is a real dispatcher: with it, the binding scenarios pass, and without
/// the binding they fail, exactly as with the interpreter's own.
#[test]
fn adv_deferred_dispatcher_is_a_working_binding_target() {
    let suite = synthesize(&model(FIXTURE)).suite;
    let report = run_against(&suite, &Deferred::new(FIXTURE));
    for id in [
        "created-starts/binding/flow",
        "created-starts/binding/mapping",
        "created-starts/binding/delivery",
        "kicked-starts/binding/flow",
        "kicked-starts/binding/mapping",
        "kicked-starts/binding/delivery",
        DROP,
    ] {
        assert_eq!(
            not_passed(&report).get(id),
            None,
            "{id} passes against the deferred dispatcher"
        );
    }
    let disabled = run_against(&suite, &Deferred::new(&without_created_starts()));
    assert!(not_passed(&disabled).contains_key("created-starts/binding/flow"));
    // Zero attempts fails delivery as well as drop (runbook, #267 row).
    let zero = run_against(&suite, &Deferred::new(&without_kicked_starts()));
    for id in ["kicked-starts/binding/delivery", DROP] {
        assert!(not_passed(&zero).contains_key(id), "{id}");
    }
}

#[test]
fn adv_fixture_suite_passes_an_honest_eventual_dispatcher() {
    let suite = synthesize(&model(FIXTURE)).suite;
    let report = run_against(&suite, &Deferred::new(FIXTURE));
    assert_eq!(not_passed(&report), BTreeMap::new());
}

#[test]
fn adv_chained_suite_passes_an_honest_eventual_dispatcher() {
    let chained = format!("{FIXTURE}{STARTED_STOPS}");
    let suite = synthesize(&model(&chained)).suite;
    let report = run_against(&suite, &Deferred::new(&chained));
    assert_eq!(not_passed(&report), BTreeMap::new());
}

/// `Created` sets off two bindings on the job it made: `created-starts` and a relabel. Where the
/// row rests is not nameable (two at once, `binding_effects.rs:361`), so `Create/outcome/created` is
/// meant to keep "what the branch itself does and assert no view of the row" (`BindingGap::EffectUnsettled`).
/// Every synthesized scenario must still pass an honest target, synchronous or eventual.
#[test]
fn adv_two_bindings_on_the_created_row_suite_passes_honest_targets() {
    let text = with_relabel_bound_to("Created");
    let suite = synthesize(&model(&text)).suite;
    // What the relabel's own flow scenario expects of the row at the end.
    let flow = suite
        .scenarios
        .iter()
        .find(|(id, _)| id.to_string() == "relabels-on-created/binding/flow")
        .map(|(_, scenario)| {
            scenario
                .steps
                .iter()
                .map(|step| serde_json::to_value(step).unwrap())
                .filter(|step| step["step"] == "eventually_view")
                .map(|step| step["expectation"].to_string())
                .collect::<Vec<_>>()
        });
    eprintln!("relabels-on-created/binding/flow eventually_view: {flow:?}");
    let synchronous = run_against(&suite, &interpreted(&text));
    let eventual = run_against(&suite, &Deferred::new(&text));
    assert_eq!(
        (not_passed(&synchronous), not_passed(&eventual)),
        (BTreeMap::new(), BTreeMap::new())
    );
}

/// The relabel binding set off by `Started`: the row moves, then a second binding acts on it. A
/// two-link chain whose second link changes a field rather than the state.
#[test]
fn adv_chain_ending_in_a_field_change_suite_passes_honest_targets() {
    let text = with_relabel_bound_to("Started");
    let suite = synthesize(&model(&text)).suite;
    let synchronous = run_against(&suite, &interpreted(&text));
    let eventual = run_against(&suite, &Deferred::new(&text));
    assert_eq!(
        (not_passed(&synchronous), not_passed(&eventual)),
        (BTreeMap::new(), BTreeMap::new())
    );
}

/// `Kick` acts on one job and names another: `kicked-starts` addresses a row of the trigger's own
/// entity that is not the trigger's row (a `TriggerInput` destination beside an arranged subject).
#[test]
fn adv_trigger_on_one_job_naming_another_suite_passes_honest_targets() {
    let text = rewrite(
        FIXTURE,
        "  - name: jobs.job.Kick\n    input:\n      - {name: job_id, type: jobs.job.JobId}\n    outcomes:\n      - name: kicked\n        emits: [jobs.job.Kicked]\n        payload: {jobs.job.Kicked: {job_id: input.job_id}}",
        "  - name: jobs.job.Kick\n    input:\n      - {name: job_id, type: jobs.job.JobId}\n      - {name: other_id, type: jobs.job.JobId}\n    outcomes:\n      - name: kicked\n        updates: jobs.job.Job\n        instance: job_id\n        sets: {label: kicker}\n        emits: [jobs.job.Kicked]\n        payload: {jobs.job.Kicked: {job_id: input.other_id}}",
    );
    let synthesis = synthesize(&model(&text));
    assert!(
        synthesis
            .suite
            .scenarios
            .keys()
            .any(|id| id.to_string() == DROP),
        "drop is synthesized for a destination the trigger does not touch"
    );
    let honest = run_against(&synthesis.suite, &interpreted(&text));
    assert_eq!(not_passed(&honest), BTreeMap::new());
    let eventual = run_against(&synthesis.suite, &Deferred::new(&text));
    assert_eq!(not_passed(&eventual), BTreeMap::new());
    // The drop control still bites: the forced refusal ignored starts the named job.
    let swallowed = run_against(&synthesis.suite, &Faulty::new(&text, Fault::SwallowForce));
    assert!(not_passed(&swallowed).contains_key(DROP));
}

/// `Kick` updates the job it names and sets nothing on it: the destination is the trigger's own row
/// (a `SameRow` destination reached by a trigger that is not a creation). Every synthesized
/// scenario passes the honest synchronous target. (Measured: drop is refused here as
/// `UnchangedUnobservable`, "the trigger itself changes the row", because the effect is `updates`.)
#[test]
fn adv_same_row_updating_trigger_suite_passes_the_honest_target() {
    let text = rewrite(
        FIXTURE,
        "      - name: kicked\n        emits: [jobs.job.Kicked]",
        "      - name: kicked\n        updates: jobs.job.Job\n        instance: job_id\n        emits: [jobs.job.Kicked]",
    );
    let synthesis = synthesize(&model(&text));
    eprintln!(
        "same-row drop synthesized: {}; refusals: {:?}",
        synthesis
            .suite
            .scenarios
            .keys()
            .any(|id| id.to_string() == DROP),
        synthesis
            .refusals
            .iter()
            .filter(|refusal| refusal
                .scenario
                .as_ref()
                .is_some_and(|id| id.to_string() == DROP))
            .map(ToString::to_string)
            .collect::<Vec<_>>()
    );
    let honest = run_against(&synthesis.suite, &interpreted(&text));
    assert_eq!(not_passed(&honest), BTreeMap::new());
    let eventual = run_against(&synthesis.suite, &Deferred::new(&text));
    assert_eq!(not_passed(&eventual), BTreeMap::new());
}
