//! The specification itself, selected as the implementation under test.
//!
//! `examples/billing` and `examples/oracle-fixture` each have a committed suite and a hand-written
//! target, and both of those targets decided their command behaviour by hand. An interpreter runs
//! the same suite against the document the suite was synthesized from, which makes a specification
//! falsifiable before anybody has implemented it — see
//! `docs/design/ess-model-driven-interpretation-design-v0.1.md`.
//!
//! # Two ways to hold one
//!
//! [`Interpreted::for_model`] holds a compiled model and executes its commands through
//! [`execute`]: which outcome the input selects, the transition it takes and from which states, the
//! `sets:` writes, the events it emits with their payload mappings, whether a `wrong_state:` branch
//! refuses or accepts, and what `invariants:` require of an instance at rest. What it does not
//! derive yet — views, bindings, time, redelivery, established entities — it refuses as
//! [`Status::Unsupported`](crate::report::Status::Unsupported), so a scenario that needs one reports
//! an unsatisfied obligation and never a failure the interpreter caused by guessing.
//!
//! [`Interpreted::new`] holds no model. It is the seam `--target interpreted` was introduced as, and
//! it still derives nothing: every scenario comes back `unsupported`, refused at
//! [`begin_scenario`](ConformanceTarget::begin_scenario) because with no model there is no state to
//! open.
//!
//! **`Unsupported` rather than `Error`, and that distinction is what this module claims wherever it
//! refuses.** `report.rs` writes it down: `Failed` says the implementation contradicted the
//! specification, `Error` says nobody found out. A construct this interpreter does not execute yet
//! is a capability it does not have, not an execution that went wrong. It is equally not a skip: §28
//! makes an unsupported scenario fail conformance, so a refusal here never turns a scenario green.
//!
//! # Nothing is chosen here
//!
//! [`execute`] returns every outcome the model allows. The target answers only when that is exactly
//! one; where the model leaves the outcome open it refuses rather than picking, and a refusal the
//! model does not declare is answered as
//! [`SemanticCommandResult::undeclared`], never as a declared branch. An `external:` branch is taken
//! only when a scenario forced it with
//! [`configure_external_outcome`](ConformanceTarget::configure_external_outcome), and it lapses after
//! the next invocation of that command, which is what the step says.
//!
//! # Where the identifiers come from
//!
//! A created instance's identity and every value the model leaves to the implementation are minted
//! from a counter the scenario's [`Store`] carries, reset by
//! [`begin_scenario`](ConformanceTarget::begin_scenario) — so two runs of one suite produce the
//! same ones, and the runner never compares one against an expected value.

pub mod execute;
mod protected;

use std::cell::RefCell;

use ess_compiler::ir::{EssIr, ResolvedCondition};
use ess_primitives::consistency::ConsistencyToken;

use crate::scenario::OutcomeRef;
use crate::target::{
    ConformanceTarget, EventObservationRequest, ExternalOutcomeControl, ImplementationIdentity,
    InvocationObservationRequest, ObservedEvent, ObservedInvocation, RedeliveryRequest,
    ScenarioContext, SemanticCommandRequest, SemanticCommandResult, SemanticViewRequest,
    SemanticViewResult, TargetError,
};
use execute::{Externals, Store, Undetermined};

/// How a report names this target, and the value `--target` selects it by.
///
/// One string for both, so a reader holding a report can get back to the command that produced it.
/// That is this target's choice and not a convention of the family: `billing` reports
/// `billing-reference` and `oracle-fixture` reports `oracle-reference`, because those two name a
/// hand-written implementation while this one names nothing but the selection itself.
const IDENTITY: &str = "interpreted";

/// Why every observation is refused by the target that holds no model, in one sentence.
const NOTHING_DERIVED: &str =
    "the interpreted target derives no behaviour yet: it executes no part of the model";

/// The model, run as the implementation under test.
///
/// See the [module documentation](self) for what it derives, what it refuses and why a refusal is
/// `unsupported` rather than `error`.
#[derive(Debug, Default)]
pub struct Interpreted {
    model: Option<EssIr>,
    scenario: RefCell<Scenario>,
}

/// One scenario's isolated execution context (§8).
#[derive(Debug, Default)]
struct Scenario {
    store: Store,
    published: Vec<ObservedEvent>,
    sequence: u64,
    forced: Option<OutcomeRef>,
    issued: protected::Issued,
}

impl Scenario {
    fn tick(&mut self) -> u64 {
        self.sequence += 1;
        self.sequence
    }
}

impl Interpreted {
    /// The target `--target interpreted` selects with no model in hand: it derives nothing.
    pub fn new() -> Self {
        Self::default()
    }

    /// The target that executes `model`'s commands.
    pub fn for_model(model: EssIr) -> Self {
        Self {
            model: Some(model),
            scenario: RefCell::default(),
        }
    }

    /// The model, or the refusal the seam has always answered with.
    fn model(&self, observation: impl Into<String>) -> Result<&EssIr, TargetError> {
        self.model
            .as_ref()
            .ok_or_else(|| TargetError::unsupported(observation, NOTHING_DERIVED))
    }
}

/// How the target reports what [`execute`] could not determine.
fn refusal(observation: String, why: &Undetermined) -> TargetError {
    if why.is_capability_gap() {
        TargetError::unsupported(observation, why.to_string())
    } else {
        TargetError::unavailable(observation, why.to_string())
    }
}

impl ConformanceTarget for Interpreted {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        // Answered rather than refused: §30 requires a report to name the implementation that
        // answered, whether or not it holds a model.
        Ok(ImplementationIdentity::new(
            IDENTITY,
            env!("CARGO_PKG_VERSION"),
        ))
    }

    fn begin_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.model(format!(
            "an isolated execution context for `{}`",
            scenario.scenario
        ))?;
        *self.scenario.borrow_mut() = Scenario::default();
        Ok(())
    }

    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        let observation = format!("invoking `{}`", request.command);
        let model = self.model(observation.clone())?;
        // The standard refusal for an actor no grant admits, before the command runs
        // (beyond10x/ess#265). A command sent as no actor is sent as the interpreter's own
        // authority: the suite sends a command no actor is granted that way.
        if let Some(actor) = &request.actor {
            let granted = model.actors().get(actor.name()).is_some_and(|declared| {
                declared
                    .may
                    .iter()
                    .any(|command| command.name() == request.command.name())
            });
            if !granted {
                return Err(TargetError::not_granted(Some(actor.to_string())));
            }
        }
        let mut scenario = self.scenario.borrow_mut();
        let externals = match scenario.forced.take() {
            Some(forced) if forced.command == request.command => {
                Externals::Forced(forced.outcome.clone())
            }
            other => {
                scenario.forced = other;
                Externals::Withheld
            }
        };
        let mut steps = execute::execute(
            model,
            &scenario.store,
            request.command.name(),
            &request.input,
            &externals,
        )
        .map_err(|why| refusal(observation.clone(), &why))?;
        if steps.len() != 1 {
            let open: Vec<String> = steps
                .iter()
                .map(|step| {
                    step.outcome
                        .as_ref()
                        .map_or_else(|| "no declared outcome".to_owned(), ToString::to_string)
                })
                .collect();
            return Err(TargetError::unsupported(
                observation,
                format!(
                    "the model leaves the outcome open between {} and the interpreter does not \
                     choose",
                    open.join(", ")
                ),
            ));
        }
        let step = steps.remove(0);
        let response = if crate::one_time_response::marked_model(model) {
            protected::response(
                model,
                &request.command,
                step.outcome.as_ref(),
                &mut scenario.issued,
            )?
        } else {
            None
        };
        scenario.store = step.next;
        let mut direct_events = Vec::with_capacity(step.events.len());
        for event in step.events {
            let sequence = scenario.tick();
            let event = event.in_activity(request.correlation.clone()).at(sequence);
            scenario.published.push(event.clone());
            direct_events.push(event);
        }
        let consistency = match (&step.outcome, &step.error) {
            (Some(_), None) => {
                let sequence = scenario.tick();
                Some(
                    ConsistencyToken::new(format!("seq:{sequence}")).map_err(|error| {
                        TargetError::unavailable(observation.clone(), error.to_string())
                    })?,
                )
            }
            _ => None,
        };
        Ok(SemanticCommandResult {
            outcome: step.outcome,
            error: step.error,
            consistency,
            direct_events,
            response,
        })
    }

    fn execute_command_without_input(
        &self,
        request: crate::target::AbsentInputRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        Err(TargetError::unsupported(
            format!("invoking `{}` with no input", request.command),
            NOTHING_DERIVED,
        ))
    }

    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        let observation = format!("reading `{}`", request.view);
        let model = self.model(observation.clone())?;
        if crate::one_time_response::marked_model(model) {
            return protected::view(model, &self.scenario.borrow().store, &request);
        }
        Err(TargetError::unsupported(
            observation,
            "views are not interpreted yet",
        ))
    }

    fn observe_events(
        &self,
        request: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        let observation = format!("the occurrences of `{}`", request.event);
        let model = self.model(observation.clone())?;
        // Every occurrence a command published directly is here. What a binding would publish is
        // not, because bindings are not interpreted yet — so where the model declares one, a
        // complete answer is not available and an incomplete one would read as "never happened".
        if !model.bindings().is_empty() {
            return Err(TargetError::unsupported(
                observation,
                "the model declares bindings, which are not interpreted yet, so the occurrences \
                 they would publish cannot be observed",
            ));
        }
        Ok(self
            .scenario
            .borrow()
            .published
            .iter()
            .filter(|event| event.event == request.event)
            .cloned()
            .collect())
    }

    fn configure_external_outcome(
        &self,
        request: ExternalOutcomeControl,
    ) -> Result<(), TargetError> {
        let observation = format!("forcing `{}`", request.force);
        let model = self.model(observation.clone())?;
        let external = model
            .commands()
            .get(request.force.command.name())
            .and_then(|command| {
                command
                    .outcomes
                    .iter()
                    .find(|outcome| outcome.name == request.force.outcome)
            })
            .is_some_and(|outcome| {
                matches!(
                    outcome.condition,
                    ResolvedCondition::External { .. } | ResolvedCondition::ExternalWhen { .. }
                )
            });
        if !external {
            return Err(TargetError::unavailable(
                observation,
                format!(
                    "`{}` is not an outcome the model declares external",
                    request.force
                ),
            ));
        }
        self.scenario.borrow_mut().forced = Some(request.force);
        Ok(())
    }

    fn redeliver_event(&self, request: RedeliveryRequest) -> Result<(), TargetError> {
        let observation = format!("delivering `{}` again", request.event);
        self.model(observation.clone())?;
        Err(TargetError::unsupported(
            observation,
            "bindings are not interpreted yet, so there is nothing to deliver to",
        ))
    }

    fn observe_invocations(
        &self,
        request: InvocationObservationRequest,
    ) -> Result<Vec<ObservedInvocation>, TargetError> {
        let observation = format!(
            "the invocations `{}` made of `{}`",
            request.binding, request.command
        );
        self.model(observation.clone())?;
        Err(TargetError::unsupported(
            observation,
            "bindings are not interpreted yet, so no binding has invoked anything",
        ))
    }

    fn end_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.model(format!(
            "closing the execution context of `{}`",
            scenario.scenario
        ))?;
        *self.scenario.borrow_mut() = Scenario::default();
        Ok(())
    }
}
