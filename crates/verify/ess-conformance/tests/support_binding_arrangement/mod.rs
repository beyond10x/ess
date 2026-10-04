//! The binding-arrangement fixture and its faulty dispatchers, shared by the native and the Go
//! runner controls (beyond10x/ess#266, beyond10x/ess#267,
//! `docs/design/binding-arrangement-and-drop.md`).
//!
//! The honest target is the interpreter, which runs bindings through its own dispatcher. Every
//! faulty target is either the interpreter of a rewritten model — a binding removed, a policy
//! changed, a creation landing elsewhere — or [`Faulty`], the same dispatcher with one thing wrong
//! at its edge.
#![allow(dead_code)]

use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;

use ess_compiler::{resolve::compile, source::SourceMap, EssIr};
use ess_conformance::interpret::Interpreted;
use ess_conformance::scenario::{BindingRef, CommandRef};
use ess_conformance::target::*;
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::ids::CorrelationId;
use ess_primitives::node::Node;

pub const FIXTURE: &str = include_str!("../fixtures/binding-arrangement.yaml");

pub const CREATED_STARTS: &str = "  - id: created-starts
    when: {event: jobs.job.Created}
    invoke: {command: jobs.job.Start}
    mapping: {job_id: event.job_id}
    delivery: at_least_once
    on_failure: drop
";

pub const KICKED_STARTS: &str = "  - id: kicked-starts
    when: {event: jobs.job.Kicked}
    invoke: {command: jobs.job.Start}
    mapping: {job_id: event.job_id}
    delivery: at_least_once
    on_failure: drop
";

/// The drop scenario every drop control reads.
pub const DROP: &str = "kicked-starts/binding/on-failure";

pub fn model(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).expect("the fixture parses");
    let spec = Specification::assemble([(Source::new("binding-arrangement.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("the fixture validates:\n{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|d| panic!("the fixture resolves:\n{d}"))
}

/// `text` with `from` replaced by `to`, which must occur exactly once.
pub fn rewrite(text: &str, from: &str, to: &str) -> String {
    assert_eq!(text.matches(from).count(), 1, "`{from}` occurs once");
    text.replacen(from, to, 1)
}

pub fn interpreted(text: &str) -> Interpreted {
    Interpreted::for_model(model(text))
}

/// The fixture with `created-starts` left out: a dispatcher that does not run that binding.
pub fn without_created_starts() -> String {
    rewrite(FIXTURE, CREATED_STARTS, "")
}

/// The fixture with `kicked-starts` left out: zero attempts.
pub fn without_kicked_starts() -> String {
    rewrite(FIXTURE, KICKED_STARTS, "")
}

/// The fixture whose `kicked-starts` retries what it should drop.
pub fn kicked_retries() -> String {
    rewrite(
        FIXTURE,
        KICKED_STARTS,
        &KICKED_STARTS.replace("on_failure: drop", "on_failure: retry"),
    )
}

/// The fixture with `Relabel`, an update writing the literal `relabelled`, and a sibling binding
/// `relabels-on-<event>` that relabels the job `event` names: a second binding the same trigger
/// sets off on the same row.
pub fn relabelled_on(event: &str) -> String {
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
        "{text}  - id: relabels-on-{lower}
    when: {{event: jobs.job.{event}}}
    invoke: {{command: jobs.job.Relabel}}
    mapping: {{job_id: event.job_id}}
    delivery: at_least_once
    on_failure: drop
",
        lower = event.to_lowercase()
    )
}

/// The fixture whose `creator` leaves its new job `Stopped`, where `Start` refuses.
pub fn lands_stopped(creator_event: &str) -> String {
    rewrite(
        FIXTURE,
        &format!(
            "        creates: jobs.job.Job\n        instance: job_id\n        sets: {{label: input.label}}\n        emits: [{creator_event}]"
        ),
        &format!(
            "        creates: jobs.job.Job\n        into: Stopped\n        instance: job_id\n        sets: {{label: input.label}}\n        emits: [{creator_event}]"
        ),
    )
}

// ---- a dispatcher with one thing wrong ---------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fault {
    /// The forced external refusal is accepted and ignored: the attempt succeeds.
    SwallowForce,
    /// After every `Kick`, a second attempt of `Start` is made with a different job.
    MalformedRetry,
    /// From the `n`th observation on, a second identical attempt is visible.
    LateRetry(u32),
    /// Each observation drains what it returned.
    Draining,
    /// A new deadline starts a new, empty history.
    DeadlineReset,
    /// The requested correlation is ignored.
    WrongCorrelation,
    /// Only attempts carrying the first attempt's input are returned.
    FilterExpected,
}

pub struct Faulty {
    inner: Interpreted,
    fault: Fault,
    extra: RefCell<Vec<ObservedInvocation>>,
    asks: Cell<u32>,
    returned: Cell<usize>,
    deadline: RefCell<Option<(Deadline, usize)>>,
    correlation: RefCell<Option<CorrelationId>>,
}

impl Faulty {
    pub fn new(text: &str, fault: Fault) -> Self {
        Self {
            inner: interpreted(text),
            fault,
            extra: RefCell::default(),
            asks: Cell::new(0),
            returned: Cell::new(0),
            deadline: RefCell::default(),
            correlation: RefCell::default(),
        }
    }
}

impl ConformanceTarget for Faulty {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        self.inner.identity()
    }
    fn begin_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.extra.borrow_mut().clear();
        self.asks.set(0);
        self.returned.set(0);
        *self.deadline.borrow_mut() = None;
        *self.correlation.borrow_mut() = Some(scenario.correlation.clone());
        self.inner.begin_scenario(scenario)
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        let kick = request.command.to_string() == "jobs.job.Kick";
        let correlation = request.correlation.clone();
        let result = self.inner.execute_command(request)?;
        if kick && self.fault == Fault::MalformedRetry {
            let input: BTreeMap<String, Node> = [(
                "job_id".to_owned(),
                Node::Text("00000000-0000-4000-8000-0000000000ff".into()),
            )]
            .into();
            self.extra.borrow_mut().push(ObservedInvocation {
                binding: BindingRef::new(
                    ess_domain::binding::BindingName::new("kicked-starts").unwrap(),
                ),
                command: CommandRef::new("jobs.job.Start".parse().unwrap()),
                input: input.clone(),
            });
            self.inner.execute_command(SemanticCommandRequest {
                command: CommandRef::new("jobs.job.Start".parse().unwrap()),
                actor: None,
                caller: None,
                input,
                correlation,
            })?;
        }
        Ok(result)
    }
    fn execute_command_without_input(
        &self,
        request: AbsentInputRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        self.inner.execute_command_without_input(request)
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
        request: ExternalOutcomeControl,
    ) -> Result<(), TargetError> {
        if self.fault == Fault::SwallowForce {
            return Ok(());
        }
        self.inner.configure_external_outcome(request)
    }
    fn configure_external_outcome_repeatedly(
        &self,
        request: ExternalOutcomeControl,
        times: std::num::NonZeroU32,
    ) -> Result<(), TargetError> {
        if self.fault == Fault::SwallowForce {
            return Ok(());
        }
        self.inner
            .configure_external_outcome_repeatedly(request, times)
    }
    fn redeliver_event(&self, request: RedeliveryRequest) -> Result<(), TargetError> {
        self.inner.redeliver_event(request)
    }
    fn observe_invocations(
        &self,
        mut request: InvocationObservationRequest,
    ) -> Result<Vec<ObservedInvocation>, TargetError> {
        self.asks.set(self.asks.get() + 1);
        if self.fault == Fault::WrongCorrelation {
            if let Some(own) = self.correlation.borrow().clone() {
                request.correlation = own;
            }
        }
        let deadline = request.deadline;
        let mut seen = self.inner.observe_invocations(request.clone())?;
        seen.extend(
            self.extra
                .borrow()
                .iter()
                .filter(|invocation| {
                    invocation.binding == request.binding && invocation.command == request.command
                })
                .cloned(),
        );
        match self.fault {
            Fault::LateRetry(after) if self.asks.get() > after && !seen.is_empty() => {
                let again = seen[0].clone();
                seen.push(again);
            }
            Fault::Draining => {
                let from = self.returned.get().min(seen.len());
                self.returned.set(seen.len());
                seen.drain(..from);
            }
            Fault::DeadlineReset => {
                let mut held = self.deadline.borrow_mut();
                let from = match *held {
                    Some((at, from)) if at == deadline => from,
                    _ => seen.len(),
                };
                if held.is_none() {
                    *held = Some((deadline, 0));
                    return Ok(seen);
                }
                *held = Some((deadline, from));
                seen.drain(..from.min(seen.len()));
            }
            Fault::FilterExpected => {
                if let Some(first) = seen.first().map(|invocation| invocation.input.clone()) {
                    seen.retain(|invocation| invocation.input == first);
                }
            }
            _ => {}
        }
        Ok(seen)
    }
    fn end_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.inner.end_scenario(scenario)
    }
}
