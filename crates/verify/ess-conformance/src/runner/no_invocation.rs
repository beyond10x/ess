//! The runner's half of zero-invocation observation (suite/36, ess/22, beyond10x/ess#268,
//! [`crate::no_invocation`]).
//!
//! [`ScenarioStep::ExpectNoInvocation`](crate::ScenarioStep::ExpectNoInvocation) asks the target
//! for the binding's invocations of its command under the scenario's correlation until the run's
//! deadline passes. Any invocation seen, at any ask, fails at once — a refused attempt and one
//! with the wrong input included, and one made visible only late. None seen passes only once the
//! deadline has passed. A target that cannot expose invocations leaves the check unsupported.

use super::{
    quote_input, target_failure, BindingRef, CheckCode, CheckResult, Clock, CommandRef,
    ConformanceTarget, Diagnostic, Flow, InvocationObservationRequest, Run, Runner,
};

impl<C: Clock> Runner<C> {
    /// Requires that `binding` invokes `command` no times under this scenario, through the
    /// deadline.
    pub(super) fn expect_no_invocation<T: ConformanceTarget>(
        &mut self,
        binding: &BindingRef,
        command: &CommandRef,
        obligation: Option<&str>,
        run: &mut Run,
        target: &T,
    ) -> Flow {
        if let Some(obligation) = obligation {
            return self.expect_unmet(binding, command, obligation, run, target);
        }
        let about = format!(
            "`{binding}` invokes `{command}` no times for the whole window, because its condition \
             does not hold"
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
            let made: Vec<_> = invocations
                .iter()
                .filter(|invocation| &invocation.command == command)
                .collect();
            if made.is_empty() {
                if deadline.has_passed(self.clock.now()) {
                    run.record(CheckResult::passed(CheckCode::Invocation, about));
                    return Flow::Continue;
                }
                continue;
            }
            let seen = made
                .iter()
                .map(|invocation| quote_input(&invocation.command.to_string(), &invocation.input))
                .collect::<Vec<_>>()
                .join("; ");
            run.record(CheckResult::failed(
                about,
                Diagnostic::new(CheckCode::Invocation, run.id.clone())
                    .declared_by(binding.clone())
                    .declared_by(command.clone())
                    .expected(format!(
                        "`{binding}` invokes `{command}` no times through the deadline"
                    ))
                    .observed(format!("at observation {asks}: [{seen}]")),
            ));
            return Flow::Continue;
        }
    }
}

impl<C: Clock> Runner<C> {
    /// Requires that `binding`, whose condition is Unknown on this scenario's occurrence, reports
    /// its unmet `obligation` rather than an answer, through the deadline (ess/22,
    /// beyond10x/ess#268): "Unknown cannot become a successful skip". A target that reports it —
    /// as the refusal of the observation — leaves the check unsupported, as every unmet obligation
    /// does; any invocation, and a silent answer through the whole window, fails.
    fn expect_unmet<T: ConformanceTarget>(
        &mut self,
        binding: &BindingRef,
        command: &CommandRef,
        obligation: &str,
        run: &mut Run,
        target: &T,
    ) -> Flow {
        let about = format!(
            "`{binding}` invokes `{command}` no times and owes its `{obligation}` obligation, because \
             its condition is Unknown for this occurrence"
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
                            .expected(format!("`{binding}` reports its `{obligation}` obligation"))
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
            let made: Vec<_> = invocations
                .iter()
                .filter(|invocation| &invocation.command == command)
                .collect();
            if made.is_empty() && !deadline.has_passed(self.clock.now()) {
                continue;
            }
            let observed = if made.is_empty() {
                format!("no invocation and no obligation through observation {asks}")
            } else {
                let seen = made
                    .iter()
                    .map(|invocation| {
                        quote_input(&invocation.command.to_string(), &invocation.input)
                    })
                    .collect::<Vec<_>>()
                    .join("; ");
                format!("at observation {asks}: [{seen}]")
            };
            run.record(CheckResult::failed(
                about,
                Diagnostic::new(CheckCode::Invocation, run.id.clone())
                    .declared_by(binding.clone())
                    .declared_by(command.clone())
                    .expected(format!(
                        "`{binding}` reports its `{obligation}` obligation and invokes nothing: \
                         Unknown is never a successful skip"
                    ))
                    .observed(observed),
            ));
            return Flow::Continue;
        }
    }
}
