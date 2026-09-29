//! The runner's half of an event delivered from an external channel with its delivery context
//! (suite/30, ess/18, beyond10x/ess#195, [`crate::delivery_context`]).
//!
//! [`ScenarioStep::DeliverEvent`](crate::ScenarioStep::DeliverEvent) hands one occurrence and its
//! context to the target; a target that cannot deliver one answers unsupported, the scenario is
//! recorded `unsupported` with the target's reason, and it stops there.
//!
//! [`ScenarioStep::ExpectEveryInvocation`](crate::ScenarioStep::ExpectEveryInvocation) is observed
//! for the step's whole eventual window (§15), asked of the target until the run's deadline passes:
//! an invocation for the occurrence that carries anything else fails at once, and the step passes
//! only when at least one invocation for the occurrence was seen and none disagreed by the
//! deadline. A redelivery made visible late is still read, which is the point: the redelivered
//! invocation is the one a target reading the wrong context gets wrong.

use std::collections::BTreeMap;

use ess_primitives::node::Node;

use super::{
    bounded_retry::expected, matches, quote, quote_input, target_failure, BindingRef, CheckCode,
    CheckResult, Clock, CommandRef, ConformanceTarget, Diagnostic, EventRef, Flow,
    InvocationObservationRequest, Run, Runner, ScenarioValue,
};
use crate::target::EventDeliveryRequest;

/// Delivers one occurrence of `event` from the channel `authority` with its context.
pub(super) fn deliver_event<T: ConformanceTarget>(
    event: &EventRef,
    authority: &str,
    payload: &BTreeMap<String, Node>,
    context: &BTreeMap<String, Node>,
    run: &mut Run,
    target: &T,
) -> Flow {
    let request = EventDeliveryRequest {
        event: event.clone(),
        authority: authority.to_owned(),
        payload: payload.clone(),
        context: context.clone(),
        correlation: run.context.correlation.clone(),
    };
    match target.deliver_event(request) {
        Ok(()) => Flow::Continue,
        Err(error) => {
            run.record(target_failure(
                &run.id,
                &format!(
                    "delivering `{event}` from the external channel `{authority}` with its context"
                ),
                &error,
            ));
            Flow::Stop
        }
    }
}

impl<C: Clock> Runner<C> {
    /// Requires at least one invocation of `command` by `binding` matching `selecting`, and that
    /// every such invocation carries `input`.
    pub(super) fn expect_every_invocation<T: ConformanceTarget>(
        &mut self,
        binding: &BindingRef,
        command: &CommandRef,
        selecting: &BTreeMap<String, ScenarioValue>,
        input: &BTreeMap<String, ScenarioValue>,
        run: &mut Run,
        target: &T,
    ) -> Flow {
        let Some((selected, _)) = expected(binding, selecting, run) else {
            return Flow::Stop;
        };
        let Some((wanted, absent)) = expected(binding, input, run) else {
            return Flow::Stop;
        };
        let about = format!(
            "every invocation `{binding}` makes of `{command}` for this occurrence carries what \
             its delivery carried"
        );
        let deadline = self.deadline();
        let mut asks = 0_u32;
        loop {
            asks += 1;
            let request = InvocationObservationRequest {
                binding: binding.clone(),
                command: command.clone(),
                correlation: run.context.correlation.clone(),
                deadline,
            };
            let invocations = match target.observe_invocations(request) {
                Ok(invocations) => invocations,
                Err(error) if error.is_unsupported() => {
                    run.record(CheckResult::unsupported(
                        about,
                        Diagnostic::new(CheckCode::Invocation, run.id.clone())
                            .declared_by(binding.clone())
                            .declared_by(command.clone())
                            .expected("the target exposes the commands its bindings invoke (§16)")
                            .observed(error.to_string()),
                    ));
                    return Flow::Continue;
                }
                Err(error) => {
                    run.record(target_failure(
                        &run.id,
                        &format!("observing what `{binding}` invoked"),
                        &error,
                    ));
                    return Flow::Stop;
                }
            };
            let chosen: Vec<_> = invocations
                .iter()
                .filter(|invocation| {
                    &invocation.command == command && matches(&invocation.input, &selected)
                })
                .collect();
            let disagreeing = chosen.iter().any(|invocation| {
                !matches(&invocation.input, &wanted)
                    || absent
                        .iter()
                        .any(|field| invocation.input.contains_key(field))
            });
            if !disagreeing && !deadline.has_passed(self.clock.now()) {
                continue;
            }
            if !disagreeing && !chosen.is_empty() {
                run.record(CheckResult::passed(CheckCode::Invocation, about));
                return Flow::Continue;
            }
            let mut diagnostic = Diagnostic::new(CheckCode::Invocation, run.id.clone())
                .declared_by(binding.clone())
                .declared_by(command.clone());
            for (field, value) in &wanted {
                diagnostic = diagnostic.expected(format!(
                    "every invocation for the occurrence has {command}.{field} = {}",
                    quote(value)
                ));
            }
            for field in &absent {
                diagnostic = diagnostic.expected(format!("{command}.{field} is absent"));
            }
            let seen = if chosen.is_empty() {
                format!("`{binding}` invoked `{command}` for this occurrence no times")
            } else {
                chosen
                    .iter()
                    .map(|invocation| {
                        quote_input(&invocation.command.to_string(), &invocation.input)
                    })
                    .collect::<Vec<_>>()
                    .join("; ")
            };
            run.record(CheckResult::failed(
                about,
                diagnostic.observed(format!("after {asks} observation(s): [{seen}]")),
            ));
            return Flow::Continue;
        }
    }
}
