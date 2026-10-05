//! Model-owned binding dispatch. Undetermined commands never become delivery failures.
//!
//! A refusal-selected failure policy (ess/22, beyond10x/ess#269) is answered after every attempt
//! by the attempt's actual answer: its declared refusal's policy, or the explicit fallback for an
//! answer that carries no declared outcome. An attempt is recorded once, at the dispatcher-to-port
//! boundary with a complete admitted input, before the port answers; a bounded retry compares the
//! occurrence's total attempts with its bound, whichever refusals answered them. A failure before
//! the port has a complete input is an unmet binding-input obligation: no attempt, no policy.
use super::{Interpreted, PortAnswer};
use crate::{
    accessor::Expected,
    scenario::{BindingRef, CommandRef, OutcomeRef},
    target::{
        ConformanceTarget, ObservedEvent, ObservedInvocation, SemanticCommandRequest, TargetError,
    },
};
use ess_compiler::ir::{
    EventHandle, ResolvedBinding, ResolvedMappingValue, ResolvedRefusalAction,
    ResolvedRefusalPolicy,
};
use ess_domain::binding::Failure;
use ess_domain::command::OutcomeName;
use ess_primitives::ids::CorrelationId;
use ess_primitives::node::Node;
use ess_primitives::predicate::Truth;
use std::collections::BTreeMap;
fn unsupported(reason: &str) -> TargetError {
    TargetError::unsupported("interpreted binding", reason)
}
/// The most attempts one occurrence may make before an unbounded retry is reported as exceeding
/// the finite execution budget.
const ATTEMPT_BUDGET: u32 = 65_536;
/// What the command port answered one attempt with.
enum Attempt {
    /// An accepting outcome: the binding is done.
    Accepted,
    /// A refusal, with its declared outcome, or a failure that carries none.
    Refused(Option<OutcomeName>),
}
#[derive(Debug)]
pub(super) struct Delivery {
    event: ObservedEvent,
    external: Option<(String, BTreeMap<String, Node>)>,
}
impl Interpreted {
    pub(super) fn dispatch(
        &self,
        event: &ObservedEvent,
        _context: Option<&BTreeMap<String, Node>>,
    ) -> Result<(), TargetError> {
        self.enqueue(Delivery {
            event: event.clone(),
            external: None,
        })
    }
    pub(super) fn dispatch_external(
        &self,
        event: &ObservedEvent,
        authority: &str,
        context: &BTreeMap<String, Node>,
    ) -> Result<(), TargetError> {
        self.enqueue(Delivery {
            event: event.clone(),
            external: Some((authority.into(), context.clone())),
        })
    }
    fn enqueue(&self, delivery: Delivery) -> Result<(), TargetError> {
        {
            let mut state = self.scenario.borrow_mut();
            if state.pending_bindings.len() >= 65_536 {
                return Err(unsupported("binding queue budget exhausted"));
            }
            state.pending_bindings.push_back(delivery);
            if state.dispatching {
                return Ok(());
            }
            state.dispatching = true;
        }
        // An unknown condition is that binding's unmet obligation: its siblings and every queued
        // delivery still run, and the first such obligation is reported once they have.
        let mut unmet = None;
        let result = (|| {
            loop {
                let next = self.scenario.borrow_mut().pending_bindings.pop_front();
                let Some(next) = next else {
                    break;
                };
                self.dispatch_one(&next, &mut unmet)?;
            }
            unmet.map_or(Ok(()), Err)
        })();
        let mut state = self.scenario.borrow_mut();
        state.dispatching = false;
        if result.is_err() {
            state.pending_bindings.clear();
        }
        result
    }
    fn dispatch_one(
        &self,
        delivery: &Delivery,
        unmet: &mut Option<TargetError>,
    ) -> Result<(), TargetError> {
        let event = &delivery.event;
        let model = self.model("binding dispatch")?;
        for binding in model.bindings().values().filter(|binding| {
            binding
                .cause
                .event()
                .is_some_and(|cause| cause.name() == event.event.name())
        }) {
            let context = match (&binding.context, &delivery.external) {
                (Some(required), Some((authority, context)))
                    if required.authority.as_str() == authority =>
                {
                    Some(context)
                }
                (Some(_), _) => continue,
                (None, _) => None,
            };
            match holds(binding, &event.payload) {
                Ok(true) => {}
                Ok(false) => continue,
                Err(obligation) => {
                    unmet.get_or_insert(obligation);
                    continue;
                }
            }
            let input = match mapped(model, binding, &event.payload, context) {
                Ok(input) => input,
                // A refusal-selected policy runs only once a valid input reaches the command port:
                // a mapping, conversion or selection failure before it is the binding's unmet
                // input obligation, with zero attempts and no policy, retry or escalation. Its
                // siblings and every queued delivery still run, as for an unknown condition.
                Err(why) if binding.refusal_policy.is_some() => {
                    unmet.get_or_insert(unsupported(&format!(
                        "binding `{}` has an unmet binding-input obligation and never reached its \
                         command port: {why}",
                        binding.name
                    )));
                    continue;
                }
                Err(error) => return Err(error),
            };
            let correlation = event
                .correlation
                .clone()
                .ok_or_else(|| unsupported("event lacks scenario correlation"))?;
            if let Some(policy) = &binding.refusal_policy {
                self.selected(binding, policy, &input, event, &correlation, unmet)?;
                continue;
            }
            let attempts = binding
                .retry
                .as_ref()
                .map_or(ATTEMPT_BUDGET, |bound| bound.attempts);
            let mut completed = false;
            for _ in 0..attempts {
                self.record_attempt(binding, &input, &correlation)?;
                let outcome = match self.attempt(binding, &input, &correlation)? {
                    Attempt::Accepted => {
                        completed = true;
                        break;
                    }
                    Attempt::Refused(outcome) => outcome,
                };
                if binding.retry.as_ref().is_some_and(|bound| {
                    outcome
                        .as_ref()
                        .is_some_and(|outcome| bound.is_final(outcome))
                }) {
                    completed = true;
                    break;
                }
                match binding.failure {
                    Failure::Retry => {}
                    Failure::Drop => {
                        completed = true;
                        break;
                    }
                    Failure::Escalate => {
                        let handle = binding
                            .escalation
                            .as_ref()
                            .ok_or_else(|| unsupported("escalation event missing"))?;
                        self.escalate(handle, &input, event)?;
                        completed = true;
                        break;
                    }
                }
            }
            if !completed && binding.retry.is_none() {
                return Err(unsupported(
                    "unbounded retry exceeded the finite execution budget",
                ));
            }
        }
        Ok(())
    }

    /// One occurrence under a refusal-selected policy: attempt, and answer each failed attempt with
    /// the policy its actual answer selects, until an accepting outcome, a drop, an escalation, a
    /// final refusal or the bound ends it. The total attempts are the bound's count whichever
    /// refusals answered them.
    fn selected(
        &self,
        binding: &ResolvedBinding,
        policy: &ResolvedRefusalPolicy,
        input: &BTreeMap<String, Node>,
        event: &ObservedEvent,
        correlation: &CorrelationId,
        unmet: &mut Option<TargetError>,
    ) -> Result<(), TargetError> {
        let mut attempts = 0_u32;
        loop {
            if attempts >= ATTEMPT_BUDGET {
                return Err(unsupported(
                    "unbounded retry exceeded the finite execution budget",
                ));
            }
            attempts += 1;
            self.record_attempt(binding, input, correlation)?;
            let outcome = match self.attempt(binding, input, correlation)? {
                Attempt::Accepted => return Ok(()),
                Attempt::Refused(outcome) => outcome,
            };
            match policy.select(outcome.as_ref()) {
                ResolvedRefusalAction::Drop => return Ok(()),
                ResolvedRefusalAction::Escalate { emits } => {
                    // The typed builder from the attempt's actual complete input. A builder that
                    // cannot construct the event publishes nothing and does not retry: it is the
                    // binding's reported obligation.
                    if let Err(obligation) = self.escalate(emits, input, event) {
                        unmet.get_or_insert(obligation);
                    }
                    return Ok(());
                }
                ResolvedRefusalAction::Retry { bound: None } => {}
                ResolvedRefusalAction::Retry { bound: Some(bound) } => {
                    let last = outcome
                        .as_ref()
                        .is_some_and(|outcome| bound.is_final(outcome));
                    if last || attempts >= bound.attempts {
                        return Ok(());
                    }
                }
            }
        }
    }

    /// Records one attempt at the dispatcher-to-command boundary, before the command answers: a
    /// refused attempt, and one the port fails without a declared outcome, is still an attempt
    /// (binding-arrangement-and-drop.md).
    fn record_attempt(
        &self,
        binding: &ResolvedBinding,
        input: &BTreeMap<String, Node>,
        correlation: &CorrelationId,
    ) -> Result<(), TargetError> {
        let mut state = self.scenario.borrow_mut();
        if state.invocations.len() >= 65_536 {
            return Err(unsupported("binding invocation budget exhausted"));
        }
        state.invocations.push((
            correlation.clone(),
            ObservedInvocation {
                binding: BindingRef::new(binding.name.clone()),
                command: CommandRef::new(binding.command.name().clone()),
                input: input.clone(),
            },
        ));
        Ok(())
    }

    /// The command port's answer to one attempt: a scripted answer where the scenario scripted the
    /// port ([`Interpreted::script_binding_port`]), the model's otherwise.
    fn attempt(
        &self,
        binding: &ResolvedBinding,
        input: &BTreeMap<String, Node>,
        correlation: &CorrelationId,
    ) -> Result<Attempt, TargetError> {
        let command = CommandRef::new(binding.command.name().clone());
        let scripted = {
            let mut state = self.scenario.borrow_mut();
            match &mut state.port_script {
                Some((scripted, answers)) if *scripted == command => answers.pop_front(),
                _ => None,
            }
        };
        match scripted {
            Some(PortAnswer::Untyped) => return Ok(Attempt::Refused(None)),
            Some(PortAnswer::Outcome(outcome)) => {
                self.scenario.borrow_mut().forced = Some((
                    OutcomeRef::new(command.clone(), outcome),
                    std::num::NonZeroU32::MIN,
                ));
            }
            None => {}
        }
        let result = self.execute_command(SemanticCommandRequest {
            command,
            actor: None,
            caller: None,
            input: input.clone(),
            correlation: correlation.clone(),
        })?;
        let Some(outcome) = result.outcome else {
            return Err(unsupported(
                "binding command has an unmet outcome obligation",
            ));
        };
        Ok(if result.error.is_none() {
            Attempt::Accepted
        } else {
            Attempt::Refused(Some(outcome.outcome))
        })
    }
}
fn mapped(
    ir: &ess_compiler::EssIr,
    binding: &ResolvedBinding,
    payload: &BTreeMap<String, Node>,
    context: Option<&BTreeMap<String, Node>>,
) -> Result<BTreeMap<String, Node>, TargetError> {
    let mut input = BTreeMap::new();
    for field in &binding.mapping {
        let value = match &field.value {
            ResolvedMappingValue::EventField { field, .. } => payload.get(field).cloned(),
            // Typed against the input it fills: `true` over a `Boolean`, never the text `"true"`
            // (beyond10x/ess#445).
            ResolvedMappingValue::Literal { value } => Some(
                crate::input::mapping_literal(ir, &field.target_type, value)
                    .ok_or_else(|| unsupported("binding constant is not a value of its input"))?,
            ),
            ResolvedMappingValue::DeliveryContext { field, .. } => {
                context.and_then(|values| values.get(field)).cloned()
            }
            ResolvedMappingValue::EventAccessor { plan, types, .. } => expected(
                crate::accessor::Observation::of(ir, plan, types, &observed_as(binding, field))
                    .and_then(|observation| observation.evaluate(payload)),
            )?,
            ResolvedMappingValue::Selection {
                selector,
                projection,
                ..
            } => expected(
                crate::selection::Observation::of(
                    ir,
                    binding,
                    *selector,
                    projection,
                    &field.target_type,
                )
                .and_then(|observation| observation.evaluate(payload)),
            )?,
            ResolvedMappingValue::HostContext { .. } | ResolvedMappingValue::HostRead { .. } => {
                return Err(unsupported("periodic host inputs have not been supplied"))
            }
        };
        if let Some(value) = value {
            crate::input::validate_typed_value(ir, &field.target_type, &value)
                .map_err(|_| unsupported("binding input does not satisfy its declared type"))?;
            input.insert(field.target.clone(), value);
        } else if !field.target_type.is_optional() {
            return Err(unsupported("required binding input was not supplied"));
        }
    }
    Ok(input)
}
fn expected(value: Result<Expected, String>) -> Result<Option<Node>, TargetError> {
    match value.map_err(|_| unsupported("binding projection could not be evaluated"))? {
        Expected::Absent => Ok(None),
        Expected::Present(value) => Ok(Some(value)),
    }
}

impl Interpreted {
    fn escalate(
        &self,
        handle: &EventHandle,
        input: &BTreeMap<String, Node>,
        event: &ObservedEvent,
    ) -> Result<(), TargetError> {
        let model = self.model("binding escalation")?;
        let spec = model.event(handle);
        let mut payload =
            crate::witness::fields(model, &spec.fields, crate::witness::Distinction::PLAIN)
                .map_err(|_| unsupported("escalation fields have no typed witness"))?;
        for field in &spec.fields {
            if let Some(value) = input.get(&field.name) {
                crate::input::validate_typed_value(model, &field.type_ref, value)
                    .map_err(|_| unsupported("escalation field type mismatch"))?;
                payload.insert(field.name.clone(), value.clone());
            }
        }
        let mut escalation = ObservedEvent::new(handle.into());
        escalation.payload = payload;
        escalation.correlation.clone_from(&event.correlation);
        {
            let mut state = self.scenario.borrow_mut();
            escalation.sequence = Some(state.tick());
            state.published.push(escalation.clone());
        }
        self.dispatch(&escalation, None)
    }
}

/// Whether `binding` runs for this occurrence: its condition, before selection, conversion,
/// mapping and invocation (ess/22, beyond10x/ess#268). False skips this binding alone; Unknown
/// invokes nothing and is an unmet obligation rather than a skip.
fn holds(binding: &ResolvedBinding, payload: &BTreeMap<String, Node>) -> Result<bool, TargetError> {
    let Some(condition) = &binding.condition else {
        return Ok(true);
    };
    match condition.plan.evaluate(payload) {
        Ok(Truth::True) => Ok(true),
        Ok(Truth::False) => Ok(false),
        Ok(Truth::Unknown) => Err(unsupported(
            "the binding condition is unknown for this occurrence, an unmet obligation that \
             invokes nothing",
        )),
        Err(_) => Err(unsupported(
            "the binding condition could not read the payload as declared",
        )),
    }
}

/// The type an accessor mapping is observed as: its target, or — where the binding's condition
/// proved the Optional source present for a required input (beyond10x/ess#194) — the Optional of
/// it, so an absent value is read as absent and refused below as a missing required input rather
/// than silently unwrapped.
fn observed_as(
    binding: &ResolvedBinding,
    field: &ess_compiler::ir::ResolvedMapping,
) -> ess_compiler::ir::ResolvedTypeRef {
    if binding.condition.is_some() && !field.target_type.is_optional() {
        ess_compiler::ir::ResolvedTypeRef::Optional {
            of: Box::new(field.target_type.clone()),
        }
    } else {
        field.target_type.clone()
    }
}
