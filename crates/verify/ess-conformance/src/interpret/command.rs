//! Both request shapes share grants and the publication of their actual completed step.
use super::{execute::Step, response::Prepared, Interpreted};
use crate::scenario::{ActorRef, CommandRef};
use crate::target::{SemanticCommandResult, TargetError};
use ess_primitives::{consistency::ConsistencyToken, ids::CorrelationId};

impl Interpreted {
    pub(super) fn command_grant(
        &self,
        command: &CommandRef,
        actor: Option<&ActorRef>,
    ) -> Result<(), TargetError> {
        let model = self.model(format!("invoking `{command}`"))?;
        if let Some(actor) = actor {
            let granted = model.actors().get(actor.name()).is_some_and(|declared| {
                declared
                    .may
                    .iter()
                    .any(|grant| grant.name() == command.name())
            });
            if !granted {
                return Err(TargetError::not_granted(Some(actor.to_string())));
            }
        }
        Ok(())
    }

    pub(super) fn complete_command(
        &self,
        command: &CommandRef,
        correlation: &CorrelationId,
        step: Step,
        prepared: Option<Prepared>,
    ) -> Result<SemanticCommandResult, TargetError> {
        let observation = format!("invoking `{command}`");
        let mut scenario = self.scenario.borrow_mut();
        let response = prepared.and_then(|prepared| {
            scenario.issued = prepared.issued;
            prepared.value
        });
        if scenario.store != step.next {
            let visible_after = scenario.projection_reads.saturating_add(2);
            scenario
                .projection_versions
                .push((visible_after, step.next.clone()));
        }
        scenario.store = step.next;
        let mut direct_events = Vec::with_capacity(step.events.len());
        for event in step.events {
            let sequence = scenario.tick();
            let event = event.in_activity(correlation.clone()).at(sequence);
            scenario.published.push(event.clone());
            direct_events.push(event);
        }
        // Refusals also observe held state, so unchanged-state reads have a real token.
        let sequence = scenario.tick();
        let consistency = Some(
            ConsistencyToken::new(format!("seq:{sequence}"))
                .map_err(|error| TargetError::unavailable(observation, error.to_string()))?,
        );
        let result = SemanticCommandResult {
            outcome: step.outcome,
            error: step.error,
            consistency,
            direct_events,
            response,
        };
        drop(scenario);
        for event in &result.direct_events {
            self.dispatch(event, None)?;
        }
        Ok(result)
    }
}
