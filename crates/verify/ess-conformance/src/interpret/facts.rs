//! Explicit scenario delivery facts and a relative logical clock, never an absolute timestamp.
use super::Interpreted;
use crate::{
    scenario::{EventRef, InstantName},
    target::{
        ElapsedObservation, ElapsedObservationRequest, EventDeliveryRequest, InstantMark,
        ObservedEvent, RedeliveryRequest, TargetError,
    },
};
use ess_compiler::ir::{EssIr, ResolvedField};
use ess_primitives::{ids::CorrelationId, node::Node};
use std::collections::BTreeMap;

#[derive(Debug, Default)]
pub(super) struct Facts {
    correlation: Option<CorrelationId>,
    delivered: BTreeMap<EventRef, EventDeliveryRequest>,
    now_ms: u64,
    marks: BTreeMap<InstantName, Mark>,
}

#[derive(Debug, Clone, Copy)]
struct Mark {
    at_ms: u64,
    published: usize,
}

fn missing(reason: &str) -> TargetError {
    TargetError::unsupported("interpreted scenario facts", reason)
}

impl Facts {
    pub(super) fn open(&mut self, correlation: CorrelationId) {
        *self = Self {
            correlation: Some(correlation),
            ..Self::default()
        };
    }

    fn check(&self, correlation: &CorrelationId) -> Result<(), TargetError> {
        if self.correlation.as_ref() == Some(correlation) {
            Ok(())
        } else {
            Err(missing("the request has no matching active scenario"))
        }
    }
}

impl Interpreted {
    pub(super) fn facts_deliver(&self, request: &EventDeliveryRequest) -> Result<(), TargetError> {
        let model = self.model("external delivery")?;
        self.scenario.borrow().facts.check(&request.correlation)?;
        let event = model
            .events()
            .get(request.event.name())
            .ok_or_else(|| missing("the external event is not declared"))?;
        if model.commands().values().any(|command| {
            command.outcomes.iter().any(|outcome| {
                outcome
                    .emits
                    .iter()
                    .any(|emitted| emitted.name() == request.event.name())
            })
        }) {
            return Err(missing(
                "a model-published event is not an external delivery",
            ));
        }
        validate_fields(model, &event.fields, &request.payload)?;
        let mut matched = false;
        let mut context_fields = Vec::new();
        for binding in model.bindings().values().filter(|binding| {
            binding
                .cause
                .event()
                .is_some_and(|cause| cause.name() == request.event.name())
        }) {
            if let Some(channel) = &binding.context {
                if channel.authority.as_str() == request.authority {
                    context_fields.extend(channel.fields.iter().cloned());
                    matched = true;
                }
            }
        }
        if !matched {
            return Err(missing("the event has no matching context authority"));
        }
        validate_fields(model, &context_fields, &request.context)?;
        // Retain the supplied occurrence independently of model publication history. A retry of
        // a delivery that reached dispatch must retain its own context even if dispatch refuses.
        self.scenario
            .borrow_mut()
            .facts
            .delivered
            .insert(request.event.clone(), request.clone());
        self.dispatch_external(
            &external_event(request),
            &request.authority,
            &request.context,
        )
    }

    pub(super) fn facts_redeliver(&self, request: &RedeliveryRequest) -> Result<bool, TargetError> {
        self.model("external redelivery")?;
        let held = {
            let state = self.scenario.borrow();
            let Some(held) = state.facts.delivered.get(&request.event) else {
                return Ok(false);
            };
            state.facts.check(&request.correlation)?;
            held.clone()
        };
        self.dispatch_external(&external_event(&held), &held.authority, &held.context)?;
        Ok(true)
    }

    pub(super) fn facts_mark(&self, request: InstantMark) -> Result<(), TargetError> {
        self.model("relative time mark")?;
        let mut state = self.scenario.borrow_mut();
        state.facts.check(&request.correlation)?;
        if state.facts.marks.len() >= 65_536 && !state.facts.marks.contains_key(&request.instant) {
            return Err(missing("the scenario instant budget is exhausted"));
        }
        let mark = Mark {
            at_ms: state.facts.now_ms,
            published: state.published.len(),
        };
        state.facts.marks.insert(request.instant, mark);
        Ok(())
    }

    pub(super) fn facts_elapsed(
        &self,
        request: &ElapsedObservationRequest,
    ) -> Result<ElapsedObservation, TargetError> {
        let model = self.model("relative elapsed observation")?;
        if model
            .bindings()
            .values()
            .any(|binding| binding.cause.periodic().is_some())
        {
            return Err(missing(
                "elapsed observation requires the model's periodic host facts",
            ));
        }
        if request
            .watching
            .as_ref()
            .is_some_and(|event| !model.events().contains_key(event.name()))
        {
            return Err(missing("the watched event is not declared"));
        }
        let mut state = self.scenario.borrow_mut();
        state.facts.check(&request.correlation)?;
        let mark = state
            .facts
            .marks
            .get(&request.instant)
            .copied()
            .ok_or_else(|| missing("the scenario did not mark this instant"))?;
        let until = mark
            .at_ms
            .checked_add(request.hold.millis())
            .ok_or_else(|| missing("the relative elapsed range is exhausted"))?;
        state.facts.now_ms = state.facts.now_ms.max(until);
        // Every model publication is synchronous at this logical instant. The cursor separates
        // events before/after a mark even when both occur at the same millisecond.
        let published = request.watching.as_ref().map_or(0, |watched| {
            state.published[mark.published..]
                .iter()
                .filter(|event| {
                    &event.event == watched
                        && event.correlation.as_ref() == Some(&request.correlation)
                })
                .count()
        });
        Ok(ElapsedObservation {
            elapsed_ms: state.facts.now_ms - mark.at_ms,
            published,
        })
    }
}

fn external_event(request: &EventDeliveryRequest) -> ObservedEvent {
    let mut event =
        ObservedEvent::new(request.event.clone()).in_activity(request.correlation.clone());
    event.payload.clone_from(&request.payload);
    event
}

fn validate_fields(
    ir: &EssIr,
    fields: &[ResolvedField],
    supplied: &BTreeMap<String, Node>,
) -> Result<(), TargetError> {
    if supplied
        .keys()
        .any(|key| !fields.iter().any(|field| &field.name == key))
    {
        return Err(missing("the supplied facts contain an undeclared field"));
    }
    for field in fields {
        match supplied.get(&field.name) {
            Some(value) => crate::input::validate_typed_value(ir, &field.type_ref, value)
                .map_err(|_| missing("a supplied fact does not satisfy its declared type"))?,
            None if field.type_ref.is_optional() => {}
            None => return Err(missing("a required fact was not supplied")),
        }
    }
    Ok(())
}
